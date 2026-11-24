# 0049. Measure the relay rate limit and the DHT lookup

* **Status:** in progress (one-machine measurements done 2026-11-24, see the spike results; two-network and real-NAT runs open)
* **Kind:** verification
* **Priority:** P1
* **Blocked by:** tickets 4
* **Source:** ADR 0041
* **ADRs:** [ADR 0041](../docs/adr/0041-address-lookup-and-hole-punching-relay.md)
* **EGM:** [0038-self-host-iroh-relay](../docs/egm/0038-self-host-iroh-relay/design_draft.md)

## What

Measure what ADR 0041 depends on: (1) the lowest per-client rate at which two endpoints behind different NATs still connect and migrate to a direct path, and the throughput a pair gets when it cannot punch through; (2) first-contact time with DNS only, DHT only and both, and whether iroh waits on the DHT; (3) that `iroh-mainline-address-lookup` 0.6.x builds against `iroh` 1.5.0; (4) that the `iroh-relay` client connects over an `http` URL outside tests.

## Done when

The numbers are in `docs/spike-s0.md` and the relay rate constant is set from them.

## Done so far

All four items were measured or checked on one machine (spike results, "Relay rate limit, DHT lookup and build checks"). Open: the rate on real NATs (does a direct path still form at 2 kB/s with an 8 kB burst between two home networks?), DHT lookup time from a second network, and running the relay on `89.40.6.71`. The relay now runs on `89.40.6.71:3340` and answers. Still needed: two machines behind different home NATs.
