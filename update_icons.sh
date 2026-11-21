#!/usr/bin/env bash
# Regenerate every icon from the master image assets_vc/icon_1024.png:
#   - the desktop app icons (src-tauri/icons/*, all sizes and formats) via `tauri icon`
#   - the addon's AddOns-list icon, addon/ForeverTogether/icon.tga (64x64 32-bit TGA) via uv + Pillow
# Usage: ./update_icons.sh [path/to/master.png]
set -euo pipefail
cd "$(dirname "$0")"

SRC="${1:-assets_vc/icon_1024.png}"
[ -f "$SRC" ] || { echo "update_icons: $SRC not found" >&2; exit 1; }

read -r W H < <(uv run --quiet --with pillow python -c \
  'import sys; from PIL import Image; print(*Image.open(sys.argv[1]).size)' "$SRC")
if [ "$W" != "$H" ] || [ "$W" -lt 1024 ]; then
  echo "update_icons: $SRC is ${W}x${H}; it must be square and at least 1024x1024" >&2
  exit 1
fi

pnpm tauri icon "$SRC"
uv run --quiet scripts/addon_icon.py "$SRC" addon/ForeverTogether/icon.tga
echo "done"
