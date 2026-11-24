//! The real network: an iroh endpoint, iroh-gossip (one swarm per topic) and the peer exchange
//! protocol on its own ALPN.
//!
//! Notes that follow from reading iroh 1.5 and iroh-gossip 0.103:
//!
//! * Gossip content is not signed and has no author field, so `gossip::seal` / `gossip::open`
//!   sign and verify in this app (ADR 0019 stands).
//! * Gossip has no direct send. `send_direct` is a signed broadcast addressed to one peer
//!   (`to` is inside the signed bytes); other peers verify and drop it. Party members are
//!   known to the swarm anyway, and nothing in a party message is secret.
//! * A gossip broadcast reaches only the neighbours present when it is sent. The engine
//!   re-sends ads every keep-alive, so a lost first broadcast is repaired.
//! * The default (`Default` mode) uses iroh's `N0` preset: n0's relay servers and the n0
//!   DNS/pkarr address lookup service (`iroh.link`). See `docs/spike-s0.md`.

use super::bootstrap::Dialer;
use super::exchange::{self, ALPN as EXCHANGE_ALPN, Entry, Shared, serve_stream, unix_now};
use super::gossip::{open, seal};
use super::known_peers::KnownPeers;
use crate::dungeons::{TopicId, membership_topic};
use crate::engine::{Event, Message, NetError, Network, event_from_message};
use crate::identity::{Identity, PeerId, Signer};
use bytes::Bytes;
use futures_lite::StreamExt;
use iroh::address_lookup::{
    AddrFilter, DnsAddressLookup, MemoryLookup, PkarrPublisher, PkarrResolver,
};
use iroh::endpoint::{Connection, presets};
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::{Endpoint, EndpointAddr, EndpointId, RelayMap, RelayMode, RelayUrl, SecretKey};
use iroh_gossip::{Event as GossipEvent, Gossip, GossipSender};
use iroh_mainline_address_lookup::DhtAddressLookup;
use iroh_relay::RelayConfig;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const DIAL_TIMEOUT: Duration = Duration::from_secs(10);
const EVENT_QUEUE: usize = 512;
/// Peers handed to gossip as bootstrap when joining a topic.
const BOOTSTRAP_PEERS: usize = 5;

/// How the endpoint is set up.
#[derive(Clone, Debug, Default)]
pub enum IrohConfig {
    /// n0's relays plus the project's (`trusted::RELAYS`), and address lookup through n0's DNS and
    /// the BitTorrent Mainline DHT, both publishing direct addresses (ADR 0041).
    #[default]
    Default,
    /// Tests of the relay path: one given relay, no direct paths (no IP transport), peers
    /// resolved from the in-memory lookup. The relay is where every packet goes.
    RelayOnly {
        relay: RelayUrl,
        lookup: MemoryLookup,
    },
    /// No relay, no hosted lookup, bound to one local address; peers are resolved from the
    /// given in-memory lookup. For tests and for LAN-only experiments.
    Isolated {
        bind: SocketAddr,
        lookup: MemoryLookup,
    },
}

/// n0's default relays plus the project's own (`trusted::RELAYS`).
fn relay_map() -> RelayMap {
    let map = RelayMode::Default.relay_map();
    for u in super::trusted::RELAYS {
        match u.parse::<RelayUrl>() {
            Ok(url) => {
                map.insert(url.clone(), Arc::new(RelayConfig::new(url, None)));
            }
            Err(e) => tracing::warn!("ignoring relay url {u}: {e}"),
        }
    }
    map
}

fn io<E: std::fmt::Display>(e: E) -> NetError {
    NetError::Io(e.to_string())
}

fn endpoint_id(p: PeerId) -> Result<EndpointId, NetError> {
    EndpointId::from_bytes(&p.0).map_err(io)
}

fn peer_of(id: EndpointId) -> PeerId {
    PeerId(*id.as_bytes())
}

struct ExchangeHandler {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for ExchangeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExchangeHandler")
    }
}

impl ProtocolHandler for ExchangeHandler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        serve_connection(self.shared.clone(), connection, true).await;
        Ok(())
    }
}

/// Our `Hello` (if we opened the connection or accepted it, both send one), then serve the
/// remote's streams until it closes.
async fn serve_connection(shared: Arc<Shared>, conn: Connection, send_hello: bool) {
    let peer = peer_of(conn.remote_id());
    if shared
        .guard
        .lock()
        .unwrap()
        .is_ignored(&peer, Instant::now())
    {
        conn.close(1u8.into(), b"ignored");
        return;
    }
    if send_hello {
        send_hello_on(&shared, &conn).await;
    }
    while let Ok((mut send, mut recv)) = conn.accept_bi().await {
        let shared = shared.clone();
        tokio::spawn(async move {
            let _ = serve_stream(
                &shared,
                peer,
                &mut recv,
                &mut send,
                Instant::now(),
                unix_now(),
            )
            .await;
        });
    }
}

async fn send_hello_on(shared: &Shared, conn: &Connection) {
    let topics = shared.topics.lock().unwrap().clone();
    if let Ok((mut send, _recv)) = conn.open_bi().await {
        let _ = exchange::send_hello(&mut send, &topics).await;
        let _ = send.finish();
    }
}

struct TopicHandle {
    sender: GossipSender,
    task: JoinHandle<()>,
}

impl Drop for TopicHandle {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub struct IrohNet {
    me: PeerId,
    signer: Arc<Identity>,
    endpoint: Endpoint,
    router: Router,
    gossip: Gossip,
    shared: Arc<Shared>,
    topics: HashMap<TopicId, TopicHandle>,
    events_tx: mpsc::Sender<Event>,
    events_rx: mpsc::Receiver<Event>,
    conns: HashMap<PeerId, Connection>,
    /// Dials in flight. They run as tasks, so a caller that is dropped (the core's loop drops
    /// `next_event` whenever a command or tick arrives) does not cancel them.
    dials: HashMap<PeerId, JoinHandle<Result<Connection, NetError>>>,
    member_window: Instant,
    member_joins: u32,
}

impl IrohNet {
    /// Bind the endpoint with the identity's key and start gossip and the exchange protocol.
    pub async fn start(
        identity: Arc<Identity>,
        known: KnownPeers,
        config: IrohConfig,
    ) -> Result<Self, NetError> {
        let key = SecretKey::from_bytes(&identity.secret_bytes());
        let builder = match config {
            IrohConfig::Default => Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::Custom(relay_map()))
                .address_lookup(PkarrPublisher::n0_dns().addr_filter(AddrFilter::unfiltered()))
                .address_lookup(PkarrResolver::n0_dns())
                .address_lookup(DnsAddressLookup::n0_dns())
                .address_lookup(DhtAddressLookup::builder().addr_filter(AddrFilter::unfiltered())),
            IrohConfig::RelayOnly { relay, lookup } => Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::custom([relay]))
                .clear_address_lookup()
                .address_lookup(lookup)
                .clear_ip_transports(),
            IrohConfig::Isolated { bind, lookup } => Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::Disabled)
                .clear_address_lookup()
                .address_lookup(lookup)
                .clear_ip_transports()
                .bind_addr(bind)
                .map_err(io)?,
        };
        let endpoint = builder.secret_key(key).bind().await.map_err(io)?;
        let me = peer_of(endpoint.id());
        debug_assert_eq!(me, identity.peer_id(), "iroh id is the ed25519 public key");
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let shared = Arc::new(Shared::new(known));
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .accept(
                EXCHANGE_ALPN,
                ExchangeHandler {
                    shared: shared.clone(),
                },
            )
            .spawn();
        let (events_tx, events_rx) = mpsc::channel(EVENT_QUEUE);
        let mut net = Self {
            me,
            signer: identity,
            endpoint,
            router,
            gossip,
            shared,
            topics: HashMap::new(),
            events_tx,
            events_rx,
            conns: HashMap::new(),
            dials: HashMap::new(),
            member_window: Instant::now(),
            member_joins: 0,
        };
        // Every node is in the membership swarm from the start, searching or not (ADR 0040).
        net.join(membership_topic()).await?;
        Ok(net)
    }

    pub fn peer_id(&self) -> PeerId {
        self.me
    }

    pub fn endpoint_addr(&self) -> EndpointAddr {
        self.endpoint.addr()
    }

    pub fn shared(&self) -> &Arc<Shared> {
        &self.shared
    }

    /// Connect to `peer` on the exchange ALPN, send our `Hello`, and keep the connection.
    /// Start dialing `peer` as a background task unless it is connected or already being dialed.
    pub fn start_dial_peer(&mut self, peer: PeerId) {
        if self
            .conns
            .get(&peer)
            .is_some_and(|c| c.close_reason().is_none())
            || self.dials.contains_key(&peer)
        {
            return;
        }
        let Ok(id) = endpoint_id(peer) else { return };
        let endpoint = self.endpoint.clone();
        self.dials.insert(
            peer,
            tokio::spawn(async move {
                tokio::time::timeout(DIAL_TIMEOUT, endpoint.connect(id, EXCHANGE_ALPN))
                    .await
                    .map_err(|_| NetError::Io("dial timed out".into()))?
                    .map_err(io)
            }),
        );
    }

    /// Connect to `peer` on the exchange ALPN, send our `Hello`, and keep the connection. The
    /// dial itself is a task: if this future is dropped, the next call picks the result up.
    pub async fn dial_peer(&mut self, peer: PeerId) -> Result<(), NetError> {
        if self
            .conns
            .get(&peer)
            .is_some_and(|c| c.close_reason().is_none())
        {
            return Ok(());
        }
        endpoint_id(peer)?;
        self.start_dial_peer(peer);
        let handle = self.dials.get_mut(&peer).ok_or(NetError::UnknownPeer)?;
        let joined = handle.await;
        self.dials.remove(&peer);
        let conn = joined.map_err(io)??;
        let shared = self.shared.clone();
        let c2 = conn.clone();
        tokio::spawn(async move { serve_connection(shared, c2, true).await });
        self.conns.insert(peer, conn);
        Ok(())
    }

    /// Hand `peer` to every joined swarm as a bootstrap peer (used after bootstrap reached it).
    pub async fn join_peer(&self, peer: PeerId) {
        let Ok(id) = endpoint_id(peer) else { return };
        for h in self.topics.values() {
            let _ = h.sender.join_peers(vec![id]).await;
        }
    }

    pub async fn shutdown(self) {
        let _ = self.router.shutdown().await;
        let _ = self.gossip.shutdown().await;
        self.endpoint.close().await;
    }

    fn mark_verified(&self, peer: PeerId, topics: &[TopicId]) {
        let _ = self
            .shared
            .known
            .lock()
            .unwrap()
            .mark_verified(peer, topics, unix_now());
    }

    fn sync_topics(&self) {
        let t: Vec<TopicId> = self.topics.keys().copied().collect();
        self.shared
            .known
            .lock()
            .unwrap()
            .set_active_topics(t.clone());
        *self.shared.topics.lock().unwrap() = t;
    }
}

/// New-member messages acted on per window; more are dropped (ADR 0040: a poster can name any
/// key, so each one costs a dial, and the rate is capped).
const MEMBER_JOINS_PER_WINDOW: u32 = 30;
const MEMBER_WINDOW: Duration = Duration::from_secs(60);

async fn pump(
    topic: TopicId,
    mut rx: iroh_gossip::GossipReceiver,
    tx: mpsc::Sender<Event>,
    me: PeerId,
    shared: Arc<Shared>,
    sender: GossipSender,
    signer: Arc<Identity>,
) {
    while let Some(ev) = rx.next().await {
        match ev {
            Ok(GossipEvent::Received(m)) => match open(topic, &m.content) {
                Ok(v) if v.author != me && v.is_for(me) => {
                    if tx.send(event_from_message(topic, v.msg)).await.is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) => tracing::debug!(?topic, error = %e, "dropped an unverified message"),
            },
            // A gossip neighbour is a live, authenticated peer: remember it for this topic.
            Ok(GossipEvent::NeighborUp(id)) => {
                let member = peer_of(id);
                // On the membership topic, a neighbour never seen before is announced to the
                // whole swarm (ADR 0040), so every member can introduce it to its own swarms.
                let first_time = topic == membership_topic()
                    && member != me
                    && !shared.known.lock().unwrap().is_verified(&member);
                let _ = shared
                    .known
                    .lock()
                    .unwrap()
                    .mark_verified(member, &[topic], unix_now());
                if first_time {
                    let msg = Message::NewMember { peer: me, member };
                    if let Ok(bytes) = seal(&*signer, topic, None, &msg) {
                        let _ = sender.broadcast(Bytes::from(bytes)).await;
                    }
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

impl Network for IrohNet {
    async fn join(&mut self, topic: TopicId) -> Result<(), NetError> {
        if self.topics.contains_key(&topic) {
            return Ok(());
        }
        // Peers known for this topic first, then members of the network (ADR 0040).
        let (for_topic, members) = {
            let k = self.shared.known.lock().unwrap();
            (k.remembered(&[topic]), k.remembered(&[membership_topic()]))
        };
        let bootstrap: Vec<EndpointId> = for_topic
            .into_iter()
            .chain(members)
            .take(BOOTSTRAP_PEERS)
            .filter_map(|p| endpoint_id(p).ok())
            .collect();
        let sub = self
            .gossip
            .subscribe(iroh_gossip::TopicId::from_bytes(topic.0), bootstrap)
            .await
            .map_err(io)?;
        let (sender, receiver) = sub.split();
        let task = tokio::spawn(pump(
            topic,
            receiver,
            self.events_tx.clone(),
            self.me,
            self.shared.clone(),
            sender.clone(),
            self.signer.clone(),
        ));
        self.topics.insert(topic, TopicHandle { sender, task });
        self.sync_topics();
        Ok(())
    }

    async fn leave(&mut self, topic: TopicId) -> Result<(), NetError> {
        // Dropping the handle aborts the pump and the sender, which leaves the swarm.
        self.topics.remove(&topic).ok_or(NetError::NotJoined)?;
        self.sync_topics();
        Ok(())
    }

    async fn broadcast(&mut self, topic: TopicId, msg: Message) -> Result<(), NetError> {
        let bytes = seal(&*self.signer, topic, None, &msg)?;
        let h = self.topics.get(&topic).ok_or(NetError::NotJoined)?;
        h.sender.broadcast(Bytes::from(bytes)).await.map_err(io)
    }

    async fn send_direct(
        &mut self,
        peer: PeerId,
        topic: TopicId,
        msg: Message,
    ) -> Result<(), NetError> {
        let bytes = seal(&*self.signer, topic, Some(peer), &msg)?;
        let h = self.topics.get(&topic).ok_or(NetError::NotJoined)?;
        h.sender.broadcast(Bytes::from(bytes)).await.map_err(io)
    }

    async fn next_event(&mut self) -> Result<Event, NetError> {
        loop {
            let ev = self.events_rx.recv().await.ok_or(NetError::Closed)?;
            if let Event::MemberJoined { member } = ev {
                // A new member: hand it to every swarm we are in. If it searches the same
                // dungeon topic it connects; if not, the join is ignored on its side.
                let now = Instant::now();
                if now.duration_since(self.member_window) >= MEMBER_WINDOW {
                    self.member_window = now;
                    self.member_joins = 0;
                }
                if member != self.me && self.member_joins < MEMBER_JOINS_PER_WINDOW {
                    self.member_joins += 1;
                    self.join_peer(member).await;
                }
                continue;
            }
            return Ok(ev);
        }
    }

    async fn dial(&mut self, peer: PeerId) -> Result<(), NetError> {
        self.dial_peer(peer).await?;
        // Pending claims about the peer move into the table with it. We invent no topics.
        self.mark_verified(peer, &[]);
        Ok(())
    }
}

impl Dialer for IrohNet {
    async fn dial(&mut self, peer: PeerId) -> Result<(), NetError> {
        self.dial_peer(peer).await
    }

    fn start_dial(&mut self, peer: PeerId) {
        self.start_dial_peer(peer)
    }

    fn connected(&self) -> Vec<PeerId> {
        let mut v: Vec<PeerId> = self
            .conns
            .iter()
            .filter(|(_, c)| c.close_reason().is_none())
            .map(|(p, _)| *p)
            .collect();
        v.sort();
        v
    }

    async fn request_peers(
        &mut self,
        via: PeerId,
        topics: &[TopicId],
    ) -> Result<Vec<Entry>, NetError> {
        let conn = self.conns.get(&via).ok_or(NetError::UnknownPeer)?;
        let (mut send, mut recv) = conn.open_bi().await.map_err(io)?;
        exchange::request_peers(&mut recv, &mut send, topics, 20)
            .await
            .map_err(io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::PartyId;
    use crate::store::MemStore;

    const T: TopicId = TopicId([7; 32]);

    fn known() -> KnownPeers {
        KnownPeers::new(100, Arc::new(MemStore::new()))
    }

    fn confirm(id: &Identity) -> Message {
        Message::Confirm {
            peer: id.peer_id(),
            party_id: PartyId([3; 32]),
        }
    }

    fn isolated(lookup: &MemoryLookup) -> IrohConfig {
        IrohConfig::Isolated {
            bind: "127.0.0.1:0".parse().unwrap(),
            lookup: lookup.clone(),
        }
    }

    /// Two real iroh endpoints on loopback with no relay and no hosted lookup: they find each
    /// other through an in-memory lookup, join one gossip swarm, and a signed message goes
    /// from one to the other. Also checks peer exchange over real QUIC streams. Needs no
    /// internet.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn loopback_two_endpoint_gossip_and_exchange() {
        let lookup = MemoryLookup::new();
        let ida = Arc::new(Identity::generate());
        let idb = Arc::new(Identity::generate());
        let third = PeerId([42; 32]);

        // A knows a third peer for the topic (verified), so it can hand it out.
        let mut ka = known();
        ka.mark_verified(third, &[T], 5).unwrap();
        // B remembers A, which is how it finds the swarm.
        let mut kb = known();
        kb.mark_verified(ida.peer_id(), &[T], 5).unwrap();

        let mut a = IrohNet::start(ida.clone(), ka, isolated(&lookup))
            .await
            .unwrap();
        let mut b = IrohNet::start(idb.clone(), kb, isolated(&lookup))
            .await
            .unwrap();
        assert_eq!(a.peer_id(), ida.peer_id(), "iroh id is our identity key");
        lookup.add_endpoint_info(a.endpoint_addr());
        lookup.add_endpoint_info(b.endpoint_addr());

        // Peer exchange over a real connection.
        Dialer::dial(&mut b, a.peer_id()).await.unwrap();
        assert_eq!(Dialer::connected(&b), vec![a.peer_id()]);
        let entries = Dialer::request_peers(&mut b, a.peer_id(), &[T])
            .await
            .unwrap();
        assert_eq!(
            entries.iter().map(|e| e.peer).collect::<Vec<_>>(),
            vec![third]
        );

        // Gossip: a signed message from A reaches B.
        a.join(T).await.unwrap();
        b.join(T).await.unwrap();
        let msg = confirm(&ida);
        let got = tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                a.broadcast(T, msg.clone()).await.unwrap();
                if let Ok(ev) =
                    tokio::time::timeout(Duration::from_millis(500), b.next_event()).await
                {
                    return ev.unwrap();
                }
            }
        })
        .await
        .expect("message did not arrive");
        assert!(matches!(
            got,
            Event::ConfirmReceived { from, .. } if from == ida.peer_id()
        ));
        // The gossip neighbour is now a verified known peer for the topic.
        assert!(b.shared().known.lock().unwrap().is_verified(&ida.peer_id()));

        a.shutdown().await;
        b.shutdown().await;
    }
}
