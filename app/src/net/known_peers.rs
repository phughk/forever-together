//! The known-peers table (ADR 0009): a least-recently-used cache of peers with the topics they
//! claim, persisted through the store as `known_peers/<peerId hex>` = `{ topics, lastSeen,
//! lastUsed }`.
//!
//! * Only **verified** peers (ones we dialed, or that are gossip neighbours) are in the table
//!   proper. They count against the cache size, are persisted, are dialed first at bootstrap and
//!   are handed out by peer exchange.
//! * A peer learned from someone else's claim (peer exchange, `Hello`) is **pending** until it
//!   is dialed. Pending entries do not count toward the size, are not persisted, are not served
//!   and are not remembered peers. They are kept in a small bounded queue so they can be
//!   dialed.
//! * Eviction is least recently used, but the player's active topics keep reserved slots: for
//!   each active topic the `RESERVED_PER_TOPIC` most recently used verified peers that claim it
//!   are evicted last.
//!
//! Time is unix seconds, passed in by the caller.

use crate::dungeons::TopicId;
use crate::identity::PeerId;
use crate::store::{PREFIX_KNOWN_PEERS, Store, StoreError, get_json, known_peer_key, put_json};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

/// Slots protected from eviction per active topic.
pub const RESERVED_PER_TOPIC: usize = 2;
/// Topics kept per peer (the wire allows 20 per entry).
pub const MAX_TOPICS_PER_PEER: usize = 20;

/// The stored record (spec section 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub topics: Vec<TopicId>,
    pub last_seen: u64,
    pub last_used: u64,
}

pub struct KnownPeers {
    cap: usize,
    store: Arc<dyn Store>,
    verified: HashMap<PeerId, Record>,
    /// Oldest first.
    pending: Vec<(PeerId, Record)>,
    active: BTreeSet<TopicId>,
}

fn merge_topics(into: &mut Vec<TopicId>, more: &[TopicId]) {
    for t in more {
        if !into.contains(t) {
            into.push(*t);
        }
    }
    into.truncate(MAX_TOPICS_PER_PEER);
}

impl KnownPeers {
    /// An empty table of at most `cap` verified peers (at least 1).
    pub fn new(cap: usize, store: Arc<dyn Store>) -> Self {
        Self {
            cap: cap.max(1),
            store,
            verified: HashMap::new(),
            pending: Vec::new(),
            active: BTreeSet::new(),
        }
    }

    /// Load the persisted peers. A record that cannot be read is skipped and deleted, so one
    /// bad record never blocks a start. If there are more than `cap`, the least recently used
    /// are dropped.
    pub fn load(cap: usize, store: Arc<dyn Store>) -> Result<Self, StoreError> {
        let mut me = Self::new(cap, store.clone());
        for (key, _) in store.list(PREFIX_KNOWN_PEERS)? {
            let peer = key
                .strip_prefix(PREFIX_KNOWN_PEERS)
                .and_then(PeerId::from_hex);
            let rec = get_json::<Record>(&*store, &key).ok().flatten();
            match (peer, rec) {
                (Some(p), Some(r)) => {
                    me.verified.insert(p, r);
                }
                _ => {
                    tracing::warn!(%key, "dropping unreadable known-peer record");
                    store.delete(&key)?;
                }
            }
        }
        while me.verified.len() > me.cap {
            me.evict_one()?;
        }
        Ok(me)
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    /// Verified peers only.
    pub fn len(&self) -> usize {
        self.verified.len()
    }

    pub fn is_empty(&self) -> bool {
        self.verified.is_empty()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_verified(&self, peer: &PeerId) -> bool {
        self.verified.contains_key(peer)
    }

    pub fn get(&self, peer: &PeerId) -> Option<&Record> {
        self.verified.get(peer)
    }

    /// The topics the player searches now. These get reserved slots.
    pub fn set_active_topics(&mut self, topics: impl IntoIterator<Item = TopicId>) {
        self.active = topics.into_iter().collect();
    }

    fn persist(&self, peer: &PeerId) -> Result<(), StoreError> {
        match self.verified.get(peer) {
            Some(r) => put_json(&*self.store, &known_peer_key(&peer.to_hex()), r),
            None => Ok(()),
        }
    }

    /// Record a claim about `peer`. For a verified peer this refreshes `last_seen` and adds the
    /// topics. Otherwise the peer is queued as pending.
    pub fn observe(
        &mut self,
        peer: PeerId,
        topics: &[TopicId],
        now: u64,
    ) -> Result<(), StoreError> {
        if let Some(r) = self.verified.get_mut(&peer) {
            merge_topics(&mut r.topics, topics);
            r.last_seen = now;
            return self.persist(&peer);
        }
        if let Some(i) = self.pending.iter().position(|(p, _)| *p == peer) {
            let (_, r) = &mut self.pending[i];
            merge_topics(&mut r.topics, topics);
            r.last_seen = now;
            return Ok(());
        }
        let mut rec = Record {
            topics: Vec::new(),
            last_seen: now,
            last_used: 0,
        };
        merge_topics(&mut rec.topics, topics);
        self.pending.push((peer, rec));
        while self.pending.len() > self.cap {
            self.pending.remove(0);
        }
        Ok(())
    }

    /// The peer was dialed (or is a live gossip neighbour). It joins the table, counts against
    /// the size and is persisted. May evict another peer.
    pub fn mark_verified(
        &mut self,
        peer: PeerId,
        topics: &[TopicId],
        now: u64,
    ) -> Result<(), StoreError> {
        let mut rec = match self.pending.iter().position(|(p, _)| *p == peer) {
            Some(i) => self.pending.remove(i).1,
            None => Record {
                topics: Vec::new(),
                last_seen: now,
                last_used: now,
            },
        };
        if let Some(old) = self.verified.remove(&peer) {
            merge_topics(&mut rec.topics, &old.topics);
        }
        merge_topics(&mut rec.topics, topics);
        rec.last_seen = now;
        rec.last_used = now;
        self.verified.insert(peer, rec);
        self.persist(&peer)?;
        while self.verified.len() > self.cap {
            self.evict_one()?;
        }
        Ok(())
    }

    /// The peer was used (dialed, or asked for peers). Makes it more recent in the LRU order.
    pub fn touch_used(&mut self, peer: &PeerId, now: u64) -> Result<(), StoreError> {
        if let Some(r) = self.verified.get_mut(peer) {
            r.last_used = now;
            return self.persist(peer);
        }
        Ok(())
    }

    pub fn remove(&mut self, peer: &PeerId) -> Result<(), StoreError> {
        self.pending.retain(|(p, _)| p != peer);
        if self.verified.remove(peer).is_some() {
            self.store.delete(&known_peer_key(&peer.to_hex()))?;
        }
        Ok(())
    }

    /// Verified peers that claim any of `topics`, most recently used first.
    pub fn remembered(&self, topics: &[TopicId]) -> Vec<PeerId> {
        let mut v: Vec<(&PeerId, &Record)> = self
            .verified
            .iter()
            .filter(|(_, r)| r.topics.iter().any(|t| topics.contains(t)))
            .collect();
        v.sort_by(|a, b| b.1.last_used.cmp(&a.1.last_used).then(a.0.cmp(b.0)));
        v.into_iter().map(|(p, _)| *p).collect()
    }

    /// Pending peers that claim any of `topics`, newest claim first. These are worth a dial.
    pub fn pending_for(&self, topics: &[TopicId]) -> Vec<PeerId> {
        let mut v: Vec<&(PeerId, Record)> = self
            .pending
            .iter()
            .filter(|(_, r)| r.topics.iter().any(|t| topics.contains(t)))
            .collect();
        v.sort_by(|a, b| b.1.last_seen.cmp(&a.1.last_seen).then(a.0.cmp(&b.0)));
        v.into_iter().map(|(p, _)| *p).collect()
    }

    /// Entries to hand to a requester: verified peers that claim any of `topics`, most
    /// recently seen first, at most `max`, never the requester itself.
    pub fn sample(
        &self,
        topics: &[TopicId],
        max: usize,
        exclude: &PeerId,
    ) -> Vec<(PeerId, Record)> {
        let mut v: Vec<(PeerId, Record)> = self
            .verified
            .iter()
            .filter(|(p, r)| *p != exclude && r.topics.iter().any(|t| topics.contains(t)))
            .map(|(p, r)| (*p, r.clone()))
            .collect();
        v.sort_by(|a, b| b.1.last_seen.cmp(&a.1.last_seen).then(a.0.cmp(&b.0)));
        v.truncate(max);
        v
    }

    /// Peers protected from eviction: for each active topic, the most recently used peers that
    /// claim it, up to the reserved slots.
    fn protected(&self) -> BTreeSet<PeerId> {
        let mut out = BTreeSet::new();
        for t in &self.active {
            let mut v: Vec<(&PeerId, &Record)> = self
                .verified
                .iter()
                .filter(|(_, r)| r.topics.contains(t))
                .collect();
            v.sort_by(|a, b| b.1.last_used.cmp(&a.1.last_used).then(a.0.cmp(b.0)));
            out.extend(v.into_iter().take(RESERVED_PER_TOPIC).map(|(p, _)| *p));
        }
        out
    }

    fn evict_one(&mut self) -> Result<(), StoreError> {
        let protected = self.protected();
        let pick = |only_unprotected: bool| {
            self.verified
                .iter()
                .filter(|(p, _)| !only_unprotected || !protected.contains(p))
                .min_by(|a, b| {
                    (a.1.last_used, a.1.last_seen, a.0).cmp(&(b.1.last_used, b.1.last_seen, b.0))
                })
                .map(|(p, _)| *p)
        };
        // If every peer is protected (more reserved slots than the cache holds), plain LRU.
        if let Some(victim) = pick(true).or_else(|| pick(false)) {
            self.remove(&victim)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemStore;

    fn p(n: u8) -> PeerId {
        PeerId([n; 32])
    }
    fn t(n: u8) -> TopicId {
        TopicId([n; 32])
    }
    fn mem() -> Arc<MemStore> {
        Arc::new(MemStore::new())
    }

    #[test]
    fn evicts_least_recently_used() {
        let mut k = KnownPeers::new(3, mem());
        for i in 1..=3u8 {
            k.mark_verified(p(i), &[t(1)], 100 + i as u64).unwrap();
        }
        // Use peer 1, so peer 2 is now the oldest.
        k.touch_used(&p(1), 200).unwrap();
        k.mark_verified(p(4), &[t(1)], 300).unwrap();
        assert_eq!(k.len(), 3);
        assert!(!k.is_verified(&p(2)));
        assert!(k.is_verified(&p(1)) && k.is_verified(&p(3)) && k.is_verified(&p(4)));
    }

    #[test]
    fn active_topics_keep_reserved_slots_before_lru() {
        let mut k = KnownPeers::new(4, mem());
        k.set_active_topics([t(9)]);
        // Two old peers for the active topic, two newer ones for another topic.
        k.mark_verified(p(1), &[t(9)], 10).unwrap();
        k.mark_verified(p(2), &[t(9)], 11).unwrap();
        k.mark_verified(p(3), &[t(5)], 50).unwrap();
        k.mark_verified(p(4), &[t(5)], 51).unwrap();
        // Plain LRU would drop peer 1. The reserve keeps both active-topic peers.
        k.mark_verified(p(5), &[t(5)], 52).unwrap();
        assert!(k.is_verified(&p(1)) && k.is_verified(&p(2)));
        assert!(!k.is_verified(&p(3)), "oldest unreserved peer goes first");
        k.mark_verified(p(6), &[t(5)], 53).unwrap();
        assert!(k.is_verified(&p(1)) && k.is_verified(&p(2)));
        assert_eq!(k.len(), 4);
    }

    #[test]
    fn reserve_is_per_topic_and_limited_to_the_reserved_count() {
        let mut k = KnownPeers::new(3, mem());
        k.set_active_topics([t(9)]);
        // Three peers claim the active topic: only the two most recently used are reserved.
        for i in 1..=3u8 {
            k.mark_verified(p(i), &[t(9)], i as u64).unwrap();
        }
        k.mark_verified(p(4), &[t(5)], 100).unwrap();
        assert!(
            !k.is_verified(&p(1)),
            "third active-topic peer is evictable"
        );
        assert!(k.is_verified(&p(2)) && k.is_verified(&p(3)) && k.is_verified(&p(4)));
    }

    #[test]
    fn falls_back_to_lru_when_everything_is_reserved() {
        let mut k = KnownPeers::new(1, mem());
        k.set_active_topics([t(1)]);
        k.mark_verified(p(1), &[t(1)], 1).unwrap();
        k.mark_verified(p(2), &[t(1)], 2).unwrap();
        assert_eq!(k.len(), 1);
        assert!(k.is_verified(&p(2)));
    }

    #[test]
    fn persists_across_a_restart_with_eviction_and_removal_applied() {
        let store = mem();
        let mut k = KnownPeers::new(2, store.clone());
        k.mark_verified(p(1), &[t(1)], 10).unwrap();
        k.mark_verified(p(2), &[t(2)], 20).unwrap();
        k.mark_verified(p(3), &[t(3)], 30).unwrap(); // evicts 1
        k.touch_used(&p(2), 40).unwrap();
        drop(k);
        // Stored layout: known_peers/<hex>.
        let keys: Vec<String> = store
            .list(PREFIX_KNOWN_PEERS)
            .unwrap()
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(
            keys,
            vec![
                known_peer_key(&p(2).to_hex()),
                known_peer_key(&p(3).to_hex())
            ]
        );
        let k = KnownPeers::load(2, store.clone()).unwrap();
        assert_eq!(k.len(), 2);
        assert!(!k.is_verified(&p(1)));
        assert_eq!(
            k.get(&p(2)),
            Some(&Record {
                topics: vec![t(2)],
                last_seen: 20,
                last_used: 40
            })
        );
        // A smaller cache on the next start drops the least recently used.
        let k = KnownPeers::load(1, store.clone()).unwrap();
        assert_eq!(k.len(), 1);
        assert!(k.is_verified(&p(2)));
        assert_eq!(store.list(PREFIX_KNOWN_PEERS).unwrap().len(), 1);
    }

    #[test]
    fn persists_across_a_restart_on_varaq() {
        let d = std::env::temp_dir().join(format!("wowft-known-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        {
            let store = Arc::new(crate::store::VaraqStore::open(&d).unwrap());
            let mut k = KnownPeers::new(2, store);
            k.mark_verified(p(1), &[t(1)], 10).unwrap();
            k.mark_verified(p(2), &[t(2)], 20).unwrap();
            k.mark_verified(p(3), &[t(3)], 30).unwrap(); // evicts 1
        }
        let store = Arc::new(crate::store::VaraqStore::open(&d).unwrap());
        let k = KnownPeers::load(2, store).unwrap();
        assert_eq!(k.len(), 2);
        assert!(!k.is_verified(&p(1)) && k.is_verified(&p(2)) && k.is_verified(&p(3)));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn unreadable_record_is_dropped_not_fatal() {
        let store = mem();
        store.put("known_peers/zz", b"{}").unwrap();
        store
            .put(&known_peer_key(&p(1).to_hex()), b"not json")
            .unwrap();
        let k = KnownPeers::load(10, store.clone()).unwrap();
        assert!(k.is_empty());
        assert!(store.list(PREFIX_KNOWN_PEERS).unwrap().is_empty());
    }

    #[test]
    fn unverified_entries_do_not_count_until_dialed() {
        let store = mem();
        let mut k = KnownPeers::new(2, store.clone());
        for i in 1..=5u8 {
            k.observe(p(i), &[t(1)], i as u64).unwrap();
        }
        // They do not fill the cache, are not remembered, not served and not stored.
        assert_eq!(k.len(), 0);
        assert!(k.remembered(&[t(1)]).is_empty());
        assert!(k.sample(&[t(1)], 20, &p(99)).is_empty());
        assert!(store.list(PREFIX_KNOWN_PEERS).unwrap().is_empty());
        // The pending queue is bounded and drops the oldest.
        assert_eq!(k.pending_len(), 2);
        assert_eq!(k.pending_for(&[t(1)]), vec![p(5), p(4)]);
        // Dialing makes one count.
        k.mark_verified(p(5), &[], 50).unwrap();
        assert_eq!(k.len(), 1);
        assert_eq!(k.pending_len(), 1);
        assert_eq!(k.remembered(&[t(1)]), vec![p(5)]);
        assert_eq!(store.list(PREFIX_KNOWN_PEERS).unwrap().len(), 1);
    }

    #[test]
    fn observing_a_verified_peer_updates_it() {
        let mut k = KnownPeers::new(5, mem());
        k.mark_verified(p(1), &[t(1)], 10).unwrap();
        k.observe(p(1), &[t(2), t(1)], 20).unwrap();
        let r = k.get(&p(1)).unwrap();
        assert_eq!(r.topics, vec![t(1), t(2)]);
        assert_eq!((r.last_seen, r.last_used), (20, 10));
        assert_eq!(k.pending_len(), 0);
    }

    #[test]
    fn topics_per_peer_are_capped() {
        let mut k = KnownPeers::new(5, mem());
        let many: Vec<TopicId> = (0..40u8).map(t).collect();
        k.mark_verified(p(1), &many, 1).unwrap();
        assert_eq!(k.get(&p(1)).unwrap().topics.len(), MAX_TOPICS_PER_PEER);
    }

    #[test]
    fn remembered_orders_by_last_use_and_sample_excludes_requester() {
        let mut k = KnownPeers::new(5, mem());
        k.mark_verified(p(1), &[t(1)], 10).unwrap();
        k.mark_verified(p(2), &[t(1)], 20).unwrap();
        k.mark_verified(p(3), &[t(2)], 30).unwrap();
        assert_eq!(k.remembered(&[t(1)]), vec![p(2), p(1)]);
        assert_eq!(k.remembered(&[t(1), t(2)]), vec![p(3), p(2), p(1)]);
        let s: Vec<PeerId> = k
            .sample(&[t(1)], 20, &p(2))
            .into_iter()
            .map(|(p, _)| p)
            .collect();
        assert_eq!(s, vec![p(1)]);
        assert_eq!(k.sample(&[t(1)], 1, &p(9)).len(), 1);
    }
}
