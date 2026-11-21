# 0043. Key management: export, import and reset keys from the CLI

* **Status:** open
* **Kind:** feature
* **Source:** author, 2026-11-21
* **ADRs:** [0039](../docs/adr/0039-key-management.md)
* **EGM:** [0043-key-management](../docs/egm/0043-key-management/README.md)

## What

Handle keys properly (ADR 0039; working papers in `docs/egm/0043-key-management/`). This ticket covers 0009 (closed into it), 0003 (reduced to verification) and the key side of 0008 and 0044.

Part 1 (this ticket, build first):

* The `key` subcommand group: `show`, `export [--stdout] [--public] <file>`, `import [--force] <file>`, `reset [--yes]`; `--print-public-key` as an alias for `key show`. No subcommand opens the window.
* `app/src/keys.rs`, generic over the store (no `dyn`); OpenSSH ed25519 key file, unencrypted, owner-only on Unix, refuse to overwrite on export, no permission check on read.
* Running-app guard by the store's own lock (depends on ticket 0003 verifying `varaq`).
* Tests: round trip gives the same peer ID, every refusal, a headless start after `key import`, a fresh headless start generating a key.
* Docs: README keys section and backup process, CLAUDE.md, ADR 0039, `docs/spike-s0.md` rows.

## Done when

The commands work on macOS and Linux (Windows ticketed); the Rust tests and clippy for the touched modules pass; `ssh-keygen -y -f` accepts an exported key and its public key equals the peer ID.
