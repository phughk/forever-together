# /// script
# requires-python = ">=3.10"
# dependencies = ["pillow>=10"]
# ///
"""Make the addon's AddOns-list icon: a 64x64, 32-bit, uncompressed TGA (WoW loads TGA and BLP, not PNG)."""
import sys
from pathlib import Path

from PIL import Image

SIZE = 64

src, dst = Path(sys.argv[1]), Path(sys.argv[2])
im = Image.open(src).convert("RGBA").resize((SIZE, SIZE), Image.LANCZOS)
dst.parent.mkdir(parents=True, exist_ok=True)
im.save(dst, format="TGA", compression=None)

# Check what the game needs: image type 2 (uncompressed true-colour), 32 bits per pixel, 64x64.
h = dst.read_bytes()[:18]
w, ht = h[12] | h[13] << 8, h[14] | h[15] << 8
assert (h[2], h[16], w, ht) == (2, 32, SIZE, SIZE), f"unexpected TGA header {list(h)}"
print(f"wrote {dst} ({w}x{ht}, 32-bit, uncompressed)")
