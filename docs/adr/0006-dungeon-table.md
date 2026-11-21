# 0006. The app hard-codes a dungeon table and recommends dungeons from level and zone name

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The author's table lists dungeons with level ranges, minimum levels, a faction column, zones. Nine entries have no known minimum level yet. Looking up entrance data from the client was considered, and map IDs and positions are not available to the app yet.

## Decision

Hard-code the table in the app as a data file. Recommend dungeons by level (minimum level and recommended range, with slack) and by whether the dungeon's zone name equals the character's zone name. The player picks a dungeon or searches the recommendations. Nothing refuses a player because of location. Every dungeon in the table can be searched. *Amended 2026-11-21:* the author decided that none of the 29 dungeons is "coming soon", so the table has no availability or status field and the app has no such concept. An entry with no known minimum level has `minLevel: null`, which means no minimum: it is recommended by its level range alone, and no minimum is invented for it. Search keys use a fixed English slug for the dungeon. Players search from outside a dungeon, so the code has no dungeon field and no instance handling. Map IDs and precise locations come later.

## Consequences

Clients in other languages report localized zone names, so they get no same-zone ordering until map IDs are added. The faction column is only a proximity hint.

## Alternatives considered

* Deducing the dungeon only from location and refusing players elsewhere: replaced by the author with a recommendation.
* Using the client's dungeon entrance data (`GetDungeonEntrancesForMap`): not chosen for now by the author.
* Pure proximity matching with no dungeon concept: not chosen.
