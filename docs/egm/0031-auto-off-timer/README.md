# 0031-auto-off-timer: phone link auto-off timer

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0031](../../../tickets/0031-auto-off-timer.md)
* **ADRs:** [0032](../../adr/0032-settings-phone-link-and-advanced.md) (names this as out of scope, D9), [0012](../../adr/0012-one-web-app-for-phone-and-desktop.md)
* **Rounds:** none yet

## Question

The phone link serves the app over plain HTTP to the whole network and stays on until switched off, including across restarts. This pass decides what the timer measures (a fixed time, idleness, or a time that must be chosen each time), what happens to it on restart, and whether it is on by default.

## Files

* `design_draft.md`
