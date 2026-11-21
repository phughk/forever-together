# 0003. Verify varaq key-file permissions and maturity

* **Status:** open
* **Kind:** verification
* **Source:** ADR 0008; docs/spike-s0.md
* **ADRs:** none
* **EGM:** none

**Related:** ticket 0043 adds key export and import, so permission handling matters more.

## What

Check that the `varaq` store holds the secret key with restrictive file permissions on macOS, Windows and Linux, and that varaq 0.1.x is good enough.

## Done when

Permissions checked on three OSes; ADR 0008 updated with the result.
