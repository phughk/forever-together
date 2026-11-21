//! Seeded simulation of the agreement mechanisms (design section 6, ADRs 0010 and 0025, decision
//! D1): converging announcements and the proposer model (both rejected, kept here as evidence)
//! and leader invitations (adopted, ADR 0025). It is an algorithm check on the pure engine
//! pieces, not a load test (ADR 0014).
//!
//! Model, shared by all mechanisms:
//!
//! * one topic, `peers` searching peers, all on stage 0 and the same faction;
//! * roles 15% tank, 20% healer, 65% damage; levels uniform 20 to 28; item level unknown for 30%
//!   of peers, otherwise 100 plus or minus 12%;
//! * gossip delivers an ad to a peer with probability `delivery` (this is the modelled sample
//!   divergence), the peer keeps `sample_size` ads, dropping those furthest in level from its
//!   own level first (`sample::keep_nearest`, the same rule as `Sample`);
//! * a `Taken` that was missed leaves a committed peer in a sample with probability
//!   `(1 - delivery) * delivery` per later round (stale knowledge);
//! * announcements and responses are delivered reliably inside their window, except where
//!   `direct_loss` says otherwise (leader model only).
//!
//! Metric (the plan's floor): of the peers that have a compatible five in their own sample at
//! the start (baseline epoch 0 view), the share that is committed within `rounds` rounds. A round
//! is one epoch (10 s) for converging announcements and for leader invitations (one invite,
//! accept and commit exchange per epoch). For the proposer model it is one proposal cycle of a
//! response window plus a decision window (8 s), so three epochs (30 s) are compared with three
//! cycles. A second number, `capacity_share`, compares the peers committed with what one central
//! greedy build over everyone commits (`run_oracle`): it separates the mechanism from the cap
//! the role mix puts on any mechanism (every party needs a tank, so 15% tanks cannot commit more
//! than 75% of the peers).

use super::ad::{Ad, Faction, Role, compatible, pair_stage};
use super::lead::{
    SPARE_DAMAGE, SPARE_HEALERS, choose_party, invite_preference, plan_invites_with,
};
use super::party::{PARTY_SIZE, build_parties, party_for};
use super::rank::rank;
use super::sample::{Tie, keep_nearest};
use crate::dungeons::TopicId;
use crate::identity::PeerId;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub peers: usize,
    pub sample_size: usize,
    pub delivery: f64,
    pub rounds: u32,
    pub seed: u64,
    /// Chance that a free peer starts a proposal in a cycle (proposer model).
    pub proposer_share: f64,
    /// Sample tie-break (see `Tie`). The old mechanisms were measured with `LowestId`.
    pub tie: Tie,
    /// Probability that one direct message (invite, accept, commit) is lost. Ads and `Taken`
    /// use `delivery` instead.
    pub direct_loss: f64,
    /// Spare invitations per leader (leader model).
    pub spare_healers: usize,
    pub spare_damage: usize,
    /// Leader model. 0: every epoch draws a fresh sample (the model the old mechanisms were
    /// measured with, it refills for free). N: samples persist and are redrawn every N epochs
    /// (the real engine: ads are re-sent every keep-alive, 45 s, 9 epochs of 5 s).
    pub refresh_every: u32,
    /// Share of tanks in the population (healers are always 20%, the rest damage).
    pub tank_share: f64,
    /// Extra attempts for each direct message (leader model). 0: a lost message is lost.
    pub retries: u32,
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
            tie: Tie::LowestId,
            direct_loss: 0.0,
            spare_healers: SPARE_HEALERS,
            spare_damage: SPARE_DAMAGE,
            refresh_every: 0,
            tank_share: 0.15,
            retries: 0,
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
    /// The same two numbers with eligibility judged by the sample rule of the run (`cfg.tie`)
    /// instead of the old one. Equal to the first two for the old mechanisms.
    pub eligible_own: usize,
    pub committed_eligible_own: usize,
}

impl SimResult {
    /// Peers committed as a share of what the central greedy build (`run_oracle`) commits.
    pub fn capacity_share(&self, oracle: &SimResult) -> f64 {
        if oracle.committed_total == 0 {
            0.0
        } else {
            self.committed_total as f64 / oracle.committed_total as f64
        }
    }

    /// Committed share of the peers eligible under the run's own sample rule.
    pub fn rate_own(&self) -> f64 {
        if self.eligible_own == 0 {
            0.0
        } else {
            self.committed_eligible_own as f64 / self.eligible_own as f64
        }
    }

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
            let role = if r < cfg.tank_share {
                Role::Tank
            } else if r < cfg.tank_share + 0.20 {
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
    keep_nearest(pop[i].level, &pop[i].peer, v, cfg.sample_size, cfg.tie)
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
/// Always the old sample rule, so every mechanism is scored against the same peers.
fn eligible_set(cfg: &SimConfig, pop: &[Ad]) -> Vec<bool> {
    eligible_set_with(
        &SimConfig {
            tie: Tie::LowestId,
            ..cfg.clone()
        },
        pop,
    )
}

fn eligible_set_with(cfg: &SimConfig, pop: &[Ad]) -> Vec<bool> {
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
    let eligible = eligible.iter().filter(|e| **e).count();
    SimResult {
        peers: pop.len(),
        eligible,
        committed_eligible,
        committed_total,
        cumulative_total: cumulative,
        eligible_own: eligible,
        committed_eligible_own: committed_eligible,
    }
}

/// Re-score `r` with eligibility taken from the run's own sample rule.
fn with_own(mut r: SimResult, cfg: &SimConfig, pop: &[Ad], alive: &[bool]) -> SimResult {
    let own = eligible_set_with(cfg, pop);
    r.eligible_own = own.iter().filter(|e| **e).count();
    r.committed_eligible_own = (0..pop.len()).filter(|&i| own[i] && !alive[i]).count();
    r
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

/// Leader invitations (ADR 0025), one invite, accept and commit exchange per epoch.
///
/// Each free tank plans invitations from its own view and sends them; each free healer or
/// damage peer accepts the invitation of the best-ranked leader that invited it; a leader whose
/// acceptors make a complete compatible party sends the commit to the four. A direct message
/// is lost with probability `direct_loss` and tried `1 + retries` times. A party whose commit
/// does not reach all four fails and nobody is committed that epoch.
pub fn run_leader(cfg: &SimConfig) -> SimResult {
    let mut rng = Rng(cfg.seed);
    let pop = population(cfg, &mut rng);
    let eligible = eligible_set(cfg, &pop);
    let mut rng = Rng(cfg.seed ^ 0x1EAD_0001);
    let index: HashMap<PeerId, usize> = pop.iter().enumerate().map(|(i, a)| (a.peer, i)).collect();
    let mut alive = vec![true; pop.len()];
    let mut cumulative = Vec::new();
    let sent = |rng: &mut Rng| (0..=cfg.retries).any(|_| rng.f64() >= cfg.direct_loss);
    // Persistent samples (`refresh_every > 0`): each tank keeps the sample it had at the start.
    // A committed peer leaves it when the `Taken` reaches that tank (probability `delivery`).
    // Nothing is added until the ads are re-sent, `refresh_every` epochs later.
    let mut samples: Vec<Vec<usize>> = Vec::new();
    for round in 0..cfg.rounds {
        let epoch = 1_000 + round as u64;
        let refresh = cfg.refresh_every > 0 && round % cfg.refresh_every == 0;
        if refresh {
            samples = (0..pop.len())
                .map(|i| {
                    if alive[i] && pop[i].role == Role::Tank {
                        view(i, &pop, &alive, cfg, &mut rng)
                            .iter()
                            .map(|a| index[&a.peer])
                            .collect()
                    } else {
                        Vec::new()
                    }
                })
                .collect();
        }
        let mut inbox: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for i in 0..pop.len() {
            if !alive[i] || pop[i].role != Role::Tank {
                continue;
            }
            let v: Vec<&Ad> = if cfg.refresh_every > 0 {
                samples[i].iter().map(|&j| &pop[j]).collect()
            } else {
                view(i, &pop, &alive, cfg, &mut rng)
            };
            for target in
                plan_invites_with(epoch, &pop[i], &v, 1, cfg.spare_healers, cfg.spare_damage)
            {
                let t = index[&target];
                if sent(&mut rng) {
                    inbox.entry(t).or_default().push(i);
                }
            }
        }
        let was_alive = alive.clone();
        let mut accepted: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (t, leaders) in inbox {
            if !alive[t] {
                continue;
            }
            let best = leaders
                .into_iter()
                .min_by_key(|&l| invite_preference(epoch, &pop[l].peer))
                .unwrap();
            if sent(&mut rng) {
                accepted.entry(best).or_default().push(t);
            }
        }
        for (l, acc) in accepted {
            let ads: Vec<&Ad> = acc.iter().map(|&t| &pop[t]).collect();
            let Some(party) = choose_party(&topic(), epoch, &pop[l], &ads, 1) else {
                continue;
            };
            let reached = (0..PARTY_SIZE - 1).all(|_| sent(&mut rng));
            if !reached {
                continue;
            }
            for m in party.members {
                alive[index[&m]] = false;
            }
        }
        if cfg.refresh_every > 0 {
            for j in (0..pop.len()).filter(|&j| was_alive[j] && !alive[j]) {
                for s in samples.iter_mut() {
                    if let Some(k) = s.iter().position(|&x| x == j)
                        && rng.f64() < cfg.delivery
                    {
                        s.swap_remove(k);
                    }
                }
            }
        }
        cumulative.push(alive.iter().filter(|a| !**a).count());
    }
    with_own(
        finish(&pop, &alive, eligible, cumulative),
        cfg,
        &pop,
        &alive,
    )
}

/// Upper bound: one central greedy party build over every peer's full ad list, no sample, no
/// loss. Not reachable by any peer-to-peer mechanism.
pub fn run_oracle(cfg: &SimConfig) -> SimResult {
    let mut rng = Rng(cfg.seed);
    let pop = population(cfg, &mut rng);
    let eligible = eligible_set(cfg, &pop);
    let index: HashMap<PeerId, usize> = pop.iter().enumerate().map(|(i, a)| (a.peer, i)).collect();
    let mut alive = vec![true; pop.len()];
    let all: Vec<&Ad> = pop.iter().collect();
    for p in build_parties(&topic(), 1_000, &all, 1) {
        for m in p.members {
            alive[index[&m]] = false;
        }
    }
    with_own(
        finish(&pop, &alive, eligible, Vec::new()),
        cfg,
        &pop,
        &alive,
    )
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

    fn leader_cfg(base: &SimConfig) -> SimConfig {
        SimConfig {
            tie: Tie::OwnerHash,
            ..base.clone()
        }
    }

    #[test]
    fn leader_runs_repeat_and_commit_whole_parties() {
        let cfg = leader_cfg(&small());
        let a = run_leader(&cfg);
        assert_eq!(a, run_leader(&cfg));
        assert_ne!(
            a,
            run_leader(&SimConfig {
                seed: 2,
                ..cfg.clone()
            })
        );
        assert_eq!(a.committed_total % PARTY_SIZE, 0);
        assert!(a.committed_total > 0);
        assert!(a.cumulative_total.windows(2).all(|w| w[0] <= w[1]));
        let p = run_leader(&SimConfig {
            refresh_every: 4,
            ..cfg
        });
        assert_eq!(p.committed_total % PARTY_SIZE, 0);
    }

    /// Regression guard for ADR 0025: the leader mechanism keeps the plan's 70% floor within 30 s
    /// (6 rounds of 5 s) at a size that runs quickly in a debug build, with fresh and with
    /// persistent samples. The full-size numbers are in `docs/spike-s0.md`.
    #[test]
    fn leader_mechanism_reaches_the_floor_in_thirty_seconds() {
        let cfg = leader_cfg(&SimConfig {
            peers: 1200,
            rounds: 6,
            ..SimConfig::default()
        });
        for refresh_every in [0, 9] {
            let r = run_leader(&SimConfig {
                refresh_every,
                ..cfg.clone()
            });
            assert!(
                r.rate() >= 0.70 && r.rate_own() >= 0.70,
                "refresh {refresh_every}: {:.1}% / {:.1}%",
                r.rate() * 100.0,
                r.rate_own() * 100.0
            );
        }
    }

    #[test]
    fn the_old_sample_rule_starves_high_ids_and_the_new_one_does_not() {
        let base = SimConfig {
            peers: 1200,
            sample_size: 40,
            ..SimConfig::default()
        };
        let old = run_leader(&SimConfig {
            tie: Tie::LowestId,
            ..base.clone()
        });
        let new = run_leader(&leader_cfg(&base));
        assert!(new.committed_total > old.committed_total);
    }
}
