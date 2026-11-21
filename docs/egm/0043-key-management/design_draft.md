# Key management for forever-together

*   **Author:** Hugh Kaznowski
*   **Status:** Part 1 built (2026-11-21). Revised after review round 1 (see [resolutions](resolutions.md) and [the consolidated report](reviews/r1-consolidated_review_report.md)). Part 2 is not built.
*   **Date:** 2026-11-21
*   **Ticket:** [ticket 0043](../../../tickets/0043-key-management.md), which also covers [ticket 0003](../../../tickets/0003-verify-varaq-key-permissions.md), [ticket 0008](../../../tickets/0008-trusted-peer-builtin-key.md), [ticket 0009](../../../tickets/0009-load-private-key-into-instance.md) and [ticket 0044](../../../tickets/0044-encrypted-scan-export.md).
*   **Related decisions:** [ADR 0008](../../adr/0008-iroh-and-local-storage.md), [ADR 0016](../../adr/0016-peers-identified-by-key.md), [ADR 0023](../../adr/0023-no-supernodes.md), [ADR 0034](../../adr/0034-system-overview-goals-and-risks.md), [ADR 0038](../../adr/0038-scan-log-and-opt-in-export.md), [ADR 0039](../../adr/0039-key-management.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

forever-together is a peer-to-peer random dungeon finder for World of Warcraft Forever: a desktop app (plus a web UI and a WoW addon) with no server run by the author. The addon shows a character as a text code; the app takes the pasted code and finds a party over a peer-to-peer network.

Every install has one permanent ed25519 secret key. The app creates it on first start and keeps it in its local store (`varaq`, a small embedded key-value database) under `identity/secret`. Its public half is the **peer ID** other peers see, and the same bytes are the key of the iroh network endpoint (iroh is the peer-to-peer library the app uses). A **trusted peer** is an ordinary install, run headless (the same program without a window or web server) on an always-on machine, whose peer ID is compiled into the app as a last-resort first contact (`net::trusted::BUILTIN`, currently empty).

The **scan log** is a planned local log of the character codes a player has pasted (ticket 0039, [ADR 0038](../../adr/0038-scan-log-and-opt-in-export.md)): the raw codes with times, grouped by character, exportable as one file players attach to a GitHub issue so the author can learn region and realm values from other regions. Codes contain character names, realms and zones.

This design adds the missing key tooling: a `key` command group to show, back up, restore and replace the key; a data-directory lock (needed because `varaq` allows two processes on one directory); and the interface for encrypting the scan-log export to the first trusted peer's key.

## The Problem

Today the key is created and used, and the only thing that can be done with it is print its public half (`--print-public-key`).

*   No backup. If the author's trusted node loses its data directory, its key is gone. A new key is a new peer ID, and `BUILTIN` lives in the binary, so a new key needs a new release.
*   No way to start a node with a chosen key, so a cloud node cannot be rebuilt with the same identity, and `BUILTIN` cannot be filled with a key that will still exist next month.
*   No reset for a leaked key short of deleting files by hand.
*   The scan export needs a recipient key, and a private key the author can decrypt with. `BUILTIN[0]` is the chosen recipient and it is empty.
*   `varaq` was assumed to refuse a second process on a data directory. Step 0 of the build showed it does not, so two processes (for example a key command and a running node) could corrupt the store.

## Goals

*   **Export** the secret key to a file, or to standard output, in a standard format other tools read.
*   **Import** a secret key from such a file, so a node starts with a chosen identity.
*   **Reset** the key: discard it and make a new one, after a confirmation.
*   Keep the secret key out of the web app, the HTTP API and logs (decision 11).
*   Only one process uses a data directory at a time (decision 9).
*   `BUILTIN` can be filled with the trusted key, with a written backup process.
*   The scan export is encrypted to `BUILTIN[0]` and the author decrypts it with the standard `age` tool and the exported key. The app has no decrypt command.

## Non-Goals

*   A passphrase on the key, or the operating system keychain (decision 1). The keychain stays the fallback named in [ADR 0008](../../adr/0008-iroh-and-local-storage.md) if `varaq` proves unfit.
*   Rotating `BUILTIN` without a release.
*   Key operations from the web app or the HTTP API, and Windows support for the key commands (decision 11, [ticket 0046](../../../tickets/0046-key-actions-in-settings-and-windows.md)).
*   Multi-key identities, sub-keys, revocation lists, signing releases or addon files.
*   A hosted key service ([ADR 0034](../../adr/0034-system-overview-goals-and-risks.md): nothing is hosted).

## Decisions

Decided by the author on 2026-11-21 (1 to 6 while drafting, 7 to 12 in review round 1, see [resolutions](resolutions.md)).

1.  **The key file is a plain key.** Export writes what is in the store as a standard, unencrypted key file that ssh and similar tools read. No passphrase, no encryption layer; securing the file is the user's job.
2.  **The scan export is encrypted to the public half of `BUILTIN[0]`.** The author owns that trusted peer, holds its secret and decrypts. Players expose no characters or codes.
3.  **A `key` subcommand group** (`show`, `export`, `import`, `reset`). No subcommand opens the window.
4.  **Headless is the same binary and command line.** It creates a key at first start and takes the same commands. A cloud node gets its key by `key import` before its first start.
5.  **`key export --stdout` prints the key** for piping. Without it nothing secret goes to standard output.
6.  **Running-app guard.** Originally the store's own lock; superseded by decision 9 after step 0.
7.  **No alias and no `--public`.** `--print-public-key` is removed and `key show` replaces it; `key show` prints the OpenSSH public line, so no `.pub` file option.
8.  **Read commands create a key, as `--print-public-key` did.** `key show` and `key export` on an empty store create one. A later `key import` onto that key needs `--force`.
9.  **A lock file guards the data directory.** The app and every `key` command take an exclusive lock on `<data dir>/lock`; a second holder is refused. This supersedes decision 6.
10. **`BUILTIN` entries are assumed valid.** The scan export builds its recipient from `BUILTIN[0]` and panics if it is not a valid key; a test checks every entry.
11. **Windows and Settings are deferred.** The key commands are unsupported on Windows release builds (no console) until [ticket 0046](../../../tickets/0046-key-actions-in-settings-and-windows.md). Key actions in Settings are dropped for now because they would put key operations on the web API.
12. **`age` with the `ssh` feature is accepted, with a gate.** It pulls in the `rsa` crate (a reported advisory, RUSTSEC-2023-0071, not verified here). Run `cargo audit` or `cargo deny` before adding it. A separate offline intake key was rejected again.

## Proposed Architecture

```mermaid
flowchart TD
    CLI[key show / export / import / reset] --> RUN[runtime::run_key_command]
    RUN --> LOCK[DataDirLock: lock file]
    RUN --> KEYS[keys: show, export, import, reset]
    KEYS --> ST[(Store: identity/secret, 32 bytes)]
    KEYS <--> F[OpenSSH ed25519 key file]
    APP[App window or headless] --> LOCK
    APP --> ST
    PL[Player: scan log] -.->|Part 2: age, to BUILTIN 0| X[armored file on a GitHub issue]
    X -.->|age -d -i| A[Author, with the exported key file]
    F -.-> A
```

Solid lines are built (Part 1); dotted lines are Part 2 (not built). The key stays 32 bytes under `identity/secret`; new code reads and writes it in a standard file format and exposes it through the command line.

### Command line

`forever-together [--headless] [--data-dir DIR] [key <command>]`. `--data-dir` is global, so it works after a subcommand. A `key` subcommand wins over `--headless`. `--print-public-key` no longer exists (clap rejects it). Every `key` command takes the data directory lock first and fails at once, with a message to stop the other process or use another `--data-dir`, if it is held.

| Command | Behaviour |
|---|---|
| `key show` | Prints two lines: the peer ID (64 hex characters, the form used in `BUILTIN` and `trusted_peers`), then the OpenSSH public key line (`ssh-ed25519 <base64>`). Creates a key if none is stored. The old `--print-public-key` printed only the hex. |
| `key export <FILE>` | Writes the key as an unencrypted OpenSSH ed25519 private key file with one create-exclusive open (mode `0600` on Unix); an existing file is never touched and a partly written file is removed. Prints `wrote the key for peer <hex> to <FILE>`. Creates a key if none is stored. |
| `key export --stdout` | Prints the key to standard output and `peer <hex>` to standard error, so a pipe carries only the key. `FILE` and `--stdout` are mutually exclusive and one is required. A shell redirect creates the target with the user's umask, so the user should `chmod 600` it. |
| `key import [--force] <FILE>` | Reads an unencrypted OpenSSH ed25519 key file, checks it fully, then replaces the stored key in one write. Refuses (with the old and new peer IDs and a hint) if any value is already stored, including a damaged one, unless `--force`. Refuses an encrypted file, another key type, malformed or oversized input. Prints `imported the key; peer <old|none|unreadable> -> <new>` and, when it replaced something, the warning below. |
| `key reset [--yes]` | If a key (or damaged value) is stored, asks `Replace this install's key (peer <old>) with a new one? ... [y/N]` unless `--yes`; with no stored key it does not ask. A refusal or an end of input exits with an error and changes nothing. Makes a new random key in one write and prints `new key; peer <old|none|unreadable> -> <new>` and the warning. |

The warning printed after a replacement: other nodes now see a new peer ID, and a trusted peer's entry in `BUILTIN` needs a new release.

Exit status is 0 on success and 1 on any error (the message goes to standard error).

### The `keys` module contract

`app/src/keys.rs` is generic over the store (`S: Store + ?Sized`, no `dyn`) and **returns data and never prints**:

*   `show(store) -> KeyInfo { peer_id, openssh_public }`, creating a key if missing.
*   `export_text(store) -> (PeerId, String)` and `export_to_file(store, path) -> PeerId`.
*   `import_from_file(store, path, force) -> Replaced { old, new }`.
*   `reset(store) -> Replaced { old, new }`.
*   `old_key(store) -> OldKey`, which is `None`, `Id(PeerId)` or `Unreadable` (a stored value that is not 32 bytes). It never creates anything.
*   `KeyError`: `FileExists`, `Io`, `NotOpenSsh`, `Encrypted`, `WrongKeyType`, `BadKeyFile`, `BadStoredKey`, `WouldReplace(Replaced)`, `Store`.

`runtime::run_key_command(data_dir, cmd, out, err, confirm)` does everything that touches the outside: it takes the lock, opens the concrete `VaraqStore`, calls `keys`, writes to the output writers and asks the `confirm` closure for the reset prompt. `lib.rs` only wires standard output, standard error and standard input. Tests pass in-memory writers and a closure.

**Damaged stored key.** If `identity/secret` is not 32 bytes, `key show` and `key export` fail with `BadStoredKey` (the message names `key reset` and `key import --force`); `import --force` and `reset` replace it and report the old peer as `unreadable`.

### Key file format

An OpenSSH private key (`-----BEGIN OPENSSH PRIVATE KEY-----`): the `openssh-key-v1` container, cipher and KDF `none`, one `ssh-ed25519` key, empty comment, 70-column base64. The code is written in `app/src/keys.rs` (about 100 lines, no crate). The reader refuses wrong key types and passphrase-protected files with their own errors, a wrong check number, wrong padding, a public half that does not match the private half, data after the end marker, and files over 16 KiB. It accepts CRLF line endings. The same file works with `ssh-keygen -y -f` and `age -d -i`. The iroh endpoint key and an ssh ed25519 key are both ed25519, so the 32 secret bytes convert without changing the stored format or the peer ID.

### The data directory lock

`app/src/store/dir_lock.rs` holds `DataDirLock`: an exclusive lock (std `File::try_lock`) on `<data dir>/lock` (mode `0600` on Unix), refused at once if another process holds it, released when dropped or when the process dies. The app (window and headless) takes it in `runtime::open_store` and keeps it for as long as it runs (`Running._lock` is the last field, so it is released after the store). Every `key` command takes it. This also stops a second app on the same directory.

Consequence: all key commands, including `show` and `export`, fail while a node runs on that directory, because they open the store and may create a key. **Backing up a running trusted node means stopping it first.**

### Scan-log encryption (Part 2, interface only; ticket 0044)

The export module takes `BUILTIN[0]` (32 bytes) and the export JSON and returns `age` ASCII armor, built with an ssh-ed25519 recipient (the line `ssh-ed25519 <base64>` is formed from the 32 bytes; it is not verified that `age` offers a raw-bytes constructor). `age` converts Ed25519 to X25519 itself, so the project writes no cryptography. The file is `scans-<date>.wowft-scans.age`; the `age` header carries a key tag so the author can tell which key to use after a rotation. Only `BUILTIN[0]` is a recipient; if `BUILTIN` is empty, export is unavailable. An invalid `BUILTIN[0]` panics (decision 10). The review-before-export step required by ADR 0038 belongs to the export route and UI (tickets 0040 and 0041). Details are in ticket 0044.

### `BUILTIN` and the trusted key

*   **Filling it in:** on the always-on node, stop it, run `key show`, paste the peer ID into `net::trusted::BUILTIN` ([ticket 0008](../../../tickets/0008-trusted-peer-builtin-key.md)).
*   **Backup:** `key export` to an offline disk with the node stopped. Keep every historical `BUILTIN[0]` secret, because installs on older releases keep encrypting to the key compiled into their build. This is a written process in `README.md`.
*   **Losing the key:** a new key, a new `BUILTIN` entry and a release; old exports can no longer be read. Old builds fall back to remembered peers and peer exchange ([ADR 0023](../../adr/0023-no-supernodes.md)).
*   **Deploy:** a cloud node gets its key by `key import` before its first start. A start, `key show` or `key export` on a fresh volume creates a random key and the import then needs `--force` (decision 8).

### Permissions

The app creates exported files and the lock file owner-only on Unix and does not check permissions when reading a key file (decision 1). The `varaq` store sets its directory to `0700` and its files to `0600` on every open. [Ticket 0003](../../../tickets/0003-verify-varaq-key-permissions.md) is now verification only: record file modes on Linux and Windows. An exported file's mode protects nothing if the store file itself is readable by others.

## Risks and Mitigations

| Risk | Impact | Mitigation or acceptance |
|---|---|---|
| Two processes use one data directory (`varaq` allows it) | Store corruption; a running node keeps the old key | `DataDirLock` on every start and every key command (decision 9); test `varaq_does_not_refuse_a_second_opener` fails if `varaq` changes, so decision 9 is revisited |
| Key file or data directory readable by others | Anyone holding it has the identity | Accepted (decision 1). Exports are created `0600`; the README says to protect the file |
| `key reset` or `key import --force` discards the only copy | A lost trusted key means a new release and unreadable exports | Reset asks for confirmation; import refuses without `--force`; both print old and new peer IDs and the warning; the README says to export before replacing. No automatic copy of the old key (rejected as extra scope) |
| A node with no import generates a new random identity | The trusted node silently breaks bootstrap | The README says to run `key show` after a deploy and compare the peer ID with `BUILTIN`. (A start-up log line exists in `run_headless` but no log output is installed, so nothing is printed.) No opt-in refuse-to-generate flag (rejected) |
| `key show` or `key export` on an empty volume creates a key | A later `key import` needs `--force` | Accepted (decision 8); documented in the README |
| Compromise of the trusted node exposes every export (the decrypting secret is its identity key) and lets an attacker impersonate it | Past and future exports readable; exports are public and permanent on GitHub, so a later leak is retroactive | Accepted (decision 2). Decrypt on an offline machine; keep the key backup offline |
| Resetting the trusted key makes old exports unreadable unless the old secret is kept | Lost samples | Keep every historical `BUILTIN[0]` secret (README) |
| `age` with the `ssh` feature pulls in `rsa` (reported advisory) and `age` is beta | Dependency risk | Accepted (decision 12), with `cargo audit` or `cargo deny` run before adding it (ticket 0044) |
| Wrong or mistyped `BUILTIN[0]` | Every export undecryptable | Entries assumed valid (decision 10): the export panics on an invalid key and a test checks every entry. There is no release step that decrypts with the real key |
| GitHub limits: 65,536 characters per issue body and an attachment extension allowlist (reported, unverified; armor adds about 33%) | Large exports cannot be posted | Open: ADR 0038 leaves the size cap undecided; settle in ticket 0044 (size cap, a `.txt` suffix, documented fallback) |
| Forward secrecy: exports stay public | A leaked key reads all past exports | Accepted with the first risk of this kind; no mitigation beyond offline decryption |
| Windows release builds have no console | Key commands print nothing there | Unsupported until ticket 0046 (decision 11) |
| `--stdout` output captured in logs or terminal history | Secret leaks | Accepted; the key is only printed when asked for, and the secret never goes to standard output otherwise |

## Open Questions

*   **GitHub size and extension limits** (risk table): to be settled in ticket 0044.
*   **Is a raw-bytes ssh-ed25519 recipient available in `age`, and is `age` acceptable as a dependency?** Checked when ticket 0044 starts (`cargo audit` or `cargo deny` and a try-build).
*   **Windows and Linux file modes and the Windows console:** tickets 0003 and 0046.

## Alternatives Considered

Each entry: what it is, the verdict, the reason and the cost accepted. "Rejected by author" records an author decision in [resolutions](resolutions.md) or [ADR 0039](../../adr/0039-key-management.md); the technical trade-off is the reason.

### A separate intake key for the scan export
*   **Description:** a dedicated X25519 key, secret offline, public half compiled in beside `BUILTIN`.
*   **Verdict:** rejected by author (twice, the second time with the `rsa` cost shown; resolution A1).
*   **Reason:** the author owns `BUILTIN[0]` and holds its secret, so no new constant or secret is needed.
*   **Cost accepted:** the decrypting secret is the live trusted node's identity key (see the risk table); the Ed25519 to X25519 conversion and the `rsa` dependency come with the `ssh` feature of `age`.

### Sealing each export to two keys
*   **Description:** two sealed copies of one random file key; either secret opens the file.
*   **Verdict:** rejected by author.
*   **Reason:** not needed once the separate key was declined; compromising either key exposes every file and the format grows. Its one benefit, surviving the loss of one key, is covered by the offline backup of the single key.
*   **Cost accepted:** none beyond the above.

### A plain (unencrypted) scan export
*   **Verdict:** rejected by author. Players would expose characters and codes in a public issue.

### A passphrase-protected file or keychain-stored key
*   **Verdict:** rejected by author (decision 1): a key is a key, export writes what is in the store, the user secures the file.
*   **Cost accepted:** anyone who can read the data directory or an exported file has the identity.

### Top-level flags instead of a `key` subcommand group
*   **Verdict:** rejected by author (decision 3).
*   **Reason:** flags combine with each other and with `--headless`, so more invalid combinations need rejecting, and each new operation adds a top-level flag.

### A key path or environment variable read at every start
*   **Verdict:** rejected by author (decision 4): headless is the same program and takes `key import`.
*   **Cost accepted:** a container with an ephemeral volume that skips the import step starts with a new random key.

### A decrypt command in the CLI
*   **Verdict:** rejected by author. `age -d -i <exported key>` already decrypts; verified with the real `age` tool on 2026-11-21 (see `docs/spike-s0.md`).
*   **Cost accepted:** the author needs `age` or `rage` installed.

### The store's own lock as the running-app guard
*   **Verdict:** chosen first (decision 6), then dropped after step 0 showed `varaq` takes no cross-process lock (decision 9).

### A warning only, with no guard
*   **Verdict:** rejected. A user could import into a running node and get no error while the node keeps its old identity and two processes write the store.

### Nothing secret ever on standard output
*   **Verdict:** rejected by author. An explicit `--stdout` is wanted for piping (decision 5).

### A custom envelope and header for the scan export
*   **Verdict:** left out. `age` has an established format, a key tag in its header and tools to read it; a custom envelope would be new cryptographic code to write and review.

### Raw hex, or PKCS#8 PEM, as the key file format
*   **Verdict:** left out. Raw hex has no standard tooling. PKCS#8 PEM is a realistic competitor, but ssh and `age` read the OpenSSH format.

### Backing up by copying `identity/secret` or the data directory
*   **Verdict:** left out. It needs knowledge of the store layout, copies settings and peers, and cannot be restored on another version of the app or read by other tools.

### Reviewer ideas not adopted (review round 1)
*   **Watchers that reread the stored key, and an `--expect-peer-id` start-up flag:** not adopted. The lock removes the cause of the first and the second adds a start-up input decision 4 rejected.
*   **`--save-old` and `--discard-old` on reset and forced import:** not adopted; the README says to export first.
*   **A standby trusted key in `BUILTIN`:** not adopted; belongs in ticket 0008 if wanted.
*   **Read commands that refuse on an empty store instead of creating a key:** rejected by author (decision 8).

## Detailed Implementation

### Part 1 (ticket 0043): built

File list. "Create" and "Modify" are marked; nothing is deleted except the `--print-public-key` flag, `Mode::PrintPublicKey` and `runtime::public_key`.

| File | Change | Why |
|---|---|---|
| `app/src/store/dir_lock.rs` | Create: `DataDirLock`, `LockError` and tests | Decision 9 |
| `app/src/store/mod.rs` | Modify: `mod dir_lock;` and re-export `DataDirLock`, `LockError` | Expose the lock |
| `app/src/store/varaq_store.rs` | Modify: test `varaq_does_not_refuse_a_second_opener` documents step 0 | Detect a future `varaq` change |
| `app/src/keys.rs` | Create: the contract above, the OpenSSH codec and tests | The key operations |
| `app/src/identity.rs` | Modify: `load_or_create` is generic over `S: Store + ?Sized` (existing `&dyn Store` callers still compile) | New code needs static dispatch without churning callers |
| `app/src/lib.rs` | Modify: `pub mod keys;`; dispatch `Mode::Key` to `runtime::run_key_command` before any window or network start-up, wiring standard streams and the confirm closure | Key commands never start the app |
| `app/src/cli.rs` | Modify: `Command`, `KeyCommand`, `ExportArgs`; `--data-dir` global; `Mode::Key`; remove `--print-public-key` | The command line |
| `app/src/runtime.rs` | Modify: `open_store` returns `(DataDirLock, Arc<dyn Store>)`; `Running._lock` last; `run_key_command`; remove `public_key` | Lock and runner |
| `app/src/runtime_tests.rs` | Modify: replace the `public_key` test with key command, deploy, lock and reset tests | Tests |
| `app/src/net/trusted.rs` | Modify: comment on `BUILTIN` names `key show` and `key export` | Stale comment |
| `README.md`, `CLAUDE.md` | Modify: "Keys" section (commands, backup process, deploy, no Windows); trusted-peer and flags text; `DataDirLock` in the module list | Written backup process |
| `docs/adr/0039-key-management.md`, `docs/spike-s0.md` | Modify: built notes; "Key commands" results | Record decisions and facts |
| `tickets/0003-…`, `0008-…`, `0043-…`, `0044-…`, `0046-…` | Modify or create | Ticket fields (ADR 0036) |

No change to `app/Cargo.toml` (no new dependency in Part 1).

### Tests (Part 1)

`keys::tests` (16): reading an `ssh-keygen` key and matching its public line; export then import round trip; the exported text shape and 70-column lines; mode `0600`; refusal to overwrite an existing file; export into a missing directory; `show` and `export` create a key once and keep it; import needs `--force` over a key made by `show`, and a failed import leaves the store unchanged; wrong key type (ecdsa) and passphrase key refused with their own errors; malformed files (empty, truncated, bad base64, data after the end marker, no end marker, a public-key line); a tampered private section; CRLF line endings; a file over the size limit; `reset` reports the old key and changes it; a damaged stored key blocks `show` and `export` but `reset` and forced import repair it.
`store::dir_lock::tests` (3): a second holder is refused until the first drops; different directories do not conflict; the lock file is private.
`store::varaq_store::tests`: a second `varaq` opener is not refused (step 0).
`cli::tests`: every subcommand parses; `export` needs exactly one of a file and `--stdout`; missing arguments and unknown subcommands or flags are errors; `--data-dir` after a subcommand; a subcommand wins over `--headless`; `--print-public-key` is rejected.
`runtime_tests` (key and lock): `key show` output shape and stability; the deploy path (export from one directory, import into a fresh one, the app loads that identity); `--force` over a `show`-created key; `--stdout` output and stream split; reset prompt, `--yes` and the empty-directory case; refusal while another holder has the directory; refusal while the app runs and a second app refused.

### Verification

Run per touched module (one filter per `cargo test --lib` invocation): `cargo test --lib keys::`, `store::`, `cli::`, `runtime`, `identity::`, `net::trusted::`; the whole `cargo test --lib` before merge; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check`. Results on 2026-11-21: 313 tests pass, clippy and fmt clean.

Manual checks (macOS, built binary, `ssh-keygen`, `age`) are recorded in `docs/spike-s0.md`: `ssh-keygen -y -f` on an exported file prints the same line as `key show`; `age -d -i <exported file>` decrypts a message encrypted with `age -r <that public line>`; a live headless node blocks `key show` and `key reset`; the old flag is rejected. To compare `ssh-keygen -y` output with the peer ID, decode the base64 key blob and take its last 32 bytes. The file must be `chmod 600` for `ssh-keygen` to accept it.

### Part 2 (ticket 0044): not built

Depends on the scan log (tickets 0039 and 0040), a `BUILTIN` key (ticket 0008) and the dependency gate (decision 12). Interface in the Scan-log encryption section; files to create then: `app/src/scans/export.rs` and its tests, an `age` dependency in `app/Cargo.toml` and `app/Cargo.lock`, and an amendment to ADR 0038 line 42 pointing at the README backup process. Tests then: encrypt with the recipient code and decrypt with the `age` library using an identity loaded from a file written by `key export`; a wrong key fails; an invalid `BUILTIN[0]` panics; every `BUILTIN` entry is a valid key.

### Out of scope

Settings and Windows ([ticket 0046](../../../tickets/0046-key-actions-in-settings-and-windows.md)); replacing the remaining `dyn Store` code, about 30 uses in `config.rs`, `history.rs`, `runtime.rs` (the `NetContext.store` field and `open_store`) and the app and server layers ([ticket 0045](../../../tickets/0045-replace-dyn-store-with-generics.md)). New key code uses generics and enums; this is a review-only rule, not enforced by clippy.
