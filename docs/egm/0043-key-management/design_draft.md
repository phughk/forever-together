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

## Detailed Implementation

Paths are from the repository root. No file is deleted. The webapp is not touched by this design: no key operation is reachable from the web API (a goal). Work is in two parts. **Part 1** (ticket 0043) is the CLI and key file and can be built and tested alone. **Part 2** (ticket 0044) is the scan-log encryption; it depends on the scan log (0039, 0040), which does not exist yet, and on a `BUILTIN` key (0008).

### Part 1: key commands

| File | Change | Why |
|------|--------|-----|
| `app/Cargo.toml` | Add the OpenSSH key crate (`ssh-key` with the `ed25519` feature, to be confirmed against the open question). | Read and write the standard key file. Only `ed25519-dalek` is a dependency today. |
| `app/src/keys.rs` (new) | The key operations over `&dyn Store` and file paths: `show`, `export_to_file`, `import_from_file`, `reset`. Pure of the CLI: they take a store and a path and return a result or a typed error (`KeyError`: file exists, not an ed25519 OpenSSH key, key is encrypted, would replace an existing key without force, store failure). OpenSSH encode and decode live here. | One home for the logic, testable with `MemStore` and a temp directory, no network or window. |
| `app/src/identity.rs` | Add `Identity::from_secret` use for import (already present), a function to replace the stored secret in one write, and one to delete and regenerate (reset). Keep the stored format: 32 bytes under `KEY_IDENTITY_SECRET`. | The stored key must not change shape; import and reset need atomic replace semantics (parse first, write once). |
| `app/src/cli.rs` | Add an optional `key` subcommand with `show`, `export <file>`, `import <file> [--force]`, `reset [--yes]`. Keep `--print-public-key` as an alias for `key show`, and keep `--headless` and `--data-dir` as they are. `Mode` gains a `Key(KeyCommand)` variant. Update the tests for flag and subcommand parsing, including that no subcommand still gives `Window`, `--headless` still gives `Headless`, and `--print-public-key` still wins. | The decided command shape. |
| `app/src/lib.rs` | Declare `mod keys`; in `run`, dispatch `Mode::Key` before any window, runtime or network start-up, and exit with a failure status and a message on error. | Key commands must never start the app. |
| `app/src/runtime.rs` | `public_key` is reused by `key show`; no behaviour change. Add a short comment that headless and window modes both create a key on first start through `Identity::load_or_create`. | Records decision 4 where the start-up code is. |
| `app/src/runtime_tests.rs` | Add a test: `key import` a known key into a fresh data directory, start headless, and check the peer ID equals the imported key's. Add a test that a fresh directory with no import auto-generates a key at start. | Proves the deploy path from the plan. |
| `app/src/net/trusted.rs` | Update the comment on `BUILTIN` to say the entry comes from `key show` on the trusted node and that its secret is backed up with `key export`. The value itself is ticket 0008, not this change. | The comment currently names `--print-public-key`. |
| `README.md` | Add a "Keys" section: the commands, the backup process for the trusted key (export to offline storage), and the effect of a reset. Update the run-a-trusted-peer instructions. | The backup is a written process, not code. |
| `CLAUDE.md` | Update the "Adding a trusted peer" line and the commands list to use `key show`, and mention the `key` group. | Keeps project guidance current. |
| `docs/adr/0039-key-management.md` | Replace the open details with the final names, the OpenSSH format, and the decisions in this design (plain key, no passphrase, no permission check on read, headless same CLI, no decrypt command). Link `docs/egm/0043-key-management/`. | ADR 0036: decisions leave the iteration as ADRs. |
| `docs/spike-s0.md` | Add rows: whether `varaq` refuses a second process on a live data directory; the store file's mode on each OS; that a key written by `key export` is accepted by `ssh-keygen -y -f`. | They are unverified facts, in line with how the project tracks them. |
| `tickets/0043-key-management.md`, `0009-load-private-key-into-instance.md`, `0003-verify-varaq-key-permissions.md`, `0008-trusted-peer-builtin-key.md` | Update status, ADR and EGM links; close 0009 into 0043; reshape 0003 as a verification-only ticket. | Ticket fields in ADR 0036. |
| `tickets/README.md` and new tickets | Add small tickets for the Windows check and the running-app guard. | They are the leftover work from this pass. |
| `docs/egm/0043-key-management/README.md`, `docs/egm/README.md` | Status, rounds and links. | Iteration bookkeeping (ADR 0036). |

Pseudocode of the three file operations (no code is written in this draft):

```
export(store, path):
    secret = store.get(identity/secret)  // 32 bytes, create if missing
    if exists(path): error FileExists
    write path with mode 0600 (Unix) containing OpenSSH-encode(secret)   // unencrypted ed25519
    print peer_id

import(store, path, force):
    key = OpenSSH-decode(read(path))   // error if encrypted, wrong type, or malformed
    if store has identity/secret and not force: error WouldReplace(old_peer_id, new_peer_id)
    store.put(identity/secret, key.secret_bytes)   // one write, after all checks
    print old and new peer ids

reset(store, yes):
    require confirmation unless yes
    store.put(identity/secret, new random 32 bytes)   // one write
    print old and new peer ids, and the warning
```

### Part 2: encrypted scan export (ticket 0044; after 0039 and 0040)

| File | Change | Why |
|------|--------|-----|
| `app/Cargo.toml` | Add `age` with the `ssh` and `armor` features. | The recipient type for ssh-ed25519 keys and ASCII armor. |
| `app/src/scans/export.rs` (new, inside the module from ticket 0039) | Build an `age` ssh-ed25519 recipient from the 32-byte `BUILTIN[0]`, encrypt the export JSON, emit armored text. Return a typed error if `BUILTIN` is empty. | The only place that does encryption. |
| `app/src/scans/mod.rs`, `app/src/server/` export route (tickets 0039, 0040) | Call the export module; the route returns the armored file or a typed "no recipient key yet" error. | Wiring owned by those tickets; this design only fixes the format. |
| `app/src/scans/export_tests.rs` or inline tests | Round trip: encrypt to a test key with the same recipient code; load an identity from an OpenSSH file written by `key export`; decrypt with the `age` library; compare to the original JSON with invented characters. A wrong key fails. Empty `BUILTIN` gives the typed error. | Proves the author's `age -d -i` workflow without a decrypt command in the app. |
| `docs/adr/0038-scan-log-and-opt-in-export.md` | Amend the 2026-11-21 amendment: the file is `age` armored output, the key tag is age's own, no custom header; the key-loss paragraph points to the backup process in the README. | Removes the custom header the amendment sketched. |
| `tickets/0044-encrypted-scan-export.md`, `0040-scan-log-export-api.md`, `0041-scan-log-ui.md` | Link the ADR and this iteration; note the order dependency. | Ticket fields. |

### Build order

1.  `keys.rs` with export and import and their tests (a round trip gives the same peer ID; every refusal case).
2.  `reset` and the CLI restructure in `cli.rs` and `lib.rs`; the headless deploy test in `runtime_tests.rs`.
3.  Docs, ADR 0039 final text, spike rows and ticket updates.
4.  Later, after 0039, 0040 and 0008: Part 2.

### Verification

*   `cargo test --lib keys::`, `cargo test --lib cli::`, `cargo test --lib runtime::` and `cargo clippy --all-targets -- -D warnings`, per the project's "only test what you changed".
*   Manual: `key export`, then `ssh-keygen -y -f <file>` prints a public key whose bytes equal the peer ID; `key import` the file into a second data directory and `key show` prints the same peer ID.
*   Manual on the trusted node after 0008: backup export taken and stored offline.
*   Part 2 round trip as above, plus the author decrypts a real export with `age -d -i`.
