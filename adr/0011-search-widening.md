# 0011. A search loosens over time on a shared schedule

* **Status:** Accepted (revised: no player control, no API)
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

A topic can run dry even when most topics are crowded.

## Decision

A search starts strict and loosens in a fixed, visible order: item level and role mix, then a larger allowed level spread, then more shards of the topic, then more dungeons. The schedule is part of the matching algorithm. It is not controlled through the API: the player cannot pause, skip or change it, and the app only shows the current stage. Its numbers are settings with defaults. Each ad carries the peer's current stage.

## Consequences

The step sizes, intervals and limits are open questions. A player cannot stop a search from reaching the wider stages except by stopping the search.

## Revision history

* An earlier version let the player pause widening or set limits through the API. Reversed by the author: widening is only the matching algorithm.
