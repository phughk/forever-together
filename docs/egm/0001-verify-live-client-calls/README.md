# 0001-verify-live-client-calls: Verify addon client calls on a live client

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0001](../../../tickets/0001-verify-live-client-calls.md)
* **ADRs:** [0004](../../adr/0004-realm-is-a-ruleset.md), [0038](../../adr/0038-scan-log-and-opt-in-export.md)
* **Rounds:** none yet

## Question

How the remaining unverified game values (region numbers for US, KR, TW and CN, and the Hardcore realm value) are collected from players' scan-log exports, what counts as verified, and which spec, addon and app tables change when a value arrives. The ticket waits on the key work (0043), the trusted key (0008) and the export (0044).

## Files

* `design_draft.md`
