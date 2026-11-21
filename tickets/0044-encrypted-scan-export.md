# 0044. Encrypt the scan export to the first trusted peer key

* **Status:** open
* **Kind:** feature
* **Source:** author, 2026-11-21
* **ADRs:** [0038](../docs/adr/0038-scan-log-and-opt-in-export.md)
* **EGM:** none

## What

Encrypt the exported scan log to `net::trusted::BUILTIN[0]` (ADR 0038 amendment).

* Decide: reuse the trusted peer's Ed25519 key (converted to X25519) or ship a dedicated intake key beside it. Default in the ADR is a sealed box on the converted key.
* File: header (format version, recipient key id) plus ciphertext, suffix `.wowft-scans`. No plain export. Export unavailable until `BUILTIN` has a key (ticket 0008).
* Review step: show the player what will be sent before it is encrypted.
* No decrypt command in the app: the author decrypts with `age -d -i <exported key>`. Add a round-trip test (encrypt with the recipient code, decrypt with an identity loaded from the exported OpenSSH file), using invented characters.

## Done when

Round-trip test: encrypt with a test key, decrypt with the matching key, a wrong key fails; the app cannot read its own export; the file is safe to attach to a public issue.
