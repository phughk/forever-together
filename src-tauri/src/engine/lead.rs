//! Leader invitations (ADR 0025): how a tank builds a party by inviting from its own sample.
//!
//! Pure functions shared by the state machine and the simulation. No clock, no I/O.
//!
//! * `plan_invites`: the leader (the party's tank) picks one healer and three damage peers it is
//!   compatible with, plus a few spares, from its own sample. Samples differ between peers and
//!   that does not matter: nobody else has to build the same party.
//! * `invite_preference`: an invitee that got several invitations accepts the one from the
//!   leader with the lowest `rank(epoch, leader)`. Every invitee uses the same order, so the
//!   best-ranked leader always gets everyone it asked for.
//! * `choose_party`: the leader completes a party from the peers that accepted.

use super::ad::{Ad, Role, compatible, pair_stage};
use super::party::{PARTY_SIZE, Party, party_id};
use super::rank::{pair_rank, rank};
use crate::dungeons::TopicId;
use crate::identity::PeerId;

/// Healers invited beyond the one needed.
pub const SPARE_HEALERS: usize = 2;
/// Damage peers invited beyond the three needed.
pub const SPARE_DAMAGE: usize = 5;
/// Compatible candidates of each role the search looks at, in the leader's own order.
const SEARCH_HEALERS: usize = 8;
const SEARCH_DAMAGE: usize = 14;

fn ok(a: &Ad, b: &Ad, min_level: u8) -> bool {
    compatible(a, b, pair_stage(a, b), min_level)
}

/// A healer and three damage peers from `healers` and `damage` (already compatible with the
/// leader, in preference order) that are compatible with each other. The first set in order.
fn fill<'a>(healers: &[&'a Ad], damage: &[&'a Ad], min_level: u8) -> Option<(&'a Ad, [&'a Ad; 3])> {
    for &h in healers {
        let dmg: Vec<&Ad> = damage
            .iter()
            .copied()
            .filter(|d| ok(h, d, min_level))
            .collect();
        for i in 0..dmg.len() {
            for j in i + 1..dmg.len() {
                if !ok(dmg[i], dmg[j], min_level) {
                    continue;
                }
                for k in j + 1..dmg.len() {
                    if ok(dmg[i], dmg[k], min_level) && ok(dmg[j], dmg[k], min_level) {
                        return Some((h, [dmg[i], dmg[j], dmg[k]]));
                    }
                }
            }
        }
    }
    None
}

fn pool_for<'a>(epoch: u64, leader: &Ad, candidates: &[&'a Ad], min_level: u8) -> Vec<&'a Ad> {
    let mut pool: Vec<&Ad> = candidates
        .iter()
        .copied()
        .filter(|a| a.peer != leader.peer && ok(leader, a, min_level))
        .collect();
    pool.sort_by_cached_key(|a| (pair_rank(epoch, &leader.peer, &a.peer), a.peer));
    pool
}

fn by_role<'a>(pool: &[&'a Ad], role: Role) -> Vec<&'a Ad> {
    pool.iter().copied().filter(|a| a.role == role).collect()
}

/// The peers `leader` (a tank) invites for `epoch`: first the party it would build, then spares.
/// Empty when its sample holds no complete compatible party. Never contains the leader, a tank
/// or a peer incompatible with the leader.
pub fn plan_invites(epoch: u64, leader: &Ad, candidates: &[&Ad], min_level: u8) -> Vec<PeerId> {
    plan_invites_with(
        epoch,
        leader,
        candidates,
        min_level,
        SPARE_HEALERS,
        SPARE_DAMAGE,
    )
}

/// `plan_invites` with explicit spare counts (the simulation sweeps them).
pub fn plan_invites_with(
    epoch: u64,
    leader: &Ad,
    candidates: &[&Ad],
    min_level: u8,
    spare_healers: usize,
    spare_damage: usize,
) -> Vec<PeerId> {
    if leader.role != Role::Tank {
        return Vec::new();
    }
    let pool = pool_for(epoch, leader, candidates, min_level);
    let healers = by_role(&pool, Role::Healer);
    let damage = by_role(&pool, Role::Damage);
    let top_h = &healers[..healers.len().min(SEARCH_HEALERS)];
    let top_d = &damage[..damage.len().min(SEARCH_DAMAGE)];
    let Some((h, d)) = fill(top_h, top_d, min_level) else {
        return Vec::new();
    };
    let primary = [h, d[0], d[1], d[2]];
    let mut out: Vec<PeerId> = primary.iter().map(|a| a.peer).collect();
    for (list, n) in [(&healers, spare_healers), (&damage, spare_damage)] {
        out.extend(
            list.iter()
                .filter(|a| {
                    !primary.iter().any(|p| p.peer == a.peer)
                        && primary.iter().all(|p| ok(p, a, min_level))
                })
                .take(n)
                .map(|a| a.peer),
        );
    }
    out
}

/// The order in which an invitee prefers leaders: lowest first.
pub fn invite_preference(epoch: u64, leader: &PeerId) -> (u64, PeerId) {
    (rank(epoch, leader), *leader)
}

/// Complete a party from the peers that accepted `leader`'s invitation. `accepted` are their
/// ads (from the leader's sample). The first complete compatible set in the leader's order.
pub fn choose_party(
    topic: &TopicId,
    epoch: u64,
    leader: &Ad,
    accepted: &[&Ad],
    min_level: u8,
) -> Option<Party> {
    let pool = pool_for(epoch, leader, accepted, min_level);
    let (h, d) = fill(
        &by_role(&pool, Role::Healer),
        &by_role(&pool, Role::Damage),
        min_level,
    )?;
    let mut members: [PeerId; PARTY_SIZE] = [leader.peer, h.peer, d[0].peer, d[1].peer, d[2].peer];
    members.sort();
    Some(Party {
        id: party_id(topic, epoch, &members),
        epoch,
        members,
    })
}

#[cfg(test)]
mod tests {
    use super::super::ad::test_ad;
    use super::*;

    fn topic() -> TopicId {
        TopicId([1; 32])
    }

    /// Index 0 and 14 are tanks, 1 to 3 healers, 4 to 13 damage.
    fn pool() -> Vec<Ad> {
        let mut v = vec![test_ad(1, Role::Tank, 24, 100)];
        for i in 0..3 {
            v.push(test_ad(10 + i, Role::Healer, 24, 100));
        }
        for i in 0..10 {
            v.push(test_ad(30 + i, Role::Damage, 24, 100));
        }
        v.push(test_ad(60, Role::Tank, 24, 100));
        v
    }

    #[test]
    fn invites_cover_one_party_plus_spares_and_never_the_leader() {
        let ads = pool();
        let refs: Vec<&Ad> = ads.iter().collect();
        let inv = plan_invites(5, &ads[0], &refs, 1);
        let roles = |role| {
            inv.iter()
                .filter(|p| ads.iter().find(|a| a.peer == **p).unwrap().role == role)
                .count()
        };
        assert_eq!(roles(Role::Healer), 3); // one needed, two spares
        assert_eq!(roles(Role::Damage), 8); // three needed, five spares
        assert_eq!(roles(Role::Tank), 0);
        assert!(!inv.contains(&ads[0].peer));
        // The first four are a party.
        let first: Vec<&Ad> = inv[..4]
            .iter()
            .map(|p| ads.iter().find(|a| a.peer == *p).unwrap())
            .collect();
        assert_eq!(first.iter().filter(|a| a.role == Role::Healer).count(), 1);
        assert_eq!(first.iter().filter(|a| a.role == Role::Damage).count(), 3);
    }

    #[test]
    fn only_a_tank_leads_and_a_sample_without_a_healer_gives_no_invites() {
        let ads = pool();
        let refs: Vec<&Ad> = ads.iter().collect();
        assert!(plan_invites(5, &ads[1], &refs, 1).is_empty()); // a healer
        let no_healer: Vec<&Ad> = ads.iter().filter(|a| a.role != Role::Healer).collect();
        assert!(plan_invites(5, &ads[0], &no_healer, 1).is_empty());
    }

    #[test]
    fn leaders_order_candidates_differently() {
        let ads = pool();
        let refs: Vec<&Ad> = ads.iter().collect();
        let a = plan_invites(5, &ads[0], &refs, 1);
        let b = plan_invites(5, &ads[14], &refs, 1);
        assert_eq!(b.len(), a.len());
        assert_ne!(a[..4], b[..4]);
    }

    #[test]
    fn far_levels_are_not_invited() {
        let mut ads = pool();
        ads[5].level = 40;
        let refs: Vec<&Ad> = ads.iter().collect();
        let inv = plan_invites(5, &ads[0], &refs, 1);
        assert!(!inv.contains(&ads[5].peer));
    }

    #[test]
    fn choose_party_needs_one_healer_and_three_damage_compatible_with_all() {
        let ads = pool();
        let acc: Vec<&Ad> = vec![&ads[1], &ads[4], &ads[5], &ads[6]];
        let p = choose_party(&topic(), 5, &ads[0], &acc, 1).unwrap();
        assert!(p.contains(&ads[0].peer) && p.contains(&ads[1].peer));
        // Only two damage accepted: no party.
        assert!(choose_party(&topic(), 5, &ads[0], &acc[..3], 1).is_none());
        // Two healers and two damage is not a party either.
        let acc2: Vec<&Ad> = vec![&ads[1], &ads[2], &ads[4], &ads[5]];
        assert!(choose_party(&topic(), 5, &ads[0], &acc2, 1).is_none());
    }

    #[test]
    fn choose_party_skips_an_acceptor_that_clashes_with_the_others() {
        let mut ads = pool();
        // The damage peer at index 4 has an item level too far from the others.
        ads[4].item_level = 150;
        let acc: Vec<&Ad> = vec![&ads[1], &ads[4], &ads[5], &ads[6], &ads[7]];
        let p = choose_party(&topic(), 5, &ads[0], &acc, 1).unwrap();
        assert!(!p.contains(&ads[4].peer));
    }

    #[test]
    fn preference_is_the_lowest_leader_rank_and_is_the_same_for_everyone() {
        let a = PeerId([1; 32]);
        let b = PeerId([2; 32]);
        let ka = invite_preference(9, &a);
        let kb = invite_preference(9, &b);
        assert_ne!(ka, kb);
        assert_eq!(ka < kb, rank(9, &a) < rank(9, &b));
    }
}
