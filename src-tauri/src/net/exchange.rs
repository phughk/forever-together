//! Peer exchange protocol (spec section 4), ALPN `wowft/peers/1`.
//!
//! Messages are postcard, length-prefixed with a 4-byte big-endian length, at most 16 KiB. A
//! frame that is not complete in 5 seconds is dropped. A peer that exceeds a limit is ignored
//! for 10 minutes (memory only). A peer may send one request per 10 seconds.
//!
//! The framing and the server side are generic over tokio's `AsyncRead`/`AsyncWrite`, so they
//! are tested over an in-memory pipe and run over iroh streams unchanged.

use super::known_peers::KnownPeers;
use crate::config::PEER_EXCHANGE_MIN_INTERVAL;
use crate::dungeons::TopicId;
use crate::identity::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const ALPN: &[u8] = b"wowft/peers/1";
pub const MAX_FRAME: usize = 16 * 1024;
pub const MAX_HELLO_TOPICS: usize = 20;
pub const MAX_REQUEST_TOPICS: usize = 3;
pub const MAX_REQUEST_ENTRIES: usize = 20;
pub const MAX_RESPONSE_ENTRIES: usize = 20;
pub const MAX_ENTRY_TOPICS: usize = 20;
pub const FRAME_TIMEOUT: Duration = Duration::from_secs(5);
pub const IGNORE_FOR: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub peer: PeerId,
    pub topics: Vec<TopicId>,
    pub last_seen: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wire {
    /// Sent by each side on its own stream when a connection opens.
    Hello {
        topics: Vec<TopicId>,
    },
    Request {
        topics: Vec<TopicId>,
        max: u8,
    },
    Response {
        entries: Vec<Entry>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExchangeError {
    #[error("a limit was exceeded: {0}")]
    LimitExceeded(&'static str),
    #[error("the peer is rate limited")]
    RateLimited,
    #[error("the peer is ignored")]
    Ignored,
    #[error("no complete message in time")]
    Timeout,
    #[error("malformed message")]
    Malformed,
    #[error("stream closed or failed: {0}")]
    Io(String),
}

/// Check the per-message limits.
pub fn validate(w: &Wire) -> Result<(), ExchangeError> {
    match w {
        Wire::Hello { topics } if topics.len() > MAX_HELLO_TOPICS => {
            Err(ExchangeError::LimitExceeded("hello topics"))
        }
        Wire::Request { topics, max } => {
            if topics.len() > MAX_REQUEST_TOPICS {
                Err(ExchangeError::LimitExceeded("request topics"))
            } else if *max as usize > MAX_REQUEST_ENTRIES {
                Err(ExchangeError::LimitExceeded("request max"))
            } else {
                Ok(())
            }
        }
        Wire::Response { entries } => {
            if entries.len() > MAX_RESPONSE_ENTRIES {
                Err(ExchangeError::LimitExceeded("response entries"))
            } else if entries.iter().any(|e| e.topics.len() > MAX_ENTRY_TOPICS) {
                Err(ExchangeError::LimitExceeded("entry topics"))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

pub fn encode_frame(w: &Wire) -> Result<Vec<u8>, ExchangeError> {
    let body = postcard::to_stdvec(w).map_err(|_| ExchangeError::Malformed)?;
    if body.len() > MAX_FRAME {
        return Err(ExchangeError::LimitExceeded("frame size"));
    }
    let mut v = (body.len() as u32).to_be_bytes().to_vec();
    v.extend_from_slice(&body);
    Ok(v)
}

pub async fn write_frame<W: AsyncWrite + Unpin>(
    w: &mut W,
    msg: &Wire,
) -> Result<(), ExchangeError> {
    let bytes = encode_frame(msg)?;
    let io = async {
        w.write_all(&bytes).await?;
        w.flush().await
    };
    tokio::time::timeout(FRAME_TIMEOUT, io)
        .await
        .map_err(|_| ExchangeError::Timeout)?
        .map_err(|e| ExchangeError::Io(e.to_string()))
}

/// Read one complete frame within `FRAME_TIMEOUT` and check its limits. A declared length
/// above 16 KiB is refused before any of the body is read.
pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> Result<Wire, ExchangeError> {
    let io = async {
        let mut len = [0u8; 4];
        r.read_exact(&mut len)
            .await
            .map_err(|e| ExchangeError::Io(e.to_string()))?;
        let len = u32::from_be_bytes(len) as usize;
        if len > MAX_FRAME {
            return Err(ExchangeError::LimitExceeded("frame size"));
        }
        let mut body = vec![0u8; len];
        r.read_exact(&mut body)
            .await
            .map_err(|e| ExchangeError::Io(e.to_string()))?;
        postcard::from_bytes::<Wire>(&body).map_err(|_| ExchangeError::Malformed)
    };
    let msg = tokio::time::timeout(FRAME_TIMEOUT, io)
        .await
        .map_err(|_| ExchangeError::Timeout)??;
    validate(&msg)?;
    Ok(msg)
}

/// Per-peer rate limit and the ignore list. Memory only.
#[derive(Default)]
pub struct ExchangeGuard {
    last_request: HashMap<PeerId, Instant>,
    ignored: HashMap<PeerId, Instant>,
}

impl ExchangeGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ignore(&mut self, peer: PeerId, now: Instant) {
        self.ignored.insert(peer, now + IGNORE_FOR);
    }

    pub fn is_ignored(&mut self, peer: &PeerId, now: Instant) -> bool {
        match self.ignored.get(peer) {
            Some(until) if now < *until => true,
            Some(_) => {
                self.ignored.remove(peer);
                false
            }
            None => false,
        }
    }

    /// Admit a request from `peer`: one per `PEER_EXCHANGE_MIN_INTERVAL`. A second one inside
    /// the interval exceeds the limit and the peer is ignored for 10 minutes.
    pub fn admit_request(&mut self, peer: PeerId, now: Instant) -> Result<(), ExchangeError> {
        if self.is_ignored(&peer, now) {
            return Err(ExchangeError::Ignored);
        }
        if let Some(last) = self.last_request.get(&peer)
            && now.duration_since(*last) < PEER_EXCHANGE_MIN_INTERVAL
        {
            self.ignore(peer, now);
            return Err(ExchangeError::RateLimited);
        }
        self.last_request.insert(peer, now);
        // Bound the table: forget peers whose interval is long over.
        if self.last_request.len() > 4096 {
            self.last_request
                .retain(|_, t| now.duration_since(*t) < PEER_EXCHANGE_MIN_INTERVAL);
        }
        Ok(())
    }
}

/// State the exchange server shares with the rest of `net`.
pub struct Shared {
    pub known: Mutex<KnownPeers>,
    pub guard: Mutex<ExchangeGuard>,
    /// The topics we search, sent in our `Hello`.
    pub topics: Mutex<Vec<TopicId>>,
}

impl Shared {
    pub fn new(known: KnownPeers) -> Self {
        Self {
            known: Mutex::new(known),
            guard: Mutex::new(ExchangeGuard::new()),
            topics: Mutex::new(Vec::new()),
        }
    }
}

pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Handle one stream from `peer`: a `Hello` (recorded, no reply) or a `Request` (rate limited,
/// answered with verified entries only). Any limit violation ignores the peer.
pub async fn serve_stream<R, W>(
    shared: &Shared,
    peer: PeerId,
    r: &mut R,
    w: &mut W,
    now: Instant,
    unix: u64,
) -> Result<(), ExchangeError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    if shared.guard.lock().unwrap().is_ignored(&peer, now) {
        return Err(ExchangeError::Ignored);
    }
    let msg = match read_frame(r).await {
        Ok(m) => m,
        Err(e @ (ExchangeError::LimitExceeded(_) | ExchangeError::Malformed)) => {
            shared.guard.lock().unwrap().ignore(peer, now);
            return Err(e);
        }
        Err(e) => return Err(e),
    };
    match msg {
        Wire::Hello { topics } => {
            // A claim by an authenticated connection. It stays pending until we dial it.
            shared
                .known
                .lock()
                .unwrap()
                .observe(peer, &topics, unix)
                .map_err(|e| ExchangeError::Io(e.to_string()))?;
            Ok(())
        }
        Wire::Request { topics, max } => {
            shared.guard.lock().unwrap().admit_request(peer, now)?;
            let entries: Vec<Entry> = shared
                .known
                .lock()
                .unwrap()
                .sample(&topics, (max as usize).min(MAX_RESPONSE_ENTRIES), &peer)
                .into_iter()
                .map(|(p, r)| Entry {
                    peer: p,
                    topics: r.topics,
                    last_seen: r.last_seen,
                })
                .collect();
            write_frame(w, &Wire::Response { entries }).await?;
            w.shutdown().await.ok();
            Ok(())
        }
        // A client must not send a response.
        Wire::Response { .. } => {
            shared.guard.lock().unwrap().ignore(peer, now);
            Err(ExchangeError::Malformed)
        }
    }
}

/// Send our `Hello` on a stream.
pub async fn send_hello<W: AsyncWrite + Unpin>(
    w: &mut W,
    topics: &[TopicId],
) -> Result<(), ExchangeError> {
    let topics = topics.iter().take(MAX_HELLO_TOPICS).copied().collect();
    write_frame(w, &Wire::Hello { topics }).await?;
    w.shutdown().await.ok();
    Ok(())
}

/// Ask for peers on `topics` (at most 3 are sent) and read the response. A response over the
/// limits is an error, and the caller should stop talking to that peer.
pub async fn request_peers<R, W>(
    r: &mut R,
    w: &mut W,
    topics: &[TopicId],
    max: u8,
) -> Result<Vec<Entry>, ExchangeError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let topics = topics.iter().take(MAX_REQUEST_TOPICS).copied().collect();
    let max = max.min(MAX_REQUEST_ENTRIES as u8);
    write_frame(w, &Wire::Request { topics, max }).await?;
    w.shutdown().await.ok();
    match read_frame(r).await? {
        Wire::Response { entries } => Ok(entries),
        _ => Err(ExchangeError::Malformed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemStore;
    use std::sync::Arc;

    fn p(n: u8) -> PeerId {
        PeerId([n; 32])
    }
    fn t(n: u8) -> TopicId {
        TopicId([n; 32])
    }

    fn shared_with(peers: &[(u8, u8)]) -> Shared {
        let mut k = KnownPeers::new(100, Arc::new(MemStore::new()));
        for (peer, topic) in peers {
            k.mark_verified(p(*peer), &[t(*topic)], 1000 + *peer as u64)
                .unwrap();
        }
        Shared::new(k)
    }

    /// Run one client request against the server over an in-memory pipe.
    async fn ask(
        shared: &Shared,
        from: PeerId,
        topics: &[TopicId],
        max: u8,
        now: Instant,
    ) -> Result<Vec<Entry>, ExchangeError> {
        let (mut c_to_s_w, mut c_to_s_r) = tokio::io::duplex(64 * 1024);
        let (mut s_to_c_w, mut s_to_c_r) = tokio::io::duplex(64 * 1024);
        let server = serve_stream(shared, from, &mut c_to_s_r, &mut s_to_c_w, now, 5000);
        let client = request_peers(&mut s_to_c_r, &mut c_to_s_w, topics, max);
        let (s, c) = tokio::join!(server, client);
        match (s, c) {
            (Ok(()), c) => c,
            (Err(e), _) => Err(e),
        }
    }

    #[test]
    fn limits_are_enforced_by_validate() {
        let ok = |w: Wire| validate(&w).is_ok();
        assert!(ok(Wire::Hello {
            topics: vec![t(1); 20]
        }));
        assert!(!ok(Wire::Hello {
            topics: vec![t(1); 21]
        }));
        assert!(ok(Wire::Request {
            topics: vec![t(1); 3],
            max: 20
        }));
        assert!(!ok(Wire::Request {
            topics: vec![t(1); 4],
            max: 1
        }));
        assert!(!ok(Wire::Request {
            topics: vec![],
            max: 21
        }));
        let e = |n| Entry {
            peer: p(1),
            topics: vec![t(1); n],
            last_seen: 0,
        };
        assert!(ok(Wire::Response {
            entries: vec![e(20); 20]
        }));
        assert!(!ok(Wire::Response {
            entries: vec![e(1); 21]
        }));
        assert!(!ok(Wire::Response {
            entries: vec![e(21)]
        }));
    }

    #[tokio::test]
    async fn frame_round_trip_and_size_limit() {
        let (mut a, mut b) = tokio::io::duplex(64 * 1024);
        let m = Wire::Request {
            topics: vec![t(1), t(2)],
            max: 7,
        };
        write_frame(&mut a, &m).await.unwrap();
        assert_eq!(read_frame(&mut b).await.unwrap(), m);

        // A declared length above 16 KiB is refused without reading a body.
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&((MAX_FRAME as u32 + 1).to_be_bytes()))
            .await
            .unwrap();
        assert_eq!(
            read_frame(&mut b).await,
            Err(ExchangeError::LimitExceeded("frame size"))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn incomplete_frame_times_out_after_five_seconds() {
        let (mut a, mut b) = tokio::io::duplex(64);
        // Length says 10 bytes, only 3 arrive.
        a.write_all(&[0, 0, 0, 10, 1, 2, 3]).await.unwrap();
        let start = tokio::time::Instant::now();
        assert_eq!(read_frame(&mut b).await, Err(ExchangeError::Timeout));
        assert_eq!(start.elapsed(), FRAME_TIMEOUT);
        drop(a);
    }

    #[tokio::test]
    async fn garbage_body_is_malformed() {
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&[0, 0, 0, 2, 0xff, 0xff]).await.unwrap();
        assert_eq!(read_frame(&mut b).await, Err(ExchangeError::Malformed));
    }

    #[tokio::test]
    async fn answers_a_request_with_verified_entries_for_the_topics() {
        let shared = shared_with(&[(1, 1), (2, 1), (3, 2)]);
        // An unverified claim must not be handed out.
        shared
            .known
            .lock()
            .unwrap()
            .observe(p(4), &[t(1)], 1)
            .unwrap();
        let now = Instant::now();
        let got = ask(&shared, p(9), &[t(1)], 20, now).await.unwrap();
        let mut peers: Vec<PeerId> = got.iter().map(|e| e.peer).collect();
        peers.sort();
        assert_eq!(peers, vec![p(1), p(2)]);
        // max is honoured, and the requester is never in its own answer.
        let got = ask(&shared, p(1), &[t(1)], 20, now + Duration::from_secs(20))
            .await
            .unwrap();
        assert_eq!(got.iter().map(|e| e.peer).collect::<Vec<_>>(), vec![p(2)]);
        let got = ask(&shared, p(8), &[t(1)], 1, now + Duration::from_secs(20))
            .await
            .unwrap();
        assert_eq!(got.len(), 1);
    }

    #[tokio::test]
    async fn one_request_per_ten_seconds_then_ignored_for_ten_minutes() {
        let shared = shared_with(&[(1, 1)]);
        let t0 = Instant::now();
        assert!(ask(&shared, p(9), &[t(1)], 5, t0).await.is_ok());
        // Another request after 9 s exceeds the rate and gets the peer ignored.
        assert_eq!(
            ask(&shared, p(9), &[t(1)], 5, t0 + Duration::from_secs(9)).await,
            Err(ExchangeError::RateLimited)
        );
        // Even a well-spaced request is refused while ignored (10 minutes).
        assert_eq!(
            ask(&shared, p(9), &[t(1)], 5, t0 + Duration::from_secs(60)).await,
            Err(ExchangeError::Ignored)
        );
        assert_eq!(
            ask(&shared, p(9), &[t(1)], 5, t0 + Duration::from_secs(605)).await,
            Err(ExchangeError::Ignored)
        );
        assert!(
            ask(&shared, p(9), &[t(1)], 5, t0 + Duration::from_secs(610))
                .await
                .is_ok()
        );
        // Other peers are not affected, and 10 s apart is fine.
        assert!(
            ask(&shared, p(7), &[t(1)], 5, t0 + Duration::from_secs(1))
                .await
                .is_ok()
        );
        assert!(
            ask(&shared, p(7), &[t(1)], 5, t0 + Duration::from_secs(11))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn oversize_request_ignores_the_peer() {
        let shared = shared_with(&[(1, 1)]);
        let t0 = Instant::now();
        let (mut a, mut s_r) = tokio::io::duplex(64 * 1024);
        let (mut s_w, _c_r) = tokio::io::duplex(64 * 1024);
        // Hand-built request with 4 topics (the limit is 3).
        let body = postcard::to_stdvec(&Wire::Request {
            topics: vec![t(1); 4],
            max: 5,
        })
        .unwrap();
        a.write_all(&(body.len() as u32).to_be_bytes())
            .await
            .unwrap();
        a.write_all(&body).await.unwrap();
        assert_eq!(
            serve_stream(&shared, p(9), &mut s_r, &mut s_w, t0, 0).await,
            Err(ExchangeError::LimitExceeded("request topics"))
        );
        assert!(shared.guard.lock().unwrap().is_ignored(&p(9), t0));
        assert!(
            !shared
                .guard
                .lock()
                .unwrap()
                .is_ignored(&p(9), t0 + IGNORE_FOR + Duration::from_secs(1))
        );
    }

    #[tokio::test]
    async fn hello_is_recorded_as_a_pending_claim() {
        let shared = shared_with(&[]);
        let (mut a, mut s_r) = tokio::io::duplex(64 * 1024);
        let (mut s_w, _c_r) = tokio::io::duplex(64);
        send_hello(&mut a, &[t(1), t(2)]).await.unwrap();
        serve_stream(&shared, p(9), &mut s_r, &mut s_w, Instant::now(), 77)
            .await
            .unwrap();
        let k = shared.known.lock().unwrap();
        assert_eq!(k.len(), 0, "not verified until dialed");
        assert_eq!(k.pending_for(&[t(2)]), vec![p(9)]);
    }

    #[tokio::test]
    async fn client_rejects_an_oversize_response() {
        let (mut c_w, _s_r) = tokio::io::duplex(64 * 1024);
        let (mut s_w, mut c_r) = tokio::io::duplex(64 * 1024);
        let entries = vec![
            Entry {
                peer: p(1),
                topics: vec![],
                last_seen: 0
            };
            21
        ];
        // Write it past the sender-side checks, as a hostile server would.
        let body = postcard::to_stdvec(&Wire::Response { entries }).unwrap();
        s_w.write_all(&(body.len() as u32).to_be_bytes())
            .await
            .unwrap();
        s_w.write_all(&body).await.unwrap();
        assert_eq!(
            request_peers(&mut c_r, &mut c_w, &[t(1)], 20).await,
            Err(ExchangeError::LimitExceeded("response entries"))
        );
    }
}
