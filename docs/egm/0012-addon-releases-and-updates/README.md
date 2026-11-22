# 0012-addon-releases-and-updates: addon release flow and update tracking

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0012](../../../tickets/0012-addon-releases-and-updates.md)
* **ADRs:** [0028](../../adr/0028-the-desktop-app-installs-the-bundled-addon.md), [0020](../../adr/0020-one-new-repository-standalone-addon.md), [0018](../../adr/0018-code-format.md), [0023](../../adr/0023-no-supernodes.md)
* **Rounds:** none yet

## Question

Which artefacts make up a release (the app, the addon, or both), where players get the addon, and whether the app ever asks the network if a newer addon exists. The second part changes a stated property of the app (no network use for the addon) and so needs the author's answer before any code.

## Files

* `design_draft.md`
