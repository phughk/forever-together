# 0032-remote-flag-from-server: replace the hostname guess for the phone read-only flag

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0032](../../../tickets/0032-remote-flag-from-server.md)
* **ADRs:** [0032](../../adr/0032-settings-phone-link-and-advanced.md) (records the guess as a known weakness), [0012](../../adr/0012-one-web-app-for-phone-and-desktop.md)
* **Rounds:** none yet

## Question

The web app guesses whether it runs on the desktop from the page's hostname, while the server decides from the caller's socket address. This pass decides how the server tells the page who it is talking to, and whether that is one yes/no value or a set of permissions.

## Files

* `design_draft.md`
