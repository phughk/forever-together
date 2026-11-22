# 0022-encrypted-lan-link: encrypt the phone link

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0022](../../../tickets/0022-encrypted-lan-link.md)
* **ADRs:** [0012](../../adr/0012-one-web-app-for-phone-and-desktop.md), [0032](../../adr/0032-settings-phone-link-and-advanced.md), [0034](../../adr/0034-system-overview-goals-and-risks.md), [0023](../../adr/0023-no-supernodes.md)
* **Rounds:** none yet

## Question

Whether the phone link over the local network stays plain HTTP with its warning, or moves to an encrypted transport, and if so how a phone's browser can be made to trust a certificate for a private address without a hosted service. The draft is short because the choice is mostly the author's, and the most useful part is what each option costs.

## Files

* `design_draft.md`
