# Concepts Brief: Search diagnosis

Author: Hugh Kaznowski. Date: 2026-11-21. Mode: BRAINSTORM.
Origin: `docs/notes/ui_notes.txt` line 11.
This brief is separate from `docs/design-process/concepts_brief.md`, which is an older, unrelated brainstorm.

## Reviewer status

| Reviewer | Status |
| --- | --- |
| `egm-design` | completed |
| `egm-codebase-details` | completed |
| `egm-zoom-out` | completed |
| `egm-comprehension` | completed |
| `egm-implementation-readiness` | completed |
| `egm-structural` | completed |
| `egm-inventive-catalyst` | completed |

## Overview

Hypothesis, agreed with the author: while a search runs, and in a one-to-two sentence summary when it ends, the app tells the player in plain language whether it is working and whether a group is plausible. It uses only what this node observed: peers connected, players seen, a rough role mix such as "8 players, 0 tanks", and invitations and parties attempted. The player can then decide to wait, change dungeon, or give up and turn the app off.

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

## Alternatives Weighed

| Option | What it covers | Cost | Verdict |
| --- | --- | --- | --- |
| A. Cheap slice: add role counts and a frozen peak snapshot to the existing card; a static end summary from the same counts | "players seen", "0 tanks", "no peers" | A few engine fields, one `SearchView` field, one Field row, stories | Zoom-out lens: covers most of the value. Build first. |
| B. Cheap slice plus pure `diagnose` with 5-6 named situations, shown with the raw counts | Rolling sentence, end summary, replaces `lowCount` | A (above) plus counters for drops and invitations, copy deck, stories per situation | Recommended target. |
| C. Full version: hysteresis state machine, feasibility counts, "what would change this" counterfactuals, differential display, ghost ratio | Most of the wish list | Second product; copy, tests and tuning for each new situation; coupled to ADR 0025 internals | Defer. |
| D. Numeric plausibility score | A single number | False precision, uncalibrated | Reject. |
| E. Do nothing beyond fixing the end card copy | End card only | Minimal | Misses the rolling feedback the author asked for. |

Dissent to record: the zoom-out lens argues that the invitation and party-attempt counters couple the feature to a mechanism that is still in flux (0010 was just replaced by 0025), and wants them deferred. Design and inventive-catalyst lenses see the invitation signal as the only way to separate "nobody answers" from "no tank". Resolution proposed below: stage the counters behind the first slice.

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

## Key Decisions & Staging for Next Steps

### Decisions the author needs to make

1. Verdict scope: confirm that the app reports observations plus a few hedged situations, not a plausibility judgement (all lenses recommend this).
2. Whether invitation and party counters are in the first slice or staged after it (zoom-out dissent above).
3. Where classification lives: a pure module in the engine crate is the proposal; confirm.
4. Multi-dungeon scope: report per dungeon, or an aggregate with its scope stated ("across 3 dungeons"). Extra topics join only at stage 4 (`state.rs:573-583`), and `topics` in the view falls back after expiry because `stage` reads 0.
5. Counting rule for "players": compatible under the current stage's rules, or all stored ads; and whether banned peers (60 s local ban) are included. Either way the UI label needs to change from "Players searching".
6. Lifecycle of the end snapshot: kept until Clear or the next search. What `UserStop` does with it. Manual stop currently maps to Idle and shows nothing.
7. Whether to add ADR 0026 (proposed title: the app reports search health from this peer's own observations; amends 0011, cites 0003, 0005, 0025). Structural and implementation-readiness lenses both say yes.

### Decided by the author after the review

- Keep history as per-search entries with start and end datetimes, role counts at checkpoints and the outcome (section above).
- All four counts (players, tanks, healers, damage) are tracked.
- History is internal and never shown.
- Expired searches, and searches the player stopped after more than one minute, count as failure samples.
- Estimates only when the history is large enough and dependable.
- Time of day and day of week are recorded but need not be used by the estimate.

### Proposed staging

1. **ADR 0026 and a short design-doc note.** Record the observation-only rule, the hedging, in-memory retention and the engine/UI split. Update the implementation plan's API table and file list.
2. **Engine facts.** `SearchStats` on `Engine`: role histogram of the compatible sample, `sample_full`, own role, peak players, stage reached. Snapshot before `stop()`; clear on `start()`; do not reuse `last_drop` unguarded. Test with a solo tank through expiry (the existing test at `engine/mod.rs:715-741` is the base).
3. **Pure `diagnose`.** A closed enum: not connected, alone, no tank, no healer, short on damage, healthy-looking, too early. Constants named with their source. Table tests at fixed durations. Include an invariant test that no message claims more than the observations support.
4. **API.** One optional `diagnosis` object on `SearchView`, additive and tolerant of unknown codes, so an older bundle (or a `WOWFT_WEB_DIR` override) falls back to the old copy. Change `server/core.rs`, `types.ts`, `fixtures.ts`, `mockApi.ts`, `server/tests.rs`, `app/mod.rs` (`placeholder_state`) and `app/tests.rs` together.
5. **Webapp.** A `DiagnosisSummary` molecule built from `Notice`, `Field` and `RoleIcon` (no new atom unless `Notice` lacks an info tone). Integrate in `SearchProgress` for both the running and ended cards, and remove the inline `lowCount` Notice so there is one source of truth. Story variations: warming up, not connected, alone, no tank, no healer, full sample versus partial, multi-dungeon, long dungeon list, ended per situation, one versus two sentences. Add the character-length limits to the molecule's story docs. Use invented data only ("Testa Fakename", "Testville").
3. (later) **Counters for attempts.** `invites_sent`, `accepts_got`, `invites_got`, drops by cause, `taken_seen`; role-aware copy; the "nobody answered" situation gated on a minimum attempt count.
4. (later) Hysteresis, doubt level, feasibility sentence, counterfactual hints.
5. (later) **History and estimates.** The entry type, the store key, prune-on-append, a pure estimator with a minimum-entries gate, and tests with invented data. Build after the observation slice, so the checkpoint counts it records already exist. Needs its own ADR (supersedes the "no persistence" assumption above).
6. **Live check.** Mark all thresholds unverified until run against a live network, in line with the S0 status.

### Acceptance criteria

- At expiry the summary shows the real numbers; it never shows 0 caused by `stop()`.
- The same observations always give the same situation; no wall clock in the classifier.
- No message asserts more than was observed ("none seen", never "none exist"), and none names a person.
- The advice agrees with the stage reached.
- An old web bundle against the new server renders the previous copy.
- The first slice makes no store or schema change. The history stage adds one new store key and nothing else.
- Every new component has stories at its level; `mise run lint`, `cargo test --lib engine::`, `pnpm --dir webapp typecheck` and `build-storybook` pass.

## Deduplicated findings

Severity: Blocker (B), Major (M), Nit (N). Lens names in brackets.

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

## Three strongest objections

1. **The data the end summary needs is gone at expiry, and the live counts do not mean what the label says.** `stop()` clears the search before the Expired view is built, so a naive summary shows 0 players. `peers_seen` also counts incompatible and stale ads from a capped, level-biased sample.
2. **An honest verdict cannot be built from this node's view.** The sample is partial and forgeable, "0 tanks" can be a sampling artifact or the player's own role, and invitation silence has several causes. The feature must stay at hedged observations and a few asymmetric situations, and thresholds are unvalidated without a live network.
3. **The full concept is oversized for what is proven.** Attempt counters and a narrating state machine couple the feature to ADR 0025, which was recently replaced, and add a copy deck to maintain. Role counts, a frozen peak snapshot and a short deterministic summary cover most of the stated need.
