# Verifying that a phone scans the addon QR

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0002](../../../tickets/0002-verify-qr-phone-scan.md)
*   **Related decisions:** [ADR 0014](../../adr/0014-functional-tests-only.md) (the scan is a manual check), [ADR 0018](../../adr/0018-code-format.md) (code format), [ADR 0002](../../adr/0002-addon-shows-data-only.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

The addon window (`/wowft`) shows the character code as text and as a QR. The QR matrix is produced by `addon/ForeverTogether/Qr.lua` (alphanumeric mode, error correction level M, versions 1 to 10) and checked against two independent encoders in `docs/rdf-code-vectors.json`. The drawing is done in `addon/ForeverTogether/Window.lua` (`Window.DrawQr`): one black `SetColorTexture` square per dark module on a white square, with a quiet zone of `QUIET = 4` modules, in an area of at most `QR_AREA = 230` UI units. Each module is a whole number of screen pixels (`px = floor(QR_AREA * scale / total)`, where `scale` is the frame's effective scale and `total` is the matrix size plus eight).

What is not checked: that a real phone camera, looking at a real game window, decodes this drawing. The test VM cannot show that.

## The Problem

A typical code is about 70 characters, which is QR version 3 or 4 (29 or 33 modules, 37 to 41 with the quiet zone). At `QR_AREA` 230 and a UI scale of 1.0 on a 1080p screen, that is about 5 to 6 pixels per module. At UI scale 0.8 the same 230 UI units are fewer screen pixels per module, and at 1.2 more. Phone cameras decode small modules poorly, and the white background is a single flat texture next to the dark game scene, so the contrast and the quiet zone are what matter. Rounding of module size to whole pixels removes seams, but nothing shows it is enough.

## Goals

*   A recorded result for two phone scanners at UI scales 0.8, 1.0 and 1.2.
*   If a scan fails, a defined change to `Window.lua` and a re-run.

## Non-Goals

*   An in-app camera scanner (ADR 0034 non-goals; the phone uses its own camera app).
*   Changing the code format or shortening the code to fit a smaller QR (a format change is an ADR 0018 matter).
*   Testing every phone model.

## Technical Plan

### Setup

*   A running Forever client with the addon installed (by the app's Addon card or `mise run install-addon`), a character that produces a code (the QR is only drawn when the code builds).
*   Two scanners. They should differ in kind, for example the phone's built-in camera app and one third-party scanner app. The author chooses which (see open question 2).
*   A way to set the UI scale: the game's graphics setting for UI scale, and a way to read it back (Window logs `Window.modulePixels` and `Window.moduleSize`; read them with a one-line `/dump` of those fields, which needs no addon change).
*   Test codes: use the author's real character for the live run. Nothing from this run is committed except the result table with no names.

### Steps (repeat for each UI scale 0.8, 1.0, 1.2, then for the longest code case)

1.  Set the UI scale, `/reload`, `/wowft`. Note the dark-module count, `Window.qr.version`, and `Window.modulePixels`.
2.  Hold scanner A at a normal reading distance (about 30 cm from the monitor) and aim at the QR. Count three attempts, each from a fresh frame (move the phone away and back). Success means the scanner shows text equal to the text box content (compare by pasting into the app: the app decodes it and shows the character).
3.  Repeat with scanner B.
4.  Repeat with the game window not full screen (windowed) if the author normally plays that way, since monitor scaling changes the pixel grid.
5.  Repeat once with the longest plausible code: a character with a long first name and surname, a long zone, a long realm string, so the QR reaches version 5 to 8. The synthetic name "Testa Fakename" in Testville is too short to test this; construct one with the addon test helper if the author has no long-named character, or accept the maximum-size test as covered by the vector alone and say so.

### Pass and fail criteria

*   **Pass for a scale and scanner:** at least two of three attempts decode to exactly the text in the box, within 5 seconds each.
*   **Pass for the ticket:** both scanners pass at all three scales for the typical code.
*   **Fail:** any scanner and scale below two of three. Record the scale, `Window.qr.version`, `Window.modulePixels`, distance and lighting.
*   The decoded text must be byte-equal, not just "a scan happened". A QR that decodes with a wrong character is a fail and points at the encoder, not the drawing; the encoder is already checked against two independent encoders, so this is unlikely.

### What to record, and where

In `docs/spike-s0.md`, the row "A phone camera scans a QR drawn with `SetColorTexture`...": change "unverified, needs phone" to the result, with the date, the phone and scanner names, the three scales, the version and the pixels per module seen, and pass or fail. Add the pixels-per-module table (scale to px) because it explains any marginal result.

### What changes on a fail

The ticket says "the QR is changed if it fails". The candidates, none chosen here:

*   Larger area: raise `QR_AREA` and perhaps the frame size (470 x 470 today) so modules get more pixels.
*   Larger quiet zone (`QUIET`) or a pure white border.
*   Text only: remove the QR and keep the text box, the Copy-and-paste path in the README.

## Alternatives Considered

*   **Test only in the VM with a QR decoder.** Rejected: the VM does not draw pixels, and ADR 0014 names the phone scan as a manual check.
*   **Draw the QR as a single image texture generated outside the game.** Rejected: the QR depends on the live character, and an addon cannot load a generated image at run time.
*   **Skip the check and ship text only.** Not rejected by any ADR; it is the fallback of a fail.

## Detailed Implementation

Nothing is built for the check itself. Files that may change afterwards:

*   `docs/spike-s0.md`: record the result (always).
*   `addon/ForeverTogether/Window.lua`: `QR_AREA`, `QUIET` and the frame size, only on a fail. The module-size rule in `Window.DrawQr` stays whole pixels.
*   `addon/tests/` (the existing Lua VM tests for the window, if any assert module size): update if the constants change. Check with grep before changing.
*   `README.md`, section "Getting your character code and QR in game": the last paragraph says the QR is a convenience and text is the fallback; change it only if the QR is removed or its limits change.
*   `tickets/0002-verify-qr-phone-scan.md`: status update.
*   If the QR is dropped, an ADR amendment is needed on 0014 and 0018 (the QR is mentioned in both).

## Test and verification plan

*   Before the run: `mise run test-addon` passes (needs the submodule; see [ticket 0036](../../../tickets/0036-mise-test-addon-submodule.md)).
*   After a `Window.lua` change: `mise run test-addon`, then repeat the full run above.
*   The manual run is the test. Record it as a table with scale, scanner, attempts passed (n of 3), version and pixels per module.

## Open questions for the author

1.  **What is the minimum pass?**
    *   **Both scanners, all three scales, typical code.** Mechanism: the ticket's wording. Trade-off: clearest. Fails when one cheap scanner never decodes a screen QR for reasons that have nothing to do with this drawing, so a fix would be chased for no gain.
    *   **The phone's built-in camera passes at all scales; the third-party scanner is informational.** Mechanism: only the scanner players actually have counts. Trade-off: less effort on a fix. Fails when players use other apps, which is likely on Android.
2.  **Which two scanners?**
    *   **Built-in camera plus one popular third-party app.** Mechanism: covers the default and a common alternative. Trade-off: realistic. Fails when the author owns one platform only, so the other platform's default is not tested.
    *   **One iOS and one Android device.** Mechanism: covers both default cameras. Trade-off: platform coverage. Fails when the author has no second device, so a borrowed phone and a second date for the run are needed.
3.  **If it fails, which fix first?**
    *   **Enlarge the area or the frame.** Mechanism: more pixels per module. Trade-off: keeps the QR. Fails when the code is long enough that version 8 modules at any allowed size stay too small for a phone.
    *   **Drop the QR and keep text only.** Mechanism: removes `Qr.lua` use in the window. Trade-off: no more unverified feature; the app has no scanner anyway. Fails the "phone as a second screen" journey in ADR 0034, which depends on pasting from the phone.
    *   **Test text-on-phone flow first and decide the QR's worth.** Mechanism: measure how often players would use the QR at all. Trade-off: avoids work on a feature few use. Fails because there are no players yet to measure.
