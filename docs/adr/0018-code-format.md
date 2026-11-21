# 0018. The character code is a versioned binary record in base45 text with shared test vectors

* **Status:** Accepted (2026-11-21, by the author; decision D8). Details marked provisional in the spec depend on spike S0.
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The code must be small enough to scan as a QR and to paste, and both the Lua encoder and the Rust decoder must agree on it.

## Decision

A fixed-layout binary record with a version byte, encoded as base45 text with a short prefix such as `P2R1:`. It uses numeric IDs and carries names only where the client gives only names (first name and surname as two separate fields, the realm value and the zone for now). There is no dungeon field, because players search from outside a dungeon. A specification file and test vectors under `docs/` that the addon tests and the Rust tests both read. An unknown version is refused with an update message.

## Consequences

The size is roughly 45 bytes including the name and zone text. QR scan reliability from a game window is unverified.

## Alternatives considered

* Plain JSON in the QR: not chosen, too large.
