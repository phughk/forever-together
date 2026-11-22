# Supply the addon page URL

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0011](../../../tickets/0011-addon-page-url.md). Related: [ticket 0012](../../../tickets/0012-addon-releases-and-updates.md) (release flow).
*   **Related decisions:** [ADR 0031](../../adr/0031-one-screen-per-state-and-click-to-search.md) (first run, `ADDON_URL`), [ADR 0028](../../adr/0028-the-desktop-app-installs-the-bundled-addon.md) (the app installs the bundled addon), [ADR 0002](../../adr/0002-addon-shows-data-only.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

Step 1 of the first run says "Get the addon". `webapp/src/lib/links.ts` holds `ADDON_URL: string | null = null`, and `CharacterPanel` renders the step as a link only when the value is set (the `addonUrl` prop defaults to `ADDON_URL`). Today it is plain text. This is a prerequisite ticket: the code change is one line, and the work is publishing a page for it to point at.

## The Problem

A new player on the first-run screen is told to get an addon and given nowhere to get it. The desktop app can install the addon it carries (ADR 0028), but only from Settings, and a phone cannot reach that. The link is the one path that works for every player and every device.

## Goals

*   The first-run step 1 links to a page where a player can obtain the addon.
*   The page exists, is public, and stays at the same address across releases.

## Non-Goals

*   The release process and update tracking (ticket 0012).
*   Installer changes or a first-run install button (ticket 0007).
*   Verifying that the Forever client loads addons installed from that page. That is part of [ticket 0006](../../../tickets/0006-verify-addon-installer-real-install.md).

## Technical Plan

The author creates the page first. Then one constant changes. No server, API or schema change is needed. The existing story `FirstRunWithAddonLink` in `CharacterPanel.stories.tsx` already shows the linked state with an invalid example address, and `FirstRun` shows the plain state, which stays valid for a build with no URL.

## Alternatives Considered

*   **A link to a file download inside the app's own server.** Rejected: the first run is also shown on phones, and the desktop server's addon files are a developer convenience. No decision backs it and it adds a route.
*   **Removing step 1 when the URL is empty.** Rejected for now: the step also carries the instruction order (get addon, type `/wowft`, paste). Not a recorded decision, so noted here only.

## Detailed Implementation

*   `webapp/src/lib/links.ts`: set `ADDON_URL` to the chosen address. Change the doc comment, which says the author supplies it. `ISSUES_URL` in the same file already points to `https://github.com/phughk/forever-together/issues/new`, so the repository address is known.
*   `webapp/src/components/organisms/CharacterPanel.stories.tsx`: the comment above `FirstRun` says step 1 "is plain text because ADDON_URL is not set yet". Reword it when the constant is set; replace the invalid example address in `FirstRunWithAddonLink` with the real one or keep it, since stories should not depend on a live address.
*   `docs/adr/0031-one-screen-per-state-and-click-to-search.md`: its first-run bullet states that `ADDON_URL` is empty until the author supplies it. ADRs are immutable except for status ([ADR 0036](../../adr/0036-how-designs-and-decisions-are-maintained.md)), so record the change in the ADR that ticket 0012 produces, or in a new one, not by editing 0031. This is an author's choice and is not decided here.
*   `tickets/0011-addon-page-url.md`, `tickets/README.md`: status when done (not part of this draft).

No Rust change. No new files.

## Test and verification plan

*   `pnpm --dir webapp typecheck` and `pnpm --dir webapp build-storybook`.
*   Open the built first-run story and check that step 1 is a link with `target="_blank"` and `rel="noreferrer"` (that markup is in `CharacterPanel.tsx` already).
*   Follow the link from a clean browser profile and a phone: the page loads without a login and offers the addon.
*   Not verifiable here: that a player on the Forever client can use what the page offers (ticket 0006).

## Open questions for the author

1.  **Which page the link points to.**
    *   **A. The GitHub release page of this repository** (the latest release). Mechanism: a release with the addon zip attached, linked as `releases/latest`. Trade-off: no second account, versions match the tags the author already plans to use ([ADR 0028](../../adr/0028-the-desktop-app-installs-the-bundled-addon.md) mentions "GitHub tags and release artefacts"). Fails if the repository is private or the player cannot read GitHub's release page, or if there is no release yet when the app ships.
    *   **B. A CurseForge project page.** Mechanism: a project for the addon with its own file list. Trade-off: players know the site and its client can install addons. Fails if CurseForge does not list addons for the Forever client (unverified in this repository) or if moderation delays the page.
    *   **C. The addon's folder in the repository** (`addon/ForeverTogether` on GitHub). Mechanism: a link to the source tree, no release step. Trade-off: exists today. Fails because a player gets source files rather than a zip, and the link shows whatever the main branch holds, including unreleased changes.
2.  **Whether the page must exist before the app's first release, or the link may stay plain text.** Option 1 sets it before release and blocks the release on the page. Option 2 ships with plain text and sets the link in a later release. Option 1 fails if ticket 0012 is not decided in time; option 2 fails because the first-run screen is the first thing a new player sees.
