# forever-together

A peer-to-peer random dungeon finder for World of Warcraft Forever (short code `wowft`). Forever has no random dungeon finder, so this app finds a group of five for you without a hosted matchmaking service.

How it works, in short: a small in-game addon shows your character as a text code (and a QR). You install the addon from the desktop app's Settings (or by script, for developers), paste the code into the desktop app (or open the app's page on your phone), pick dungeons, and the app finds other players over a peer-to-peer network. When five compatible players are found, everyone confirms, and then the names are shown so you can invite each other in game.

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
| `mise run install-addon` | copy the addon into `$WOW_ADDONS_DIR` (must end in `AddOns`). For developers; players install the addon from the desktop app's Settings, in the Addon card |
| `mise run update-icons` | regenerate the app icons (`app/icons/`) and the addon icon (`addon/ForeverTogether/icon.tga`) from `assets/icon_1024.png` |
| `mise run check` | typecheck, lint, test and test-addon: everything CI should run |
| `mise run build` | bundle the app for this platform |

## Getting your character code and QR in game

1. Install the addon. In the desktop app, open Settings and use the Addon card (it finds the game's AddOns folder or takes a pasted path; the first install asks you to confirm the folder; it replaces the whole `ForeverTogether` folder; unverified on a real install, see [ADR 0028](docs/adr/0028-the-desktop-app-installs-the-bundled-addon.md)). Developers can instead set `WOW_ADDONS_DIR` (see `mise.local.toml`; it must end in `AddOns`) and run `mise run install-addon`.
2. In game, type `/reload` so the client loads the addon (a full client restart also works).
3. Type `/wowft`. A window opens showing the code as text, the QR, the fields it understood, how old the code is and a Refresh button. Escape closes it. `/wowft code` prints the code in chat instead, and `/wowft help` lists the commands.
4. Click the text box (it selects all) and press Ctrl+C, then paste it into the app's "Character code from the addon" box.
5. The code is a snapshot. After a level up, a gear change or a zone change, press Refresh in the window and paste again.

The QR holds the same text. The app has no camera scanner, so on a phone you scan it with the camera app, copy the text it shows and paste it into the app's page. If the QR is missing, the window says why: the code was too long to draw (the text still works) or a field could not be read (no code is built).

## Layout

- `addon/ForeverTogether/`: the WoW addon (Lua). It shows the character code and has no networking. Tests are in `addon/tests/`.
- `app/`: the Rust and Tauri app (`app/src/`). Modules: `code` (decode the character code), `engine` (the pure matching state machine), `net` (iroh network, bootstrap, peer exchange, an in-memory fake), `store` (identity, settings, known peers), `server` (the local web server and API), `app` (`LiveCore`, joins engine, network and server), `runtime` (start-up for the three modes), `cli`, `config`, `dungeons`, `identity`.
- `webapp/`: the React and TypeScript web app (`webapp/src/`), shown in the app window and on phones. Atomic design, with stories next to each component.
- `docs/`: the code format spec (`rdf-code-spec.md`), spike results (`spike-s0.md`). `docs/adr/`: one decision record per file, including the overview and plan (0034, 0035). `tickets/`: open work, one file per ticket. `docs/design-process/archive/`: superseded briefs, kept as history.
- `assets/`: the icon master (`icon_1024.png`) and its project file. The Overpass (UI) and Lora (game entity names) fonts and their licenses are in `webapp/src/assets/fonts/`. `scripts/`: Python and shell helpers (icons, code vectors, addon install).
- `vendor/wow-forever-addons`: git submodule with the headless Lua test VM.

## Running the app

```sh
forever-together                              # window mode
forever-together --headless                   # network only, no window, no web server
RUST_LOG=app_lib=debug,iroh=info forever-together --headless   # logs to stderr; default is warnings only
forever-together key show                     # print this install's peer ID and public key
forever-together key export ./node.key        # back up the key (new file, owner-only)
forever-together key import ./node.key        # restore or deploy a key
forever-together key reset                    # replace the key with a new one
forever-together --data-dir /some/folder      # keep identity, settings and peers here
```

Environment variables:

- `WOWFT_WEB_DIR`: a folder whose files override the embedded web app (path-safe, `/api` is never overridden). Useful for changing the page without rebuilding.
- `WOWFT_NET=memory`: use the in-memory network that hears nobody, so the UI can be tried with no network.

Two instances on one machine: give each its own `--data-dir`. The web server and the network pick random free ports, so nothing else is needed.

Trusted peers (the last-resort way in for a new install): run an always-on copy with `--headless`, print its key with `forever-together key show` (stop the node first, see below), and either add the key to the built-in list in `app/src/net/trusted.rs` or add it in the settings page of a running app (`trusted_peers`).

### Keys

Every install has one permanent ed25519 key. Its public half is the peer ID. Headless is the same program, so a node creates a key at first start and takes the same `key` commands (ADR 0039).

- `key show` prints the peer ID (hex, the form used in `BUILTIN` and `trusted_peers`) and the OpenSSH public key line. `key show` and `key export` create a key if there is none.
- `key export <file>` writes the key as an unencrypted OpenSSH ed25519 key file: a new file, owner-only, never overwriting. `key export --stdout` prints it instead (a shell redirect creates the file with your umask, so `chmod 600` it). The file works with `ssh-keygen -y -f <file>` and `age -d -i <file>`.
- `key import <file>` replaces the stored key. It refuses if a key is already stored, unless you pass `--force`. A key created by an earlier `key show` counts, so import into a fresh data directory before anything else, or use `--force`.
- `key reset` makes a new key after a confirmation (`--yes` skips it). Other nodes see a new peer ID. A trusted peer's entry in `BUILTIN` then needs a new release.
- The commands take a lock on the data directory, so they refuse while the app or a headless node is running on it. Stop the node first. Back up before you reset or force an import.
- Not supported on Windows release builds yet (no console, ticket 0046).

**Trusted-node key backup (the author's process).** Stop the always-on node, run `key export` to a file on an offline disk, and start it again. Keep every historical `BUILTIN[0]` secret: players on older releases keep encrypting scan exports to the key compiled into their build. The private key that decrypts exports is the same file.

A cloud node gets its key by `forever-together key import ./node.key --data-dir <dir>` before its first start. If a start has already created a random key, add `--force`. After a deploy, stop the node and run `key show` to check its peer ID is the one in `BUILTIN` (a headless node logs nothing at start unless `RUST_LOG` is set).

## Mobile

`app/src/lib.rs` is mobile-ready, but phones in this project use the web app over the local network, not a native build.
