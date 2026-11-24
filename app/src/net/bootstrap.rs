//! Bootstrap order (ADR 0009, 0023): remembered peers, then peer exchange with connected peers,
//! then trusted peers as a last resort.
//!
//! A trusted peer is never contacted while an earlier step works. When the trusted step runs
//! and fails, the retry waits `config::trusted_backoff` (30 s doubling to 15 min, jitter applied
//! before the cap). After a trusted peer answers, one exchange through it fills the table, so
//! trusted peers are where a new install learns peers, not where it keeps them.

use super::exchange::Entry;
use super::known_peers::KnownPeers;
use super::trusted::TrustedPeers;
use crate::config::trusted_backoff;
use crate::dungeons::TopicId;
use crate::engine::NetError;
use crate::identity::PeerId;
use std::sync::Mutex;
use std::time::Instant;

/// Most pending (exchange-learned) peers dialed in one bootstrap run.
pub const MAX_EXCHANGE_DIALS: usize = 10;
/// Peers asked for more peers per topic set, in one run.
pub const MAX_EXCHANGE_ASKS: usize = 3;

/// What bootstrap needs from the network: dial a peer (connect and say hello), list the
/// connected peers, and ask one for more peers.
#[allow(async_fn_in_trait)]
pub trait Dialer {
    async fn dial(&mut self, peer: PeerId) -> Result<(), NetError>;
    /// Begin dialing `peer` in the background without waiting, so several dials can run at
    /// once and a dial survives the caller being dropped. `dial` then picks up the result.
    /// The default does nothing (a dialer that connects instantly needs no head start).
    fn start_dial(&mut self, _peer: PeerId) {}
    fn connected(&self) -> Vec<PeerId>;
    async fn request_peers(
        &mut self,
        via: PeerId,
        topics: &[TopicId],
    ) -> Result<Vec<Entry>, NetError>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BootstrapState {
    /// Failed trusted rounds since the last success.
    pub trusted_attempt: u32,
    /// The trusted step does not run before this.
    pub trusted_not_before: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Dialed a remembered peer.
    Remembered(PeerId),
    /// Dialed a peer learned by peer exchange.
    Exchange(PeerId),
    /// Reached a trusted peer: every other step failed.
    Trusted(PeerId),
    /// Nothing worked. `trusted_retry_at` says when the trusted step may run again (`None`
    /// when there are no trusted peers).
    Failed { trusted_retry_at: Option<Instant> },
}

/// Time and randomness, passed in so tests are exact.
#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub now: Instant,
    pub unix: u64,
    /// A random number in `[0, 1)` for the backoff jitter.
    pub unit: f64,
}

fn store_err(e: crate::store::StoreError) -> NetError {
    NetError::Io(e.to_string())
}

/// Ask connected peers for more peers on `topics`, record the answers as pending claims, then
/// dial the pending ones. Returns the first peer that answered the dial.
async fn exchange_step<D: Dialer>(
    d: &mut D,
    known: &Mutex<KnownPeers>,
    me: PeerId,
    topics: &[TopicId],
    clk: Clock,
) -> Option<PeerId> {
    for via in d.connected().into_iter().take(MAX_EXCHANGE_ASKS) {
        if let Ok(entries) = d.request_peers(via, topics).await {
            let mut k = known.lock().unwrap();
            for e in entries.into_iter().filter(|e| e.peer != me) {
                let _ = k.observe(e.peer, &e.topics, clk.unix);
            }
            let _ = k.touch_used(&via, clk.unix);
        }
    }
    let pending: Vec<PeerId> = known
        .lock()
        .unwrap()
        .pending_for(topics)
        .into_iter()
        .take(MAX_EXCHANGE_DIALS)
        .collect();
    for peer in pending {
        if d.dial(peer).await.is_ok() {
            let _ = known.lock().unwrap().mark_verified(peer, topics, clk.unix);
            return Some(peer);
        }
    }
    None
}

/// One bootstrap run for `topics`.
pub async fn run<D: Dialer>(
    d: &mut D,
    known: &Mutex<KnownPeers>,
    trusted: &TrustedPeers,
    me: PeerId,
    topics: &[TopicId],
    state: &mut BootstrapState,
    clk: Clock,
) -> Outcome {
    // 1. Remembered peers.
    let remembered = known.lock().unwrap().remembered(topics);
    for peer in remembered {
        if d.dial(peer).await.is_ok() {
            let _ = known.lock().unwrap().touch_used(&peer, clk.unix);
            state.trusted_attempt = 0;
            state.trusted_not_before = None;
            return Outcome::Remembered(peer);
        }
    }
    // 2. Peer exchange with whoever we are connected to.
    if let Some(peer) = exchange_step(d, known, me, topics, clk).await {
        state.trusted_attempt = 0;
        state.trusted_not_before = None;
        return Outcome::Exchange(peer);
    }
    // 3. Trusted peers, only now, and not before the backoff is over.
    if trusted.is_empty() {
        return Outcome::Failed {
            trusted_retry_at: None,
        };
    }
    if let Some(at) = state.trusted_not_before
        && clk.now < at
    {
        return Outcome::Failed {
            trusted_retry_at: Some(at),
        };
    }
    // Start every trusted dial first: a dead entry must not make the live ones wait for its
    // timeout, and this whole function is dropped whenever the caller's loop wakes.
    for &peer in trusted.peers() {
        if peer != me {
            d.start_dial(peer);
        }
    }
    for &peer in trusted.peers() {
        if peer != me && d.dial(peer).await.is_ok() {
            let _ = known
                .lock()
                .unwrap()
                .mark_verified(peer, topics, clk.unix)
                .map_err(store_err);
            state.trusted_attempt = 0;
            state.trusted_not_before = None;
            // Learn through the exchange, so the table does not depend on the trusted list.
            let _ = exchange_step(d, known, me, topics, clk).await;
            return Outcome::Trusted(peer);
        }
    }
    let at = clk.now + trusted_backoff(state.trusted_attempt, clk.unit);
    state.trusted_attempt = state.trusted_attempt.saturating_add(1);
    state.trusted_not_before = Some(at);
    Outcome::Failed {
        trusted_retry_at: Some(at),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TRUSTED_BACKOFF_CAP, TRUSTED_BACKOFF_START};
    use crate::store::MemStore;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use std::time::Duration;

    fn p(n: u8) -> PeerId {
        PeerId([n; 32])
    }
    fn t(n: u8) -> TopicId {
        TopicId([n; 32])
    }

    /// Records every call, in order. `up` are the peers that answer a dial.
    #[derive(Default)]
    struct Mock {
        up: HashSet<PeerId>,
        connected: Vec<PeerId>,
        answers: HashMap<PeerId, Vec<Entry>>,
        log: Vec<String>,
    }

    impl Dialer for Mock {
        async fn dial(&mut self, peer: PeerId) -> Result<(), NetError> {
            self.log.push(format!("dial {}", peer.0[0]));
            if self.up.contains(&peer) {
                if !self.connected.contains(&peer) {
                    self.connected.push(peer);
                }
                Ok(())
            } else {
                Err(NetError::UnknownPeer)
            }
        }
        fn connected(&self) -> Vec<PeerId> {
            self.connected.clone()
        }
        async fn request_peers(
            &mut self,
            via: PeerId,
            _: &[TopicId],
        ) -> Result<Vec<Entry>, NetError> {
            self.log.push(format!("ask {}", via.0[0]));
            self.answers.get(&via).cloned().ok_or(NetError::Closed)
        }
    }

    fn setup() -> (Mutex<KnownPeers>, Instant) {
        (
            Mutex::new(KnownPeers::new(100, Arc::new(MemStore::new()))),
            Instant::now(),
        )
    }

    fn clk(now: Instant, unit: f64) -> Clock {
        Clock {
            now,
            unix: 1000,
            unit,
        }
    }

    fn entry(n: u8) -> Entry {
        Entry {
            peer: p(n),
            topics: vec![t(1)],
            last_seen: 1,
        }
    }

    #[tokio::test]
    async fn order_is_remembered_then_exchange_then_trusted() {
        let (known, now) = setup();
        known
            .lock()
            .unwrap()
            .mark_verified(p(1), &[t(1)], 10)
            .unwrap();
        let trusted = TrustedPeers::new(vec![p(100)]);
        let mut st = BootstrapState::default();
        let mut d = Mock {
            connected: vec![p(50)],
            answers: HashMap::from([(p(50), vec![entry(60)])]),
            ..Mock::default()
        };
        d.up.extend([p(60), p(100)]);

        // Remembered peer 1 is down, exchange via 50 yields 60 which is up: no trusted dial.
        let out = run(
            &mut d,
            &known,
            &trusted,
            p(0),
            &[t(1)],
            &mut st,
            clk(now, 0.5),
        )
        .await;
        assert_eq!(out, Outcome::Exchange(p(60)));
        assert_eq!(d.log, vec!["dial 1", "ask 50", "dial 60"]);
        assert!(known.lock().unwrap().is_verified(&p(60)));

        // With nothing else working, the trusted peer is the last resort.
        let mut d = Mock::default();
        d.up.insert(p(100));
        let (known2, _) = setup();
        known2
            .lock()
            .unwrap()
            .mark_verified(p(1), &[t(1)], 10)
            .unwrap();
        let out = run(
            &mut d,
            &known2,
            &trusted,
            p(0),
            &[t(1)],
            &mut BootstrapState::default(),
            clk(now, 0.5),
        )
        .await;
        assert_eq!(out, Outcome::Trusted(p(100)));
        assert_eq!(d.log, vec!["dial 1", "dial 100", "ask 100"]);
    }

    #[tokio::test]
    async fn trusted_peers_are_never_contacted_while_remembered_peers_work() {
        let (known, now) = setup();
        known
            .lock()
            .unwrap()
            .mark_verified(p(1), &[t(1)], 10)
            .unwrap();
        let trusted = TrustedPeers::new(vec![p(100)]);
        let mut d = Mock::default();
        d.up.extend([p(1), p(100)]);
        let out = run(
            &mut d,
            &known,
            &trusted,
            p(0),
            &[t(1)],
            &mut BootstrapState::default(),
            clk(now, 0.5),
        )
        .await;
        assert_eq!(out, Outcome::Remembered(p(1)));
        assert_eq!(d.log, vec!["dial 1"]);
    }

    #[tokio::test]
    async fn unverified_exchange_entries_are_dialed_before_they_count() {
        let (known, now) = setup();
        let trusted = TrustedPeers::new(vec![]);
        let mut d = Mock {
            connected: vec![p(50)],
            answers: HashMap::from([(p(50), vec![entry(60), entry(61)])]),
            ..Mock::default()
        };
        // Neither claimed peer answers.
        let out = run(
            &mut d,
            &known,
            &trusted,
            p(0),
            &[t(1)],
            &mut BootstrapState::default(),
            clk(now, 0.5),
        )
        .await;
        assert_eq!(
            out,
            Outcome::Failed {
                trusted_retry_at: None
            }
        );
        assert_eq!(known.lock().unwrap().len(), 0, "claims alone do not count");
        assert_eq!(known.lock().unwrap().pending_len(), 2);
    }

    #[tokio::test]
    async fn own_id_in_an_exchange_answer_is_ignored() {
        let (known, now) = setup();
        let mut d = Mock {
            connected: vec![p(50)],
            answers: HashMap::from([(p(50), vec![entry(0)])]),
            ..Mock::default()
        };
        run(
            &mut d,
            &known,
            &TrustedPeers::default(),
            p(0),
            &[t(1)],
            &mut BootstrapState::default(),
            clk(now, 0.5),
        )
        .await;
        assert_eq!(known.lock().unwrap().pending_len(), 0);
        assert_eq!(d.log, vec!["ask 50"]);
    }

    #[tokio::test]
    async fn trusted_retries_back_off_with_jitter_before_the_cap() {
        let (known, now) = setup();
        let trusted = TrustedPeers::new(vec![p(100)]);
        let mut st = BootstrapState::default();
        let mut d = Mock::default(); // everything down

        let out = run(
            &mut d,
            &known,
            &trusted,
            p(0),
            &[t(1)],
            &mut st,
            clk(now, 0.5),
        )
        .await;
        // First failure: 30 s with the mid jitter value (no jitter).
        assert_eq!(
            out,
            Outcome::Failed {
                trusted_retry_at: Some(now + TRUSTED_BACKOFF_START)
            }
        );
        assert_eq!(d.log, vec!["dial 100"]);

        // Before the backoff is over, the trusted peer is not dialed at all.
        let soon = now + Duration::from_secs(29);
        let out = run(
            &mut d,
            &known,
            &trusted,
            p(0),
            &[t(1)],
            &mut st,
            clk(soon, 0.5),
        )
        .await;
        assert_eq!(
            out,
            Outcome::Failed {
                trusted_retry_at: Some(now + TRUSTED_BACKOFF_START)
            }
        );
        assert_eq!(d.log, vec!["dial 100"]);

        // After it: dialed again, and the next delay doubles (60 s), bounded by +-20% jitter.
        let mut at = now + TRUSTED_BACKOFF_START;
        for attempt in 1..12u32 {
            let out = run(
                &mut d,
                &known,
                &trusted,
                p(0),
                &[t(1)],
                &mut st,
                clk(at, 0.999),
            )
            .await;
            let Outcome::Failed {
                trusted_retry_at: Some(next),
            } = out
            else {
                panic!("{out:?}")
            };
            let delay = next - at;
            let base = 30.0 * 2f64.powi(attempt as i32);
            assert!(delay <= TRUSTED_BACKOFF_CAP, "never above the cap");
            assert!(delay.as_secs_f64() <= base * 1.2 + 0.01);
            if base * 0.8 < 900.0 {
                assert!(delay.as_secs_f64() >= (base * 0.8).min(900.0) - 0.01);
            }
            at = next;
        }
        // Jitter is applied before the cap: a late attempt is exactly the cap for any jitter.
        for unit in [0.0, 0.5, 1.0] {
            assert_eq!(trusted_backoff(20, unit), TRUSTED_BACKOFF_CAP);
        }
    }

    #[tokio::test]
    async fn success_resets_the_backoff() {
        let (known, now) = setup();
        let trusted = TrustedPeers::new(vec![p(100)]);
        let mut st = BootstrapState::default();
        let mut d = Mock::default();
        run(
            &mut d,
            &known,
            &trusted,
            p(0),
            &[t(1)],
            &mut st,
            clk(now, 0.5),
        )
        .await;
        assert_eq!(st.trusted_attempt, 1);
        d.up.insert(p(100));
        let later = now + Duration::from_secs(31);
        let out = run(
            &mut d,
            &known,
            &trusted,
            p(0),
            &[t(1)],
            &mut st,
            clk(later, 0.5),
        )
        .await;
        assert_eq!(out, Outcome::Trusted(p(100)));
        assert_eq!(st, BootstrapState::default());
    }
}
