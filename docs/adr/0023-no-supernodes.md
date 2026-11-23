# 0023. Avoid supernodes: trusted peers are ordinary peers

* **Status:** Accepted. Extended by [0041](0041-address-lookup-and-hole-punching-relay.md) (headless relay nodes)
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

First contact needs at least one known peer. Review of the bootstrap design noted that every cold start reaches the trusted peers, so they behave like central hubs, and that peer exchange on quiet topics depends on neighbours who already know someone.

## Decision

Reduce supernodes as a rule. Trusted peers are the same app running headless. They take part in peer exchange like everyone else and learn their peers from non-trusted peers, so the exchange is the source of truth and the built-in list is not. They are only the fallback when remembered peers and peer exchange fail. When the network is quiet, the trusted peers can be reused as the fallback. Peer exchange stays in v1 with no spike gate (this keeps [0009](0009-bootstrap-and-peer-exchange.md) as accepted).

## Consequences

A new install has nothing but the trusted peers, so their load follows the number of cold starts. This is an accepted risk, to be measured. Designs that make trusted peers learn every topic (per-topic lists published by trusted peers) are rejected for this reason.

## Alternatives considered

* Prove in spike S0 that peer exchange beats "remembered peers plus trusted peers": not chosen by the author.
* Defer peer exchange: not chosen by the author.
