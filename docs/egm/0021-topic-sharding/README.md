# 0021-topic-sharding: shard busy topics

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0021](../../../tickets/0021-topic-sharding.md)
* **ADRs:** [0005](../../adr/0005-topic-without-level.md), [0035](../../adr/0035-protocol-constants-engine-contract-and-api.md)
* **Rounds:** none yet

## Question

Whether a topic with too many peers should be split into shards, when that is worth doing, and what rule assigns a peer to a shard without starving shards of tanks and healers.

## Files

* `design_draft.md`
