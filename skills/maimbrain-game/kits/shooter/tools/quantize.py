#!/usr/bin/env python3
"""Quantizes a PNG in place to an 8-bit palette with alpha (256 colors,
Floyd-Steinberg dither): the sprite atlas drops from ~290 KB to ~65 KB with
no visible change on a phone. Used by tools/regen.sh after `mb art atlas`.

    python3 tools/quantize.py assets/sprites.png [colors]

Needs Pillow (pip install pillow).
"""

import os
import sys

from PIL import Image


def main() -> int:
    path = sys.argv[1]
    colors = int(sys.argv[2]) if len(sys.argv) > 2 else 256
    before = os.path.getsize(path)
    im = Image.open(path).convert("RGBA")
    q = im.quantize(colors, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.FLOYDSTEINBERG)
    q.save(path, optimize=True)
    print(f"{path}: {before // 1024} KB -> {os.path.getsize(path) // 1024} KB ({colors} colors)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
