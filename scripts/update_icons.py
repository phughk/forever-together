"""Regenerate every icon from the master image (default assets_vc/icon_1024.png).

- Desktop app: `pnpm tauri icon` writes all sizes and formats into src-tauri/icons/.
- Addon: a 64x64, 32-bit, uncompressed TGA (WoW loads TGA and BLP, not PNG) at addon/ForeverTogether/icon.tga.

Run from anywhere: uv run --project scripts scripts/update_icons.py [master.png]
"""

import subprocess
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MASTER = ROOT / "assets_vc" / "icon_1024.png"
ADDON_ICON = ROOT / "addon" / "ForeverTogether" / "icon.tga"
ADDON_ICON_SIZE = 64
MIN_MASTER_SIZE = 1024


def load_master(path: Path) -> Image.Image:
    if not path.is_file():
        sys.exit(f"update_icons: {path} not found")
    img = Image.open(path)
    w, h = img.size
    if w != h or w < MIN_MASTER_SIZE:
        sys.exit(f"update_icons: {path} is {w}x{h}; it must be square and at least {MIN_MASTER_SIZE}x{MIN_MASTER_SIZE}")
    return img.convert("RGBA")


def tauri_icons(master: Path) -> None:
    subprocess.run(["pnpm", "tauri", "icon", str(master)], cwd=ROOT, check=True)


def addon_icon(img: Image.Image, dst: Path) -> None:
    small = img.resize((ADDON_ICON_SIZE, ADDON_ICON_SIZE), Image.LANCZOS)
    dst.parent.mkdir(parents=True, exist_ok=True)
    small.save(dst, format="TGA", compression=None)
    # Check what the game needs: image type 2 (uncompressed true-colour), 32 bits per pixel, 64x64.
    h = dst.read_bytes()[:18]
    w, ht = h[12] | h[13] << 8, h[14] | h[15] << 8
    if (h[2], h[16], w, ht) != (2, 32, ADDON_ICON_SIZE, ADDON_ICON_SIZE):
        sys.exit(f"update_icons: unexpected TGA header {list(h)}")
    print(f"wrote {dst.relative_to(ROOT)} ({w}x{ht}, 32-bit, uncompressed)")


def main() -> None:
    master = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else DEFAULT_MASTER
    img = load_master(master)
    tauri_icons(master)
    addon_icon(img, ADDON_ICON)
    print("done")


if __name__ == "__main__":
    main()
