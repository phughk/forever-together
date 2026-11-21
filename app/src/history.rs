//! Search history (ADR 0027): one small entry per finished search, kept in the key-value store
//! so data accumulates. Nothing reads it for the player yet (estimates are stage 8b), and it is
//! never part of the API, logs, exports or pairing.
//!
//! Pure functions over `&dyn Store` with the time passed in, so `MemStore` and a fake clock
//! cover the tests. The engine only emits a `SearchFacts` record; the app layer builds the
//! entry and writes it at the terminal event, never at a checkpoint. A search that is still
//! running when the app quits loses its entry: there is no shutdown hook and that is accepted.
//!
//! Entries hold no name, no peer id and no zone text. They do hold level, own role, dungeon,
//! realm, faction and UTC times, which is a log of when the player plays: see the ADR for the
//! retention rules (clear action, cleared with the character).

use crate::code::{Faction, Role};
use crate::config::{
    AD_EXPIRY, CHECKPOINT_1, CHECKPOINT_2, EPOCH, MIN_RECORDED_STOP, SAMPLE_SIZE, SEARCH_LIMIT,
    WIDEN_STEP,
};
use crate::engine::{Checkpoint, RoleCounts, SearchFacts, SearchOutcome};
use crate::store::{PREFIX_HISTORY, Store, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The entry layout. An entry with another value is skipped when read, never an error.
pub const SCHEMA_VERSION: u32 = 1;
/// What the counts mean: epoch length (s), widening step (s), sample size, limit (s), ad
/// lifetime (s), checkpoints (s) and the matching-rules version (bump the trailing `match`
/// number when the compatibility or counting rule in `engine` changes). A test keeps this equal
/// to the config values.
pub const RULES_VERSION: &str = "epoch5-widen60-sample100-limit600-adlife120-cp55-115-match1";

/// Starting values, all unverified (ADR 0027).
pub const MAX_AGE_SECS: u64 = 180 * 24 * 60 * 60;
pub const MAX_PER_DUNGEON: usize = 100;
pub const MAX_TOTAL: usize = 500;

pub fn rules_version_from_config() -> String {
    format!(
        "epoch{}-widen{}-sample{}-limit{}-adlife{}-cp{}-{}-match1",
        EPOCH.as_secs(),
        WIDEN_STEP.as_secs(),
        SAMPLE_SIZE,
        SEARCH_LIMIT.as_secs(),
        AD_EXPIRY.as_secs(),
        CHECKPOINT_1.as_secs(),
        CHECKPOINT_2.as_secs()
    )
}

/// How a recorded search ended. `Ignored` searches are never recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// All members confirmed.
    Formed,
    /// The limit passed: a failure sample.
    Expired,
    /// The player stopped after more than a minute. Incomplete data, not a failure: a later
    /// estimator must treat it as censored ("at least this long").
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub players: u32,
    pub tanks: u32,
    pub healers: u32,
    pub damage: u32,
}

impl From<RoleCounts> for Counts {
    fn from(c: RoleCounts) -> Self {
        Self {
            players: c.players,
            tanks: c.tanks,
            healers: c.healers,
            damage: c.damage,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointEntry {
    pub counts: Counts,
    /// The widening stage in force at that tick.
    pub stage: u8,
    pub sample_full: bool,
}

impl From<Checkpoint> for CheckpointEntry {
    fn from(c: Checkpoint) -> Self {
        Self {
            counts: c.counts.into(),
            stage: c.stage,
            sample_full: c.sample_full,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub schema_version: u32,
    pub rules_version: String,
    /// UTC seconds. The start is the key.
    pub start_utc: u64,
    pub end_utc: u64,
    /// Local offset from UTC in seconds, so local time of day can be derived later. 0 while the
    /// app cannot learn it (see `app::local_utc_offset_secs`).
    pub utc_offset_secs: i32,
    pub region: u8,
    pub realm: String,
    pub faction: String,
    /// The first dungeon chosen.
    pub dungeon: String,
    pub topic_count: u32,
    pub level: u8,
    pub own_role: String,
    /// The counts about 1 and 2 minutes in. `None` when the search ended first, never zeros.
    pub checkpoint_1: Option<CheckpointEntry>,
    pub checkpoint_2: Option<CheckpointEntry>,
    /// Each role's own maximum over the search (not the roles at one moment).
    pub peak: Counts,
    pub stage_reached: u8,
    pub outcome: Outcome,
    /// Parties that fell apart because a member did not answer.
    pub drops: u32,
    /// Parties the player declined or left unconfirmed, or that timed out: not a failure.
    pub declines: u32,
    /// Seconds from the start to the first commit, if a party ever formed.
    pub first_formed_at: Option<u64>,
    /// Invitations this node sent as a leader (one per invited peer). Added to schema 1 without
    /// a bump: entries written before it read as 0.
    #[serde(default)]
    pub invites_sent: u32,
    /// Accepts received for them.
    #[serde(default)]
    pub accepts_got: u32,
    /// Distinct peers whose ads left the sample because a party-taken notice named them.
    #[serde(default)]
    pub taken_seen: u32,
}

impl Entry {
    pub fn key(&self) -> String {
        key_for(self.start_utc)
    }
}

pub fn key_for(start_utc: u64) -> String {
    format!("{PREFIX_HISTORY}{start_utc:012}")
}

/// What the engine does not know about a search.
#[derive(Debug, Clone)]
pub struct Context {
    pub end_utc: u64,
    pub utc_offset_secs: i32,
    pub region: u8,
    pub realm: String,
    pub faction: Faction,
    pub dungeon: String,
}

fn role_name(r: Role) -> &'static str {
    match r {
        Role::Tank => "tank",
        Role::Healer => "healer",
        Role::Damage => "damage",
    }
}

/// The entry for a finished search, or `None` when the outcome is not recorded: `Ignored`, and a
/// player stop of a minute or less.
pub fn entry_from_facts(facts: &SearchFacts, ctx: &Context) -> Option<Entry> {
    let outcome = match facts.outcome {
        SearchOutcome::Ignored => return None,
        SearchOutcome::Stopped if facts.elapsed <= MIN_RECORDED_STOP => return None,
        SearchOutcome::Stopped => Outcome::Stopped,
        SearchOutcome::Expired => Outcome::Expired,
        SearchOutcome::Formed => Outcome::Formed,
    };
    Some(Entry {
        schema_version: SCHEMA_VERSION,
        rules_version: RULES_VERSION.to_string(),
        start_utc: ctx.end_utc.saturating_sub(facts.elapsed.as_secs()),
        end_utc: ctx.end_utc,
        utc_offset_secs: ctx.utc_offset_secs,
        region: ctx.region,
        realm: ctx.realm.clone(),
        faction: match ctx.faction {
            Faction::Alliance => "alliance",
            Faction::Horde => "horde",
        }
        .to_string(),
        dungeon: ctx.dungeon.clone(),
        topic_count: facts.topic_count as u32,
        level: facts.level,
        own_role: role_name(facts.own_role).to_string(),
        checkpoint_1: facts.checkpoint_1.map(Into::into),
        checkpoint_2: facts.checkpoint_2.map(Into::into),
        peak: facts.peak.into(),
        stage_reached: facts.stage_reached,
        outcome,
        drops: facts.drops,
        declines: facts.declines,
        first_formed_at: facts.first_formed_at.map(|d| d.as_secs()),
        invites_sent: facts.invites_sent,
        accepts_got: facts.accepts_got,
        taken_seen: facts.taken_seen,
    })
}

/// Write the entry, then prune as of its end time.
pub fn record(store: &dyn Store, entry: &Entry) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec(entry).map_err(|e| StoreError::Decode(e.to_string()))?;
    store.put(&entry.key(), &bytes)?;
    prune(store, entry.end_utc)
}

/// Record the facts of a finished search. A failed write is logged (never the entry) and the
/// search carries on. Returns whether an entry was stored.
pub fn record_facts(store: &dyn Store, facts: &SearchFacts, ctx: &Context) -> bool {
    let Some(entry) = entry_from_facts(facts, ctx) else {
        return false;
    };
    match record(store, &entry) {
        Ok(()) => true,
        Err(e) => {
            tracing::warn!("search history not saved: {e}");
            false
        }
    }
}

/// The readable entries with their keys, oldest first. A corrupt entry or one with another
/// schema version is skipped (and left in place: a newer build may read it).
pub fn load(store: &dyn Store) -> Result<Vec<(String, Entry)>, StoreError> {
    Ok(store
        .list(PREFIX_HISTORY)?
        .into_iter()
        .filter_map(|(k, v)| {
            let e: Entry = serde_json::from_slice(&v).ok()?;
            (e.schema_version == SCHEMA_VERSION).then_some((k, e))
        })
        .collect())
}

/// Keep the history bounded. Order: entries older than 180 days, then more than 100 per
/// dungeon, then more than 500 in all, oldest first each time. Each delete stands alone, so a
/// crash half way only leaves the next prune some work.
pub fn prune(store: &dyn Store, now: u64) -> Result<(), StoreError> {
    let mut rows = load(store)?;
    rows.sort_by(|a, b| (a.1.start_utc, &a.0).cmp(&(b.1.start_utc, &b.0)));
    let oldest_kept = now.saturating_sub(MAX_AGE_SECS);
    let mut doomed: Vec<String> = Vec::new();
    rows.retain(|(k, e)| {
        let keep = e.start_utc >= oldest_kept;
        if !keep {
            doomed.push(k.clone());
        }
        keep
    });
    let mut per: BTreeMap<&str, Vec<&String>> = BTreeMap::new();
    for (k, e) in &rows {
        per.entry(e.dungeon.as_str()).or_default().push(k);
    }
    let mut over: Vec<String> = Vec::new();
    for keys in per.values() {
        let extra = keys.len().saturating_sub(MAX_PER_DUNGEON);
        over.extend(keys.iter().take(extra).map(|k| (*k).clone()));
    }
    rows.retain(|(k, _)| !over.contains(k));
    doomed.extend(over);
    let extra = rows.len().saturating_sub(MAX_TOTAL);
    doomed.extend(rows.iter().take(extra).map(|(k, _)| k.clone()));
    for k in doomed {
        store.delete(&k)?;
    }
    Ok(())
}

/// Delete every history entry, including unreadable ones.
pub fn clear(store: &dyn Store) -> Result<(), StoreError> {
    for (k, _) in store.list(PREFIX_HISTORY)? {
        store.delete(&k)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::store::MemStore;
    use std::time::Duration;

    const DAY: u64 = 24 * 60 * 60;

    pub fn entry(start: u64, dungeon: &str) -> Entry {
        Entry {
            schema_version: SCHEMA_VERSION,
            rules_version: RULES_VERSION.into(),
            start_utc: start,
            end_utc: start + 600,
            utc_offset_secs: 0,
            region: 3,
            realm: "15".into(),
            faction: "alliance".into(),
            dungeon: dungeon.into(),
            topic_count: 1,
            level: 24,
            own_role: "tank".into(),
            checkpoint_1: None,
            checkpoint_2: None,
            peak: Counts {
                players: 0,
                tanks: 0,
                healers: 0,
                damage: 0,
            },
            stage_reached: 0,
            outcome: Outcome::Expired,
            drops: 0,
            declines: 0,
            first_formed_at: None,
            invites_sent: 0,
            accepts_got: 0,
            taken_seen: 0,
        }
    }

    fn put(s: &MemStore, e: &Entry) {
        s.put(&e.key(), &serde_json::to_vec(e).unwrap()).unwrap();
    }

    fn facts(outcome: SearchOutcome, secs: u64) -> SearchFacts {
        SearchFacts {
            outcome,
            elapsed: Duration::from_secs(secs),
            stage_reached: 1,
            own_role: Role::Tank,
            level: 24,
            topic_count: 2,
            peak: RoleCounts {
                players: 3,
                tanks: 0,
                healers: 1,
                damage: 2,
            },
            first_formed_at: None,
            drops: 0,
            declines: 0,
            checkpoint_1: None,
            checkpoint_2: None,
            invites_sent: 0,
            accepts_got: 0,
            taken_seen: 0,
        }
    }

    fn ctx(end: u64) -> Context {
        Context {
            end_utc: end,
            utc_offset_secs: 0,
            region: 3,
            realm: "15".into(),
            faction: Faction::Alliance,
            dungeon: "testville-depths".into(),
        }
    }

    fn keys(s: &MemStore) -> Vec<String> {
        s.list(PREFIX_HISTORY)
            .unwrap()
            .into_iter()
            .map(|(k, _)| k)
            .collect()
    }

    #[test]
    fn rules_version_matches_the_config() {
        assert_eq!(RULES_VERSION, rules_version_from_config());
    }

    #[test]
    fn keys_are_zero_padded_and_sort_by_time() {
        assert_eq!(key_for(42), "history/000000000042");
        let s = MemStore::new();
        for t in [1_700_000_000, 999, 1_800_000_000] {
            put(&s, &entry(t, "d"));
        }
        assert_eq!(
            keys(&s),
            [
                "history/000000000999",
                "history/001700000000",
                "history/001800000000"
            ]
        );
        let starts: Vec<u64> = load(&s).unwrap().iter().map(|(_, e)| e.start_utc).collect();
        assert_eq!(starts, [999, 1_700_000_000, 1_800_000_000]);
    }

    #[test]
    fn entry_round_trips_and_carries_no_identity() {
        let s = MemStore::new();
        let mut f = facts(SearchOutcome::Expired, 600);
        f.first_formed_at = Some(Duration::from_secs(70));
        f.drops = 1;
        assert!(record_facts(&s, &f, &ctx(1_800_000_600)));
        let (_, e) = load(&s).unwrap().remove(0);
        assert_eq!(e.start_utc, 1_800_000_000);
        assert_eq!(e.end_utc, 1_800_000_600);
        assert_eq!(e.outcome, Outcome::Expired);
        assert_eq!(e.first_formed_at, Some(70));
        assert_eq!((e.drops, e.declines), (1, 0));
        assert_eq!(e.dungeon, "testville-depths");
        assert_eq!(e.topic_count, 2);
        assert_eq!(e.peak.damage, 2);
        assert_eq!(e.rules_version, RULES_VERSION);
    }

    #[test]
    fn entry_carries_the_attempt_counters_and_old_entries_still_decode() {
        let s = MemStore::new();
        let mut f = facts(SearchOutcome::Expired, 600);
        (f.invites_sent, f.accepts_got, f.taken_seen) = (14, 2, 5);
        assert!(record_facts(&s, &f, &ctx(1_800_000_600)));
        let (_, e) = load(&s).unwrap().remove(0);
        assert_eq!((e.invites_sent, e.accepts_got, e.taken_seen), (14, 2, 5));
        // An entry written before the counters existed has no such keys: schema 1 still reads.
        let mut v: serde_json::Value = serde_json::to_value(&e).unwrap();
        for k in ["invites_sent", "accepts_got", "taken_seen"] {
            v.as_object_mut().unwrap().remove(k);
        }
        let old: Entry = serde_json::from_value(v).unwrap();
        assert_eq!(
            (old.invites_sent, old.accepts_got, old.taken_seen),
            (0, 0, 0)
        );
        assert_eq!(old.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn outcome_rules() {
        let s = MemStore::new();
        let c = ctx(1_800_000_000);
        assert!(!record_facts(&s, &facts(SearchOutcome::Ignored, 300), &c));
        assert!(!record_facts(&s, &facts(SearchOutcome::Stopped, 30), &c));
        assert!(!record_facts(&s, &facts(SearchOutcome::Stopped, 60), &c));
        assert!(keys(&s).is_empty());
        assert!(record_facts(&s, &facts(SearchOutcome::Stopped, 90), &c));
        assert_eq!(load(&s).unwrap()[0].1.outcome, Outcome::Stopped);
        let c = ctx(1_800_001_000);
        assert!(record_facts(&s, &facts(SearchOutcome::Formed, 200), &c));
        assert!(record_facts(&s, &facts(SearchOutcome::Expired, 600), &c));
        let outs: Vec<Outcome> = load(&s).unwrap().iter().map(|(_, e)| e.outcome).collect();
        assert_eq!(outs, [Outcome::Stopped, Outcome::Expired, Outcome::Formed]);
    }

    #[test]
    fn prune_drops_entries_older_than_180_days() {
        let s = MemStore::new();
        let now = 400 * DAY;
        put(&s, &entry(now - 181 * DAY, "d"));
        put(&s, &entry(now - 179 * DAY, "d"));
        prune(&s, now).unwrap();
        assert_eq!(keys(&s), [key_for(now - 179 * DAY)]);
    }

    #[test]
    fn prune_per_dungeon_cap_alone() {
        let s = MemStore::new();
        for i in 0..105u64 {
            put(&s, &entry(1_800_000_000 + i, "a"));
        }
        for i in 0..5u64 {
            put(&s, &entry(1_800_001_000 + i, "b"));
        }
        prune(&s, 1_800_002_000).unwrap();
        let rows = load(&s).unwrap();
        assert_eq!(rows.len(), 105);
        assert_eq!(rows[0].1.start_utc, 1_800_000_005);
        assert_eq!(rows.iter().filter(|(_, e)| e.dungeon == "b").count(), 5);
    }

    #[test]
    fn prune_applies_age_then_per_dungeon_then_total() {
        let s = MemStore::new();
        let base = 1_800_000_000u64;
        // 5 entries older than 180 days, then 120 in "a" and 100 each in "b" to "f".
        let now = base + 100 * DAY;
        for i in 0..5 {
            put(&s, &entry(base - 90 * DAY + i, "old"));
        }
        let mut t = base;
        for _ in 0..120 {
            put(&s, &entry(t, "a"));
            t += 1;
        }
        for d in ["b", "c", "d", "e", "f"] {
            for _ in 0..100 {
                put(&s, &entry(t, d));
                t += 1;
            }
        }
        prune(&s, now).unwrap();
        let rows = load(&s).unwrap();
        // Age removed "old". "a" lost its 20 oldest (100 left, 600 in all). The total cap then
        // removed the 100 oldest, which are all of "a".
        assert_eq!(rows.len(), MAX_TOTAL);
        assert!(
            rows.iter()
                .all(|(_, e)| e.dungeon != "old" && e.dungeon != "a")
        );
        assert_eq!(rows[0].1.start_utc, base + 120);
        assert!(rows.windows(2).all(|w| w[0].1.start_utc < w[1].1.start_utc));
    }

    #[test]
    fn corrupt_and_unknown_version_entries_are_skipped() {
        let s = MemStore::new();
        record(&s, &entry(1_800_000_000, "d")).unwrap();
        s.put("history/000000000001", b"not json").unwrap();
        let mut future = entry(1_800_000_100, "d");
        future.schema_version = 99;
        put(&s, &future);
        s.put("history/000000000002", br#"{"schema_version":1}"#)
            .unwrap();
        let rows = load(&s).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1.start_utc, 1_800_000_000);
        // Pruning copes and leaves the unreadable ones alone.
        prune(&s, 1_800_000_200).unwrap();
        assert_eq!(keys(&s).len(), 4);
    }

    #[test]
    fn clear_removes_everything_under_the_prefix_only() {
        let s = MemStore::new();
        record(&s, &entry(1_800_000_000, "d")).unwrap();
        s.put("history/000000000001", b"junk").unwrap();
        s.put("settings", b"{}").unwrap();
        clear(&s).unwrap();
        assert!(keys(&s).is_empty());
        assert!(s.get("settings").unwrap().is_some());
    }

    /// A store whose history writes fail (other keys work, so a character can still be saved).
    pub struct FailingStore(pub MemStore);
    impl Store for FailingStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            self.0.get(key)
        }
        fn put(&self, key: &str, value: &[u8]) -> Result<(), StoreError> {
            if key.starts_with(PREFIX_HISTORY) {
                return Err(StoreError::Backend("disk full".into()));
            }
            self.0.put(key, value)
        }
        fn delete(&self, key: &str) -> Result<bool, StoreError> {
            self.0.delete(key)
        }
        fn list(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
            self.0.list(prefix)
        }
    }

    #[test]
    fn a_failed_write_is_reported_not_raised() {
        let s = FailingStore(MemStore::new());
        assert!(!record_facts(
            &s,
            &facts(SearchOutcome::Expired, 600),
            &ctx(1_800_000_600)
        ));
    }
}
