# 0005. A topic is region + realm + faction + dungeon, and level is not in it

* **Status:** Accepted (supersedes the earlier decision to put exact level in the key; amended 2026-11-21: one shard per topic in v1)
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The first design keyed searches by `region + realm + faction + dungeon + level`. The author expects very large player numbers, wants rare activities to stay rare and deterministic, and wants fewer topics to gossip well.

## Decision

A topic is `region + realm + faction + dungeon`, hashed with a protocol prefix into a 32-byte topic ID. Level travels in each player's ad and is a matching condition (maximum level spread). About 400 topics exist (5 realms, 2 regions, 2 factions, 20 available dungeons). Ads are sent on change plus a slow keep-alive, and a peer's sample of ads favours similar levels (proposed). *Amended 2026-11-21:* v1 has one shard per topic. The shard number stays in the topic hash and is always 0, so shards can be added later without renaming topics. Sharding by peer ID alone would starve each shard of tanks and healers, so adding it needs a role-aware rule and real traffic numbers.

## Consequences

Swarms are bigger, so the broadcast load needs care (see the load estimate in the design). The level-widening step that joined adjacent-level topics is gone: widening raises the allowed level spread instead.

## Alternatives considered

* Exact level in the key: replaced, because it made swarms tiny and spread rare activities across many keys.
* Level bands in the key: not chosen.
