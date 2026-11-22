# 0036. Document the submodule clone failure for test-addon

* **Status:** open
* **Kind:** chore
* **Priority:** P1
* **Source:** CLAUDE.md
* **ADRs:** none
* **EGM:** [initial draft](../docs/egm/0036-mise-test-addon-submodule/design_draft.md) (2026-11-22, unreviewed)

## What

`mise run test-addon` can fail to clone with 'transport file not allowed' until `git -c protocol.file.allow=always submodule update --init vendor/wow-forever-addons` is run. Consider a task that does it.

## Done when

A fresh clone passes `mise run test-addon` with one documented step.
