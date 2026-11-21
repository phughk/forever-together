# Concepts Brief: Search diagnosis

Author: Hugh Kaznowski. Date: 2026-11-21. Mode: BRAINSTORM.
Origin: `docs/notes/ui_notes.txt` line 11.
This brief is separate from `docs/design-process/concepts_brief.md`, which is an older, unrelated brainstorm.

## Reviewer status

| Reviewer | Round 1 | Round 2 (history and estimates section) |
| --- | --- | --- |
| `egm-design` | completed | completed |
| `egm-codebase-details` | completed | completed |
| `egm-zoom-out` | completed | completed |
| `egm-comprehension` | completed | completed |
| `egm-implementation-readiness` | completed | completed |
| `egm-structural` | completed | completed |
| `egm-inventive-catalyst` | completed | completed |

Round 2 reviewed only the section "Search history and estimates", the edited Non-Goals, "Decided by the author after the review" and the staging items about history, plus their conflicts with the rest. Findings 1-24 are from round 1. Findings 25-40 are from round 2. The author's wording is kept as written. Where a reviewer disagrees with it, the point is marked **Conflict flagged** and nothing is overwritten.

## Overview

Hypothesis, agreed with the author: while a search runs, and in a one-to-two sentence summary when it ends, the app tells the player in plain language whether it is working and whether a group is plausible. It uses only what this node observed: peers connected, players seen, a rough role mix such as "8 players, 0 tanks", and invitations and parties attempted. The player can then decide to wait, change dungeon, or give up and turn the app off.

Round 2 note: the author later allowed this node's own search history to feed estimates and conclusions (section "Search history and estimates"). "Uses only what this node observed" now means this search plus this node's own past searches. The phrase "whether a group is plausible" is the claim round 1 called unsupportable. See finding 25.

Failure is not one situation. The app should tell these apart:

1. Not connected to anyone.
2. Connected, but nobody else is searching.
3. Players are visible, but the role mix cannot make a party (tanks are scarce, per the S0 results).
4. Invitations were sent but nobody accepted.
5. A party formed and then fell through (declined, unreachable or timed out).

The unverified parts of the project apply here. The agreement mechanism is simulation-only and no live network has been tried. Any threshold in this feature is an assumption until it is checked live.

## Analysis

### What the code does today (verified by reviewers)

- The UI (`webapp/src/components/organisms/SearchProgress.tsx`) shows players searching, network peers, stage steps and one `lowCount` Notice (stage 4 and `searching <= 3`). The "Search ended" card is a fixed paragraph with no data.
- `engine::View` (`app/src/engine/state.rs` ~179) has only `mode`, `stage`, `peers_seen`, `next_stage_in`, `confirm_prompt`, `party`, `committed` and `last_drop`. There is no role mix, no counters, no attempt history.
- **Expiry discards the data.** `tick_searching` (`state.rs:564-568`) calls `stop()`, which does `self.search.take()` (`state.rs:456-465`). After that `peers_seen` is 0, `stage` is 0 and the samples are gone. `Model::app_state` (`app/src/app/mod.rs:375`) sets `searching` only while the mode is Searching. A summary built from the post-expiry view would say "0 players" exactly when it matters.
- **`peers_seen` is not "searching for this dungeon".** It is a de-duplicated union across all topic samples (`state.rs:1111-1119`), built with `eligible(ZERO, MAX)`, so it includes level- or stage-incompatible peers and ads up to 120 s old. It excludes self and `Taken` peers.
- **The sample is a subset.** It holds at most 100 ads per topic (`SAMPLE_SIZE`, `config.rs:18`), so one topic shows at most 99 other players. Eviction keeps the ads nearest to the owner's level, with a hash tie-break (`sample.rs:50-56`). Existing stories that show `searching: 312` cannot occur for one topic.
- Each ad carries `role` (`engine/ad.rs:28`). Nothing aggregates roles. `RoleView` already exists in `server/core.rs:113`. `ad::compatible` (`ad.rs:111`) holds the level and item-level rules.
- Nothing counts invitations sent, received or accepted, or parties committed. Natural increment sites exist: `run_epoch` (`state.rs:666-676`), `on_invite` (688-718), `on_accept` (754-797), `commit_to` (837-918) and `drop_party` (1051-1094). `Lead`, `inbox`, `decided` and `lease` are reset every epoch or on drop.
- Only tanks send invitations. A tank with no healer in its sample skips silently (`state.rs:652`). Non-tanks only receive.
- `last_drop` is cleared only in `commit_to` (`state.rs:908`). A drop from an earlier search can leak into a new one.
- Stage 1 (secondary roles) does nothing in v1 (`widen.rs:14`), yet the UI lists it as a widening step.
- `drop_party` resets the stage to 0 but keeps `search_start`, so the 10-minute clock and the widening clock disagree after a failed party.

### Can a diagnosis be honest on partial, self-observed data?

Yes, in a limited form. The reviewers converge on this:

- **Observations are honest; verdicts are not.** "8 players seen, 0 tanks" is a fact about this node. "A group is plausible" or "give up" is a claim about the network that the node cannot support. All five failure classes are not equally observable: not-connected, no-peers and dropped parties are observable; role shortage is observable with caveats; "invitations not accepted" is the weakest, because a tank cannot tell lost messages (no retransmit), a better-ranked leader winning (normal behaviour), and absent peers apart.
- **The sample size decides how strong "0 tanks" is.** Below the 100 cap, the sample is close to a census and "none seen" is nearly literal. At the cap it is a floor over a level-biased slice. The engine can expose a `sample_full` flag.
- **Claims are forgeable (ADR 0003).** One forged tank ad flips "no tank" to "tank seen". Copy must say "claimed" or "seen", and positive reassurance should be avoided.
- **Asymmetric risk.** A false alarm costs a few minutes. False reassurance can cost the full 10-minute wait. Only a few states should be conclusive (not connected; no tank in a near-complete sample; repeated drops). Everything else should read "nothing obvious, still searching".

### Rule-based versus numeric

All reviewers prefer a small closed set of named situations over a score. A percentage would be false precision; the S0 figures (74.6% mean, 69.7% worst seed) come from a model with no churn and cannot calibrate a per-player probability. Show the raw counts next to the sentence so the player can overrule the rule.

Round 2: the author's history section reintroduces a percentage, calibrated on this node's own outcomes instead of S0. That is a different source, but it is still a number about the network and still positive reassurance when high. Round 2 reviewers want the policy stated once (finding 25).

### State to add, and where it lives

Convergent position (design, structural, implementation-readiness, codebase-details, inventive-catalyst):

- The engine keeps facts only: monotonic counters and a role histogram, updated inside the existing handlers. It stays pure; no clock, no I/O.
- Counters live on `Engine` (not on `Search`) so they survive stage resets and `drop_party`. They are snapshotted before `stop()` at `state.rs:565` and kept until the next `Start`, or Clear.
- Classification is a separate pure function, for example `diagnose(snapshot, stage, own_role, network) -> Situation`, a closed enum with table-driven tests. It sits next to `widen.rs`, not inside `View`.
- Prose lives in the webapp, keyed by enum variant. Putting English in Rust would block wording changes and duplicate copy; putting the thresholds in TypeScript would drift from the engine's compatibility rules.
- Thresholds are defined once, in the engine, and the UI receives derived enums or booleans. `lowCount <= 3` and `WIDENING_STEP_S` are already duplicated between TS and Rust.

### Pacing rolling feedback

- Evaluate once per 5 s epoch, which is when the data changes.
- Warm-up: no negative verdict for roughly the first 15 s (one epoch plus the 2 s invite window), shown as "still looking".
- Hysteresis: enter a situation only after it holds for N epochs and leave on a stricter condition, so the text does not flicker. The cost is about 15-20 s of lag.
- Stage awareness: "change dungeon" advice only at the last stage, as the existing notice does. Early-stage zeros can change after widening.
- `publish` (`app/mod.rs:503-514`) de-duplicates by equality, so counters that change every epoch cause more republishing. Coarse bucketing, or emitting the situation rather than raw counts, avoids churn.
- Dwell can be pure (derived from epoch counts in the snapshot) or a UI concern tested with a fake timer; pick one.

### Honesty and wording (comprehension lens)

- Never use "peer", "sample", "epoch", "invitation" or "widening" in player-facing text. "Peers" and "this peer" already appear in the UI and read as engineer vocabulary.
- "0 tanks" reads as a fact about the server. Every count needs its scope ("among the players this app can see"), and zero means "none visible".
- Say whether the count includes the player. If the player is the only tank, "0 tanks" is wrong.
- Distinguish "not connected" from "nobody searching". "0 peers" is the offline or firewall case.
- Do not say "declined" unless a Decline message was received. Use "didn't confirm" or "no reply".
- The advice must match the stage. "Try another dungeon" repeats what stage 4 already did. The current limit notice ("Nobody is searching for you now") is ambiguous and the low-count notice says both "try other dungeons" and "keep waiting".
- State the party requirement once (1 tank, 1 healer, 3 damage).

### History and estimates: analysis (round 2)

Facts re-checked against the code:

- **Store.** `Store` (`app/src/store/mod.rs:34-41`) is a synchronous key-value trait with `get`, `put`, `delete` and an ordered `list(prefix)`. There is no append, no transaction, no batch. `get_json` and `put_json` live in `store/mod.rs`, not in `config.rs`; `config.rs` only wraps them for `Settings`. `known_peers/<hex>` is an existing many-keys precedent. A corrupt value returns `StoreError::Decode`; the existing callers log and fall back to defaults (`app/mod.rs:817-825`). `varaq` is 0.1.x and its file says the on-disk format may change (`varaq_store.rs:14`). The ring is 4 MiB and the page pool about 1 MiB. I could not find a documented value-size limit; do not claim one.
- **Clock.** `Clock` (`app/mod.rs:30-38`) returns wall-clock time since the Unix epoch as a `Duration`. `Model` already holds `started_at` in epoch milliseconds and the engine's `search_start` uses the same epoch. `Cargo.toml` has no date, time or time-zone crate (`chrono`, `time` and `jiff` exist only as transitive crates in the lockfile). The standard library gives no local time zone. So "no new dependency" works only if datetimes are UTC. The brief's risk line about daylight saving understates this: there is no local zone at all, and for an EU player UTC hour and weekday can be 1-2 h and sometimes a day off.
- **Search lifecycle.** `Model` sees every Start, `UserStop`, Expired and Final. `drop_party` sets the mode back to Searching and keeps `search_start`, so one search can form, drop, re-form and then expire. Other paths also end a search without being a player decision: `stop_search` called from `SetCharacter` and `ClearCharacter` (`app/mod.rs:584,596`), an implicit stop inside `start()` (`state.rs:421`), and process exit (no shutdown hook, so an in-progress entry is lost). A declined or unclicked Confirm is not a network failure (CLAUDE.md).
- **Checkpoints.** The engine has no live role counts and no checkpoint trigger. The 60 s and 120 s marks coincide with widening steps (`WIDEN_STEP` 60 s; stage 2 widens the item-level tolerance, `widen.rs:30`) and with the 120 s ad lifetime. So a count at 119 s and one at 121 s follow different compatibility rules.
- **ADRs.** No ADR says "no persistence". ADR 0008 already accepts local storage (identity, known peers, settings, token, character). History is a new kind of data in that store (behavioural, per search), not the first persistence. It touches 0008 (extends), 0003 (forged claims get recorded), 0011 (the widening step changes what a checkpoint means), 0013 (needs a module home), 0014 (tests), 0025 (outcomes and counts depend on it).

What the data can and cannot support:

- A player makes a few searches a week. At 2-5 a week, 90 days holds about 25-65 entries in total, spread over several dungeons. A gate of 20-30 entries per dungeon is reached only for a favourite dungeon, and by then a levelling player may have outgrown it. The total cap (500) and per-dungeon cap would never bind inside 90 days.
- Eight to twelve predictors (four counts at two checkpoints plus peak) on 20-30 rows is over-parameterised. `players` is the sum of the other three, so including all four makes the matrix singular. Peak grows with search length and formed searches stop early, so it leaks the outcome. Searches that ended before a checkpoint have no value there, and dropping them removes exactly the quick successes.
- A linear fit on a yes/no outcome can leave 0-1; clamping hides that. A prediction outside 0-1 should mean "no estimate".
- Expired is a true failure. A player stop is censored: it says nothing about minutes after the stop. Counting a stop after 61 s as a failure but a stop at 59 s as ignored is a hard edge. It also creates a loop: a pessimistic sentence at minute 1-2 makes the player stop, the stop is logged as a failure, and the next sentence is more pessimistic.
- "Typical time among successes" is the figure round 1's own rule warns about (line "estimates must not be built from successes only"). Showing it next to the chance helps, but it is still a success-only number.
- Entries come from one player, one role and one set of play hours. They describe that player's luck, not the network. A forged tank ad (ADR 0003) or a stretch with no connection gets written into the history as a result.

How history could be used safely (ideas from round 2, not decisions):

- **Asymmetry rule.** History may switch an alarm off or soften it. It never produces a reassuring sentence and never overrides a live fact ("not connected", "no tank in a near-complete sample").
- **One predictor.** Replace four counts with the party bottleneck `min(tanks, healers, damage / 3)` (counting self), computed with `engine::party` so the UI, engine and history agree. Keep storing all four counts. Keep own role as a category.
- **Honest intervals.** Per bin of the bottleneck, use a success rate with an interval (Wilson or Beta-Binomial) and define "dependable" as the interval width, not a fixed entry count. Use the lower bound for suppressing alarms. Output coarse bands, not a percent.
- **Censoring.** Treat player stops as censored at their duration and use a survival-style or per-minute hazard estimate for time. Tag entries where a negative sentence had been shown before the stop and leave them out of the failure count.
- **Calibration.** Store the situation `diagnose` showed at each checkpoint. Counting how often "no tank" was followed by a group on this node gives a per-situation false-alarm rate, which is a way to check the unverified first-slice thresholds against real use.

## Goals vs. Non-Goals

### Goals

- Show the player, during the search, whether this app is connected and what it can see, in plain language.
- Show a one-to-two sentence summary with numbers when the search ends at the limit.
- Tell the five failure situations apart where the node's own data allows it.
- Keep every statement hedged to what this node observed.
- Keep the engine pure and the logic testable at fixed durations.

### Non-Goals

- No claim about the global network and no "a group will form". Estimates are allowed only from this node's own history, and only once that history is dependable (see "Search history and estimates").
- No new gossip messages, no protocol change.
- History is never shown to the player. It is an implementation detail behind estimates and conclusions.
- No peer ids, names or character data in the retained snapshot or in history entries.
- No explanation of why an invitation went unanswered beyond what was actually received.
- No change to the matching or agreement rules (ADR 0025 stays as is).

**Conflict flagged (round 2, findings 25 and 34).** The first and third bullets pull against each other in practice. A "rough chance of finding a group within the limit" is a forecast about whether a group will form, and an estimate that chooses the wording is history showing through. "Never shown" needs a definition (per-search records, dates and counts are never shown; may one clause such as "in your earlier searches here" appear?). The fourth bullet says no character data, but entries hold level, own role, dungeon and start and end times, which is a log of when this player plays. Define "character data" for this rule, or narrow the bullet to "no names, no ids, no zone". These are the author's words and are left unchanged until the author decides.

## Alternatives Weighed

| Option | What it covers | Cost | Verdict |
| --- | --- | --- | --- |
| A. Cheap slice: add role counts and a frozen peak snapshot to the existing card; a static end summary from the same counts | "players seen", "0 tanks", "no peers" | A few engine fields, one `SearchView` field, one Field row, stories | Zoom-out lens: covers most of the value. Build first. |
| B. Cheap slice plus pure `diagnose` with 5-6 named situations, shown with the raw counts | Rolling sentence, end summary, replaces `lowCount` | A (above) plus counters for drops and invitations, copy deck, stories per situation | Recommended target. |
| C. Full version: hysteresis state machine, feasibility counts, "what would change this" counterfactuals, differential display, ghost ratio | Most of the wish list | Second product; copy, tests and tuning for each new situation; coupled to ADR 0025 internals | Defer. |
| D. Numeric plausibility score | A single number | False precision, uncalibrated | Reject for live data. Round 2: the history estimate (F) is this option again, from a different source. State the reversal or keep F out of the player's text. |
| E. Do nothing beyond fixing the end card copy | End card only | Minimal | Misses the rolling feedback the author asked for. |
| F. Author's history estimate: per-search entries, linear regression on role counts at about 1 and 2 minutes, chance 0-100% plus typical time, gate about 20-30 entries per dungeon | A forecast from this node's own results | Entry schema, store layout, recorder in the app layer, solver and gate, ADR, tests, copy for estimate-aware wording | Author decision, kept. Round 2 reviewers advise against this form for the reasons in "History and estimates: analysis". Not recommended until the open questions in finding 28 and 29 are answered. |
| G. Plain per-dungeon (or pooled) counts of recent searches: last N outcomes and durations in one small record, success count with an interval, median of successful durations, no regression, no checkpoints | "Of your last 8 searches here, 5 found a group" | One small record, a few functions, no model | Zoom-out and design lenses: same player benefit at a fraction of the cost. Conflicts with the author's "history is never shown" if displayed. Use it internally as the first estimator, or show it. Author to choose. |
| H. Collect entries only (no estimator) in the first live build, read them by hand later, then decide on an estimator | Settles checkpoint and counting rules against real data before code depends on them | The entry type, recorder and prune only | Design lens: cheapest way to learn what the data looks like. Compatible with the author's staging (item 5 split in two). |
| I. History used only to tune the live wording: bottleneck predictor, per-bin interval, history may soften or suppress an alarm and never reassure | Keeps "no numbers in player text" and "history never shown" both true | Pure function taking a history summary as an input to `diagnose` | Inventive-catalyst lens. Fits the Non-Goals better than F. Author to choose. |

Dissent to record: the zoom-out lens argues that the invitation and party-attempt counters couple the feature to a mechanism that is still in flux (0010 was just replaced by 0025), and wants them deferred. Design and inventive-catalyst lenses see the invitation signal as the only way to separate "nobody answers" from "no tank". Resolution proposed below: stage the counters behind the first slice.

Round 2 dissent: zoom-out and design say the history estimate should not be built until the observation slice has run live and the author wants numbers, and that its payoff (months of data, one dungeon, a change of wording only) is small. Codebase-details, implementation-readiness and inventive-catalyst accept building it but want the schema, outcomes, predictors and gate fixed first. The author's decisions stand; the disagreement is about order and form, recorded here.

Ideas from the inventive-catalyst lens worth keeping for later stages:

- **Feasibility from the S0 result.** `parties_possible = min(tanks, healers, damage/3)` over compatible visible players, and name the bottleneck role as the sentence subject. Reuse `engine::party` so the UI and the engine cannot disagree.
- **Doubt level** (complete view, slice, thin) from `sample_full` and connection state; "give up" guidance is only allowed on a complete view.
- **`taken_seen` counter.** A burst of commits empties the sample (taken ads are removed), which looks like a dead network but is success. Without it the app raises a false alarm at the best moment.
- **"What would change this" hints** tied to levers that exist, computed by re-running the check under the next stage's rules. Skip stage 1, which changes nothing.
- **Differential display** ("last minute") for trend.
- **Ghost ratio:** invitations sent versus accepts received as a reliability signal for the sample. Tanks only; others use invitations received and lost-to-rank.

## Search history and estimates (author decisions, added after the Goldfish review)

The first review assumed no persistence. The author reopened that: this node keeps its own history of searches, and uses it only internally to decide what to say and to give estimates. It is not displayed.

**Entries, not buckets.** One entry per search, scanned and pruned as needed. A rolled-up table by dungeon, day and hour was rejected: it would say "low numbers on Thursday 12:00" where the real cause is often a role shortage that can happen at any time.

Each entry holds:

- Start datetime and end datetime, stored separately. Day of week, time of day and duration are derived from these two.
- Dungeon, the player's level and own role.
- Players seen, tanks seen, healers seen and damage seen. All four are tracked, because a shortage can be damage as well as tanks (unlucky matching). Recorded at checkpoints (proposed: about 1 and 2 minutes in) and at the peak, so a running search can be compared with past ones at the same point.
- Outcome: formed a party, expired at the limit, stopped by the player, or party dropped after forming.

**Outcome rules.**

- Expired counts as a failure sample.
- Stopped by the player counts as a failure sample only if the search lasted more than one minute. A shorter stop is ignored.
- Time to find and time to expire are both kept. Expired and stopped searches are incomplete timing data (a group might have come later), so the estimate must not be built from successful searches alone, or it will say "usually 4 minutes" while ignoring the searches that never found anyone.

**Estimates.**

- Linear regression over the entries. Predictors are the role counts seen at the checkpoints. Time of day and day of week are recorded but may end up unused; the author accepts that.
- Output is a rough chance of finding a group within the limit and a typical time among successes, always stated together. The chance is clamped to 0-100%.
- Estimates appear only when there are enough entries for the dungeon and the fit looks dependable. Below that threshold the app uses the live-only wording. Proposed starting threshold: about 20-30 entries per dungeon. This is unverified and needs checking against real data.

**Bounds.** Prune on append: a cap on total entries (proposed about 500), a cap per dungeon so one popular dungeon does not push out the rest, and an age limit (proposed about 90 days).

**Implementation shape.** A pure module (record, prune, estimate) over the key-value store, with no new dependency. Regression on a handful of predictors is a few dozen lines. It stays outside the engine's state machine, which stays pure and free of I/O.

**Risks to record.**

- Thin data. A player makes a few searches per week, so the threshold above will take weeks to reach for any one dungeon.
- Bias. Entries exist only when this player searches, so they reflect this player's times and role, not the server.
- Linear regression on a yes/no outcome is a rough model. It is acceptable here because the author wants rough descriptions, and the output is clamped.
- Clock changes (time zone, daylight saving) shift derived day and hour slightly. Accepted.

### Round 2 reviewer notes on this section (not author decisions)

The text above is the author's and is unchanged. These are the gaps the reviewers found, to be settled before the ADR. Each has an owner finding in the table below.

**Definitions the section needs**

- **Success and the outcomes.** Say what "formed" means (all five confirmed, `Mode::Final`, not the commit in `commit_to`). Party dropped after forming is not terminal: `drop_party` returns to Searching, so one search can form, drop, re-form and expire. One entry needs `first_formed_at` and a drop count separate from the final outcome, or a rule for which event closes it. A declined party is the player's choice, not a network failure. Add an "ignored" outcome for searches ended by `SetCharacter`, `ClearCharacter`, a restart, sleep, no connection or quitting. A quit loses the in-progress entry (no shutdown hook); say that is accepted.
- **Checkpoint.** Measured from `search_start`, not the widening clock, on the first tick at or after the offset. Record the stage in force and `sample_full`. A search that ends first stores a missing value, not 0. Decide whether the estimator skips such entries. The 60 s and 120 s offsets sit on widening steps and on the ad lifetime; consider 55 s and 115 s, or counts under the stage-0 rule only.
- **Peak.** Say whether it is each role's own maximum or the roles at the busiest moment, and which sample (compatible or all). Players is the sum of the three roles, so do not use all four as predictors.
- **Dependable.** Needs a test, for example minimum successes and failures, a predictor count small against the entry count, an interval narrower than a stated width, and a finite, non-singular solve. The result type should say why there is no estimate (not enough history, unreliable, value).
- **Multi-dungeon search.** A search may cover several dungeons, and extra topics join at stage 4. Choose one entry per search with the primary dungeon and a topic count, or one per dungeon. The topic also includes region, realm and faction, so two realms are different networks; entries need those or the fit mixes them. Level at start or at end.

**Schema and store**

- Add a schema version and a rules version (epoch length, widening step, sample size, limit, matching rules). Ignore or age out entries from other versions. A decode failure, an unknown version or a `varaq` format change must give "no history", never an error.
- Prefer one key per entry (`history/<start time, zero-padded>`) with `list` and `delete`, as `known_peers/` does. A single JSON array of about 500 entries (about 100-150 KB at roughly 250-300 bytes each) is rewritten on every append, one bad byte loses all of it, and the write blocks the single core thread that also runs the 1 s tick and heartbeats (4 s timeout). Pruning with many keys is several deletes with no transaction; a crash mid-prune is harmless. If one key is kept, keep a version field and reset on decode failure. Write only at the terminal event, never at checkpoints. A failed write must never fail a search; log it.
- The pure module takes the store, the current time and the entries as arguments, so `MemStore` and a fake clock cover the tests.
- Store UTC seconds and a duration taken from the engine's own `now` values, with `saturating_sub`, because the wall clock can step backwards. If local time of day matters, record a UTC offset at that moment (the browser knows it). Otherwise state that the first estimator does not read the datetimes except for age pruning.

**Recording site**

- The app layer (`Model`) owns the clock, the store and the lifecycle events, so it should write the entry. The engine should expose live role counts and emit a pure facts record on each terminal transition (expired, player stop, first commit, drop). The engine stays free of I/O and clocks.

**Pruning order and numbers.** State the order (age, then per dungeon, then total), the per-dungeon cap, and what goes first (oldest). Reconcile the 90-day limit with the gate: at a few searches a week the gate may never be reached. Options: a longer age limit, a lower gate, pooling across dungeons on one predictor.

**Privacy.** The history is the first behavioural log on disk (when the player plays, what they queue for, their level and role). Nothing in the brief lets the player see or delete it. `ClearCharacter` deletes only the character key, so history for a cleared character would survive. State retention, add a clear-history action (at least tied to `ClearCharacter`), and say that entries never appear in logs, exports, pairing or the API. No real character data in the author's own dev files or tests; invented data only.

**Tests (functional, invented data, per CLAUDE.md).** Record and prune with `MemStore` and a fake clock (order, caps, age). Estimator on a seeded generator with a known effect (sign of the effect, the 0-100% bounds). Edge cases: all failures, all successes, collinear data, too few entries, each giving the right result variant. End to end from the engine: a solo tank to expiry, a stop at 30 s (ignored) and at 90 s, a drop after a commit, a decline. A corrupt or unknown-version entry is skipped. A failing store does not stop a search. Run only the touched module.

## Key Decisions & Staging for Next Steps

### Decisions the author needs to make

1. Verdict scope: confirm that the app reports observations plus a few hedged situations, not a plausibility judgement (all lenses recommend this). Round 2: re-ask in the light of the history estimate (decision 8).
2. Whether invitation and party counters are in the first slice or staged after it (zoom-out dissent above).
3. Where classification lives: a pure module in the engine crate is the proposal; confirm.
4. Multi-dungeon scope: report per dungeon, or an aggregate with its scope stated ("across 3 dungeons"). Extra topics join only at stage 4 (`state.rs:573-583`), and `topics` in the view falls back after expiry because `stage` reads 0.
5. Counting rule for "players": compatible under the current stage's rules, or all stored ads; and whether banned peers (60 s local ban) are included. Either way the UI label needs to change from "Players searching". Round 2: this rule now also fixes what history entries mean, so changing it later invalidates stored entries.
6. Lifecycle of the end snapshot: kept until Clear or the next search. What `UserStop` does with it. Manual stop currently maps to Idle and shows nothing.
7. Whether to add ADR 0026 (proposed title: the app reports search health from this peer's own observations; amends 0011, cites 0003, 0005, 0025). Structural and implementation-readiness lenses both say yes.

Added in round 2 (open):

8. What reaches the player from history: a number, a band, or only a choice between existing situations (alternatives F, G, I). Define "never shown". State whether Alternative D's rejection is reversed for own-history estimates.
9. Entry definition: success and failure meaning, the outcome for a drop followed by a re-form, which events are ignored, handling of a player stop (failure at more than 1 minute, as decided, or censored; see finding 27), multi-dungeon rule, realm and faction in the entry, level at start or end.
10. Estimator form and "dependable" test (finding 28), and how the gate is made reachable given the 90-day limit (finding 29).
11. Store layout (one key or one per entry), schema and rules versions, and what a version change does to old entries.
12. Who records (proposal: the app layer, fed by an engine facts record), and the clock convention (proposal: UTC seconds, engine durations for duration).
13. Retention and deletion: a clear-history action, tie to `ClearCharacter`, no history in logs or the API.
14. Whether history gets its own ADR (round 2 proposal: ADR 0027, extending 0008, separate from 0026), and which module holds it (ADR 0013 fixes the module list).

### Decided by the author after the review

- Keep history as per-search entries with start and end datetimes, role counts at checkpoints and the outcome (section above).
- All four counts (players, tanks, healers, damage) are tracked.
- History is internal and never shown.
- Expired searches, and searches the player stopped after more than one minute, count as failure samples.
- Estimates only when the history is large enough and dependable.
- Time of day and day of week are recorded but need not be used by the estimate.

Round 2 note: the bounds (about 500, a per-dungeon cap, about 90 days), the checkpoints (about 1 and 2 minutes), the gate (about 20-30) and linear regression appear in the section above as proposals or implementation shape, not in this decided list. Treat them as proposed until the author says otherwise. **Conflicts flagged, none overwritten:** player stop counted as failure after one minute (findings 27), linear regression as the estimator (finding 28), "never shown" next to a stated estimate (finding 25), and the gate against the 90-day limit (finding 29).

### Proposed staging

The list is renumbered 1-9 in round 2 because the original numbering repeated 3-6. Wording is unchanged except where a round 2 note is added.

1. **ADR 0026 and a short design-doc note.** Record the observation-only rule, the hedging, in-memory retention and the engine/UI split. Update the implementation plan's API table and file list. Round 2 note: "in-memory retention" means the end snapshot only; history is persisted and is a separate, later ADR. No existing ADR says "no persistence" (0008 already stores identity, settings and the character).
2. **Engine facts.** `SearchStats` on `Engine`: role histogram of the compatible sample, `sample_full`, own role, peak players, stage reached. Snapshot before `stop()`; clear on `start()`; do not reuse `last_drop` unguarded. Test with a solo tank through expiry (the existing test at `engine/mod.rs:715-741` is the base). Round 2 note: if history is built later, checkpoint capture (live role counts per tick, or at elapsed offsets from `search_start`) is not in this item and must be added here or in the history stage.
3. **Pure `diagnose`.** A closed enum: not connected, alone, no tank, no healer, short on damage, healthy-looking, too early. Constants named with their source. Table tests at fixed durations. Include an invariant test that no message claims more than the observations support.
4. **API.** One optional `diagnosis` object on `SearchView`, additive and tolerant of unknown codes, so an older bundle (or a `WOWFT_WEB_DIR` override) falls back to the old copy. Change `server/core.rs`, `types.ts`, `fixtures.ts`, `mockApi.ts`, `server/tests.rs`, `app/mod.rs` (`placeholder_state`) and `app/tests.rs` together.
5. **Webapp.** A `DiagnosisSummary` molecule built from `Notice`, `Field` and `RoleIcon` (no new atom unless `Notice` lacks an info tone). Integrate in `SearchProgress` for both the running and ended cards, and remove the inline `lowCount` Notice so there is one source of truth. Story variations: warming up, not connected, alone, no tank, no healer, full sample versus partial, multi-dungeon, long dungeon list, ended per situation, one versus two sentences. Add the character-length limits to the molecule's story docs. Use invented data only ("Testa Fakename", "Testville").
6. (later) **Counters for attempts.** `invites_sent`, `accepts_got`, `invites_got`, drops by cause, `taken_seen`; role-aware copy; the "nobody answered" situation gated on a minimum attempt count.
7. (later) Hysteresis, doubt level, feasibility sentence, counterfactual hints.
8. (later) **History and estimates.** The entry type, the store key, prune-on-append, a pure estimator with a minimum-entries gate, and tests with invented data. Build after the observation slice, so the checkpoint counts it records already exist. Needs its own ADR (supersedes the "no persistence" assumption above). Round 2 notes: (a) there is no "no persistence" assumption in any ADR; the ADR extends 0008 and records retention, privacy and the estimator, and is better numbered 0027, separate from 0026; (b) checkpoint counts do not yet exist (see item 2); (c) round 2 reviewers suggest splitting this item: 8a collect entries only, 8b an estimator after a live capture shows the fields are stable (Alternative H).
9. **Live check.** Mark all thresholds unverified until run against a live network, in line with the S0 status. Round 2 note: this is the gate for every threshold above, including the history ones, so it should come before item 8b, not after it.

### Acceptance criteria

- At expiry the summary shows the real numbers; it never shows 0 caused by `stop()`.
- The same observations always give the same situation; no wall clock in the classifier. Round 2 note: if history informs the situation, history is an explicit input to `diagnose` (a summary value, not a store read) so this stays true.
- No message asserts more than was observed ("none seen", never "none exist"), and none names a person. Round 2 note: a history-based sentence needs a provenance clause and must not print a bare 0% or 100%; decide the wording (finding 25).
- The advice agrees with the stage reached.
- An old web bundle against the new server renders the previous copy.
- The first slice makes no store or schema change. The history stage adds one new store key and nothing else. Round 2 note: "one new store key" holds only for the single-array layout; per-entry keys are one new prefix. Also true only if the history stage adds no API or `SearchView` field, which an estimate shown to the player would need.
- Every new component has stories at its level; `mise run lint`, `cargo test --lib engine::`, `pnpm --dir webapp typecheck` and `build-storybook` pass.
- Added in round 2, for the history stage: a corrupt or unknown-version entry is skipped; a failing store never stops a search; a history estimate never overrides a live "not connected" or "no tank" and never produces reassuring wording; a clear-history path exists.

## Deduplicated findings

Severity: Blocker (B), Major (M), Nit (N). Lens names in brackets. Findings 25-40 are from round 2.

| # | Sev | Finding | Lens |
| --- | --- | --- | --- |
| 1 | B | Expiry discards samples, stage and counters; the end summary reads zeros unless a snapshot is taken before `stop()` and kept in `View` while Expired | design, codebase, structural, implementation, zoom-out |
| 2 | B | A "plausible / give up" verdict from a hashed, level-biased, forgeable sample overclaims; emit observations and a few hedged situations | design, zoom-out, implementation |
| 3 | B | "0 tanks" is read as a fact about the server; every count needs its scope, zero means "none visible", and the player's own role must be handled | comprehension, design |
| 4 | B | The role mix exists per ad but is not aggregated or exposed; semantics (compatible versus all, per peer across topics, stale ads, self-reported) are undefined | codebase, implementation |
| 5 | B | Where diagnosis logic lives is undecided; thresholds in TS cannot be tested by the Rust suite or the simulation | implementation, structural, design |
| 6 | M | `peers_seen` counts all stored ads, including incompatible and up to 120 s old; capped at 99 per topic; the existing "Players searching" label and the `312` stories are misleading | codebase, inventive-catalyst |
| 7 | M | Invitation data is role-asymmetric (only tanks send); "no reply" cannot separate message loss, a better-ranked leader and absent peers; no retransmit | design, implementation, codebase, comprehension |
| 8 | M | Not-connected needs its own state and must suppress role sentences (`network.peers == 0`, `bootstrap == none`) | design, comprehension, codebase |
| 9 | M | A burst of commits empties the sample via `Taken`, which looks like a dead network; a `taken_seen` counter avoids a false alarm | inventive-catalyst |
| 10 | M | Reassurance is the worse error; positive "plausible" wording should be avoided and verdicts gated on warm-up and attempts | design, comprehension |
| 11 | M | Stage-awareness: stage 1 does nothing in v1; "change dungeon" advice conflicts with stage 4 already including other dungeons; the two clocks disagree after a drop | inventive-catalyst, comprehension, design |
| 12 | M | Rolling pacing: warm-up, hysteresis, evaluation per epoch, republish churn from counters | design, implementation, inventive-catalyst, codebase |
| 13 | M | Multi-dungeon scope and topic semantics (extra topics join at stage 4; `topics` field falls back after expiry) | codebase, implementation |
| 14 | M | Wire contract: `SearchView` is mirrored by hand in four-plus places; add one optional field, tolerate unknown codes, update every constructor | structural, implementation, codebase |
| 15 | M | An ADR is needed (0026), amending 0011 and citing 0003, 0005, 0025 | structural, implementation |
| 16 | M | Atomic design: new molecule built from `Notice`/`Field`/`RoleIcon`; remove the inline `lowCount` Notice; stories per situation and a page story for the ended state | structural, implementation |
| 17 | M | Jargon in player copy (peer, sample, epoch, invitation, widening); "declined" asserted without a Decline; end summary should name one action and say why | comprehension |
| 18 | M | Scope creep: attempt counters and narration couple the feature to ADR 0025 internals and form a second product | zoom-out |
| 19 | M | Only `limit_reached` gets a summary today; manual stop maps to Idle and shows nothing; define the lifecycle of the snapshot | design, implementation |
| 20 | N | `last_drop` is not cleared by `start()` or `stop()`; stale drops can leak into a new search | codebase |
| 21 | N | `DropCause::Timeout` is a fallback that includes lost messages; copy should not blame players | design |
| 22 | N | Local 60 s bans may leave banned peers in the count | design |
| 23 | N | Stage name "Strict" reads as a penalty; use one term for players seen/found/searching | comprehension |
| 24 | N | One `tracing` line at expiry with the same struct would help triage | codebase |
| 25 | B | **The history estimate contradicts the brief's own rules.** A "chance of finding a group" is a number about the network (Alternative D, Non-Goal 1, decision 1, reassurance rule, "no message asserts more than was observed"), and an estimate that chooses the wording is history shown. "Never shown" is undefined. Pick one policy: banded wording with a provenance clause, or history only tunes existing situations (Alternative I), or reverse D explicitly. Never print a bare 0% or 100% (reads as "no group exists"). **Conflict flagged** with the Non-Goal and the output spec | design, zoom-out, structural, comprehension, inventive-catalyst, implementation |
| 26 | B | **Outcome model does not match the engine.** `drop_party` returns to Searching, so one search can form, drop, re-form and expire; "formed" must mean Final, not commit; a declined party is not a network failure; `SetCharacter`, `ClearCharacter`, restart, sleep, no connection and quitting are not player stops and a quit loses the entry; party dropped has no success or failure rule; "time to find" is undefined for a drop. Need `first_formed_at`, a drop count, a closed outcome list with an "ignored" outcome | codebase, implementation, design, comprehension, inventive-catalyst |
| 27 | B | **A player stop counted as failure is censored data and creates a feedback loop.** A stop at 61 s says nothing about minute 5; a pessimistic sentence at minute 1-2 causes stops that are logged as failures that deepen the sentence; the app's own advice and in-game grouping cause stops. The 59 s versus 61 s edge is arbitrary. Suggest: stops are censored at their duration, record whether a negative sentence was shown, and leave those out of the failure count. **Conflict flagged** with the author's outcome rule | design, inventive-catalyst, comprehension, zoom-out |
| 28 | B | **Linear regression is unsound and under-specified.** Eight to twelve predictors for 20-30 rows; `players` is the sum of the other three (singular); peak leaks outcome and length; searches ending before a checkpoint have no value there (early successes dropped); a linear model leaves 0-1 and clamping hides it; own role changes the process (only tanks invite); no solver, ridge or singular-case rule; "typical time among successes" repeats the success-only bias the brief forbids; "dependable" has no test. Suggest: one bottleneck predictor plus own role, per-bin interval or survival-style time, gate on interval width and minimum successes and failures. **Conflict flagged** with the author's estimator choice | all seven |
| 29 | B | **The gate is unreachable.** At a few searches a week, 90 days holds about 25-65 entries in total; a per-dungeon gate of 20-30 is met only for a favourite dungeon, by which time a levelling player may have outgrown it; the 500 and per-dungeon caps never bind. Options: longer age limit, lower gate, pool across dungeons on one predictor, or treat the feature as an experiment and collect only. **Conflict flagged** with the proposed gate and age limit | zoom-out, design, implementation, catalyst, comprehension |
| 30 | B | **Checkpoint data does not exist and is not comparable.** The engine has no live role counts or checkpoint trigger; 60 s and 120 s sit on widening steps (stage 2 changes the item-level tolerance) and the 120 s ad lifetime; after a drop `search_start` and the stage clock disagree; self is excluded from counts; a missing checkpoint must be null, not 0. Record stage, `sample_full` and measure from `search_start` | codebase, design, implementation, comprehension, catalyst |
| 31 | M | **Clock and recorder.** `Model` owns the wall clock, `started_at` and the lifecycle, so it should write the entry; the engine emits facts. No date or time-zone crate, so datetimes are UTC only and local time of day cannot be derived without a new dependency or a recorded offset. The brief's daylight-saving risk is wrong in kind. Wall clock can step back (guard `end >= start`, use engine durations). If datetimes are unused, store UTC seconds and a duration only | codebase, implementation, design, zoom-out |
| 32 | M | **Store layout.** `Store` is get/put/delete/list with no append or transaction. One array rewrites about 100-150 KB per search on the single core thread, loses everything on one decode error, and `varaq` is 0.1.x. One key per entry (`history/<start>`) fits the `known_peers/` pattern and the prune. The criterion "one new store key" changes to one prefix. Writes at terminal events only, failure non-fatal, `MemStore` and a fake clock for tests | codebase, implementation, design, structural, zoom-out |
| 33 | M | **Schema gaps.** No schema or rules version (0010 was just replaced by 0025; epoch, widening step, sample size and limit change what a count means), no region, realm or faction (topic includes them), multi-dungeon search has one dungeon field, level at start or end undefined, no stage or `sample_full`. Old entries silently describe a different game. Ignore other versions; tolerate decode failure | structural, design, implementation, catalyst, comprehension |
| 34 | M | **Privacy and retention.** A persisted log of when the player plays, what they queue for, level and role; no way to see or delete it; `ClearCharacter` deletes only the character key; the Non-Goal "no character data" conflicts with the entry fields. Add a clear-history action, a retention statement, no history in logs, exports, pairing or the API. **Conflict flagged** with Non-Goal 4 wording | structural, codebase, design, zoom-out, comprehension |
| 35 | M | **ADR and staging.** No ADR says "no persistence"; 0008 already covers local storage, so history extends 0008. A separate ADR (suggest 0027) for retention, privacy, schema and the estimator; 0026 stays observation-only. Staging item 1 ("in-memory retention") and item 8 disagree; the old numbering repeated 3-6 (fixed here); the live check should precede the estimator; checkpoint capture is missing from stage 2; the implementation plan's file list and API table need rows; ADR 0013 fixes the module list so history needs a named home | structural, comprehension, design, codebase, catalyst |
| 36 | M | **Interaction with live wording.** History may soften or suppress an alarm, never reassure, and never overrides a live "not connected" or "no tank". History needs to be an explicit input to `diagnose` to keep the determinism criterion. Stored situations per checkpoint could calibrate the first slice's unverified thresholds (excluding entries where a shown sentence caused the stop) | catalyst, design, comprehension |
| 37 | M | **Bad samples enter the history.** Forged tank ads (ADR 0003) and searches with no connection or after sleep are logged as failures or role counts. Record a connected flag or minimum peers, weight by `sample_full`, exclude zero-peer entries | design, catalyst |
| 38 | M | **Cheaper variants exist.** Alternative G (last N outcomes and median duration, counts with an interval), H (collect only, decide later) and I (tune live wording only). Zoom-out and design: build neither the estimator nor checkpoints before a live run | zoom-out, design, catalyst |
| 39 | M | **Tests are unspecified.** No test list for record, prune, estimator, version skip, failing store, engine to history. Use `MemStore`, a fake clock and a seeded generator with a known effect and invented data. ADR 0014 allows functional tests; nothing validates estimator quality | implementation, structural |
| 40 | N | **Brief hygiene.** Decided versus proposed is blurred (regression, bounds, checkpoints and gate are not in the decided list); the history section duplicates "Decided by the author"; the new section sits between Alternatives and Decisions instead of in the template; round 1 findings and objections did not mention history; observability: one `tracing` line per recorded entry and per estimate, and a debug-only dump (headless CLI) since the history is never shown | structural, comprehension, codebase |

## Three strongest objections

Round 1:

1. **The data the end summary needs is gone at expiry, and the live counts do not mean what the label says.** `stop()` clears the search before the Expired view is built, so a naive summary shows 0 players. `peers_seen` also counts incompatible and stale ads from a capped, level-biased sample.
2. **An honest verdict cannot be built from this node's view.** The sample is partial and forgeable, "0 tanks" can be a sampling artifact or the player's own role, and invitation silence has several causes. The feature must stay at hedged observations and a few asymmetric situations, and thresholds are unvalidated without a live network.
3. **The full concept is oversized for what is proven.** Attempt counters and a narrating state machine couple the feature to ADR 0025, which was recently replaced, and add a copy deck to maintain. Role counts, a frozen peak snapshot and a short deterministic summary cover most of the stated need.

Round 2 (new, about the history section):

1. **The estimate reverses the brief's own rules, and "never shown" cannot hold.** The brief rejected a numeric score (Alternative D, "percentage = false precision", reassurance is the worse error) and forbids a claim that a group will form, then specifies a displayed "chance of finding a group". An estimate that picks the wording is shown history. The policy must be stated once: banded wording with provenance, history only tuning live situations, or an explicit reversal.
2. **The labels and the model do not fit the data.** A stop after a minute is censored, not a failure, and the app's own advice produces those stops (a feedback loop that deepens pessimism). Party dropped after forming is not a terminal outcome in the engine. Eight to twelve collinear predictors on 20-30 rows, a peak that leaks, missing checkpoints for early successes, a success-only "typical time" and an undefined "dependable" test mean the fit cannot be validated.
3. **The estimate may never exist, and its inputs are not built.** At a few searches a week and a 90-day age limit, the 20-30 per dungeon gate is rarely reached before the player outlevels the dungeon. Checkpoint counts do not exist in the engine, 60 s and 120 s sit on widening steps, entries carry no version, realm or stage, and the log is a persisted record of play habits with no delete path. A collect-only stage, or plain counts with an interval, gets most of the value at a fraction of the cost.
