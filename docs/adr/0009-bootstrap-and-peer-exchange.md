# 0009. Bootstrap through remembered peers, peer exchange, then trusted peers, with an LRU known-peers table

* **Status:** Accepted (supersedes the DHT noticeboard)
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

Nothing should be hosted by the author, the trusted peer must not be overloaded, and iroh finds addresses only for a known peer ID.

## Decision

A peer tries, in order: remembered peers, then peer exchange with connected peers, then trusted peers, only when the earlier steps fail. The DHT is used only through iroh's address lookup to find trusted peers from their built-in public keys, which a setting can extend. Each app keeps a known-peers table (peer ID, claimed topics, last seen, last used) as a least-recently-used cache. The maximum size is a setting, 100 by default. Peers dial listed entries before trusting them. Trusted peers are the same app running headless with a flag. They back off with jitter, and trusted peers limit their own connections and rates.

## Consequences

A topic with few peers is found only if someone asked already knows one. The first trusted peer is run by the author. Peer tables are claims and can be abused, so entries are verified and capped.

## Alternatives considered

* DHT noticeboard (topic-derived BEP 44 slots holding peer IDs): chosen at first, then replaced by the author because anyone can overwrite the slots and it was untested.
* Trusted peers publishing per-topic lists on the DHT: considered, then replaced by plain peer exchange.
* libp2p provider records on the public IPFS DHT: not chosen.
* Remembered peers plus introducers only: close to the final design.
