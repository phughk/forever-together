# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Always commit, preferably use multiple agents where possible.

Tauri 2 app: Rust backend (`src-tauri`), React/TS frontend (`src`), Storybook, a WoW addon (`addon/`). Tooling is pinned in `mise.toml`; see `README.md` for tasks and layout. The repo started from `template-rust-app` at commit `9048274` (2026-11-21).

## Working rules

- **Clean up agent worktrees.** When fanning work out to agents with worktree isolation, remove each worktree once its branch is merged and verified: `git worktree remove --force .claude/worktrees/<name>`, `git branch -D <branch>`, then `git worktree prune`. Never remove a worktree that is locked (a live agent) or has unmerged commits or uncommitted changes.
- **Don't set `CARGO_TARGET_DIR`.** mbx (mr-boxington) owns target storage and the shared build cache, so a manual override would fight it. If a worktree still grows its own `src-tauri/target` (about 7 GB each in practice), plain `cargo` is bypassing mbx: run `mbx doctor`, and `mbx setup` if it warns that the Cargo shim isn't installed. Delete stray `target/` dirs.
- **Only test what you changed.** Run targeted tests and clippy/typecheck for the touched side only; run everything for cross-cutting changes or when asked.

## Design system

The frontend follows atomic design. Every level must be covered in Storybook.

### Levels

- **Atoms**: the foundational elements that can't be broken down further without ceasing to be functional: form labels, inputs, buttons, and other basic HTML elements. They show all base styles at a glance and serve as the reference for the whole system. On their own they are abstract; they only come to life when composed.
- **Molecules**: relatively simple groups of atoms functioning together as a unit, with a single purpose. Example: a label + search input + button form a search-form molecule, where the label defines the input and the button submits. Molecules should be simple, portable and reusable anywhere that functionality is needed.
- **Organisms**: relatively complex components composed of molecules, atoms, and/or other organisms, forming a distinct section of an interface. They can mix similar or dissimilar parts. Example: a header organism containing a logo, primary navigation, and the search-form molecule.
- **Templates**: page-level layouts that place components and articulate the underlying *content structure*, not final content. Show how components work together in a layout, and document content constraints such as image sizes and character-length limits for headings and text passages, since real content is dynamic.
- **Pages**: specific instances of templates filled with real, representative text, images, and media. This is the most concrete level: what users see and what stakeholders sign off on.

### Rules

- Build upward: compose molecules from atoms, organisms from molecules/atoms, templates from organisms, pages from templates. Don't skip levels, and don't duplicate lower-level styling in higher levels.
- Put each component at the lowest level that fits: if it can't be broken down further it's an atom; if it's a small single-purpose group of atoms it's a molecule; and so on.
- Templates take their content as props/slots and stay content-agnostic; only pages supply real content.
- Pages are the test of the design system. If a page looks or behaves wrong with real content, loop back and fix the molecules, organisms, or templates rather than patching the page.
- Page stories must cover template variations, not just the happy path. Examples:
  - 1 item in a cart vs. 10 items.
  - Sections suppressed for first-time users (e.g. no recent activity on a dashboard).
  - A 40-character vs. a 340-character headline.
  - Admin users seeing extra buttons/options compared to regular users.
- Storybook organization should mirror the levels (Atoms / Molecules / Organisms / Templates / Pages), and each component ships with stories for its meaningful states and variants.

## Commands

- `mise run dev` desktop app; `mise run web` frontend only; `mise run storybook`; `mise run test` Rust tests; `mise run lint` clippy + rustfmt; `mise run test-addon` addon tests in the headless Lua VM; `mise run install-addon`; `mise run check` runs typecheck, lint, test and test-addon.
- Rust, from `src-tauri/`: `cargo test <filter>`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt`.
- Frontend: `pnpm typecheck`, `pnpm build`.
- Verify a task exists in `mise.toml` before relying on it. If mise shims fail in a worktree, set `MISE_TRUSTED_CONFIG_PATHS=<repo root>`; if `cd` into `src-tauri` is refused, use `cargo ... --manifest-path src-tauri/Cargo.toml`.
- The Rust suite includes multi-instance integration tests (`src-tauri/src/integration_tests.rs`, about 16 s each, they run a clock faster than real time or at real speed) and a loopback iroh test (no internet needed). `cargo test --release --lib -- --ignored --nocapture engine::sim::tests::report` runs the seeded simulation.

## This project: forever-together (short code `wowft`)

A peer-to-peer random dungeon finder for World of Warcraft Forever. Forever has a classic-style manual group finder and no random dungeon finder, so players find groups slowly by hand. This project is a faster, more automatic finder with no hosted matchmaking service. The first draft used the working name `p2prdf`. On 2026-11-21 the author renamed it `forever-together` (short code `wowft`): addon `ForeverTogether`, topic prefix `wowft/1/`, peer exchange ALPN `wowft/peers/1`, slash command `/wowft`, env var `WOWFT_WEB_DIR`, code prefix `WFT1:`. Older ADRs and the brief still say `p2prdf` as history.

### Current state

The code exists and passes its tests: the addon (`addon/ForeverTogether/`, tested in the headless Lua VM), the Rust app (`code`, `engine`, `net`, `store`, `server`, `app`, `runtime`) and the web app. The app runs on the real iroh network (`net::live::LiveNet` over `IrohNet`: bootstrap by remembered peers, then peer exchange, then trusted peers; status is published to the UI) unless `WOWFT_NET=memory` selects the in-memory network. The host loop sends heartbeats during a committed party and drops a member that stays silent for 4 s (local 60 s ban, never gossiped).

What is **not** verified (details and the full table in `docs/spike-s0.md`; treat all of it as unconfirmed):

- **Spike S0 is still unverified against a live client and a live network.** Realm and region calls, `UnitName` returning first name and surname, `GetAverageItemLevel`, a phone scanning the QR, `varaq` key-file permissions, and iroh address lookup through n0's hosted DNS/relays have not been run live. Only loopback iroh and the Lua VM were exercised.
- **The simulation result is bad.** `docs/spike-s0.md` reports that neither converging announcements nor the proposer model reaches the 70% commit floor in the model (converging: 56% at perfect delivery and about 2% at 0.95; proposer: 9 to 25% in 3 to 4 cycles). The plan says to reopen the design or revisit the model assumptions; that has not been decided. The app ships the converging-announcements baseline.
- No real window, phone, or live game has been tried. The integration tests drive the HTTP API of several in-process instances on an in-memory network.
- `net::trusted::BUILTIN` is empty until the author supplies a key. A re-audit of the revised design has not been run.

### Where the truth lives

- `docs/designs/forever-together.md`: the design (overview, CUJs, architecture, risks, alternatives). `docs/designs/forever-together-implementation-plan.md`: build order, spike S0, defaults, API table, file list, test plan. `docs/rdf-code-spec.md`: the code format and wire messages. The first draft is archived at `docs/design-process/archive/p2prdf-draft-2026-11-20.md`.
- `adr/` (0001-0022, indexed in `adr/README.md`): one decision record per file. **ADRs win over the draft and over `docs/design-process/concepts_brief.md`**, the older brainstorm. `docs/design-process/reconciliation_report.md` lists where the brief is out of date.
- `docs/design-process/reviews/`: the review reports. `consolidated_review_report.md` is the summary, `author_decisions.md` records the author's answers to the open questions D1-D9 and the new timing rules, and `goldfish_review_report.md` is the annotated fix list (applied).
- When a decision changes, add an ADR or revise the existing one and say what it supersedes.

### Architecture

Three parts in one repo (ADR 0020):

1. **Standalone WoW addon** `addon/ForeverTogether/` (Lua, own TOC and window, no saved variables). It only shows the character as a copyable text code (base45, versioned binary record) and the same data as a QR. No networking, pairing or tokens (0002, 0018).
2. **Desktop app**: Tauri 2, one Rust crate `src-tauri` (0013). Modules: `code` (decoder), `engine` (pure state machine, no I/O, not named `core`), `net` (`iroh_net` real transport, `live` wrapper that runs bootstrap and publishes status, `bootstrap`, `known_peers`, `exchange`, `trusted`, `gossip` signed envelope, `fake` in-memory hub for tests), `store` (`varaq` store and a memory store), `server` (axum API, auth, pairing, static files), `app` (`LiveCore`: owns the engine, network and character on its own thread; also `memnet`), `runtime` (start-up for window, headless and print-key modes; `build_network` is the one place that picks the network), plus `cli`, `config`, `dungeons`, `identity`. Networking is iroh + iroh-gossip with one gossip swarm per topic. Flags: `--headless`, `--print-public-key`, `--data-dir`. Environment: `WOWFT_WEB_DIR` (web file override), `WOWFT_NET=memory` (in-memory network).
3. **One React/TypeScript web app** (the template's `src/`), embedded in the binary and served over HTTP by the desktop app to its own window and to phones, over one REST + event-stream API. Phones are approved by a per-install token in a pairing QR. An environment variable can point at a folder that overrides embedded files (0012).

The headless WoW Lua test VM lives in the `wow-forever-addons` repo (`/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons`) and is a git submodule at `vendor/wow-forever-addons` (0021). The code spec and test vectors live once under `docs/` (`rdf-code-spec.md`, `rdf-code-vectors.json`) and are read by both the Lua and Rust tests.

**Running two instances on one machine:** give each its own `--data-dir`; ports are random. **Adding a trusted peer:** run an always-on copy with `--headless`, print its key with `--print-public-key`, then add it to `net::trusted::BUILTIN` (ships with the app) or to `trusted_peers` in settings (loopback only).

### Domain rules that shape the design

- **Topic** = `region + realm + faction + dungeon` (hashed with a protocol prefix). Level is not in the topic: it travels in ads as a matching condition (0005). v1 has one shard per topic, and the shard number stays in the hash, always 0, so sharding can be added later without renaming topics. A realm is a ruleset (PvP, PvE, RP-PvP, RP-PvE, Hardcore) on a shared mega server. The addon supplies realm and region as opaque values and the app never hard-codes a realm list (0004). Parties are same-faction only (0007).
- **Names are two fields, first name and surname.** Never read a realm from the second return of `UnitFullName` or `UnitName`; on Forever that is a surname (0022). Peers are identified by their iroh key, never by character name (0016).
- **Everything the addon reports is a self-reported claim** and can be forged (0003). Gossip messages are signed by the author's iroh key (0019, accepted, with the caveat that it is replaced if iroh-gossip already identifies authors).
- **Bootstrap**: remembered peers, then peer exchange, then built-in trusted peers only as a last resort (0009). Principle: avoid supernodes. Trusted peers are ordinary peers (the same app, headless) that learn through peer exchange like everyone else.
- **Agreeing on a party**: the baseline is epoch-based converging announcements with a human Confirm (0010, 0024). A proposer model with response windows is to be simulated against it in the spike, and the better commit rate wins. Either way: a formed party broadcasts that its members are taken, an unclicked Confirm sends a Decline with no penalty, and a peer that does not answer a heartbeat is banned locally for about 1 minute, never gossiped. Decision windows are 3-5 s. Search-level windows are about 1 minute.
- Dungeons come from a hard-coded table, recommended by level and zone name, with no location-based refusal (0006).

### Project conventions

- Diagrams in docs and ADRs are **Mermaid only**, no ASCII art (0015).
- Tests are functional. The author has allowed a small seeded simulation of the agreement rule on the pure `engine` as an algorithm check, not a load test. QR appearance and a real phone scan are manual checks (0014, amended by decision D1).
- Unverified assumptions are in the ADRs (client calls for realm and region, `varaq` maturity, iroh-gossip author identification). They are settled in a spike before the code format freezes. Do not treat them as confirmed.
