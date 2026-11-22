# 0005-verify-agreement-live: Verify leader invitations on a live network

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0005](../../../tickets/0005-verify-agreement-live.md)
* **ADRs:** [0024](../../adr/0024-liveness-and-decline.md), [0025](../../adr/0025-leader-invitations.md), [0035](../../adr/0035-protocol-constants-engine-contract-and-api.md)
* **Rounds:** none yet

## Question

How to run enough real instances to measure whether leader invitations commit parties at the 70% floor, whether role scarcity caps the result, and whether the engine's lack of retransmission for lost direct messages matters, and how live numbers are recorded next to the simulation.

## Files

* `design_draft.md`
