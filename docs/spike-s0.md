# Spike S0 results

Status: **not run.** Every row below is `unverified, needs live client/phone` (or live network for the iroh rows). The addon (step 2) was built on the spec's provisional values so that it can be tested in the headless VM. Rows are the S0 outputs in [the implementation plan](designs/forever-together-implementation-plan.md), item 0. The author fills in the result column when the spike runs and then edits the spec lines named in the last column.

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
