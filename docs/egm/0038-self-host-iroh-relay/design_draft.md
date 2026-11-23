# Relays built into the product

*   **Author:** Hugh Kaznowski
*   **Status:** Rewritten 2026-11-23 after the author's decision to build relays into the product. Sections so far: Overview, Problem, Goals, Non-Goals. Technical plan, alternatives and implementation not yet drafted. The earlier version of this file (a keep-or-replace decision between three options) is in git history.
*   **Date:** 2026-11-23
*   **Ticket:** [ticket 0038](../../../tickets/0038-self-host-iroh-relay.md). Related: [ticket 0004](../../../tickets/0004-verify-iroh-live-network.md), [ticket 0010](../../../tickets/0010-trusted-peer-deployment-policy.md), [ticket 0021](../../../tickets/0021-topic-sharding.md).
*   **Related decisions:** [ADR 0008](../../adr/0008-iroh-and-local-storage.md), [ADR 0009](../../adr/0009-bootstrap-and-peer-exchange.md), [ADR 0023](../../adr/0023-no-supernodes.md), [ADR 0034](../../adr/0034-system-overview-goals-and-risks.md), [ADR 0040](../../adr/0040-membership-topic.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

A relay is a server that two peers use to reach each other when they cannot yet, or cannot at all, connect directly. Peers behind strict NATs stay on a relay for the whole connection; every connection starts on one. The app uses iroh's default `N0` preset, so it uses n0's public relays and n0's DNS-based address lookup (`IrohConfig::Default` in `app/src/net/iroh_net.rs`, chosen in `runtime::build_network`).

n0's price list ([Pricing - iroh](https://www.iroh.computer/pricing), saved by the author as a local file, not stored in the repository) charges for relays only: concurrent connected endpoints, egress, and rate limits. The free tier is public, rate limited, and has no uptime guarantee. The same price list does not mention address lookup. Traffic that goes directly between peers costs nothing. Our own run (2026-11-23, [spike results](../../spike-s0.md)) saw one TCP 443 connection to n0's EU relay on a machine whose peers could have connected directly, which fits the reading that every online endpoint holds a relay connection and is counted as a concurrent connection.

**The author's decisions (2026-11-23):** relay capability is built into the product, so the network does not depend on n0's relays. A node serves as a relay **only when it is run with `--headless` and relay serving is specified**; window installs never serve. The two trusted peers will get IP addresses of their own; trusted1 is `89.40.6.71` for now.

The iroh relay is a library, not only a separate program. `iroh-relay` 1.5.0 has a `server` feature that exposes a `Server` taking a `ServerConfig` (an HTTP or HTTPS relay endpoint, an optional QUIC endpoint, per-client rate limits, access control). iroh's `RelayMode::Custom(RelayMap)` takes a list of relays, and a `RelayMap` can be changed while the endpoint runs (`insert` and `remove`). These facts come from reading the crate source (`~/.cargo/registry`, versions in `app/Cargo.lock`), not from running it.

## The Problem

The first idea was that every replica is a relay. That collides with three facts. The author narrowed it to opt-in headless nodes (see Overview), which removes the first one for window installs; the other two remain.

*   **A relay must accept connections from outside.** A relay is a server: other peers open a connection to its address. Most players' computers sit behind home routers (NAT) with no inbound route, so they cannot serve. Only a node with a reachable public address can be a relay. That is true of the trusted peers once they have their own IP addresses, and of any other operator who runs a headless node on a public address. It does not apply to window installs, which never serve.
*   **Relays need an address that other nodes know.** iroh asks for a relay URL, and the relay's address must be known before a connection is made, because the relay is the thing that makes the connection possible. Today `KnownPeers` stores peer ID, topics and two timestamps per peer and no addresses (`Record` in `app/src/net/known_peers.rs`), and `net::trusted::BUILTIN` stores keys only. A relay URL, or an IP and port, has to be stored, found, and passed on somewhere.
*   **A relay needs a TLS certificate that clients accept.** The relay's HTTPS endpoint presents a certificate. `iroh-relay` can get one from Let's Encrypt for a domain name (`CertConfig::LetsEncrypt`) or use one supplied by the operator (`CertConfig::Manual`), and it can also serve plain HTTP with no TLS (`RelayConfig::tls = None`). The client trusts a fixed set of roots by default and can be told to trust specific extra roots or use its own verifier (`CaTlsConfig`). A server known only by an IP address cannot easily get a normal certificate. Whether Let's Encrypt can issue one for a bare IP is not verified here.

Other effects the design must account for:

*   **Metadata.** A relay is a server that every client connects to, so it sees the IP address and port of every client, which endpoint ID each one is, and the timing and size of its packets. It cannot read the traffic (iroh encrypts end to end), so it is not a gossip supernode, but it is a place where who-talks-to-whom can be observed. Anyone can run a headless relay, so clients must decide which relays they use (open question 2).
*   **Abuse.** A relay carries traffic for strangers. Without limits, one peer can use another's bandwidth. `iroh-relay` has a per-client rate limit and an access-control hook.
*   **Bandwidth and trust.** A relay uses its operator's upload bandwidth and exposes the operator's IP address to every connecting peer.
*   **A relay knows addresses, but does not share them as a lookup.** It knows which endpoint ID is connected to it and from where. It forwards packets between connected clients; it does not give one client another's IP address (the QUIC address-discovery endpoint tells a client its own address only). Peers swap direct addresses over the relayed path while they try to hole-punch. So a node can reach a peer through a relay only if it knows that peer's relay URL. Relays therefore shrink the lookup problem but do not remove it. With the trusted peers' IP addresses known, the app can dial them by address instead of by key, which needs no lookup for first contact. For every other peer the app still dials by key. This draft covers relays; the lookup question is open question 4.
*   **Gossip is already a form of relaying.** Messages hop between swarm neighbours (ADR 0040 adds a swarm every node is in). That carries application messages but not connection setup, so it does not replace a relay.

## Goals

*   The network keeps working with no dependence on n0's relays.
*   Relay capability ships in the app: the same binary. It is off by default and turned on only for a headless node by an explicit setting. A window install has no way to serve.
*   The two trusted peers act as relays at their own addresses, reachable at a known address with a certificate the app accepts.
*   Other operators can run a relay by starting a headless node with the same setting.
*   Clients find relays without a release for every change (open question 2).
*   A relay's bandwidth, connection count and per-client rate are capped.
*   An ADR amends [ADR 0023](../../adr/0023-no-supernodes.md): relays are a hosted role the network now has, and who may run one.

## Non-Goals

*   Replacing iroh, or writing a relay protocol of our own.
*   Hiding peers' IP addresses from each other or from relays (a relay sees them).
*   Paying for or contracting n0's hosted relays; they stay available as a fallback only if the author chooses (open question 3).
*   The address lookup service, unless open question 4 brings it in.
*   Running the live tests of ticket 0004 (they remain the evidence source).

## Relayed traffic is TCP, not UDP

The author wants relayed traffic to go over UDP, on the belief that it is more reliable because it is not tied to one TCP stream. Checked against `iroh-relay` 1.5.0 source:

*   The relay data protocol is a variant of Tailscale's DERP carried in WebSocket binary frames after an HTTP upgrade (`app` crate dependency tree: `iroh-relay` `protos::relay` and `client`). WebSocket runs on TCP, so all of a client's relayed packets share one TCP stream to its relay.
*   The relay server's QUIC endpoint (`QuicConfig`, port 7842 by default) is for address discovery only. It does not carry relayed data.
*   iroh's own traffic between two peers is QUIC, which runs over UDP. A relayed connection therefore puts QUIC datagrams inside a TCP stream. One lost TCP segment stalls every packet behind it until it is resent, including packets of unrelated connections that share the same relay stream. QUIC's own recovery works above that, so loss is handled twice. The author's concern is correct for the case where traffic stays on the relay.
*   Direct paths use UDP and are not affected. The relay is used only until a direct path is punched, and for peers whose NAT prevents one.
*   iroh 1.5.0 has an `unstable-custom-transports` feature (`CustomTransport` in `src/socket/transports/custom.rs`). It is a way to add a transport such as a UDP relay without changing iroh, but it is marked unstable, and the relay protocol and server would be written by this project. Not verified: how much of the transport contract (addressing, path selection) a relay needs.

So `iroh-relay` cannot be switched to UDP by configuration. See open question 5.

## Open questions raised so far

These are decided in later sections of the draft, one at a time. They are listed here so the Goals above are read as conditional on them.

1.  ~~Which replicas act as relays.~~ Answered 2026-11-23: headless nodes only, and only when relay serving is specified. How it is specified (flags, a settings file) is a detail for the Technical Plan.
2.  How clients learn relay addresses, and which relays they trust now that anyone can run one.
3.  Whether n0's relays stay as a fallback.
4.  What replaces address lookup for dialing by key.
5.  Whether relayed traffic stays on the TCP relay protocol, or the project builds a UDP relay.
