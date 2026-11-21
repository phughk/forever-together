# 0014. Tests cover functional behaviour, with no performance or large-scale tests

* **Status:** Accepted (amended 2026-11-21: a seeded algorithm simulation is allowed)
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The review suggested simulating 10,000 or more peers.

## Decision

Test normal behaviour: correct roles in parties, one confirmed party per peer, dropped unconfirmed parties and resumed searches, and widening that follows the schedule, with a handful to a few dozen simulated peers on the in-memory fake network. The addon's encoder and QR are tested in the headless VM. A real phone scan and the QR's on-screen look are manual checks.

## Consequences

Tuning of epoch length, cutoffs, sample size and shard counts comes from use on the real network.

## Alternatives considered

* A seeded simulator with 10,000 or more peers: rejected by author.

## Amendment 2026-11-21

The author allowed a small seeded simulation of the agreement rule on the pure `engine`: a few thousand synthetic ads with modelled sample divergence, covering the timing windows. It checks the algorithm's commit rate and is not a load or performance test. It runs in spike S0 and stays as a test. The 10,000-peer simulator remains rejected.
