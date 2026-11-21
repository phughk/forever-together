# 0029. The web app follows atomic design and "show, don't tell"

* **Status:** Accepted (decided and implemented 2026-11-21). Extends [0012](0012-one-web-app-for-phone-and-desktop.md). Cites [0026](0026-search-health-from-own-observations.md).
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski (the UI audit, decisions D1 to D9, resolved 2026-11-21)

## Context

The first web app explained itself in paragraphs: a card with a dozen lines of text, jargon such as "peer", "token", "bootstrap" and "stage", and pages that scrolled to be understood. A UI audit ranked 16 changes. The author resolved every open decision with the audit's own recommendation.

## Decision

* **Atomic design, covered in Storybook.** Atoms, molecules, organisms, templates, pages. Build upward, put each component at the lowest level that fits, templates stay content-agnostic, pages supply real (invented) content and cover template variations. Storybook mirrors the levels. The rules are in `CLAUDE.md`.
* **Show, don't tell.** A component is understood from its shape, state and labels. Order of preference: a clear label, an icon or visual state, a short value next to its label, one short sentence. A card with more than about two sentences of explanation is redesigned, not trimmed. Warnings state the consequence in one sentence. Explanations of a setting go in a tooltip (`InfoTip`) or a collapsed `Details`, never the default view.
* **InfoTip rule (D5).** InfoTip text is a string of at most about 160 characters. If removing a tip would not change what the player does, the content should have been a visual or a label.
* **Viewport rule.** A screen that needs scrolling to be understood is too big. Primary actions stay in view at 1280x720 and 390x844. Edge variations that scroll are listed as tickets, not hidden.
* **Plain words (D4).** Player-facing words replace internal ones: players (not peers), backup contacts (trusted peers), phone link (LAN mode and token), searching wider (stage, widening), level range (level spread), Accept (Confirm), Updated and Out of date (code age, stale), "Seen from here" (what this computer sees). Code, ADRs and the API keep their technical names.
* **Hedge stays visible (D6).** The honesty of ADR 0026 moves from long sentences to the label "Seen from here" and the `~` prefix on counts. The long reason sits behind a tap. This amends the presentation in 0026, not its rules.
* **Smallest copy for notices.** One sentence that states the consequence, for example the plain-HTTP warning on the phone link.

## Consequences

* Several ADRs and the design doc still quote the old words. They are history; the rename table above is current.
* No unit tests cover the UI. The gates are `pnpm --dir webapp typecheck`, `build` and `build-storybook`, and play functions run in Firefox against the built Storybook.
* Not measured: real phones and real window sizes beyond the two named viewports.
