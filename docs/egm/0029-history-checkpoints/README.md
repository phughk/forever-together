# 0029-history-checkpoints: record history checkpoints at about 1 and 2 minutes

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0029](../../../tickets/0029-history-checkpoints.md)
* **ADRs:** [0027](../../adr/0027-search-history.md), [0026](../../adr/0026-search-health-from-own-observations.md)
* **Rounds:** none yet

## Question

Checkpoint capture is already in the engine and the history entry. This pass decides how to close the ticket and what to do with checkpoints that are taken late (after a committed party returns to searching), which the entry cannot tell apart from on-time ones.

## Files

* `design_draft.md`
