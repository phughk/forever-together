# Submodule clone for `mise run test-addon`

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0036](../../../tickets/0036-mise-test-addon-submodule.md)
*   **Related decisions:** [ADR 0021](../../adr/0021-test-vm-referenced-from-wow-forever-addons.md) (the test VM is referenced from the `wow-forever-addons` repository). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

A chore. The addon tests run in a headless Lua VM that lives in another repository, mounted as the submodule `vendor/wow-forever-addons`. `.gitmodules` gives its URL as a local absolute path on the author's machine (`/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons`). Recent git versions refuse a submodule whose URL uses the `file` transport unless `protocol.file.allow` is set, so the first `git submodule update --init` fails with "transport 'file' not allowed".

## The Problem

*   The task `test-addon` in `mise.toml` starts with `git submodule update --init vendor/wow-forever-addons`, which fails on a fresh clone for the reason above. The workaround, `git -c protocol.file.allow=always submodule update --init vendor/wow-forever-addons`, is already written in `README.md` (the `mise run test-addon` row) and in `CLAUDE.md`. So the ticket's "document" part is done; the open part is "consider a task that does it".
*   The URL is a path that exists only on the author's machine. On any other machine the clone fails for a second reason, even with the transport allowed. This draft cannot tell whether other machines matter (the repository may stay single-author); that is question 1.
*   `mise run clean` removes `vendor/wow-forever-addons/wowluavm/target`, so the clone is expected to be reused, not repeated.

## Goals

*   A fresh clone passes `mise run test-addon` with at most one documented step (the ticket's Done when).
*   The step is stated in one place that `README.md` and `CLAUDE.md` both point to, instead of two copies.

## Non-Goals

*   Moving the Lua VM into this repository ([ADR 0021](../../adr/0021-test-vm-referenced-from-wow-forever-addons.md) decided to reference it).
*   Changing how the VM is built or how the tests run (the `cargo build` and `wowluavm` lines of the task).
*   CI. There is none in the repository.

## Technical Plan

The task already has the structure `git submodule update --init ...`, then build, then run. Whatever the answer to the question below, the change is confined to that first line, the URL in `.gitmodules`, or the documentation. If the transport is allowed per command with `-c`, nothing is written to any git configuration, so the permission does not outlive the command.

## Alternatives Considered

*   **Set `protocol.file.allow=always` globally.** Rejected: it widens a protection git added on purpose (the file transport in submodules was restricted after a published local-clone vulnerability) for every repository on the machine. A per-command `-c` limits it to this one clone of a repository the author owns.
*   **Vendor the VM by copying it in.** Rejected by [ADR 0021](../../adr/0021-test-vm-referenced-from-wow-forever-addons.md).

## Detailed Implementation

Depends on the answer. Files that exist and could change:

*   `mise.toml`: the first command of `[tasks.test-addon]`, from `git submodule update --init vendor/wow-forever-addons` to the form with `-c protocol.file.allow=always` (option A), or unchanged (options B and C). Optionally a separate `[tasks.init-submodules]` that `test-addon` lists in `depends`, so `mise run init-submodules` can be run alone.
*   `.gitmodules`: the `url` line, option B only. A changed URL needs `git submodule sync` in existing clones, which the author must run once; add it to the documentation of that change.
*   `README.md`: the `mise run test-addon` table row, which today carries the whole workaround sentence. Reduce it to one pointer once the task handles it (A or B).
*   `CLAUDE.md`: the "Commands" bullet about the submodule URL. This is the project instruction file; the author edits it, not the draft.
*   `tickets/0036-mise-test-addon-submodule.md` and `tickets/README.md`: status, when done.

## Test and verification plan

*   On a scratch clone made with `git clone` into a temporary directory (outside the repository), run `mise run test-addon` with no preparation. Expected after the change: the submodule is cloned, the VM builds, the Lua tests in `addon/tests/` (five test files plus `setup.json`) run and exit 0.
*   Run it a second time in the same clone: expect no error and no repeated clone.
*   Run it in the main working copy, where the submodule is already initialised (`git submodule status` shows a commit, not a leading `-`): expect no change in behaviour.
*   Fail case: for option B, clone with the old `.gitmodules` URL on a machine where the path does not exist and confirm the failure message the documentation describes.

## Open questions for the author

1.  **How the submodule is obtained.**
    *   **A. Allow the file transport inside the task** (`git -c protocol.file.allow=always submodule update --init ...`). Mechanism: the permission applies to that one command. Trade-off: a one-line change, the URL stays local. Fails on any machine where the local path does not exist (for example a CI runner or a second developer), because the URL itself is unreachable.
    *   **B. Point `.gitmodules` at a remote URL** (a hosted copy of `wow-forever-addons`). Mechanism: the transport is ordinary, the problem does not occur. Trade-off: works on every machine. Fails if the repository has no public or reachable remote (unverified here), and every existing clone needs `git submodule sync`.
    *   **C. Keep the manual step and document it better.** Mechanism: no code change; one sentence in `README.md`. Trade-off: nothing to maintain. Fails the ticket's "one documented step" only if the sentence is missed, which is the current failure.
