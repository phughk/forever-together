# 0044. Encrypt the scan export to the first trusted peer key

* **Status:** open
* **Kind:** feature
* **Source:** author, 2026-11-21
* **ADRs:** [0038](../docs/adr/0038-scan-log-and-opt-in-export.md)
* **EGM:** [0043-key-management](../docs/egm/0043-key-management/README.md)

## What

Encrypt the exported scan log to `net::trusted::BUILTIN[0]` (ADR 0038 amendment).

* Decided: encrypt to `BUILTIN[0]` with the `age` ssh-ed25519 recipient (no dedicated intake key).
* File: `age` ASCII armor, `scans-<date>.wowft-scans.age`, no custom header. No plain export. Export unavailable until `BUILTIN` has a key (ticket 0008).
* `BUILTIN` entries are assumed valid: the export panics if `BUILTIN[0]` is not a valid key, and a test checks every entry. Run `cargo audit` or `cargo deny` before adding `age` with the `ssh` feature (the `rsa` advisory is an accepted cost, ADR 0039).
* Review step: show the player what will be sent before it is encrypted.
* No decrypt command in the app: the author decrypts with `age -d -i <exported key>`. Add a round-trip test (encrypt with the recipient code, decrypt with an identity loaded from the exported OpenSSH file), using invented characters.

## Done when

Round-trip test: encrypt with a test key, decrypt with the matching key, a wrong key fails; the app cannot read its own export; the file is safe to attach to a public issue.
