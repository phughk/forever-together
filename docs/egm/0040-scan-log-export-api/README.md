# 0040-scan-log-export-api: Scan log, export and clear API

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0040](../../../tickets/0040-scan-log-export-api.md)
* **ADRs:** [0038](../../adr/0038-scan-log-and-opt-in-export.md), [0039](../../adr/0039-key-management.md), [0035](../../adr/0035-protocol-constants-engine-contract-and-api.md) (API table), [0027](../../adr/0027-search-history.md) (the loopback clear route this follows)
* **Rounds:** none yet

## Question

Which loopback-only routes the app needs to preview, export (encrypted) and clear the scan log, what the exported JSON contains, and how the encrypted file gets from the app to the player, given that the export function itself (ticket 0044) and the store (ticket 0039) are separate pieces.

## Files

* `design_draft.md`
