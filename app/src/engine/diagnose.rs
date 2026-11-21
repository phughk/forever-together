//! Search diagnosis (ADR 0026): a pure map from what this node observed to a situation code.
//! No clock and no I/O, so the same observations always give the same situation. The backend
//! sends the code and the counts; the web app owns the wording.
//!
//! Every count is a self-reported claim seen by this node only (ADR 0003): "none seen" never
//! means "none exist". All thresholds are unverified until a live run (docs/spike-s0.md).

use super::party::{DAMAGE_PER_PARTY, HEALERS_PER_PARTY, TANKS_PER_PARTY};
use super::state::RoleCounts;
use crate::code::Role;
use std::time::Duration;

/// Before this much search time no verdict is given. Source: ads are re-sent every keep-alive
/// and a joining peer needs a few epochs (5 s each) to be heard; 30 s is a guess, unverified.
pub const WARM_UP: Duration = Duration::from_secs(30);

/// What the player is told is happening. The web app maps each code to a sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situation {
    /// Too early to say.
    WarmingUp,
    /// No peers are connected: nothing can be seen.
    NotConnected,
    /// Connected, but no compatible player seen.
    Alone,
    NoTank,
    NoHealer,
    ShortOnDamage,
    /// The seen players could fill a party by role. Never a promise that a group will form.
    Healthy,
}

impl Situation {
    /// The wire code (snake_case).
    pub fn code(self) -> &'static str {
        match self {
            Situation::WarmingUp => "warming_up",
            Situation::NotConnected => "not_connected",
            Situation::Alone => "alone",
            Situation::NoTank => "no_tank",
            Situation::NoHealer => "no_healer",
            Situation::ShortOnDamage => "short_on_damage",
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
}

/// First match wins, in this order: warming up (live only), not connected, alone, then the
/// role gaps counting this player in their own role against the party mix of 1 tank, 1 healer
/// and 3 damage, otherwise healthy.
pub fn diagnose(i: &Input) -> Situation {
    if !i.ended && i.elapsed < WARM_UP {
        return Situation::WarmingUp;
    }
    if i.network_peers == 0 {
        return Situation::NotConnected;
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
    fn codes_are_snake_case() {
        assert_eq!(Situation::ShortOnDamage.code(), "short_on_damage");
        assert_eq!(Situation::WarmingUp.code(), "warming_up");
    }
}
