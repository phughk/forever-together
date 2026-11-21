# 0019. Gossip messages are signed by their author

* **Status:** Accepted (2026-11-21, by the author; decision D8). Replaced if spike S0 shows that iroh-gossip already identifies message authors.
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

`iroh-gossip` delivers a message together with the neighbour that forwarded it, not with its original author (to be confirmed in the spike). Parties are agreed by converging announcements, and each member must know who said what.

## Decision

Every gossip message (`Ad`, `Taken`, `PartyAnnounce`, `Confirm`, `Decline`) carries the author's peer ID and an ed25519 signature made with the author's iroh key. Receivers verify it before use. Messages are serialized with `postcard`. The peer exchange protocol runs on its own iroh connections, so the remote peer is authenticated by the connection.

## Consequences

Signatures prove that a message came from the peer that holds the key, not that its contents are true. Verification adds a small cost per message. If the spike shows that gossip already identifies authors, this record is replaced.

## Alternatives considered

* Trust the forwarding neighbour: not chosen, a neighbour could forge messages from others.
