# 0045-replace-dyn-store-with-generics: remove `dyn Store`

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0045](../../../tickets/0045-replace-dyn-store-with-generics.md)
* **ADRs:** [0013](../../adr/0013-single-rust-crate.md), [0039](../../adr/0039-key-management.md), [0008](../../adr/0008-iroh-and-local-storage.md)
* **Rounds:** none yet

## Question

For each place that holds or receives `Arc<dyn Store>` or `&dyn Store`, whether it becomes a generic parameter, a concrete `VaraqStore`, or an enum over the two stores, and whether the churn is worth doing at all given that the trait has no generic methods and the code works.

## Files

* `design_draft.md`
