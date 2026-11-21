//! Search diagnosis (ADR 0026): a pure map from what this node observed to a situation code.
//! No clock and no I/O, so the same observations always give the same situation. The backend
//! sends the code and the counts; the web app owns the wording.
//!
//! Every count is a self-reported claim seen by this node only (ADR 0003): "none seen" never
//! means "none exist". All thresholds are unverified until a live run (docs/spike-s0.md).

use super::party::{DAMAGE_PER_PARTY, HEALERS_PER_PARTY, PARTY_SIZE, TANKS_PER_PARTY};
use super::state::{Attempts, RoleCounts};
use crate::code::Role;
use std::time::Duration;

/// Before this much search time no verdict is given. Source: ads are re-sent every keep-alive
/// and a joining peer needs a few epochs (5 s each) to be heard; 30 s is a guess, unverified.
pub const WARM_UP: Duration = Duration::from_secs(30);

/// Invitations this node must have sent, with no accept at all, before `nobody_answered` is
/// said. One round invites about 4 to 11 peers and every one of them is a separate chance to
/// answer, so 6 is a little over one small round; fewer would call a single epoch's silence a
/// pattern. A guess, unverified until a live run.
pub const MIN_ATTEMPTS: u32 = 6;

/// A sample with fewer compatible others than this cannot fill a party by itself (a party is
/// `PARTY_SIZE` players, one of them this player), so a taken notice can explain it.
pub const TINY_SAMPLE: u32 = PARTY_SIZE as u32 - 1;

/// What the player is told is happening. The web app maps each code to a sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situation {
    /// Too early to say.
    WarmingUp,
    /// No peers are connected: nothing can be seen.
    NotConnected,
    /// Taken notices removed peers from the sample and little is left: other players are
    /// forming parties right now, so the empty-looking sample is not an empty network.
    OthersGrouping,
    /// Connected, but no compatible player seen.
    Alone,
    NoTank,
    NoHealer,
    ShortOnDamage,
    /// The seen players could fill a party by role. Never a promise that a group will form.
    /// The roles look fine, but this node led invitations and nobody accepted any.
    NobodyAnswered,
    Healthy,
}

impl Situation {
    /// The wire code (snake_case).
    pub fn code(self) -> &'static str {
        match self {
            Situation::WarmingUp => "warming_up",
            Situation::NotConnected => "not_connected",
            Situation::OthersGrouping => "others_grouping",
            Situation::Alone => "alone",
            Situation::NoTank => "no_tank",
            Situation::NoHealer => "no_healer",
            Situation::ShortOnDamage => "short_on_damage",
            Situation::NobodyAnswered => "nobody_answered",
            Situation::Healthy => "healthy",
        }
    }
}

/// Everything `diagnose` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    /// The search expired: `seen` is the frozen peak and the warm-up rule does not apply.
    pub ended: bool,
    /// Time since the search began.
    pub elapsed: Duration,
    /// Peers currently connected to the network.
    pub network_peers: u32,
    /// Other compatible players seen (live counts, or the peak when `ended`).
    pub seen: RoleCounts,
    pub own_role: Option<Role>,
    /// This node's own attempt counters (frozen when `ended`).
    pub attempts: Attempts,
}

/// First match wins, in this order: warming up (live only), not connected, others grouping
/// (taken notices seen and a tiny sample), alone, then the role gaps counting this player in
/// their own role against the party mix of 1 tank, 1 healer and 3 damage, then nobody answered
/// (at least `MIN_ATTEMPTS` invitations sent, no accept), otherwise healthy.
pub fn diagnose(i: &Input) -> Situation {
    if !i.ended && i.elapsed < WARM_UP {
        return Situation::WarmingUp;
    }
    if i.network_peers == 0 {
        return Situation::NotConnected;
    }
    if i.attempts.taken_seen > 0 && i.seen.players < TINY_SAMPLE {
        return Situation::OthersGrouping;
    }
    if i.seen.players == 0 {
        return Situation::Alone;
    }
    let own = |r: Role| u32::from(i.own_role == Some(r));
    if i.seen.tanks + own(Role::Tank) < u32::from(TANKS_PER_PARTY) {
        Situation::NoTank
    } else if i.seen.healers + own(Role::Healer) < u32::from(HEALERS_PER_PARTY) {
        Situation::NoHealer
    } else if i.seen.damage + own(Role::Damage) < u32::from(DAMAGE_PER_PARTY) {
        Situation::ShortOnDamage
    } else if i.attempts.invites_sent >= MIN_ATTEMPTS && i.attempts.accepts_got == 0 {
        Situation::NobodyAnswered
    } else {
        Situation::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(t: u32, h: u32, d: u32) -> RoleCounts {
        RoleCounts {
            players: t + h + d,
            tanks: t,
            healers: h,
            damage: d,
        }
    }

    fn input(secs: u64, peers: u32, seen: RoleCounts, own: Option<Role>) -> Input {
        Input {
            ended: false,
            elapsed: Duration::from_secs(secs),
            network_peers: peers,
            seen,
            own_role: own,
            attempts: Attempts::default(),
        }
    }

    #[test]
    fn table() {
        use Role::*;
        use Situation::*;
        let cases = [
            // (elapsed s, peers, seen, own, expected)
            (0, 0, counts(0, 0, 0), Some(Tank), WarmingUp),
            (29, 5, counts(4, 4, 4), Some(Tank), WarmingUp),
            (30, 0, counts(4, 4, 4), Some(Tank), NotConnected),
            (30, 3, counts(0, 0, 0), Some(Tank), Alone),
            (60, 3, counts(0, 2, 5), Some(Damage), NoTank),
            (60, 3, counts(1, 0, 5), Some(Damage), NoHealer),
            (60, 3, counts(1, 1, 1), Some(Damage), ShortOnDamage),
            (60, 3, counts(1, 1, 2), Some(Damage), Healthy),
            // Own role fills the gap.
            (60, 3, counts(0, 1, 3), Some(Tank), Healthy),
            (60, 3, counts(1, 0, 3), Some(Healer), Healthy),
            // Unknown own role fills nothing.
            (60, 3, counts(0, 1, 3), None, NoTank),
        ];
        for (secs, peers, seen, own, want) in cases {
            assert_eq!(
                diagnose(&input(secs, peers, seen, own)),
                want,
                "{secs}s {peers} peers {seen:?} {own:?}"
            );
        }
    }

    fn att(sent: u32, got: u32, taken: u32) -> Attempts {
        Attempts {
            invites_sent: sent,
            accepts_got: got,
            taken_seen: taken,
            ..Attempts::default()
        }
    }

    fn with(secs: u64, peers: u32, seen: RoleCounts, own: Role, a: Attempts) -> Situation {
        let mut i = input(secs, peers, seen, Some(own));
        i.attempts = a;
        diagnose(&i)
    }

    #[test]
    fn attempts_table() {
        use Role::*;
        use Situation::*;
        let full = counts(1, 1, 3);
        let cases = [
            // others grouping: taken seen and a tiny (or empty) sample
            (
                with(60, 3, counts(0, 0, 0), Tank, att(0, 0, 2)),
                OthersGrouping,
            ),
            (
                with(60, 3, counts(0, 1, 2), Tank, att(0, 0, 1)),
                OthersGrouping,
            ),
            // four others is enough to fill a party: the explanation no longer applies
            (with(60, 3, counts(0, 1, 3), Tank, att(0, 0, 1)), Healthy),
            // no taken notice: plain alone
            (with(60, 3, counts(0, 0, 0), Tank, att(0, 0, 0)), Alone),
            // never hides not_connected or warming_up
            (
                with(60, 0, counts(0, 0, 0), Tank, att(0, 0, 5)),
                NotConnected,
            ),
            (with(10, 3, counts(0, 0, 0), Tank, att(0, 0, 5)), WarmingUp),
            // others grouping comes before the role gaps
            (
                with(60, 3, counts(0, 1, 1), Damage, att(0, 0, 1)),
                OthersGrouping,
            ),
            // nobody answered: at the threshold, roles fine, no accepts
            (with(60, 3, full, Tank, att(6, 0, 0)), NobodyAnswered),
            (with(60, 3, full, Tank, att(5, 0, 0)), Healthy),
            (with(60, 3, full, Tank, att(40, 1, 0)), Healthy),
            // a missing role is reported before unanswered invitations
            (with(60, 3, counts(0, 1, 3), Healer, att(9, 0, 0)), NoTank),
            // a big sample with taken notices is not "others grouping"
            (with(60, 3, full, Tank, att(6, 0, 3)), NobodyAnswered),
        ];
        for (n, (got, want)) in cases.into_iter().enumerate() {
            assert_eq!(got, want, "case {n}");
        }
    }

    #[test]
    fn ended_ignores_warm_up_and_uses_the_given_counts() {
        let mut i = input(10, 2, counts(0, 3, 3), Some(Role::Damage));
        i.ended = true;
        assert_eq!(diagnose(&i), Situation::NoTank);
        i.network_peers = 0;
        assert_eq!(diagnose(&i), Situation::NotConnected);
    }

    #[test]
    fn healthy_never_occurs_when_a_role_is_missing() {
        let roles = [
            None,
            Some(Role::Tank),
            Some(Role::Healer),
            Some(Role::Damage),
        ];
        for own in roles {
            for t in 0..3 {
                for h in 0..3 {
                    for d in 0..5 {
                        let got = diagnose(&input(120, 1, counts(t, h, d), own));
                        if got != Situation::Healthy {
                            continue;
                        }
                        let tot = |r: Role, n: u32| n + u32::from(own == Some(r));
                        assert!(tot(Role::Tank, t) >= 1, "{t} {h} {d} {own:?}");
                        assert!(tot(Role::Healer, h) >= 1, "{t} {h} {d} {own:?}");
                        assert!(tot(Role::Damage, d) >= 3, "{t} {h} {d} {own:?}");
                        assert!(t + h + d > 0);
                    }
                }
            }
        }
    }

    #[test]
    fn healthy_never_occurs_with_unanswered_invitations() {
        for t in 0..3 {
            for h in 0..3 {
                for d in 0..5 {
                    let got = with(
                        120,
                        1,
                        counts(t, h, d),
                        Role::Damage,
                        att(MIN_ATTEMPTS, 0, 0),
                    );
                    assert_ne!(got, Situation::Healthy, "{t} {h} {d}");
                }
            }
        }
    }

    #[test]
    fn codes_are_snake_case() {
        assert_eq!(Situation::OthersGrouping.code(), "others_grouping");
        assert_eq!(Situation::NobodyAnswered.code(), "nobody_answered");
        assert_eq!(Situation::ShortOnDamage.code(), "short_on_damage");
        assert_eq!(Situation::WarmingUp.code(), "warming_up");
    }
}
