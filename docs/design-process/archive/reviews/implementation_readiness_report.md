# Goldfish Review Report

**Target:** `/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons/docs/designs/p2prdf.md` (lines 370-627, Detailed Implementation Plan)

## Reviewer Status

| Reviewer | Status |
| --- | --- |
| `egm-implementation-readiness` | completed |
| `egm-codebase-details` | completed |
| `egm-style` | completed |

Overall verdict: not ready. The addon half (items 1-12) is close once the issues below are fixed. The Rust half needs engine and net contracts, API payloads, committed data tables and phase gates before it passes the Zero-Decision Rule.

## Prioritized Findings (Blockers -> Major -> Nits)

### Phases have no build/test gates and do not map to items - Severity: Blocker
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Lines 376-389, 459-608
* **Discrepancy Description**: The diagram's steps 0-7 do not map to the 28 numbered items (grouped only as Part A and Part B). No step has an exit criterion. Item 15 adds every dependency up front. Item 23 (`net/`) needs `Network` types that item 22 (`engine`) consumes, and the order of 22 versus 23 is not stated.
* **Rationale & Recommendation**: Map items to steps S0-S7, give each an exit command (for example "`cargo test code dungeons engine store` passes"), and state which items must compile together.

### Spike (S0) gates the plan but no outcome branches are given - Severity: Blocker
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Lines 378, 421, 491, 558, 614 (items 1, 4, 6, 19, 23)
* **Discrepancy Description**: Items depend on unknown spike results: the realm and region calls, whether gossip identifies authors (ADR 0019), `varaq` suitability for the secret key, and address lookup of trusted peers. Items say "if the spike shows..." with no alternative design. The implementer cannot write `Collect()` without knowing the realm call. The realm may also not fit "24 bytes".
* **Rationale & Recommendation**: Make the spike its own work item with required outputs (API call per field, `varaq` yes/no, gossip author identity yes/no) and a fallback per outcome. Mark S1 onward as blocked on it and name the spec lines to update.

### Addon data collection (item 6) not specified precisely enough to code - Severity: Blocker
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-codebase-details`, `egm-style`
* **Affected Scope:** Line 491 (item 6)
* **Discrepancy Description**: Sources are a mix of confirmed and unconfirmed calls. Undecided: role derivation from spec; which `C_SpecializationInfo` functions; the item-level fallback's async retry (conflicts with the "never poll" rule in CLAUDE.md); the "unknown" encoding for class, race, level and region; what zone and realm hold when empty; clamping and truncation order; and how secondary role (flag bits 2-3) is filled. The sentence is also a long nested parenthesis that mixes assumed Forever behaviour with general API rules, and states "never a realm" absolutely.
* **Rationale & Recommendation**: Replace with a per-field table (source call, fallback, unknown value). State the secondary-role rule (for example "always 0 in v1"). Mark `GetUnitName`/`UnitName` behaviour as assumed and confirmed by the client check in Prerequisites.

### Engine, gossip and party logic is prose, not a contract - Severity: Blocker
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Lines 421-428, 570-578 (items 22, 23)
* **Discrepancy Description**: Undefined: the compatibility predicate (item level +-15% of whose value, symmetry, unknown item level, level spread measure, dungeon `minLevel`); greedy partition tie-breaks, relaxed and secondary role assignment, and whether a peer builds its own party or the whole partition; epoch time source, boundary and ad cutoff (no Defaults row); `partyId` serialization and which topic is hashed; the hash used in `hash(epoch, peerId)`; behaviour when members agree but dungeons differ; shard-join rule and shard count; and the `Network` trait signature, the `Event` and `Action` types and the error model.
* **Rationale & Recommendation**: Add an engine contract section with Rust type definitions, pseudocode for the compatibility check and greedy walk, and a worked example with expected output a test can assert.

### Required data inputs are missing: dungeon/shard table and trusted peers - Severity: Blocker
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Lines 419, 578 (items 21, 23, 27)
* **Discrepancy Description**: Item 21 says to embed "the 29 entries from the author's table", which is neither a path nor committed content, so the item 27 assertion (29 entries, 20 available) cannot be met. The shard table is not a listed file and `shardCount` has no value. `trusted.rs` is empty with no test-key mechanism.
* **Rationale & Recommendation**: Commit the table (or pin a file and revision), specify the shard table format (for example "all shard counts 1 in v1"), define `shardCount`/shard table in the Defaults table, and specify a test-key mechanism for trusted peers.

### `GetCurrentRegion` already exists in the VM; the "add" step is wrong - Severity: Blocker
* **Reviewer Lens:** `egm-codebase-details`, `egm-implementation-readiness`
* **Affected Scope:** Line 516 (item 11); `wowluavm/src/prelude.lua:154`
* **Discrepancy Description**: `function G.GetCurrentRegion() return 3 end` is already at `prelude.lua:154`, before `local world = setup` (line 523). A new definition using `world` there would read nil. A later duplicate overrides the first. The plan names neither the edit nor the placement relative to the `C_` wrapping block (lines 1056/1060).
* **Rationale & Recommendation**: Say "Modify the existing `G.GetCurrentRegion`" with the exact body (for example `return setup.region or 3`), preserving the default 3 so the Social Hub suites are unchanged.

### Setup key for region is ambiguous - Severity: Blocker
* **Reviewer Lens:** `egm-codebase-details`, `egm-implementation-readiness`
* **Affected Scope:** Lines 506, 516 (items 9, 11)
* **Discrepancy Description**: In the VM `world = setup`, so the top-level JSON is the world. The plan says `world.region`, which could mean a top-level `"region"` or a nested `"world": {...}` that the prelude never reads. Realm and surname keys are also unnamed, and no VM function returns the realm in tests, so `test_core` cannot be written.
* **Rationale & Recommendation**: Give the exact setup JSON, the Lua expression reading each key, and the VM function that returns the realm value.

### `GetSpecializationRole` is not in `C_SpecializationInfo` in the API export - Severity: Major
* **Reviewer Lens:** `egm-codebase-details`
* **Affected Scope:** Lines 491, 516; `prelude.lua:659`
* **Discrepancy Description**: The export has `GetSpecializationRole` only as a global. The plan adds it to `C_SpecializationInfo`, so with `--api` the real call is reported unimplemented while the fake goes unused. `GetSpecializationInfo` already returns a role as its 5th value.
* **Rationale & Recommendation**: Pick one call (global, or `GetSpecialization` plus `GetSpecializationInfo`), make the VM addition match, and give the TANK/HEALER/DAMAGER to 1/2/3 mapping.

### VM name functions do not behave as the plan assumes - Severity: Major
* **Reviewer Lens:** `egm-codebase-details`, `egm-implementation-readiness`
* **Affected Scope:** Line 491; `prelude.lua:547-559`
* **Discrepancy Description**: `G.UnitName` returns `name, realm` (realm nil at home) and `UnitFullName` returns `name, realm`. A test taking the surname from the second return of `UnitName` gets nil. The plan does not say `player.name` carries "First Surname" or whether the VM changes.
* **Rationale & Recommendation**: State that `Collect` splits `GetUnitName("player")` on the first space (no VM change), or list the `UnitName` change.

### Pasted-text normalization breaks base45 - Severity: Major
* **Reviewer Lens:** `egm-codebase-details`, `egm-implementation-readiness`
* **Affected Scope:** Line 563 against line 393 (items 1, 20)
* **Discrepancy Description**: Base45's alphabet contains the space. Stripping all spaces corrupts valid codes and fails the CRC.
* **Rationale & Recommendation**: Strip only leading and trailing whitespace and line breaks. Add a test vector with an internal space.

### Codec and validation details not pinned - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Lines 393, 466, 481 (items 1, 4)
* **Discrepancy Description**: Undecided: `counter` wraparound at 65535, the `created` time source, trailing-byte rejection in `FromText`, valid ranges (level 0, empty first name), encoding of Neutral or unknown faction (flag bit 4), and region byte versus `GetCurrentRegion()` values.
* **Rationale & Recommendation**: Add a validation table (field, valid range, error variant) to item 1.

### QR scope, test vectors and capacity claim - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-style`
* **Affected Scope:** Lines 413, 468-471, 483-487, 511
* **Discrepancy Description**: Item 5 covers levels L-H, versions 1-10 and all masks, though only alphanumeric level M is needed. Vectors come from an unnamed "independent reference encoder". Line 413 says "from my memory of the QR tables" and "version 4 or 5 (122 characters)" is ambiguous. The maximum record size (about 126 bytes, 190 characters) is not computed against version 10 level M (213), and the window's behaviour when text does not fit is undefined. "Looks right" and "real look of the QR" are subjective.
* **Rationale & Recommendation**: State intended scope, name the reference encoder and inputs, give the capacity table per version, state the maximum record size and the refusal behaviour, and replace subjective criteria with observable ones (for example "decodes with two phone scanners at UI scales 0.8, 1.0, 1.2").

### Load-order bug: `RD` namespace created after files that use it - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Items 3, 6, 8
* **Discrepancy Description**: Item 6 sets `NS.RD = {}` in `Core.lua`, but TOC order loads `Codec.lua` and `Qr.lua` first and they use `RD.Codec`.
* **Rationale & Recommendation**: Create `NS.RD` in the first loaded file, or add an `Init.lua` first in the TOC and file list.

### Item 11 VM additions incomplete or unclear - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope:** Lines 513-516
* **Discrepancy Description**: ADR 0021 lists `GetInstanceInfo` but item 11 omits it. `vm.read_json` has no signature, error behaviour, registration point (after `main.rs:153`, pattern `vm_fn` at `main.rs:107`) or working-directory assumption. The cross-repo submodule step has no ordering guard, URL or pin procedure. New setup keys and helpers are not added to the VM docs in CLAUDE.md. The `serde_json`/`mlua` serialize claim is true (`Cargo.toml` lines 9-11).
* **Rationale & Recommendation**: Specify these details, and add a documentation update for CLAUDE.md.

### Dungeon recommendation rule gaps - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-style`
* **Affected Scope:** Lines 417, 419, 565-568
* **Discrepancy Description**: No tie-break for equal distance, no placement of `none` in the faction-hint order, locale-unaware case-insensitive zone compare, and an unspecified topic-list function (realm and region are opaque user values). "Shard table" and `shardCount` are undefined in this section.
* **Rationale & Recommendation**: Add a final sort key (slug), define the `none` ordering, state folding, give the signature, and define shard terms with a Defaults row or cross-reference.

### Widening stage table ambiguous - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Line 441, Defaults table, item 18
* **Discrepancy Description**: Stage 4 does not say which or how many shards, or whether they count in the sample. Stage 5 level checks are unstated. The stage clock start, restart after a dropped party, and search-limit end behaviour are undefined. Peer-local settings changes break the shared schedule (ADR 0011).
* **Rationale & Recommendation**: Specify each, and mark which settings are read-only protocol constants.

### Settings editability contradicts protocol convergence and the Settings page spec - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-style`
* **Affected Scope:** Lines 430, 439, 455, 597
* **Discrepancy Description**: Line 430 says all defaults can be changed on the Settings page, yet item 26 lists only LAN mode, token reset, known-peers cache size and trusted peers. Letting each peer change epoch length or confirm timeout breaks convergence.
* **Rationale & Recommendation**: List exactly which keys are user settings versus protocol constants and align the two statements.

### Peer exchange and bootstrap protocol incomplete - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-style`
* **Affected Scope:** Lines 428, 575-578
* **Discrepancy Description**: No framing, size limit, serialization for exchange, timeouts, or response to a missing `Hello`. Whether `Hello` is its own stream is unclear against "one request and one response per stream". Rate-limit enforcement, ignore duration and storage are unspecified. "First that works", "a few at a time" and the neighbour timeout are undefined. "Use" for `known_peers` LRU is an open choice. The record format and index for `known_peers/<peerId>`, trusted-peer address and relay sources are unspecified.
* **Rationale & Recommendation**: Pin each decision in the contract section.

### Server and API contracts missing bodies and codes - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Lines 444-457, 585-588 (items 17, 25, 26)
* **Discrepancy Description**: Only `POST /api/character` has a body. Missing: `GET /api/state` and SSE payloads, error JSON, status codes (403 vs 401), size and rate limits, token format and comparison, port choice, Host allow-list, SSE reconnection and heartbeat, LAN interface selection, whether a loopback check uses the peer socket address, restart behaviour when `lan_mode` changes, how the desktop window recovers after a token reset, and the asset-serving mechanism (Tauri asset resolver versus `rust-embed`/`include_dir`).
* **Rationale & Recommendation**: Define these so items 25 and 26 can be built independently.

### Error vocabulary inconsistent - Severity: Major
* **Reviewer Lens:** `egm-style`
* **Affected Scope:** Lines 450, 453, 563 (items 4, 20)
* **Discrepancy Description**: Item 20 lists `UnknownVersion`, `BadCheck`, `Truncated`, `BadText`; the route lists the first three; Lua returns an unspecified `err`.
* **Rationale & Recommendation**: Define one error vocabulary in Shared definitions and reference it.

### Dependencies and template unpinned or hedged - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-style`
* **Affected Scope:** Lines 372, 530-543 (items 14, 15, 26)
* **Discrepancy Description**: The template (`template-rust-app`) has no starting commit or instantiation method. Crate versions are unpinned (`iroh`, `iroh-gossip`, `varaq`, `axum`, `tauri`), feature sets unspecified, and phrases like "only if needed", "`base64` or `data-encoding` as needed", "a QR library such as `qrcode`" and "`ed25519` through iroh" leave choices open. A base45 crate and an OS keychain crate (line 558) are missing from the list.
* **Rationale & Recommendation**: Name and pin crates (or state hand-written base45), and pin the template commit.

### Test plan lacks concrete assertions and several tests - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Item 27, line 603
* **Discrepancy Description**: Expected outputs are missing (for example the Barrens levels 15, 25, 38 order). The integration test has no instance count or scenario. Missing tests: signature rejection, `partyId` determinism, `Taken`, ad expiry, `Decline`, search limit, LRU ordering, pairing URL, SSE, identity persistence, CLI flags, and Horde versus Alliance exclusion. The web app has only stories and typecheck.
* **Rationale & Recommendation**: Add the data and the missing tests.

### File boundary incomplete and inconsistent with the scope claim - Severity: Major
* **Reviewer Lens:** `egm-implementation-readiness`, `egm-style`
* **Affected Scope:** Lines 372, 459-608, 518-526 (items 12, 13, 28)
* **Discrepancy Description**: Line 372 says "its only change is the small set of test VM additions" and "every file touched is listed", but item 13 moves `adr/`, `concepts_brief.md`, `design_draft.md` out of `wow-forever-addons` (and the working tree has modified and untracked files). Unlisted: a shared message types file, `Character` fields, `Cargo.lock`, `.gitmodules`, `.gitignore`, `vite.config.ts`, `index.html`, `.storybook/`, individual stories and components. `mise.toml` and `test-addon` appear in both items 12 and 28 with an unclear Create/Modify mark. The install script needs `set -u` and a path guard.
* **Rationale & Recommendation**: List destinations and deletions, mark each path Create/Modify/Delete once, and fix the scope sentence.

### Acceptance criteria and spike deliverables subjective or ownerless - Severity: Major
* **Reviewer Lens:** `egm-style`
* **Affected Scope:** Lines 612-614, 622-623
* **Discrepancy Description**: "Stores the identity safely", "meaningful states" and "someone logs..." have no observable condition or owner.
* **Rationale & Recommendation**: State observable conditions (for example key file readable only by the user, key survives restart) and name the owner and calls to log.

### Local personal path and circular wording in scope paragraph - Severity: Nit
* **Reviewer Lens:** `egm-style`
* **Affected Scope:** Line 372
* **Discrepancy Description**: `/Users/hughkaznowski/Projects/github/forever-together` is a personal path. "The author's template" and "the author" are vague. "Because the design says they are tuned from use" is circular.
* **Rationale & Recommendation**: Drop the path, name the template or use "maintainer", and say "expected to change after field use".

### Item 6/7/10 small inaccuracies - Severity: Nit
* **Reviewer Lens:** `egm-implementation-readiness`
* **Affected Scope:** Items 2, 3, 7, 10, 22, 23
* **Discrepancy Description**: Item 7 lists "dungeon" in the window though the code has no dungeon field. Item 10 "number of QR textures equals dark modules" should say shown textures. QR vectors store 0/1 rows but `Qr.Encode` returns booleans and the comparison is unspecified. The TOC flavour marker is not stated. Signature verification sits in `net`, so the engine receives only verified messages (state this). Jitter applies before or after the 15 min cap is unstated. `check` versus `test-addon` coverage of `cargo test` is unclear.
* **Rationale & Recommendation**: Fix each as noted.

### Wording nits - Severity: Nit
* **Reviewer Lens:** `egm-style`
* **Affected Scope:** Lines 409-413, 428, 485, 495, 511, 516, 521, 524, 593, 598, 603, 608, 621
* **Discrepancy Description**: "next" and "last 2" are not offsets; "The tests say plainly" is filler; the `BackdropTemplate` sentence reads as a contradiction (reword to "no Blizzard button or edit-box templates; call `SetBackdrop` before showing"); "about 250 to 350 lines" and "small set" are size judgements with no decision value; duplicated ADR-README restatements (items 13, 28); "when the design is finished" has no trigger (use "after Stage 9 sign-off"); "a long first name and surname" should read "24-byte"; stray space before the period at line 603; "There are no load tests" can cite ADR 0014.
* **Rationale & Recommendation**: Apply the rewordings; drop the duplicated or filler text.

### Confirmed claims (no action)
* **Reviewer Lens:** `egm-codebase-details`
* **Affected Scope:** Item 11 and related
* **Discrepancy Description**: `GetAverageItemLevel` and `player.itemLevel` are absent and correctly to be added. `UISpecialFrames`, `GetRealZoneText`, `UnitFactionGroup`, `vm.spy/count/reload/slash`, `./run-tests.sh`, the CLAUDE.md gotcha about defining globals before the `C_` wrapping, and `Interface: 16001` all check out. `scripts/install-addons.sh` is unaffected by the plan.
* **Rationale & Recommendation**: None needed.
