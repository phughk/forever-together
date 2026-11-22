# 0013-blocking-and-trust: blocking, verification and trust networks

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0013](../../../tickets/0013-blocking-and-trust.md)
* **ADRs:** [0003](../../adr/0003-self-reported-data.md), [0016](../../adr/0016-peers-identified-by-key.md), [0019](../../adr/0019-signed-gossip-messages.md), [0024](../../adr/0024-liveness-and-decline.md), [0034](../../adr/0034-system-overview-goals-and-risks.md) (none decided for this ticket)
* **Rounds:** none yet

## Question

Whether and how a player can keep a peer out of their parties for longer than the 60 second local ban, and whether any block or trust signal is shared between peers, given that ADR 0024 rejected gossiped bans and ADR 0034 lists shared reputation and blocking as a non-goal.

## Files

* `design_draft.md`
