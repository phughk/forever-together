# forever-together

A peer-to-peer random dungeon finder for World of Warcraft Forever (short code `wowft`). Forever has no random dungeon finder, so this app finds a group of five for you without a hosted matchmaking service.

How it works, in short: a small in-game addon shows your character as a text code (and a QR). You paste the code into the desktop app (or open the app's page on your phone), pick dungeons, and the app finds other players over a peer-to-peer network. When five compatible players are found, everyone confirms, and then the names are shown so you can invite each other in game.

Status: the code is built and tested without a live game. The unverified assumptions (client calls, the hosted iroh components, the simulation result) are listed in [`docs/spike-s0.md`](docs/spike-s0.md). Read that file before trusting the matching rate.

## Getting started

```sh
mise install        # node, pnpm, rust, mr-boxington
mise run setup      # pnpm install (in webapp/)
mise run dev        # desktop app (Tauri + Vite)
```

## Tasks

| Task | What it does |
|------|--------------|
| `mise run setup` | `pnpm --dir webapp install` |
| `mise run dev` | the desktop app (Tauri + Vite) |
| `mise run web` | the web app alone in the browser (no backend, so no API) |
| `mise run storybook` | the component workshop on port 6006 |
| `mise run typecheck` | typecheck the web app |
| `mise run test` | Rust unit and integration tests (about 40 s, the multi-instance tests are slow on purpose) |
| `mise run lint` | clippy with warnings denied, and a rustfmt check |
| `mise run test-addon` | build the headless WoW Lua VM and run the addon tests in it. The submodule URL is a local path, so git may refuse to clone it ("transport 'file' not allowed"); run `git -c protocol.file.allow=always submodule update --init vendor/wow-forever-addons` once in your own shell |
| `mise run install-addon` | copy the addon into `$WOW_ADDONS_DIR` (must end in `AddOns`) |
| `mise run update-icons` | regenerate the app icons (`app/icons/`) and the addon icon (`addon/ForeverTogether/icon.tga`) from `assets/icon_1024.png` |
| `mise run check` | typecheck, lint, test and test-addon: everything CI should run |
| `mise run build` | bundle the app for this platform |

## Getting your character code and QR in game

1. Set `WOW_ADDONS_DIR` (see `mise.local.toml`; it must end in `AddOns`) and run `mise run install-addon`.
2. In game, type `/reload` so the client loads the addon (a full client restart also works).
3. Type `/wowft`. A window opens showing the code as text, the QR, the fields it understood, how old the code is and a Refresh button. Escape closes it. `/wowft code` prints the code in chat instead, and `/wowft help` lists the commands.
4. Click the text box (it selects all) and press Ctrl+C, then paste it into the app's "Character code from the addon" box.
5. The code is a snapshot. After a level up, a gear change or a zone change, press Refresh in the window and paste again.

The QR holds the same text. The app has no camera scanner, so on a phone you scan it with the camera app, copy the text it shows and paste it into the app's page. If the QR is missing, the window says why: the code was too long to draw (the text still works) or a field could not be read (no code is built).

## Layout

- `addon/ForeverTogether/`: the WoW addon (Lua). It shows the character code and has no networking. Tests are in `addon/tests/`.
- `app/`: the Rust and Tauri app (`app/src/`). Modules: `code` (decode the character code), `engine` (the pure matching state machine), `net` (iroh network, bootstrap, peer exchange, an in-memory fake), `store` (identity, settings, known peers), `server` (the local web server and API), `app` (`LiveCore`, joins engine, network and server), `runtime` (start-up for the three modes), `cli`, `config`, `dungeons`, `identity`.
- `webapp/`: the React and TypeScript web app (`webapp/src/`), shown in the app window and on phones. Atomic design, with stories next to each component.
- `docs/`: the design, the implementation plan, the code format spec (`rdf-code-spec.md`), spike results (`spike-s0.md`) and review reports. `docs/adr/`: one decision record per file. `docs/notes/`: loose notes.
- `assets/`: the icon master (`icon_1024.png`) and its project file. The Lora font and its license are in `webapp/src/assets/fonts/`. `scripts/`: Python and shell helpers (icons, code vectors, addon install).
- `vendor/wow-forever-addons`: git submodule with the headless Lua test VM.

## Running the app

```sh
forever-together                              # window mode
forever-together --headless                   # network only, no window, no web server
forever-together --print-public-key           # print this install's peer ID and exit
forever-together --data-dir /some/folder      # keep identity, settings and peers here
```

Environment variables:

- `WOWFT_WEB_DIR`: a folder whose files override the embedded web app (path-safe, `/api` is never overridden). Useful for changing the page without rebuilding.
- `WOWFT_NET=memory`: use the in-memory network that hears nobody, so the UI can be tried with no network.

Two instances on one machine: give each its own `--data-dir`. The web server and the network pick random free ports, so nothing else is needed.

Trusted peers (the last-resort way in for a new install): run an always-on copy with `--headless`, print its key with `--print-public-key`, and either add the key to the built-in list in `app/src/net/trusted.rs` or add it in the settings page of a running app (`trusted_peers`).

## Mobile

`app/src/lib.rs` is mobile-ready, but phones in this project use the web app over the local network, not a native build.
