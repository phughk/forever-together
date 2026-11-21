# 0020. One repository (forever-together) holds a standalone addon, the app and the web app

* **Status:** Accepted (supersedes 0017; amended 2026-11-21: names)
* **Date:** 2026-11-20
* **Deciders:** Hugh Kaznowski

## Context

The design started in the `wow-forever-addons` repository because it holds a lot of WoW addon context, and the first plan added the addon as a module of Social Hub. The author says this is a separate project that must not touch any existing addon.

## Decision

Use one repository, `forever-together` (GitHub `phughk/forever-together`, at `/Users/hughkaznowski/Projects/github/forever-together`, currently only a BSD 3-Clause `LICENSE`), filled with files from the author's template. *Amended 2026-11-21:* the project is `forever-together`, short code `wowft`. The addon is `ForeverTogether`, the topic prefix `wowft/1/`, the peer exchange ALPN `wowft/peers/1`, the slash command `/wowft`, the environment variable `WOWFT_WEB_DIR` and the code prefix `WFT1:`. The earlier working name `p2prdf` remains in older ADR text and in the brief as history. The new repository was started from `template-rust-app` at commit `9048274`. It holds a standalone addon (`addon/ForeverTogether/`, its own TOC and window, no saved variables), the desktop app and the web app. The shared code specification and test vectors live once under `docs/`. The `wow-forever-addons` repository keeps all its addons unchanged. Its only change is the test VM additions described in 0021. The brief, the design draft and these records move into the new repository when the design is finished.

## Consequences

The addon cannot use Social Hub's widgets, window or commands, so it has its own small window. It uses the headless test VM from `wow-forever-addons` as a submodule (see 0021). There is one copy of the spec and vectors, so no copy check is needed.

## Alternatives considered

* The addon as a Social Hub module (0017): rejected by author, this is a separate project that touches no existing addon.
* A separate addon repository next to the app repository: not chosen by author, one repository for the whole project.
