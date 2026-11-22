# 0027-history-estimator: search history estimator (stage 8b)

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0027](../../../tickets/0027-history-estimator.md)
* **ADRs:** [0027](../../adr/0027-search-history.md), [0026](../../adr/0026-search-health-from-own-observations.md), [0003](../../adr/0003-self-reported-data.md)
* **Rounds:** none yet

## Question

The estimator the ticket describes is already in the code and wired to the diagnosis, although the ticket says to build it only after a live capture shows the fields are stable. This pass decides whether it stays enabled in builds before any real entries exist, how the unused second checkpoint is treated, and how the constants get checked against real data.

## Files

* `design_draft.md`
