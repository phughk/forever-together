# 0021. Shard busy topics

* **Status:** open
* **Kind:** idea
* **Priority:** P3
* **Blocked by:** tickets 4, 5
* **Source:** ADR 0005; design doc Future Improvements
* **ADRs:** none
* **EGM:** none

## What

The shard number is in the hash and fixed at 0. Sharding needs real traffic numbers and a role-aware rule, since sharding by peer ID alone starves shards of tanks and healers.

## Done when

Shard table and rule decided after live traffic data.
