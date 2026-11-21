# Spike S0 results


| Output | Result | Assumption used in the code now | Fallback if the assumption is wrong | Spec / code to change |
|--------|--------|--------------------------------|-------------------------------------|-----------------------|
| Realm (ruleset) call, its values for PvP, PvE, RP-PvP, RP-PvE, Hardcore, and whether it fits 24 bytes | unverified, needs live client | `C_GameRules.GetCurrentGameModeRecordID()` returns a number, sent as a decimal string. If that call is missing or fails, `GetRealmName()` lowercased in ASCII is used (a realm name is not a ruleset, so this fallback can put two rulesets in one topic). Neither is confirmed to be the ruleset. | add a mapping table to canonical IDs in the spec; if no call gives the ruleset, the code is not built | `docs/rdf-code-spec.md` "Realm and region"; `addon/ForeverTogether/Core.lua` `realmValue` |
| Region call and its values | unverified, needs live client | `GetCurrentRegion()` returns a number 0 to 255, copied as is, 0 when the call returns nothing | none, the code is not built | spec "Realm and region"; `Core.lua` `Collect` |
| `UnitName("player")` and `GetUnitName("player")` return first name and surname, and the in-game invite accepts "First Surname" | unverified, needs live client | the first return of `UnitName("player")`, else of `GetUnitName("player")`, is "First Surname" and is split on its first space. The second return (realm) is never read (ADR 0022). A name with no space cannot be built. | split the combined name differently; spec change | spec "Collecting each field"; `Core.lua` `splitName` |
| `GetAverageItemLevel` works | unverified, needs live client | `GetAverageItemLevel()` first return is the item level; 0 or nil means unknown | slot sum on `PLAYER_EQUIPMENT_CHANGED` (`RD.SumSlots`: mean over 17 slots, a two-hander counted in both hands; the exact formula is a guess) | `Core.lua` `sumSlots` |
| `AreCrossFactionGroupQueuesAllowed` value, and whether cross-faction grouping exists | unverified, needs live client | parties are same-faction only (ADR 0007); faction is in the topic and in the code (flag bit 4) | faction stays in the topic and also goes in the ad (already the design) | ADR 0007, design |
| A phone camera scans a QR drawn with `SetColorTexture` at UI scales 0.8, 1.0 and 1.2 | unverified, needs phone | one texture per dark module, whole screen pixels per module (the largest that fits 230 UI units, with a four-module quiet zone), black on a white square. The matrix itself is checked against two independent encoders in `docs/rdf-code-vectors.json`; only the on-screen drawing and the scan are unchecked. | change the module size or the area, or show text only | `addon/ForeverTogether/Window.lua` `QR_AREA`, `QUIET` |
| `varaq` keeps the secret key with file mode readable only by the user and survives a restart | unverified, needs live app | not used yet (app code not started) | OS keychain for the key | ADR 0008, `src-tauri/src/store/` |
| iroh-gossip identifies message authors | unverified, needs live network | gossip messages carry the author's peer ID and a signature (ADR 0019) | if yes, drop signatures (ADR 0019 replaced) | spec section 3 |
| iroh address lookup resolves a trusted peer from its public key | unverified, needs live network | trusted peers are listed by public key only | list trusted peer addresses in the app | `src-tauri/src/net/trusted.rs` |
| iroh's default hosted components (relays, DNS) are named, and whether they can be self-hosted or switched off | unverified, needs live network | recorded nowhere yet | record as a dependency in the design's risks table | design risks table |
| Seeded simulation (D1): commit rate for converging announcements and the proposer model at 3,000 peers and a sample of 100 | unverified, needs the engine (step 3) | converging announcements with a human Confirm (baseline) | adopt the better one; if neither reaches the 70% floor in 3 epochs, reopen the design | engine, ADR 0010 |

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

Source: `src-tauri/src/engine/sim.rs`. Run: `cargo test --release --lib -- --ignored --nocapture engine::sim::tests::report` from `src-tauri/`. Seed 1, one topic, same faction, stage 0. Model and assumptions are in the module comment.

Metric: of the peers that have a compatible five in their own sample at the start, the share committed within 3 rounds. A round is one 10 s epoch for converging announcements and one 8 s proposal cycle (4 s response window plus 4 s decision window) for the proposer model. Floor: 70% (plan).

| Scenario | Peers with a compatible five | Converging announcements, 3 epochs | Proposer model, 3 cycles | Proposer model, 4 cycles |
|----------|------------------------------|------------------------------------|--------------------------|--------------------------|
| 3000 peers, sample 100, delivery 0.95 | 2062 / 3000 | 1.9% | 12.1% | 15.8% |
| 3000 peers, sample 100, delivery 0.80 | 2156 / 3000 | 0.0% | 18.9% | 25.0% |
| 3000 peers, sample 100, delivery 1.00 | 2027 / 3000 | 56.3% | 8.6% | 11.0% |
| 3000 peers, sample 300, delivery 0.95 | 2212 / 3000 | 1.9% | 11.0% | 15.3% |
| 300 peers, sample 100, delivery 0.95 | 196 / 300 | 0.0% | 25.5% | 39.3% |

Proposer share (chance a free peer starts a proposal per cycle) from 0.05 to 0.6 moves the 3 cycle result only between 10.8% and 12.1% at 3000 peers, delivery 0.95.

Reading: **neither mechanism reaches the 70% floor in this model.** Converging announcements need every member to build the same party from its own sample, so any divergence (delivery below 1.0) collapses them, and even with perfect delivery they reach 56% in 3 epochs because the level-biased sample drops the same far-level ads for some members but not others. The proposer model is far less sensitive to divergence but slow, since a group gains members only one cycle at a time and fails when a cycle adds nobody. The design should be reopened per the plan ("if neither reaches the floor, reopen the design"), or the model assumptions (role mix, level spread, one swarm of 3000 for one topic, proposer share, no group merging in the proposer model) should be revisited first.
