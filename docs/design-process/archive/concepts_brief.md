> **Archived 2026-11-21.** Its decisions are contained in ADRs, mainly 0034 and 0035. History only; the ADRs win.

# p2prdf: peer-to-peer random dungeon finder

> **Status: superseded in part by `docs/designs/forever-together.md` and `adr/`.** This brief is the brainstorm record. Where it differs from the design draft, the draft and the decision records win. The ten differences found on 2026-11-21 are listed in `reconciliation_report.md`: leases versus converging announcements, five crates versus one, two transports versus one, instance ID versus zone name only, one name field versus first name and surname, two repositories versus one, a 10,000-peer simulator versus functional tests, the Overview's dungeon and key wording, the noticeboard bootstrap versus peer exchange, and professions in the code (still an open question).

* **Author:** Hugh Kaznowski
* **Date:** 2026-11-20

## Overview

p2prdf helps World of Warcraft players find a group for a dungeon without a central server. It has two parts.

1. **Addon module (Lua).** A new module in the Social Hub addon (`rdf/`, with a page in the sidebar window). It shows a character code the player can copy with Ctrl+C. It also shows the same code as a QR (in v1), which the phone's camera app scans. The addon does no networking, no pairing, no tokens and no authentication: the QR only carries character data. The code holds: name, realm (the ruleset, see below), region, class, race, level, role and spec taken from talents, professions, a gear summary such as item level, location, and a short typo check.
2. **Desktop app (Tauri 2, Rust and React/TypeScript, separate open-source repo built from the author's template).** The player pastes the code. The app works out the dungeon from the location, which the player can change, and searches for a group on a Kademlia network under the key `dungeon + region + level`. Peers propose parties, accept or reject them, and balance roles. The app shows live progress: searchers, matches, rejections. Once the network agrees on a party, a player stops searching and rejects other proposals. When all five players press Confirm, retries stop and the app shows the party's names, so they can invite each other in game. If someone does not confirm, searching resumes.

The app also hosts the same UI over plain HTTP on the local network (critical, in v1), so a phone can open it. The phone and the desktop UI use the same API. The phone's native camera app scans the addon QR and the player pastes the text into the web app. The web app holds an approved token and presents it on every call to the desktop app over the LAN. The desktop app shows a pairing QR (LAN URL plus a token) that approves the phone, and the token can be reset in the app, which un-pairs all phones.

Decided by the author: no tokens or authentication in the addon or the QR, and none on the p2p side (a token approves a phone for the LAN API only), user theming and an overridable web app folder (environment variable, priority over embedded files) are in scope, no in-page camera, no WebRTC, no hosted web page. A forged code is a known, accepted gap, and the code is a self-reported claim.

## Analysis

**Ideal Final Result.** A player opens the addon page, copies a live code and pastes it into the app. Within minutes the app shows a role-balanced party of five with names. Everyone confirms and invites each other in game. The author runs no server, and the addon and the app share only a versioned code format. Every field is treated as a claim and not as verified.

**Main objections from the review (7 lenses)**

1. **Scale: the author expects too many players, not too few (revised).** The review assumed about 25 users. The author's figures are about 2 million players per realm per region. On Forever a realm is a ruleset (PvP, PvE, RP-PvP, RP-PvE, and soon Hardcore), and each realm is a sharded mega server in each region (NA, EU, maybe more). Only a share of players will run the app, but the author expects plenty of searchers, so the "too few per key" objection is withdrawn for exact-level keys. The risk flips to overload:
   - **Hot keys.** Popular dungeons at popular levels put thousands of peers under one key. The mainline DHT returns only a limited list of peers per lookup, and every searcher announcing to the same few nodes is load on them.
   - **Match quality.** With many candidates, the finder should pick well, not first: close item level, a fitting role mix, and low rejection rates, with a cheap way to choose among thousands.
   - **Proposal storms.** Many peers proposing at once means many rejections and wasted leases. The lease rule below needs simulation at this scale.
   - **Remaining low-traffic keys.** Odd levels and rarely run dungeons can still be sparse, and a short wait followed by adjacent levels or a wider set covers that.

   A measured adoption number is still worth having, but it now sizes the load and does not decide whether to build.
2. **A realm is a ruleset on a shared mega server, and the key uses it (corrected by the author).** The review and the first draft of this brief wrongly treated Forever realms like classic realm names, which separate small worlds. On Forever, everyone on the same realm can play with each other, even though the server is sharded inside, and players on different realms cannot group. So "realm" in the key means exactly one of the rulesets (PvP, PvE, RP-PvP, RP-PvE, Hardcore) in one region, and there is no further realm identity or realm grouping to resolve. The author does not want a hard-coded list of them, so the addon supplies the realm as an opaque value and the app only compares values. The API export lists candidates that could give it, none verified in the real client: `C_GameRules.GetActiveGameMode` / `GetCurrentGameModeRecordID` / `GetCurrentGameModeDisplayInfo`, `GetForeverExperiencePreset`, `IsHardcoreActive`, plus `GetCurrentRegion` for the region. `GetNativeRealmID` and `GetNormalizedRealmName` may be a secondary check, since they may describe the shard and not the ruleset. The key is `region + realm + faction + dungeon + level` (parties are same-faction only; the dungeon table's faction column is only a proximity hint). The CLAUDE.md note in this repo about "other realms" describes the older model and does not apply here.
3. **The code format needs an owner and a version.** The Lua encoder and the Rust decoder live in two repos. Needed: a short written spec with a version byte, shared test vectors in both repos, a CI check that they match, and a clear "update addon" or "update app" message on an unknown version. The DHT key and all messages need a version too, for example `p2prdf/1/...`.
4. **No coordinator, but no player in two parties.** A player may be listed under several level keys, so two parties can claim them at once. The review's answer is below.
5. **"Dungeon from location" does not fit a random finder.** Players queue from cities, not from dungeon entrances. The dungeon should be a set the player confirms in the app, with location only as the default.
6. **The mainline DHT carries no data.** It returns only IP and port, anyone can announce under any key, and peers' IP addresses are visible to strangers. NAT traversal is the hidden infrastructure cost: raw UDP or QUIC fails for a large share of home connections, so relays are needed in practice.
7. **The phone path is a longer clipboard.** Scanning a QR to paste its text into a phone page is more steps than Ctrl+C on the PC. It also adds an HTTP server and attack surface. (Superseded: the author made the phone and LAN web app critical, so the QR now has a consumer and is in v1.)
8. **The checksum is a typo check.** It does not prove anything about the character. Name it that way in code and UI, and show all fields as self-reported. A pasted code also ages, so it needs a timestamp and the app should show when it is stale.
9. **Open LAN API.** The author chose a per-install token that approves a phone (revised from the earlier no-token decision). The token stops other devices on the network and web pages from calling the API. Remaining mitigations: a visible on/off switch showing the server state, bind to localhost unless LAN mode is on, check the Host and Origin headers, send no CORS headers, random port, size and rate limits, and never return the raw character code. The token travels over plain HTTP on the LAN, so anyone who can sniff the network could read it. Say so in the app and treat it as an accepted risk on untrusted networks.
10. **QR drawing.** Squares drawn with `SetColorTexture` can leave gaps at non-integer UI scales. Use whole-pixel module sizes and a quiet zone. The headless test VM cannot show how it looks or whether a phone scans it, so those are manual checks.

**Contradictions and the resolutions the review suggests**

| Contradiction | Resolution |
|---|---|
| Informative code vs compact QR and short text | Fixed-width binary record in base45 text, using numeric IDs and not strings. About 48 bytes, so a small QR. |
| One-way addon data vs fresh location | Redraw the code on events and include a timestamp and counter. The app accepts only newer codes. |
| Exact level keys vs hot keys (revised for the author's scale) | Keep exact-level keys. If a key is too hot, shard it, for example with a role suffix or a small hash-derived shard number, and let peers look at neighbouring shards. |
| Deterministic group decision vs no coordinator | Per-peer leases and a total order, below. |
| Role from talents vs hybrids | The player picks the role, with the talent-derived role as the default. |

**Group decision rule (the review's recommendation).** A proposal is `(proposerId, epoch, members, roles, dungeon)`. The proposer is the lowest `hash(epoch, peerId)` among compatible candidates. Each peer holds at most one lease (about 15 to 20 seconds) and accepts a new proposal only if it holds no lease, or the new one ranks higher in a global order, which prevents cycles. The proposer commits only when everyone accepts. Any reject or timeout releases all leases. Because the lease belongs to the peer and not to a key, a player cannot join two parties. After commit, a human Confirm step follows. A non-confirmer is dropped, the others search again, and the dropper gets a local backoff. Role balance: 1 tank, 1 healer, 3 damage dealers, relaxed after about 60 seconds, with players able to mark "can also tank or heal". Search times out after 5 to 10 minutes. The simulator should tune the timeouts. Messages: PROPOSE, ACCEPT, REJECT, CONFIRM, CANCEL. States: Idle, Searching, Proposed, Invited, Committed, Confirmed, Failed.

**Search that loosens over time (author's decision).** The author expects too many players most of the time, but a key can still run dry: an odd level, a rarely run dungeon, the matches run out, or this peer's corner of the gossip network is thin. So a search starts strict and relaxes as time passes:
- **What loosens, in this order:** (1) item level tolerance and role mix (already in the group rule); (2) level: the player's own level, then plus or minus 1, then 2, and so on, up to a limit and never outside the dungeon's level range; (3) more shards of the same key; (4) more dungeons from the player's confirmed set.
- **Two kinds of loosening, which need different handling.** *Matching constraints* (level spread, item level, role) change who may be in a party. *Network reach* (joining more topics, reading more noticeboard slots, asking for more peers) changes who the peer can see. If a swarm is small or silent, widen reach before widening constraints. Joining the adjacent-level topics does both, which is why it comes third and not first.
- **Cost.** Each extra key is another topic to join, with its own bootstrap, so widening is paced (for example one step every 30 to 60 seconds), capped, and the extra topics are left again when the search ends.
- **Rule for proposals.** Constraints differ per peer and change with time, so a proposal is valid only if every member accepts it under their own current constraints. A party also has a maximum level spread and respects the dungeon's minimum level. Without a shared schedule, a very wide peer could keep proposing parties that strict peers reject. Every peer therefore follows the same default widening schedule, and the player's limits (maximum spread, "do not widen") can only make it stricter.
- **Visible to the player.** The app shows the current constraints ("levels 19 to 21, item level within 15%") and the next step and when it comes, and lets the player pause widening or set limits.
- **To test.** The simulator needs peers on different schedules, thin keys, and a network where most peers sit in one corner, and should measure time to match under widening against strict-only.

**Suggested structure**
- Rust crates: `p2prdf-code` (decode, no I/O), `p2prdf-core` (a pure state machine: event in, actions out), `p2prdf-net` (network trait plus an in-memory fake), `p2prdf-lan` (optional HTTP), and a thin `src-tauri` shell. The network sits behind a trait so the DHT choice can change.
- One frontend with two transports (Tauri calls and HTTP), with TypeScript types generated from the Rust definitions.
- Addon files: `rdf/Core.lua`, `Codec.lua`, `Qr.lua` and `ui/PageRdf.lua`. Codec and QR stay free of game API calls so they can be tested in plain Lua. The app owns the dungeon table. The addon sends raw instance IDs.
- Addon integration points found in this repo: `NS.RDF` in `Init.lua`, a TOC entry before `ui\Window.lua`, a page entry in the hard-coded `GROUPS` table in `ui/Window.lua`, a slash shim, and a multiline or resettable box for the long code because `W.EditBox` is single-line. Missing VM fakes (`GetInstanceInfo`, skill lines, region from setup) go in `prelude.lua` before the `C_` wrapping block. No new saved variables in v1.

**Client facts to verify.** The review found these calls in the API file or in installed addons: `GetCurrentRegion`, `UnitFullName`, `C_SpecializationInfo.GetSpecialization*`, `GetInstanceInfo`, `GetProfessions` and `GetSkillLineInfo`. `GetAverageItemLevel` is listed but installed addons treat it as retail-only, so the gear number may need an 18-slot loop with item data that loads asynchronously. No QR library is installed, so the encoder is about 250 to 350 lines of Lua. None of this has been checked in the real client.

**Network direction (proposed by the author, with notes from the Kademlia book in `../kademlia-book/`).** The author does not want to host anything. The idea: use a public Kademlia DHT only to find the first peers, then let the app's own peers exchange peer lists and gossip search info, with each peer keeping a bounded random sample of candidates (about 50 to 100) and matching inside it. The book supports this:
- **Bootstrap sources (ch. 8.2).** Hard-coded public bootstrap hosts (mainline: `router.bittorrent.com` and others; IPFS: `/dnsaddr/bootstrap.libp2p.io`), a cached routing table from the last run, and peers telling each other their contact info. The book warns that bootstrap nodes are a weak point, so use several sources and prefer the cache on restart.
- **Unreachable peers (ch. 8.7, 11.3, 14.5).** Amino splits peers into servers (publicly reachable, in routing tables) and clients (only issue queries), decided by AutoNAT. Most home users would be clients. Hole punching and relays (Circuit Relay v2 with DCUtR) cover direct connections. Peers with open ports can act as relays.
- **Hot keys (ch. 14.4, Coral).** "Sloppy" storage stops publishing at nodes that are full and stores slightly further out, to avoid hot spots. This is the same problem as the author's busy dungeon keys.
- **Same pattern in production (ch. 11.4, 12).** Ethereum's consensus layer finds peers through discv5 and then gossips. Node records advertise which gossip subnets a node serves. This is the model to copy: the DHT locates peers, gossip carries the traffic.
- **Security (ch. 10.9).** Treat the DHT as a hint, verify what comes back, and keep fallbacks. This matches the accepted forgery gap.
- **Correction to the review.** The review said the mainline DHT carries no data. BEP 44 allows up to 1000 bytes of signed data per publisher, but each item is keyed by the publisher's key and cannot hold a shared list under one dungeon key. For discovery, `get_peers` (IP and port only) or Amino provider records are still the way, so the point stands for this use.
- **Provider records (ch. 11.3).** They sit on the 20 closest peers, expire after 48 hours and are republished about every 22 hours, so a search announcement needs a refresh timer much shorter than that.
- **Not covered by the book:** gossipsub and peer-sampling protocols. Those details need the spike. The book is described by the author as "somewhat fact based", and the numbers above have not been checked against the protocol specs.

## Goals vs. Non-Goals

* **In-Scope Goals:**
  - Addon module that shows a copyable character code, with a version byte and a typo check.
  - Desktop app that takes the code and finds role-balanced 5-player parties on a Kademlia-based network, with live progress, confirm and name reveal.
  - A written code-format spec with shared test vectors.
  - A seeded multi-peer simulator and property tests for the group rule.
  - Realm in the search key.
  - A dungeon picker defaulted from location.
* **Out-of-Scope Non-Goals:**
  - Proof that a character is real (forgery is a known gap).
  - Tokens or authentication on the addon, the QR code or the p2p side (decided by the author).
  - Raids and groups larger than 5.
  - Full gear lists.
  - Automatic party invites in game.
  - Hosted web page, WebRTC and in-page camera.
* **Candidates to defer (author to decide):** adjacent-level registration beyond the time-based widening, and professions and gear detail. The QR and the phone and LAN web app are in v1 (decided by the author).

## Alternatives Weighed

- **Browser-only single HTML file with libp2p or iroh.** Rejected by author: browsers cannot open UDP sockets or accept inbound connections and need relays. The author moved to a native app.
- **Signed QR.** Rejected by author: the addon is open-source Lua, so any signing key is public. A typo check replaces it.
- **Reading the addon's `SavedVariables` file.** Rejected: WoW writes it only on `/reload` or logout, so data is stale.
- **Hosted HTTPS page at `wow.lloopback.com` using the phone camera, with WebRTC or LAN calls.** Rejected by author: a static host cannot relay the handshake, and an HTTPS page cannot call an `http://` LAN address. The author chose a local, self-hosted app.
- **Desktop app capturing the QR from the game window.** Not chosen. Copy and paste makes it unneeded.
- **Public signaling brokers (Trystero-style).** Not chosen by author: dependence on third parties and extra machinery.
- **Network options still open:**
  - Mainline DHT plus iroh. No servers to run, but it carries no data, is open to key poisoning and exposes IP addresses, and it relies on relays.
  - libp2p Kademlia with a few bootstrap nodes you run. Provider records and NAT tools fit, at the cost of a small operator duty.
  - Gossip topics per dungeon and realm, or a tiny rendezvous server. Simpler at about 25 users, but not peer-to-peer.

## Key Decisions & Staging for Next Steps

**Resolved**
- Addon shows information only. No pairing and no networking.
- The desktop app does the searching. The phone and LAN web app is critical and in v1. It shares one API with the desktop UI and uses a per-install token to approve a phone. The addon and QR carry no tokens.
- Forgery is a known gap and every field is self-reported.
- Level is exact per key, and the player may register adjacent levels (subject to the band question below).
- Players form the party in game from the revealed names.
- **Character names (author's input):** names are unique across all Forever realms, so a name identifies one character. The design does not depend on that, because the code is self-reported and anyone can claim any name. Consequences:
  - A peer is identified by its iroh key, never by a name. The name is a display label and the thing you type into the in-game invite.
  - Do not reject a code because another peer already claims the same name. That would let a forger lock a real player out. Show duplicates as a warning, and a party where two members claim the same name is refused by the members.
  - At the name reveal, label names as "claimed by the other player". The in-game invite is the real check: an unknown name fails and the wrong player can decline. The remaining abuse is someone causing strangers to invite a victim's name, which is a harassment risk for the named player and a reason for per-peer rate limits on proposals.
  - The code may also carry the character's GUID as a consistency check. It is forgeable like every other field.
- **Networking stack (author's decision):** iroh for connections and iroh-gossip for the swarm per key, with a public Kademlia DHT used only to find first contacts. No hosted infrastructure beyond one optional seed run by the author. iroh resolves a known peer ID to addresses but has no search by topic, and `iroh-gossip` joins a topic from at least one known peer ID (its HyParView keeps about 5 active and 30 passive peers). Finding that first peer ID is the open design problem: options are cached peers from the last run, (dropped later: a DHT noticeboard of topic-derived slots), libp2p provider records, and the hard-coded seed.
- **TLS:** rustls. (iroh's QUIC is built on it, as far as I know. Verify that no OpenSSL dependency is pulled in.)
- **Local storage:** the `varaq` crate (embedded transactional key-value store, version 0.1.1 on crates.io, young). It holds the node identity, the cached peers and settings.
- **Revised project shape (author's decision, supersedes the Social Hub module and the player widening controls).** This is a separate project that touches no existing addon. The WoW addon (standalone, `P2PRDF`), the Rust app and the web app all go into one new repository, and this repository is only where the context came from. Widening is part of the matching algorithm and is not controlled through the API. Wherever this brief says the addon is a Social Hub module, or that the player can pause or limit widening, this decision overrides it.
- **Revised topic (author's decision, supersedes level in the key).** Topics no longer encode level. A topic is `region + realm + faction + dungeon`, a broadcast of one activity for one set of matching conditions. Level is carried in each player's ad and is a matching condition (maximum level spread, widened over time). This removes the adjacent-level topic step, makes rare dungeons rare in a deterministic way, and cuts the topic count to about 400 (5 realms, 2 regions, 2 factions, 20 available dungeons). Hot topics are sharded by peer ID, ads are sent on change plus a slow keep-alive, and samples favour similar-level peers. Wherever this brief says the key includes level, this decision overrides it.
- **Revised bootstrap design (author's decision, supersedes the noticeboard).** There is no DHT noticeboard. The DHT is used only through iroh's address lookup to reach a list of trusted peers whose public keys are built into the app (the first is run by the author; more can be added). Order: remembered peers, then peer exchange with connected peers (each peer keeps a known-peers table with the topics each peer claims, as an LRU cache of configurable size, default 100, stored locally), then the trusted peers only as a last resort. The bootstrap rules below about not overloading the seed apply to the trusted peers.
- **Bootstrap order (author's decision: do not overload the seed).** On start, a peer tries these in order and stops at the first that gives it a working connection to the topic's swarm: (1) the peers remembered from earlier runs, tried a few at a time, not all at once; (2) the DHT noticeboard for the topic; (3) the hard-coded seed, only when every remembered peer is unreachable and the noticeboard returned nothing usable. Rules that follow from this:
  - The seed is never contacted while any step above still works.
  - A peer that does use the seed asks for a handful of peers, saves them to the cache, and disconnects, so the seed does not hold long-lived connections or join the gossip mesh as a hub.
  - Retries against the seed use exponential backoff with random jitter and a per-install rate limit. The seed itself caps connections and request rates and drops the excess.
  - Cached peers are kept fresh (refreshed while running, aged out when unreachable) so that fallbacks to the seed stay rare.
  - Success measure: the fraction of app starts that touch the seed, which should be small. The spike checks this number.
- **Permanent keys:** each install generates its iroh secret key once and stores it. A fresh key is created only if the stored one is missing. The author generates the seed node's key, keeps the private part private, and gives the public key (EndpointId), which is hard-coded in the app as the last-resort entry in the known-hosts bootstrap list (see the bootstrap order below). The private key is never committed. Trade-off to record: a permanent key lets other peers recognise the same player across sessions and ties that identity to their IP address.

**Proposed answers to the earlier open questions (Claude's recommendations; the author may overrule any of them)**
1. **Network:** iroh plus iroh-gossip, cached peers first, then peer exchange, with trusted peers as the last resort only (the noticeboard was dropped). The network sits behind a trait in `p2prdf-net`. A spike confirms it (see Staging).
2. **Realm:** the addon sends the region and the realm (the ruleset) from the real client, and the app treats both as opaque strings. Before the format freezes, someone logs the candidate API calls on the real client and records what they return, in particular which call tells PvP, PvE, RP-PvP, RP-PvE and Hardcore apart. If the game-mode calls do not, try the realm calls and the preset call. No fallback of asking the player is planned, since a wrong realm would put someone in a pool that cannot invite them.
3. **Level model:** exact level per key to start, hot keys sharded, and the search widens to adjacent levels over time (see "Search that loosens over time").
3a. **Matching at scale:** pick candidates by closeness of item level, then role fit, then low rejection history. Shard a key into swarms of a few hundred peers. (Revised by the author: tests cover normal behaviour only, with no performance or large-scale simulation.)
4. **v1 scope (revised by the author):** the addon shows the copyable text code and the QR. The desktop app and the phone and LAN web app are both in v1 and share one API. Later: adjacent-level registration beyond the time-based widening, and profession and gear detail. Keep in the code: name, realm, region, class, level, role and spec, item level if cheap, instance ID, a version byte and a typo check.
5. **LAN API (revised by the author):** in v1. A per-install token approves each phone through a pairing QR shown by the desktop app. The token can be reset. Visible on/off switch, bound to localhost unless LAN mode is on, Host and Origin checks, no CORS headers, size and rate limits, and the raw code is never returned. The token is sent over plain HTTP on the LAN, which is an accepted risk.
6. **Dungeon source (decided by the author):** a hard-coded dungeon table in the app (level range, minimum level, faction, zone, status). Proximity and level give a recommendation, and the player picks or searches the recommendations. Location is a preference, not a check. The author's table lives in the design draft's open questions and needs map IDs and a client check.
7. **Hardcore:** just another realm in the key. Nothing special is needed. Players on a Hardcore realm cannot group with other realms, so their pool is separate by construction.
8. **Adoption kill criterion:** dropped as a go or no-go gate, since the author expects many users. Keep it as a load measurement: number of concurrent searchers per key, time to a match, and rejection rate.

**Suggested staging:** (1) code spec plus addon text code and QR, tested in the VM (the QR look and a real phone scan are manual checks); (2) group rule in Rust with a fake network and functional tests; (3) real network behind the trait; (4) Tauri UI and the shared API with the LAN web app and token pairing; (5) measure load (searchers per key, time to match, rejections); (6) later items. A spike comes first, before step 3: BEP 44 noticeboard slots from Rust, iroh-gossip with a few hundred peers per topic, varaq with the identity and peer cache, and the realm-identity calls on the real client.
