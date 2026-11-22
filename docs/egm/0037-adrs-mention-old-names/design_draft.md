# Pointer to the rename table from the ADR index

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0037](../../../tickets/0037-adrs-mention-old-names.md)
*   **Related decisions:** [ADR 0029](../../adr/0029-atomic-design-and-show-dont-tell.md) (the plain-words list), [ADR 0036](../../adr/0036-how-designs-and-decisions-are-maintained.md) (accepted ADRs are immutable except status; the archive is frozen). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

A chore of one or two lines. ADR 0029 replaced internal words with player-facing ones in the UI (players for peers, backup contacts for trusted peers, phone link for LAN mode and token, searching wider for stage and widening, and others). Older ADRs and the archived design doc keep the old words. The ticket asks that the index `README.md` link the rename list.

## The Problem

*   About 26 files in `docs/adr/` contain "peer", "peers", "token" or "stage" (checked with a word-match search), and so do the files in `docs/design-process/archive/`. A reader who then looks at the UI finds different words.
*   ADR 0029 already says in its Consequences that the old words are history and "the rename table above is current". But `docs/adr/README.md` does not link to it, and the "table" is a bullet, "Plain words (D4)", under the heading `Decision`, with no heading of its own to link to.
*   Editing 26 accepted ADRs is ruled out by [ADR 0036](../../adr/0036-how-designs-and-decisions-are-maintained.md) (immutable except status), and the archive is frozen. The pointer therefore has to live outside them.

## Goals

*   `docs/adr/README.md` links to the rename list in ADR 0029 (the ticket's Done when).
*   The link target is stable: it points at something that will not move when ADR 0029 changes.

## Non-Goals

*   Rewording any ADR or the archived design doc.
*   Changing the code's technical names (ADR 0029: "Code, ADRs and the API keep their technical names").
*   Adding a glossary of every domain term.

## Technical Plan

Add one sentence under the introduction paragraph of `docs/adr/README.md` that links to ADR 0029's list. ADR 0029 has three headings (`Context`, `Decision`, `Consequences`), so a link can only reach `#decision` unless the author adds a heading, which would be an edit to an accepted ADR. A plain link to the file, with the words "Plain words (D4)" in the sentence, finds the bullet without needing an anchor.

## Alternatives Considered

*   **A pointer line inside each affected ADR.** Rejected: 26 edits to accepted ADRs, against ADR 0036.
*   **A pointer in the archive README** (`docs/design-process/archive/README.md`). Not rejected, but it is an edit to a frozen area, and ADR 0036 rule 8 says the archive is legacy. See question 1.

## Detailed Implementation

*   `docs/adr/README.md`: one added sentence after the opening paragraph. No table row changes.
*   Optional: `docs/design-process/archive/README.md`, one sentence under its heading, if question 1 allows it.
*   `tickets/0037-adrs-mention-old-names.md`, `tickets/README.md`: status when done.

No code files change.

## Test and verification plan

*   Open `docs/adr/README.md` in a Markdown renderer and follow the new link; it must open ADR 0029 at (or near) the list.
*   Run a link check by hand: the target file `docs/adr/0029-atomic-design-and-show-dont-tell.md` exists and the sentence names the bullet it points to.
*   Search for the words from the list in the README itself: the README's own text uses "peer" in its introduction row titles (for example 0009's title); the sentence must say those are the old words.

## Open questions for the author

1.  **Whether the pointer also goes in the archive's README.**
    *   **A. ADR index only.** Mechanism: one line in `docs/adr/README.md`. Trade-off: honours "archive is frozen". Fails for a reader who opens an archived file directly and never sees the index.
    *   **B. ADR index plus the archive README.** Mechanism: one more line in `docs/design-process/archive/README.md`. Trade-off: covers both entry points. Fails the strict reading of ADR 0036 rule 8 (frozen), though a README pointer changes no archived content.
2.  **Whether the list becomes a real table.**
    *   **A. Link to the existing bullet.** Mechanism: the sentence names "Plain words (D4)". Trade-off: no ADR edit. Fails if ADR 0029 is later reworded and the name no longer matches.
    *   **B. Move the list into a small glossary file** (for example `docs/glossary.md`, a table of old word, UI word, where it applies) and link that from the index. Mechanism: copy the list, ADR 0029 stays as it is. Trade-off: a stable target and a real table, as the ticket words it. Fails by creating a second copy of the list that can drift from ADR 0029, which is the decision record, and the ADR would win on disagreement.
