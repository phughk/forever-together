//! An in-memory `Network`: records what the engine asks of the network and yields only the
//! events a test injects. It is also what the desktop app runs on until the iroh `net` module
//! is plugged in (see `runtime::build_network`), so a search starts, widens and times out, it
//! just never finds anyone.

use crate::dungeons::TopicId;
use crate::engine::{Event, Message, NetError, Network};
use crate::identity::PeerId;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetCall {
    Join(TopicId),
    Leave(TopicId),
    Broadcast(TopicId, Message),
    Direct(PeerId, TopicId, Message),
    Dial(PeerId),
}

/// Shared view of a `MemNetwork`: what it was asked to do, and a way to feed it events.
#[derive(Clone)]
pub struct MemNetHandle {
    pub calls: Arc<Mutex<Vec<NetCall>>>,
    pub inject: mpsc::UnboundedSender<Event>,
}

impl MemNetHandle {
    pub fn calls(&self) -> Vec<NetCall> {
        self.calls.lock().unwrap().clone()
    }
}

pub struct MemNetwork {
    calls: Arc<Mutex<Vec<NetCall>>>,
    inbox: mpsc::UnboundedReceiver<Event>,
    // Kept so the inbox never closes: `next_event` then waits forever, as a quiet network does.
    _keep_open: mpsc::UnboundedSender<Event>,
}

impl MemNetwork {
    pub fn new() -> (Self, MemNetHandle) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                calls: calls.clone(),
                inbox: rx,
                _keep_open: tx.clone(),
            },
            MemNetHandle { calls, inject: tx },
        )
    }

    fn log(&self, c: NetCall) {
        self.calls.lock().unwrap().push(c);
    }
}

impl Network for MemNetwork {
    async fn join(&mut self, topic: TopicId) -> Result<(), NetError> {
        self.log(NetCall::Join(topic));
        Ok(())
    }
    async fn leave(&mut self, topic: TopicId) -> Result<(), NetError> {
        self.log(NetCall::Leave(topic));
        Ok(())
    }
    async fn broadcast(&mut self, topic: TopicId, msg: Message) -> Result<(), NetError> {
        self.log(NetCall::Broadcast(topic, msg));
        Ok(())
    }
    async fn send_direct(
        &mut self,
        peer: PeerId,
        topic: TopicId,
        msg: Message,
    ) -> Result<(), NetError> {
        self.log(NetCall::Direct(peer, topic, msg));
        Ok(())
    }
    /// Cancel-safe: it only awaits a channel receive.
    async fn next_event(&mut self) -> Result<Event, NetError> {
        match self.inbox.recv().await {
            Some(e) => Ok(e),
            None => Err(NetError::Closed),
        }
    }
    async fn dial(&mut self, peer: PeerId) -> Result<(), NetError> {
        self.log(NetCall::Dial(peer));
        Ok(())
    }
}
