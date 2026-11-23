# Direct peer-to-peer connections with n0 relays as fallback only

*   **Author:** Hugh Kaznowski
*   **Status:** Rewritten 2026-11-23 after the author's decisions below. Sections drafted: Overview, Problem, Goals, Non-Goals, open questions. Technical plan, alternatives and implementation not yet drafted. The first version of this file (keep or replace n0's services) and the second (relays built into the product) are in git history.
*   **Date:** 2026-11-23
*   **Ticket:** [ticket 0038](../../../tickets/0038-self-host-iroh-relay.md). Related: [ticket 0004](../../../tickets/0004-verify-iroh-live-network.md), [ticket 0010](../../../tickets/0010-trusted-peer-deployment-policy.md), [ticket 0021](../../../tickets/0021-topic-sharding.md).
*   **Related decisions:** [ADR 0008](../../adr/0008-iroh-and-local-storage.md), [ADR 0009](../../adr/0009-bootstrap-and-peer-exchange.md), [ADR 0023](../../adr/0023-no-supernodes.md), [ADR 0034](../../adr/0034-system-overview-goals-and-risks.md), [ADR 0040](../../adr/0040-membership-topic.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

A relay is a server that two peers use to reach each other when they cannot yet, or cannot at all, connect directly. The app uses iroh's default `N0` preset, so it uses n0's public relays and n0's DNS-based address lookup (`IrohConfig::Default` in `app/src/net/iroh_net.rs`, chosen in `runtime::build_network`).

n0's price list (saved by the author as a local file outside the repository) charges for relays only: concurrent connected endpoints, egress and rate limits. The free tier is public, rate limited, and has no uptime guarantee. The price list does not mention address lookup. Direct traffic between peers costs nothing. Our own run (2026-11-23, [spike results](../../spike-s0.md)) saw one TCP 443 connection to n0's EU relay on a machine whose peers could have connected directly, which fits every online endpoint holding a relay connection and counting as a concurrent connection.

**The author's decisions (2026-11-23):**

1.  **The project does not carry traffic between nodes.** Nodes must have direct peer-to-peer UDP connections. No node of ours, trusted or not, forwards other peers' packets.
2.  **n0's relays stay as they are, as a fallback, and the project does no work on relaying.** Peers that cannot connect directly (symmetric NAT, carrier-grade NAT, strict firewalls) solve their own networking problems, for example by forwarding a port. The author does not want to spend effort on carrying TCP or other relayed traffic.
3.  **Anything the project adds is run by a headless node that is explicitly set to do it.** Window installs never serve other peers. The two trusted peers will get IP addresses of their own; trusted1 is `89.40.6.71` for now.
4.  **The author's model of how direct connections would work:** a server captures the remote address of every node that connects to it, and gives that address to whoever asks; the asker then sends UDP to that address directly and gets answers directly.

### Is n0's address lookup free?

Not stated anywhere I could find. [n0's pricing page](https://www.iroh.computer/pricing) (the local copy the author saved) lists relays and metrics only. [The iroh docs on DNS address lookup](https://docs.iroh.computer/connecting/dns-address-lookup) say the N0 preset publishes to and resolves from the n0-hosted `dns.iroh.link`, and that if more capacity or uptime guarantees are needed one can run `iroh-dns-server` or contact n0 about hosted DNS. No rate limits, capacity or uptime numbers for the public service were found, and no price. So today it is free to use with no published limits and no guarantee. A change of terms would reach every install, because the preset is compiled in. The same docs describe self-hosting: build the endpoint from `presets::Minimal` and configure a pkarr publisher and resolver against your own `iroh-dns-server`. That server needs a public IP and DNS delegation for its zone (from the same docs; not verified here).

### How that model compares with what `iroh-relay` does

Checked against `iroh-relay` 1.5.0 source. Not run.

*   **The first half is what the relay's QUIC endpoint does.** A node opens a QUIC connection to it (ALPN `/iroh-qad/0`) and the server reports the address it saw the connection come from (`send_observed_address_reports` in `src/quic.rs`). A node learns its own public IP address and port, the way STUN works.
*   **The second half is not part of it.** The endpoint does not keep a table of nodes, and nobody can ask it for another node's address. It tells each connecting node its own address, nothing else. The "whoever asks gets the address" part is a lookup or rendezvous service. iroh has one as a separate service: nodes publish their addresses (including addresses found by that QUIC endpoint) to n0's DNS-based lookup, and a node that knows only a key resolves it.
*   **Direct UDP needs more than knowing the address.** Most home routers drop an incoming packet from an address the inside machine has not sent to first. If A sends to B's address and B has not sent to A's, B's router discards it. Both sides must send at about the same time. So whoever tells A about B must also tell B about A, and B must be listening for that message on a channel that is already open. In iroh today that channel is the connection each node keeps to its relay, which is how hole punching is coordinated. A router that picks a different outside port for each destination (a symmetric NAT) and carrier-grade NAT often defeat hole punching, and those peers cannot connect directly at all. Their only route is a relay carrying their traffic, which is where decision 2 (n0 as fallback) applies.

## The Problem

*   **What replaces the relay's coordination role.** With n0's relays kept as a fallback, hole punching keeps working as it does today for most peers: coordination travels over n0's relay connection. The project's own nodes add nothing there, and the dependency on n0's relays and DNS stays for first contact and coordination. If the aim is also to stop depending on n0 for those, the project needs its own introduction service, and the design must say what channel the target node listens on. This is open question 1.
*   **The project has no addresses to hand out.** `KnownPeers` stores peer ID, topics and two timestamps per peer and no addresses (`Record` in `app/src/net/known_peers.rs`), and `net::trusted::BUILTIN` stores keys only. The peer-exchange protocol (`app/src/net/exchange.rs`) exchanges peer IDs, not addresses. Any lookup the project provides has to store and pass on addresses.
*   **A server that learns addresses also learns who connects from where.** A rendezvous server sees every client's IP address and port and which key it belongs to. It cannot read traffic (iroh encrypts end to end), but it is a place where who-is-online-from-where is visible. Anyone can run a headless node, so clients must decide which of them they trust (open question 2).
*   **Certificates.** `iroh-relay`'s QUIC endpoint needs a TLS server configuration (`QuicConfig`; the server refuses to start without one), and the client must accept it. A server known only by an IP address cannot easily get a normal certificate. Whether Let's Encrypt issues one for a bare IP is not verified. The client can be told to trust a specific root or use its own verifier (`CaTlsConfig`).
*   **Relayed traffic is TCP.** The author prefers UDP because a lost TCP segment stalls everything behind it. In `iroh-relay` 1.5.0 the relay data protocol is a variant of Tailscale's DERP over WebSocket, so it is TCP, and the QUIC endpoint carries no data. Decision 1 keeps this out of our own nodes; it still applies to anyone who ends up on an n0 relay, which is outside our control.
*   **Abuse.** Any service that answers "where is X" can be asked about every key. `iroh-relay` has a per-client rate limit and an access-control hook, and the project's own peer exchange already limits requests (one per 10 s per peer).

## Goals

*   Peers connect directly over UDP whenever the network allows it, and no node of ours carries other peers' traffic.
*   Peers that cannot connect directly are not the project's problem to solve. n0's relays stay in the default configuration; the project neither builds relaying nor promises it works.
*   The two trusted peers, once they have their own IP addresses, are reachable at known addresses by new installs.
*   A headless node can be started in a role that serves the project's own address or introduction service, only when it is explicitly set to; window installs cannot.
*   The ADRs say that n0's relays and, if kept, its address lookup are accepted dependencies, with the evidence for it ([ADR 0023](../../adr/0023-no-supernodes.md) amended or cited).

## Non-Goals

*   Replacing iroh, or writing a relay protocol of our own. Decision 1 rules out a UDP relay of ours.
*   Hiding peers' IP addresses from each other or from a rendezvous server.
*   Making peers behind symmetric NATs or carrier-grade NAT connect. They use n0's relays if those work, or fix their own network.
*   Running the live tests of ticket 0004 (they remain the evidence source).

## Open questions raised so far

Decided in later sections of the draft, one at a time. Answered so far: n0's relays stay as they are and the project does no relaying work; our nodes do not carry traffic; serving any new role is opt-in on headless nodes.

1.  **Does the project run its own introduction service at all?** That is, an address lookup plus a channel to tell a node who is about to contact it, so that direct connections no longer depend on n0's relay for coordination. Or does coordination stay with n0 and the project only adds addresses to peer exchange?
2.  **How clients learn addresses and which servers they trust**, now that anyone can run a headless node.
3.  **What first contact with a trusted peer looks like** when its IP address is known: dial the address directly (no lookup needed for it), or still by key.
4.  **Whether to keep n0's address lookup service** alongside any lookup of our own, or only as the fallback.
