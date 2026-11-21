//! Seeded simulation of the two agreement mechanisms (design section 6, ADR 0010, decision D1).
//! It is an algorithm check on the pure engine pieces, not a load test (ADR 0014).
//!
//! Model, shared by both mechanisms:
//!
//! * one topic, `peers` searching peers, all on stage 0 and the same faction;
//! * roles 15% tank, 20% healer, 65% damage; levels uniform 20 to 28; item level unknown for 30%
//!   of peers, otherwise 100 plus or minus 12%;
//! * gossip delivers an ad to a peer with probability `delivery` (this is the modelled sample
//!   divergence), the peer keeps `sample_size` ads, dropping those furthest in level from its
//!   own level first (`sample::keep_nearest`, the same rule as `Sample`);
//! * a `Taken` that was missed leaves a committed peer in a sample with probability
//!   `(1 - delivery) * delivery` per later round (stale knowledge);
//! * announcements and responses are delivered reliably inside their window.
//!
//! Metric: of the peers that have a compatible five in their own sample at the start (baseline
//! epoch 0 view), the share that is committed within `rounds` rounds. A round is one epoch (10 s)
//! for converging announcements. For the proposer model it is one proposal cycle of a response
//! window plus a decision window (8 s), so three epochs (30 s) are compared with three or four
//! cycles.

use super::ad::{Ad, Faction, Role, compatible, pair_stage};
use super::party::{PARTY_SIZE, build_parties, party_for};
use super::rank::rank;
use super::sample::keep_nearest;
use crate::dungeons::TopicId;
use crate::identity::PeerId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub peers: usize,
    pub sample_size: usize,
    pub delivery: f64,
    pub rounds: u32,
    pub seed: u64,
    /// Chance that a free peer starts a proposal in a cycle (proposer model).
    pub proposer_share: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            peers: 3000,
            sample_size: 100,
            delivery: 0.95,
            rounds: 3,
            seed: 1,
            proposer_share: 0.2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimResult {
    pub peers: usize,
    /// Peers with a compatible five in their own sample at the start.
    pub eligible: usize,
    pub committed_eligible: usize,
    pub committed_total: usize,
    /// Committed peers (of all peers) after each round.
    pub cumulative_total: Vec<usize>,
}

impl SimResult {
    pub fn rate(&self) -> f64 {
        if self.eligible == 0 {
            0.0
        } else {
            self.committed_eligible as f64 / self.eligible as f64
        }
    }
}

/// splitmix64.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn topic() -> TopicId {
    TopicId([42; 32])
}

fn population(cfg: &SimConfig, rng: &mut Rng) -> Vec<Ad> {
    (0..cfg.peers)
        .map(|i| {
            let mut id = [0u8; 32];
            for chunk in id.chunks_mut(8) {
                chunk.copy_from_slice(&rng.next().to_le_bytes());
            }
            let r = rng.f64();
            let role = if r < 0.15 {
                Role::Tank
            } else if r < 0.35 {
                Role::Healer
            } else {
                Role::Damage
            };
            let item_level = if rng.f64() < 0.3 {
                0
            } else {
                (88.0 + rng.f64() * 24.0) as u16
            };
            Ad {
                peer: PeerId(id),
                counter: 1,
                role,
                secondary_role: None,
                level: 20 + rng.below(9) as u8,
                item_level,
                faction: Faction::Alliance,
                stage: 0,
                first_name: String::new(),
                surname: String::new(),
                zone: String::new(),
                created_at: i as u32,
            }
        })
        .collect()
}

/// What peer `i` holds: itself plus the ads that reached it, cut to the sample size.
fn view<'a>(
    i: usize,
    pop: &'a [Ad],
    alive: &[bool],
    cfg: &SimConfig,
    rng: &mut Rng,
) -> Vec<&'a Ad> {
    let stale = (1.0 - cfg.delivery) * cfg.delivery;
    let mut v: Vec<&Ad> = Vec::with_capacity(pop.len() / 8);
    for (j, ad) in pop.iter().enumerate() {
        if j == i {
            v.push(ad);
            continue;
        }
        let p = if alive[j] { cfg.delivery } else { stale };
        if rng.f64() < p {
            v.push(ad);
        }
    }
    keep_nearest(pop[i].level, &pop[i].peer, v, cfg.sample_size)
}

/// Converging announcements, run for `cfg.rounds` epochs.
pub fn run_baseline(cfg: &SimConfig) -> SimResult {
    let mut rng = Rng(cfg.seed);
    let pop = population(cfg, &mut rng);
    let eligible = eligible_set(cfg, &pop);
    let index: HashMap<PeerId, usize> = pop.iter().enumerate().map(|(i, a)| (a.peer, i)).collect();
    let mut alive = vec![true; pop.len()];
    let mut cumulative = Vec::new();
    for round in 0..cfg.rounds {
        let epoch = 1_000 + round as u64;
        // Every peer builds the party it would announce.
        let parties: Vec<_> = (0..pop.len())
            .map(|i| {
                if !alive[i] {
                    return None;
                }
                let v = view(i, &pop, &alive, cfg, &mut rng);
                party_for(&build_parties(&topic(), epoch, &v, 1), &pop[i].peer)
            })
            .collect();
        // A party commits when all five members announced it.
        let mut commit = Vec::new();
        for p in parties.iter().flatten() {
            let all = p.members.iter().all(|m| {
                let k = index[m];
                alive[k] && parties[k].as_ref().is_some_and(|q| q.id == p.id)
            });
            if all {
                commit.push(p.members);
            }
        }
        for members in commit {
            for m in members {
                alive[index[&m]] = false;
            }
        }
        cumulative.push(alive.iter().filter(|a| !**a).count());
    }
    finish(&pop, &alive, eligible, cumulative)
}

/// Peers that have a compatible five in their own sample at the start (baseline epoch 0 view).
fn eligible_set(cfg: &SimConfig, pop: &[Ad]) -> Vec<bool> {
    let mut rng = Rng(cfg.seed ^ 0xE119_1B1E);
    let alive = vec![true; pop.len()];
    (0..pop.len())
        .map(|i| {
            let v = view(i, pop, &alive, cfg, &mut rng);
            party_for(&build_parties(&topic(), 1_000, &v, 1), &pop[i].peer).is_some()
        })
        .collect()
}

fn finish(pop: &[Ad], alive: &[bool], eligible: Vec<bool>, cumulative: Vec<usize>) -> SimResult {
    let committed_total = alive.iter().filter(|a| !**a).count();
    let committed_eligible = (0..pop.len()).filter(|&i| eligible[i] && !alive[i]).count();
    SimResult {
        peers: pop.len(),
        eligible: eligible.iter().filter(|e| **e).count(),
        committed_eligible,
        committed_total,
        cumulative_total: cumulative,
    }
}

struct Group {
    leader: usize,
    members: Vec<usize>,
}

impl Group {
    fn needs(&self, pop: &[Ad], role: Role) -> bool {
        let n = self
            .members
            .iter()
            .filter(|&&m| pop[m].role == role)
            .count();
        n < match role {
            Role::Tank | Role::Healer => 1,
            Role::Damage => 3,
        }
    }
    fn fits(&self, pop: &[Ad], r: usize) -> bool {
        self.needs(pop, pop[r].role)
            && self
                .members
                .iter()
                .all(|&m| compatible(&pop[m], &pop[r], pair_stage(&pop[m], &pop[r]), 1))
    }
    fn missing(&self) -> usize {
        PARTY_SIZE - self.members.len()
    }
}

/// Proposer with response windows: a partial group advertises the roles it needs, a responder
/// answers one proposer (the lease), the proposer accepts responders that fit, a group that
/// gains nobody in a cycle fails and its members are free again.
pub fn run_proposer(cfg: &SimConfig) -> SimResult {
    let mut rng = Rng(cfg.seed);
    let pop = population(cfg, &mut rng);
    let eligible = eligible_set(cfg, &pop);
    let mut rng = Rng(cfg.seed ^ 0x9209_5E12);
    let index: HashMap<PeerId, usize> = pop.iter().enumerate().map(|(i, a)| (a.peer, i)).collect();
    let mut alive = vec![true; pop.len()];
    let mut group_of: Vec<Option<usize>> = vec![None; pop.len()];
    let mut groups: Vec<Option<Group>> = Vec::new();
    let mut cumulative = Vec::new();
    for cycle in 0..cfg.rounds {
        // Free peers either start a proposal or answer one.
        let mut responders = Vec::new();
        for i in 0..pop.len() {
            if !alive[i] || group_of[i].is_some() {
                continue;
            }
            if rng.f64() < cfg.proposer_share {
                groups.push(Some(Group {
                    leader: i,
                    members: vec![i],
                }));
                group_of[i] = Some(groups.len() - 1);
            } else {
                responders.push(i);
            }
        }
        // Each responder answers at most one proposer it can see: the group closest to full,
        // then the lowest `hash(cycle, leader)`.
        let mut offers: HashMap<usize, Vec<usize>> = HashMap::new();
        for &r in &responders {
            let v = view(r, &pop, &alive, cfg, &mut rng);
            let best = v
                .iter()
                .filter_map(|ad| {
                    let k = *index.get(&ad.peer)?;
                    let g = group_of[k]
                        .filter(|&g| groups[g].as_ref().is_some_and(|gr| gr.leader == k))?;
                    let gr = groups[g].as_ref()?;
                    gr.fits(&pop, r)
                        .then(|| ((gr.missing(), rank(cycle as u64, &pop[k].peer)), g))
                })
                .min_by_key(|(key, _)| *key);
            if let Some((_, g)) = best {
                offers.entry(g).or_default().push(r);
            }
        }
        // Proposers decide.
        #[allow(clippy::needless_range_loop)] // `groups` is read and written by index
        for g in 0..groups.len() {
            let Some(mut gr) = groups[g].take() else {
                continue;
            };
            let mut got = offers.remove(&g).unwrap_or_default();
            got.sort_by_key(|&r| rank(cycle as u64, &pop[r].peer));
            let mut grew = false;
            for r in got {
                if gr.members.len() < PARTY_SIZE && gr.fits(&pop, r) {
                    gr.members.push(r);
                    group_of[r] = Some(g);
                    grew = true;
                }
            }
            if gr.members.len() == PARTY_SIZE {
                for &m in &gr.members {
                    alive[m] = false;
                    group_of[m] = None;
                }
            } else if grew {
                groups[g] = Some(gr);
            } else {
                for &m in &gr.members {
                    group_of[m] = None;
                }
            }
        }
        cumulative.push(alive.iter().filter(|a| !**a).count());
    }
    finish(&pop, &alive, eligible, cumulative)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> SimConfig {
        SimConfig {
            peers: 400,
            ..SimConfig::default()
        }
    }

    #[test]
    fn seeded_runs_repeat() {
        assert_eq!(run_baseline(&small()), run_baseline(&small()));
        assert_eq!(run_proposer(&small()), run_proposer(&small()));
        let other = SimConfig { seed: 2, ..small() };
        assert_ne!(run_baseline(&small()), run_baseline(&other));
    }

    #[test]
    fn parties_are_whole_and_rates_are_sane() {
        for r in [run_baseline(&small()), run_proposer(&small())] {
            assert_eq!(
                r.committed_total % PARTY_SIZE,
                0,
                "only whole parties commit"
            );
            assert!(r.committed_total > 0);
            assert!(r.eligible > 0 && r.committed_eligible <= r.eligible);
            assert!((0.0..=1.0).contains(&r.rate()));
            assert!(r.cumulative_total.windows(2).all(|w| w[0] <= w[1]));
        }
    }

    /// Prints the numbers for `docs/spike-s0.md`. Run with
    /// `cargo test --release --lib -- --ignored --nocapture sim::tests::report`. It does not fail on the floor: the floor
    /// is a decision for the author (70% within 3 epochs).
    #[test]
    #[ignore = "slow in debug; run with --release for the S0 numbers"]
    fn report() {
        let base = SimConfig::default();
        for (label, cfg) in [
            ("3000 peers, sample 100, delivery 0.95", base.clone()),
            (
                "3000 peers, sample 100, delivery 0.80",
                SimConfig {
                    delivery: 0.80,
                    ..base.clone()
                },
            ),
            (
                "3000 peers, sample 100, delivery 1.00",
                SimConfig {
                    delivery: 1.0,
                    ..base.clone()
                },
            ),
            (
                "3000 peers, sample 300, delivery 0.95",
                SimConfig {
                    sample_size: 300,
                    ..base.clone()
                },
            ),
            (
                "300 peers, sample 100, delivery 0.95",
                SimConfig {
                    peers: 300,
                    ..base.clone()
                },
            ),
        ] {
            let b = run_baseline(&cfg);
            let p3 = run_proposer(&cfg);
            let p4 = run_proposer(&SimConfig {
                rounds: 4,
                ..cfg.clone()
            });
            println!(
                "SIM {label}: eligible {}/{} | baseline 3 epochs {:.1}% | proposer 3 cycles {:.1}% | proposer 4 cycles {:.1}%",
                b.eligible,
                b.peers,
                b.rate() * 100.0,
                p3.rate() * 100.0,
                p4.rate() * 100.0,
            );
        }
        for share in [0.05, 0.1, 0.2, 0.4, 0.6] {
            let cfg = SimConfig {
                proposer_share: share,
                ..base.clone()
            };
            println!(
                "SIM proposer share {share}: 3 cycles {:.1}% | 4 cycles {:.1}%",
                run_proposer(&cfg).rate() * 100.0,
                run_proposer(&SimConfig {
                    rounds: 4,
                    ..cfg.clone()
                })
                .rate()
                    * 100.0
            );
        }
    }
}
