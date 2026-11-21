//! Deterministic party building of the converging-announcements mechanism (ADR 0010), which
//! ADR 0025 replaced. The engine no longer uses `build_parties`: it stays as the baseline the
//! simulation compares against and as the central greedy build for its upper bound. `Party` and
//! `party_id` are still the engine's.

use super::ad::{Ad, PartyId, Role, compatible, pair_stage};
use super::rank::order_key;
use crate::dungeons::TopicId;
use crate::identity::PeerId;
use sha2::{Digest, Sha256};

pub const PARTY_SIZE: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    pub id: PartyId,
    pub epoch: u64,
    /// Sorted ascending.
    pub members: [PeerId; PARTY_SIZE],
}

impl Party {
    pub fn contains(&self, p: &PeerId) -> bool {
        self.members.contains(p)
    }
}

/// `SHA-256(topic || epoch as 8 bytes big-endian || the five peer IDs sorted ascending)`.
pub fn party_id(topic: &TopicId, epoch: u64, members: &[PeerId; PARTY_SIZE]) -> PartyId {
    let mut sorted = *members;
    sorted.sort();
    let mut h = Sha256::new();
    h.update(topic.0);
    h.update(epoch.to_be_bytes());
    for m in &sorted {
        h.update(m.0);
    }
    PartyId(h.finalize().into())
}

#[derive(Default)]
struct Open<'a> {
    members: Vec<&'a Ad>,
    tank: u8,
    healer: u8,
    damage: u8,
}

impl Open<'_> {
    fn needs(&self, role: Role) -> bool {
        match role {
            Role::Tank => self.tank < 1,
            Role::Healer => self.healer < 1,
            Role::Damage => self.damage < 3,
        }
    }
    fn add(&mut self, ad: &Ad) {
        match ad.role {
            Role::Tank => self.tank += 1,
            Role::Healer => self.healer += 1,
            Role::Damage => self.damage += 1,
        }
    }
}

/// Build the complete parties from `candidates` for `epoch`.
///
/// Sort peers by `hash(epoch, peerId)`, walk the list, place each peer in the first open party
/// where it fills a needed role (1 tank, 1 healer, 3 damage) and is compatible with every
/// member at the stricter stage of each pair, otherwise open a new party with it. Only full
/// parties are returned. Secondary roles are not used: the code carries none in v1.
pub fn build_parties(topic: &TopicId, epoch: u64, candidates: &[&Ad], min_level: u8) -> Vec<Party> {
    let mut order: Vec<&Ad> = candidates.to_vec();
    order.sort_by_cached_key(|a| order_key(epoch, &a.peer));
    let mut open: Vec<Open> = Vec::new();
    for ad in order {
        let slot = open.iter().position(|p| {
            p.members.len() < PARTY_SIZE
                && p.needs(ad.role)
                && p.members
                    .iter()
                    .all(|m| compatible(m, ad, pair_stage(m, ad), min_level))
        });
        match slot {
            Some(i) => {
                open[i].add(ad);
                open[i].members.push(ad);
            }
            None => {
                let mut p = Open::default();
                p.add(ad);
                p.members.push(ad);
                open.push(p);
            }
        }
    }
    open.into_iter()
        .filter(|p| p.members.len() == PARTY_SIZE)
        .map(|p| {
            let mut members: [PeerId; PARTY_SIZE] = p
                .members
                .iter()
                .map(|a| a.peer)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            members.sort();
            Party {
                id: party_id(topic, epoch, &members),
                epoch,
                members,
            }
        })
        .collect()
}

/// The party containing `me`, if the build produced one.
pub fn party_for(parties: &[Party], me: &PeerId) -> Option<Party> {
    parties.iter().find(|p| p.contains(me)).cloned()
}

#[cfg(test)]
mod tests {
    use super::super::ad::test_ad;
    use super::*;

    fn topic() -> TopicId {
        TopicId([1; 32])
    }

    fn five() -> Vec<Ad> {
        vec![
            test_ad(1, Role::Tank, 24, 100),
            test_ad(2, Role::Healer, 24, 105),
            test_ad(3, Role::Damage, 25, 110),
            test_ad(4, Role::Damage, 25, 100),
            test_ad(5, Role::Damage, 26, 102),
        ]
    }

    #[test]
    fn worked_example_builds_one_party_of_exactly_these_five() {
        let ads = five();
        let refs: Vec<&Ad> = ads.iter().collect();
        let parties = build_parties(&topic(), 7, &refs, 1);
        assert_eq!(parties.len(), 1);
        let want: Vec<PeerId> = ads.iter().map(|a| a.peer).collect();
        assert_eq!(parties[0].members.to_vec(), want);
    }

    #[test]
    fn input_order_does_not_matter() {
        let ads = five();
        let mut refs: Vec<&Ad> = ads.iter().collect();
        let a = build_parties(&topic(), 7, &refs, 1);
        refs.reverse();
        let b = build_parties(&topic(), 7, &refs, 1);
        assert_eq!(a, b);
    }

    #[test]
    fn exactly_one_tank_one_healer_three_damage() {
        // Two tanks, two healers, six damage: two parties need 2 tanks 2 healers 6 damage,
        // so only two full parties of 1/1/3 can be made and one damage is left over.
        let mut ads = vec![];
        for i in 0..2 {
            ads.push(test_ad(10 + i, Role::Tank, 24, 0));
            ads.push(test_ad(20 + i, Role::Healer, 24, 0));
        }
        for i in 0..7 {
            ads.push(test_ad(30 + i, Role::Damage, 24, 0));
        }
        let refs: Vec<&Ad> = ads.iter().collect();
        let parties = build_parties(&topic(), 3, &refs, 1);
        assert_eq!(parties.len(), 2);
        for p in &parties {
            let roles: Vec<Role> = p
                .members
                .iter()
                .map(|m| ads.iter().find(|a| a.peer == *m).unwrap().role)
                .collect();
            assert_eq!(roles.iter().filter(|r| **r == Role::Tank).count(), 1);
            assert_eq!(roles.iter().filter(|r| **r == Role::Healer).count(), 1);
            assert_eq!(roles.iter().filter(|r| **r == Role::Damage).count(), 3);
        }
        // Parties are disjoint.
        assert!(parties[0].members.iter().all(|m| !parties[1].contains(m)));
    }

    #[test]
    fn missing_role_gives_no_party() {
        let mut ads = five();
        ads[1].role = Role::Damage; // no healer
        let refs: Vec<&Ad> = ads.iter().collect();
        assert!(build_parties(&topic(), 7, &refs, 1).is_empty());
    }

    #[test]
    fn incompatible_peers_are_left_out() {
        let mut ads = five();
        ads.push(test_ad(6, Role::Damage, 40, 100)); // level too far
        let refs: Vec<&Ad> = ads.iter().collect();
        let parties = build_parties(&topic(), 7, &refs, 1);
        assert_eq!(parties.len(), 1);
        assert!(!parties[0].contains(&PeerId([6; 32])));
    }

    #[test]
    fn party_id_is_deterministic_and_order_independent() {
        let m: [PeerId; 5] = std::array::from_fn(|i| PeerId([i as u8 + 1; 32]));
        let mut r = m;
        r.reverse();
        assert_eq!(party_id(&topic(), 9, &m), party_id(&topic(), 9, &r));
        assert_ne!(party_id(&topic(), 9, &m), party_id(&topic(), 10, &m));
        assert_ne!(
            party_id(&TopicId([2; 32]), 9, &m),
            party_id(&topic(), 9, &m)
        );
        // Spec: sha256(topic || epoch be8 || sorted ids)
        let mut input = vec![1u8; 32];
        input.extend_from_slice(&9u64.to_be_bytes());
        for i in 1..=5u8 {
            input.extend_from_slice(&[i; 32]);
        }
        assert_eq!(
            party_id(&topic(), 9, &m).0.to_vec(),
            Sha256::digest(&input).to_vec()
        );
    }

    #[test]
    fn stricter_stage_of_a_pair_applies() {
        // Levels 20 and 26 are fine at stage 3 but not at stage 0: if one of the pair is still
        // at stage 0 they do not match.
        let mut ads = five();
        ads[4] = test_ad(5, Role::Damage, 30, 100);
        for a in ads.iter_mut() {
            a.stage = 3;
        }
        let refs: Vec<&Ad> = ads.iter().collect();
        assert_eq!(build_parties(&topic(), 7, &refs, 1).len(), 1);
        ads[4].stage = 0;
        let refs: Vec<&Ad> = ads.iter().collect();
        assert!(build_parties(&topic(), 7, &refs, 1).is_empty());
    }
}
