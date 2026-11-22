# Verify that no variation scrolls the page

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0033](../../../tickets/0033-ui-scroll-edge-cases.md), related to [ticket 0034](../../../tickets/0034-empty-space-components.md) and [ticket 0035](../../../tickets/0035-web-app-only-mock-tested.md)
*   **Related decisions:** [ADR 0029](../../adr/0029-atomic-design-and-show-dont-tell.md), [ADR 0031](../../adr/0031-one-screen-per-state-and-click-to-search.md), [ADR 0032](../../adr/0032-settings-phone-link-and-advanced.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

This is a verification ticket. ADR 0029 sets the viewport rule: a screen that needs scrolling to be understood is too big, and primary actions stay in view at 1280x720 and 390x844. ADR 0031 says the finder does not scroll in any measured variation. The ticket records a measurement on 2026-11-21 for "the cases tried" and asks that it be redone after any layout change. This draft turns that into a repeatable procedure. How the earlier measurement was made, and exactly which cases it covered, is not recorded in the repository (no script or result file was found), so treat it as unverified.

## The Problem

*   There is no list of which stories were measured, no stated rule for what counts as scrolling, and no place where results are kept.
*   The layout deliberately contains inner scroll regions: the dungeon list on the idle screen (`.ft-picker__region` in `webapp/src/styles/finder.css`, "the page is exactly one viewport tall and the dungeon list takes what is left, scrolling inside itself") and the middle of the QR sheet (`.ft-phonelink__scroll` in `webapp/src/styles/settings.css`). A raw "does anything scroll" test would fail on these by design.
*   Stories at the page and template levels are the test surface. The story files are `webapp/src/components/pages/FinderPage.stories.tsx` (about 60 stories), `SettingsPage.stories.tsx`, `NotPairedPage.stories.tsx`, `App.stories.tsx` and `webapp/src/components/templates/Templates.stories.tsx`. Several stories are phone variants that set their own viewport, and those must be measured at 390x844 as well as at 1280x720, not only at the size they set.

## Goals

*   A written list of what is measured, at both sizes, with exact pass and fail criteria.
*   A result record that says what was measured, when, against which commit.
*   The ticket's named cases are all in the list: the longest names, the out-of-date notice, and the QR sheet.

## Non-Goals

*   Fixing a failing variation. A fail becomes a layout change, and that is a separate change (and a re-run of this plan).
*   Real devices. Real phones are ticket 0035 and ticket 0002. This plan uses browser viewports.
*   Other sizes (wide, tall, tablet). Those belong to ticket 0034.

## Technical Plan

Setup:

1. Build Storybook: `pnpm --dir webapp build-storybook`, then serve the output, or use `mise run storybook` for a dev server on port 6006 (`webapp/package.json` `storybook` script). Record the commit hash.
2. Use a browser with a controllable viewport. The Playwright MCP tools are available in the author's setup (`browser_resize`, `browser_navigate`, `browser_evaluate`); any browser with devtools device emulation works.
3. Open each story by its iframe URL (`iframe.html?id=<story id>&viewMode=story`), so Storybook's own chrome does not take space. Story ids are the lowercased title and export name, for example `pages-finder--long-names-on-phone`.

Steps, for each story in the list below and each of the two viewports (1280x720, 390x844):

1. Resize the viewport, load the story, wait for fonts and the play function to finish (the Settings stories open the QR sheet in a play function, `addAPhone`).
2. Evaluate the page height against the viewport height and the page width against the viewport width.
3. Record pass or fail and the two numbers.
4. For a fail, take a screenshot and note which element overflows.

Stories to measure (names verified in the story files on 2026-11-22):

*   Finder, long text: `LongFirstNameAndSurname`, `LongNamesOnPhone`, `PartyFinalLongNames`.
*   Finder, out-of-date notice: `StaleCode`, `StaleCodeOnPhone`, `IdleOutOfDateUpdate`.
*   Finder, the other states at both sizes: `FirstRun`, `Idle`, `IdleThirtyDungeons`, `Searching*` (all), `Ended*` (all), `PartyFound*` (all), `PartyFinal`, `PartyDropped*`, `ConnectionLost`, `ChangeCharacterFormOpen`, `ChangeCharacterRefusedCode`.
*   Settings: `Defaults`, `AddonUpdateAvailable`, `AddonSeveralFolders`, `AddonInstallFailed`, `PhoneLinkOnWithQr`, `SeveralAddresses` (the QR sheet), `AdvancedOpen`, `AdvancedOpenWithContacts`, `ManyContactsLargeCache`, `ViewedOnPhone`, `SaveRefused`.
*   Not paired: `Steps`, `OnPhone`.
*   Templates: all eight in `Templates.stories.tsx`.

Pass criteria (exact):

*   Page: the document's scroll height is no more than the viewport height, and its scroll width is no more than the viewport width, at both sizes. A variation that fails this is a fail.
*   Primary action in view: the story's main action button (Stop, Accept, Done, Sign out, install) lies fully inside the viewport. This is the test ADR 0029 states.
*   Inner scroll regions are allowed only where the CSS above defines them; they are reported as "inner scroll" and not as a failure, subject to open question 1.

What to record: a table in the place the author chooses (open question 2) with columns story, viewport, document height, viewport height, document width, result, note; plus date, commit hash, browser and version.

On each outcome:

*   All pass: update the ticket's "measured" date and add the result record. No ADR changes. ADR 0029's "Not measured" line about sizes beyond the two viewports still stands.
*   Any fail: list the failing variations in the ticket as the remaining work, fix them (a layout change), and re-run the full list, because a layout change can move any other story.
*   A scroll region found that is not in the allowed list: the author decides whether it is intended; if so it goes in the allowed list, if not it is a fail.

## Alternatives Considered

*   **Measure only the stories that changed.** Rejected: the ticket asks for a recheck after any layout change, and a shared template or CSS rule (`.ft-page`, `.ft-page__sections` in `webapp/src/styles/templates.css`) affects all pages.
*   **Count a story as a pass if the screenshot looks right.** Rejected: it cannot be repeated or compared.

## Detailed Implementation

This plan changes no file. If the author picks a scripted check in open question 2, the new file is named there. No code changes are needed to run it manually.

## Test and verification plan

The plan is the verification. To confirm the plan itself works, run it once on the current commit and compare with the 2026-11-21 result in ADR 0031 (no scroll for the finder in any measured variation) and ADR 0032 (Settings does not scroll at 1280x720 with Advanced collapsed). If a story that those ADRs call clean fails here, the first suspect is the method (for example a story that sets its own viewport or a play function not finished) before the layout.

## Open questions for the author

1.  **How are scroll regions that exist by design judged?**
    *   **A. Allowed list.** Mechanism: only the dungeon list (`.ft-picker__region`) and the QR sheet middle (`.ft-phonelink__scroll`) may scroll internally, and the check reports them as inner scroll. Trade-off: the rule is exact. Fails when a third region is added or a listed one grows to hide the primary action, and the list is not updated.
    *   **B. No inner scroll at all in these stories.** Mechanism: any element with overflow scrolling counts as a fail. Trade-off: strictest reading of ADR 0029. Fails for the idle screen with 30 dungeons (`IdleThirtyDungeons`), which scrolls inside itself by design (ADR 0031 does not say whether that is acceptable beyond "the page does not scroll").
    *   **C. Inner scroll allowed if the primary action stays in view.** Mechanism: only the criteria above apply. Trade-off: judges what ADR 0029 actually states. Fails when the content that scrolls is needed to understand the screen, which the check cannot see.
2.  **How is the check kept repeatable?**
    *   **A. Manual routine**, results written into the ticket or `docs/spike-s0.md`. Trade-off: no new code. Fails because it is skipped when a layout change is small.
    *   **B. A Storybook play function or test-runner pass** that asserts the criteria at both sizes in each story. Mechanism: a shared helper in `webapp/` used by the page stories. Trade-off: runs with Storybook, no extra tool. Fails because ADR 0029 notes the play functions run in Firefox only against the built Storybook and a viewport-dependent check may need per-story viewport parameters.
    *   **C. A script in `scripts/`** (or a `mise` task) that drives a headless browser over the story list and writes a table. Trade-off: one command, comparable output. Fails because it adds a browser dependency to the toolchain pinned in `mise.toml`.
