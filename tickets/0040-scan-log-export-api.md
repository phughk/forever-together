# 0040. Scan log: export and clear API

* **Status:** open
* **Kind:** feature
* **Source:** ADR 0038
* **ADRs:** [0038](../docs/adr/0038-scan-log-and-opt-in-export.md)
* **EGM:** none

## What

Loopback-only routes to export the log as one JSON file (app version, spec version, export time, characters with raw code and accepted time) and to clear it. `ClearCharacter` does not clear it.

## Done when

Server tests for the routes and for the loopback restriction; the export never includes search history.
