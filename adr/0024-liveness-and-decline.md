# 0024. A committed party needs a Ready check, a human Confirm, and a Decline without penalty

* **Status:** Accepted (window lengths are starting values)
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

A committed party can fail because one of the five humans is away, wasting a full confirm timeout for four players. Review decision D6 asked for a machine-level check before the human Confirm.

## Decision

After a commit, a party's members broadcast `Taken` and each app sends a signed `Ready` within 4 seconds and shows the Confirm prompt. Confirm needs a human click, and the protocol carries it as a `Confirm` message. If nobody clicks within 15 seconds, the app sends a `Decline` (the check failed). The party is dropped, the other members return to searching, and the peer stays in the network with no penalty. If a peer does not answer a heartbeat within 4 seconds, the others mark it unreachable locally and ban it for 60 seconds. The ban is local and is never gossiped. Decision windows are 3 to 5 seconds, and search-level windows about 1 minute (the widening step is 60 seconds).

## Consequences

Absent or crashed apps are caught in seconds. The ban works per key, so someone who rotates keys avoids it (accepted, see the design's risk table). This replaces the earlier rule that a non-confirmer gets a local backoff in [0010](0010-converging-announcements.md).

## Alternatives considered

* Confirm only, with one confirm timeout for a missing member: not chosen by the author.
* Gossiping bans: rejected by the author, because a shared ban list could be used as a weapon against a victim.
