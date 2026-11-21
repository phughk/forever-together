# Goldfish Review Report

**Target:** `/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons/docs/designs/p2prdf.md` (`lines 1-369`)

## Reviewer Status

| Reviewer | Status |
| --- | --- |
| `egm-design` | completed |
| `egm-structural` | completed |
| `egm-style` | completed |

Severity breakdown after deduplication: 1 Blocker, 17 Major, 15 Nit.

Notes: the design reviewer checked the API export only by name (`GetLFGInfoServer`, `GetAverageItemLevel`, `UnitFactionGroup`, `GetCurrentRegion` and an `LFGList` system exist). No signatures or behaviour were inspected. The design reviewer's claim about iroh's default relays and DNS is from memory and is unverified. The structural reviewer did not read all of `concepts_brief.md`. The style reviewer did not open the ADRs or the API export.

## Prioritized Findings (Blockers -> Major -> Nits)

### Problem premise unverified and the in-game alternative never evaluated - Severity: Blocker
* **Reviewer Lens:** `egm-design`, `egm-structural`, `egm-style`
* **Affected Scope:** lines 21-27 (Problem), 250-342 (Alternatives)
* **Discrepancy Description**: Line 23 says the problem paragraph is an assumption and that the author has not checked the client's own group-finder functions. That first-person drafting note is left in the final text. The API export lists `GetLFGInfoServer` and an `LFGList` system, so the claim is checkable. No alternative covers an in-game solution, such as the client's LFG or LFGList APIs, or a custom channel or addon messages used as a rendezvous. The whole architecture rests on "an addon cannot talk to the network" (line 27), which holds only for the external network. `CLAUDE.md` records that public-chat sends are restricted outside hardware events on this client. That is a possible rejection reason, but the document never makes the argument. The Problem section also restates the solution (lines 25-27) and does not stand alone.
* **Rationale & Recommendation**: Verify what the client offers. Add an "in-game channel / native LFG" alternative with a concrete rejection reason, or a go/no-go spike. Remove the first-person note, or move it to Open Questions with an owner. Keep the Problem section to the problem. Remove the unprovable "slow" and "works worst at the levels..." claims.

### No authentication of gossip messages in the protocol - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-structural`
* **Affected Scope:** lines 193-209 (sections 5 and 6), line 346
* **Discrepancy Description**: Section 6 commits a party on "seen announcements from all five members", which needs a known author for each message. Neither the section nor the Components list mentions signing or author IDs. Line 346 says the network "has no authentication". ADR 0019 (Proposed) says iroh-gossip may deliver the forwarding neighbour as the sender and requires an author peer ID with an ed25519 signature. As written, anyone could forge an announcement or a confirmation. The Alternatives have no entry for signed gossip.
* **Rationale & Recommendation**: Put signed messages into section 6 and the Components list, or mark them as an explicit unresolved dependency. Reword line 346. Add the "trust the forwarding neighbour" alternative.

### Agreement rule has no evidence of convergence - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-style`
* **Affected Scope:** lines 169-171, 199-207, 70, 81
* **Discrepancy Description**: Peers hold about 100 ads out of thousands per topic, and level-biased eviction (line 171) makes each peer's sample differ by design. A commit needs five peers to compute an identical member list. The greedy walk over `hash(epoch, peerId)` is brittle, because one differing early ad changes every later party. Widening to more shards and dungeons gives party members different topic sets. Section 6 reassures "when two peers have the same sample", which is the case the design avoids. Line 171 states level-biased eviction as the design, while line 81 lists it as an open question about whether it hurts convergence. ADR 0014 tests only a few dozen peers on a fake network, which cannot expose this failure.
* **Rationale & Recommendation**: Before approval, add a small analytical or simulated model with partial views, or an early spike. State a pass or kill criterion, such as the commit rate at N peers and sample size S. Mark line 171 as proposed, with a pointer to Open Questions.

### "Never in two committed parties" contradicts section 6 - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-style`, `egm-structural`
* **Affected Scope:** lines 33, 37, 45, 205-209
* **Discrepancy Description**: Line 37 says a player is never in two committed parties. Line 209 says the guarantee holds "at the confirmation step" and that a commit can waste work for four players. Commit is defined (line 205) as seeing matching announcements from all five, so a player can be in two committed parties. There is no final-agreement step, so one peer can reveal names while another times out and resumes searching. "Retries stop when all five confirm" does not say which retries. The vocabulary also drifts: "proposals" (37), "invitations" (205), "announcements" (209), and "retries", left over from the lease design that ADR 0010 replaced.
* **Rationale & Recommendation**: Reword the goal to "a peer confirms at most one party". Specify the split-brain case, meaning who is final and what the late peer does. Use the announcement and commit vocabulary throughout.

### No load estimate for crowded topics - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-style`
* **Affected Scope:** lines 17, 169, 193, 199-205, 348
* **Discrepancy Description**: Every peer announces every epoch of a few seconds, and ads are rebroadcast every 30-60 seconds, to swarms of thousands. A player can join several topics and shards. Inbound traffic per peer grows with swarm size. The scope of announcements (whole swarm or only the proposed members) is not stated. Line 193 claims low broadcast load with no evidence, and line 348 admits there are no load tests.
* **Rationale & Recommendation**: Add a back-of-envelope estimate of messages and bytes per second per peer for a 3,000-peer shard. State where announcements and confirmations go. Reword line 193 as intent.

### Sybil, grinding and name-harassment risks unaddressed - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-structural`
* **Affected Scope:** lines 59, 183, 199-209, 245, 346
* **Discrepancy Description**: A peer ID costs nothing. The local backoff for non-confirmers (lines 59, 209) is per key, so a griefer who rotates keys makes four real players wait a full confirm timeout each time. Per-peer rate limits are meaningless when keys are free. An attacker can grind keys to rank first in `hash(epoch, peerId)`. Self-reported names let an attacker cause strangers to invite a named victim (ADR 0016). Forged ads can pollute the 100-entry sample, and "drop ads furthest from my own level" can be gamed. Line 245 says a small table leaves little room for a peer to dominate it, but a small table is easier to dominate.
* **Rationale & Recommendation**: Add these as entries in a Risks section. State what bounds the damage (a per-IP or per-epoch limit, or an accepted risk). Correct or remove line 245.

### Risks and Mitigations section missing and thin - Severity: Major
* **Reviewer Lens:** `egm-structural`, `egm-design`
* **Affected Scope:** lines 242-248, 344-349
* **Discrepancy Description**: The standard requires a Risks and Mitigations section. The document has a five-bullet "10. Failure and risk summary" inside Technical Plan, plus limits scattered across section 4, section 6, Open Questions and Non-Functional Requirements. Several bullets name a risk with no mitigation ("Unverified client assumptions", 248). Missing: the Sybil and name-harassment risks above, trusted-peer single point of failure and compromise, young dependencies (`iroh-gossip`, `varaq`, line 78 and 225), and IP exposure (only under Privacy, 347).
* **Rationale & Recommendation**: Add a top-level "Risks and Mitigations" H2 with one row per risk giving failure mode, impact and mitigation. Move section 10 and the scattered limits into it. Define a trigger and a test for the `varaq` fallback.

### Hosted dependencies and trusted-peer single point of failure not in the risks - Severity: Major
* **Reviewer Lens:** `egm-design`
* **Affected Scope:** lines 19, 39, 141-143, 179, 185, 189
* **Discrepancy Description**: The first trusted peer is run by the author, so it is a single point of failure and a recurring cost, which cuts against "no hosted service". A compromised trusted peer can feed every cold-starting peer attacker peers. Keys are built in, so rotation needs an app release, and there is no revocation story. The diagram shows relays and hole punching. The reviewer recalls that iroh's defaults use third-party relays and DNS or pkarr servers, which is unverified. If true, "DHT only" (line 19) understates a hosted dependency.
* **Rationale & Recommendation**: Verify which hosted components iroh uses by default and name them. Add trusted-peer SPOF, compromise and key rotation to the risks.

### Discovery of quiet topics depends on the weakest mechanism - Severity: Major
* **Reviewer Lens:** `egm-design`
* **Affected Scope:** lines 17, 67-68, 102, 181, 185-189
* **Discrepancy Description**: Peer exchange reaches a quiet topic only if a neighbour already knows someone in it. A 100-entry LRU table lets popular topics evict rare ones, so trusted peers become the primary path for exactly the quiet topics. That contradicts "load follows cold starts, not users" (185) and "most starts never contact a trusted peer" (189). Eviction and coverage are left open, yet the architecture depends on both.
* **Rationale & Recommendation**: Specify an eviction policy now, for example protecting active topics and keeping a minimum per topic. Estimate trusted-peer load for the quiet-topic case.

### Protocol and table versioning fragments the network - Severity: Major
* **Reviewer Lens:** `egm-design`
* **Affected Scope:** lines 75, 169, 349
* **Discrepancy Description**: The shard-count and dungeon tables are part of the protocol version, and different versions use different topic names and do not meet. Shard counts are unknown and to be tuned on the live network, and new dungeons become searchable over time. Each change splits the population until everyone updates.
* **Rationale & Recommendation**: Decouple the data tables from the wire version, or support N and N-1 overlap, or define a graceful table-update path.

### No success metrics or critical-mass analysis - Severity: Major
* **Reviewer Lens:** `egm-design`
* **Affected Scope:** whole range, lines 205-209, 330
* **Discrepancy Description**: There is no target for time-to-match or match rate, and nothing on the empty-room problem. Roughly 400 topics plus shards divide a small early user base. The human Confirm has no stated timeout, notification path or success estimate. Alternative K lists "works with very few users" as a pro, which highlights the gap.
* **Rationale & Recommendation**: Add measurable goals and an adoption risk. Give the confirm timeout and notification mechanism values, or mark them as to be tuned.

### Time source and clock skew unspecified - Severity: Major
* **Reviewer Lens:** `egm-design`
* **Affected Scope:** lines 193, 199-201
* **Discrepancy Description**: Epochs, the ad cutoff and ad expiry depend on timestamps that are self-set by ad authors and taken from desktop clocks that may be skewed. Peers near a boundary land in different epochs, and a forged timestamp can steer the cutoff. Ad expiry (193) has no duration.
* **Rationale & Recommendation**: Specify the time source, the tolerated skew, and whether receivers use their own arrival time. State the expiry value or point to the setting.

### Code format depends on unknown values and has no size budget - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-style`
* **Affected Scope:** lines 31, 62-66, 82, 150-157, 269
* **Discrepancy Description**: Realm and region are "opaque values" with unknown source and type. If they arrive as localised strings, a non-English client lands in a different topic from an English client on the same ruleset. A fixed-layout record needs a defined size, but the QR size budget is never stated. Line 31 says "small enough that the QR scans reliably", which is not measurable, and line 82 admits scannability is untested. Role derivation "from talents" (153) has no open question. "Flags" is undefined. Line 159 says numeric IDs keep localised text out of the app, but line 156 and line 72 say zone names, which are localised, reach it, and names are text.
* **Rationale & Recommendation**: State a maximum payload in bytes or QR version as the goal. Define a canonical numeric ID or normalisation for realm and region. Add role derivation to Open Questions. Reword line 159 to "class, race and faction are numeric IDs".

### Names are needed before commit but no message carries them - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-structural`
* **Affected Scope:** lines 150-157, 193, 205-211
* **Discrepancy Description**: ADR 0016 (Accepted) says to refuse parties where two members claim the same first name and surname, and allows a GUID consistency check. The ad fields in section 5 contain no name and no GUID, and section 6 does not say how names reach the other four members after Confirm, or where the duplicate-name check runs.
* **Rationale & Recommendation**: Specify which message carries names, when, and where the check runs. Settle the GUID question in section 1, or record that the rule is deferred.

### Topic set cannot be "fixed and known" while realm and region are opaque - Severity: Major
* **Reviewer Lens:** `egm-structural`, `egm-style`
* **Affected Scope:** lines 13, 17, 66, 169
* **Discrepancy Description**: Line 169 says the topic set is fixed and known from the dungeon table, about 400 topics (5 realms, 2 regions), with a shard-count table shipped in the app. Line 13 and ADR 0004 say the app hard-codes no realm list. The 400 figure and the shard table both need an enumeration the document says it will not have. The count also includes Hardcore, which line 17 calls "later". "Few topics make peer exchange work better", the `iroh-gossip` defaults of 5 and 30, and "thousands of players" are stated as facts without evidence (line 70 admits the scale is unknown).
* **Rationale & Recommendation**: Call 400 an estimate. Define how shard counts are keyed. Reconcile with ADR 0004. Mark the other figures as assumed or cite a source.

### Line 240 contradicts ADR 0021 and the plan - Severity: Major
* **Reviewer Lens:** `egm-structural`
* **Affected Scope:** line 240, goals line 45, section 9
* **Discrepancy Description**: Line 240 says the design does not touch `wow-forever-addons`, "which only supplied context". ADR 0020 and 0021 commit VM additions there and take it as a submodule at `vendor/wow-forever-addons`. Plan item 11 (line 513, outside this stage's range) confirms it. Nothing in lines 1-369 mentions the submodule. The wording "only supplied context" is also unclear.
* **Rationale & Recommendation**: Rewrite line 240 to say the only change is the VM additions. Name the submodule dependency in section 9 and in the Goals test bullet.

### Alternatives rejected on "the author chose it", with unsupported claims - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-structural`, `egm-style`
* **Affected Scope:** lines 252-342, especially 258, 264, 270, 276, 282, 288, 299, 306, 310, 312, 316, 318, 322, 324, 328, 330, 336, 342
* **Discrepancy Description**: The standard requires concrete technical or scope reasons, and a link when a rejection is attributed to a maintainer. Many entries say only "Rejected by author" or "Not chosen by author" (J, L, M, I, E). Alternative L's main con, running bootstrap nodes, applies to the chosen design as well. Alternative M says only "excessive" and N says "after discussion" with no link. ADR citations are bare backticked paths, and relative paths such as `adr/0001` do not resolve from `docs/designs/` (the correct prefix is `../../adr/`). Subjective or unsourced phrases: "Simple and fast. Easy to test." (316), "Mature and widely used." (322), "better matches" (310), "20 nodes", "48 hours", "untested at scale" (299), "About ten times more topics" (287), "Faster incremental builds" (328). Line 252 already says "by author", so the field repeats it.
* **Rationale & Recommendation**: Give each rejection a technical reason, such as why iroh-gossip over gossipsub or why no per-shard coordinator, or link the discussion. Use real relative links. Cite or calibrate each figure, and drop subjective phrases.

### Alternatives are missing or bundled - Severity: Major
* **Reviewer Lens:** `egm-structural`
* **Affected Scope:** lines 250-342 (A-O)
* **Discrepancy Description**: There is no entry for signed gossip or for base45 versus other encodings. G, H and O each bundle two or three unrelated options, so Pros, Cons and Reason apply unevenly.
* **Rationale & Recommendation**: Add short entries for the missing choices and split G, H and O.

### Widening order and slash command inconsistent between sections - Severity: Major
* **Reviewer Lens:** `egm-style`, `egm-structural`
* **Affected Scope:** lines 35, 64, 89, 102, 195
* **Discrepancy Description**: Line 35 gives four widening stages (item level and role, level spread, shards, dungeons), line 195 says "item level and role mix", and line 102 lists five (secondary roles, item level range, level spread, shards, dungeons). The slash command is `/rdf` in CUJ 1 (89) and `/p2prdf` in Open Questions (64). Plan line 501 (outside this stage's range) says both exist.
* **Rationale & Recommendation**: Use one list and one set of terms, and state both slash commands once.

### Desktop-only route lists differ between section 7 and the NFRs - Severity: Major
* **Reviewer Lens:** `egm-style`, `egm-structural`, `egm-design`
* **Affected Scope:** lines 218-220, 346
* **Discrepancy Description**: The NFR says LAN mode and trusted-peer changes are loopback-only and that request rates are limited. Section 7 lists only pairing QR data and token reset. The text also says progress is pushed by server-sent events with the token "sent with every API call". Browser `EventSource` cannot set an Authorization header, and ADR 0012 specifies `fetch` streaming for this reason. A single per-install token also means a lost phone forces un-pairing all phones (N5 from the design review), which Alternative N does not mention.
* **Rationale & Recommendation**: List all loopback-only routes and the rate limit in section 7. Say "streamed with `fetch`". Add per-device tokens to Alternative N.

### Unproven phrasing of guarantees and unlinked ADR dependencies - Severity: Major
* **Reviewer Lens:** `egm-design`, `egm-structural`
* **Affected Scope:** header, lines 148-159, 225
* **Discrepancy Description**: ADR 0018 and 0019 are still Proposed, but the base45 encoding and the binary record are presented as settled. The header Date (2026-11-20) predates ADR 0022 (2026-11-21), which the document cites.
* **Rationale & Recommendation**: Mark the Proposed-dependent parts. Update the date.

### Additional Resources, Future Improvements and Open Questions misplaced - Severity: Nit
* **Reviewer Lens:** `egm-structural`
* **Affected Scope:** lines 61-82, 351, 359, "Components" heading
* **Discrepancy Description**: "Additional Resources" and "Future Improvements" are H3s under Non-Functional Requirements but are document-level sections. Open Questions sits before the architecture it asks about, and holds 21 untriaged bullets, some of which are statements ("Shared zones", "Name matching", "Where the table lives"). Blocking items (realm values, name calls) are mixed with tuning items (epoch length, shard counts). The "Components" H3 is unnumbered while sections 1-10 are numbered.
* **Rationale & Recommendation**: Promote the two H3s to H2. Split Open Questions into blocking (with an owner or gate) and tuning. Number or retitle Components.

### Repeated statements - Severity: Nit
* **Reviewer Lens:** `egm-style`, `egm-structural`, `egm-design`
* **Affected Scope:** lines 13, 15, 17, 19, 27, 39-45, 49-54, 77, 159, 165, 175-189, 217, 221, 244-247, 346-348, 362
* **Discrepancy Description**: The same content is stated in several places: forgery as a known gap (27, 49, 159, 244, 346), the HTTP token risk (51, 221, 247, 346), trusted peers (179, 185, 189, with 185 and part of 189 nearly identical), the permanent key (41, 189, 225), the known-peers table (40, 181), bootstrap order, the override path, the faction hint, "no load tests" (45, 348), professions (54, 77, 362), and line 13 versus line 50.
* **Rationale & Recommendation**: State each once in the authoritative place and cross-reference it.

### Overview and wording nits - Severity: Nit
* **Reviewer Lens:** `egm-style`, `egm-structural`
* **Affected Scope:** lines 13-17, 23, 27, 63-64, 156, 169, 211
* **Discrepancy Description**: Line 17 is a single paragraph of about 190 words mixing rules, topics, table contents and scale. "Sharded mega server" is jargon, and "two million per realm per region" has no source. Line 15 says "statically hosted web app", which clashes with "embedded in the binary" and with the non-goal at line 56. Line 27 says "a pair of small tools" but the Overview describes three parts. `forever-together` first appears at line 64 and the desktop OS targets from ADR 0001 are not stated. Line 63 says "someone checks its numbers" without naming a role. Line 156 has a double colon. Line 211 repeats the unverified "First Surname" text from line 65.
* **Rationale & Recommendation**: Split line 17, call the figures assumptions, say "static web app", introduce the repository name and OS targets in the Overview, name the role, and rewrite line 156.

### Vague quantities and terminology drift - Severity: Nit
* **Reviewer Lens:** `egm-style`
* **Affected Scope:** lines 165, 177, 183, 187, 193, 199, 205-209
* **Discrepancy Description**: "A few at a time" (177), "a few topics" (183), "a few seconds" (199), "some slack at the edges" (165), "several neighbours" (187) and an ad expiry with no duration (193) have no value and no pointer to a setting. The document also drifts between swarm, topic and key, and between proposal, invitation and announcement.
* **Rationale & Recommendation**: Give the default or write "value TBD, see Open Questions". Fix one term for each concept.

### Architecture diagram and field list incomplete - Severity: Nit
* **Reviewer Lens:** `egm-structural`
* **Affected Scope:** lines 114-157, 232
* **Discrepancy Description**: The Mermaid diagram is valid but omits the Tauri window and the headless trusted-peer mode, and there is no sequence for the epoch, commit and Confirm flow. The field list packs six fields into one bullet, has no sizes, and gives rationale inside the "location" bullet. Line 232 puts a parenthetical about the `core` crate clash in the middle of a module bullet.
* **Rationale & Recommendation**: Add a sequence diagram. Use a table with one field per row, size and notes. Move the parenthetical out of the bullet.

### Broken or unresolvable references - Severity: Nit
* **Reviewer Lens:** `egm-structural`, `egm-style`
* **Affected Scope:** lines 353-357
* **Discrepancy Description**: `adr/`, `concepts_brief.md` and `../kademlia-book/` do not resolve from `docs/designs/`, line 354 uses an absolute macOS path, and the brief and `design_draft.md` are planned to move to the new repository.
* **Rationale & Recommendation**: Use repository-correct relative links, or state that the paths are relative to the project root and mark local notes as such.
