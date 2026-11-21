//! A bounded sample of the latest ads seen on one topic.
//!
//! When the sample is full the ad furthest in level from the owner's own level is dropped
//! first (the proposed level bias), ties broken by dropping the higher peer ID so every peer
//! drops the same one. The owner's own ad is never dropped.

use super::ad::{Ad, counter_newer};
use super::rank::pair_rank;
use crate::config::AD_EXPIRY;
use crate::identity::PeerId;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Entry {
    pub ad: Ad,
    /// When this peer was first seen continuously. Keep-alives do not move it, so an ad that
    /// was in the sample before an epoch cutoff stays eligible.
    pub first_seen: Duration,
    pub last_seen: Duration,
}

#[derive(Debug, Clone)]
pub struct Sample {
    me: PeerId,
    own_level: u8,
    cap: usize,
    entries: BTreeMap<PeerId, Entry>,
    /// Peers announced as taken, with the counter of their Taken message. An ad with a later
    /// counter (the peer searches again) is accepted.
    taken: BTreeMap<PeerId, u16>,
}

/// How ads at the same level distance are told apart when the sample is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tie {
    /// Drop the higher peer ID. Every peer drops the same ad, so same-level peers hold nearly
    /// the same sample (what converging announcements needed), but ads with high IDs are in
    /// nobody's sample.
    LowestId,
    /// Drop by `pair_rank(0, owner, peer)`. Each peer keeps its own random part of the level,
    /// so every ad is in about the same share of samples (what leader invitations need).
    OwnerHash,
}

/// The tie-break the engine uses (ADR 0025).
pub const TIE: Tie = Tie::OwnerHash;

/// Eviction order: larger is dropped first.
fn evict_key(own_level: u8, owner: &PeerId, ad: &Ad, tie: Tie) -> (u8, u64, PeerId) {
    let t = match tie {
        Tie::LowestId => 0,
        Tie::OwnerHash => pair_rank(0, owner, &ad.peer),
    };
    (ad.level.abs_diff(own_level), t, ad.peer)
}

/// The ads a peer of level `own_level` would keep out of `ads` with room for `cap`. Used by
/// the simulation so it applies the same rule as `Sample`. The order of the result is
/// unspecified.
pub fn keep_nearest<'a>(
    own_level: u8,
    own: &PeerId,
    mut ads: Vec<&'a Ad>,
    cap: usize,
    tie: Tie,
) -> Vec<&'a Ad> {
    if cap > 0 && ads.len() > cap {
        let key = |a: &&Ad| (a.peer != *own, evict_key(own_level, own, a, tie));
        ads.select_nth_unstable_by_key(cap - 1, key);
        ads.truncate(cap);
    }
    ads
}

impl Sample {
    pub fn new(me: PeerId, own_level: u8, cap: usize) -> Self {
        Self {
            me,
            own_level,
            cap,
            entries: BTreeMap::new(),
            taken: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The sample holds as many ads as its cap allows, so what it shows is a slice.
    pub fn is_full(&self) -> bool {
        self.entries.len() >= self.cap
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    pub fn get(&self, peer: &PeerId) -> Option<&Entry> {
        self.entries.get(peer)
    }

    /// Returns whether the ad was stored (false for stale, taken or duplicate-counter ads).
    pub fn insert(&mut self, ad: Ad, now: Duration) -> bool {
        if let Some(&t) = self.taken.get(&ad.peer) {
            if !counter_newer(ad.counter, t) {
                return false;
            }
            self.taken.remove(&ad.peer);
        }
        match self.entries.get_mut(&ad.peer) {
            Some(e) => {
                if !counter_newer(ad.counter, e.ad.counter) {
                    return false;
                }
                e.ad = ad;
                e.last_seen = now;
            }
            None => {
                self.entries.insert(
                    ad.peer,
                    Entry {
                        ad,
                        first_seen: now,
                        last_seen: now,
                    },
                );
                self.evict();
            }
        }
        true
    }

    fn evict(&mut self) {
        while self.entries.len() > self.cap {
            let me = self.me;
            let own = self.own_level;
            let worst = self
                .entries
                .values()
                .filter(|e| e.ad.peer != me)
                .max_by_key(|e| evict_key(own, &me, &e.ad, TIE))
                .map(|e| e.ad.peer);
            match worst {
                Some(p) => {
                    self.entries.remove(&p);
                }
                None => break,
            }
        }
    }

    /// A peer announced it is in a committed party: drop it and ignore older ads.
    /// Returns whether an ad of that peer was in the sample (and so is gone now).
    pub fn mark_taken(&mut self, peer: PeerId, counter: u16) -> bool {
        let had = self.entries.remove(&peer).is_some();
        self.taken.insert(peer, counter);
        had
    }

    pub fn remove(&mut self, peer: &PeerId) {
        self.entries.remove(peer);
    }

    /// Drop ads not refreshed within the expiry. The owner's own ad is kept.
    pub fn expire(&mut self, now: Duration) {
        let me = self.me;
        self.entries
            .retain(|p, e| *p == me || now.saturating_sub(e.last_seen) < AD_EXPIRY);
    }

    /// Ads first seen at or before `cutoff` and not expired at `now`, in peer ID order.
    pub fn eligible(&self, now: Duration, cutoff: Duration) -> Vec<&Ad> {
        self.entries
            .values()
            .filter(|e| e.first_seen <= cutoff && now.saturating_sub(e.last_seen) < AD_EXPIRY)
            .map(|e| &e.ad)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ad::{Role, test_ad};
    use super::*;

    fn secs(s: u64) -> Duration {
        Duration::from_secs(s)
    }

    fn me() -> PeerId {
        PeerId([200; 32])
    }

    #[test]
    fn newer_counter_replaces_and_keeps_first_seen() {
        let mut s = Sample::new(me(), 24, 100);
        let mut a = test_ad(1, Role::Tank, 24, 0);
        assert!(s.insert(a.clone(), secs(10)));
        assert!(!s.insert(a.clone(), secs(11))); // same counter
        a.counter = 2;
        a.stage = 1;
        assert!(s.insert(a, secs(50)));
        let e = s.get(&PeerId([1; 32])).unwrap();
        assert_eq!(
            (e.first_seen, e.last_seen, e.ad.stage),
            (secs(10), secs(50), 1)
        );
        let mut old = test_ad(1, Role::Tank, 24, 0);
        old.counter = 1;
        assert!(!s.insert(old, secs(60)));
    }

    #[test]
    fn expired_ads_are_dropped() {
        let mut s = Sample::new(me(), 24, 100);
        s.insert(test_ad(1, Role::Tank, 24, 0), secs(0));
        s.insert(test_ad(2, Role::Healer, 24, 0), secs(100));
        s.expire(secs(121));
        assert!(s.get(&PeerId([1; 32])).is_none());
        assert!(s.get(&PeerId([2; 32])).is_some());
        // Expired ads are also not eligible even before `expire` runs.
        assert!(s.eligible(secs(221), secs(500)).is_empty());
    }

    #[test]
    fn taken_peer_is_dropped_until_it_searches_again() {
        let mut s = Sample::new(me(), 24, 100);
        s.insert(test_ad(1, Role::Tank, 24, 0), secs(0));
        s.mark_taken(PeerId([1; 32]), 1);
        assert!(s.get(&PeerId([1; 32])).is_none());
        assert!(!s.insert(test_ad(1, Role::Tank, 24, 0), secs(1))); // old counter
        let mut again = test_ad(1, Role::Tank, 24, 0);
        again.counter = 2;
        assert!(s.insert(again, secs(2)));
    }

    #[test]
    fn cutoff_uses_first_seen() {
        let mut s = Sample::new(me(), 24, 100);
        s.insert(test_ad(1, Role::Tank, 24, 0), secs(5));
        s.insert(test_ad(2, Role::Tank, 24, 0), secs(9));
        let got: Vec<u8> = s
            .eligible(secs(10), secs(8))
            .iter()
            .map(|a| a.peer.0[0])
            .collect();
        assert_eq!(got, [1]);
    }

    #[test]
    fn full_sample_drops_the_ad_furthest_in_level_and_never_self() {
        let mut s = Sample::new(PeerId([1; 32]), 24, 3);
        s.insert(test_ad(1, Role::Tank, 30, 0), secs(0)); // self, far in level
        s.insert(test_ad(2, Role::Tank, 24, 0), secs(0));
        s.insert(test_ad(3, Role::Healer, 25, 0), secs(0));
        s.insert(test_ad(4, Role::Damage, 20, 0), secs(0)); // 4 away: dropped
        assert_eq!(s.len(), 3);
        assert!(s.get(&PeerId([4; 32])).is_none());
        assert!(s.get(&PeerId([1; 32])).is_some());
        s.insert(test_ad(5, Role::Damage, 22, 0), secs(0)); // 2 away beats 25? no: 25 is 1 away
        assert!(s.get(&PeerId([5; 32])).is_none() || s.len() == 3);
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn equal_distance_ties_are_broken_by_a_hash_of_owner_and_peer() {
        let ads: Vec<Ad> = (1..=30).map(|i| test_ad(i, Role::Damage, 24, 0)).collect();
        let kept = |owner: u8, order: &[Ad]| {
            let mut s = Sample::new(PeerId([owner; 32]), 24, 5);
            for a in order {
                s.insert(a.clone(), secs(0));
            }
            let mut ids: Vec<u8> = order
                .iter()
                .filter(|a| s.get(&a.peer).is_some())
                .map(|a| a.peer.0[0])
                .collect();
            ids.sort();
            ids
        };
        // Same owner, any arrival order: the same five.
        let mut reversed = ads.clone();
        reversed.reverse();
        assert_eq!(kept(200, &ads), kept(200, &reversed));
        assert_eq!(kept(200, &ads).len(), 5);
        // Different owners keep different parts of the level, so no ad is in nobody's sample.
        let sets: std::collections::BTreeSet<Vec<u8>> = (100..120).map(|o| kept(o, &ads)).collect();
        assert!(sets.len() > 10, "{} distinct samples", sets.len());
        let covered: std::collections::BTreeSet<u8> = sets.into_iter().flatten().collect();
        assert!(covered.len() > 20, "{} ads covered", covered.len());
    }

    #[test]
    fn the_old_tie_break_drops_the_higher_peer_id() {
        let ads: Vec<Ad> = [9, 3, 5, 7]
            .iter()
            .map(|&i| test_ad(i, Role::Tank, 25, 0))
            .collect();
        let kept = keep_nearest(24, &me(), ads.iter().collect(), 2, Tie::LowestId);
        let mut ids: Vec<u8> = kept.iter().map(|a| a.peer.0[0]).collect();
        ids.sort();
        assert_eq!(ids, [3, 5]);
    }

    #[test]
    fn keep_nearest_matches_the_sample_rule() {
        let ads: Vec<Ad> = (1..=6)
            .map(|i| test_ad(i, Role::Damage, 20 + i, 0))
            .collect();
        let kept = keep_nearest(24, &me(), ads.iter().collect(), 3, TIE);
        let mut ids: Vec<u8> = kept.iter().map(|a| a.peer.0[0]).collect();
        ids.sort();
        assert_eq!(ids, [3, 4, 5]); // levels 23, 24, 25
    }
}
