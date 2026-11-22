# 0033. Remove remaining scroll in edge variations

* **Status:** open
* **Kind:** bug
* **Priority:** P2
* **Source:** ADR 0029; ui audit follow-up
* **ADRs:** none
* **EGM:** [initial draft](../docs/egm/0033-ui-scroll-edge-cases/design_draft.md) (2026-11-22, unreviewed)

## What

Verify no variation scrolls the page at 1280x720 and 390x844, including the longest names, the out-of-date notice and the QR sheet. Measured clear on 2026-11-21 for the cases tried; recheck after any layout change.

## Done when

Measured and recorded after each layout change.
