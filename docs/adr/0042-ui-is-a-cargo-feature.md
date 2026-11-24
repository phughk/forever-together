# 0042. The window is a Cargo feature, so a headless build needs none of its libraries

* **Status:** Accepted (decided by the author 2026-11-24; built the same day)
* **Date:** 2026-11-24
* **Deciders:** Hugh Kaznowski

## Context

The trusted peers run headless on servers. Building the crate pulled in Tauri, and on Linux that needs the GTK and WebKit development packages (`gdk-3.0` and others) even though a headless node never opens a window. A build on a bare server failed in `gdk-sys` for that reason.

## Decision

1. A Cargo feature `ui` (on by default) holds everything that needs the window: the `tauri` dependency, `tauri-build` in the build script, `run_window` and the embedded web assets in `app/src/lib.rs`, the Windows no-console attribute in `app/src/main.rs`.
2. A build without it is `cargo build --release --no-default-features` (`mise run build-headless`). It has no Tauri, GTK, WebKit or related crates in its dependency tree (checked with `cargo tree`), and the same tests pass.
3. Run without a window, the binary refuses to start in window mode with a one-line message and says to use `--headless`. `--headless`, `--relay` and the `key` commands work as before.
4. The feature is positive (`ui`), not a `noui` flag. Cargo features only add, so a feature cannot remove another feature's dependencies; `--features noui` would still compile Tauri. `--no-default-features` is the way to leave it out.

## Consequences

* `cargo build` and the Tauri CLI (`mise run dev`, `mise run build`) are unchanged: the default is the full app.
* The tests run in both configurations (316 pass each; the window code has no tests of its own).
* The web app (`webapp/`) is built only for the windowed binary; a headless build does not need Node.
