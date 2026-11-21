# 0030. The web app is dark-only, with palette tokens and two fonts

* **Status:** Accepted (decided and implemented 2026-11-21). Extends [0012](0012-one-web-app-for-phone-and-desktop.md).
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

The template shipped a light and dark theme with system fonts. The app is used beside a game, mostly in a dark room, and game entity names need a different look from interface text.

## Decision

* **Dark only.** One theme with deeper layered surfaces. There is no light theme and no theme switch. Storybook defaults to the app's dark background.
* **Brand palette as tokens.** Colours are CSS custom properties built from a brand palette. Text and interactive colours were checked for WCAG AA contrast against their surfaces.
* **Two fonts, both bundled** (no network fetch, so the app works offline): Overpass (`--font`, variable woff2, OFL) for every UI string where legibility matters (numbers, labels, buttons, badges, notices, sentences); Lora (`--font-entity`) only for game entities (character, class, race, realm, zone and dungeon names).
* **Entities go through the `Entity` atom** (`.ft-entity`). Nothing else sets the entity font. Entity text takes the colour of its context and is not dimmed.

## Consequences

* A component that shows a game name and does not use `Entity` is a defect.
* A light theme would be a new decision that supersedes this one.
* Font files add to the embedded bundle size (not measured).
