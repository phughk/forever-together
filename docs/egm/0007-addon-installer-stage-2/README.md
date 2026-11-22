# 0007-addon-installer-stage-2: native folder dialog, Linux detection, first-run install

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0007](../../../tickets/0007-addon-installer-stage-2.md)
* **ADRs:** [0028](../../adr/0028-the-desktop-app-installs-the-bundled-addon.md), [0031](../../adr/0031-one-screen-per-state-and-click-to-search.md), [0012](../../adr/0012-one-web-app-for-phone-and-desktop.md)
* **Rounds:** none yet

## Question

How does the installer get its stage 2 parts: how the app opens a native folder dialog from Rust behind a loopback-only route, what Linux detection means when the Forever client's Linux layout is unknown, and where in the first run the install offer appears. The draft also records that ticket 0006 (real-install verification) blocks the parts that depend on facts about real installs.

## Files

* `design_draft.md`
