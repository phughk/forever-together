# 0004. Verify iroh over the real network

* **Status:** open
* **Kind:** verification
* **Priority:** P0
* **Blocked by:** tickets 8
* **Source:** ADR 0008, 0009, 0019; docs/spike-s0.md
* **ADRs:** none
* **EGM:** [initial draft](../docs/egm/0004-verify-iroh-live-network/design_draft.md) (2026-11-22, unreviewed)

## What

Only loopback iroh has been run. Run two machines across the internet: address lookup through n0's hosted DNS and relays, bootstrap order, whether `iroh-gossip` identifies message authors (ADR 0019 is replaced if it does), and which hosted components iroh uses by default.

## Done when

Findings in `docs/spike-s0.md`; ADR 0019 and ADR 0023 revised if needed.
