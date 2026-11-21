# 0033. The header shows only a gear and Back, and the character strip always offers Change character

* **Status:** Accepted (decided and implemented 2026-11-21). Extends [0029](0029-atomic-design-and-show-dont-tell.md) and [0031](0031-one-screen-per-state-and-click-to-search.md).
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Decision

* **Header:** a gear (Settings) and Back, nothing else. The connection chip appears only when something is wrong (not connected, the server unreachable); a healthy connection shows nothing.
* **Character strip on every finder screen** shows name, level and role in one line. A **Change character** button opens the paste form inline. It is highlighted and reads Update when the code is out of date. While searching or in a party it is disabled and says why. **Remove** is behind Details, not a top-level button.
* **Game entity names** in the strip, party list and search cards are set through the `Entity` atom ([0030](0030-dark-only-theme-and-fonts.md)), so the strip uses the entity font without any component setting it.
* **Long names ellipsize** instead of wrapping; the out-of-date notice sits inside the strip.

## Consequences

* A player can always correct a wrong or old code without leaving the screen.
* A healthy connection is never confirmed in the UI; absence of the chip is the signal.
