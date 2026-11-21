# 0008. Use iroh and iroh-gossip, rustls, varaq storage and permanent keys

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The app needs direct peer connections with NAT traversal, a gossip swarm per topic, and local storage.

## Decision

Use iroh for connections and iroh-gossip for one swarm per topic. TLS uses rustls (iroh's QUIC is built on it, to be verified). Store the identity, known peers, settings and API token with the `varaq` crate (version 0.1.1 on crates.io, young). Each install creates its iroh secret key once and keeps it. The public keys of the trusted peers are built into the app.

## Consequences

`varaq` maturity is unverified. If it is not good enough for the secret key, the key moves to the operating system keychain. A permanent key lets other peers recognise the same player across sessions.

## Alternatives considered

* libp2p Kademlia with gossipsub: considered, a second stack was not wanted.
* Mainline DHT only: considered for first contact, see the bootstrap decision.
