# 0027. The app keeps a local history of finished searches, one entry each, and shows none of it

* **Status:** Accepted for stage 8a (record only). Extends [0008](0008-iroh-and-local-storage.md) (local storage). Amends [0013](0013-single-rust-crate.md) (the module list gains `history`). Cites [0003](0003-self-reported-data.md) and [0026](0026-search-health-from-own-observations.md). Every number below is an unverified starting value.
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski (resolutions 5, 9, 11 to 14 in `docs/notes/search_diagnosis_concepts_brief.md`)

## Context

[0026](0026-search-health-from-own-observations.md) tells the player what this node saw during one search. A later estimate ("searches like this usually find a group") needs the results of earlier searches, and the data takes weeks to collect: a player makes a few searches a week. The thresholds and the counting rule are still unverified against a live network. So the first step is to record, and to change nothing the player sees.

## Decision

* **Entries, not buckets.** One entry per search, written once, at the terminal event. A table by dungeon, day and hour was rejected: it would blame the clock for what is usually a missing role.
* **Stage 8a records only.** No estimate, no new field on `AppState` or `SearchView`, no UI. History is never shown as data. Estimates and any player-visible wording are stage 8b, and wait for a live capture that shows the fields are stable.
* **Entry fields** (`history::Entry`, serde): `schema_version`, `rules_version`; `start_utc` and `end_utc` in UTC seconds; `utc_offset_secs` and `start_utc_offset_secs`; `region`, `realm` and `faction` (each is a separate network, so a fit must not mix them); `dungeon` (the first one chosen) and `topic_count`; `level` at the start; `own_role`; `checkpoint_1` and `checkpoint_2`; `peak`; `stage_reached`; `outcome`; `drops`, `declines`; `first_formed_at`. No name, no peer id, no zone text ([0003](0003-self-reported-data.md): all counts are self-reported claims from a sample).
* **Outcomes.**
  * `formed`: every member confirmed (`Mode::Final`).
  * `expired`: the limit passed. A failure sample.
  * `stopped`: the player stopped after more than 60 s. Incomplete data, not a failure: the app's own advice causes stops, and a group might have come later. A later estimator treats it as censored ("at least this long"). A stop of 60 s or less is not recorded.
  * Ignored, not recorded: a restart, a character change or clearing the character, and anything else that is not a result. A quit loses the search in progress: there is no shutdown hook, and that is accepted.
  * A drop after a commit returns the engine to searching and does not end the entry: it adds to `drops` (a member did not answer) and `first_formed_at` keeps the first commit. A declined or unconfirmed party adds to `declines`: the player's choice, not a failure.
* **Checkpoints.** The counts of compatible players (total and by role), the widening stage in force and `sample_full`, taken on the first searching tick at or after 55 s and at 115 s from the search start (`config::CHECKPOINT_1`, `CHECKPOINT_2`). They sit just before the 60 s and 120 s widening steps, and 120 s is also the ad lifetime, so the counts describe the rules still in force. A search that ends first stores `None`, never zeros. No checkpoint is taken while a party is committed.
* **Peak.** Each role's own maximum over the search (the roles need not come from one moment), from the same compatible counts as [0026](0026-search-health-from-own-observations.md).
* **Where it is recorded.** The engine stays pure: no clock, no I/O. On each terminal event it emits a plain `SearchFacts` record (`Action::SearchEnded`), with durations from its own `now` values using `saturating_sub`. The app layer (`Model`) owns the clock, the store and the lifecycle: it adds region, realm, faction, dungeon and the wall-clock time, builds the entry and calls `history::record_facts`. A character change turns the engine's `Stopped` into an ignored search. The write happens only at the terminal event, and a failed write is logged (never the entry) and the search carries on.
* **Time.** UTC seconds. Each timestamp is stored with its own local offset, so local time is derivable: local end time = `end_utc + utc_offset_secs`, local start time = `start_utc + start_utc_offset_secs`. There are two offsets because daylight saving can change during a search. The app reads the offset from the system through `chrono` (the `clock` feature only), at the instant of each timestamp. Entries written before the offsets were recorded carry 0, which means "not known" (`start_utc_offset_secs` is `#[serde(default)]`, so they still decode, and `schema_version` stays 1).
* **Store layout and versions.** One key per entry, `history/<start UTC seconds, zero-padded to 12 digits>`, like `known_peers/` ([0008](0008-iroh-and-local-storage.md)), so keys sort by time and the module lists, writes and deletes single entries. `schema_version` is the layout. `rules_version` names what the counts mean (epoch length, widening step, sample size, limit, ad lifetime, checkpoints and the matching-rules number); a test keeps it equal to the config. An entry that fails to decode or has an unknown schema version is skipped when read and never an error. A change of the counting rule bumps `rules_version`, and an estimator then ignores older entries.
* **Pruning, on append, in this order:** entries older than 180 days, then more than 100 per dungeon, then more than 500 in all, oldest first each time. Each delete stands alone, so a crash half way is harmless. The age limit is longer than the first proposal of 90 days so that a later estimator can reach its gate at a few searches a week.
* **Retention and privacy.** The history is a log of when the player plays, what they queue for, their level and role. So: a clear action (`DELETE /api/history`, 204, loopback only like pairing, so a phone cannot call it; the UI is a separate change), the history is also deleted with the character (`ClearCharacter`), and it never appears in logs, exports, pairing or the API. Tests and dev data use invented characters only. This narrows the non-goal "no character data" to "no names, peer ids or zone text".

```mermaid
flowchart LR
    E[Engine: search ends] --> F[SearchFacts]
    F --> M[App layer Model]
    C[Clock, character, dungeon] --> M
    M --> R{Outcome}
    R -- formed or expired --> W[history::record]
    R -- stopped over 60 s --> W
    R -- ignored or short stop --> X[Nothing written]
    W --> P[prune: age, per dungeon, total]
    P --> S[(Store: history/ keys)]
    K[ClearCharacter or DELETE /api/history] --> Z[history::clear]
    Z --> S
```

## Consequences

* Data accumulates from the first live run, before any estimator exists. The thresholds (180 days, 100, 500, 55 s, 115 s, 60 s) are starting values to check against the data.
* The history is the first behavioural log on disk. The clear action and the link to `ClearCharacter` are part of the decision, not extras.
* Adds one key prefix to the store and one loopback route to the API, and nothing else to the API or `AppState`.
* The engine gains one action (`SearchEnded`) that hosts other than the app ignore. The checkpoint rule and the counting rule are part of what `rules_version` describes, so changing either is a version bump.

## Amendment 2026-11-21: three attempt counters per entry

`Entry` gains `invites_sent`, `accepts_got` and `taken_seen` (u32, the counters of [0026](0026-search-health-from-own-observations.md) at the terminal event, from `SearchFacts`). They are additive with `#[serde(default)]`: entries written before decode with 0. `schema_version` stays 1. `rules_version` is not bumped: no existing field changed meaning. An estimator must not read a 0 in an old entry as "no invitations": it cannot tell an old entry from a quiet search, so it should only use these fields for entries whose `rules_version` and write time it trusts (stage 8b's concern). `invites_got`, `drops` and `declines` are not repeated: `drops` and `declines` already exist, and `invites_got` is not needed to tell the situations apart.

## Amendment 2026-11-21 (later): the estimator (stage 8b)

`history::estimate` is a pure function over the entries and one observation. It gives at most one banded `Band` for the running search and nothing else leaves the module. **Unverified:** no real entries exist yet, every constant below is a starting value, and the gate defaults to silence.

* **Observation.** The counts at the 55 s checkpoint of the current search (`SearchStats::checkpoint`) and the player's own role. An estimate is only asked for once the search has reached that checkpoint, so before 55 s there is none. A search that is then still unformed is only comparable to entries that were also unformed at 55 s, so entries that ended earlier (`checkpoint_1 == None`) are excluded on purpose.
* **Bin.** The bottleneck role: the one with the lowest seen-to-need ratio against the party mix 1 tank, 1 healer, 3 damage, counting this player in their own role, bins `tank | healer | damage | none` (ties go tank, healer, damage). `engine::diagnose::bottleneck` computes it with the same constants and counting convention as the situations, so they cannot disagree.
* **Pool.** Entries with the same bin and own role, across dungeons, with this build's `schema_version` and `rules_version` (others are skipped). The same dungeon alone is used instead only when it passes the same gate by itself.
* **Outcome mapping.** `Formed` is a success, `Expired` a failure. `Stopped` is incomplete data: in neither count. A drop followed by a re-form is judged by the entry's final outcome.
* **Statistic and gate.** The success share with a 95% Wilson score interval. Dependable means at least 3 successes and 3 failures (`NotEnoughHistory` otherwise) and an interval no wider than 0.40 (`Unreliable` otherwise). Constants: `MIN_SUCCESSES`, `MIN_FAILURES`, `MAX_WIDTH`.
* **Bands.** `UsuallyFound` if the interval's lower end is at least 0.60, `RarelyFound` if its upper end is at most 0.35, else `SometimesFound`.
* **Emission.** Wired as `diagnosis.outlook` with the rules in [0026](0026-search-health-from-own-observations.md): never when ended, warming up or not connected, and `usually_found` never next to a live problem.
* **Cache.** `Model` loads the readable entries once, refreshes them after each recorded entry and empties them on a history clear or `ClearCharacter`. The 1 s tick never reads the store, and a failed read means no estimate.
* **Deferred.** A typical wait among the successes (with `Stopped` searches as "at least this long") is not part of this cut, so there is no wait-time band yet.

