# 0039. The app's keys can be exported, imported and reset from the command line

* **Status:** Accepted (decided by the author 2026-11-21; Part 1 built 2026-11-21, ticket 0043). Extends [0008](0008-iroh-and-local-storage.md) (permanent keys in the `varaq` store), [0016](0016-peers-identified-by-key.md) and [0023](0023-no-supernodes.md). Used by [0038](0038-scan-log-and-opt-in-export.md). Working papers: `docs/egm/0043-key-management/`.
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

The app generates an iroh secret key on first start and stores it (32 bytes under `identity/secret`, [0008](0008-iroh-and-local-storage.md)). The only key tool is `--print-public-key`. The author runs trusted nodes on cloud machines, needs backups of the trusted key, and holds the private key that decrypts exported scan logs ([0038](0038-scan-log-and-opt-in-export.md)).

## Decision

* **The OpenSSH ed25519 file format is written and read by this project** (about 100 lines in `app/src/keys.rs`), with no new crate: unencrypted `openssh-key-v1`, one ed25519 key, empty comment, 70-column base64. It is checked against real `ssh-keygen` files and read back by `ssh-keygen -y` and `age -d -i`. Encrypted and other-type files are refused with their own errors.
* **A `key` subcommand group:** `key show`, `key export`, `key import`, `key reset`. `--print-public-key` is removed; `key show` replaces it (review round 1). There is no `--public` option. Running with no subcommand opens the window. Headless is the same binary and the same command line: it generates a key at first start and takes the same commands. A cloud node gets a chosen key by `key import` before its first start; there is no key path or environment variable read at every start.
* **The key file is a plain key.** Export writes what is in the store as an unencrypted OpenSSH ed25519 private key file, so ssh, `age` and similar tools can read it. No passphrase and no encryption layer. Keeping the file safe is the user's job; the app does not treat it as a risk to solve. The app creates exported files owner-only on Unix, refuses to overwrite an existing file, and does not check permissions when it reads one.
* **`key export --stdout`** prints the key for piping. Without it nothing secret goes to standard output. The key is never reachable from the web app, the API or logs.
* **`key import`** parses and checks the file fully, then replaces the stored key in one write. It refuses to replace an existing key without `--force`, and rejects an encrypted file or another key type.
* **`key reset`** discards the key and generates a new one, after confirmation (`--yes` skips it). Both commands print the old and new peer IDs.
* **Read commands create a key:** `key show` and `key export` create a key on an empty store, as `--print-public-key` did. A later `key import` onto that key needs `--force`.
* **Running-app guard (built):** step 0 showed `varaq` does not refuse a second opener (two processes can write the same files). The app (window and headless) takes an exclusive lock on `<data dir>/lock` for as long as it runs, and **every** `key` command takes it too, so all of them refuse while a node runs on that directory (`show` and `export` open the store and may create a key, so they are not safe to run beside a live node). The operating system drops the lock if the process dies. This supersedes the earlier "rely on the store's own lock".
* **Windows and Settings:** the key commands are unsupported on Windows release builds (GUI subsystem, no console) until ticket 0046. Key actions in Settings are deferred (ticket 0046) so no key operation is on the web API.
* **`BUILTIN` validity:** entries are assumed valid; the scan export panics if `BUILTIN[0]` is not a valid key, and a test checks every entry.
* **A key change changes the peer ID** ([0016](0016-peers-identified-by-key.md)). Remembered peers stay. A changed trusted-peer key needs a release ([0034](0034-system-overview-goals-and-risks.md)). The command says so.
* **The scan-log decryption key is the trusted peer's own key** (`BUILTIN[0]`, see [0038](0038-scan-log-and-opt-in-export.md)). The author decrypts with `age -d -i <exported key file>`. The app has no decrypt command and holds no decrypting identity.
* **Backups:** the trusted key is exported with `key export` and kept offline by the author, as a written process in `README.md`.
* **Style:** new code uses generics and enums, not `dyn` (existing `dyn Store` use is ticket 0045).

## Consequences

* A cloud node can be rebuilt with the same identity from a backup.
* Losing the trusted key without a backup means a new key, a new release, and unreadable old scan files.
* Anyone who can read the data directory or an exported file has the identity.
* New dependencies: none so far. `age` with the `ssh` feature is added for the scan export (ticket 0044). `age`'s `ssh` feature pulls in `rsa` (reported advisory RUSTSEC-2023-0071, unverified): accepted, with `cargo audit` or `cargo deny` run before the dependency is added. Not yet in `app/Cargo.toml`.
