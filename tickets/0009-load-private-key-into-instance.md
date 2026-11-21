# 0009. Load a chosen private key into an instance

* **Status:** open
* **Kind:** feature
* **Source:** the tickets in `tickets/`
* **ADRs:** none
* **EGM:** none

**Folded into ticket 0043 (key management).** Keep this open only until 0043 lands.

## What

Trusted nodes run on cloud instances and need a stable key supplied by the operator instead of a generated one. Decide the interface (flag, env var or file) and its permission checks.

## Done when

A node starts with a supplied key and prints the same public key every time. Needs an ADR (extends 0008).
