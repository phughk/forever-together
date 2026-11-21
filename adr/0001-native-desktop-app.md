# 0001. Use a native Tauri desktop app instead of a browser-only page

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The first idea was a single self-contained HTML file with browser peer-to-peer networking (libp2p or iroh in the browser). Browsers cannot open UDP sockets or accept inbound connections, so they need relay or signaling servers, which the author does not want to host.

## Decision

Build a Tauri 2 desktop app (Rust backend, React and TypeScript web app), started from the author's template. It can use native UDP and QUIC. The addon only shows character data and does no networking.

## Consequences

The app has to be built and distributed for desktop platforms. Browser-only use is not supported.

## Alternatives considered

* Single HTML file with browser p2p (libp2p or iroh): rejected by author because browsers lack UDP and inbound connections and need relays.
* Hosted HTTPS page at a project domain using the phone camera: rejected by author because a static host cannot relay a handshake and an HTTPS page cannot call an `http://` LAN address.
* WebRTC between phone and desktop with public signaling brokers: not chosen by author as more machinery than needed.
