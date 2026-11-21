# 0016. Peers are identified by their iroh key, never by character name

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

Every Forever character has a first name and a surname, and the full name is unique across all Forever realms, but the code is self-reported, so a name cannot be trusted.

## Decision

Treat the first name and surname (kept as two fields) as a display label and as the text the player types into the in-game invite. Do not reject a code because another peer claims the same first name and surname, since that would let a forger lock a real player out. Show duplicates as a warning, refuse parties where two members claim the same first name and surname, and label revealed names as claimed by the other player. The code may carry the character GUID as a consistency check.

## Consequences

Someone can cause strangers to invite a victim's name. Per-peer rate limits on proposals reduce that.
