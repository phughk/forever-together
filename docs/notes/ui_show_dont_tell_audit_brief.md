# "Show, don't tell" UI audit of the webapp: concepts brief

Author: Hugh Kaznowski. Date: 2026-11-21. Mode: brainstorm. The author judges the result.

Reviewer status (all seven completed, none retried): design, codebase-details, zoom-out, comprehension, implementation-readiness, structural, inventive-catalyst. No reviewer rendered the app or measured pixels, so every height and scroll figure below is an estimate from reading code and CSS. The design reviewer's persona file was not at the expected path, so it used `design_critic.md`.

## Overview

The rule (CLAUDE.md, design system, "Show, don't tell."): a component is understood from its shape, state and labels. Order of preference: a label, an icon or visual state, a short value ("Players seen: 8"), one short sentence only where the screen cannot show the fact. More than about two sentences in a card means redesign. A warning states its consequence in one sentence. Explanations of a setting go in a tooltip or collapsed Details. A component that needs scrolling to be understood is too big.

Target player: knows the dungeon finder, does not know or care about peer-to-peer, on desktop or phone. Most players leave the phone link on, a few turn it off. They should be able to configure things when they need to.

Scope read: `App.tsx`, every file in `webapp/src/components` (atoms, molecules, organisms, templates, pages), `lib/diagnosisCopy.ts`, `api/constants.ts`, and `docs/notes/ui_notes.txt`.

Findings in one paragraph: the finder and settings screens explain themselves in paragraphs and in the words of the network layer ("peer", "token", "LAN mode", "bootstrap", "stage", "widening", "this computer sees"). The finder has no height budget, so a character card with nine fields, a dungeon list of unbounded length, and a live search panel stack into two to three screens. The first-run state is a paragraph of instructions plus a disabled list. The diagnosis copy built recently states the same counts three times and hedges inside every sentence.

## Analysis

### What the reviewers agree on

- Click-to-search in the dungeon list, with no Start button and no "Pick one dungeon, or several" sentence (design, zoom-out, structural, implementation). Multi-select is the open question.
- Delete the five-row stage list and the "Network" field from the search card (zoom-out, comprehension, inventive, structural). Stage 1 is a no-op in v1, and the row text says so ("no change in v1").
- Shrink the nine-field character card to a one-line summary with a Stale badge kept outside any collapsed part (all except implementation).
- Move Peers (known-peers size, trusted peers) and Search history behind a collapsed Advanced section. Merge LAN mode and token into one "Phone access" card (design, zoom-out, comprehension, inventive, structural).
- Replace the addon paragraph with a state-driven first run (design, comprehension, structural, inventive). The app cannot know whether the addon is installed. The only fact it has is `character === null` (codebase-details, implementation). So the first-run state can say "Paste your code", it cannot tick "addon installed".
- Third-person wording ("this computer sees", "this app did not reach", "The app does not know the zone") should go (comprehension, design).

### Where they disagree

- Info button: design wants a new atom (`InfoTip`, text capped at about 160 characters, string only, tap to open, Escape and outside tap to close). Zoom-out says native `<details>` and `title` are enough. Codebase-details found that a raw `<details>` exists once (`LanSettings.tsx:47`) and there is no Details or info component, so a second and third use would copy it. Position taken in this brief: build one small atom, because the rule allows an info button and a phone has no hover, but cap its text and use it in few places (see Key Decisions).
- Multi-select: zoom-out would drop it, design and structural keep it behind a mode, implementation flags it as undecided. The engine and `search.dungeons` are plural, and widening stage 4 adds "the other recommended dungeons".
- Diagnosis: zoom-out says leave `diagnosisCopy.ts` alone and change only where it is shown. Design and inventive want the icon row as the main display, a one-sentence headline, and the hedge stated once and structurally. Comprehension wants plain wording. See the reconciliation below.
- Phone count and auto-off (inventive): "2 phones connected" and "off after 1 hour" need per-device identity or a timer in the server, which does not exist (only one shared token). Treated as out of scope.

### Facts that change the audit (from reading the code)

- The search limit is `SEARCH_LIMIT_S = 600`, which is 10 minutes. Two reviewers wrote "5 minutes". Widening steps are 60 s each, so the last stage starts at 240 s.
- The stage clock restarts when a dropped party returns the player to searching, so elapsed time, stage and the limit bar run on different clocks (`SearchProgress.tsx:38-40`). Any single visual must pick one axis.
- A phone is not fully read-only on Settings. Loopback-only on the server: clear history, get pairing, reset token, and changing `lan_mode`, `lan_address`, `trusted_peers`. A phone can change `known_peers_size`. `remote` is guessed from the hostname in `App.tsx:28` and corrected on a 403.
- Players cannot invite anyone themselves in the dungeon finder. The "invited 6, accepted 0" wording in the diagnosis is the app's invitation round (ADR 0025), shown to a player as "invited". Rename it to match what the player sees.
- `PartyPanel` says "Nobody is penalised" for every drop, but an `unreachable` drop skips the peer for about a minute. Keep those two cases apart.
- There are no vitest, playwright or `*.test.*` files in the webapp. The only assertions are story play functions. `ConfirmAction.stories.tsx` clicks "Reset token" and expects "Yes, reset"; check whether it passes its own labels. No story asserts the diagnosis sentences. The length constants `HEADLINE_MAX_CHARS` (220), `DETAIL_MAX_CHARS` (140) and `OUTLOOK_MAX_CHARS` (110) are comments and constants only.
- The failure-reason data the author asked for already exists (`diagnosis.seen`, `attempts`, nine situation codes). The audit needs no new API fields for the diagnosis.

### Concepts that cannot be shown visually

| Concept | Least-text honest treatment | What must stay as words |
| --- | --- | --- |
| Phone link on or off | A switch row "Phone link" with On or Off and a state chip. No "LAN mode" in the default view. | Nothing beyond the label. |
| Plain HTTP risk | A warn chip next to the switch plus one sentence: "Anyone on this Wi-Fi could connect." The long reason goes in the info tap. | The one sentence. It is a security consequence and must be visible, not behind a tap. |
| Pairing token | Never name it. The QR appears on demand ("Add a phone" button or sheet), not on page open. | The word "QR" and the one-line step. |
| Reset token | "Sign out all phones", with a confirm that states the consequence in one sentence. | The consequence: phones must scan again. |
| Trusted peer | Under Advanced, labelled "Backup contacts" (name to be chosen by the author), listed by short id, not 64 hex characters. | The key field for adding one. |
| Known peers size | Under Advanced, a number field labelled "Players to remember". | Nothing else. |
| Search history | "Clear search history" row with a one-line hint in an info tap, under Advanced. | The deletion confirm. |
| "What I can see" hedge | A label "Seen from here" over the counts, and `~` before a rough number. | See reconciliation. |

### Reconciliation: diagnosis copy

The author's earlier brief required hedged, honest wording: every claim is about what this computer sees, numbers are rough, nothing promises a group. That intent stays. What changes is where the hedge lives.

- Say the hedge once, structurally: a label "Seen from here" on the counts row, `~` for a rough count (replaces "about" in every sentence), and the full sentence ("Not a promise that a group will form") in one info tap beside the headline.
- Make the role icon row primary: three roles against what a party needs (1 tank, 1 healer, 3 damage), with your own role pre-filled. Reviewers differ on colour. The inventive reviewer warns that a red empty pip reads as "none exist" instead of "none seen", so use a neutral grey and the "Seen from here" label.
- Reduce the headline to the consequence in one sentence ("No tank seen, so a party can't form yet."). Keep the detail and outlook sentences inside Details.
- Keep short chips for situations the icon row cannot show ("Invited 6, accepted 0", "Others are forming groups").
- Drop sentences that explain a missing field or defend the app: "Rough counts from this computer, not including you.", "The app is working: others nearby are forming groups.", "Others may be searching where this computer cannot see."
- The "healthy" case must still not reassure while a role is missing or the app is offline. The existing rule that drops `usually_found` next to a live problem must carry over to any visual outlook meter.
- Unresolved tension (see objections): a hedge behind a tap may not be read. Keep the label "Seen from here" and `~` visible. They are the load-bearing part.
- Update ADR 0026 with a note saying where the hedge moved. Update the three length constants and their comment together with the copy.

## Goals vs. Non-Goals

Goals

- A ranked list of changes, per component, that a person can accept or reject one by one.
- Default views carry labels, state and short values. Words about how the system works leave the default view.
- Each screen fits a 1280x720 window and a 390x844 phone without scrolling to understand its main action.
- First run tells the player what to do next from state, with no instruction paragraph.
- Jargon renamed or hidden. Settings still reachable.
- Honest wording kept where a consequence matters (plain HTTP, sign out, delete).

Non-goals

- No in-app addon installer, no GitHub release tracking, no issue-report form, no x/y location (from `ui_notes.txt`; separate features). A plain static link to the addon page and a plain "Report a problem" link are allowed.
- No new server fields. Per-phone counts, auto-off timers and an "addon installed" flag are out.
- No change to matching, widening or diagnosis logic. Presentation and copy only.
- No new routing or screens beyond what the viewport fit needs.
- No removal of any setting. Hidden settings stay reachable.
- The LAN address picker as its own feature is not part of this audit. Only its wording and placement are.

## Alternatives Weighed

| Alternative | For | Against | Verdict |
| --- | --- | --- | --- |
| New `InfoTip` atom | Works on a phone, one place to cap text, reuse in many rows | One more component and story set | Chosen, small and capped |
| Native `<details>` and `title` only | No new component | `title` does nothing on touch, the raw `<details>` would be copied nine times | Use `<details>` for longer content only, via a small `Details` molecule |
| Click-to-search for one dungeon, multi-select behind a "Combine" mode | Matches the author's note, fixes the scroll, keeps the plural-dungeon feature | A mode to build and explain with state | Chosen |
| Click-to-search, drop multi-select | Simplest | Loses a real feature (widening stage 4 uses it) | Rejected unless the author says multi-select is unused |
| Keep Start button, trim the sentence | Smallest | Does not meet the author's note | Rejected |
| Settings as tabs | Fits a laptop window | A new tab state and routing at both page and container level | Use a collapsed Advanced section first, tabs only if still too tall |
| Hide Peers entirely with an auto-sized default | Fewest settings | Removes a knob the project exposes on purpose; needs ADR changes | Not now. Advanced section instead |
| Phone count, auto-off, per-phone names | Closest to a normal service | Needs per-device identity on the server | Out of scope, note for later |
| Replace the stage list with a notched bar | Shows widening visually | Two clocks on one bar | Show the limit on the bar, the stage as a small "Searching wider" chip with the rule in an info tap |
| Leave `diagnosisCopy.ts` untouched | No test risk, hedging preserved | Same counts shown three times, long sentences | Change layout first (batch 2), then shorten copy per the reconciliation |

## Ranked audit

Severity: High means a first-run or core-flow player is blocked or misled. Medium means confusing or heavy. Low means wording or tidy-up. Effort: S under half a day of agent work, M about a day, L more. Quoted text is current.

| # | Component, file | What breaks the rule | Severity | Change (what to show instead) | Effort | Depends on |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `organisms/DungeonPicker.tsx`, `molecules/DungeonOption.tsx`, `pages/FinderPage.tsx`, `App.tsx` (`selected` state) | "Pick one dungeon, or several to search them together. Your location is only a hint, nothing is refused." Start button at the bottom of a list of 15 to 30 rows. "Select all recommended" button. | High | The dungeon row is the action: tap a row, the search starts. Drop both sentences and the Start button. Multi-select behind a "Combine dungeons" toggle that adds ticks and a sticky "Search N" bar. Keep "Recommended for you", "You are here" and "Levels 15 to 25". The list scrolls inside its own region. | M | none |
| 2 | `organisms/SearchProgress.tsx`, `molecules/StageStep.tsx` | Five-row stage list with "Level spread up to 6", "Secondary roles allowed (no change in v1)", "Passed". "Next widening in 42 s. The search stops after 10 minutes." Field "Network": "3 other computers connected, found through computers remembered from earlier runs". Field "Players seen: 4 besides you (rough, from this computer)". The card runs about 550 px and Stop sits below the fold on a laptop. | High | One bar for time to the limit, one chip "Searching wider" (rule in an info tap), Stop button pinned at the top of the card. Remove the stage list, the Network field (a status dot in the header covers it), the timing sentences, and the duplicate "Players seen" field (the role icon row shows it). Show the card in place of the dungeon list, not above it. | M | #1, #8 (info tap) |
| 3 | `organisms/CharacterPanel.tsx` (empty state), `organisms/DungeonPicker.tsx` (`needsCharacter`), `pages/FinderPage.tsx` | "Open the ForeverTogether addon in game with /wowft, copy the code with Ctrl+C and paste it here. On a phone, scan the addon's QR with the camera app and paste the text." Beside a disabled list of 15 to 30 rows with "Paste your character code first." | High | First run is three steps in a row: "1 Get the addon" (link), "2 Type /wowft, copy" (`/wowft` in a copy chip), "3 Paste here" (the form), then "4 Pick a dungeon" (greyed until a code is accepted). Do not tick "addon installed", the app cannot know. Do not render the dead dungeon list. Phone: scan icon beside the paste field. | M | needs one constant for the addon link (author to supply the URL) |
| 4 | `organisms/CharacterPanel.tsx` (filled state) | Nine `Field` rows (name, class and race, level, role, item level, realm, region, zone, code age) plus "Everything above is read from your code and is not verified." plus up to two notices. About 560 px. | High | One line: name, class, level, role icon, item level, Stale badge. Realm, region, zone, code age under Details. The "not verified" sentence becomes an info tap on the line. Zone unknown becomes a badge "Zone not recognised". Stale notice becomes one sentence: "Code is 40 min old. Refresh in the addon." Buttons "Update character" and "Remove character" in a small menu. | M | #8, uses Details |
| 5 | `organisms/PeerSettings.tsx`, `pages/SettingsPage.tsx` | Card titled "Peers": "Known peers to remember", "Trusted peers are ordinary peers. The app asks them for other peers only when no remembered peer answers. The built-in ones are not listed here.", "Add a trusted peer (public key)", "64 hex characters", rows of bare 64-character keys. Four network-engineer terms. | High | Move under one collapsed "Advanced" section at the bottom of Settings, with a summary chip ("12 remembered, 0 added"). Rename: "Players to remember", "Backup contacts" (author to pick the name). Show contacts by short id. Remove both paragraphs (one info tap each). A phone sees it read-only or not at all (decision D3). | M | #8, #6 |
| 6 | `organisms/LanSettings.tsx`, `organisms/TokenSettings.tsx`, `molecules/PairingQr.tsx` | Card title "Phone access (LAN mode)". "The link between phone and computer is plain HTTP. On a network you do not trust, someone could read the pairing token. Turn LAN mode off when you are done." Token card: "Each phone holds a copy of this install's token. Resetting it signs out every phone at once. Lost a phone? Reset, then pair the others again." Always-on QR of about 200 px. "Phones connect to X. Not working? Change address". | High | One card "Phone link". Row: switch with On or Off and a warn chip "Not encrypted"; one visible sentence "Anyone on this Wi-Fi could connect." The QR opens from an "Add a phone" button (sheet or inline), closing when done. The address picker moves into that sheet, shown only when more than one address exists. "Reset token" becomes "Sign out all phones" with the one-sentence consequence. Words "LAN", "HTTP" and "token" leave the default view. | M | #8 |
| 7 | `pages/NotPairedPage.tsx`, `organisms/AppHeader.tsx` | "The app refused the request because this browser has no valid token." "On the desktop app open Settings, turn on LAN mode and scan the pairing QR with this phone's camera app. If you reset the token on the desktop, every phone must pair again." Dead Finder and Settings buttons in the header. | High (the only screen a phone with no token sees) | Title "This phone isn't linked yet". Three numbered steps with icons: open Settings on the PC, switch on Phone link, scan the QR. Remove the warn notice and the last sentence. Hide the nav. | S | none |
| 8 | New `atoms/InfoTip.tsx` (and `molecules/Details.tsx`) | There is no info or Details component. Raw `<details>` appears once. A phone has no hover. | Enabler | `InfoTip`: one round button with `aria-label`, opens a small popover on tap, closes on Escape or outside tap, minimum 44 px tap area (the existing small button is 32 px). `children: string`, cap about 160 characters, a dev warning or story fails above the cap. `Details` wraps the existing `ft-details` for longer content. Optional `hint` prop on `SettingRow` and `Card` that renders an `InfoTip` beside the title. | S | none |
| 9 | `molecules/DiagnosisSummary.tsx`, `lib/diagnosisCopy.ts` | Headline over 100 characters: "Over 10 min this computer saw about 8 other players: 0 tanks, 2 healers, 6 damage players. It saw no tank." Then a detail sentence, then an outlook sentence, then the counts row again, then "Rough counts from this computer, not including you." The same numbers appear three times. | Medium | Per the reconciliation: role icon row primary with "Seen from here" and `~`, one-sentence consequence headline, detail and outlook inside Details, hedge sentence in one info tap, short chips for "Invited 6, accepted 0". Plain wording: "Asked 6 players to join, none replied", "They may be offline". Update constants and ADR 0026. | M | #2, #8 |
| 10 | `organisms/PartyPanel.tsx`, `molecules/MemberRow.tsx` | "Five players fit together. Press Confirm if you want to play with them. Waiting on 3 players." "Names appear after all five confirm." Badge "Ready, not confirmed" next to "Waiting". `DROP_COPY`: "...The party was dropped and everyone is searching again. Nobody is penalised." Confirm sits below the fold on a phone during a 15 s window. | Medium | Five role slots with a tick for each confirmed player, a countdown, Accept and Decline buttons first and reachable (fixed at the bottom on a phone). Remove the two sentences. Two badge states only: Waiting, Accepted. Drops: one sentence of cause and consequence ("Someone went offline. Searching again."). Do not say "Nobody is penalised" for the unreachable case. In the final state, a "Copy name" button per player and no repeated names footer. | M | none |
| 11 | `organisms/HistorySettings.tsx` | "This app keeps a private record of your finished searches on this computer. It never shows it and never sends it anywhere; it only uses it to give you better advice." | Medium | One row in Advanced: "Search history" with a "Clear" button and the sentence in an info tap. Keep the confirm. Keep it first in Advanced so it does not look hidden. | S | #5, #8 |
| 12 | Header and navigation: `organisms/AppHeader.tsx`, `molecules/ConnectionStatus.tsx` | Subtitle "Peer-to-peer dungeon finder". Status text "Connected" with no object. The header wraps to about 130 px on a phone. | Medium | Remove the subtitle. Status dot with "Online" or "Offline" in the same row. Settings as an icon button. | S | none |
| 13 | `pages/FinderPage.tsx`, `templates/FinderTemplate.tsx` | Search and party are a banner over the idle finder (character card and full dungeon list stay underneath). A finished search stacks "Search ended" over the whole list. | Medium | One screen per state, chosen by an explicit `screen` value derived from state: first run, idle, searching, party found, party ready, ended. Replace `top`, `left`, `right` with a `main` slot plus a compact strip. Restate the slot constraints in the template doc comment as viewport-fit limits. | M | #1, #2, #3, #4, #10 |
| 14 | `organisms/SearchProgress.tsx` (ended state) | "The search reached its 10 minute limit without forming a party." followed by the diagnosis text, then "Searched: X", then "Clear". Fallback Notice with three sentences for older servers. | Medium | Title "Search ended", result headline and the role icon row, buttons "Search again" and "Change dungeon". Remove the sentence about the limit (the title says it). Delete the `lowCount` fallback notice, it repeats the diagnosis. Rename "Clear" to "OK". | S | #9 |
| 15 | Remote notices in `TokenSettings`, `HistorySettings`, `PeerSettings`, `LanSettings` | "The token can be reset only in the desktop app." "Trusted peers can be changed only in the desktop app." "Change this on the desktop app." The same fact in four cards. | Low | Disabled controls with a lock icon, and one banner at the top of Settings: "Change settings on your PC." The loopback fact lives in one shared atom. | S | #5, #6 |
| 16 | Jargon sweep across `SearchProgress.tsx`, `diagnosisCopy.ts`, `CharacterPanel.tsx`, `PartyPanel.tsx` | "peer", "computers", "this computer", "this app did not reach", "The app does not know the zone", "(claimed by the player)", "Code age", "Stale", "Forget this character". | Low | Rename per the table in the Key Decisions section. Use "players", "you", and plain statements. | S | after #2, #4, #9, #10 so the sweep is not done twice |

## Key Decisions & Staging for Next Steps

### Decisions for the author

- D1. Multi-select: keep it behind a "Combine dungeons" toggle (recommended), or drop it. One accidental tap now starts a search. Stop is one tap, and the choice is frozen while searching, so say if that is acceptable.
- D2. First-run: accept that the app can only show "Paste your code", never "addon not installed", unless a server field is added later. Supply the addon link URL.
- D3. On a phone, show Advanced read-only, or hide it. Today a phone can change only `known_peers_size`.
- D4. Choose the word for trusted peers ("Backup contacts" is a placeholder) and for the phone link ("Phone link" versus "Use on your phone").
- D5. Info tap text: hard cap near 160 characters, string only, and it may hold only what the screen cannot show. Agree the rule that if removing an info tap would not change what the player does, the content should have been a visual or a label.
- D6. Diagnosis: approve moving the hedge to "Seen from here" and `~`, and amending ADR 0026.
- D7. Stage display: one chip "Searching wider" with the rules in an info tap, and no countdown to the next step.
- D8. Settings: Advanced section first. Tabs only if Settings still needs scrolling at 1280x720 afterwards.
- D9. Out of scope, confirm: per-phone count, auto-off timer, installer, issue form.

### Resolutions (auto-resolved at the author's request, 2026-11-21)

The author asked for every open decision to be handled and the work driven to completion. Each decision takes the brief's own recommendation. All are revisable.

- **D1.** Click-to-search for one dungeon, with multi-select behind a "Combine dungeons" toggle. The choice is frozen while searching; Stop stays one tap.
- **D2.** The first run shows "Paste your code" and never claims to know whether the addon is installed. No addon link URL was supplied, so the link lives in one constant (`ADDON_URL`) that is empty for now. While it is empty, step 1 renders as plain text without a link. Supply the URL there.
- **D3.** On a phone, Advanced is shown read-only, with a lock icon on each control that only the desktop may change.
- **D4.** "Backup contacts" for trusted peers, "Phone link" for the LAN feature. Rename table below stands.
- **D5.** InfoTip text is a string, capped at about 160 characters. If removing an info tap would not change what the player does, the content should have been a visual or a label.
- **D6.** Approved: the hedge moves to the label "Seen from here" and `~`. ADR 0026 gets a dated note.
- **D7.** One "Searching wider" chip, the rules in an info tap, no countdown to the next step.
- **D8.** Advanced section first. Tabs only if Settings still scrolls at 1280x720 afterwards.
- **D9.** Confirmed out of scope: per-phone count, auto-off timer, installer, issue form.

### Rename table (proposal for D4 and #16)

| Now | Proposed |
| --- | --- |
| peer, known peers | players, players to remember |
| trusted peer | backup contact |
| token, pair | link, sign out all phones |
| LAN mode | phone link |
| stage, widening, level spread | searching wider, level range |
| Network, bootstrap lines | status dot, hidden |
| Confirm, Ready not confirmed | Accept, Waiting and Accepted only |
| Start search | removed (tap the dungeon) |
| Code age, Stale | Updated, Out of date |
| this computer sees | Seen from here |

### Staging

Batch 0 (first, one agent): `atoms/InfoTip.tsx` with stories (closed, open, at the cap, focus, beside a label), `molecules/Details.tsx`, optional `hint` on `SettingRow` and `Card`. Verify with typecheck and build-storybook.

Batch 1 (parallel, after batch 0, files owned per agent, no overlaps):

- Agent A, Settings: `SettingsPage.tsx`, `SettingsTemplate.tsx`, `LanSettings.tsx`, `TokenSettings.tsx`, `PeerSettings.tsx`, `HistorySettings.tsx`, `NotPairedPage.tsx`, `ConfirmAction.tsx` (copy only), their stories, the Settings export in `Templates.stories.tsx`. Items #5, #6, #7, #11, #15.
- Agent B, search: `SearchProgress.tsx`, `StageStep.tsx`, `DiagnosisSummary.tsx`, `diagnosisCopy.ts`, their stories, ADR 0026 note. Items #2, #9, #14.
- Agent C, finder entry: `DungeonPicker.tsx`, `DungeonOption.tsx`, `CharacterPanel.tsx`, `CodePasteForm.tsx`, their stories. Items #1, #3, #4.
- Agent D, party and header: `PartyPanel.tsx`, `MemberRow.tsx`, `AppHeader.tsx`, `ConnectionStatus.tsx`, their stories. Items #10, #12.

Batch 2 (integrator, sequential): `App.tsx`, `FinderPage.tsx`, `FinderTemplate.tsx`, the Finder exports in `Templates.stories.tsx`, `App.stories.tsx`, `FinderPage.stories.tsx`, `api/types.ts` and `fixtures.ts` if props change. Items #13, then #16. The shared CSS (`styles/atoms.css`, `molecules.css`, `organisms.css`, `templates.css`) is edited in clearly marked blocks per agent or only by the integrator, to avoid conflicts. Props wired in `App.tsx` change when `selected` state goes away (#1) and the pairing flow changes (#6), so those two meet there.

Order that matters: batch 0 before everything that uses it. #1 and #2 are the largest single effect and mostly deletions. #13 after #1 to #4 and #10. #16 last.

Smallest change set with the biggest effect: #1, #2, #3 and #4 together (about 80 percent of the scroll and first-run problem), then #5 and #6.

### Tests and stories to update

- There are no unit tests. Gates are `pnpm --dir webapp typecheck` and `pnpm --dir webapp build-storybook`, run once after merge since the work crosses layers. No Rust checks unless an API field is added.
- `ConfirmAction.stories.tsx` play function ("Reset token" then "Yes, reset"): check whether it uses its own labels, and rewrite it if the interaction changes.
- Stories that restate current copy and need updates: `SettingsPage`, `PeerSettings`, `TokenSettings`, `LanSettings`, `SearchProgress`, `StageStep`, `DiagnosisSummary`, `DungeonPicker`, `DungeonOption`, `CharacterPanel`, `FinderPage`, `App`, and `Templates.stories.tsx` (slot labels "LAN mode" and "peers").
- New stories required by CLAUDE.md: Advanced collapsed, open and read-only on a phone; combine mode on and off; first-run steps; 1 versus 30 dungeons; 40 versus 340-character headline in the diagnosis; each party state at 5 slots; NotPaired steps.
- First new play functions: InfoTip open and close, Combine toggle.
- ADR 0026 note on the hedge. The ADR index may need a line.
- Remove the stories of any component that is deleted (`StageStep` if the list goes).

### Three strongest objections and risks

1. The honest hedging may stop being read. The diagnosis copy was written so a player does not take "I see no tank" for "no tank exists". Moving it behind a tap could undo that. The same applies to the plain HTTP risk. Mitigation: keep "Seen from here", `~`, and the one-sentence HTTP warning visible, and only the long reasons behind taps.
2. Click-to-search with one tap can start a search by accident and leaves multi-select second-class. Widening stage 4 and `search.dungeons` both assume several dungeons. Decide D1 with real usage in mind. Nobody measured how often multi-select is used.
3. The numbers are estimates, and several nice ideas need data the app does not have. No reviewer rendered a screen, so scroll heights are unverified, and "addon installed", phone counts and auto-off need server work that this audit excludes. Scope can grow without a stopping point if those creep in. Measure the screens at 1280x720 and 390x844 before and after batch 1.

Other risks: the audit's rename table changes wording that ADRs and the design doc quote; the `remote` flag is guessed from the hostname, so lock icons can briefly be wrong until a 403 arrives; shortening copy does not by itself reduce scrolling (about 3 of 12 lines saved per card), the layout changes do.

### Top 5 ranked changes

1. Click-to-search dungeon list, with the Start button and picker sentences removed (#1).
2. Search card: delete the stage list, Network field and timing sentences, show one bar and a "Searching wider" chip (#2).
3. State-driven first run in place of the addon paragraph, with no dead dungeon list (#3).
4. One-line character summary in place of nine fields (#4).
5. Settings: merge phone access, token and QR into one "Phone link" card with a one-sentence warning, and move Peers and history behind Advanced (#5, #6, #11).
