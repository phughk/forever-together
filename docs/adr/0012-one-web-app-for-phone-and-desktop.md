# 0012. One web app served by the desktop app to its own window and to phones, with a token for phones

* **Status:** Accepted (revises two earlier decisions: no user theming, no tokens)
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The phone is a critical second client. The phone's native camera app scans the addon QR, and the player pastes the text into the web app. An HTTPS page cannot call a LAN `http://` address, so the page must be served by the desktop app itself.

## Decision

The desktop app embeds one statically built web app and serves it over HTTP, on localhost by default and on the local network when LAN mode is on. The desktop window loads the same web app, and both use one REST and server-sent-events API. If the environment variable `P2PRDF_WEB_DIR` names a folder, its files take priority over the embedded ones, so users can change themes and layout. A per-install token approves a phone: the desktop shows a pairing QR with the LAN URL and the token in the URL fragment, the web app stores the token and sends it with every call, and resetting the token un-pairs every phone. The server also checks Host and Origin, sends no cross-origin permissions, and never returns the raw code. The addon and the p2p network use no tokens.

## Consequences

The token travels over plain HTTP on the LAN, which the app states as a risk on untrusted networks. A custom web app has the same access as the embedded one. The environment variable name, server-sent events and the fragment-based token handover are the draft's choices and were accepted by the author on 2026-11-21 (decision D8). The same applies to these implementation details from the Detailed Implementation Plan: live updates are read with `fetch` streaming so the token travels in a header, the pairing and token-reset routes and the LAN-mode and trusted-peer settings accept only loopback requests so a phone cannot change them, and the desktop window loads the local URL with no Tauri IPC capabilities.

## Alternatives considered

* No token (decided earlier by the author, reversed later): reversed by the author.
* User theming and overridable files out of scope (decided earlier, reversed later): reversed by the author.
* Per-session secret in the URL: replaced by the per-install token.
