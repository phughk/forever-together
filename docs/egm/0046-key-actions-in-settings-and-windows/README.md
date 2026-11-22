# 0046-key-actions-in-settings-and-windows: Key actions in Settings, and the key commands on Windows

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0046](../../../tickets/0046-key-actions-in-settings-and-windows.md)
* **ADRs:** [0039](../../adr/0039-key-management.md), [0034](../../adr/0034-system-overview-goals-and-risks.md) (loopback-only list), [0032](../../adr/0032-settings-phone-link-and-advanced.md), [0016](../../adr/0016-peers-identified-by-key.md); earlier pass: [0043-key-management](../0043-key-management/README.md)
* **Rounds:** none yet

## Question

Which key actions (show the public key, import, export, reset) appear in Settings, how the secret key is handled so the browser does not see it (or what is accepted if it does), how a key change is applied to the running app, and what a Windows user gets, given that Windows release builds print nothing from the `key` commands.

## Files

* `design_draft.md`
