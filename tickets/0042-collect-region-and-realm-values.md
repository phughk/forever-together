# 0042. Collect region and realm values from players

* **Status:** open
* **Kind:** verification
* **Priority:** P2
* **Blocked by:** tickets 41, 44
* **Source:** ADR 0038; ticket 0001
* **ADRs:** [0038](../docs/adr/0038-scan-log-and-opt-in-export.md)
* **EGM:** none

## What

Encrypted exports are safe to attach to a public issue. Ask players on US, KR, TW and CN (and any new ruleset such as Hardcore) to send their export in a GitHub issue; add an issue template that says what to attach and warns about names. Record the mapped values in the spec and ADR 0004.

## Done when

The spec's region and realm tables list each value as verified, with the date, without the exported files being committed.
