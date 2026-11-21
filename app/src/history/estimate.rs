//! The estimator (ADR 0027, stage 8b): from this node's own finished searches, one banded and
//! hedged outlook for a search that is still running. Pure: it takes entries and an observation
//! and returns a result, with no store read and no clock.
//!
//! Nothing here has been checked against real entries (none exist yet), so every constant is an
//! unverified starting value and the gate defaults to silence: too little or too uncertain
//! history gives no outlook and the live diagnosis stands alone. A number leaves this module only
//! as a `Band`, never as data (Resolution 8 of the brief).
//!
//! Not part of this cut: a typical wait among the searches that found a group. It needs the
//! incomplete (`Stopped`) searches treated as "at least this long" and is deferred to a later
//! change, so no wait-time band exists yet.

use super::{Entry, Outcome, RULES_VERSION, SCHEMA_VERSION};
use crate::code::Role;
use crate::engine::RoleCounts;
use crate::engine::diagnose::{Bottleneck, Situation, bottleneck};

/// Successes and failures each needed in the pool (Resolution 10). Below this the share is
/// meaningless, so there is no estimate. Unverified.
pub const MIN_SUCCESSES: u32 = 3;
pub const MIN_FAILURES: u32 = 3;
/// Widest 95% interval (as a share, 0.40 = 40 points) that still counts as dependable. Wider and
/// the answer is `Unreliable`. Unverified.
pub const MAX_WIDTH: f64 = 0.40;
/// z for a 95% interval.
const Z95: f64 = 1.96;
/// The interval's lower end at or above this is `UsuallyFound`. Clearly more than even odds, so
/// a band never rests on a lucky streak. Unverified.
pub const USUALLY_FROM: f64 = 0.60;
/// The interval's upper end at or below this is `RarelyFound`. Unverified.
pub const RARELY_UP_TO: f64 = 0.35;

/// How the player's earlier searches here tended to end. The only form history takes outside
/// this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    UsuallyFound,
    SometimesFound,
    RarelyFound,
}

impl Band {
    /// The wire code (snake_case).
    pub fn code(self) -> &'static str {
        match self {
            Band::UsuallyFound => "usually_found",
            Band::SometimesFound => "sometimes_found",
            Band::RarelyFound => "rarely_found",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estimate {
    /// Fewer than `MIN_SUCCESSES` successes or `MIN_FAILURES` failures in the pool.
    NotEnoughHistory,
    /// Enough entries, but the interval is wider than `MAX_WIDTH`.
    Unreliable,
    Band(Band),
}

impl Estimate {
    pub fn band(self) -> Option<Band> {
        match self {
            Estimate::Band(b) => Some(b),
            _ => None,
        }
    }
}

/// The current search at its 55 s checkpoint.
#[derive(Debug, Clone, Copy)]
pub struct Observation<'a> {
    /// What was seen at the checkpoint (`SearchStats::checkpoint`).
    pub counts: RoleCounts,
    pub own_role: Role,
    /// The first dungeon chosen, for the secondary split.
    pub dungeon: &'a str,
}

fn parse_role(s: &str) -> Option<Role> {
    match s {
        "tank" => Some(Role::Tank),
        "healer" => Some(Role::Healer),
        "damage" => Some(Role::Damage),
        _ => None,
    }
}

/// The 95% Wilson score interval for `successes` of `n` (`n > 0`), as `(low, high)`.
pub fn wilson(successes: u32, n: u32) -> (f64, f64) {
    let n = f64::from(n);
    let p = f64::from(successes) / n;
    let z2 = Z95 * Z95;
    let denom = 1.0 + z2 / n;
    let centre = (p + z2 / (2.0 * n)) / denom;
    let half = Z95 * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denom;
    ((centre - half).max(0.0), (centre + half).min(1.0))
}

/// Judge a success share from its counts alone.
fn judge(successes: u32, failures: u32) -> Estimate {
    if successes < MIN_SUCCESSES || failures < MIN_FAILURES {
        return Estimate::NotEnoughHistory;
    }
    let (lo, hi) = wilson(successes, successes + failures);
    if hi - lo > MAX_WIDTH {
        Estimate::Unreliable
    } else if lo >= USUALLY_FROM {
        Estimate::Band(Band::UsuallyFound)
    } else if hi <= RARELY_UP_TO {
        Estimate::Band(Band::RarelyFound)
    } else {
        Estimate::Band(Band::SometimesFound)
    }
}

/// The similar entries: this build's schema and rules, a checkpoint that was taken, the same own
/// role and the same bottleneck bin. An entry whose search ended before the 55 s checkpoint has
/// `checkpoint_1 == None` and is excluded on purpose: a live estimate is only asked for once the
/// current search has reached the checkpoint still unformed, so only searches that were also
/// still unformed at that point are comparable.
fn similar(entries: &[Entry], bin: Bottleneck, own: Role) -> impl Iterator<Item = &Entry> {
    entries.iter().filter(move |e| {
        e.schema_version == SCHEMA_VERSION
            && e.rules_version == RULES_VERSION
            && parse_role(&e.own_role) == Some(own)
            && e.checkpoint_1.is_some_and(|cp| {
                let c = cp.counts;
                let seen = RoleCounts {
                    players: c.players,
                    tanks: c.tanks,
                    healers: c.healers,
                    damage: c.damage,
                };
                bottleneck(&seen, Some(own)) == bin
            })
    })
}

/// Success and failure counts. `Formed` is a success and `Expired` a failure, judged by the
/// entry's final outcome (a drop followed by a re-form is a success). `Stopped` is incomplete
/// data and is in neither count.
fn tally<'a>(it: impl Iterator<Item = &'a Entry>) -> (u32, u32) {
    it.fold((0, 0), |(s, f), e| match e.outcome {
        Outcome::Formed => (s + 1, f),
        Outcome::Expired => (s, f + 1),
        Outcome::Stopped => (s, f),
    })
}

/// The outlook for the observed search. The pool is every similar entry across dungeons. The
/// same dungeon alone is used instead only when it passes the same gate by itself.
pub fn estimate(entries: &[Entry], obs: &Observation<'_>) -> Estimate {
    let bin = bottleneck(&obs.counts, Some(obs.own_role));
    let own = obs.own_role;
    let (s, f) = tally(similar(entries, bin, own).filter(|e| e.dungeon == obs.dungeon));
    if let Estimate::Band(b) = judge(s, f) {
        return Estimate::Band(b);
    }
    let (s, f) = tally(similar(entries, bin, own));
    judge(s, f)
}

/// Whether history may speak next to this live situation. History never reassures against a
/// live problem: `UsuallyFound` only goes with `Healthy` or `OthersGrouping` (parties forming
/// elsewhere is not a fault of this search). `SometimesFound` and `RarelyFound` may accompany any
/// situation that has a verdict. Never for a search that ended, while warming up or while not
/// connected.
pub fn outlook(situation: Situation, ended: bool, band: Option<Band>) -> Option<Band> {
    let band = band?;
    if ended || matches!(situation, Situation::WarmingUp | Situation::NotConnected) {
        return None;
    }
    if band == Band::UsuallyFound
        && !matches!(situation, Situation::Healthy | Situation::OthersGrouping)
    {
        return None;
    }
    Some(band)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::tests::entry;
    use crate::history::{CheckpointEntry, Counts};

    fn counts(t: u32, h: u32, d: u32) -> Counts {
        Counts {
            players: t + h + d,
            tanks: t,
            healers: h,
            damage: d,
        }
    }

    /// A damage dealer's entry whose checkpoint saw no tank (the tank bin).
    fn e(start: u64, dungeon: &str, outcome: Outcome) -> Entry {
        let mut x = entry(start, dungeon);
        x.own_role = "damage".into();
        x.checkpoint_1 = Some(CheckpointEntry {
            counts: counts(0, 2, 3),
            stage: 0,
            sample_full: false,
        });
        x.outcome = outcome;
        x
    }

    /// The matching observation: a damage dealer who saw no tank.
    fn obs(dungeon: &str) -> Observation<'_> {
        Observation {
            counts: RoleCounts {
                players: 5,
                tanks: 0,
                healers: 2,
                damage: 3,
            },
            own_role: Role::Damage,
            dungeon,
        }
    }

    fn many(formed: u32, expired: u32, dungeon: &str) -> Vec<Entry> {
        let mut v = Vec::new();
        for i in 0..formed {
            v.push(e(1_000 + u64::from(i), dungeon, Outcome::Formed));
        }
        for i in 0..expired {
            v.push(e(2_000 + u64::from(i), dungeon, Outcome::Expired));
        }
        v
    }

    /// A small seeded generator (xorshift) so the known-effect tests repeat.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> f64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    /// `n` entries spread over two invented dungeons with a known success share.
    fn seeded(seed: u64, n: u32, p_formed: f64) -> Vec<Entry> {
        let mut r = Rng(seed);
        (0..n)
            .map(|i| {
                let out = if r.next() < p_formed {
                    Outcome::Formed
                } else {
                    Outcome::Expired
                };
                let d = ["testville-depths", "fakeford-keep"][(i % 2) as usize];
                e(1_000 + u64::from(i), d, out)
            })
            .collect()
    }

    #[test]
    fn wilson_matches_known_values() {
        let (lo, hi) = wilson(5, 10);
        assert!((lo - 0.2366).abs() < 1e-3 && (hi - 0.7634).abs() < 1e-3);
        let (lo, hi) = wilson(0, 10);
        assert!(lo == 0.0 && (hi - 0.2775).abs() < 1e-3);
        let (lo, hi) = wilson(10, 10);
        assert!(hi == 1.0 && (lo - 0.7225).abs() < 1e-3);
    }

    #[test]
    fn a_bin_with_high_success_is_usually_found() {
        let v = seeded(7, 60, 0.9);
        let want = Estimate::Band(Band::UsuallyFound);
        assert_eq!(estimate(&v, &obs("testville-depths")), want);
    }

    #[test]
    fn a_bin_with_low_success_is_rarely_found() {
        let v = seeded(11, 60, 0.1);
        let want = Estimate::Band(Band::RarelyFound);
        assert_eq!(estimate(&v, &obs("testville-depths")), want);
    }

    #[test]
    fn a_middling_bin_is_sometimes_found() {
        let v = seeded(3, 80, 0.5);
        let want = Estimate::Band(Band::SometimesFound);
        assert_eq!(estimate(&v, &obs("testville-depths")), want);
    }

    #[test]
    fn too_few_of_either_outcome_is_not_enough_history() {
        for (s, f) in [(0, 0), (2, 40), (40, 2), (3, 2), (2, 3), (1, 1)] {
            assert_eq!(
                estimate(&many(s, f, "d"), &obs("d")),
                Estimate::NotEnoughHistory,
                "{s} formed {f} expired"
            );
        }
    }

    #[test]
    fn all_failures_or_all_successes_are_never_a_band() {
        let none = Estimate::NotEnoughHistory;
        assert_eq!(estimate(&many(30, 0, "d"), &obs("d")), none);
        assert_eq!(estimate(&many(0, 30, "d"), &obs("d")), none);
    }

    #[test]
    fn a_wide_interval_is_unreliable() {
        assert_eq!(estimate(&many(3, 3, "d"), &obs("d")), Estimate::Unreliable);
        assert_eq!(estimate(&many(5, 5, "d"), &obs("d")), Estimate::Unreliable);
    }

    #[test]
    fn the_bands_have_their_edges() {
        let (lo, hi) = wilson(28, 31);
        assert!(hi - lo <= MAX_WIDTH && lo >= USUALLY_FROM);
        assert_eq!(
            estimate(&many(28, 3, "d"), &obs("d")),
            Estimate::Band(Band::UsuallyFound)
        );
        let (_, hi) = wilson(3, 40);
        assert!(hi <= RARELY_UP_TO);
        assert_eq!(
            estimate(&many(3, 37, "d"), &obs("d")),
            Estimate::Band(Band::RarelyFound)
        );
    }

    #[test]
    fn stopped_searches_are_not_in_the_rate() {
        let mut v = many(3, 37, "d");
        let alone = estimate(&v, &obs("d"));
        assert!(alone.band().is_some());
        for i in 0..200 {
            v.push(e(5_000 + i, "d", Outcome::Stopped));
        }
        assert_eq!(estimate(&v, &obs("d")), alone);
        let stops: Vec<Entry> = (0..50).map(|i| e(i, "d", Outcome::Stopped)).collect();
        assert_eq!(estimate(&stops, &obs("d")), Estimate::NotEnoughHistory);
    }

    #[test]
    fn entries_that_ended_before_the_checkpoint_are_excluded() {
        let mut v = many(3, 37, "d");
        let alone = estimate(&v, &obs("d"));
        for i in 0..100 {
            let mut x = e(5_000 + i, "d", Outcome::Formed);
            x.checkpoint_1 = None;
            v.push(x);
        }
        assert_eq!(estimate(&v, &obs("d")), alone);
    }

    #[test]
    fn other_versions_are_skipped() {
        let mut v = many(3, 37, "d");
        let alone = estimate(&v, &obs("d"));
        for i in 0..100 {
            let mut x = e(5_000 + i, "d", Outcome::Formed);
            if i % 2 == 0 {
                x.rules_version = "epoch9-other".into();
            } else {
                x.schema_version = SCHEMA_VERSION + 1;
            }
            v.push(x);
        }
        assert_eq!(estimate(&v, &obs("d")), alone);
    }

    #[test]
    fn only_the_same_bin_and_own_role_count() {
        let mut v = many(3, 37, "d");
        let alone = estimate(&v, &obs("d"));
        for i in 0..100 {
            let mut a = e(5_000 + i, "d", Outcome::Formed);
            a.own_role = "healer".into();
            let mut b = e(6_000 + i, "d", Outcome::Formed);
            b.checkpoint_1.as_mut().unwrap().counts = counts(2, 2, 3);
            v.push(a);
            v.push(b);
        }
        assert_eq!(estimate(&v, &obs("d")), alone);
    }

    #[test]
    fn a_drop_then_a_reform_is_judged_by_its_final_outcome() {
        let mut v = Vec::new();
        for i in 0..30 {
            let mut x = e(1_000 + i, "d", Outcome::Formed);
            x.drops = 1;
            x.first_formed_at = Some(80);
            v.push(x);
        }
        v.extend(many(0, 3, "d"));
        assert_eq!(estimate(&v, &obs("d")), Estimate::Band(Band::UsuallyFound));
    }

    #[test]
    fn the_pool_spans_dungeons_and_the_dungeon_split_needs_the_same_gate() {
        let mut v = many(30, 3, "elsewhere");
        // No entries here: the pool answers.
        assert_eq!(
            estimate(&v, &obs("testville-depths")),
            Estimate::Band(Band::UsuallyFound)
        );
        // A few entries here do not pass the gate and do not override the pool.
        v.extend(many(0, 5, "testville-depths"));
        assert_eq!(
            estimate(&v, &obs("testville-depths")),
            Estimate::Band(Band::UsuallyFound)
        );
        // This dungeon alone passes the gate with another result, so it is used.
        v.extend(many(3, 37, "testville-depths"));
        assert_eq!(
            estimate(&v, &obs("testville-depths")),
            Estimate::Band(Band::RarelyFound)
        );
    }

    #[test]
    fn codes_are_snake_case() {
        assert_eq!(Band::UsuallyFound.code(), "usually_found");
        assert_eq!(Band::SometimesFound.code(), "sometimes_found");
        assert_eq!(Band::RarelyFound.code(), "rarely_found");
    }

    #[test]
    fn emission_rules_table() {
        use Band::*;
        use Situation::*;
        let all = [
            WarmingUp,
            NotConnected,
            OthersGrouping,
            Alone,
            NoTank,
            NoHealer,
            ShortOnDamage,
            NobodyAnswered,
            Healthy,
        ];
        for sit in all {
            for band in [UsuallyFound, SometimesFound, RarelyFound] {
                let silent = matches!(sit, WarmingUp | NotConnected)
                    || (band == UsuallyFound && !matches!(sit, Healthy | OthersGrouping));
                let want = if silent { None } else { Some(band) };
                assert_eq!(outlook(sit, false, Some(band)), want, "{sit:?} {band:?}");
                // Never once the search ended, whatever the situation.
                assert_eq!(
                    outlook(sit, true, Some(band)),
                    None,
                    "ended {sit:?} {band:?}"
                );
            }
            assert_eq!(outlook(sit, false, None), None);
        }
    }
}
