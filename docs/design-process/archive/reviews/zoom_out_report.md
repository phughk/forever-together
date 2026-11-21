# Goldfish Review Report

**Target:** `/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons/docs/designs/p2prdf.md` (lines 1-626)

## Reviewer Status

| Reviewer | Status |
| --- | --- |
| `egm-zoom-out` | completed |
| `egm-inventive-catalyst` | completed |

Severity totals (after cross-lens grouping, no Blockers): 0 Blocker, 14 Major, 4 Nit.
Zoom-out: 7 Major, 3 Nit (the reviewer's findings 9 to 11 are Nits). Inventive-catalyst: 6 Major, 2 Nit.
Note: the two zoom-out findings 1 and the inventive-catalyst finding 1 overlap on convergence risk; both are kept in their own lens section.

Coverage notes: zoom-out read the design, standard, persona, CLAUDE.md, adr/README.md and ADRs 0010, 0011, 0016, 0017, 0019, 0020; it did not read the other ADRs, concepts_brief.md or the API export. Inventive-catalyst read the design, ADRs 0009, 0010, 0019 and the README; it did not read the other ADRs, concepts_brief.md, CLAUDE.md or the API export.

---

# Section 1: Zoom-out lens (`egm-zoom-out`)

**Verdict (reviewer):** the design solves a real problem and the decisions are mostly well recorded. It is heavy for a v1, and several parts add speculative machinery before the central assumption (converging announcements on gossip with partial views) is proven. Context sufficiency passes.

## Prioritized Findings (Blockers -> Major -> Nits)

### Core agreement mechanism unproven but foundational - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 196-210, 69, 206
* **Discrepancy Description**: The epoch-based "same function over the same sample" scheme converges only if samples overlap heavily. The document admits this, calls the first parameter values "best guesses", and tests cover behaviour but not scale. Peers hold about 5 active and 30 passive connections and sample about 100 ads, yet topics are expected to hold thousands of players. Samples will differ systematically (biased toward the peer's own level, different ad cutoffs), so commits on crowded topics may be near zero.
* **Rationale & Recommendation**: Make the spike (step 0) a go/no-go gate for this mechanism, including a simulation of divergent samples. Alternatively evaluate a simpler fallback now (deterministic per-shard leader by hash, or proposer-with-lease). ADR 0010 rejected those as "not chosen by author" without convergence evidence, and ADR 0014 (functional tests only) leaves no way to find out.

### Speculative layers that could be cut or deferred - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 17, 168, 194, 440
* **Discrepancy Description**: Peer-ID-hash sharding, a shard table fixed in the protocol version, and widening into "other shards" assume a scale of two million players per realm. The shard counts are listed as an open question needing real traffic numbers. Sharding splits swarms and so worsens sample overlap; widening to other shards requires joining new swarms.
* **Rationale & Recommendation**: Ship v1 with one shard per topic and reserve the shard field in the topic ID. Defer level-biased sample eviction and the six-stage widening table, whose numbers have no stated basis.

### WoW client side unverified but planned at code level - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 63-77, 392-412, 490
* **Discrepancy Description**: Realm, region, name format, `GetAverageItemLevel`, `UnitFactionGroup` and QR scanning from a game window are open questions, yet the code format already fixes byte offsets, a `region` byte, an opaque realm string, an item-level fallback and a QR capacity estimate "from my memory" (line 412).
* **Rationale & Recommendation**: Keep the format in the planned `docs/rdf-code-spec.md`; keep only principles in the design. Make step 1 explicitly gated on the client spike results.

### LAN web server, token pairing and web-dir override add heavy attack surface - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 15, 212-220, 331-335
* **Discrepancy Description**: A phone can already scan the QR and paste text. The design adds an HTTP server on `0.0.0.0`, a bearer token over plain HTTP, Host/Origin checks, loopback-only routes, SSE over `fetch`, pairing QR, token reset and an executable web-content override folder that no stated goal requires.
* **Rationale & Recommendation**: For v1 serve loopback only with the Tauri window as sole client; defer LAN mode, pairing and override. The API boundary is clean so the phone can be added later. If the phone stays, drop `P2PRDF_WEB_DIR`.

### Reuse position on the existing repository inconsistent - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 239, 371, 511-515, 520
* **Discrepancy Description**: The text says the project "does not touch" `wow-forever-addons`, but item 11 modifies `wowluavm/src/prelude.lua` and `main.rs`, and the new repository takes a git submodule on it (ADRs 0020, 0021). A grep showed `prelude.lua` has one hit across `GetCurrentRegion`, `GetAverageItemLevel` and `GetSpecializationRole`, so the claim that all three are missing needs a check. No QR encoder exists in the repository, so a Lua one is genuinely new. The addon window plan re-creates the `SetColorTexture`/1px backdrop conventions from CLAUDE.md.
* **Rationale & Recommendation**: Add a short "dependency on wow-forever-addons" paragraph (submodule as test infrastructure, pins the VM version) and say the UI conventions are copied, not shared.

### Network layer: bespoke peer exchange - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 177-188, 232, 574-577
* **Discrepancy Description**: Peer exchange brings its own ALPN, hello and request messages, rate limits, topic-claim caps and an LRU table. The document does not show why remembered peers plus trusted peers (Alternative I, rejected on speculative load) is insufficient. A quiet-topic peer is "found only if a peer you ask already knows it" (lines 66, 186).
* **Rationale & Recommendation**: Start with remembered peers plus trusted peers returning peers per topic; add the exchange protocol if cold-start load appears. If kept, check whether `iroh-gossip` neighbours and iroh address lookup already cover part of it.

### Trusted-peer role conflicts with "no hosted service" - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 19, 39, 184, 346
* **Discrepancy Description**: The author's always-on trusted peer is in effect a bootstrap service for every cold start, while the goals say no hosted service.
* **Rationale & Recommendation**: State the soft dependency on volunteer bootstrap peers in Goals/Non-Goals and define behaviour when none is reachable (say so, show remembered-peer state, do not fail silently).

### Alliance/Horde assumption could invalidate the topic design - Severity: Major
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 17, 58, 73
* **Discrepancy Description**: Topics include faction because players can only group with their own faction, but the document notes the client has `AreCrossFactionGroupQueuesAllowed` and that this is unverified. If cross-faction grouping exists, the topic key is wrong and topics double for nothing.
* **Rationale & Recommendation**: Resolve in the spike before the topic ID format freezes. At minimum record faction as an ad field as well, so removing it is a protocol-version change, not a redesign.

### Problem statement rests on an unconfirmed assumption - Severity: Nit
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** line 23
* **Discrepancy Description**: The paragraph on what the client offers today is an admitted assumption; group-finder functions such as `GetLFGInfoServer` were not checked. If a usable finder exists the premise changes.
* **Rationale & Recommendation**: Do the one-hour check and replace the parenthetical with a fact.

### Code-level detail belongs in the spec or ADRs - Severity: Nit
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 414-441, 443-456, 458-607
* **Discrepancy Description**: Shared definitions, defaults table, API table and 27 file-by-file items are very long; the stage table numbers and item-level percentages are settings; items name `Codec.lua` functions, pooled textures and Storybook atoms. The ADR README marks ADRs 0018 and 0019 "Proposed", yet the design treats code format and signing as settled.
* **Rationale & Recommendation**: Keep architecture, interfaces and data flow; move the rest to the spec and plan, marked as a work list. State the proposed status of 0018/0019 or have the author accept them.

### Smaller inconsistencies - Severity: Nit
* **Reviewer Lens:** `egm-zoom-out`
* **Affected Scope:** lines 183-188, 64, 97-101, 35, item 9
* **Discrepancy Description**: The trusted peer and permanent key are described twice (183-188). Line 64 and ADR 0016 differ on whether the character GUID is carried as a consistency check, while the code format has no GUID field. The CUJ stage order (97-101) differs from the stage table and the Goals order (line 35). Item 9 says `UnitName` is "checked against `GetUnitName`" while the Open Questions still treat the full-name behaviour as unverified. (Line 168's "about 400" topics checks out: 5 x 2 x 2 x 20.)
* **Rationale & Recommendation**: Align these; treat the name-call behaviour as unverified everywhere.

**Sound (reviewer):** level kept out of the topic, one crate with modules, pure state machine with fake network, forgery accepted as a documented limit, versioned protocol refusing unknown versions, standalone addon with no saved variables.

---

# Section 2: Inventive-catalyst lens (`egm-inventive-catalyst`)

**Summary (reviewer):** false compromises detected; six Major, two Nit. Judgements on unread ADRs rest on the document's own summaries.

## Prioritized Findings (Blockers -> Major -> Nits)

### Exact-match agreement across diverging samples - Severity: Major
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** lines 196-208, 166-170, 69, 80, 572
* **Discrepancy Description**: A party commits only if all five members independently compute the identical member list from their own 100-ad samples. The sample policy (each peer drops ads furthest from its own level) makes samples differ by construction; the greedy walk is non-local, so one differing ad can change every later assignment; commit and Confirm each need all five. In a crowded topic commit probability falls roughly as the per-peer agreement chance to the fifth power. The central contradiction is "no coordinator" versus "deterministic agreement", resolved by hoping samples overlap.
* **Rationale & Recommendation**: Combine Segmentation with Inversion. Option A: priority arbitration. Each epoch the peer with the lowest `hash(epoch, peerId)` in its own sample acts as anchor, builds one party around itself by a deterministic key and announces it; members validate against their own ads instead of recomputing; a peer in several valid announcements joins the lowest anchor rank; Confirm resolves duplicates. Option B: segment the topic into hash-keyed cells of about 20 to 50 peers so peers see nearly the same set. The pure `engine` makes either cheap to simulate. Liabilities: more overlapping proposals and wasted commits; members must verify forgeable ads; partial overlap with the ADR 0010 / Alternative J rejection of an elected coordinator (author should decide whether a stateless, rotating, untrusted anchor is different enough to revisit it).

### Ghost or non-confirming peers: 60 s human timeout, key rotation defeats backoff - Severity: Major
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** lines 59, 208, 97-101, 420-425, 241-247
* **Discrepancy Description**: Free identity plus a per-peer-ID backoff means anyone can mint keys and publish valid signed ads for fake or offline players. Such parties never confirm and cost four real humans 60 s each; the backoff resets on key rotation. Honest churn (AFK, closed app) has the same effect. The only liveness check is a human.
* **Rationale & Recommendation**: Add an automatic machine-level Ready round after commit (direct iroh dial or signed gossip ack within about 3 to 5 s, no user action); prompt the human only if all five arrive. Optionally push the prompt to the phone web app, and weigh ads by liveness or a small hashcash. Liabilities: earlier IP exposure for direct dials (gossip-only acks are weaker), hashcash cost, NAT failures wrongly excluding real peers (needs a fallback), added happy-path latency.

### Role-blind sharding starves the scarce roles - Severity: Major
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** lines 166-168, 418, 194, 440
* **Discrepancy Description**: A peer's shard is `SHA-256(peerId)[0] mod shardCount`, spreading tanks and healers evenly so each shard holds about 1/k of the scarce roles, the binding constraint for a 1-1-3 party. Players wait through stages 0 to 3 (240 s) before seeing other shards' tanks. The shard count is static and shipped with the protocol version.
* **Rationale & Recommendation**: Shard only the abundant role (damage) by peer ID; tanks and healers subscribe to every shard or a per-topic unsharded scarce swarm. Let depth follow a gossiped swarm-size estimate (prefix-bit splitting), keeping the shipped table as a floor. Liabilities: more subscriptions per scarce-role peer (cap them), dynamic depth reintroduces a bounded convergence problem, role is self-reported and forgeable.

### "No topic-keyed lookup" accepted as a law; first-start discovery mostly vacuous - Severity: Major
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** lines 172-188, 186, 180, 295-299, 66-67
* **Discrepancy Description**: On first start both remembered peers and neighbours are empty, so every new install goes to the trusted peers; launch-day load equals the user count, contrary to "follows cold starts, not users". The trusted peer's answer is limited to its own LRU. Alternative H was dismissed for needing trusted peers to learn who is in every topic, yet the topic already travels in `Hello` and in the request. The 100-entry LRU shared across about 400 topics evicts quiet-topic peers quickly.
* **Rationale & Recommendation**: Combine Intermediary with Cheap Dispensables. Treat the topic in `Hello` and requests as a signed TTL registration in a bounded soft-state directory on trusted or volunteer peers; add topic-pinned slots or per-topic sub-tables in the LRU; ship or fetch a signed static seed list of volunteer peers. Liabilities: directory peers learn who searches for what, registrations are Sybil-floodable (per-source quotas, dial-back verification), pinned slots reduce LRU diversity.

### Epoch agreement depends on wall-clock time without skew tolerance - Severity: Major
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** lines 198-204, 200, 422, 433
* **Discrepancy Description**: Epochs, the ad cutoff, expiry and widening stages all assume a shared clock. Home desktops often skew by seconds; on a 10 s epoch this shifts epoch numbering and cutoffs so peers see different ads, compounding the convergence problem. The document does not say how announcements for epoch n+-1 or attacker-supplied `createdAt` are handled.
* **Rationale & Recommendation**: Apply Inversion and Self-Service. Use a logical epoch derived from gossip, or accept announcements within +-1 epoch and re-derive membership from ad content hashes; estimate skew as the median offset from neighbours' signed timestamps; bound the cutoff window. Liabilities: skew estimates can be manipulated (use median with outlier rejection), harder debugging, n+-1 acceptance raises double-announcement cases that the "at most one confirmed" rule must absorb.

### "Functional tests only" leaves the core risk untested although the engine is pure - Severity: Major
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** line 45, ADR 0014, lines 615-622, 68-69
* **Discrepancy Description**: The design defers all tuning to the real network, but a new app with few users cannot generate the traffic needed, so the first release is both experiment and product. The engine is I/O-free, so a seeded in-process simulation is cheap and is an algorithm check, not a load test.
* **Rationale & Recommendation**: Add a deterministic simulation harness over the pure `engine` (a few thousand synthetic ads, controllable divergence model: drop probability, level bias, clock skew; Monte-Carlo commit rate, time-to-party, waste) to set the first epoch, cutoff and sample-size defaults. Amend ADR 0014 to state this is within its intent. Liabilities: the divergence model is itself a guess (calibrate against the spike's measured overlap), extra test code, and iroh-gossip scale is still uncovered, so the spike must test it.

### Token travels in clear on the LAN - Severity: Nit
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** lines 51, 219-220, 345
* **Discrepancy Description**: A bearer token is sent in cleartext on every request, framed as a trade between simple pairing and secrecy; sending a secret versus proving possession is not a real trade-off.
* **Rationale & Recommendation**: Put a random per-phone secret in the pairing QR and send only a per-request HMAC over method, path, timestamp and nonce. `crypto.subtle` is unavailable on plain `http://<lan-ip>`, so a small pure-JS HMAC/SHA-256 is needed. Liabilities: server nonce/timestamp state and clock skew, response bodies still readable, hand-rolled JS crypto may cost more than the risk it removes. (Largely moot if the zoom-out recommendation to drop LAN mode in v1 is taken.)

### Gossip signatures do not bind to topic or epoch - Severity: Nit
* **Reviewer Lens:** `egm-inventive-catalyst`
* **Affected Scope:** lines 420-425, ADR 0019
* **Discrepancy Description**: `Ad` has no topic or epoch in its signed payload, and one key can sit in several swarms (multi-dungeon search, widening), so a valid ad can be replayed into another swarm or shard.
* **Rationale & Recommendation**: Include topic ID, protocol version and a freshness field in the signed bytes (domain separation). It can be bound implicitly in the preimage without transmitting it, if the receiver knows the topic from swarm context.

**Balanced items (reviewer):** DHT used only for trusted-peer address lookup with remembered peers first; human Confirm as final arbiter (finding 2 only adds a cheap liveness check before it); same-faction topics with level in ads, not topics (ADR 0005); one-crate decision and shared test vectors.
