# 0043. Key management: export, import and reset keys from the CLI

* **Status:** done for Part 1 (built 2026-11-21; Part 2 is ticket 0044, Settings and Windows are ticket 0046)
* **Kind:** feature
* **Source:** author, 2026-11-21
* **ADRs:** [0039](../docs/adr/0039-key-management.md)
* **EGM:** [0043-key-management](../docs/egm/0043-key-management/README.md)

## What

Handle keys properly (ADR 0039; working papers in `docs/egm/0043-key-management/`). This ticket covers 0009 (closed into it), 0003 (reduced to verification) and the key side of 0008 and 0044.

Part 1 (this ticket, build first):

* The `key` subcommand group: `show`, `export [--stdout] <file>`, `import [--force] <file>`, `reset [--yes]`. `--print-public-key` is removed (`key show` replaces it); no `--public`. No subcommand opens the window. `show` and `export` create a key on an empty store.
* `app/src/keys.rs`, generic over the store (no `dyn`); OpenSSH ed25519 key file, unencrypted, owner-only on Unix, refuse to overwrite on export, no permission check on read.
* **Step 0:** verify whether `varaq` refuses a second opener. Guard by the store's own lock if it does; if it does not, a lock file taken by the app and the key commands (ADR 0039).
* Key commands are unsupported on Windows (ticket 0046). Run `cargo audit` or `cargo deny` before adding `age` with the `ssh` feature.
* Tests: round trip gives the same peer ID, every refusal, a headless start after `key import`, a fresh headless start generating a key.
* Docs: README keys section and backup process, CLAUDE.md, ADR 0039, `docs/spike-s0.md` rows.

## Done when

The commands work on macOS and Linux (Windows ticketed); the Rust tests and clippy for the touched modules pass; `ssh-keygen -y -f` accepts an exported key and its public key equals the peer ID.

## Built

`app/src/keys.rs`, `app/src/store/dir_lock.rs`, the `key` subcommands in `app/src/cli.rs`, `runtime::run_key_command`, README "Keys". Step 0 result and manual checks are in `docs/spike-s0.md` ("Key commands"). 313 Rust lib tests pass and clippy is clean. Windows is not supported (ticket 0046). Linux is not run.

