# 0017. The addon is an rdf/ module inside the existing Social Hub addon

* **Status:** Superseded by 0020
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

This repository already holds the Social Hub addon with nav, trade and mute modules and one window with sidebar pages.

## Decision

Add a module under `social/rdf/` with `Core.lua`, `Codec.lua`, `Qr.lua` and a page in the sidebar window, following the repository's conventions. No new saved variables in v1. The QR encoder is written in plain Lua because no QR library is installed.

## Consequences

The code specification and its test vectors are shared with the app repository, with a check that both copies match.

## Alternatives considered

* A separate addon: not chosen.
