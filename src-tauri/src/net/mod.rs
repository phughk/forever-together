//! The network: iroh transport, signed gossip, peer exchange, bootstrap and the known-peers
//! table (plan item 23).
//!
//! * `gossip`: the signed envelope. Every received message is verified here before an event can
//!   reach the engine (ADR 0019).
//! * `known_peers`: the LRU table with reserved slots for active topics, persisted in the store.
//! * `exchange`: the peer exchange protocol (ALPN `wowft/peers/1`) and its limits.
//! * `bootstrap`: remembered peers, then peer exchange, then trusted peers with backoff.
//! * `trusted`: the trusted peer keys.
//! * `iroh_net`: the real `Network` on iroh and iroh-gossip.
//! * `fake`: an in-memory `Network` for tests.

pub mod bootstrap;
pub mod exchange;
pub mod fake;
pub mod gossip;
pub mod iroh_net;
pub mod known_peers;
pub mod trusted;

pub use fake::{FakeHub, FakeNet};
pub use iroh_net::{IrohConfig, IrohNet};
pub use known_peers::KnownPeers;
pub use trusted::TrustedPeers;
