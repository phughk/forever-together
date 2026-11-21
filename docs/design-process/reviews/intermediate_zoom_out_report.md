# Goldfish Review Report

**Target:** `/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons/docs/designs/p2prdf.md` (lines 1-627)

Stage 8, intermediate zoom out. The reviewer read the Stage 5, 6 and 7 reports first and focused on what later stages added or mitigated.

## Reviewer Status

| Reviewer | Status |
| --- | --- |
| `egm-zoom-out` | completed |

Coverage note: the reviewer read the full target, the persona and standard, the three earlier reports, `concepts_brief.md`, `CLAUDE.md`, `adr/README.md` and ADRs 0009, 0010, 0011, 0014, 0019, 0020 and 0021, with narrow greps of `wowluavm/src` and the API export. It did not open ADRs 0001-0008, 0012, 0013, 0015-0018 or 0022, so judgements on those rest on the README table and the design's own summaries.

## Verdict

Context sufficiency passes and the root problem is the right one. Every Stage 6 and 7 finding the reviewer spot-checked is still present at the cited lines. The only visible repair is a partial one at line 372. The Stage 5 findings still hold, and Stages 6 and 7 add evidence on the same central risk. The second half of the document is still at the wrong altitude.

## Prioritized Findings (Blockers -> Major -> Nits)

### The central agreement claim is contradictory and unproven - Severity: Blocker
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 37, 169-171, 193-209, 69-70, 81, 205-209, 426; ADRs 0010 and 0014
* **Discrepancy Description**: Four problems sit on the one mechanism the design depends on.
  * Goal contradicts behaviour. Line 37 says "a player is never in two committed parties". Line 209 admits the guarantee holds only at the Confirm step, and that a peer can be in two commits. The Goal was not corrected (Stage 6 Blocker 2, Stage 7 Major).
  * Stage semantics are undefined. Line 195 says the schedule is shared and the stage travels in the ad. Line 203 judges compatibility under "their ad's own constraints". The document never says whose stage applies when two peers are at different stages (Stage 6 Blocker 1).
  * Samples differ by design. Line 171 makes level-biased eviction part of the design, so samples differ by construction. Line 81 still lists that bias as an open question about convergence. A commit needs five independent computations to agree exactly (Stage 5 Major, Stage 7 Major).
  * The time source is unspecified. Epochs, the ad cutoff and expiry assume a shared clock, with no skew tolerance or time source (Stage 7 Major, Stage 5 inventive-catalyst).
* **Rationale & Recommendation**: This is a root-cause issue. If the mechanism fails, the rest of the design is wasted, and ADR 0014 forbids large-scale simulation, so nothing in the plan can reveal the failure before release.
  * Reword the Goal at line 37 to "a peer confirms at most one party", and state the split-brain outcome in section 6.
  * Define one stage rule, for example "judge a candidate by the stricter of the two stages", plus a clock-skew tolerance.
  * Make spike step S0 an explicit go/no-go gate with a stated kill criterion, such as commit rate at N peers and sample size S, measured on divergence-modelled samples.
  * The pure `engine` makes a small seeded algorithm simulation cheap. ADR 0014 rejected a 10,000-peer simulator. Ask the author whether that also excludes a small divergence-model check. If it does, record the commit rate as an accepted unknown.
  * Mark line 171 "proposed, see Open Questions".

### The problem premise is unverified and written as a first-person drafting note - Severity: Blocker
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 21-27, 250-342, line 23
* **Discrepancy Description**: Line 23 still says "This paragraph is an assumption of the draft ... I have not checked". The API export lists `GetLFGInfoServer` and an LFGList system. No Alternative covers an in-game channel or native finder (Stage 7 Blocker, Stage 5 Nit). The Stage 5 one-hour check was not acted on.
* **Rationale & Recommendation**: A finder that already exists in the client would change the problem. Do the check and state the result in two sentences. Add one Alternative for the client's own finder or addon-message rendezvous, with a concrete rejection reason. Remove the "I" note or move it to Open Questions with an owner.

### Altitude: about 260 lines of file-level detail duplicate planned specs - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 370-623, especially 391-457 and 478-488
* **Discrepancy Description**: The byte-offset table, defaults table, API table and function signatures sit in the main design, though plan item 1 (`docs/rdf-code-spec.md`) is meant to hold that content. The duplication has already drifted:
  * the epoch is "a few seconds" at line 199 and 10 s at line 433;
  * the keep-alive is 30-60 s at line 193 and 45 s at line 433;
  * the dungeon ordering at line 165 differs from line 417, and line 34 says "proximity" where line 165 says no distance calculation;
  * the widening order has four forms: line 35 (four stages), CUJ 3 at line 102 (five), line 195, and the stage table at line 441 (six stages, with level spread already in stage 0).
* **Rationale & Recommendation**: This breaks the anti-weeds standard, and best-guess defaults appear as precise numbers in four places. Keep the flow, interfaces and data flow in the design. Move the byte table, defaults and per-file listings into the spec or an implementation plan and link to them. Keep one authoritative copy of the widening order, dungeon ordering and timings.

### Shard semantics, topic count and sharding speculation are unspecified - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 17, 169, 195, 419, 441, 71, 13; ADR 0004
* **Discrepancy Description**: Line 169 gives a peer one shard, derived from its peer ID. Line 195 and stage 4 widen to "other shards", but the document never says how a peer joins them or how their ads reach its sample (Stage 6 Major 5). The "about 400 topics, fixed and known" claim needs a realm enumeration, yet ADR 0004 says no realm list is hard-coded (Stage 7). The shard table is "part of the protocol version" while its counts are open. Sharding by peer ID spreads scarce roles (tank, healer) evenly across shards (Stage 7).
* **Rationale & Recommendation**: This is speculative scale machinery on an unproven agreement rule, and it splits swarms, which worsens the Blocker. Ship v1 with a single shard per topic and keep the shard field in the topic ID. Move the shard table, its keying and role-aware sharding to Future Improvements. Call 400 an estimate.

### "No hosted service" versus the trusted-peer reality and the quiet-topic path - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 19, 39, 108, 179-189, 346
* **Discrepancy Description**: The author's always-on trusted peer is the first-start path (CUJ 4, line 108). A 100-entry LRU shared across about 400 topics evicts quiet-topic peers first, so trusted peers become the primary path for exactly the quiet topics. That contradicts "most starts never contact a trusted peer" (line 189) and "load follows cold starts, not users" (line 185), and it reads against the "no hosted service" Goal (Stage 5, Stage 7). The section 10 risk list omits trusted-peer single point of failure, compromise and key rotation.
* **Rationale & Recommendation**: The story over-claims independence. State the soft dependency on volunteer or trusted peers in Goals and Non-Goals. Define behaviour when none is reachable. Decide the eviction rule now, for example reserved slots for active topics. Add the peer-exchange-versus-trusted-peers-only question to the spike's exit criteria. ADR 0009 is Accepted, so frame this as "prove the peer-exchange layer earns its cost" rather than reversing it.

### Scope versus delivery risk: phone, LAN, override, signing, bespoke QR encoder - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 15, 42-43, 212-221, 346; plan items 5, 25, 26
* **Discrepancy Description**: Stage 5 recommended deferring the LAN server, token and override folder. The brief and ADR 0012 record these as author decisions ("critical, in v1"), so the recommendation stands only as a sequencing risk. Later additions have grown the surface: signed gossip (ADR 0019, Proposed), SSE over `fetch` streaming, the git submodule and VM changes, and a new Lua QR encoder of 250-350 lines with matrix vectors.
* **Rationale & Recommendation**: None of this is an architectural dead end, but the whole stack must work before any party can form. Add a "v1 cut line" paragraph defining a minimal vertical slice (code paste, desktop window, engine and net, Confirm) that is releasable without phone pairing and the override, and put those in the slice after it.

### Client assumptions: the spike gates S1 but its checklist is incomplete - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 63-77, 378, 392-413, 612-614
* **Discrepancy Description**: The diagram makes S0 a prerequisite of S1, which partly mitigates Stage 5 Major 3. S0's label lists only "realm and region calls". Not listed as gating: the name-call behaviour (ADR 0022), item level, the faction question (`AreCrossFactionGroupQueuesAllowed` exists in the API export, so ADR 0007 and the topic key remain exposed), and QR scannability. Line 413 still contains "from my memory" inside normative text.
* **Rationale & Recommendation**: The topic ID format and code layout freeze at S1, and a wrong answer there is a protocol version change. List every open client question in S0's gate. Resolve the cross-faction question before the topic format freezes. Replace the "from my memory" QR capacity claim with a pointer to the test vectors.

### Reuse and repository story is still inconsistent - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 13, 240, 372, 513-516; ADRs 0020 and 0021
* **Discrepancy Description**: Line 240 still says the design "does not touch" `wow-forever-addons`, "which only supplied context". Line 372 partly corrected this ("only change is ... item 11") but also says the addons stay "unchanged", and sections 1-9 never mention the submodule. All three prior stages flagged this. In `wowluavm/src/prelude.lua` line 154, `GetCurrentRegion` already exists and returns 3, so item 11's "add `GetCurrentRegion`" modifies an existing stub. `GetAverageItemLevel`, `GetSpecializationRole` and `vm.read_json` are absent. ADR 0021 also lists `GetInstanceInfo` among the VM additions, and item 11 does not.
* **Rationale & Recommendation**: This is a cheap factual contradiction in the high-level story that survived three stages. Rewrite line 240 to match line 372, name the submodule dependency in section 9 and in the Goals test bullet, and reconcile the VM addition list with ADR 0021.

### Risks and mitigations are thin and one claim is inverted - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 242-248, 346-348, 245
* **Discrepancy Description**: Section 10 has five bullets. Missing are key rotation defeating the per-peer backoff (free identities let a griefer cost four players a full 60 s each), grinding for rank in `hash(epoch, peerId)`, name harassment (ADR 0016), young dependencies (`iroh-gossip`, `varaq`), trusted-peer compromise and key rotation, and network fragmentation when tables or versions change. Line 245 says a small table "gives little room for a single peer to dominate it", which is backwards: a small table is easier to dominate.
* **Rationale & Recommendation**: The accepted-risk list is part of the high-level story and is incomplete. Expand section 10 into a short risks table with an owner or an accepted marker per row, and fix line 245. A machine-level Ready check before the human Confirm is a cheap mitigation to weigh.

### Gossip authentication appears in the plan but not the architecture - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 183, 193-209, 346, 421; ADR 0019
* **Discrepancy Description**: The section 6 commit rule needs a known author per message, but signing appears only in "Shared definitions" and the plan, marked "to be confirmed in the spike". Line 346 still says the network "has no authentication". ADR 0019 is Proposed. No fallback is stated if the spike fails. Messages do not bind a topic ID, so signed ads can be replayed into another swarm (Stage 7 Nit).
* **Rationale & Recommendation**: Signing is a precondition of agreement and belongs in the architecture. Add it to Components and section 6. Reword line 346 to "no authentication of content; messages are signed for authorship". State the fallback and add the topic ID to the signed bytes.

### Nits - Severity: Nit
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 5, 15, 17, 23, 56, 65, 185, 189, 354, 413, 491, 603
* **Discrepancy Description**:
  * The line 5 date (2026-11-20) predates ADR 0022 (2026-11-21), which line 65 cites. The "I" at lines 23 and 413 has no defined speaker.
  * Line 15 says "statically hosted web app" while the app is embedded in the binary and a hosted page is a Non-Goal (line 56).
  * Line 17 is one paragraph of about 190 words mixing rules, topics, table contents and scale. The "two million" figure has no source or assumption label.
  * The trusted peer and permanent key are described twice in section 4 (lines 185 and 189). Forgery and token risk are repeated in five places.
  * `adr/`, `concepts_brief.md` and `../kademlia-book/` links do not resolve from `docs/designs/`, and line 354 uses an absolute path. ADRs 0018 and 0019 are Proposed while the code format reads as settled.
  * "A few at a time", "some slack" and "several neighbours" have no value or pointer. Line 603 has a stray space ("the schedule .").
  * Line 491 states `UnitName` "checked against `GetUnitName`" as fact while line 65 treats it as unverified.
* **Rationale & Recommendation**: Minor consistency and clarity fixes. Keep one authoritative statement per repeated fact, fix the links, and replace vague quantities with values or pointers.

## Earlier zoom-out findings: status

| Stage 5 finding | Status |
| --- | --- |
| Core agreement unproven | Holds and is stronger (Blocker 1) |
| Speculative layers (shards, widening, level-biased eviction) | Holds (shard Major) |
| Client side unverified | Partly mitigated by the S0 to S1 dependency (client assumptions Major) |
| LAN, token and override attack surface | Holds as a sequencing risk, since the author decided it (scope Major) |
| Reuse position inconsistent | Holds (reuse Major) |
| Bespoke peer exchange | Holds, but ADR 0009 is Accepted, so frame it as proof-of-need (trusted-peer Major) |
| Trusted peer versus no hosted service | Holds (trusted-peer Major) |
| Faction assumption | Holds; the cross-faction call exists in the API export (client assumptions Major) |
| Problem premise unconfirmed | Holds, now Blocker per Stage 7 (Blocker 2) |
| Code-level detail in the design | Holds and is worse (altitude Major) |
