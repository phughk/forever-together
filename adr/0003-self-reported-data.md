# 0003. Character data is self-reported, with a typo check and no forgery protection

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The addon is open source Lua, so any signing key in it would be public. Anyone can build a valid-looking code for any character.

## Decision

Do not sign or authenticate the code. Include a short check value that catches mistyped or truncated pastes, and call it a typo check. Treat every field as a claim and label names as claimed by the other player.

## Consequences

Forged data is a known, accepted gap. The in-game invite is the real check. Per-peer rate limits reduce abuse.

## Alternatives considered

* Signed QR: rejected by author because the signing key would be public and a signature adds about 64 bytes.
