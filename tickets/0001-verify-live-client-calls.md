# 0001. Verify addon client calls on a live client

* **Status:** open
* **Kind:** verification
* **Source:** docs/spike-s0.md
* **ADRs:** none
* **EGM:** none

## What

Confirm against a live Forever client the region values other than EU (3), and the Hardcore realm value once that ruleset is released.

Done 2026-11-21: `GetAverageItemLevel` works, and RP-PvE is realm value 16 (see `docs/spike-s0.md` and ADR 0004).

## Done when

Each remaining value is recorded in `docs/spike-s0.md` with the date; Names.lua and code.rs updated if they differ.
