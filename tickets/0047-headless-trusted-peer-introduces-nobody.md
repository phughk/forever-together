# 0047. A headless trusted peer does not introduce players to each other

* **Status:** open
* **Kind:** bug
* **Priority:** P0
* **Source:** ticket 0004 live run, 2026-11-23 (`docs/spike-s0.md`, "Live network run")
* **ADRs:** [ADR 0009](../docs/adr/0009-bootstrap-and-peer-exchange.md), [ADR 0023](../docs/adr/0023-no-supernodes.md)
* **EGM:** none

## What

On the real network, two fresh players who know only a trusted peer's key both reach it but never find each other if that peer is not itself searching the topic. A `--headless` node never joins a topic, and it answers peer-exchange requests only with peers it has verified for the topic. A player's Hello is recorded as an unverified claim, and nothing dials claims on a headless node, so it has nothing to hand out. Players are told they are `alone`.

With the trusted peer also searching the topic, the same two players met in 15 s.

## Done when

The design decides how a trusted peer introduces players (it could verify claims by dialling them, join topics from Hello claims, or relay gossip), the fix is built, and the live test in `app/src/live_tests.rs` passes with a headless-style T (no `LIVE_T_SEARCHES`). ADR 0009 or 0023 amended if the decision changes them.
