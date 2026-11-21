# Consolidated Review Report

**Target:** `docs/designs/p2prdf.md` (627 lines; FeatureRange 1-369, DetailedRange 370-627)
**Date:** 2026-11-21
**Stages run:** 5 (initial zoom out), 6 (comprehension), 7 (critic), 8 (intermediate zoom out), 9 (implementation readiness). Run as one uninterrupted pass.
**Nothing has been changed in the design document or the ADRs.** Every item below is a proposed task. Items that would reverse an Accepted ADR are listed under "Decisions only the author can make".

Stage reports: `zoom_out_report.md`, `comprehension_report.md`, `critic_report.md`, `intermediate_zoom_out_report.md`, `implementation_readiness_report.md`.

## Verdict

The problem is real, and the recorded decisions are mostly sound. The document is not ready for implementation. Three things hold it back:

1. **The central mechanism is unproven and its own text contradicts it.** Peers must independently compute the same party from different samples of ads, and nothing in the plan can show that works (Blocker B1).
2. **The first paragraph of the Problem section is an unchecked assumption, left as a first-person note** (B2). The client's API export lists group-finder functions that nobody has looked at.
3. **The plan lacks the contracts an implementer needs** (B3 to B8): phase gates, spike outcomes, the engine and network interfaces, committed data tables, and correct facts about the existing test VM.

## Reviewer status

| Stage | Reviewer | Status |
| --- | --- | --- |
| 5 | `egm-zoom-out`, `egm-inventive-catalyst` | completed |
| 6 | `egm-comprehension` | completed |
| 7 | `egm-design`, `egm-structural`, `egm-style` | completed |
| 8 | `egm-zoom-out` | completed |
| 9 | `egm-implementation-readiness`, `egm-codebase-details`, `egm-style` | completed |

No reviewer is "not run".

## Counts

Consolidated action items: **8 Blocker, 24 Major, 5 Nit** (37 in all), built from 102 raw findings across the five reports (Stage 5: 8 + 3 zoom-out and 6 + 2 inventive-catalyst; Stage 6: 18; Stage 7: 27; Stage 8: 11; Stage 9: 27 plus one "confirmed claims" entry with no action).

The headers of the stage reports count by their own method and differ from their bodies in three places (for example Stage 5 says 7 major zoom-out findings but lists 8; Stage 7 says 17 major and 15 nit but lists 20 major and 6 nit entries). This report counts entries as listed in the report bodies.

## Coverage caveats and unverified claims

*   **Not fully read.** The Stage 5 reviewers skipped most ADRs, `concepts_brief.md` and the API export. The Stage 8 reviewer did not open ADRs 0001-0008, 0012, 0013, 0015-0018 and 0022. The Stage 7 structural reviewer did not read all of `concepts_brief.md`, and the style reviewer did not open the ADRs. The Stage 9 reviewer could not check claims about the template, because the template is not in this repository.
*   **From memory, unverified.** The Stage 7 design reviewer recalls that iroh's defaults use third-party relays and DNS or pkarr servers. Nobody has verified this. It matters for the "DHT only" and "no hosted service" wording (M5).
*   **Checked against the code.** The Stage 9 codebase reviewer confirmed that `GetCurrentRegion` already exists in `wowluavm/src/prelude.lua` (line 154, returns 3), that `GetAverageItemLevel` and `player.itemLevel` are absent, and that `GetSpecializationRole` is a global in the API export and not part of `C_SpecializationInfo`.
*   **Found by me while consolidating.** `adr/0021` still lists `GetInstanceInfo` among the VM additions, although item 11 in the design dropped it when the dungeon field was removed (see M12).

## Decisions only the author can make

Several reviewer suggestions would reverse or reopen an Accepted ADR. They are listed here as questions and are not turned into tasks until you decide.

| # | Question | Raised by | Related ADR |
| --- | --- | --- | --- |
| D1 | Does "functional tests only" allow a small, seeded simulation of the agreement rule on the pure `engine` (a few thousand synthetic ads, modelled sample divergence)? It is an algorithm check, not a load test. If not, record the commit rate as an accepted unknown. | Stage 5 (inventive-catalyst), Stage 7, Stage 8 | 0014 |
| D2 | Evaluate an anchor-based or segmented agreement rule (one rotating, untrusted anchor builds the party; or topics split into cells of 20 to 50 peers) next to converging announcements? | Stage 5, Stage 7, Stage 8 | 0010 |
| D3 | Define a v1 cut line: ship a vertical slice (paste code, desktop window, engine, network, Confirm) first, and put the phone pairing, LAN server and web-folder override in a later slice? | Stage 5, Stage 8 | 0012 |
| D4 | Ship v1 with one shard per topic and move shard tables and role-aware sharding to Future Improvements? | Stage 5, Stage 8 | 0005, 0004 |
| D5 | Prove that the peer-exchange layer earns its cost against "remembered peers plus trusted peers" before building it (add it to the spike's exit criteria)? | Stage 5, Stage 8 | 0009 |
| D6 | Add an automatic machine-level "Ready" round after a commit (signed acknowledgement within 3 to 5 seconds) before asking the human to Confirm? | Stage 5 (inventive-catalyst), Stage 8 | 0010 |
| D7 | Change the project, addon and protocol names (`p2prdf`) to match the repository name (`forever-together`)? Already an Open Question in the design. | Your message during Stage 5 | 0020 |
| D8 | Treat the Proposed ADRs (0018 code format, 0019 signed gossip, and the details in 0012) as settled in the design, or accept them and update their status? | Stages 5, 7, 8 | 0012, 0018, 0019 |
| D9 | Move the implementation-level detail (byte table, defaults, API table, per-file lists, about 260 lines) out of the design into `docs/rdf-code-spec.md` and a separate plan, leaving the design at the level of flow, interfaces and data flow? | Stage 5, Stage 8 | n/a |

## 🔴 Blockers

### B1. The agreement mechanism is contradictory and unproven
*   **Problem.** A party commits only when five peers independently compute the same member list from their own samples. The design makes samples differ on purpose (each peer drops ads furthest from its own level), so agreement can fail on busy topics. Nothing in the plan can reveal this before release. The text also contradicts itself about the guarantee, the stage rule and the clock.
*   **Tasks.**
    1.  Reword the Goal on line 37 to "a peer confirms at most one party", state the split-brain outcome in section 6 (what a late peer does), say whether a committed peer keeps announcing and when `Decline` is sent, and use one vocabulary (announcement, commit, confirm) instead of proposal, invitation, retry.
    2.  Define whose stage applies when two peers are at different stages (for example, the stricter of the two), and how "relaxed after a wait" relates to the stage table.
    3.  State the time source, the tolerated clock skew, whether receivers use their own arrival time, how announcements for epoch n+1 or n-1 are handled, and the ad expiry value.
    4.  Mark level-biased sample eviction (line 171) as proposed, pointing to Open Questions, because line 81 still asks whether it hurts convergence.
    5.  Make spike S0 a go or no-go gate for this mechanism, with a stated kill criterion (for example a commit rate at N peers and sample size S, measured on samples with modelled divergence).
    6.  Decisions D1 and D2 apply here.
*   **Source findings:** Z5-1, Z5-IC1, Z5-IC5, Z5-IC6, C6-1, C6-2, C7-3, C7-4, C7-12, Z8-1.

### B2. The problem premise is unverified and the in-game alternative was never evaluated
*   **Problem.** Line 23 says the paragraph on how groups are found today is an assumption and that the author has not checked the client's own group-finder. The API export lists `GetLFGInfoServer` and an `LFGList` system. No alternative covers an in-game solution (native finder, a custom channel, or addon messages as a rendezvous). `CLAUDE.md` of the source repository records that public chat sends are restricted outside hardware events, which could be a reason to reject the in-game route, but the design never argues it.
*   **Tasks.**
    1.  Check what the client offers (the group-finder functions and any addon-message route) and state the result in two sentences.
    2.  Add an alternative for the in-game route, with a concrete rejection reason or a go or no-go spike.
    3.  Remove the first-person "I have not checked" note, or move it to Open Questions with an owner. Keep the Problem section to the problem, without the unprovable "slow" and "works worst" claims.
*   **Source findings:** C7-1, Z8-2, Z5-9.

### B3. The plan has no phase gates and its phases do not map to its items
*   **Problem.** The order-of-work diagram has steps 0 to 7. The 28 numbered items are grouped only as Part A and Part B. No step has an exit criterion. Item 15 adds every dependency up front, and the order of `engine` (item 22) against `net` (item 23), which consumes its types, is not stated.
*   **Tasks.** Map each item to a step. Give each step an exit command (for example "`cargo test code dungeons engine store` passes"). State which items must compile together.
*   **Source findings:** R9-1.

### B4. The spike gates the plan, but it is not a work item and has no outcome branches
*   **Problem.** Several items depend on spike results that are unknown: the realm and region calls, whether gossip identifies message authors (ADR 0019), whether `varaq` suits the secret key, and whether address lookup resolves a trusted peer from its key. Items say "if the spike shows..." with no alternative design. An implementer cannot write `Collect()` without knowing the realm call, and the realm may not fit in 24 bytes. The spike label lists only the realm and region calls, not the name calls, item level, cross-faction grouping or QR scannability.
*   **Tasks.** Make the spike its own work item with required outputs (the call used for each field, `varaq` yes or no, gossip author identity yes or no, address lookup yes or no, name calls, item level, cross-faction, a phone scan of a drawn QR), a fallback per outcome, and the spec lines each result would change. Mark the steps after it as blocked until it finishes. Name an owner.
*   **Source findings:** R9-2, Z8-7, C6-9, C6-12.

### B5. The addon's data collection (item 6) is not specified enough to code
*   **Problem.** Sources mix confirmed and unconfirmed calls. Undecided: role derivation from spec; which `C_SpecializationInfo` functions to call; how the item-level fallback retries without breaking the repository's "never poll" rule; how unknown values are encoded for class, race, level and region; what zone and realm hold when empty; the order of clamping and truncation; and how the secondary role (flag bits 2 and 3) is filled. The sentence also states "never a realm" about the surname return as fact while the name calls are unverified.
*   **Tasks.** Replace the paragraph with a per-field table (source call, fallback, unknown value). State the secondary-role rule (for example "always 0 in v1"). Mark `UnitName` and `GetUnitName` behaviour as assumed until the client check confirms it.
*   **Source findings:** R9-3, Z5-11 (name wording).

### B6. The engine, gossip and party logic is prose and not a contract
*   **Problem.** Undefined: the compatibility check (item level plus or minus 15% of whose value, symmetry, unknown item level, how level spread is measured, the dungeon's minimum level); the greedy partition's tie-breaks and how secondary roles fill gaps; whether a peer builds only its own party or the whole partition; the epoch time source and ad cutoff (no Defaults row); how `partyId` is serialized and which topic is hashed; the hash used in `hash(epoch, peerId)`; what happens when members agree but their dungeons differ; how shards are joined; and the `Network` trait, the event and action types and the error model.
*   **Tasks.** Add an engine contract section with Rust type definitions, pseudocode for the compatibility check and the greedy walk, and a worked example with expected output that a test can assert. State that the network module verifies signatures, so the engine only receives verified messages.
*   **Source findings:** R9-4, R9-26 (signature placement).

### B7. Required data inputs are not committed
*   **Problem.** Item 21 says to embed "the 29 entries from the author's table", which is neither a file path nor committed content, so the test that expects 29 entries and 20 available cannot be met. The shard table is not a listed file and `shardCount` has no value. `trusted.rs` is empty with no way to test with a key.
*   **Tasks.** Commit the dungeon table (or pin a file and revision). Define the shard table format (decision D4 would make every count 1 in v1) and a `shardCount` row in the Defaults table. Specify a mechanism for test trusted-peer keys.
*   **Source findings:** R9-5, Z8-4.

### B8. The plan's claims about the existing test VM are wrong or ambiguous
*   **Problem.** `function G.GetCurrentRegion() return 3 end` already exists at `wowluavm/src/prelude.lua:154`, above the line that defines `world`, so the plan's "add `GetCurrentRegion` reading `world.region`" is a modification and a new definition there would read nil. The VM sets `world = setup`, so `world.region` is ambiguous (a top-level `region` key, or a nested `world` object that the prelude never reads). Realm and surname keys are unnamed, and no VM function returns the realm, so the core tests cannot be written.
*   **Tasks.** Say "modify the existing `G.GetCurrentRegion`" with the exact body (for example `return setup.region or 3`), keeping the default 3 so the Social Hub suites are unchanged. Give the exact setup JSON, the Lua expression that reads each key, and the VM function that returns the realm.
*   **Source findings:** R9-6, R9-7.

## 🟡 Majors

### M1. Gossip authentication belongs in the architecture, with a fallback
Section 6 commits a party when announcements from all five members are seen, which needs a known author per message. Signing appears only in the shared definitions and `adr/0019` (Proposed), and line 346 still says the network "has no authentication". Add signing to the components list and section 6, reword line 346 ("no authentication of content; messages are signed for authorship"), state the fallback if the spike shows gossip already identifies authors or fails, include the topic ID and protocol version in the signed bytes so an ad cannot be replayed into another swarm, and add "trust the forwarding neighbour" as an alternative. Sources: C7-2, Z8-10, Z5-IC8.

### M2. No load estimate for crowded topics
Every peer announces each epoch and re-broadcasts ads every 30 to 60 seconds to swarms of thousands, across several topics and shards. Add a back-of-envelope estimate of messages and bytes per second for a 3,000-peer shard, state where announcements and confirmations go (whole swarm or the proposed members only), and reword the "low broadcast load" claim as intent. Sources: C7-5.

### M3. Sharding is speculative and its rules are unspecified
Sharding by peer ID splits swarms (which worsens sample overlap), spreads tanks and healers evenly so each shard is starved of scarce roles, and widening "to other shards" has no rule for joining them or merging their ads. The "about 400 topics, fixed and known" figure needs a realm list the design says it will not hard-code (ADR 0004). Tasks: call 400 an estimate; define how shard counts are keyed and joined; reconcile with ADR 0004; decision D4 (one shard in v1) and role-aware sharding (shard only damage dealers) belong here. Sources: Z5-2, Z5-IC3, Z8-4, C6-5, C7-15, R9-16.

### M4. Abuse resistance: free identities, ghost peers and name harassment
A peer ID costs nothing, so a per-key backoff does nothing against someone who rotates keys, and fake or absent peers can make four real players wait a full confirm timeout each. A key can be ground to rank first in `hash(epoch, peerId)`. Self-reported names let someone cause strangers to invite a victim. Forged ads can pollute samples, and "drop ads furthest from my level" can be gamed. State what bounds the damage (a per-IP or per-epoch limit, or an accepted risk) and fix line 245, which says a small known-peers table limits domination when the opposite is true. Decision D6 (machine-level Ready round) applies. Sources: C7-6, Z5-IC2, Z8-9.

### M5. Risks and Mitigations section, trusted-peer dependency and hosted components
The standard asks for a Risks and Mitigations section. The design has five bullets under "Failure and risk summary", several without mitigations. Missing: the abuse risks in M4, trusted-peer single point of failure, compromise and key rotation (keys are built in, so rotation needs a release), young dependencies (`iroh-gossip`, `varaq`), IP exposure, and fragmentation when tables or versions change. The author's always-on trusted peer is a hosted dependency that sits against the "no hosted service" Goal, and iroh may use third-party relays and DNS servers by default (unverified). Tasks: add a top-level risks table (failure, impact, mitigation, owner or accepted); verify and name iroh's default hosted components; state the soft dependency on trusted peers in Goals and Non-Goals; define behaviour when none is reachable; define a trigger and test for the `varaq` fallback. Sources: C7-7, C7-8, Z5-7, Z8-5, Z8-9.

### M6. Quiet topics depend on the weakest discovery path
Peer exchange reaches a quiet topic only if a neighbour already knows someone in it, a 100-entry table shared across about 400 topics evicts quiet-topic peers first, and on first start both remembered peers and neighbours are empty, so every new install goes to the trusted peers. That contradicts "load follows cold starts, not users" and "most starts never contact a trusted peer". Tasks: specify the eviction rule now (for example reserved slots for active topics); estimate trusted-peer load for first start and quiet topics; consider treating the topic in `Hello` and in requests as a signed, expiring registration on trusted peers; add "peer exchange versus trusted-only" to the spike exit criteria (decision D5). Sources: C7-9, Z5-IC4, Z5-6, Z8-5, C6-7.

### M7. Peer exchange and bootstrap protocol details are incomplete
No framing, size limit, serialization for exchange messages, timeouts, or response to a missing `Hello`. It is unclear whether `Hello` is its own stream given "one request and one response per stream". Rate-limit enforcement, how long an ignored peer stays ignored, and where that is stored are unspecified. "The first that works", "a few at a time" and the neighbour timeout have no definition. The `known_peers/<peerId>` record format, the trusted peers' address and relay sources, and cold-start behaviour (a peer with no remembered peers has no neighbours to ask) are not described. Sources: R9-18, C6-7.

### M8. Protocol and table versioning can split the network
The shard-count and dungeon tables are part of the protocol version, and different versions use different topic names, so each table change (new dungeons, tuned shard counts) splits the population until everyone updates. Decouple data tables from the wire version, or support an N and N-1 overlap, or define a table-update path. Sources: C7-10.

### M9. No success metrics or critical-mass analysis
No target for time to match or match rate, nothing on the empty-room problem (about 400 topics plus shards across a small early user base), and the human Confirm has no stated notification path or success estimate. Add measurable goals and an adoption risk, and give the confirm timeout and notification mechanism values or mark them as tuning. Sources: C7-11.

### M10. The code format depends on unknown values and several details are not pinned
*   Realm and region have unknown source and type. If they arrive localized, a non-English client lands in a different topic from an English client on the same ruleset. Define a canonical numeric ID or normalization, and state how two peers are sure to produce the same topic string.
*   State a size goal in bytes or QR version instead of "scans reliably". Define "flags". Add role derivation from talents to Open Questions. Reword "numeric IDs keep localized text out of the app", because zone names and character names are text.
*   Names are needed before commit, but no message carries them and the ad has no name field. ADR 0016 (Accepted) refuses parties with a duplicate first name and surname and allows a GUID check. Specify which message carries names, when, and where the check runs. Settle the GUID question.
*   Add a validation table (field, valid range, error variant). Pin counter wraparound at 65535, the source of the `created` time, rejection of trailing bytes, level 0, an empty first name, and the encoding of a neutral or unknown faction.
*   Pasted-text normalization strips all spaces, which corrupts valid base45 codes because the base45 alphabet contains a space. Strip only leading and trailing whitespace and line breaks, and add a test vector with an internal space.
*   Define one error vocabulary (`UnknownVersion`, `BadCheck`, `Truncated`, `BadText` and the Lua `err`).
*   Narrow the QR scope in item 5 to alphanumeric mode at level M (the design needs only that), name the reference encoder and its inputs, and give the capacity table per version, the maximum record size and what the window does when text does not fit.

Sources: C7-13, C7-14, C6-3, C6-4, Z5-3, R9-10, R9-11, R9-12, R9-20.

### M11. Client assumptions: gating list and normative text
Faction grouping (`AreCrossFactionGroupQueuesAllowed` exists in the API export), realm and region values, name calls, item level and QR scanning from a game window are open, yet the code layout, topic key and byte offsets are fixed as if they were not. Resolve cross-faction grouping before the topic format freezes, and record faction as an ad field too, so removing it from the topic is a version change and not a redesign. Remove "from my memory of the QR tables" from normative text and point to the test vectors. Sources: Z5-8, Z5-3, Z8-7, C6-12.

### M12. Repository story and file boundary are inconsistent
Line 240 says the design does not touch `wow-forever-addons` and that it "only supplied context", while items 11 and 13 modify the VM there and take it as a submodule, and line 372 says the only change is the VM additions. Sections 1 to 9 never mention the submodule. `adr/0021` lists `GetInstanceInfo` among the VM additions, but item 11 no longer does (this was dropped with the dungeon field). Item 13 moves `adr/`, the brief and the draft out of `wow-forever-addons`, which contradicts "every file touched is listed". Unlisted files: a shared message types file, `Cargo.lock`, `.gitmodules`, `.gitignore`, `vite.config.ts`, `index.html`, `.storybook/`, and individual components and stories. `mise.toml` and `test-addon` appear in two items with unclear Create or Modify marks. The install script needs a path guard. Tasks: rewrite line 240 to match line 372, name the submodule in section 9 and in the Goals test bullet, reconcile the VM addition list with ADR 0021, list destinations and deletions, and mark each path Create, Modify or Delete once. Sources: C7-16, Z5-5, Z8-8, C6-10, R9-14, R9-23.

### M13. Scope and delivery risk
The phone pairing, LAN server, web-folder override, signing, the submodule and a new Lua QR encoder all have to work before any party can form. These are author decisions (ADR 0012), so the risk is sequencing. Add a v1 cut line and a minimal vertical slice (decision D3). If the token stays in the LAN design, a per-request HMAC over a per-phone secret would avoid sending a secret in clear, at the cost of hand-written JavaScript crypto, which is probably more risk than it removes. Sources: Z5-4, Z8-6, Z5-IC7.

### M14. Altitude: implementation detail duplicated in the design, and it has drifted
About 260 lines (byte table, defaults, API table, function signatures, per-file items) sit in the design, although `docs/rdf-code-spec.md` is meant to hold the format. The copies disagree: the epoch is "a few seconds" at line 199 and 10 s at line 433; the keep-alive is 30 to 60 s at line 193 and 45 s at line 433; the dungeon ordering differs between lines 165 and 417 and line 34 says "proximity" where line 165 says there is no distance calculation; the widening order has four forms (line 35 with four stages, CUJ 3 with five, line 195, and the stage table with six and level spread already in stage 0); the slash command is `/rdf` in CUJ 1 and `/p2prdf` in Open Questions. Keep flow, interfaces and data flow in the design, move the rest to the spec or a plan, and keep one authoritative copy of each repeated fact (decision D9). Sources: Z5-10, Z8-3, C7-19, C6-6, C6-16.

### M15. The Alternatives section needs reasons, not attributions
Many entries say only "Rejected by author" or "Not chosen by author" (E, I, J, L, M). The alternative L con (running bootstrap nodes) also applies to the chosen design. "Excessive", "after discussion" and unsupported figures ("20 nodes", "48 hours", "untested at scale", "about ten times more topics", "mature and widely used", "faster incremental builds") have no source. Several entries bundle two or three options (G, H, O). Missing alternatives: signed gossip versus trusting the forwarding neighbour, base45 versus other encodings, an in-game native finder (B2). ADR citations are bare backticked paths that do not resolve from `docs/designs/`. Tasks: give each rejection a technical reason or a link, split the bundled entries, add the missing ones, use real relative links, and drop subjective phrases. Sources: C7-17, C7-18.

### M16. The local API contract is incomplete
Only `POST /api/character` has a body. Missing: the `GET /api/state` and event payloads, an error body, status codes (401 against 403), size and rate limits, the token format and comparison, the port choice, the Host allow-list, event-stream reconnection and heartbeat, LAN interface selection, whether the loopback check uses the socket's peer address, what restarts when `lan_mode` changes, how the desktop window recovers after a token reset, and how assets are served (Tauri's asset resolver or an embedding crate). Section 7 lists only two loopback-only routes while the Non-Functional Requirements and the API table name more, and the draft says "pushed with server-sent events" while a browser `EventSource` cannot set an Authorization header (ADR 0012 specifies `fetch` streaming). A single per-install token forces un-pairing every phone when one is lost, which Alternative N does not mention. Sources: C7-20, R9-19, C6-13.

### M17. Settings versus protocol constants
Line 430 says every default can be changed on the Settings page, but item 26 lists only LAN mode, token reset, known-peers size and trusted peers. If each peer can change the epoch length or confirm timeout, convergence breaks. List which keys are user settings and which are protocol constants, and align the two statements. Sources: R9-17, R9-16.

### M18. The widening stage table is ambiguous
Stage 4 does not say which or how many shards, or whether they count in the sample. Stage 5 level checks are unstated. The stage clock start, the restart after a dropped party, and what happens at the search limit are undefined. Settings changes on one peer break the shared schedule (ADR 0011). Sources: R9-16, C7-19, C6-16.

### M19. Dungeon recommendation rule gaps
No tie-break for equal distance, no place for `none` in the faction-hint order, a locale-unaware case-insensitive zone compare, an unspecified topic-list function (realm and region are opaque user values), and "shard table" and `shardCount` are undefined where they first appear. "Slack" is "some" in one place and 3 in another. Add a final sort key (the slug), define the `none` ordering, state the text folding, give the signature, and define the shard terms. Sources: R9-15, C6-6.

### M20. Details about the existing VM and the addon load order
*   `GetSpecializationRole` is a global in the API export and not part of `C_SpecializationInfo`, so with `--api` the real call is reported as unimplemented while the fake goes unused. Pick one call (the global, or `GetSpecialization` plus `GetSpecializationInfo`, whose fifth return is a role) and give the TANK, HEALER, DAMAGER to 1, 2, 3 mapping.
*   In the VM, `UnitName` returns `name, realm` with realm nil at home, so a test that reads the surname from its second return gets nil. State that `Collect` splits `GetUnitName("player")` on the first space with no VM change, or list the `UnitName` change.
*   `NS.RD` is created in `Core.lua`, but the TOC loads `Codec.lua` and `Qr.lua` first and they use `RD.Codec`. Create `NS.RD` in the first loaded file, or add an init file first.
*   `vm.read_json` has no signature, error behaviour, registration point (after `main.rs:153`, following the `vm_fn` pattern at `main.rs:107`) or working-directory assumption. The submodule step has no URL, pin or ordering guard. The new setup keys and helpers need a documentation update in `CLAUDE.md`.

Sources: R9-8, R9-9, R9-13, R9-14.

### M21. Dependencies and template are unpinned or hedged
No starting commit or instantiation method for the template. Unpinned crate versions and feature sets (`iroh`, `iroh-gossip`, `varaq`, `axum`, `tauri`), and open choices ("only if needed", "`base64` or `data-encoding` as needed", "a QR library such as `qrcode`", "`ed25519` through iroh"). A base45 crate and an OS keychain crate are missing from the list. Name and pin the crates (or say base45 is hand-written) and pin the template commit. Sources: R9-21.

### M22. The test plan lacks assertions, and "functional tests only" leaves the core risk uncovered
Expected outputs are missing (for example the order for Barrens levels 15, 25 and 38). The integration test has no instance count or scenario. Missing tests: signature rejection, `partyId` determinism, `Taken`, ad expiry, `Decline`, the search limit, LRU ordering, the pairing URL, event streaming, identity persistence, command-line flags, and Horde against Alliance exclusion. The web app has only stories and a type check. Because the engine does no I/O, a small seeded simulation is cheap, and the inventive-catalyst reviewer argues it fits ADR 0014's intent (decision D1). Sources: R9-22, Z5-IC6.

### M23. Acceptance criteria and spike deliverables are subjective or have no owner
"Stores the identity safely", "meaningful states", "looks right" and "someone logs the calls" have no observable condition or owner. State observable conditions (for example "the key file is readable only by the user and survives a restart", "decodes with two phone scanners at UI scales 0.8, 1.0 and 1.2") and name an owner and the calls to log. Sources: R9-24, R9-12.

### M24. ADR status and references
ADR 0018 (code format) and 0019 (signed gossip) are Proposed, but the design presents the base45 record and signing as settled (decision D8). The header date (2026-11-20) predates ADR 0022 (2026-11-21), which the design cites. The ADRs, `concepts_brief.md` and the kademlia notes are referenced as if the reader holds them, and the references do not resolve from `docs/designs/`. State whether the ADRs are required reading, summarize `adr/0022` where it is cited, and mark the parts that depend on Proposed records. Sources: C7-21, C7-27, C6-11, Z8-11.

## 🟢 Nits

*   **N1. Repeated statements.** Forgery as a known gap, the HTTP token risk, trusted peers, the permanent key, the known-peers table, the bootstrap order, the faction hint, "no load tests" and professions are each stated in several places, and lines 185 and 189 in section 4 nearly duplicate each other. State each once and cross-reference it. Sources: C7-23, Z5-11, Z8-11.
*   **N2. Wording and vagueness.** Line 17 is a 190-word paragraph that mixes rules, topics, table contents and scale; "sharded mega server" is jargon; "two million per realm per region" has no source; line 15 says "statically hosted web app" while the app is embedded in the binary; "a pair of small tools" against three parts; "a few at a time", "a few topics", "some slack" and "several neighbours" have no value; the document drifts between swarm, topic and key; line 603 has a stray space before a full stop; "next" and "last 2" in the field table are not offsets; "The tests say plainly" is filler; "when the design is finished" has no trigger (use "after sign-off"); "someone checks its numbers" names no role. Sources: C7-24, C7-25, C6-14, C6-15, C6-17, C6-18, R9-27.
*   **N3. Structure.** "Additional Resources" and "Future Improvements" are H3 headings under Non-Functional Requirements but are document-level sections. Open Questions has 21 untriaged bullets before the architecture, mixes blocking items (realm values, name calls) with tuning items (epoch length), and includes statements. The "Components" heading is unnumbered. The diagram omits the Tauri window and the headless trusted-peer mode, and no sequence diagram shows the epoch, commit and Confirm flow. The field list packs six fields into one bullet. Sources: C7-22, C7-26.
*   **N4. References and paths.** `adr/`, `concepts_brief.md` and `../kademlia-book/` do not resolve from `docs/designs/`; line 354 and the scope paragraph use absolute or personal paths (`/Users/.../forever-together`); "the author's template" and "the author" are vague (use the template name and "maintainer"). Sources: C7-27, R9-25, Z8-11.
*   **N5. Small inconsistencies.** Line 64 and ADR 0016 differ on whether the GUID is carried while the code has no GUID field; item 9 states `UnitName` is "checked against `GetUnitName`" while Open Questions treat it as unverified; item 7 lists "dungeon" in the window although the code has none; "number of QR textures equals the number of dark modules" should say shown textures (and whether the quiet zone counts); QR vectors store 0 and 1 rows while `Qr.Encode` returns booleans and the comparison is unspecified; the TOC flavour marker is not stated; whether jitter applies before or after the 15-minute cap is unstated; `check` against `test-addon` coverage of `cargo test` is unclear. Sources: Z5-11, C6-16, C6-18, R9-26.

## Traceability matrix

Raw findings are numbered in the order they appear in each report: Z5- (Stage 5 zoom-out, 1-11), Z5-IC (Stage 5 inventive-catalyst, 1-8), C6- (Stage 6, 1-18), C7- (Stage 7, 1-27), Z8- (Stage 8, 1-11), R9- (Stage 9, 1-27; the "confirmed claims" entry has no action).

| Consolidated item | Raw findings |
| --- | --- |
| B1 Agreement mechanism | Z5-1, Z5-IC1, Z5-IC5, Z5-IC6, C6-1, C6-2, C7-3, C7-4, C7-12, Z8-1 |
| B2 Problem premise | C7-1, Z8-2, Z5-9 |
| B3 Phase gates | R9-1 |
| B4 Spike | R9-2, Z8-7, C6-9, C6-12 |
| B5 Addon data collection | R9-3, Z5-11 |
| B6 Engine and gossip contract | R9-4, R9-26 |
| B7 Data inputs | R9-5, Z8-4 |
| B8 Existing VM facts | R9-6, R9-7 |
| M1 Gossip authentication | C7-2, Z8-10, Z5-IC8 |
| M2 Load estimate | C7-5 |
| M3 Sharding | Z5-2, Z5-IC3, Z8-4, C6-5, C7-15, R9-16 |
| M4 Abuse resistance | C7-6, Z5-IC2, Z8-9 |
| M5 Risks, trusted peer, hosted parts | C7-7, C7-8, Z5-7, Z8-5, Z8-9 |
| M6 Quiet-topic discovery | C7-9, Z5-IC4, Z5-6, Z8-5, C6-7 |
| M7 Peer exchange details | R9-18, C6-7 |
| M8 Versioning | C7-10 |
| M9 Success metrics | C7-11 |
| M10 Code format | C7-13, C7-14, C6-3, C6-4, Z5-3, R9-10, R9-11, R9-12, R9-20 |
| M11 Client assumptions | Z5-8, Z5-3, Z8-7, C6-12 |
| M12 Repository and file boundary | C7-16, Z5-5, Z8-8, C6-10, R9-14, R9-23 |
| M13 Scope and delivery | Z5-4, Z8-6, Z5-IC7 |
| M14 Altitude and drift | Z5-10, Z8-3, C7-19, C6-6, C6-16 |
| M15 Alternatives | C7-17, C7-18 |
| M16 API contract | C7-20, R9-19, C6-13 |
| M17 Settings against constants | R9-17, R9-16 |
| M18 Widening table | R9-16, C7-19, C6-16 |
| M19 Recommendation rules | R9-15, C6-6 |
| M20 VM and load-order details | R9-8, R9-9, R9-13, R9-14 |
| M21 Dependencies | R9-21 |
| M22 Test plan | R9-22, Z5-IC6 |
| M23 Acceptance criteria | R9-24, R9-12 |
| M24 ADR status and references | C7-21, C7-27, C6-11, Z8-11 |
| N1 Repeated statements | C7-23, Z5-11, Z8-11 |
| N2 Wording | C7-24, C7-25, C6-14, C6-15, C6-17, C6-18, R9-27 |
| N3 Structure | C7-22, C7-26 |
| N4 References and paths | C7-27, R9-25, Z8-11 |
| N5 Small inconsistencies | Z5-11, C6-16, C6-18, R9-26 |
