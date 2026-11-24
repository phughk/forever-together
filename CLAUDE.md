# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Always commit, preferably use multiple agents where possible.

Tauri 2 app: Rust backend (`app/`), React/TS frontend (`webapp/`), Storybook, a WoW addon (`addon/`). Tooling is pinned in `mise.toml`; see `README.md` for tasks and layout. The repo started from `template-rust-app` at commit `9048274` (2026-11-21).

## Working rules

- **Clean up agent worktrees.** When fanning work out to agents with worktree isolation, remove each worktree once its branch is merged and verified: `git worktree remove --force .claude/worktrees/<name>`, `git branch -D <branch>`, then `git worktree prune`. Never remove a worktree that is locked (a live agent) or has unmerged commits or uncommitted changes.
- **Don't set `CARGO_TARGET_DIR`.** mbx (mr-boxington) owns target storage and the shared build cache, so a manual override would fight it. If a worktree still grows its own `app/target` (about 7 GB each in practice), plain `cargo` is bypassing mbx: run `mbx doctor`, and `mbx setup` if it warns that the Cargo shim isn't installed. Delete stray `target/` dirs.
- **Write ADRs as things are decided, and tickets for what is left.** Every decision, and every piece of work finished that embodies one, gets an ADR in `docs/adr/` (next number, indexed in `docs/adr/README.md`) in the same change that makes it, not afterwards. Open work, ideas and unverified items go in `tickets/` (`NNNN-slug.md`, indexed in `tickets/README.md`). Do not leave decisions in briefs, notes or chat: briefs and loose design docs are archived under `docs/design-process/archive/` once their decisions are in ADRs.
- **EGM runs follow ADR 0036.** Start from a ticket. Every EGM prompt names its output paths inside `docs/egm/<ticket number>-<slug>/` (`brief.md`, `reviews/r<round>-<reviewer>.md`, `resolutions.md`); an agent never writes elsewhere in `docs/` or edits an ADR. Check and move anything misplaced before committing. Resolutions become ADRs in the same change, leftovers become tickets, and each step is its own commit: `egm(NNNN): brief | review round N | resolutions`, `adr(MMMM): title`, `ticket(NNNN): status`. Do not put new work in `docs/design-process/archive/`.
- **Never commit real character data.** The author's live codes, character names and zones are personal. Test vectors, stories and docs use invented characters (for example "Testa Fakename", zone "Testville"). Record facts from live captures, not the captures themselves.
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
- **Show, don't tell.** A component must be intuitive from its shape, state and labels. Do not use paragraphs of text to explain what a component is or how to use it. Prefer, in this order: a clear label, an icon or visual state (a switch that is on or off, a selected row, a progress bar, role icons with counts), a short value next to its label ("Players seen: 8"), and one short sentence only where the screen cannot show the fact. A card with more than about two sentences of explanation is a signal to redesign the component, not to trim the words. Warnings and notices state the consequence in one sentence and nothing else. Text that explains a setting belongs in a tooltip or a collapsed "Details", never in the default view. Respect the viewport: a component that needs scrolling to be understood is too big.
- **Fonts.** Overpass (`--font`) is the UI font wherever legibility matters: numbers, labels, buttons, badges, notices, sentences. Lora (`--font-entity`) is only for game entities (character, class, race, realm, zone and dungeon names), always through the `Entity` atom (`.ft-entity`), never by setting the font on a component. Entity text takes the colour of its context; it is not dimmed.
- Storybook organization should mirror the levels (Atoms / Molecules / Organisms / Templates / Pages), and each component ships with stories for its meaningful states and variants.

## Commands

- `mise run build-headless` builds with no window and no Tauri, GTK or WebKit (`--no-default-features`, ADR 0042); `mise run dev` desktop app; `mise run web` frontend only; `mise run storybook`; `mise run test` Rust tests; `mise run lint` frontend typecheck + clippy + rustfmt; `mise run clean` removes build outputs; `mise run test-addon` addon tests in the headless Lua VM; `mise run install-addon`; `mise run check` runs lint (which includes typecheck), test and test-addon.
- Rust, from `app/`: `cargo test --lib <filter>` (for example `cargo test --lib engine::` or `cargo test --lib code::`; cargo takes one filter), `cargo clippy --all-targets -- -D warnings`, `cargo fmt`.
- Frontend, from the repo root: `pnpm --dir webapp typecheck`, `pnpm --dir webapp build`, `pnpm --dir webapp build-storybook`.
- Icons: `mise run update-icons` regenerates `app/icons/` and `addon/ForeverTogether/icon.tga` from `assets/icon_1024.png`. The Python tools are a uv project in `scripts/`; run them with `uv run --project scripts scripts/<name>.py` (`gen-code-vectors.py` also needs `--group vectors`).
- `mise run dev` and `mise run build` run the Tauri CLI from `app/` (the CLI only finds `tauri.conf.json` from its working directory); `webapp/node_modules/.bin` is put on PATH by `mise.toml`.
- The submodule URL is a local path, so `mise run test-addon` may fail to clone with "transport 'file' not allowed". Run `git -c protocol.file.allow=always submodule update --init vendor/wow-forever-addons` once in your own shell.
- Verify a task exists in `mise.toml` before relying on it. If mise shims fail in a worktree, set `MISE_TRUSTED_CONFIG_PATHS=<repo root>`; if `cd` into `app/` is refused, use `cargo ... --manifest-path app/Cargo.toml`.
- The Rust suite includes multi-instance integration tests (`app/src/integration_tests.rs`, about 16 s each, they run a clock faster than real time or at real speed) and a loopback iroh test (no internet needed). The seeded simulation has fast regression tests in `engine::sim::tests`; the table of numbers in `docs/spike-s0.md` came from a report test that has since been removed (see that file to restore it).

## This project: forever-together (short code `wowft`)

A peer-to-peer random dungeon finder for World of Warcraft Forever. Forever has a classic-style manual group finder and no random dungeon finder, so players find groups slowly by hand. This project is a faster, more automatic finder with no hosted matchmaking service. The first draft used the working name `p2prdf`. On 2026-11-21 the author renamed it `forever-together` (short code `wowft`): addon `ForeverTogether`, topic prefix `wowft/1/`, peer exchange ALPN `wowft/peers/1`, slash command `/wowft`, env var `WOWFT_WEB_DIR`, code prefix `WFT1:`. Older ADRs and the brief still say `p2prdf` as history.

### Current state

The code exists and passes its tests: the addon (`addon/ForeverTogether/`, tested in the headless Lua VM), the Rust app (`code`, `engine`, `net`, `store`, `server`, `app`, `runtime`, `addon_install`) and the web app. The app runs on the real iroh network (`net::live::LiveNet` over `IrohNet`: bootstrap by remembered peers, then peer exchange, then trusted peers; status is published to the UI) unless `WOWFT_NET=memory` selects the in-memory network. The host loop sends heartbeats during a committed party and drops a member that stays silent for 4 s (local 60 s ban, never gossiped).

What is **not** verified (details and the full table in `docs/spike-s0.md`; treat all of it as unconfirmed):

- **Spike S0 is partly verified.** A live capture on 2026-11-21 confirmed region 3 = EU, realm values 14 = PvP, 15 = PvE, 16 = RP-PvE, 18 = RP-PvP (the Hardcore value is not known, the ruleset is not released), `GetAverageItemLevel` (valid with gear, flag clear without), and a first name plus surname from `UnitName`. Still unverified against a live client and network: a phone scanning the QR, `varaq` key-file permissions, and iroh address lookup through n0's hosted DNS/relays have not been run live. Only loopback iroh and the Lua VM were exercised.
- **The simulation reopened the agreement mechanism, and a new one passes in the model.** `docs/spike-s0.md` reports that converging announcements (about 2% at delivery 0.95) and the proposer model (9 to 25%) did not reach the 70% floor. Leader invitations ([ADR 0025](docs/adr/0025-leader-invitations.md), implemented in `engine`) reach it within 30 s (mean 78.4%, worst seed 74.6%, 6 rounds of 5 s; 73.0% mean and 69.7% worst at 3 rounds). That is simulation only. The role mix caps any mechanism, so the floor as worded cannot be met when tanks are scarce (the S0 results explain). The engine does not retransmit lost direct messages.
- No real window, phone, or live game has been tried. The integration tests drive the HTTP API of several in-process instances on an in-memory network.
- **The addon installer (ADR 0028) is unverified on a real install.** Not confirmed: the Forever folder layout beyond the author's macOS machine, the Windows and Linux default paths, game behaviour during an in-place swap, Windows file locking and admin rights, and the client ignoring dot-prefixed folders (`.ForeverTogether.new`).
- `net::trusted::BUILTIN` holds two keys (trusted1, trusted2; backed up by the author, not yet running anywhere), so the last-resort fallback has not been exercised on a real node. A re-audit of the revised design has not been run.

### Where the truth lives

- `docs/adr/` (0001-0042, indexed in `docs/adr/README.md`): one decision record per file, and the only place decisions live. ADR 0034 is the system overview, goals, journeys, non-functional requirements and risks; ADR 0035 is the protocol constants, widening schedule, engine contract and API table. `docs/rdf-code-spec.md`: the code format and wire messages. `docs/spike-s0.md`: spike results.
- `tickets/`: open work, one file per ticket (`tickets/README.md` is the index).
- `docs/design-process/archive/`: history only (the first draft, the former design doc and implementation plan, the concept briefs, and the review reports under `reviews/`, including `author_decisions.md` with the author's answers D1-D9). **ADRs win over everything there.** Do not edit the archive to record new decisions.
- When a decision changes, add an ADR or revise the existing one and say what it supersedes.

### Architecture

Three parts in one repo (ADR 0020):

1. **Standalone WoW addon** `addon/ForeverTogether/` (Lua, own TOC and window, no saved variables). It only shows the character as a copyable text code (base45, versioned binary record) and the same data as a QR. No networking, pairing or tokens (0002, 0018). The desktop app embeds a copy and can install it into the game folder (0028).
2. **Desktop app**: Tauri 2, one Rust crate `app/` (0013). Modules: `code` (decoder), `engine` (pure state machine, no I/O, not named `core`), `net` (`iroh_net` real transport, `live` wrapper that runs bootstrap and publishes status, `bootstrap`, `known_peers`, `exchange`, `trusted` (keys and the project's relay URLs), `relay_server` (the rate-limited hole-punching relay a headless node runs with `--relay`, ADR 0041), `gossip` signed envelope, `fake` in-memory hub for tests). Every node joins a membership topic (ADR 0040). The real endpoint uses n0's relays plus the project's, and publishes its address (with IPs) to n0's DNS and the Mainline DHT (ADR 0041), `store` (`varaq` store, a memory store, and `DataDirLock`: the app and the `key` commands hold an exclusive lock on the data directory because `varaq` does not refuse a second opener), `history` (search history entries over the store, ADR 0027), `addon_install` (the addon embedded in the binary by `build.rs`; detects the AddOns folder, reports status and installs by stage and swap, ADR 0028), `server` (axum API, auth, pairing, static files), `app` (`LiveCore`: owns the engine, network and character on its own thread; also `memnet`), `runtime` (start-up for window, headless and print-key modes; `build_network` is the one place that picks the network), plus `cli`, `config`, `dungeons`, `identity`. Networking is iroh + iroh-gossip with one gossip swarm per topic. Flags: `--headless`, `--relay <addr:port>` (headless only), `--data-dir`; subcommands: `key show|export|import|reset` (ADR 0039, `keys` module). Environment: `WOWFT_WEB_DIR` (web file override), `WOWFT_NET=memory` (in-memory network).
3. **One React/TypeScript web app** (`webapp/`, bundled with the Overpass and Lora fonts), embedded in the binary and served over HTTP by the desktop app to its own window and to phones, over one REST + event-stream API. Its Settings has an Addon card that installs the bundled addon (desktop only, ADR 0028). Phones are approved by a per-install token in a pairing QR. An environment variable can point at a folder that overrides embedded files (0012).

The headless WoW Lua test VM lives in the `wow-forever-addons` repo (`/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons`) and is a git submodule at `vendor/wow-forever-addons` (0021). The code spec and test vectors live once under `docs/` (`rdf-code-spec.md`, `rdf-code-vectors.json`) and are read by both the Lua and Rust tests.

**Running two instances on one machine:** give each its own `--data-dir`; ports are random.

### Domain rules that shape the design

- **Topic** = `region + realm + faction + dungeon` (hashed with a protocol prefix). Level is not in the topic: it travels in ads as a matching condition (0005). v1 has one shard per topic, and the shard number stays in the hash, always 0, so sharding can be added later without renaming topics. A realm is a ruleset (PvP, PvE, RP-PvP, RP-PvE, Hardcore) on a shared mega server. The addon supplies realm and region as opaque values and topics hash the raw values. Display names (region 3 = EU, realm 14/15/18 = PvP/PvE/RP-PvP, anything else shown as `realm <id>`) are cosmetic and live in `addon/ForeverTogether/Names.lua` and `app/src/code.rs`; the app never hard-codes a realm list for matching (0004). Parties are same-faction only (0007).
- **Names are two fields, first name and surname.** Never read a realm from the second return of `UnitFullName` or `UnitName`; on Forever that is a surname (0022). Peers are identified by their iroh key, never by character name (0016).
- **Everything the addon reports is a self-reported claim** and can be forged (0003). Gossip messages are signed by the author's iroh key (0019, accepted, with the caveat that it is replaced if iroh-gossip already identifies authors).
- **Bootstrap**: remembered peers, then peer exchange, then built-in trusted peers only as a last resort (0009). Principle: avoid supernodes. Trusted peers are ordinary peers (the same app, headless) that learn through peer exchange like everyone else.
- **Agreeing on a party**: leader invitations (0025, replacing 0010). Once per 5 s epoch a tank invites a healer, three damage peers and spares from its own sample; an invitee accepts the lowest-ranked leader 2 s into the epoch; the leader commits when its acceptors make a party and tells the four. The sample drops ads by a hash of owner and peer, not by lowest peer ID. After a commit (0024): a formed party broadcasts that its members are taken, `Ready` carries the member's own ad, an unclicked Confirm sends a Decline with no penalty, and a peer that does not answer a heartbeat is banned locally for about 1 minute, never gossiped. Decision windows are 2-5 s. Search-level windows are about 1 minute.
- Dungeons come from a hard-coded table, recommended by level and zone name, with no location-based refusal (0006).

### Project conventions

- Diagrams in docs and ADRs are **Mermaid only**, no ASCII art (0015).
- Tests are functional. The author has allowed a small seeded simulation of the agreement rule on the pure `engine` as an algorithm check, not a load test. QR appearance and a real phone scan are manual checks (0014, amended by decision D1).
- Unverified assumptions are in the ADRs (client calls for realm and region, `varaq` maturity, iroh-gossip author identification). They are settled in a spike before the code format freezes. Do not treat them as confirmed.
