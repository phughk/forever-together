# 0002-verify-qr-phone-scan: Verify a phone scans the addon QR

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0002](../../../tickets/0002-verify-qr-phone-scan.md)
* **ADRs:** [0014](../../adr/0014-functional-tests-only.md), [0018](../../adr/0018-code-format.md)
* **Rounds:** none yet

## Question

How to test whether two phone scanners read the addon's QR, drawn from `SetColorTexture` fills, at UI scales 0.8, 1.0 and 1.2, what counts as a pass, and what changes in `Window.lua` if it fails.

## Files

* `design_draft.md`
