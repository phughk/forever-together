# 0030-per-phone-count: show how many phones are linked

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0030](../../../tickets/0030-per-phone-count.md)
* **ADRs:** [0032](../../adr/0032-settings-phone-link-and-advanced.md) (names this as out of scope, D9)
* **Rounds:** none yet

## Question

The server has one shared token per install, so it does not know how many phones are linked. This pass decides what "a linked phone" means for counting (open connections, recently seen addresses, or per-phone tokens), and whether a P3 convenience justifies the server work.

## Files

* `design_draft.md`
