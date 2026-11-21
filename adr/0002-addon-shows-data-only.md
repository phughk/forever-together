# 0002. The addon only shows character data and does no networking or pairing

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The WoW addon cannot talk to the network, and `SavedVariables` are written only on `/reload` or logout, so a file watcher would see stale data.

## Decision

The addon (the standalone `P2PRDF` addon, see 0020) shows the character as a copyable text code and as a QR code of the same data. It has no networking, no pairing, no tokens and no authentication. The QR is in v1 because the phone's camera app scans it.

## Consequences

The player copies or scans the code each time the character or location changes. The code carries its creation time so the app can show its age.

## Alternatives considered

* Reading the `SavedVariables` file from the app: rejected, data is stale until `/reload`.
* Desktop app capturing the QR from the game window: not chosen, copy and paste makes it unneeded.
