# 0039-scan-log-store: Scan log, store module

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0039](../../../tickets/0039-scan-log-store.md)
* **ADRs:** [0038](../../adr/0038-scan-log-and-opt-in-export.md), [0027](../../adr/0027-search-history.md) (the pattern this follows), [0039](../../adr/0039-key-management.md) (style rule: generics, no `dyn`)
* **Rounds:** none yet

## Question

How the `scans` module keeps the log of accepted character codes in the key-value store: the key layout, the append, dedupe, per-character cap, expiry and pruning rules, how corrupt entries and failing writes are handled, where the app calls it, and whether anything bounds the number of characters.

## Files

* `design_draft.md`
