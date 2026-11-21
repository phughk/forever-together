# 0007. Parties are same-faction only, and faction is in the code and the topic

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

Players can only form a party with their own faction on the same realm.

## Decision

The code carries the character's faction, and the topic includes it. Horde and Alliance players are never matched. The dungeon table's faction column is a hint for recommendations and not a rule about who may enter.

## Consequences

Whether to check `AreCrossFactionGroupQueuesAllowed` on the real client in case cross-faction groups arrive later is an open question.
