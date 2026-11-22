# 0004-verify-iroh-live-network: Verify iroh over the real network

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0004](../../../tickets/0004-verify-iroh-live-network.md)
* **ADRs:** [0008](../../adr/0008-iroh-and-local-storage.md), [0009](../../adr/0009-bootstrap-and-peer-exchange.md), [0019](../../adr/0019-signed-gossip-messages.md), [0023](../../adr/0023-no-supernodes.md)
* **Rounds:** none yet

## Question

How to run the app on two machines across the internet to check address lookup through n0's hosted DNS and relays, the bootstrap order, gossip author handling and the hosted components iroh uses, with exact pass and fail criteria. The run needs a way to see what the network is doing, which the app currently lacks outside the status line.

## Files

* `design_draft.md`
