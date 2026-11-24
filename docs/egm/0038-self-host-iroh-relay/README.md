# 0038-self-host-iroh-relay: Self-host iroh relay and address lookup, or accept n0's

* **Status:** closed (draft completed with the author through six decisions on 2026-11-23, no review round run; built 2026-11-24)
* **Ticket:** [0038](../../../tickets/0038-self-host-iroh-relay.md)
* **ADRs:** [0041](../../adr/0041-address-lookup-and-hole-punching-relay.md), [0034](../../adr/0034-system-overview-goals-and-risks.md) (risk row), [0009](../../adr/0009-bootstrap-and-peer-exchange.md), [0023](../../adr/0023-no-supernodes.md), [0008](../../adr/0008-iroh-and-local-storage.md); [spike results](../../spike-s0.md)
* **Rounds:** none yet

## Question

Whether the app keeps depending on n0's hosted iroh relays and address lookup (the default `N0` preset) or points at relay and lookup servers the author runs, or both, and what evidence from ticket 0004 the choice should wait for.

## Files

* `design_draft.md`
