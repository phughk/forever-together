# 0025-diagnosis-attempt-counters: search diagnosis attempt counters

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0025](../../../tickets/0025-diagnosis-attempt-counters.md)
* **ADRs:** [0026](../../adr/0026-search-health-from-own-observations.md), [0027](../../adr/0027-search-history.md), [0025](../../adr/0025-leader-invitations.md)
* **Rounds:** none yet

## Question

The ticket asks for attempt counters and two new situations in the search diagnosis. The code and ADR 0026's amendments already contain them, so this pass decides whether the ticket can close as done, what still needs checking, and what to do about the cases the counters do not cover (a node that never sends invitations).

## Files

* `design_draft.md`
