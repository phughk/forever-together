# Key management for forever-together

*   **Author:** Hugh Kaznowski
*   **Status:** Draft
*   **Date:** 2026-11-21
*   **Ticket:** [0043](../../../tickets/0043-key-management.md), which also covers 0003, 0008, 0009 and 0044.
*   **Related decisions:** [ADR 0008](../../adr/0008-iroh-and-local-storage.md), [0023](../../adr/0023-no-supernodes.md), [0038](../../adr/0038-scan-log-and-opt-in-export.md), [0039](../../adr/0039-key-management.md).

--------------------------------------------------------------------------------

## Overview

Every install has one permanent ed25519 secret key. The app creates it on first start, keeps it in the local store (`KEY_IDENTITY_SECRET`) and uses it twice: its public half is the peer ID other peers see, and the same bytes are the iroh endpoint key (`app/src/identity.rs`, `app/src/net/iroh_net.rs`).

This design adds what is missing around that key: commands to back it up, restore it and replace it; a trusted-peer key list that can actually be filled in; a way for a node on a cloud machine to start with a chosen key; and encryption of the exported scan log to the first trusted peer's key, so players can post it in a public GitHub issue.

## The Problem

Today the key is created and used, and nothing else can be done with it.

*   There is no backup. If the author's trusted node loses its data directory, its key is gone. A new key is a new peer ID, and the built-in trusted peer list lives in the binary, so a new key needs a new release.
*   There is no way to start a node with a chosen key, so a cloud node cannot be rebuilt with the same identity, and `net::trusted::BUILTIN` cannot be filled in with a key that is still around next month.
*   There is no reset. A leaked or compromised key can only be replaced by deleting files by hand.
*   The scan export in ADR 0038 needs a public key to encrypt to, and a private key the author can keep offline to decrypt with. The first `BUILTIN` entry is the chosen recipient, and it is empty.
*   How well the store protects the key file (permissions on each operating system) was never checked (ticket 0003), and exporting the key makes that check matter more.

Five tickets describe parts of this (0003, 0008, 0009, 0043, 0044). They overlap, and several cannot be finished before others. This design treats them as one subsystem.

## Goals

*   **Export** the install's secret key to a file from the command line, for backup and deployment.
*   **Import** a secret key from a file, so a node starts with a chosen identity and prints the same public key every time.
*   **Reset** the key: discard it and generate a new one, with a clear warning of what changes.
*   The secret key is never reachable from the web app, the HTTP API, logs or standard output by default.
*   A key file is written and read with owner-only permissions, refusing anything weaker, on macOS, Linux and Windows (Windows may be ticketed if its model needs its own design).
*   `BUILTIN` can be filled in with the author's trusted key, with a written process for keeping its private half backed up offline.
*   The scan-log export (ADR 0038) is encrypted to `BUILTIN[0]`, and the author can decrypt it from the command line with the matching private key.
*   The five tickets collapse into one tracked piece of work, with the small items that remain as their own tickets.

## Non-Goals

*   Passphrase-protecting the key in the store, or using the operating system keychain, in this pass. Ticket 0003 asks whether the `varaq` store is good enough; the keychain stays the fallback named in ADR 0008.
*   Rotating the trusted key without a release. `BUILTIN` is compiled in; changing it is a release.
*   Any key operation from the web app or over HTTP.
*   Multi-key identities, delegated or sub-keys, or key revocation lists.
*   Signing releases or addon files.
*   A hosted key service of any kind (nothing is hosted, ADR 0034).

## Open Questions

*   **Reuse the trusted peer's iroh key for decrypting exports, or ship a separate intake key?** The iroh key is a signing key. Using it for encryption works through a standard conversion but mixes two uses of one key. To be decided by the author (Technical Plan).
*   **Key file format**: raw bytes, hex, or a self-describing text format; and whether a passphrase is allowed.
*   **Command names and where they live**: subcommands of the app binary or flags next to `--print-public-key`.
*   **How a cloud node receives its key**: an import step at deploy time, a path flag, or an environment variable.
*   **Windows**: what "owner-only" means and how the check works.
*   **What happens to remembered peers and history on reset**: the design assumes they stay (they hold no secrets), to be confirmed.
