# 0032. Replace the hostname guess for the phone read-only flag

* **Status:** open
* **Kind:** bug
* **Priority:** P2
* **Source:** ADR 0032
* **ADRs:** none
* **EGM:** [initial draft](../docs/egm/0032-remote-flag-from-server/design_draft.md) (2026-11-22, unreviewed)

## What

The `remote` flag is guessed from the hostname, so lock icons can be wrong until a 403 arrives.

## Done when

The server reports whether the caller may change Advanced.
