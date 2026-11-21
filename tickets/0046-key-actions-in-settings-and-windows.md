# 0046. Key actions in Settings, and the key commands on Windows

* **Status:** open
* **Kind:** feature
* **Source:** author, egm 0043 round 1 (A6)
* **ADRs:** [0039](../docs/adr/0039-key-management.md)
* **EGM:** [0043-key-management](../docs/egm/0043-key-management/README.md)

## What

Windows release builds use the GUI subsystem (`app/src/main.rs`), so `key show`, `key export --stdout`, the reset prompt and error messages print nothing from a terminal. The author asked for the key functionality to be possible from Settings, then deferred it, because it puts key operations on the loopback web API and the secret key must not be reachable from the web app.

Decide and design: which key actions appear in Settings (show the public key, import, export, reset); how the secret is handled (the reviewed options were: a typed file path read and written by the server so the browser never sees the key, or download and upload through HTTP, or show and reset only); what a Windows user gets before then. This needs its own ADR because it amends the goal that no key operation is reachable from the web API.

## Done when

A design and ADR exist, the key commands either work on Windows or are documented as unsupported there, and the Settings card (if built) has stories and is loopback-only and locked on a phone.
