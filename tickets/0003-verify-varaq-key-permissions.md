# 0003. Verify varaq key-file permissions and maturity

* **Status:** open
* **Kind:** verification
* **Source:** ADR 0008; docs/spike-s0.md
* **ADRs:** none
* **EGM:** [0043-key-management](../docs/egm/0043-key-management/README.md)

**Also:** check whether `varaq` refuses a second process opening the same data directory. The key commands (ticket 0043, decision 6) rely on that for their running-app guard; if it does not, the design is reopened.

**Related:** ticket 0043 adds key export and import, so permission handling matters more.

## What

Check that the `varaq` store holds the secret key with restrictive file permissions on macOS, Windows and Linux, and that varaq 0.1.x is good enough.

## Done when

Permissions checked on three OSes; ADR 0008 updated with the result.
