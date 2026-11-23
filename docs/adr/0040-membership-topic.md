# 0040. Every node joins a membership topic, and new members are announced on it

* **Status:** Accepted (decided by the author 2026-11-23; built 2026-11-23, ticket 0047). Extends [0009](0009-bootstrap-and-peer-exchange.md) and [0023](0023-no-supernodes.md).
* **Date:** 2026-11-23
* **Deciders:** Hugh Kaznowski
* **Source:** the live run in [spike results, "Live network run"](../spike-s0.md); ticket [0047](../../tickets/0047-headless-trusted-peer-introduces-nobody.md).

## Context

Dungeon topics are only joined by players who search them. On the real network, two fresh players who knew only a trusted peer's key both reached it but never found each other when the trusted peer searched nothing, which is what a headless node does. It held their Hellos as unverified claims, answered only for verified peers, and was in no dungeon swarm to bring them together. Players saw `alone`.

## Decision

1. **One membership topic.** `membership_topic()` is `SHA-256("wowft/1/membership")` (`app/src/dungeons.rs`). Every node, window or headless, searching or not, joins it when the network starts and never leaves it. It carries only `NewMember` messages.
2. **Announcing.** When a node's membership swarm gains a neighbour it has never verified before, it broadcasts a signed `NewMember { peer: me, member }` on the topic.
3. **Acting on it.** A node that receives a `NewMember` hands `member` to every swarm it is in as a bootstrap peer (`join_peer`). If the member searches the same dungeon topic, the two connect. If not, the join is ignored.
4. **Joining a dungeon topic** bootstraps from peers remembered for that topic first, then from remembered members, up to 5.
5. **Bootstrap** always includes the membership topic, first in the list (a peer-exchange request names at most 3 topics), so a headless node bootstraps and answers like everyone else.
6. **Limit.** A poster can name any key, and each one costs a dial, so a node acts on at most 30 `NewMember` messages per 60 s and drops the rest.
7. `NewMember` is the last variant of `Message`, so existing message numbers do not move. The engine ignores the resulting `Event::MemberJoined`.

## Consequences

* A trusted peer needs no special role: it is a member that happens to be always on.
* Verified result on the real network (one machine, 2026-11-23): with a trusted peer that never searches, two fresh players saw each other in 45.8 s (15 s when the trusted peer also searched) and stayed connected 30 s after it stopped. The slower time is not explained.
* Every node holds the whole membership swarm's gossip. The known-peers cache fills with members, which compete for its slots (active dungeon topics keep reserved slots). Neither the swarm size nor the cache pressure has been measured at scale; ticket 0021 (sharding) is where that shows up.
* A hostile poster can make nodes dial junk keys at up to the rate in point 6.
* Not covered: whether a node should leave a quiet membership swarm to save traffic.
