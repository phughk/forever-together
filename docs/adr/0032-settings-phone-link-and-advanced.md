# 0032. Settings group phone access in one card, put expert settings behind Advanced, and lock them on a phone

* **Status:** Accepted (decided and implemented 2026-11-21). Extends [0012](0012-one-web-app-for-phone-and-desktop.md) and [0028](0028-the-desktop-app-installs-the-bundled-addon.md) (card placement).
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski (decisions D3, D4, D8, D9)

## Decision

* **One "Phone link" card** replaces separate LAN mode, token and QR cards. It carries one sentence about plain HTTP. The pairing QR opens as a sheet so the page never scrolls and Done stays visible. Token reset reads "sign out all phones".
* **Card order:** Phone link, Addon, then Advanced. Settings is a card grid.
* **Advanced section** holds players to remember (known peers), backup contacts (trusted peers) and search history controls. Tabs are used only if Settings still scrolls at 1280x720 after this (it does not).
* **Phone is read-only for Advanced (D3).** Controls only the desktop may change show a lock icon. Today a phone can change only `known_peers_size`. The `remote` flag is guessed from the hostname, so a lock may briefly be wrong until the server answers 403.
* **A quiet "Report a problem" link** sits in the Settings footer (`ISSUES_URL`).
* **Out of scope (D9):** per-phone count, an auto-off timer, an issue form. (The installer was out of scope in the audit and was then built, see 0028.)

## Consequences

* Not built, tracked as tickets: per-phone count, auto-off timer.
* The remote-flag guess is a known weakness.
