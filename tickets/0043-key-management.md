# 0043. Key management: export, import and reset keys from the CLI

* **Status:** open
* **Kind:** feature
* **Source:** author, 2026-11-21
* **ADRs:** [0039](../docs/adr/0039-key-management.md)
* **EGM:** none

## What

Handle keys properly. This ticket covers ticket 0009 (load a chosen key) and the key side of 0008 and 0003.

* Command-line export, import and reset of the iroh secret key (ADR 0039).
* Decide: command names (for example `key export`, `key import`, `key reset` beside `--print-public-key`), the key file format, whether a file can be passphrase-protected, and how a cloud node gets its key (file path, environment variable or import at deploy).
* File permissions on write and on read (refuse a world-readable key file); refuse to overwrite; refuse while the app runs on that data directory; reset needs confirmation.
* Say what a key change does: new peer ID, remembered peers kept, trusted-peer key change needs a release.
* Back up the author's trusted and decryption keys offline (a process, written down).
* Coordinate with 0003 (`varaq` store permissions) and 0008 (`BUILTIN` key).

## Done when

Commands work on macOS and Linux (Windows checked or ticketed); tests with a temp data dir cover export, import round trip (same public key), reset (new public key), refusal cases; ADR 0039 updated with the final names and format.
