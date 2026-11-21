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
*   A key file is a standard key that other software, such as ssh, can read. The app creates it owner-only on Unix and does not check permissions when reading one (decided by the author).
*   `BUILTIN` can be filled in with the author's trusted key, with a written process for keeping its private half backed up offline.
*   The scan-log export (ADR 0038) is encrypted to `BUILTIN[0]`, and the author decrypts it with the standard `age` tool and the exported key. The app has no decrypt command.
*   The five tickets collapse into one tracked piece of work, with the small items that remain as their own tickets.

## Non-Goals

*   Passphrase-protecting the key in the store or in an exported file, or using the operating system keychain. Decided by the author: the key is a plain key, and securing the file is the user's job. The keychain stays the fallback named in ADR 0008 if `varaq` proves unfit (ticket 0003).
*   Rotating the trusted key without a release. `BUILTIN` is compiled in; changing it is a release.
*   Any key operation from the web app or over HTTP.
*   Multi-key identities, delegated or sub-keys, or key revocation lists.
*   Signing releases or addon files.
*   A hosted key service of any kind (nothing is hosted, ADR 0034).

## Open Questions

*   **Does a second process get refused while the app runs?** The commands change the stored key, so they must not run against a data directory a running app has open. Whether `varaq` refuses a second opener is not verified.
*   **Which crate reads and writes the OpenSSH key file**: `ssh-key` (RustCrypto) or the parser inside `age`. Neither is in `app/Cargo.toml` today (only `ed25519-dalek` is).
*   **Is the `age` crate good enough?** It is marked beta, and its `ssh` feature pulls in `rsa` and other ciphers the app does not need.
*   **Does `key export` also write a `.pub` file** (an OpenSSH public key line) next to the private file?
*   **Windows**: no permission step; the file gets the directory's default access rules. To be confirmed with a test on Windows (ticket).

## Decisions by the author (2026-11-21)

1.  **The key file is a plain key.** Export writes what is in the store as a standard, unencrypted key file that ssh and similar tools can read. No passphrase, no encryption layer. The user decides how to protect the file. The app does not treat this as a security risk to solve.
2.  **The scan-log export is encrypted to the public half of `BUILTIN[0]`.** The author owns that trusted peer, holds its secret, and decrypts. Players expose no characters or codes. Rejected: a separate decryption key, or sealing to two keys.
3.  **The command line gets a `key` subcommand group** (`key export`, `key import`, `key reset`, `key show`). `--print-public-key` stays as an alias for `key show`. Running with no subcommand still opens the window. Rejected: top-level flags.
4.  **Headless is the same binary and the same command line.** It generates a key at first start and takes the same `key` commands. A cloud node gets its key by `key import` before its first start. Rejected: a key file path or environment variable read at every start.

## Technical Plan

### What changes

The key itself does not change. It stays 32 bytes under `KEY_IDENTITY_SECRET` in the store, as `app/src/identity.rs` keeps it today. New code reads and writes it in a standard file format and exposes it through the command line.

```mermaid
flowchart TD
    subgraph CLI
        S[key show]
        E[key export]
        I[key import]
        R[key reset]
    end
    ST[(Store: identity secret)]
    F[OpenSSH ed25519 key file]
    S --> ST
    E --> ST --> F
    I --> F --> ST
    R --> ST
    PL[Player: scan log] -->|age, to BUILTIN 0| X[.age file on a GitHub issue]
    X -->|age -d -i| A[Author, with the exported key file]
    F --> A
```

### Commands

*   **`key show`**: prints the peer ID (hex, the form used in `trusted_peers` and `BUILTIN`) and the OpenSSH public key line. Same output as today's `--print-public-key`, which stays as an alias.
*   **`key export <file>`**: writes the stored secret as an OpenSSH ed25519 private key file, unencrypted. Created owner-only on Unix. Refuses to overwrite an existing file. Prints the peer ID so the operator can note which key it is. `--public` also writes `<file>.pub` (open question).
*   **`key import <file>`**: reads an unencrypted OpenSSH ed25519 private key. Parses and checks it fully before touching the store, then replaces the stored secret in one write. Refuses to replace an existing key unless `--force`. Rejects an encrypted file or another key type with a plain message. Prints the old and new peer IDs.
*   **`key reset`**: discards the stored secret and generates a new one. Asks for confirmation unless `--yes`. Prints old and new peer IDs and one warning: other nodes see a new peer, and a changed trusted-peer key needs a release.
*   **Where they live**: a `Key` subcommand in `app/src/cli.rs`; `Mode` gains a variant that carries it; `runtime` dispatches before any network or window start-up, and exits without starting the app.
*   **Running app**: the commands open the store directly. If another process holds it they must fail with a clear message. Whether `varaq` guarantees that is unverified (open question); until verified the commands print a warning not to run against a live data directory.

### Key file format

OpenSSH private key (`-----BEGIN OPENSSH PRIVATE KEY-----`), ed25519, unencrypted. It is a standard file, so the same file works for `ssh-keygen -y -f`, for `age -d -i`, and for a key backup in any password manager. The public half is derivable. An iroh endpoint key and an ssh ed25519 key are both ed25519, so the 32 secret bytes convert without any change to the stored format or to the peer ID.

### Scan-log encryption (ticket 0044)

*   The export module takes `BUILTIN[0]` (32 bytes), forms an `age` ssh-ed25519 recipient from it, and encrypts the JSON. The `age` ssh-ed25519 recipient converts the Ed25519 key to X25519 internally, so no custom cryptography is written in the project.
*   Output is `age` ASCII armor, so it can be pasted into an issue as text or attached as a file: `scans-<date>.wowft-scans.age`. The `age` header already carries a short key tag, so the author can tell which key to use after a rotation. This replaces the custom header sketched in the ADR 0038 amendment.
*   Only `BUILTIN[0]` is ever a recipient. If `BUILTIN` is empty, export is disabled and the card says why.
*   The app never holds a decrypting identity for exports. The author decrypts outside the app with `age -d -i <exported key file> <file>` (or `rage`). A test in the app encrypts with the real recipient code and decrypts through the `age` library with an identity loaded from the same OpenSSH file that `key export` writes, so the workflow is proven without the app shipping a decrypt command.

### BUILTIN and the trusted key

*   **Filling it in**: on the always-on node, `key show`, paste the peer ID into `net::trusted::BUILTIN` (ticket 0008).
*   **Backup**: `key export` from that node, stored offline by the author. This is a written process in `README.md`, not code.
*   **Losing the key**: a new key, a new `BUILTIN` entry and a release, and old exports can no longer be read. The process says so.

### Permissions (ticket 0003)

The app sets owner-only on files it creates (Unix). It does not verify permissions when reading a key file, and it does not change the permissions of the store files. Ticket 0003 shrinks to recording what the `varaq` store's file mode is on each operating system, so the author knows what exists, not to enforce anything.

### Tickets after this design

*   0043 stays as the umbrella and is implemented as: CLI restructure, identity export and import, reset, docs.
*   0009 is closed into 0043 (delivery is `key import`).
*   0044 depends on 0008 (a `BUILTIN` key) and on the export module in 0040.
*   0003 becomes a documentation and verification ticket.
*   New small tickets: Windows check, running-app guard.

## Alternatives Considered

Each is something the author declined on 2026-11-21 or I considered and left out. "Rejected by author" gives the author's reason where they gave one; the costs listed under it are the trade-offs I put to them.

### Encrypting the scan export to a separate intake key

A second keypair, secret kept offline, public half compiled in beside `BUILTIN`.

*   **Rejected by author:** the author owns `BUILTIN[0]`, holds its secret, and can decrypt with it, so a second key adds nothing they need. Players still expose no characters or codes.
*   **Cost accepted:** the decrypting secret is also the live trusted node's identity key. If that server is compromised, the attacker can read every export and impersonate the trusted peer. Resetting the key makes old exports unreadable unless the old secret is kept. The same key now does signing and, through the Ed25519 to X25519 conversion that `age` performs, encryption. The conversion is standard and `age` supports it, but it is two uses of one key.

### Sealing each export to two keys

Each file carries two sealed copies of one random file key, so either secret opens it.

*   **Rejected by author:** not needed once the first alternative was declined.
*   **Why it is weaker for this case:** compromising either key exposes every file, and the file format grows. Its one benefit, surviving the loss of one key, is covered by the offline backup of the single key.

### Plain export of the scan log

Players post the unencrypted JSON.

*   **Rejected by author:** players would expose their characters and codes in a public issue.

### Passphrase-protected or keychain-stored keys

An encrypted key file, a passphrase prompt, or the operating system keychain for the stored secret.

*   **Rejected by author:** a key is a key. Export writes what is in the store, in a format other software can read, and the user secures the file. The author does not treat this as a risk for the app to solve.
*   **Cost accepted:** anyone who can read the data directory or an exported file has the identity. The keychain stays the fallback in ADR 0008 only if `varaq` proves unfit (ticket 0003).

### Top-level flags instead of a `key` subcommand group

`--export-key`, `--import-key`, `--reset-key`, `--decrypt-scans` beside `--print-public-key`.

*   **Rejected by author:** chose the subcommand group.
*   **Why it is weaker:** flags can be combined with each other and with `--headless`, so more invalid combinations need rejecting, and each new operation adds another top-level flag.

### A key file path or environment variable on every start

`--key-file` or `WOWFT_KEY_FILE`, read each time the app starts, with or without the store as the primary.

*   **Rejected by author:** headless is the same binary and CLI as the desktop app. It generates a key at start and takes `key import` and `key export`, so a deploy step covers cloud nodes.
*   **Cost accepted:** a container with an ephemeral data volume that skips the import step starts with a new random key and a new peer ID. The deployer has to run the import.

### A decrypt command in the CLI

`key decrypt <file> --key <keyfile>`.

*   **Rejected by author:** `age -d -i <exported key>` already decrypts the files, so the app needs no decrypt code.
*   **Cost accepted:** the author needs the `age` or `rage` tool installed. The round-trip test in the plan is what proves it works with a `key export` file.

### A custom envelope and header for the scan export

A project-specific file header carrying a version and recipient key id, around a sealed box.

*   **Left out:** `age` has an established format, a key tag in its header, and tools to read it. A custom envelope would be new cryptographic code to write, review and keep.

### Using the key as a raw hex or other project-specific file

*   **Left out:** the author wants a key other software can read. Raw hex works only with this app.
