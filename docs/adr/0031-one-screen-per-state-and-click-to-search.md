# 0031. The finder shows one screen per state, and a tap on a dungeon starts the search

* **Status:** Accepted (decided and implemented 2026-11-21). Amends [0011](0011-search-widening.md) (what the player sees of widening) and [0026](0026-search-health-from-own-observations.md) (presentation only).
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski (decisions D1, D2, D7 of the UI audit)

## Context

The finder was a long page: character panel, dungeon picker, a Start button, a stage list, timing sentences and a party panel, all at once. It scrolled on small windows and phones.

## Decision

* **One screen per state:** first run, idle, searching, party found, party final, search ended, each with a main slot and a compact character strip. The page does not scroll at 1280x720 or 390x844 in any measured variation.
* **Tap to search (D1).** Tapping a dungeon starts a search for it; there is no Start button. Choosing several dungeons is behind a "Combine dungeons" toggle. The choice is frozen while searching. Stop is one tap.
* **Searching wider is one chip (D7).** The stage list is gone. A single "Searching wider" chip appears when the search loosens, with the rules behind an info tap and no countdown to the next step.
* **Search screens are scaled up:** a large timer, a thick bar, role tiles, larger actions when the search ends. Search is two cards: progress, and what is seen from here.
* **First run (D2).** The first run says "Paste your code" and never claims to know whether the addon is installed. `ADDON_URL` (`webapp/src/lib/links.ts`) holds the addon page link and is empty until the author supplies it; while empty, the step shows as plain text. The installer ([0028](0028-the-desktop-app-installs-the-bundled-addon.md)) does not change the first-run step in its first slice.
* **Change character is always available** in the character strip, and the strip shows one line (name, level, role) with an Edit disclosure.
* **Short screens are offset from the top** so the strip and card share a balanced top margin; wide screens use a two-pane grid for focus screens.

## Consequences

* An accidental tap starts a search. The cost is one Stop tap. Nobody measured how often multi-select is used.
* Widening stage 4 and `search.dungeons` still assume several dungeons.
