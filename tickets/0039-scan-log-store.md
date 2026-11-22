# 0039. Scan log: store module

* **Status:** open
* **Kind:** feature
* **Priority:** P2
* **Source:** ADR 0038
* **ADRs:** [0038](../docs/adr/0038-scan-log-and-opt-in-export.md)
* **EGM:** [initial draft](../docs/egm/0039-scan-log-store/design_draft.md) (2026-11-22, unreviewed)

## What

Add the `scans` module over the key-value store: append on a successful code paste, dedupe on `created` and `counter`, max 20 entries per character (region, realm, first name, surname), expire after 90 days, prune on write and before export, skip corrupt entries, never block a paste on a failed write.

## Done when

Unit tests with invented characters for append, dedupe, the 21st entry, expiry, corrupt entry and failing store.
