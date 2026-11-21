//! Protocol constants (not user settings: changing them would break agreement) and user
//! settings with load and save (plan "Constants and settings").

use crate::store::{KEY_SETTINGS, Store, StoreError, get_json, put_json};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Epoch length: one round of leader invitations (ADR 0025). It was 10 s for converging
/// announcements; an invite, accept and commit exchange needs about 2 s, so 5 s fits.
pub const EPOCH: Duration = Duration::from_secs(5);
/// Ad keep-alive: an unchanged ad is re-sent this often.
pub const AD_KEEPALIVE: Duration = Duration::from_secs(45);
/// An ad not refreshed within this is dropped.
pub const AD_EXPIRY: Duration = Duration::from_secs(120);
/// Only ads first seen at least this long before an epoch start are used for that epoch.
pub const AD_CUTOFF: Duration = Duration::from_secs(2);
/// Ads kept per topic.
pub const SAMPLE_SIZE: usize = 100;
/// Ready and heartbeat reply window.
pub const READY_WINDOW: Duration = Duration::from_secs(4);
/// Human Confirm click window.
pub const CONFIRM_WINDOW: Duration = Duration::from_secs(15);
/// Leader invitations (ADR 0025): an invitee answers the invitations of an epoch this long after
/// the epoch starts, so invitations from slower leaders are in the same decision. A leader
/// commits as soon as its acceptors make a party, so a party forms about this long into the
/// epoch.
pub const INVITE_WINDOW: Duration = Duration::from_secs(2);
/// Local ban after no heartbeat reply. Never gossiped.
pub const BAN: Duration = Duration::from_secs(60);
/// Search time limit.
pub const SEARCH_LIMIT: Duration = Duration::from_secs(600);
/// Widening step.
pub const WIDEN_STEP: Duration = Duration::from_secs(60);
/// Search-history checkpoints (ADR 0027), measured from the search start. Each sits just before
/// a widening step so the counts describe the rules still in force, not the next stage's:
/// 55 s is before the 60 s step (`WIDEN_STEP`), 115 s before the 120 s step, which is also the
/// ad lifetime (`AD_EXPIRY`), so counts taken later would already be dropping stale ads.
pub const CHECKPOINT_1: Duration = Duration::from_secs(55);
pub const CHECKPOINT_2: Duration = Duration::from_secs(115);
/// A player stop shorter than this is not recorded in the search history (ADR 0027).
pub const MIN_RECORDED_STOP: Duration = Duration::from_secs(60);
/// Peer exchange: at most one request per this long per peer.
pub const PEER_EXCHANGE_MIN_INTERVAL: Duration = Duration::from_secs(10);
/// Trusted peer retry backoff: first delay and cap.
pub const TRUSTED_BACKOFF_START: Duration = Duration::from_secs(30);
pub const TRUSTED_BACKOFF_CAP: Duration = Duration::from_secs(15 * 60);
pub const TRUSTED_BACKOFF_JITTER: f64 = 0.20;

/// Delay before retry number `attempt` (0 is the first retry) against the trusted peers.
/// `unit` is a random number in `[0, 1)`. The jitter (plus or minus 20%) is applied before the
/// cap, so the result never exceeds `TRUSTED_BACKOFF_CAP`.
pub fn trusted_backoff(attempt: u32, unit: f64) -> Duration {
    let base = TRUSTED_BACKOFF_START.as_secs_f64() * 2f64.powi(attempt.min(30) as i32);
    let factor = 1.0 + TRUSTED_BACKOFF_JITTER * (2.0 * unit.clamp(0.0, 1.0) - 1.0);
    Duration::from_secs_f64((base * factor).min(TRUSTED_BACKOFF_CAP.as_secs_f64()))
}

/// User settings (plan: LAN mode, known-peers cache size, extra trusted peers).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub lan_mode: bool,
    /// The LAN address the pairing link uses; `None` picks the one on the active network.
    pub lan_address: Option<String>,
    pub known_peers_cache: usize,
    /// Extra trusted peer keys, hex.
    pub trusted_peers: Vec<String>,
    /// The game's `Interface/AddOns` folder the addon is installed into (canonical path). Set
    /// only through `PUT /api/addon/folder`, which is loopback only; not character data.
    pub addon_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lan_mode: false,
            lan_address: None,
            known_peers_cache: 100,
            trusted_peers: Vec::new(),
            addon_dir: None,
        }
    }
}

impl Settings {
    pub fn load(store: &dyn Store) -> Result<Self, StoreError> {
        Ok(get_json(store, KEY_SETTINGS)?.unwrap_or_default())
    }
    pub fn save(&self, store: &dyn Store) -> Result<(), StoreError> {
        put_json(store, KEY_SETTINGS, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemStore;

    #[test]
    fn defaults_match_the_plan() {
        let s = Settings::default();
        assert!(!s.lan_mode);
        assert_eq!(s.known_peers_cache, 100);
        assert!(s.trusted_peers.is_empty());
        assert_eq!(EPOCH.as_secs(), 5);
        assert_eq!(SAMPLE_SIZE, 100);
        assert_eq!(CONFIRM_WINDOW.as_secs(), 15);
        assert_eq!(BAN.as_secs(), 60);
        assert_eq!(SEARCH_LIMIT.as_secs(), 600);
    }

    #[test]
    fn settings_round_trip_and_default_when_missing() {
        let store = MemStore::new();
        assert_eq!(Settings::load(&store).unwrap(), Settings::default());
        let s = Settings {
            lan_mode: true,
            lan_address: Some("10.0.0.7".into()),
            known_peers_cache: 7,
            trusted_peers: vec!["ab".into()],
            addon_dir: Some("/games/wow/_forever_/Interface/AddOns".into()),
        };
        s.save(&store).unwrap();
        assert_eq!(Settings::load(&store).unwrap(), s);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let store = MemStore::new();
        store.put(KEY_SETTINGS, br#"{"lan_mode":true}"#).unwrap();
        let s = Settings::load(&store).unwrap();
        assert!(s.lan_mode);
        assert_eq!(s.known_peers_cache, 100);
        assert_eq!(s.addon_dir, None);
    }

    #[test]
    fn backoff_doubles_and_is_capped_with_jitter_before_the_cap() {
        // Middle of the jitter range: exactly the base delay.
        assert_eq!(trusted_backoff(0, 0.5), Duration::from_secs(30));
        assert_eq!(trusted_backoff(1, 0.5), Duration::from_secs(60));
        assert_eq!(trusted_backoff(2, 0.5), Duration::from_secs(120));
        // Jitter bounds of 20%.
        assert_eq!(trusted_backoff(0, 0.0), Duration::from_secs(24));
        assert_eq!(trusted_backoff(0, 1.0), Duration::from_secs(36));
        // The cap holds even with the largest jitter.
        for attempt in 0..40 {
            for u in [0.0, 0.3, 0.999, 1.0] {
                assert!(trusted_backoff(attempt, u) <= TRUSTED_BACKOFF_CAP);
            }
        }
        assert_eq!(trusted_backoff(20, 1.0), TRUSTED_BACKOFF_CAP);
    }
}
