# 0041. Address lookup through n0 DNS and the DHT, direct addresses published, and a rate-limited hole-punching relay in headless nodes

* **Status:** Accepted (decided by the author 2026-11-23; not built, tickets 0048 and 0049). Extends [0008](0008-iroh-and-local-storage.md), [0009](0009-bootstrap-and-peer-exchange.md) and [0023](0023-no-supernodes.md).
* **Date:** 2026-11-23
* **Deciders:** Hugh Kaznowski
* **Source:** [EGM iteration 0038](../egm/0038-self-host-iroh-relay/design_draft.md) (draft, decisions and rejected alternatives), ticket [0038](../../tickets/0038-self-host-iroh-relay.md).

## Context

The app used iroh's `N0` preset: n0's relays, and n0's DNS for address lookup. n0's price list charges for relays only (concurrent endpoints, egress, rate limits); its DNS documentation says the public lookup servers are free, rate-limited and have no uptime guarantee. By default a lookup record holds a peer's home relay URL and no IP address, so every first connection starts on a relay. The author does not want the project to carry other peers' traffic, and wants the network to keep working if n0's DNS or relays change. In iroh 1.5.0 hole punching is done by QUIC NAT-traversal frames inside a QUIC connection that is first carried by a relay, so a relay that helps hole punching also carries the first packets of every connection; iroh has no signalling-only relay mode. (Source reading of `iroh` and `iroh-relay` 1.5.0; none of it has been run.)

## Decision

1. **Address lookup is n0's public DNS plus the BitTorrent Mainline DHT**, both on every node and every dial, in every build (crate `iroh-mainline-address-lookup`). The project runs no lookup or rendezvous server.
2. **Records publish direct IP addresses as well as the relay URL** (`AddrFilter::unfiltered()` on the DNS publisher and the DHT lookup). Anyone who knows a peer ID can read that peer's IP addresses from DNS or the DHT. The author accepts this.
3. **The endpoint is built from `presets::Minimal`,** with n0's relays kept as they are and the project's relays added to the relay map. The `N0` preset is not used, because it already holds a relay-only publisher.
4. **A headless node can serve as a hole-punching relay,** only when started with `--headless --relay <addr:port>`. Window installs never serve. The server is `iroh-relay`'s embedded `Server` with plain HTTP (no TLS), no QUIC endpoint, and a low per-client rate limit (`Limits.client_rx`) so handshakes and NAT traversal pass and sustained traffic starves. The rate is chosen from a measurement (ticket 0049).
5. **Clients learn the project's relays from a compiled-in list** of plain `http://` URLs (an IP and port) next to `net::trusted::BUILTIN`. `BUILTIN` stays keys only: first contact with a trusted peer is by key through the lookup.
6. **The project does no other relaying.** No UDP relay of its own, no patched `iroh-relay`. Peers that cannot connect directly, and cannot use n0's relays, fix their own networking.

## Consequences

* New installs find trusted peers by key even if n0's DNS is down (the DHT), and public peers can be dialed by address without a relay.
* **Risks to add to [ADR 0034](0034-system-overview-goals-and-risks.md)** when it is built: IP addresses of every peer are readable from DNS and the DHT by anyone with the peer ID, and peer IDs travel in gossip to every swarm member; the relay connection is plain HTTP and WebSocket, so an observer on its path sees which endpoint IDs connect; a rate limit that lets the handshake through also lets some traffic through; the project still depends on n0's relays and DNS for peers that need them.
* Unknowns this ADR depends on, measured in ticket 0049: the lowest rate at which connections still complete and migrate to a direct path; whether iroh waits on the DHT before using DNS's answer; whether the `iroh-mainline-address-lookup` 0.6.x crate builds against `iroh` 1.5.0; whether the `iroh-relay` client connects over an `http` URL outside tests.
* The `iroh-relay` `server` feature adds crates (an ACME client, `clap`, `toml`, `tracing-subscriber`); build size and time grow.
* ADR 0023 stands: a headless relay node carries only what the rate limit allows and learns no more than any relay (endpoint IDs and IP addresses), but it is a role the network now has, run by whoever starts it with the flag.
