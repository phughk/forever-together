# 0001. Verify addon client calls on a live client

* **Status:** open
* **Kind:** verification
* **Priority:** P2
* **Blocked by:** tickets 42
* **Source:** docs/spike-s0.md
* **ADRs:** none
* **EGM:** none

## What

Confirm against a live Forever client the region values other than EU (3), and the Hardcore realm value once that ruleset is released.

Samples from other regions are collected through the scan log (tickets 0039 to 0042), exported encrypted to the first trusted peer key (0044) so a public issue is safe. That needs the key handling in 0043 and the trusted key in 0008, so this ticket now waits on them. Hardcore stays open until the ruleset is released.

Done 2026-11-21: `GetAverageItemLevel` works, and RP-PvE is realm value 16 (see `docs/spike-s0.md` and ADR 0004).

## Done when

Each remaining value is recorded in `docs/spike-s0.md` with the date; Names.lua and code.rs updated if they differ.
