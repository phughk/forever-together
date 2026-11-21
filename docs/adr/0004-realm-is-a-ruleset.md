# 0004. A realm is a ruleset on a shared mega server, supplied by the addon as an opaque value

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

On Forever a realm is a ruleset (PvP, PvE, RP-PvP, RP-PvE, later Hardcore) running as a sharded mega server in each region. Everyone on the same realm can group, and players on different realms cannot. An earlier draft wrongly treated realms like classic realm names.

## Decision

The addon sends the region and the realm as opaque values from the real client, and the app only compares them. The app hard-codes no realm list. Hardcore is just another realm.

## Consequences

Which client calls return the realm and region is unverified. Candidates are `C_GameRules.GetActiveGameMode`, `GetCurrentGameModeRecordID`, `GetForeverExperiencePreset`, `IsHardcoreActive` and `GetCurrentRegion`. Someone must log them on the real client before the code format freezes.

**Amendment 2026-11-21 (first live capture):** the realm value is the game-mode record ID as a decimal string (14 = PvP, 15 = PvE, 18 = RP-PvP) and the region is `GetCurrentRegion()` (3 = EU). RP-PvE was captured later the same day: 16 = RP-PvE. The Hardcore ID is unknown because the ruleset is not released. Names are for display only; topics still hash the raw values. See `docs/rdf-code-spec.md` "Realm and region".

## Alternatives considered

* Asking the player to pick the realm type: rejected, a wrong choice puts them in a pool that cannot invite them.
