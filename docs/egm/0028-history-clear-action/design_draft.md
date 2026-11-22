# Clear search history action

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0028](../../../tickets/0028-history-clear-action.md)
*   **Related decisions:** [ADR 0027](../../adr/0027-search-history.md) (retention and privacy), [ADR 0032](../../adr/0032-settings-phone-link-and-advanced.md) (Advanced holds search history controls), [ADR 0035](../../adr/0035-protocol-constants-engine-contract-and-api.md) (API table lists `DELETE /api/history`), [ADR 0038](../../adr/0038-scan-log-and-opt-in-export.md) (a separate log, not yet built). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

The search history is a log of when the player plays, what they queue for, their level and role. ADR 0027 therefore requires a way to delete it and says it never appears in logs, exports, pairing or the API. The ticket asks for the Settings action, with a story and tests.

**Finding: it is built.** Evidence:

*   Store: `history::clear` in `app/src/history.rs` deletes every key under the history prefix, including unreadable ones.
*   Engine side: `Cmd::ClearHistory` and `Cmd::ClearCharacter` in `app/src/app/mod.rs`. Both call `history::clear` and empty `history_cache`; `ClearCharacter` also forgets the character.
*   API: `DELETE /api/history` in `app/src/server/routes.rs`, behind `require_loopback` (a phone gets a refusal), 204 on success. `AppCore::clear_history` in `app/src/server/core.rs`.
*   Web: `clearHistory` in `webapp/src/api/httpApi.ts`, `types.ts` and `mockApi.ts`; `HistorySettings` (`webapp/src/components/organisms/HistorySettings.tsx`) inside `AdvancedSettings`, wired in `SettingsPage` and `App.tsx`; the button is a `ConfirmAction` and is locked on a phone. Stories: `Desktop`, `Confirming`, `ViewedOnPhone`, `ClearFailed`.
*   Tests: `clear_history_deletes_every_entry_and_keeps_the_character` and `a_character_change_is_not_a_result_and_clearing_the_character_clears_history` in `app/src/app/history_tests.rs`; `delete_history_is_loopback_only_and_needs_a_token` in `app/src/server/tests.rs`; an end-to-end `DELETE` at `app/src/integration_tests.rs`.

## The Problem

*   The author has not confirmed the ticket's "Done when" against the code. The ticket is P1 and open.
*   "Never in logs, exports, pairing or the API" is a claim about the whole system, and only partly testable today. The scan-log export (tickets [0039](../../../tickets/0039-scan-log-store.md) to [0041](../../../tickets/0041-scan-log-ui.md)) does not exist yet, so nothing there can leak history, but nothing yet prevents it later.
*   Entries are written at the end of a search. If the player clears history while a search is running, the clear empties the store and the search then writes its own entry when it ends. The ADR does not say whether that is intended.
*   `history::clear` stops at the first failed delete, so a failure part-way leaves some entries. `HistorySettings` shows an error; a retry deletes the rest.

## Goals

*   Check each part of the "Done when" against the code (steps below).
*   Check the privacy claim by search, and leave a note for the scan-log tickets.
*   Decide what a clear means for a search in progress.

## Non-Goals

*   Selective deletion (by date or dungeon).
*   Clearing from a phone (ADR 0027 makes it loopback only).
*   The scan-log clear action (ADR 0038, separate and independent of `ClearCharacter`).

## Technical Plan

No new mechanism. The mechanism is the clear path in ADR 0027's diagram: `ClearCharacter` or `DELETE /api/history` calls `history::clear`, which removes every `history/` key.

## Alternatives Considered

*   **Clear also available from the phone:** rejected in ADR 0027 (loopback only).
*   **Delete only readable entries:** rejected by the code: unreadable entries are the ones a player cannot inspect, so `clear` removes them too.

## Detailed Implementation

No file needs creating. Possible changes:

| File | Change | Why |
|---|---|---|
| `tickets/0028-history-clear-action.md`, `tickets/README.md` | Mark done. Not edited by this draft. | Backlog matches code. |
| `app/src/app/mod.rs` | Only if question 1 chooses to affect the running search. | `Model` owns the lifecycle and the write. |
| `app/src/history.rs` | Only if clear should continue past a failed delete. Mechanical, decided by the implementer. | A part-failure leaves entries. |
| `tickets/0040-scan-log-export-api.md` | Add a line: the export must not include `history/` keys. Left to the author. | The ADR's "never in exports" applies to the future export. |

## Test and verification plan

1.  `cargo test --lib app::history_tests` and `cargo test --lib server::` from `app/`. Pass: the tests named above pass.
2.  `cargo test --lib integration_tests` (about 16 s per test). Pass: the `DELETE /api/history` end-to-end test passes, leaving zero entries.
3.  `pnpm --dir webapp typecheck`, `pnpm --dir webapp build-storybook`. Pass: the four `HistorySettings` stories render; `ViewedOnPhone` shows a disabled button.
4.  Privacy search: grep the source for `history::` and for the history key prefix. Pass: callers are only `app/src/app/mod.rs`, the `history` module, and tests; no logging macro receives an entry (only the existing "search history not read" warning, which prints an error, not an entry); no route returns entries; `SearchView` and `AppState` carry no entry.
5.  Manual on a desktop run with an invented character: finish a search, clear, restart; confirm the store has no `history/` keys (`DELETE` returns 204). Pass: none remain. On a phone browser, the button is locked and a direct `DELETE` returns the loopback refusal.

Record: result per step and the grep output summary for step 4. If a leak is found, open a ticket and keep this one open.

## Open questions for the author

1.  **What does a clear do to a search that is running?**
    *   **A. Nothing special.** Mechanism: the running search writes its entry when it ends. Trade-off: no code; the cleared player gets one fresh entry. Fails if a player clears to remove traces and the in-progress search recreates one, which they did not expect.
    *   **B. Mark the running search as not to be recorded.** Mechanism: `Model` remembers a "cleared at" time and skips recording searches that started before it. Trade-off: the clear is complete; a little state in `Model`. Fails if the flag is not reset on the next search and later searches are lost.
    *   **C. Refuse to clear while searching.** Mechanism: the button is disabled with a short label. Trade-off: simple rule, no surprise; the player must stop first. Fails if the player cannot find out why the button is disabled, or if searches run for the full 10 minutes.
