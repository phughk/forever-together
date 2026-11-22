# Search diagnosis: attempt counters

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0025](../../../tickets/0025-diagnosis-attempt-counters.md)
*   **Related decisions:** [ADR 0026](../../adr/0026-search-health-from-own-observations.md) (amendments of 2026-11-21 define the counters and situations), [ADR 0027](../../adr/0027-search-history.md), [ADR 0025](../../adr/0025-leader-invitations.md), [ADR 0003](../../adr/0003-self-reported-data.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

The search diagnosis tells a searching player what this install saw (roles seen, peers connected). The ticket adds what this install did: how many invitations it sent, how many accepts came back, how many invitations it received, how many players were removed from its view because a party took them, and drops and declines. From those it adds the situations `others_grouping` and `nobody_answered`, with copy in the web app.

**Finding: the work is already in the repository.** This draft is therefore a closure check, not a build plan. Evidence:

*   Counters: `engine::state::Attempts` (`app/src/engine/state.rs`) holds `invites_sent`, `invite_epochs`, `accepts_got`, `invites_got`, `taken_seen`; the engine keeps a `BTreeSet<PeerId>` for `taken_seen` so peers are counted once.
*   Classifier: `engine::diagnose::diagnose` (`app/src/engine/diagnose.rs`) with `MIN_ATTEMPTS` (6), `MIN_INVITE_EPOCHS` (2) and `TINY_SAMPLE` (4), and `Situation::OthersGrouping` and `Situation::NobodyAnswered`.
*   Wire: `diagnosis.attempts` and the two situation strings in `app/src/server/core.rs`, built in `app/src/app/mod.rs`.
*   Copy and stories: `webapp/src/lib/diagnosisCopy.ts` (`attemptsPhrase`, chips, headlines) and `webapp/src/components/molecules/DiagnosisSummary.stories.tsx` (stories `NobodyAnswered`, `OthersGrouping`, their ended, multi-dungeon, phone-width and "no attempts from an older server" variants), plus `FinderPage.stories.tsx`.
*   Tests: `app/src/engine/facts_tests.rs` (`unanswered_invitations_give_nobody_answered_after_min_attempts`, `invitees_count_the_invitations_they_got_and_leaders_the_accepts`, `a_party_forming_next_to_a_searching_node_shows_others_grouping`, `the_counters_are_frozen_at_expiry_and_reach_the_history_facts`), a table test in `diagnose.rs`, and a server test with an attempts payload in `app/src/server/tests.rs`.

All thresholds are unverified against a live network (ADR 0026).

## The Problem

*   Whether the ticket's "Done when" (engine counters, `diagnosis` fields, copy and stories, tests with invented data) is met has not been checked by the author.
*   The tickets list still shows it open, so the backlog misstates the state of the code.
*   `nobody_answered` can only fire for a node that sends invitations. By ADR 0025 a tank invites; a healer or damage player sends none, so `invites_sent` stays 0 for them. Such a player gets `invites_got` counted but nothing built on it. The ticket text does not say whether that matters.

## Goals

*   Confirm each "Done when" item against the code and tests, with the checks listed below.
*   Settle whether the ticket closes now or waits for live thresholds.
*   Decide what a non-leader sees when invitations arrive but no party forms.

## Non-Goals

*   Changing the thresholds. They are unverified and can only be tuned from a live run (ticket [0005](../../../tickets/0005-verify-agreement-live.md)).
*   Telling why an invitee did not accept. ADR 0026 deliberately does not count it and adds no message for it.
*   Hysteresis, a doubt level or hints (ticket [0026](../../../tickets/0026-diagnosis-hysteresis-and-hints.md)).

## Technical Plan

No new mechanism. The existing one: the engine increments counters where it emits or accepts invitations, freezes them in the end snapshot before `stop()`, `diagnose` reads them as pure input, and the web app words the result. ADR 0026 has the diagram.

## Alternatives Considered

None for the closure check. Alternatives for the non-leader case are in the open questions.

## Detailed Implementation

No file is created. Changes depend on the author's answers:

| File | Change | Why |
|---|---|---|
| `tickets/0025-diagnosis-attempt-counters.md` and `tickets/README.md` | Set status to done with a note pointing at the code above. Not edited by this draft. | The backlog should match the code. |
| `docs/adr/0026-search-health-from-own-observations.md` | Only if open question 1 or 2 changes a decision: an amendment. | ADRs hold decisions (ADR 0036). |
| `app/src/engine/diagnose.rs`, `webapp/src/lib/diagnosisCopy.ts`, stories | Only if open question 2 picks an option that adds a situation. | New situation needs classifier, copy and stories. |

## Test and verification plan

Setup: repository at `main`, mise tools installed.

1.  `cargo test --lib engine::` from `app/`. Pass: all tests in `facts_tests` and `diagnose` pass, including the four named above.
2.  `cargo test --lib server::`. Pass: the attempts payload test passes.
3.  `pnpm --dir webapp typecheck` and `pnpm --dir webapp build-storybook`. Pass: no errors; the stories listed above render.
4.  Read the `Attempts` struct (`app/src/engine/state.rs`) and the `attempts` type in `app/src/server/core.rs`. Pass: counts only, no peer id or name leaves the engine (ADR 0026).
5.  Check that the test data uses invented characters only (CLAUDE.md rule).

Record: pass or fail per step. On all pass: close the ticket per the author's answer to question 1. On a failure: keep it open and file the failure as its own ticket.

## Open questions for the author

1.  **When does the ticket close?** The thresholds `MIN_ATTEMPTS`, `MIN_INVITE_EPOCHS` and `TINY_SAMPLE` are guesses.
    *   **A. Close now.** Mechanism: the ticket's "Done when" is met; threshold tuning stays with ticket 0005. Trade-off: the backlog shows no open diagnosis work. Fails if the live run shows the counters are the wrong signal and nobody remembers they were closed unverified.
    *   **B. Keep open until the live run.** Mechanism: close it when ticket 0005 records live numbers. Trade-off: an open ticket for finished code. Fails if the live run is delayed for months and the ticket becomes noise.
    *   **C. Close and open a small threshold ticket.** Mechanism: a new verification ticket lists the three constants and what to record. Trade-off: one more ticket. Fails if it duplicates the thresholds list in ADR 0026 and the two drift.
2.  **What does a healer or damage player see when invitations arrive and no party forms?** Today nothing special: `nobody_answered` needs `invites_sent`.
    *   **A. Nothing.** Mechanism: status quo, the role checks and `healthy` apply. Trade-off: no extra code. Fails if a player is invited repeatedly by a leader whose party never commits and sees "healthy" for minutes.
    *   **B. A new situation from `invites_got`.** Mechanism: invitations kept in at least N epochs, no party committed. Trade-off: another threshold and another sentence; invitees cannot tell whether the leader was slow or lost the accepts. Fails if the invitation is lost and the invitee never receives anything to count.
    *   **C. A chip only.** Mechanism: show "Invited N times" next to the role row without changing the situation. Trade-off: low risk, less explanation. Fails if players read the chip as a promise that a group is coming.
