# 0045. Replace `dyn Store` with generics or enums

* **Status:** open
* **Kind:** chore
* **Priority:** P3
* **Source:** author, 2026-11-21 (egm 0043)
* **ADRs:** none
* **EGM:** [0043-key-management](../docs/egm/0043-key-management/README.md); [initial draft](../docs/egm/0045-replace-dyn-store-with-generics/design_draft.md) (2026-11-22, unreviewed)

## What

The code uses `&dyn Store` and `Arc<dyn Store>` in about 31 places (`config.rs`, `history.rs`, `runtime.rs`, the app layer, the server). The variable implementations are `VaraqStore` and `MemStore`. Replace the trait objects with generics, or an enum over the two stores where a single concrete type must be held (for example `Runtime.store`). New code from ticket 0043 is already written without `dyn`.

## Done when

No `dyn Store` remains outside tests; `cargo clippy --all-targets -- -D warnings` and the Rust suite pass. Decide generics or an enum per call site (an ADR if it changes the module layout, ADR 0013).
