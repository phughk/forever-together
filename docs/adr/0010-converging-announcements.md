# 0010. Peers agree on parties by converging announcements and a human Confirm

* **Status:** Superseded by [0025](0025-leader-invitations.md) on 2026-11-21. The simulation in spike S0 showed converging announcements commit about 2% of eligible peers at 3,000 peers, a sample of 100 and delivery 0.95. The human Confirm, `Taken` and Decline rules below stay in force.
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

No coordinator exists, and a player must never end up in two final parties.

## Decision

Time is divided into epochs. Every peer builds parties with the same deterministic rule from its sample of ads (shared order by `hash(epoch, peerId)`, role and level compatibility) and announces the party that includes it. A party is committed when all five members announced the same set. Each member then presses Confirm, and a party is final only when all five confirmed. A peer confirms at most one party. A party not confirmed in time is dropped and its members search again.

## Consequences

A late-seen commit can waste work for four players, because the one-party guarantee holds at confirmation. If samples differ, matches stall until a later epoch. Epoch length, ad cutoff, sample size and confirm timeout start as best guesses.

## Alternatives considered

* Proposer with per-peer leases: not chosen by author.
* Elected coordinator per shard: not chosen by author.

## Revision 2026-11-21

Decided by the author (decisions D2 and D6 in `docs/design-process/archive/reviews/author_decisions.md`):

* A proposer model with short response windows is a second candidate. A partial group advertises the roles it needs, responders reply, and the proposer decides within a few seconds. A responder accepts one proposer at a time, which is a lease. Spike S0 simulates both mechanisms and the better commit rate wins. This record stays Accepted until then.
* A party that forms broadcasts `Taken` for its members, so others drop them from their candidate lists.
* After a commit each app sends `Ready`, then the human clicks Confirm. Details are in [0024](0024-liveness-and-decline.md). This replaces "the non-confirmer gets a local backoff": a member who does not click sends a `Decline` and is not penalised.
