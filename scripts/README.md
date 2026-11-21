# scripts

Python tooling, run with `uv` from the repo root (see `mise run update-icons`).

- `update_icons.py [master.png]`: regenerates every icon from `assets_vc/icon_1024.png`: the desktop app icons in `src-tauri/icons/` (through `pnpm tauri icon`) and the addon's 64x64 32-bit TGA, `addon/ForeverTogether/icon.tga`.
  Run it with `uv run --project scripts scripts/update_icons.py`.
- `gen-code-vectors.py`: regenerates the QR matrices in `docs/rdf-code-vectors.json`. Needs the extra group: `uv run --project scripts --group vectors scripts/gen-code-vectors.py`.
- `install-addon.sh`: copies the addon into `$WOW_ADDONS_DIR`.
