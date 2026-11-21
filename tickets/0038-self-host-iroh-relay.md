# 0038. Decide whether to self-host iroh relay and address lookup

* **Status:** open
* **Kind:** decision
* **Source:** ADR 0034 (risks)
* **ADRs:** none
* **EGM:** none

## What

The app uses iroh's default N0 preset, so first contact by key and NAT traversal depend on n0's hosted relays and DNS. Decide whether to self-host `iroh-relay` and lookup, or accept the dependency.

## Done when

An ADR records the choice, after ticket 0004 shows how it behaves live.
