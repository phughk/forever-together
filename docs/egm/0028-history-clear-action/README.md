# 0028-history-clear-action: clear search history action

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0028](../../../tickets/0028-history-clear-action.md)
* **ADRs:** [0027](../../adr/0027-search-history.md), [0032](../../adr/0032-settings-phone-link-and-advanced.md), [0035](../../adr/0035-protocol-constants-engine-contract-and-api.md)
* **Rounds:** none yet

## Question

The desktop-only "Clear search history" action and the deletion with the character are already in the code. This pass decides how to check the privacy promise (history never in logs, exports, pairing or the API) and what a clear should do to a search that is still running.

## Files

* `design_draft.md`
