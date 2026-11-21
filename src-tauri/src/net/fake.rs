//! An in-memory `Network` for tests. Nodes share a `FakeHub`. Messages travel as the same
//! signed bytes as on the real network and are opened with the same verifier, so a test of the
//! host or the engine also exercises signature rejection.

use super::gossip::{open, seal};
use crate::dungeons::TopicId;
use crate::engine::{Event, Message, NetError, Network, event_from_message};
use crate::identity::{PeerId, Signer};
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type Raw = (TopicId, Vec<u8>);

struct Node {
    subs: BTreeSet<TopicId>,
    tx: mpsc::UnboundedSender<Raw>,
}

#[derive(Default)]
struct HubState {
    nodes: HashMap<PeerId, Node>,
    rejected: usize,
}

/// The shared "network". Cheap to clone.
#[derive(Clone, Default)]
pub struct FakeHub {
    state: Arc<Mutex<HubState>>,
}

impl FakeHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// A new node that signs with `signer`.
    pub fn connect(&self, signer: Arc<dyn Signer>) -> FakeNet {
        let (tx, rx) = mpsc::unbounded_channel();
        let me = signer.peer_id();
        self.state.lock().unwrap().nodes.insert(
            me,
            Node {
                subs: BTreeSet::new(),
                tx,
            },
        );
        FakeNet {
            hub: self.clone(),
            me,
            signer,
            rx,
        }
    }

    /// Deliver raw bytes to every node subscribed to `topic`, as a hostile peer could.
    pub fn inject_raw(&self, topic: TopicId, bytes: Vec<u8>) {
        self.deliver(topic, bytes, None);
    }

    /// How many received messages failed verification and were dropped, over all nodes.
    pub fn rejected(&self) -> usize {
        self.state.lock().unwrap().rejected
    }

    /// Drop every node's channel: pending `next_event` calls end with `NetError::Closed`.
    pub fn shutdown(&self) {
        self.state.lock().unwrap().nodes.clear();
    }

    fn deliver(&self, topic: TopicId, bytes: Vec<u8>, except: Option<PeerId>) {
        let st = self.state.lock().unwrap();
        for (peer, node) in &st.nodes {
            if Some(*peer) != except && node.subs.contains(&topic) {
                let _ = node.tx.send((topic, bytes.clone()));
            }
        }
    }
}

pub struct FakeNet {
    hub: FakeHub,
    me: PeerId,
    signer: Arc<dyn Signer>,
    rx: mpsc::UnboundedReceiver<Raw>,
}

impl FakeNet {
    pub fn peer_id(&self) -> PeerId {
        self.me
    }

    fn joined(&self, topic: &TopicId) -> bool {
        self.hub
            .state
            .lock()
            .unwrap()
            .nodes
            .get(&self.me)
            .is_some_and(|n| n.subs.contains(topic))
    }
}

impl Network for FakeNet {
    async fn join(&mut self, topic: TopicId) -> Result<(), NetError> {
        let mut st = self.hub.state.lock().unwrap();
        let n = st.nodes.get_mut(&self.me).ok_or(NetError::Closed)?;
        n.subs.insert(topic);
        Ok(())
    }

    async fn leave(&mut self, topic: TopicId) -> Result<(), NetError> {
        let mut st = self.hub.state.lock().unwrap();
        let n = st.nodes.get_mut(&self.me).ok_or(NetError::Closed)?;
        n.subs.remove(&topic);
        Ok(())
    }

    async fn broadcast(&mut self, topic: TopicId, msg: Message) -> Result<(), NetError> {
        if !self.joined(&topic) {
            return Err(NetError::NotJoined);
        }
        let bytes = seal(&*self.signer, topic, None, &msg)?;
        self.hub.deliver(topic, bytes, Some(self.me));
        Ok(())
    }

    async fn send_direct(
        &mut self,
        peer: PeerId,
        topic: TopicId,
        msg: Message,
    ) -> Result<(), NetError> {
        if !self.joined(&topic) {
            return Err(NetError::NotJoined);
        }
        if !self.hub.state.lock().unwrap().nodes.contains_key(&peer) {
            return Err(NetError::UnknownPeer);
        }
        let bytes = seal(&*self.signer, topic, Some(peer), &msg)?;
        self.hub.deliver(topic, bytes, Some(self.me));
        Ok(())
    }

    async fn next_event(&mut self) -> Result<Event, NetError> {
        loop {
            let (topic, bytes) = self.rx.recv().await.ok_or(NetError::Closed)?;
            // Left the topic after the message was queued: drop it.
            if !self.joined(&topic) {
                continue;
            }
            match open(topic, &bytes) {
                Ok(v) if v.author != self.me && v.is_for(self.me) => {
                    return Ok(event_from_message(topic, v.msg));
                }
                Ok(_) => {}
                Err(_) => self.hub.state.lock().unwrap().rejected += 1,
            }
        }
    }

    async fn dial(&mut self, peer: PeerId) -> Result<(), NetError> {
        if self.hub.state.lock().unwrap().nodes.contains_key(&peer) {
            Ok(())
        } else {
            Err(NetError::UnknownPeer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::PartyId;
    use crate::identity::Identity;
    use std::time::Duration;

    const T: TopicId = TopicId([7; 32]);

    fn node(hub: &FakeHub) -> (FakeNet, Arc<Identity>) {
        let id = Arc::new(Identity::generate());
        (hub.connect(id.clone()), id)
    }

    fn confirm(id: &Identity) -> Message {
        Message::Confirm {
            peer: id.peer_id(),
            party_id: PartyId([1; 32]),
        }
    }

    async fn next(n: &mut FakeNet) -> Result<Event, NetError> {
        tokio::time::timeout(Duration::from_millis(200), n.next_event())
            .await
            .expect("no event in time")
    }

    async fn none_pending(n: &mut FakeNet) -> bool {
        tokio::time::timeout(Duration::from_millis(50), n.next_event())
            .await
            .is_err()
    }

    #[tokio::test]
    async fn broadcast_reaches_other_subscribers_only() {
        let hub = FakeHub::new();
        let (mut a, ida) = node(&hub);
        let (mut b, _) = node(&hub);
        let (mut c, _) = node(&hub);
        for n in [&mut a, &mut b] {
            n.join(T).await.unwrap();
        }
        a.broadcast(T, confirm(&ida)).await.unwrap();
        assert!(matches!(
            next(&mut b).await.unwrap(),
            Event::ConfirmReceived { from, .. } if from == ida.peer_id()
        ));
        assert!(none_pending(&mut a).await, "no echo to the sender");
        assert!(none_pending(&mut c).await, "c is not in the swarm");
    }

    #[tokio::test]
    async fn broadcast_needs_a_joined_topic_and_leave_stops_delivery() {
        let hub = FakeHub::new();
        let (mut a, ida) = node(&hub);
        let (mut b, _) = node(&hub);
        assert_eq!(
            a.broadcast(T, confirm(&ida)).await,
            Err(NetError::NotJoined)
        );
        a.join(T).await.unwrap();
        b.join(T).await.unwrap();
        b.leave(T).await.unwrap();
        a.broadcast(T, confirm(&ida)).await.unwrap();
        assert!(none_pending(&mut b).await);
    }

    #[tokio::test]
    async fn forged_and_replayed_messages_never_reach_the_engine() {
        let hub = FakeHub::new();
        let (mut a, ida) = node(&hub);
        let (mut b, _) = node(&hub);
        a.join(T).await.unwrap();
        b.join(T).await.unwrap();
        let other = TopicId([8; 32]);
        b.join(other).await.unwrap();

        // Garbage.
        hub.inject_raw(T, b"not an envelope".to_vec());
        // A flipped bit in a real message.
        let mut bytes = seal(&*ida, T, None, &confirm(&ida)).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        hub.inject_raw(T, bytes);
        // A valid message replayed into another topic.
        let good = seal(&*ida, T, None, &confirm(&ida)).unwrap();
        hub.inject_raw(other, good);
        assert!(none_pending(&mut b).await);
        assert_eq!(hub.rejected(), 3);

        // The real thing still gets through afterwards.
        a.broadcast(T, confirm(&ida)).await.unwrap();
        assert!(matches!(
            next(&mut b).await.unwrap(),
            Event::ConfirmReceived { .. }
        ));
    }

    #[tokio::test]
    async fn direct_messages_are_for_one_peer() {
        let hub = FakeHub::new();
        let (mut a, ida) = node(&hub);
        let (mut b, idb) = node(&hub);
        let (mut c, _) = node(&hub);
        for n in [&mut a, &mut b, &mut c] {
            n.join(T).await.unwrap();
        }
        a.send_direct(idb.peer_id(), T, confirm(&ida))
            .await
            .unwrap();
        assert!(matches!(
            next(&mut b).await.unwrap(),
            Event::ConfirmReceived { .. }
        ));
        assert!(
            none_pending(&mut c).await,
            "c verifies it, then drops it: not addressed to c"
        );
        assert_eq!(hub.rejected(), 0);
        assert_eq!(
            a.send_direct(PeerId([99; 32]), T, confirm(&ida)).await,
            Err(NetError::UnknownPeer)
        );
    }

    #[tokio::test]
    async fn dial_and_close() {
        let hub = FakeHub::new();
        let (mut a, _) = node(&hub);
        let (b, _) = node(&hub);
        assert_eq!(a.dial(b.peer_id()).await, Ok(()));
        assert_eq!(a.dial(PeerId([1; 32])).await, Err(NetError::UnknownPeer));
        hub.shutdown();
        assert_eq!(a.next_event().await, Err(NetError::Closed));
    }
}
