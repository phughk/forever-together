# 0013. The desktop app is one Rust crate with modules

* **Status:** Accepted
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

An earlier draft split the app into several crates (code, core, net, store, server).

## Decision

Use the template's `app/` crate (the template's `src-tauri`, renamed) with modules `code`, `engine`, `net`, `store` and `server` (not `core`, which would clash with Rust's standard `core` crate), plus the Tauri shell and a headless flag. `engine` does no I/O and `net` has a trait with an in-memory fake, so tests can still drive the state machine directly.

## Consequences

Module boundaries are not enforced by the compiler as crate boundaries would be.

## Alternatives considered

* A Rust workspace with several crates: rejected by author as excessive.
