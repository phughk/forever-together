# 0038. Direct connections only; n0 relays as fallback (was: decide whether to self-host iroh relay and lookup)

* **Status:** open
* **Kind:** decision
* **Priority:** P1
* **Blocked by:** tickets 4
* **Source:** ADR 0034 (risks)
* **ADRs:** none
* **EGM:** [initial draft](../docs/egm/0038-self-host-iroh-relay/design_draft.md) (2026-11-22, unreviewed)

## What

The app uses iroh's default N0 preset, so first contact by key and NAT traversal depend on n0's hosted relays and DNS. Decide whether to self-host `iroh-relay` and lookup, or accept the dependency.

## Done when

An ADR records the choice, after ticket 0004 shows how it behaves live.
