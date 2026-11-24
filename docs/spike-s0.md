# Spike S0 results


| Output | Result | Assumption used in the code now | Fallback if the assumption is wrong | Spec / code to change |
|--------|--------|--------------------------------|-------------------------------------|-----------------------|
| Realm (ruleset) call, its values for PvP, PvE, RP-PvP, RP-PvE, Hardcore, and whether it fits 24 bytes | **Partly verified, live capture 2026-11-21.** The realm value is the game-mode record ID as a decimal string: 14 = PvP, 15 = PvE, 16 = RP-PvE, 18 = RP-PvP. **Still unknown:** the Hardcore value (not released, not guessed). | `C_GameRules.GetCurrentGameModeRecordID()` returns a number, sent as a decimal string. If that call is missing or fails, `GetRealmName()` lowercased in ASCII is used (a realm name is not a ruleset, so this fallback can put two rulesets in one topic). Neither is confirmed to be the ruleset. | add a mapping table to canonical IDs in the spec; if no call gives the ruleset, the code is not built | `docs/rdf-code-spec.md` "Realm and region"; `addon/ForeverTogether/Core.lua` `realmValue` |
| Region call and its values | **Partly verified, live capture 2026-11-21.** `GetCurrentRegion()` works and 3 = EU. US 1, KR 2, TW 4, CN 5 are assumed from the standard numbering and not yet seen. | `GetCurrentRegion()` returns a number 0 to 255, copied as is, 0 when the call returns nothing | none, the code is not built | spec "Realm and region"; `Core.lua` `Collect` |
| `UnitName("player")` and `GetUnitName("player")` return first name and surname, and the in-game invite accepts "First Surname" | **Verified by live capture 2026-11-21:** the name call gives first name and surname as two fields. Whether the in-game invite accepts "First Surname" is not shown by that capture. | the first return of `UnitName("player")`, else of `GetUnitName("player")`, is "First Surname" and is split on its first space. The second return (realm) is never read (ADR 0022). A name with no space cannot be built. | split the combined name differently; spec change | spec "Collecting each field"; `Core.lua` `splitName` |
| `GetAverageItemLevel` works | **Verified live 2026-11-21.** A PvE character with some gear gave a valid item level (flag bit 5 set, value 20 at level 26, a low-gear character). A level 1 character with no gear gave no item level, so the flag was clear and the code carries 0, as designed. | `GetAverageItemLevel()` first return is the item level; 0 or nil means unknown | slot sum on `PLAYER_EQUIPMENT_CHANGED` (`RD.SumSlots`: mean over 17 slots, a two-hander counted in both hands; the exact formula is a guess) | `Core.lua` `sumSlots` |
| `AreCrossFactionGroupQueuesAllowed` value, and whether cross-faction grouping exists | unverified, needs live client | parties are same-faction only (ADR 0007); faction is in the topic and in the code (flag bit 4) | faction stays in the topic and also goes in the ad (already the design) | ADR 0007, design |
| A phone camera scans a QR drawn with `SetColorTexture` at UI scales 0.8, 1.0 and 1.2 | unverified, needs phone | one texture per dark module, whole screen pixels per module (the largest that fits 230 UI units, with a four-module quiet zone), black on a white square. The matrix itself is checked against two independent encoders in `docs/rdf-code-vectors.json`; only the on-screen drawing and the scan are unchecked. | change the module size or the area, or show text only | `addon/ForeverTogether/Window.lua` `QR_AREA`, `QUIET` |
| `varaq` keeps the secret key with file mode readable only by the user and survives a restart | unverified, needs live app | not used yet (app code not started) | OS keychain for the key | ADR 0008, `app/src/store/` |
| iroh-gossip identifies message authors | **No. Checked from docs/source (iroh-gossip 0.103.0 `api::Message`), not run live.** A gossip `Message` has `content`, `scope` and `delivered_from`; the docs say `delivered_from` "is not the same as the original author" and "a gossip message has no author field, and the content is not signed. If you need to know the author, or need to trust the content, sign it in your app." | gossip messages carry the author's peer ID and an ed25519 signature (ADR 0019). **Stands.** Implemented in `app/src/net/gossip.rs` (`seal` / `open`): the signature covers `topic \|\| 0x01 \|\| payload`; the message's own `peer` must equal the envelope author; a direct message has `to` inside the signed bytes. | none needed | spec section 3 (unchanged); ADR 0019 (stays accepted) |
| iroh address lookup resolves a trusted peer from its public key | **Yes by design, not run live. Checked from docs/source (iroh 1.5.0).** With the `N0` preset the endpoint publishes to and resolves from n0's DNS/pkarr service (`https://dns.iroh.link/pkarr`, and DNS under `iroh.link`), so `Endpoint::connect(public key)` finds a peer that has published. It works only if the trusted peer's app also uses the `N0` preset (or a service we share). Whether a freshly started peer is resolvable within seconds, and how it behaves when `iroh.link` is unreachable, need a live network. A loopback smoke test (`net::iroh_net`) shows connect by key works with an in-memory lookup and no internet. | trusted peers are listed by public key only; `IrohConfig::Default` uses the `N0` preset | list trusted peer addresses in the app (`EndpointAddr`), or self-host the lookup (see next row) | `app/src/net/trusted.rs`, `iroh_net.rs` |
| iroh's default hosted components (relays, DNS) are named, and whether they can be self-hosted or switched off | **Named. Checked from docs/source (iroh 1.5.0), not run live.** (1) Relays: the `N0` preset uses n0's production relays `use1-1.relay.n0.iroh.link`, `usw1-1.relay.n0.iroh.link`, `euc1-1.relay.n0.iroh.link`, `aps1-1.relay.n0.iroh.link` (home relay chosen by latency; relays forward only encrypted packets and also serve as the fallback path when hole punching fails). `RelayMode` can be `Disabled`, `Staging` or `Custom(RelayMap)`, and the `iroh-relay` crate can be self-hosted. (2) Address lookup: `PkarrPublisher`, `PkarrResolver` and `DnsAddressLookup` against `dns.iroh.link` / `iroh.link`. They can be replaced (`address_lookup(...)`, `clear_address_lookup()`), with `MemoryLookup` or a self-hosted `iroh-dns-server`. (3) Default features of `iroh`: `metrics`, `fast-apple-datapath`, `portmapper` (UPnP/NAT-PMP/PCP on the LAN), `tls-ring`. TLS is rustls with `ring`: **no OpenSSL, no native-tls, no aws-lc-rs** in `cargo tree` for this crate. Without n0's relays, peers behind a NAT that blocks hole punching cannot connect. No LAN discovery (mDNS) is enabled, so "LAN mode" would need an added lookup. | `IrohConfig::Default` = `N0` preset (n0 relays and DNS), recorded as an external dependency | self-host relay and DNS, or `RelayMode::Disabled` with direct addresses only | design risks table (add: "iroh default relays and DNS are hosted by n0"); `iroh_net.rs` |
| Seeded simulation (D1): commit rate for converging announcements and the proposer model at 3,000 peers and a sample of 100 | **Neither mechanism reached the 70% floor (converging 1.9%, proposer 12.1%). A third, leader invitations, does: mean 78.4% of eligible peers within 30 s (6 rounds of 5 s), min 74.6% over 8 seeds; 73.0% mean at 3 rounds (15 s), min 69.7%.** Simulated only, not run on a live network. | leader invitations with a human Confirm ([ADR 0025](adr/0025-leader-invitations.md)), epoch 5 s, sample tie-break by hash of owner and peer | if a live network shows lost direct messages, retransmit `Invite` and `PartyAnnounce` (simulated, not built) | engine, ADR 0025 |

## Other values the addon assumes (not S0 rows, but unverified)

| Value | Assumption | Where |
|-------|------------|-------|
| TOC interface number | `## Interface: 16001`, copied from the Social Hub addon on the same client | `ForeverTogether.toc` |
| Race ID | third return of `UnitRace("player")`; the test VM does not return it, so the VM tests see race 0 (the real client is expected to give an ID) | `Core.lua` `Collect` |
| Class ID | third return of `UnitClass("player")` | `Core.lua` `Collect` |
| Spec and role | `C_SpecializationInfo.GetSpecialization()` then `GetSpecializationInfo(index)`, whose fifth return is `TANK`, `HEALER` or `DAMAGER`. The API export has these only under `C_SpecializationInfo`. | `Core.lua` `Collect` |
| Events that mark the code stale | `PLAYER_LEVEL_UP`, `ZONE_CHANGED_NEW_AREA`, `PLAYER_ENTERING_WORLD`, `PLAYER_EQUIPMENT_CHANGED`, `PLAYER_SPECIALIZATION_CHANGED`, `ACTIVE_TALENT_GROUP_CHANGED` (all exist in the API export) | `Core.lua` |
| Ctrl+C copy | the edit box selects all on focus; the player presses Ctrl+C. Not checked in the client. | `Window.lua` |

Only the rows filled in so far are listed. The other S0 outputs (see the implementation plan, item 0) are still open.

## Seeded simulation (D1)

Source: `app/src/engine/sim.rs`. The table below came from a report test that printed these numbers (release build, about 3 minutes). That test was removed because it asserted nothing: restore it with `git show 67c9f5f:app/src/engine/sim.rs`. The 70% floor within 30 s is still guarded by the fast test `engine::sim::tests::leader_mechanism_reaches_the_floor_in_thirty_seconds`. Seed 1 unless stated, one topic, same faction, stage 0. Model and assumptions are in the module comment.

Metric (the plan's floor): of the peers that have a compatible five in their own sample at the start (the old sample rule's view), the share committed within the time shown. Floor: 70%, "within 3 epochs" of 10 s, which is 30 s. A round is one 10 s epoch for converging announcements, one 8 s proposal cycle for the proposer model (3 cycles, 24 s) and one 5 s epoch for leader invitations (ADR 0025), so 30 s is 6 leader rounds and 15 s is 3. The leader columns keep each tank's sample across rounds the way the engine does (the ads are re-sent every 45 s); a committed peer leaves a sample when its `Taken` arrives (probability = delivery). "Capacity" is the number of peers committed as a share of what one central greedy build over all peers commits (the oracle), which separates the mechanism from the cap that the role mix puts on every mechanism.

| Scenario | Eligible peers | Converging, 3 epochs (30 s) | Proposer, 3 cycles (24 s) | Leader, 3 rounds (15 s) | Leader, 6 rounds (30 s) | Capacity at 6 rounds |
|----------|----------------|------------------------------|---------------------------|-------------------------|-------------------------|----------------------|
| 3000 peers, sample 100, delivery 0.95 | 2062 / 3000 | 1.9% | 12.1% | 74.6% | 77.4% | 100.0% |
| 3000 peers, sample 100, delivery 0.80 | 2156 / 3000 | 0.0% | 18.9% | 71.4% | 76.2% | 99.8% |
| 3000 peers, sample 100, delivery 1.00 | 2027 / 3000 | 56.3% | 8.6% | 73.0% | 76.3% | 100.0% |
| 3000 peers, sample 300, delivery 0.95 | 2212 / 3000 | 1.9% | 11.0% | 72.2% | 76.3% | 100.0% |
| 300 peers, sample 100, delivery 0.95 | 196 / 300 | 0.0% | 25.5% | 68.9% | 68.9% | 100.0% |
| 100 peers, sample 100, delivery 0.95 | 53 / 100 | 11.3% | 50.9% | 64.2% | 64.2% | 100.0% |
| 10000 peers, sample 100, delivery 0.95 | 7046 / 10000 | 0.7% | 4.7% | 72.5% | 74.3% | 99.9% |
| 3000 peers, 8% tanks | 1056 / 3000 | 2.9% | 12.6% | 49.1% | 49.1% | 100.0% |
| 3000 peers, 30% tanks | 2234 / 3000 | 2.4% | 12.8% | 70.9% | 85.0% | 94.7% |

Seeds 1 to 8, headline scenario, leader invitations: 3 rounds 74.6, 74.3, 72.2, 69.7, 75.0, 73.7, 73.0, 71.5 (min 69.7%, mean 73.0%, max 75.0%); 6 rounds 77.4, 81.6, 81.5, 74.6, 80.7, 77.8, 77.2, 76.6 (min 74.6%, mean 78.4%, max 81.6%). Capacity at 6 rounds is 97.4% to 100.0% in every seed.

Design choices (headline scenario, 3 / 6 rounds unless stated):

| Choice | Result |
|--------|--------|
| spare invitations per leader, healers / damage: 0/0, 0/2, 1/3, **2/5**, 3/8, 6/20 | 54.6 / 73.1, 70.1 / 76.2, 73.1 / 76.3, **74.6 / 77.4**, 73.1 / 77.1, 61.1 / 77.6 |
| sample tie-break at equal level distance (fresh samples, 6 rounds): higher peer ID dropped (old) / hash of owner and peer (new) | 67.5% / 76.9% (3 rounds with the old rule: 35.9%) |
| direct messages lost, no retry: 1%, 5%, 20% | 73.8 / 77.8, 63.6 / 76.6, 26.4 / 47.3 |
| direct messages lost, one retry per message: 5%, 20% (two retries at 20%) | 74.7 / 77.3, 68.3 / 77.2 (72.5 / 77.4) |
| proposer share 0.05, 0.2, 0.6 (proposer model, 3 cycles) | 11.3%, 12.1%, 10.8% |

Reading:

* **Leader invitations reach the floor within 30 s** in every scenario with enough tanks and at least about 1,000 peers (74.3% to 85.0%), and at 3 rounds (15 s) in all but one seed in eight. Converging announcements fail because five peers cannot compute the same party from different samples. The proposer model fails because a group gains one member per cycle. Both numbers are unchanged from the earlier table.
* **The sample rule was a second cause.** The old tie-break kept the 100 lowest IDs of a level in every sample and left the other two thirds of the peers in nobody's. The new one (hash of owner and peer) gives every ad about the same share of samples and moves leader invitations from 67.5% to 76.9% at 6 rounds.
* **The role mix caps the metric.** Each party needs one tank, so 15% tanks can commit at most 75% of the peers (the oracle commits 2,235 of 3,000 in seed 1). With 8% tanks every tank is committed and the eligible share is still 49%: no mechanism can reach 70% of "peers with a compatible five in their sample" when tanks are that short, because those peers include many that no tank can take. 300 and 100 peers are in the same position (capacity 100%, share 69% and 64%). The floor as worded measures the role mix as much as the mechanism; defining it against capacity would be fairer. This table does not change the floor.
* **Not modelled:** peers that leave without a `Taken` (churn), real latency, the human Confirm, a real gossip layer (one swarm of 3,000 is assumed), several topics, secondary roles, and a lost message that comes back. Direct messages are assumed lost for good, which is pessimistic for iroh streams and ignores unreachable peers. The numbers show the mechanism works under the modelled divergence, not that it works live.
* The old mechanisms still run in `sim.rs` as the comparison. The leader mechanism and the oracle use the same population and the same eligible peers.

## Smoke run of the built binary (2026-11-21)

`forever-together --print-public-key` printed a stable 64-hex key, and `--headless` with the default iroh preset ran for 25 s and exited cleanly on SIGINT. Nothing was logged, so this shows only that the process starts and stays up; it does not show relay or DNS reachability. Still unverified: the real window, the live relay and DNS, the live game client, the phone scan and the on-screen addon window.

## Simulation re-run under the revised floor (2026-11-21)

The floor is now measured against what a central greedy build over the whole swarm could place (the `capacity` figure in the report). Re-run of the report (release, seeded):

| Scenario | Leader, 6 rounds (30 s), share of central-build capacity |
|---|---|
| 3000 peers, sample 100, delivery 0.95 | 100.0% |
| 3000 peers, delivery 0.80 | 99.8% |
| 3000 peers, delivery 1.00 | 100.0% |
| 3000 peers, sample 300 | 100.0% |
| 300 peers | 100.0% |
| 100 peers | 100.0% |
| 10000 peers | 99.9% |
| 3000 peers, 8% tanks | 100.0% |
| 3000 peers, 30% tanks | 94.7% |
| 8 seeds, 3000 peers | 97.4% to 99.8% at 3 rounds, all at least 97.4% |

Every row meets the 70% floor within 30 s under the revised measure, including the small and tank-scarce swarms. At 3 rounds (15 s) the lowest row is the 30%-tanks swarm at 77.6%. The old all-peers figures (for example 49.1% at 8% tanks) are capped by the role mix, not by the mechanism. Direct-message loss of 5% without retries costs about 10 points at 3 rounds; one retry restores it, and iroh direct streams are reliable. This is still a model: no live network, no churn.

## Key commands (ticket 0043, built 2026-11-21)

Checked on macOS with the built binary and `ssh-keygen`/`age`; Windows and Linux are not run.

| Question | Result |
|---|---|
| Does `varaq` refuse a second opener on a data directory? | **No.** A second `VaraqStore::open` on the same directory succeeds and can write; the first handle never sees the second's write (test `varaq_does_not_refuse_a_second_opener`). Two processes could corrupt the store. The app and the `key` commands therefore take an exclusive lock file (`DataDirLock`, ADR 0039 decision 9). A live headless node on a directory made `key show` and `key reset` fail with "another forever-together process is using ..." |
| Does `ssh-keygen -y -f` accept a file written by `key export`, and does it print the same public line as `key show`? | **Yes**, identical line. |
| Does a key written by `key export` decrypt with `age -d -i <file>` a message encrypted with `age -r <public line>`? | **Yes** (age installed at `/opt/homebrew/bin/age`). The author's workflow for scan exports (ADR 0038) works with the standard tool. Not yet run against an export made by the app's own encryption module (ticket 0044). |
| Does the app read an unmodified `ssh-keygen` ed25519 key? | **Yes** (a throwaway fixture in `app/src/keys.rs`; its public line is checked against `ssh-keygen -y`). Passphrase keys and ecdsa keys are refused with their own errors. |
| Mode of exported files and the lock file? | `0600` on macOS (tests). The `varaq` directory is set to `0700` and its files are `0600` on every open (existing test). Linux and Windows not run. |


## Live network run, one machine (ticket 0004, 2026-11-23)

Run with `cd app && cargo test --lib live_tests -- --ignored --nocapture` (needs the internet; `LIVE_T_SEARCHES=1` makes T search). Three apps in one process on one macOS machine behind one home NAT, each with its own data directory and key, on the real iroh network (`IrohConfig::Default`, n0 relays and DNS). A and B know T only by its public key. **This is not the two-network run**: it does not show hole punching, NAT behaviour between networks, or Windows and Linux.

| Question | Result |
|---|---|
| A fresh install reaches the trusted peer by key through n0 (step 1) | **Yes.** `bootstrap` became `trusted` in 0.5 to 5.6 s (three runs), after T had been up 20 s. |
| A restart uses remembered peers before the trusted list (step 2) | **Yes.** `remembered` in 5.1 s on a restart with the same data directory. Bootstrap runs per topic, so the source shows only once a search is running. |
| Peer exchange and signed gossip between two players (steps 3 and 4) | **Yes when T also searches the topic:** A and B saw each other in 15.3 s, and kept seeing each other and stayed connected 90 s after T stopped. **No when T never searches (what a `--headless` node does), before ADR 0040:** A and B each connected to T, stayed `alone` for the full 120 s and never met. T only holds their Hello as unverified claims and answers requests from verified peers; a headless node joins no topic and never dials claims. Fixed by the membership topic (ADR 0040): with the same non-searching T, A and B saw each other in 45.8 s and stayed connected 30 s after T stopped. |
| Hosts contacted (step 5) | Sampled with `lsof`: the only established connection seen was TCP 443 to `euc1-1.relay.n0.iroh.link` (the EU relay). Short-lived DNS and pkarr traffic and unconnected UDP sockets were not captured, so the list is incomplete. |
| Degraded cases (step 6), two networks, direct versus relayed path | **Not run.** |
| Forged-author rejection | Not run live (loopback tests only). |

## Relay rate limit, DHT lookup and build checks (ticket 0049, ADR 0041, 2026-11-24)

One macOS machine. The relay numbers are loopback (they show what the limit does, not how real NATs behave); the lookup numbers are against n0's DNS and the public Mainline DHT. Harness: `app/src/net/relay_server.rs` tests and `app/src/live_tests.rs` (both ignored measurements).

| Question | Result |
|---|---|
| Does `iroh-mainline-address-lookup` 0.6.1 build against `iroh` 1.5.0? | **Yes** (`cargo clippy` clean; used in `IrohNet::start`). |
| Does the `iroh-relay` client connect to a plain `http://` relay URL? | **Yes** (loopback: every relay test uses `http://127.0.0.1:<port>`). |
| Does a connection open through a rate-limited relay with no direct path? | **Yes**, in 6 to 10 ms at every rate tried (500 B/s with a 2 kB burst up to unlimited). |
| Does the connection still gain a direct path (hole punching) under the limit? | **Yes.** About 11 to 19 ms at 1 kB/s with a 4 kB burst and above, same as unlimited (12 to 14 ms). At 500 B/s with a 2 kB burst it took 4.0 s. |
| What does the limit do to bulk traffic through the relay? | It holds it to the rate. 20 kB took 7.3 s at 2 kB/s with an 8 kB burst, against 6.6 ms with no limit; at 500 to 1000 B/s a 20 kB transfer did not finish in 40 s (loss recovery collapses); 100 kB took 2.3 s at 20 kB/s with a 64 kB burst. **A rate limit caps sustained throughput, it cannot make it zero:** at 2 kB/s a client can still move small gossip messages through the relay. |
| Project value chosen | **2 kB/s, 8 kB burst** (`CLIENT_BYTES_PER_SECOND`, `CLIENT_BURST_BYTES`). Not checked against real NATs. |
| Lookup time for a node that knows only a key, two endpoints on one machine | DNS only 0.3 s; DHT only 5.0 s (second try); both 0.1 s. **iroh did not wait on the DHT** when DNS answered (one run). |
| The full app with the new default configuration on the real network | Same result as before the change: first contact with a trusted peer 5.1 s, remembered peers 0.5 s on restart, two players saw each other in 45.4 s with a non-searching trusted peer, still connected 30 s after it stopped. |
| Found on the way | `BUILTIN` held two keys with no node behind them. The bootstrap loop awaited each dial in turn inside a future the core drops whenever a command or tick arrives, so it never reached a live trusted peer listed after the dead ones. Fixed: dials are background tasks (`IrohNet::start_dial_peer`), and all trusted dials start before any is awaited. |
| Not run | Two networks, real NATs, the relay on `89.40.6.71`, Windows and Linux. |
