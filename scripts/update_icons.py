"""Regenerate every icon from the master image (default assets/icon_1024.png).

- Desktop app: the Tauri CLI (from webapp/node_modules) writes all sizes and formats into app/icons/.
- Web app: webapp/public/favicon.png (64x64) and apple-touch-icon.png (180x180, used by phones).
- Addon: a 64x64, 32-bit, uncompressed TGA (WoW loads TGA and BLP, not PNG) at addon/ForeverTogether/icon.tga.

Run from anywhere: uv run --project scripts scripts/update_icons.py [master.png]
"""

import subprocess
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MASTER = ROOT / "assets" / "icon_1024.png"
ADDON_ICON = ROOT / "addon" / "ForeverTogether" / "icon.tga"
ADDON_ICON_SIZE = 64
WEB_ICONS = {"favicon.png": 64, "apple-touch-icon.png": 180}  # in webapp/public, served at the site root
WEB_PUBLIC = ROOT / "webapp" / "public"
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
    cli = ROOT / "webapp" / "node_modules" / ".bin" / "tauri"
    if not cli.exists():
        sys.exit("update_icons: run `mise run setup` first (webapp/node_modules/.bin/tauri is missing)")
    out = ROOT / "app" / "icons"
    subprocess.run([str(cli), "icon", str(master), "--output", str(out)], cwd=ROOT / "app", check=True)


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


def web_icons(img: Image.Image) -> None:
    WEB_PUBLIC.mkdir(parents=True, exist_ok=True)
    for name, size in WEB_ICONS.items():
        dst = WEB_PUBLIC / name
        img.resize((size, size), Image.LANCZOS).save(dst, format="PNG", optimize=True)
        print(f"wrote {dst.relative_to(ROOT)} ({size}x{size})")


def main() -> None:
    master = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else DEFAULT_MASTER
    img = load_master(master)
    tauri_icons(master)
    addon_icon(img, ADDON_ICON)
    web_icons(img)
    print("done")


if __name__ == "__main__":
    main()
