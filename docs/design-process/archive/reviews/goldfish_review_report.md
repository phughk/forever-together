# Goldfish Annotated Review Report

**Status:** resolved 2026-11-21 (author replied "auto accept"). All PROPOSE FIX items were applied, see the end of this file.
**Target:** `docs/designs/p2prdf.md` (full file, 627 lines)
**Findings source:** the 37 consolidated items in `consolidated_review_report.md` (B1-B8, M1-M24, N1-N5)
**Author decisions applied:** D1-D9 from `author_decisions.md`

## How to read this report

I wrote the design, so I did not contest anything that I only wanted to defend. Every item that a reviewer raised and a fresh reader could not resolve is a PROPOSE FIX. There are no CONTEST items. Three items are ESCALATE because the answer sits outside this repository or needs a number only the author has.

Because of D9, most fixes after the first group land in a new file, not in the design. The design keeps flow, interfaces and data flow. The byte table, defaults, API table and per-file list move out.

**Edit targets:**

| Code | File |
| --- | --- |
| DES | `docs/designs/p2prdf.md` (the design, after the move) |
| SPEC | `docs/rdf-code-spec.md` (new, holds the code format) |
| PLAN | `docs/designs/p2prdf-implementation-plan.md` (new, holds defaults, API table, per-file list, phases) |
| ADR | a file under `adr/` |

## Escalations (need the author)

*   **E-1 (B2).** What does the real client offer for group finding today (`GetLFGInfoServer`, `LFGList`, addon messages)? The answer is a fact about the live client, not in any file here.
*   **E-2 (B8, M20).** The reviewer's claims about the test VM (`GetCurrentRegion` already defined at `wowluavm/src/prelude.lua:154`, `GetSpecializationRole` being a global) come from `wow-forever-addons`, which is not in this repository, and I could not find a local copy. Someone with that repository open must confirm them before the plan is edited.
*   **E-3 (B1, D6).** The window lengths (response window, proposer decision window, heartbeat, human click) have no values. Only the author can choose starting values, or say they come from the simulation.

## 🔴 Blockers

### [Issue 1] [`egm-design`, `egm-zoom-out`] B1. Agreement mechanism is contradictory and unproven - Severity: Blocker
*   **Affected Scope:** [Lines 37, 171, 197-211](../archive/p2prdf-draft-2026-11-20.md#L197-L211) (Goals, sections 3 and 6)
*   **Discrepancy Description:** Commit needs five identical independent computations over samples that differ by design. The text disagrees with itself on the guarantee, the stage rule and the clock.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D1 allows the simulation. D2 says evaluate converging announcements against a proposer model in spike S0. D6 and the author's timing rules add Decline, heartbeat and "taken". The design must state all of that in one vocabulary (announcement, commit, confirm, decline, taken).
*   ***Proposed Edit:***
    *   Location: Goal on line 37; section 6; Open Questions line 81; spike S0
    *   Change: (a) Reword the Goal to "a peer confirms at most one party". (b) Rewrite section 6 with the five terms above, the split-brain outcome, and that a committed peer sends `Taken`. (c) Add section 6.1 "Candidate mechanisms": converging announcements (baseline) and the proposer model with a response window, with the lease acknowledged. (d) State the stage rule: when two peers differ, the stricter stage applies. (e) State the clock: each peer uses its own arrival time, skew tolerance and the n-1/n+1 epoch rule are SPEC defaults. (f) Mark level-biased eviction as proposed, linked to Open Questions. (g) S0 exit criterion: the seeded simulation reports commit rate for both mechanisms at N and S, and the one with the higher rate, above a floor the author sets (E-3), is adopted. If neither reaches the floor, the design is reopened.

### [Issue 2] [`egm-design`] B2. Problem premise unverified, in-game alternative never evaluated - Severity: Blocker
*   **Affected Scope:** [Lines 21-27](../archive/p2prdf-draft-2026-11-20.md#L21-L27) (The Problem), Alternatives
*   **Discrepancy Description:** Line 23 is a first-person "I have not checked". No alternative covers the client's own group finder.
*   ***AI Suggestion:*** ESCALATE (E-1), then PROPOSE FIX
*   ***Justification:*** The facts are not in any readable file. The structure can be fixed now.
*   ***Proposed Edit:***
    *   Location: Problem paragraph 1; Alternatives; Open Questions
    *   Change: Replace the note and the unprovable "slow" and "works worst" claims with the checked result once E-1 is answered. Add "Alternative P: the client's native finder or an addon-message rendezvous" with the concrete rejection reason (the source repository's `CLAUDE.md` records that public chat sends need a hardware event) or a go or no-go item in S0.

### [Issue 3] [`egm-implementation-readiness`] B3. No phase gates, items not mapped to phases - Severity: Blocker
*   **Affected Scope:** [Lines 374-389, 459-608](../archive/p2prdf-draft-2026-11-20.md#L374-L389)
*   **Discrepancy Description:** Steps 0-7 have no exit criteria, and the 28 items are not mapped to them.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D3 means no cut line, so only the order is needed. D9 puts this in PLAN.
*   ***Proposed Edit:***
    *   Location: PLAN, new "Order of work" table
    *   Change: One row per step 0-7 with its items, an exit command (for example `cargo test code dungeons engine store` passes), and what must compile together (`engine` before `net`). Steps 1 to 7 are marked blocked until S0 ends.

### [Issue 4] [`egm-implementation-readiness`] B4. The spike gates the plan but is not a work item - Severity: Blocker
*   **Affected Scope:** [Lines 378, 610-614](../archive/p2prdf-draft-2026-11-20.md#L610-L614)
*   **Discrepancy Description:** Items say "if the spike shows..." with no fallback. The spike label omits several unknowns.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D1 and D2 add the simulation and the mechanism choice to S0. D5 adds no gate.
*   ***Proposed Edit:***
    *   Location: PLAN, new "Item 0: Spike S0"
    *   Change: Required outputs and a fallback for each: realm and region calls (and whether realm fits 24 bytes); first name and surname calls; item level; cross-faction grouping; QR scan from a phone; `varaq` for the key (else OS keychain); iroh-gossip author identity (else 0019 is replaced); address lookup of a trusted peer from its key; the D1 simulation and D2 choice. Name an owner (the author, unless reassigned).

### [Issue 5] [`egm-implementation-readiness`] B5. Addon data collection not specified enough to code - Severity: Blocker
*   **Affected Scope:** [Line 491](../archive/p2prdf-draft-2026-11-20.md#L491) (item 6)
*   **Discrepancy Description:** Role derivation, item-level fallback, unknown encodings, empty zone or realm, truncation order and secondary role are undefined. "Never a realm" is stated as fact while the name calls are unverified.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid. ADR 0022 itself says the name behaviour is unverified.
*   ***Proposed Edit:***
    *   Location: SPEC, new "Collecting each field" table
    *   Change: Per field: source call, fallback, unknown value. Secondary role is always 0 in v1. The item-level fallback is one retry on the next `PLAYER_EQUIPMENT_CHANGED` event, so no polling. Clamp, then truncate at a character boundary. Mark `UnitName` and `GetUnitName` behaviour as assumed until S0 confirms it. Choose one role call (E-2).

### [Issue 6] [`egm-implementation-readiness`] B6. Engine, gossip and party logic is prose, not a contract - Severity: Blocker
*   **Affected Scope:** [Lines 197-211, 421-428, 570-574](../archive/p2prdf-draft-2026-11-20.md#L421-L428)
*   **Discrepancy Description:** Compatibility check, greedy tie-breaks, hash function, `partyId` encoding, dungeon mismatch, and the `Network` trait and event types are undefined.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid. The mechanism is not yet chosen (D2), so write the shared types now and the mechanism-specific rules after S0.
*   ***Proposed Edit:***
    *   Location: PLAN, new "Engine contract"
    *   Change: Rust type definitions for `Ad`, `Event`, `Action`, `Network`; pseudocode for compatibility (item level ±15% of the lower value, symmetric, unknown item level matches anything, level spread is max minus min, dungeon minimum level applies to all); the greedy walk with tie-break by peer ID; `partyId` as the SHA-256 of the postcard bytes of sorted members; the hash named (SHA-256 truncated to 8 bytes); a worked example with expected output. State that `net` verifies signatures, so `engine` receives verified messages only. Add the new messages: `Ready`, `Heartbeat`, and `Taken` as a core rule.

### [Issue 7] [`egm-implementation-readiness`] B7. Required data inputs not committed - Severity: Blocker
*   **Affected Scope:** [Lines 415, 419, 565-568](../archive/p2prdf-draft-2026-11-20.md#L565-L568)
*   **Discrepancy Description:** The 29-entry dungeon table is not committed, the shard table has no values, and `trusted.rs` cannot be tested.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D4 removes the shard table, so only the table and the test key remain.
*   ***Proposed Edit:***
    *   Location: PLAN items 21 and 23; new data file
    *   Change: The author commits `src-tauri/data/dungeons.json` (or names a file and revision) before item 21 starts. The topic hash keeps `shard` fixed at `0`. Remove the shard table, `shardCount`, and the shard formula. `trusted.rs` takes a `Vec<PeerId>` argument so tests pass their own key.

### [Issue 8] [`egm-codebase-details`] B8. Plan's claims about the existing test VM are wrong or ambiguous - Severity: Blocker
*   **Affected Scope:** [Line 516](../archive/p2prdf-draft-2026-11-20.md#L516) (item 11)
*   **Discrepancy Description:** `GetCurrentRegion` already exists in the VM, `world.region` is ambiguous, and no VM function returns the realm.
*   ***AI Suggestion:*** ESCALATE (E-2), then PROPOSE FIX
*   ***Justification:*** The claim is checkable only in `wow-forever-addons`, which I cannot read from here.
*   ***Proposed Edit:***
    *   Location: PLAN item 11
    *   Change: After E-2 confirms: say "modify the existing `G.GetCurrentRegion`" to `return setup.region or 3`, give the exact `setup.json` keys and the Lua expression that reads each, and name the VM function that returns the realm.

## 🟡 Majors

### [Issue 9] M1. Gossip authentication belongs in the architecture - Severity: Major
*   **Affected Scope:** [Lines 125-143, 346, 421](../archive/p2prdf-draft-2026-11-20.md#L421), Alternatives
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D8 accepts ADR 0019. Line 346 ("no authentication") contradicts it.
*   ***Proposed Edit:*** Add signing to the components list and section 6. Reword line 346 to "no authentication of content; messages are signed for authorship". Put topic ID and protocol version into the signed bytes. Keep the 0019 fallback line. Add "trust the forwarding neighbour" as Alternative Q.

### [Issue 10] M2. No load estimate for crowded topics - Severity: Major
*   **Affected Scope:** [Lines 193, 348](../archive/p2prdf-draft-2026-11-20.md#L348)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D4 means one swarm per topic, which makes the estimate worse, not better.
*   ***Proposed Edit:*** Add a back-of-envelope table (messages and bytes per second for a 3,000-peer topic: ads, announcements, `Taken`, heartbeats). State whether announcements go to the whole swarm or to the proposed members only. Reword "keeps the broadcast load low" as intent.

### [Issue 11] M3. Sharding is speculative and unspecified - Severity: Major
*   **Affected Scope:** [Lines 169, 195, 419](../archive/p2prdf-draft-2026-11-20.md#L169)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D4 settles this: one shard, hook kept.
*   ***Proposed Edit:*** Remove sharding from section 3 and widening stage 4. Keep `/0` in the topic hash and say why. Move shard tables and role-aware sharding to Future Improvements. Call "about 400 topics" an estimate. Update ADR 0005 to say v1 has one shard. Reconcile with ADR 0004 (no hard-coded realm list).

### [Issue 12] M4. Abuse resistance - Severity: Major
*   **Affected Scope:** [Lines 209, 244-246](../archive/p2prdf-draft-2026-11-20.md#L244-L246)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Line 245 says a small table limits domination, but a small table is easier to fill. D6's local 1-minute ban helps against ghost peers but not against key rotation.
*   ***Proposed Edit:*** Add a risk row stating the key-rotation gap, the rank-grinding risk for `hash(epoch, peerId)`, name harassment, and the accepted limits (per-peer rate limits, human Confirm, the local non-gossiped ban). Fix line 245.

### [Issue 13] M5. Risks section, trusted-peer dependency, hosted components - Severity: Major
*   **Affected Scope:** [Lines 242-248](../archive/p2prdf-draft-2026-11-20.md#L242-L248), Goals
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D5 states the supernode principle and accepts cold-start load. The risks must say that. The iroh relay and DNS defaults are unverified.
*   ***Proposed Edit:*** Replace section 10 with a table (risk, impact, mitigation or accepted, owner). Add: trusted-peer single point of failure and key rotation needing a release, young dependencies (`iroh-gossip`, `varaq`), IP exposure, behaviour when no trusted peer answers, and a trigger and test for the `varaq` fallback. Name iroh's default hosted relay and DNS components once S0 or the iroh docs confirm them (use context7), and state in Goals that trusted peers are a soft dependency.

### [Issue 14] M6. Quiet topics depend on the weakest discovery path - Severity: Major
*   **Affected Scope:** [Lines 177-189](../archive/p2prdf-draft-2026-11-20.md#L177-L189)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D5 keeps peer exchange. The eviction rule and cold-start load still need numbers.
*   ***Proposed Edit:*** Specify eviction now: reserve slots for the player's active topics. Reword lines 185 and 189 so trusted peers follow the peers (D5) and remove the claim that "most starts never contact a trusted peer" until measured. Add an estimate of trusted-peer load on first start.

### [Issue 15] M7. Peer exchange and bootstrap details incomplete - Severity: Major
*   **Affected Scope:** [Lines 183, 428](../archive/p2prdf-draft-2026-11-20.md#L428)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** In PLAN: framing, size limit, timeouts, whether `Hello` is its own stream, how long an ignored peer stays ignored and where it is stored, the `known_peers/<peerId>` record, and the cold-start rule. Define "a few at a time" and "the first that works" with values.

### [Issue 16] M8. Versioning can split the network - Severity: Major
*   **Affected Scope:** [Lines 169, 349](../archive/p2prdf-draft-2026-11-20.md#L349)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D4 removes the shard table, which was the main churn. The dungeon table still changes.
*   ***Proposed Edit:*** Keep the dungeon table out of the wire version: the topic uses only the slug, so a new dungeon is a new topic and does not move existing ones. State that rule.

### [Issue 17] M9. No success metrics or critical-mass analysis - Severity: Major
*   **Affected Scope:** Goals, Non-Functional Requirements
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Add target time to match and match rate (values are the author's, E-3), an adoption risk for the empty-room problem, and the notification path for the human Confirm (window focus, sound, phone vibration).

### [Issue 18] M10. Code format depends on unknowns and details are not pinned - Severity: Major
*   **Affected Scope:** [Lines 146-159, 393-413](../archive/p2prdf-draft-2026-11-20.md#L393-L413)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D8 accepts ADR 0018. The details belong in SPEC (D9). The space-stripping bug is valid: the base45 alphabet contains a space.
*   ***Proposed Edit:*** In SPEC: a canonical realm and region form (numeric ID or normalization) so topic strings match; a size goal in bytes and the QR capacity table; a definition of `flags`; a validation table with error variants; counter wraparound at 65535; trailing bytes rejected; level 0 and empty first name rejected; one error vocabulary. In `code.rs`: strip leading and trailing whitespace only, with a test vector holding an internal space. Specify which message carries first name and surname, and settle the GUID question from ADR 0016. Narrow QR scope to alphanumeric mode at level M. Remove "from my memory of the QR tables".

### [Issue 19] M11. Client assumptions gate fixed layout - Severity: Major
*   **Affected Scope:** [Lines 74, 393-413](../archive/p2prdf-draft-2026-11-20.md#L74)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Make cross-faction grouping an S0 output. Record faction in the ad as well, so changing the topic key is a version bump and not a redesign. Mark spec bytes that depend on S0 as provisional.

### [Issue 20] M12. Repository story and file boundary inconsistent - Severity: Major
*   **Affected Scope:** [Lines 240, 372, 513-526](../archive/p2prdf-draft-2026-11-20.md#L240)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Rewrite line 240 to match line 372 (only VM additions in `wow-forever-addons`, used as a submodule). Name the submodule in section 9 and the test Goal. Remove `GetInstanceInfo` from ADR 0021. Fix item 13 (it moves files into this repository). In PLAN, list every file once as Create, Modify or Delete, adding `Cargo.lock`, `.gitmodules`, `.gitignore`, `vite.config.ts`, `index.html`, `.storybook/`. Add a path guard to `install-addon.sh`.

### [Issue 21] M13. Scope and delivery risk - Severity: Major
*   **Affected Scope:** Goals, section 7
*   ***AI Suggestion:*** DECLINE
*   ***Justification:*** D3 chose one v1. I record the risk as an accepted trade-off in the risks table (Issue 13) and add nothing else. The HMAC-per-request idea is not taken up.
*   ***Proposed Edit:*** None, beyond the risk row.

### [Issue 22] M14. Altitude: implementation detail duplicated and drifted - Severity: Major
*   **Affected Scope:** [Lines 370-623](../archive/p2prdf-draft-2026-11-20.md#L370-L623) and the contradictions at lines 165/417, 193/433, 195/441, 199/434, 34
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D9.
*   ***Proposed Edit:*** Move: the byte table and shared message definitions to SPEC; defaults, API table, Target Files and Verification Plan to PLAN. Leave in DES: components, flows (with a new sequence diagram for epoch, commit, ready, confirm), interfaces, risks, alternatives. Keep one authoritative copy of each repeated fact and cross-link it. Resolve the contradictions: epoch and keep-alive values live only in PLAN; the widening order has one form; the slash command is `/wowft` (D7); line 34 and line 165 agree that there is no distance calculation.

### [Issue 23] M15. Alternatives give attributions, not reasons - Severity: Major
*   **Affected Scope:** [Lines 250-342](../archive/p2prdf-draft-2026-11-20.md#L250-L342)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Replace "Rejected by author" with the technical reason in E, I, J, L, M. Remove unsourced figures ("20 nodes", "48 hours", "about ten times more topics") or add a source. Split G, H and O. Add: in-game finder (Issue 2), signed gossip versus trusting the neighbour (Issue 9), base45 versus other encodings, and the proposer model (Issue 1). Replace bare ADR paths with relative links.

### [Issue 24] M16. Local API contract incomplete - Severity: Major
*   **Affected Scope:** [Lines 213-221, 346, 444-457](../archive/p2prdf-draft-2026-11-20.md#L444-L457)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** In PLAN: payloads for `GET /api/state` and events, error body, status codes (401 against 403), size and rate limits, token format and comparison, port choice, Host allow-list, event-stream heartbeat, LAN interface choice, the loopback check by socket peer address, and what restarts when `lan_mode` changes. Align the loopback route list across section 7, Non-Functional Requirements and the table. Say "fetch streaming" instead of "server-sent events" where the token travels in a header. Add the per-install token's lost-phone cost to Alternative N.

### [Issue 25] M17. Settings versus protocol constants - Severity: Major
*   **Affected Scope:** [Lines 430, 597](../archive/p2prdf-draft-2026-11-20.md#L430)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** In PLAN, split the defaults table into "user settings" (LAN mode, known-peers size, trusted peers) and "protocol constants" (epoch, expiry, windows, widening). Constants are not changeable per peer.

### [Issue 26] M18. Widening stage table is ambiguous - Severity: Major
*   **Affected Scope:** [Line 441](../archive/p2prdf-draft-2026-11-20.md#L441)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** In PLAN: drop the shard stage (D4) and renumber. State the stage clock start, its restart after a dropped party, and the behaviour at the search limit. Reword the stage list in section 5 to the table's order. Constants are not per-peer settings (Issue 25).

### [Issue 27] M19. Dungeon recommendation rule gaps - Severity: Major
*   **Affected Scope:** [Lines 417, 568](../archive/p2prdf-draft-2026-11-20.md#L417)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Final sort key is the slug. Place `none` last in the faction-hint order. State the case folding (ASCII lowercase). Give the topic-list function signature. Use slack 3 everywhere.

### [Issue 28] M20. Existing VM details and addon load order - Severity: Major
*   **Affected Scope:** [Lines 476, 490, 516](../archive/p2prdf-draft-2026-11-20.md#L490)
*   ***AI Suggestion:*** ESCALATE (E-2) for the VM claims, PROPOSE FIX for the load order
*   ***Justification:*** The load-order defect is visible here: `Core.lua` creates `NS.RD` but the TOC loads `Codec.lua` and `Qr.lua` first.
*   ***Proposed Edit:*** Create `NS.RD` in `Codec.lua`, the first loaded file. After E-2, pin the role call and the `UnitName` handling, and add the signature, error behaviour and registration point for `vm.read_json`, the submodule URL and pin, and a `CLAUDE.md` update.

### [Issue 29] M21. Dependencies and template unpinned - Severity: Major
*   **Affected Scope:** [Lines 356, 535-538](../archive/p2prdf-draft-2026-11-20.md#L535-L538)
*   ***AI Suggestion:*** ESCALATE
*   ***Justification:*** Needs the template commit and the author's choice of crates, which are not in any file here (the template is on a private host).
*   ***Proposed Edit:*** Once supplied: pin the template commit, pin crate versions and features, state whether base45 is hand-written, and name the keychain crate.

### [Issue 30] M22. Test plan lacks assertions - Severity: Major
*   **Affected Scope:** [Lines 600-603, 618-623](../archive/p2prdf-draft-2026-11-20.md#L600-L603)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D1 permits the simulation. The missing tests are valid.
*   ***Proposed Edit:*** In PLAN: expected outputs for the Barrens cases; an instance count and scenario for the integration test; add tests for signature rejection, `partyId` determinism, `Taken`, ad expiry, `Decline`, no heartbeat, the 1-minute local ban (not gossiped), the search limit, LRU order, the pairing URL, event streaming, identity persistence, flags, and Horde against Alliance. Add the seeded simulation (D1) with its pass floor (E-3). Amend ADR 0014.

### [Issue 31] M23. Acceptance criteria subjective or ownerless - Severity: Major
*   **Affected Scope:** [Lines 593, 612-614, 623](../archive/p2prdf-draft-2026-11-20.md#L612-L614)
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Replace "stores safely", "meaningful states" and "looks right" with observable conditions (key file readable only by its user and kept after restart; the QR decodes with two phone scanners at UI scales 0.8, 1.0 and 1.2). Give the owner and the calls to log.

### [Issue 32] M24. ADR status and references - Severity: Major
*   **Affected Scope:** Header (line 4), line 252, Additional Resources
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** D8 changes 0018 and 0019 to Accepted.
*   ***Proposed Edit:*** Update ADR statuses. Update the header date. Summarize ADR 0022 where cited. State that the ADRs are required reading and link them with working relative paths.

## 🟢 Nits

### [Issue 33] N1. Repeated statements - Severity: Nit
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Cheap, and D9 forces a rewrite of those paragraphs anyway.
*   ***Proposed Edit:*** State each repeat once and cross-link: forgery, HTTP token risk, trusted peers, permanent key, known-peers table, bootstrap order, "no load tests". Merge lines 185 and 189.

### [Issue 34] N2. Wording and vagueness - Severity: Nit
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Split the 190-word paragraph at line 17. Define "sharded mega server" or drop it (D4 removes most of it). Source or remove "two million". Change "statically hosted" to "embedded". Replace "a few", "some", "several" with values. Remove the stray space at line 603 and "plainly". Replace "when the design is finished" with "after sign-off".

### [Issue 35] N3. Structure - Severity: Nit
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Promote Additional Resources and Future Improvements to H2. Triage Open Questions into blocking (realm, names, S0 outputs) and tuning. Number "Components". Add the Tauri window and headless mode to the diagram.

### [Issue 36] N4. References and paths - Severity: Nit
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Fix the links that do not resolve from `docs/designs/`. Remove the absolute `/Users/...` paths at lines 354 and 372. Name the template and use "maintainer" for "the author".

### [Issue 37] N5. Small inconsistencies - Severity: Nit
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Proposed Edit:*** Decide GUID (Issue 18). Reword item 9 as assumed until S0. Remove "dungeon" from the window field list. Say "shown textures". State that QR vectors are compared as 0 and 1 against booleans. Name the TOC flavour marker. State that jitter applies before the 15-minute cap.

## New work created by the author's decisions (not in the review)

*   **N-A.** A new ADR for the timing and liveness rules (D6): Ready check, Decline without penalty, heartbeat, local 1-minute ban that is never gossiped.
*   **N-B.** A new ADR for the supernode principle (D5).
*   **N-C.** Amendments to ADR 0005 (one shard), 0010 (Decline, Ready, proposer alternative under evaluation), 0014 (simulation), 0020 (names), and status changes to 0018 and 0019.
*   **N-D.** Rename `p2prdf` to `forever-together` or `wowft` across the design and ADR filenames that mention it (D7), after the identifiers are approved.

## Counts

PROPOSE FIX: 31 (counting Issues 2, 8 and 28 once each as their fixable half). ESCALATE: 3 open (Issues 2, 8/28, 29) plus E-3. DECLINE: 1 (Issue 21). CONTEST: 0.

## Update 2026-11-21: escalations answered

E-1, E-2 and E-3 are answered in `author_decisions.md`. Effects:

*   Issue 2 (B2) becomes PROPOSE FIX. Premise: Forever has a manual classic-style group finder, no random finder. Drop the first-person note.
*   Issue 8 (B8) and Issue 28 (M20) become PROPOSE FIX. The VM claims are checked and confirmed against `/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons`.
*   E-3 is answered with starting values.
*   Issue 29 (M21) stays partly open: the template commit is pinned (`9048274`), crate versions and the keychain crate are not.
*   The template is copied into this repository, so PLAN item 14-17 ("Modify" the template's files) now refer to files that exist.

## Resolution summary 2026-11-21

Applied by auto accept:

*   New: `docs/rdf-code-spec.md`, `docs/designs/forever-together-implementation-plan.md`, ADR 0023 (no supernodes), ADR 0024 (Ready, Decline, heartbeat).
*   Rewritten: the design is now `docs/designs/forever-together.md`. The first draft is archived at `docs/design-process/archive/p2prdf-draft-2026-11-20.md`.
*   Amended: ADR 0005, 0010, 0012, 0014, 0018 (Accepted), 0019 (Accepted), 0020, 0021, and the ADR index.
*   Identifiers renamed to the `wowft` set.

Not applied, as reported: Issue 21 (M13, declined, recorded as a risk row); Issue 29 (M21): crate versions and the keychain crate are still unpinned (the plan says they are pinned when step 3 starts); the simulation floor, window lengths and `dungeons.json` still need the author's confirmation or commit.

Known limits of this pass: a re-audit of the revised documents has not been run, and the load estimate and the Barrens test expectations are placeholders until the simulation and the committed table exist.
