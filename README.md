# template-rust-app

Template for a Tauri 2 desktop/mobile app: mise, Rust (with mr-boxington build caching), pnpm, TypeScript (strict), React, Storybook.

## Getting started

```sh
mise install        # node, pnpm, rust, mr-boxington
mise run setup      # pnpm install
mise run dev        # desktop app (Tauri + Vite)
mise run web        # frontend only, in the browser
mise run storybook  # component workshop on :6006
mise run check      # typecheck + clippy + rustfmt
mise run build      # bundle for the host platform
```

## Layout

- `src/` – React + TS frontend (`components/` hold `*.stories.tsx` next to the component)
- `src-tauri/` – Rust backend; `lib.rs` has the shared entry point used by desktop and mobile
- `.storybook/` – Storybook config

## Mobile

`src-tauri/src/lib.rs` is already mobile-ready. In a new project run `pnpm tauri android init` / `pnpm tauri ios init`, then `pnpm tauri android dev` / `ios dev`.

## After copying the template

Change the name/identifier in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` (`identifier`), and replace `app-icon.png` then run `pnpm tauri icon app-icon.png`.
