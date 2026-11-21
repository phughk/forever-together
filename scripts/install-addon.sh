#!/usr/bin/env bash
# Copy addon/ForeverTogether into the game's AddOns directory. Refuses to run unless $WOW_ADDONS_DIR is set and its
# last path component is "AddOns", because rsync --delete empties the destination folder.
set -euo pipefail
cd "$(dirname "$0")/.."

: "${WOW_ADDONS_DIR:?set WOW_ADDONS_DIR to the Interface/AddOns directory of the game}"
dir="${WOW_ADDONS_DIR%/}"
[ "$(basename "$dir")" = "AddOns" ] || { echo "WOW_ADDONS_DIR must end in AddOns, got: $WOW_ADDONS_DIR" >&2; exit 1; }
[ -d "$dir" ] || { echo "not a directory: $dir" >&2; exit 1; }

mkdir -p "$dir/ForeverTogether"
rsync -a --delete addon/ForeverTogether/ "$dir/ForeverTogether/"
echo "installed addon/ForeverTogether -> $dir/ForeverTogether (type /reload in the game)"
