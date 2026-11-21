//! Search widening (ADR 0011, plan stage table). The schedule is part of the matching
//! algorithm: nothing here can be paused, skipped or changed from outside.

use crate::config::{SEARCH_LIMIT, WIDEN_STEP};
use std::time::Duration;

pub const LAST_STAGE: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
    pub level_spread: u8,
    /// Allowed item level difference as a percent of the lower value.
    pub item_level_percent: u32,
    /// Secondary roles may fill a slot. Always false in effect in v1: the code carries none.
    pub secondary_roles: bool,
    /// The other recommended dungeons are searched too.
    pub other_dungeons: bool,
}

/// Stage 0: ilvl 15%, spread 3, exact roles. 1: secondary roles. 2: ilvl 30%.
/// 3: spread 6. 4: spread 10 and other dungeons.
pub fn rules(stage: u8) -> Rules {
    let s = stage.min(LAST_STAGE);
    Rules {
        level_spread: match s {
            0..=2 => 3,
            3 => 6,
            _ => 10,
        },
        item_level_percent: if s >= 2 { 30 } else { 15 },
        secondary_roles: s >= 1,
        other_dungeons: s >= 4,
    }
}

/// The stage reached `elapsed` after the stage clock started.
pub fn stage_at(elapsed: Duration) -> u8 {
    ((elapsed.as_secs() / WIDEN_STEP.as_secs()) as u8).min(LAST_STAGE)
}

/// Time until the next stage, `None` at the last stage.
pub fn until_next_stage(elapsed: Duration) -> Option<Duration> {
    let s = stage_at(elapsed);
    if s >= LAST_STAGE {
        None
    } else {
        Some(WIDEN_STEP * (s as u32 + 1) - elapsed)
    }
}

pub fn search_expired(since_search_start: Duration) -> bool {
    since_search_start >= SEARCH_LIMIT
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(s: u64) -> Duration {
        Duration::from_secs(s)
    }

    #[test]
    fn stages_follow_the_table() {
        assert_eq!(stage_at(secs(0)), 0);
        assert_eq!(stage_at(secs(59)), 0);
        assert_eq!(stage_at(secs(60)), 1);
        assert_eq!(stage_at(secs(120)), 2);
        assert_eq!(stage_at(secs(180)), 3);
        assert_eq!(stage_at(secs(240)), 4);
        assert_eq!(stage_at(secs(590)), 4);
    }

    #[test]
    fn rules_per_stage() {
        let r0 = rules(0);
        assert_eq!(
            (r0.level_spread, r0.item_level_percent, r0.other_dungeons),
            (3, 15, false)
        );
        assert!(!r0.secondary_roles && rules(1).secondary_roles);
        assert_eq!(rules(1).item_level_percent, 15);
        assert_eq!(rules(2).item_level_percent, 30);
        assert_eq!(rules(2).level_spread, 3);
        assert_eq!(rules(3).level_spread, 6);
        assert_eq!(rules(4).level_spread, 10);
        assert!(rules(4).other_dungeons && !rules(3).other_dungeons);
        // Stages never get stricter.
        for s in 0..LAST_STAGE {
            assert!(rules(s + 1).level_spread >= rules(s).level_spread);
            assert!(rules(s + 1).item_level_percent >= rules(s).item_level_percent);
        }
    }

    #[test]
    fn countdown_and_limit() {
        assert_eq!(until_next_stage(secs(10)), Some(secs(50)));
        assert_eq!(until_next_stage(secs(60)), Some(secs(60)));
        assert_eq!(until_next_stage(secs(250)), None);
        assert!(!search_expired(secs(599)));
        assert!(search_expired(secs(600)));
    }
}
