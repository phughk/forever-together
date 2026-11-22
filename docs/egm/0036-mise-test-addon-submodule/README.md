# 0036-mise-test-addon-submodule: make `mise run test-addon` work on a fresh clone

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0036](../../../tickets/0036-mise-test-addon-submodule.md)
* **ADRs:** [0021](../../adr/0021-test-vm-referenced-from-wow-forever-addons.md)
* **Rounds:** none yet

## Question

How a fresh clone gets the `vendor/wow-forever-addons` submodule when its URL is a local path that git refuses by default: allow the `file` transport inside the mise task, change the submodule URL, or keep it as a documented manual step.

## Files

* `design_draft.md`
