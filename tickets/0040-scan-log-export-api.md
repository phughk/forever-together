# 0040. Scan log: export and clear API

* **Status:** open
* **Kind:** feature
* **Priority:** P2
* **Blocked by:** tickets 39, 44
* **Source:** ADR 0038
* **ADRs:** [0038](../docs/adr/0038-scan-log-and-opt-in-export.md), [0039](../docs/adr/0039-key-management.md)
* **EGM:** [initial draft](../docs/egm/0040-scan-log-export-api/design_draft.md) (2026-11-22, unreviewed)

## What

Loopback-only routes to export the log as one JSON file, encrypted to the first trusted peer key (ticket 0044) (app version, spec version, export time, characters with raw code and accepted time) and to clear it. `ClearCharacter` does not clear it.

## Done when

Server tests for the routes and for the loopback restriction; the export never includes search history.
