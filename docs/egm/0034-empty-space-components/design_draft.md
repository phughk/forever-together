# Verify that components use the available space

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0034](../../../tickets/0034-empty-space-components.md), related to [ticket 0033](../../../tickets/0033-ui-scroll-edge-cases.md)
*   **Related decisions:** [ADR 0029](../../adr/0029-atomic-design-and-show-dont-tell.md), [ADR 0031](../../adr/0031-one-screen-per-state-and-click-to-search.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

This is a verification ticket. An earlier note said components leave empty space on various screen sizes. ADR 0031 then added "wide screens use a two-pane grid for focus screens", and Settings became a card grid. The ticket asks to check wide, tall and phone sizes and either close or list what remains. No measurement of the result is recorded (ADR 0029 says real window sizes beyond 1280x720 and 390x844 are not measured), so everything below the code description is unverified.

## The Problem

"Empty space" has no definition in the ticket, and the layout contains deliberate limits, so a check without a rule would produce opinions. What the CSS does today (`webapp/src/styles/templates.css`, read 2026-11-22):

*   `.ft-page`: width 100%, `max-width: 88rem` (1408 px at a 16 px root), centered. On a 2560-wide window about 580 px of each side is empty by design.
*   Focus screens (finder): below 1100 px a single column of at most 46 rem; from 1100 px a three-column grid (`minmax(0, 46rem)`, `minmax(15rem, 20rem)`, `var(--tap)`) holding the main slot, the character strip and the header controls.
*   Settings: `.ft-page__sections` is `repeat(auto-fit, minmax(min(100%, 24rem), 1fr))` with `max-width: 72rem`, so cards fill the row up to 72 rem. Advanced and the report link span the full row. `.ft-page__narrow` is 32 rem wide.
*   Idle finder: the page is one viewport tall and the dungeon list takes the remaining height (`finder.css`), so on a tall window the list grows and on a short one it scrolls inside itself.
*   Short viewports: content is offset from the top (ADR 0031), which creates empty space below on tall windows.

## Goals

*   Look at each screen at a fixed set of sizes and record what is empty and whether it is the design or a defect.
*   Close the ticket, or list each remaining defect with its size and screen.

## Non-Goals

*   Redesigning layouts. A defect becomes a layout change with its own stories (CLAUDE.md design rules: fix the lower level, not the page).
*   Real devices (tickets 0002 and 0035).
*   The scroll rule, which is ticket 0033.

## Technical Plan

Setup: as in the [0033 plan](../0033-ui-scroll-edge-cases/design_draft.md): build Storybook (`pnpm --dir webapp build-storybook`), open stories through their iframe URL, resize the browser viewport. Record the commit hash.

Viewports (width x height):

| Name | Size | Why |
|------|------|-----|
| Phone portrait | 390x844 | the named phone size |
| Phone landscape | 844x390 | very short, wide |
| Tablet portrait | 768x1024 | between the single-column and grid layouts |
| Tablet landscape | 1024x768 | just below the 1100 px grid breakpoint |
| Laptop | 1280x720 | the named desktop size |
| Desktop | 1920x1080 | wide, common |
| Ultrawide | 2560x1440 | above the 88 rem cap |
| Tall | 1080x1920 | portrait monitor |

Screens (stories): Finder `FirstRun`, `Idle`, `IdleThirtyDungeons`, `Searching*` (one of each kind), `Ended*` (one), `PartyFoundFourAccepted`, `PartyFinal`; Settings `Defaults`, `PhoneLinkOnWithQr`, `AdvancedOpenWithContacts`, `ManyContactsLargeCache`; `NotPaired` `Steps`. Both the story's own viewport parameter and the size above must be set; phone-variant stories are measured at the phone sizes only.

For each screen and size, record: the bounding box of the content area, its width and height as a share of the viewport, and a screenshot. Then a judgment per row from the pass rule (open question 1): fine, by design, or defect.

Defects to look for, from the CSS: a grid column that stays empty (the `.ft-page__strip` column when the strip is absent, see the `:not(:has(...))` rule), a card grid row with one short card and a large gap, a focus-screen main slot that stays at 46 rem while the viewport has a large free area beside it, and vertical gaps from the top offset on tall windows.

What changes on each outcome:

*   No defects: the ticket closes with the table kept (location per open question 2). ADR 0029's "Not measured" line should be amended by the author to say what sizes were measured.
*   Defects: the ticket stays open and lists each one (screen, size, what is empty). Each is fixed at the lowest component level that owns it, then the table is regenerated and ticket 0033 is re-run, because layout changes can bring scroll back.
*   A defect that is actually the 88 rem cap: depends on open question 1; the cap is an ADR-level layout decision, so changing it needs an ADR.

## Alternatives Considered

*   **Decide by eye with no table.** Rejected: not repeatable, and the ticket was opened because earlier eyeballing disagreed.
*   **Test only phone and desktop widths from the ticket text.** Rejected: the grid breakpoint is at 1100 px and the cap at 88 rem, so the interesting sizes are around and beyond them.

## Detailed Implementation

None for the verification. No file is changed. If open question 1 leads to a layout change, the likely files are `webapp/src/styles/templates.css` (page cap, grid columns), `webapp/src/styles/finder.css` and `settings.css`, the templates in `webapp/src/components/templates/` (`FinderTemplate.tsx`, `SettingsTemplate.tsx`) and their stories in `Templates.stories.tsx`. Naming exact edits before the rule exists would be guessing.

## Test and verification plan

The table above is the test. To check the method, include 1280x720 and 390x844 and compare with the finder and Settings layouts described in ADR 0031 and ADR 0032; if those do not match the stated layout, the method (viewport not applied, story not settled) is wrong before the layout is.

## Open questions for the author

1.  **What counts as wasted space for a pass?**
    *   **A. A width cap is design, only unplanned gaps are defects.** Mechanism: the 88 rem page cap, the 46 rem focus column and the 72 rem settings cap are accepted; only empty grid columns, short-card rows and gaps inside the content area count. Trade-off: reading speed on wide screens stays good and the table is short. Fails on an ultrawide window if the author sees the large side margins as the problem the ticket was opened for.
    *   **B. Content fills a minimum share of the viewport** (the author sets a number, per size class). Mechanism: the table records the width and height share and anything under the number is a defect. Trade-off: objective and repeatable. Fails because the number is arbitrary and pushes toward stretched content with long lines on wide screens.
    *   **C. The author judges screenshots**, row by row, at the sizes in the table. Trade-off: it reflects what the author actually sees. Fails on repeatability: the next layout change has no rule to check against.
2.  **Where is the result kept?** Options are a table in the ticket file, a section in `docs/spike-s0.md` (where ticket 0035 expects its record), or a file in the ticket's `docs/egm/` folder. It fails if the place is not one that is re-read after a layout change; the ticket says to recheck then.
