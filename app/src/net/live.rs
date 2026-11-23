//! `LiveNet`: the network the app runs on. It wraps any transport that can also dial (the real
//! `IrohNet`, or the fake one in tests), runs bootstrap (ADR 0009: remembered peers, then peer
//! exchange, then trusted peers as a last resort) and publishes `NetworkView` for the UI.
//!
//! `next_event` is cancel-safe: it only awaits a channel receive and an interval tick. The
//! bootstrap work runs inside the tick branch. If the caller drops the future in the middle,
//! bootstrap simply starts over on the next tick (`BootstrapState` changes only at its end).

use super::bootstrap::{self, BootstrapState, Clock, Dialer, Outcome};
use super::exchange::{Shared, unix_now};
use super::trusted::TrustedPeers;
use crate::dungeons::TopicId;
use crate::engine::{Event, Message, NetError, Network};
use crate::identity::PeerId;
use crate::server::core::{BootstrapSource, NetworkView};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// How often the status is refreshed and bootstrap is retried while no peer is reachable.
pub const TICK: Duration = Duration::from_secs(5);

/// What `LiveNet` needs from a transport beyond `Network`.
#[allow(async_fn_in_trait)]
pub trait Transport: Network + Dialer {
    /// The exchange state (known peers) shared with the protocol handler.
    fn shared(&self) -> &Arc<Shared>;
    /// Tell every joined swarm about a freshly reached peer.
    async fn introduce(&mut self, peer: PeerId);
    /// A topic this transport joined by itself and never leaves (the membership topic).
    fn always_on(&self) -> Option<TopicId> {
        None
    }
}

impl Transport for super::IrohNet {
    fn shared(&self) -> &Arc<Shared> {
        super::IrohNet::shared(self)
    }
    async fn introduce(&mut self, peer: PeerId) {
        self.join_peer(peer).await
    }
    fn always_on(&self) -> Option<TopicId> {
        Some(crate::dungeons::membership_topic())
    }
}

pub struct LiveNet<T: Transport> {
    inner: T,
    me: PeerId,
    trusted: TrustedPeers,
    state: BootstrapState,
    source: BootstrapSource,
    topics: BTreeSet<TopicId>,
    /// A headless peer has no search, so it bootstraps on no topic at all.
    headless: bool,
    status: watch::Sender<NetworkView>,
    ticker: tokio::time::Interval,
}

fn source_of(o: &Outcome) -> Option<BootstrapSource> {
    match o {
        Outcome::Remembered(_) => Some(BootstrapSource::Remembered),
        Outcome::Exchange(_) => Some(BootstrapSource::PeerExchange),
        Outcome::Trusted(_) => Some(BootstrapSource::Trusted),
        Outcome::Failed { .. } => None,
    }
}

impl<T: Transport> LiveNet<T> {
    pub fn new(
        inner: T,
        me: PeerId,
        trusted: TrustedPeers,
        headless: bool,
        status: watch::Sender<NetworkView>,
    ) -> Self {
        let mut ticker = tokio::time::interval(TICK);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let me = Self {
            inner,
            me,
            trusted,
            state: BootstrapState::default(),
            source: BootstrapSource::None,
            topics: BTreeSet::new(),
            headless,
            status,
            ticker,
        };
        me.publish();
        me
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn source(&self) -> BootstrapSource {
        self.source
    }

    fn peer_count(&self) -> u32 {
        let connected = self.inner.connected().len();
        let known = self.inner.shared().known.lock().unwrap().len();
        connected.max(known) as u32
    }

    fn publish(&self) {
        let view = NetworkView {
            peers: self.peer_count(),
            bootstrap: self.source,
        };
        self.status.send_if_modified(|v| {
            if *v == view {
                false
            } else {
                *v = view;
                true
            }
        });
    }

    /// One bootstrap run on the joined topics. Returns the peer it reached.
    pub async fn bootstrap_once(&mut self) -> Outcome {
        // The always-on topic goes first: a request names at most 3 topics.
        let always = self.inner.always_on();
        let topics: Vec<TopicId> = always
            .into_iter()
            .chain(self.topics.iter().copied().filter(|t| Some(*t) != always))
            .collect();
        let shared = self.inner.shared().clone();
        let clk = Clock {
            now: Instant::now(),
            unix: unix_now(),
            unit: rand::random::<f64>(),
        };
        let out = bootstrap::run(
            &mut self.inner,
            &shared.known,
            &self.trusted,
            self.me,
            &topics,
            &mut self.state,
            clk,
        )
        .await;
        if let Some(s) = source_of(&out) {
            self.source = s;
        }
        match out {
            Outcome::Remembered(p) | Outcome::Exchange(p) | Outcome::Trusted(p) => {
                self.inner.introduce(p).await;
            }
            Outcome::Failed { .. } => {}
        }
        self.publish();
        out
    }

    fn wants_bootstrap(&self) -> bool {
        (self.headless || self.inner.always_on().is_some() || !self.topics.is_empty())
            && self.inner.connected().is_empty()
    }
}

impl<T: Transport> Network for LiveNet<T> {
    async fn join(&mut self, topic: TopicId) -> Result<(), NetError> {
        self.inner.join(topic).await?;
        self.topics.insert(topic);
        self.publish();
        Ok(())
    }

    async fn leave(&mut self, topic: TopicId) -> Result<(), NetError> {
        let r = self.inner.leave(topic).await;
        self.topics.remove(&topic);
        self.publish();
        r
    }

    async fn broadcast(&mut self, topic: TopicId, msg: Message) -> Result<(), NetError> {
        self.inner.broadcast(topic, msg).await
    }

    async fn send_direct(
        &mut self,
        peer: PeerId,
        topic: TopicId,
        msg: Message,
    ) -> Result<(), NetError> {
        self.inner.send_direct(peer, topic, msg).await
    }

    async fn next_event(&mut self) -> Result<Event, NetError> {
        loop {
            tokio::select! {
                ev = self.inner.next_event() => return ev,
                _ = self.ticker.tick() => {
                    if self.wants_bootstrap() {
                        // Not run on top of the select: a dropped call restarts it next tick.
                        self.bootstrap_once().await;
                    } else {
                        self.publish();
                    }
                }
            }
        }
    }

    async fn dial(&mut self, peer: PeerId) -> Result<(), NetError> {
        Network::dial(&mut self.inner, peer).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Identity, Signer};
    use crate::net::{IrohConfig, IrohNet, KnownPeers};
    use crate::store::MemStore;
    use iroh::address_lookup::MemoryLookup;

    const T: TopicId = TopicId([9; 32]);

    fn cfg(l: &MemoryLookup) -> IrohConfig {
        IrohConfig::Isolated {
            bind: "127.0.0.1:0".parse().unwrap(),
            lookup: l.clone(),
        }
    }

    /// A fresh install with no remembered peers reaches the trusted peer only as the last
    /// resort, over real QUIC on loopback (no relay, no internet), and reports it.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fresh_install_bootstraps_through_a_trusted_peer() {
        let lookup = MemoryLookup::new();
        let ida = Arc::new(Identity::generate());
        let idb = Arc::new(Identity::generate());
        let known = || KnownPeers::new(100, Arc::new(MemStore::new()));
        let a = IrohNet::start(ida.clone(), known(), cfg(&lookup))
            .await
            .unwrap();
        let b = IrohNet::start(idb.clone(), known(), cfg(&lookup))
            .await
            .unwrap();
        lookup.add_endpoint_info(a.endpoint_addr());
        lookup.add_endpoint_info(b.endpoint_addr());

        let (tx, rx) = watch::channel(NetworkView {
            peers: 0,
            bootstrap: BootstrapSource::None,
        });
        let mut live = LiveNet::new(
            b,
            idb.peer_id(),
            TrustedPeers::new(vec![ida.peer_id()]),
            false,
            tx,
        );
        live.join(T).await.unwrap();
        assert_eq!(rx.borrow().bootstrap, BootstrapSource::None);
        let out = live.bootstrap_once().await;
        assert_eq!(out, Outcome::Trusted(ida.peer_id()));
        assert_eq!(live.source(), BootstrapSource::Trusted);
        let v = rx.borrow().clone();
        assert_eq!(v.bootstrap, BootstrapSource::Trusted);
        assert!(v.peers >= 1);
        a.shutdown().await;
    }
}
