"""Pixel-art canvas, PNG writer and sprite-atlas packer. Python stdlib only.

Draw sprites in code (keep the script in your game's tools/ so the art is
remixable), pack them into one atlas PNG, and get a Rust table of source
rects to pass to `gfx2d::sprite`.

    import sys; sys.path.insert(0, "<skill dir>/scripts")   # or copy this file into your tools/
    from pixel import Canvas, Atlas, hex_rgba

    hero = Canvas(16, 16)
    hero.ellipse(2, 2, 12, 12, "#f2c27b")       # face
    hero.outline("#3b2414")                       # 1-px outline around everything drawn
    hero.px(6, 6, "#000"); hero.px(9, 6, "#000")  # eyes

    atlas = Atlas(scale=4)        # each art pixel becomes a 4x4 block of texels
    atlas.add("hero", hero)
    atlas.save("games/mygame/assets/atlas.png", "games/mygame/src/atlas.rs")

Colors are "#rgb", "#rrggbb", "#rrggbbaa" or (r, g, b[, a]) tuples.

Why `scale`: the platform samples images with linear filtering and draws at up
to 2 device pixels per logical unit (SPEC §5.3). Storing each art pixel as a
block of >= 2x2 texels (4 is safe) keeps pixel art crisp. The generated rects
are in atlas texels; draw them at (w, h) * your on-screen pixel size.
"""

from __future__ import annotations

import struct
import zlib

Color = tuple


def hex_rgba(c) -> Color:
    """'#rgb' | '#rrggbb' | '#rrggbbaa' | tuple -> (r, g, b, a)."""
    if isinstance(c, tuple):
        return c if len(c) == 4 else (*c, 255)
    s = c.lstrip("#")
    if len(s) == 3:
        s = "".join(ch * 2 for ch in s)
    if len(s) == 6:
        s += "ff"
    return tuple(int(s[i : i + 2], 16) for i in (0, 2, 4, 6))


def shade(c, k: float) -> Color:
    """Darken (k < 1) or lighten (k > 1) a color, keeping alpha."""
    r, g, b, a = hex_rgba(c)
    f = (lambda v: min(255, int(v * k))) if k <= 1 else (lambda v: min(255, int(v + (255 - v) * (k - 1))))
    return (f(r), f(g), f(b), a)


class Canvas:
    """An RGBA image you draw into pixel by pixel. (0, 0) is top-left."""

    def __init__(self, w: int, h: int, fill=(0, 0, 0, 0)):
        self.w, self.h = w, h
        f = hex_rgba(fill)
        self.p = [list(f) for _ in range(w * h)]

    # --- primitives -------------------------------------------------------
    def px(self, x: int, y: int, c) -> None:
        if 0 <= x < self.w and 0 <= y < self.h:
            r, g, b, a = hex_rgba(c)
            if a == 255:
                self.p[y * self.w + x] = [r, g, b, a]
            elif a > 0:  # alpha-blend over what's there
                d = self.p[y * self.w + x]
                k = a / 255
                da = d[3] / 255
                oa = k + da * (1 - k)
                for i, v in enumerate((r, g, b)):
                    d[i] = int((v * k + d[i] * da * (1 - k)) / oa) if oa else 0
                d[3] = int(oa * 255)

    def get(self, x: int, y: int) -> Color:
        return tuple(self.p[y * self.w + x]) if 0 <= x < self.w and 0 <= y < self.h else (0, 0, 0, 0)

    def rect(self, x: int, y: int, w: int, h: int, c) -> None:
        for j in range(y, y + h):
            for i in range(x, x + w):
                self.px(i, j, c)

    def ellipse(self, x: int, y: int, w: int, h: int, c) -> None:
        """Filled ellipse inside the box (x, y, w, h)."""
        cx, cy, rx, ry = x + w / 2, y + h / 2, w / 2, h / 2
        for j in range(y, y + h):
            for i in range(x, x + w):
                if ((i + 0.5 - cx) / rx) ** 2 + ((j + 0.5 - cy) / ry) ** 2 <= 1.0:
                    self.px(i, j, c)

    def line(self, x0: int, y0: int, x1: int, y1: int, c) -> None:
        dx, dy = abs(x1 - x0), -abs(y1 - y0)
        sx, sy = (1 if x0 < x1 else -1), (1 if y0 < y1 else -1)
        err = dx + dy
        while True:
            self.px(x0, y0, c)
            if x0 == x1 and y0 == y1:
                return
            e2 = 2 * err
            if e2 >= dy:
                err += dy
                x0 += sx
            if e2 <= dx:
                err += dx
                y0 += sy

    def poly(self, pts, c) -> None:
        """Filled polygon (even-odd), pts = [(x, y), ...] in pixels."""
        ys = [p[1] for p in pts]
        for j in range(max(0, int(min(ys))), min(self.h, int(max(ys)) + 1)):
            yc = j + 0.5
            xs = []
            for (x0, y0), (x1, y1) in zip(pts, pts[1:] + pts[:1]):
                if (y0 <= yc < y1) or (y1 <= yc < y0):
                    xs.append(x0 + (yc - y0) * (x1 - x0) / (y1 - y0))
            xs.sort()
            for a, b in zip(xs[::2], xs[1::2]):
                for i in range(int(round(a)), int(round(b))):
                    self.px(i, j, c)

    def outline(self, c, *, diagonal: bool = False) -> None:
        """1-px outline around every opaque pixel (classic sprite look).
        It can't draw outside the canvas: leave a 1-px transparent margin."""
        solid = {(i, j) for j in range(self.h) for i in range(self.w) if self.p[j * self.w + i][3] > 0}
        nb = [(1, 0), (-1, 0), (0, 1), (0, -1)] + ([(1, 1), (1, -1), (-1, 1), (-1, -1)] if diagonal else [])
        edge = {(i + dx, j + dy) for i, j in solid for dx, dy in nb} - solid
        for i, j in edge:
            self.px(i, j, c)

    def erase(self, x: int, y: int, w: int, h: int) -> None:
        """Makes a box fully transparent (cut holes, trim)."""
        for j in range(max(0, y), min(self.h, y + h)):
            for i in range(max(0, x), min(self.w, x + w)):
                self.p[j * self.w + i] = [0, 0, 0, 0]

    def ring(self, x: int, y: int, w: int, h: int, thickness: int, c) -> None:
        """An elliptical ring inside the box (x, y, w, h)."""
        cx, cy, rx, ry = x + w / 2, y + h / 2, w / 2, h / 2
        ix, iy = max(0.01, rx - thickness), max(0.01, ry - thickness)
        for j in range(y, y + h):
            for i in range(x, x + w):
                dx, dy = i + 0.5 - cx, j + 0.5 - cy
                if (dx / rx) ** 2 + (dy / ry) ** 2 <= 1.0 and (dx / ix) ** 2 + (dy / iy) ** 2 > 1.0:
                    self.px(i, j, c)

    def dither(self, x: int, y: int, w: int, h: int, c, density: float = 0.5) -> None:
        """Ordered (Bayer 4x4) dither of color c over a box: cheap pixel-art shading."""
        bayer = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5]
        for j in range(y, y + h):
            for i in range(x, x + w):
                if bayer[(j % 4) * 4 + (i % 4)] < density * 16 and self.get(i, j)[3] > 0:
                    self.px(i, j, c)

    def blit(self, src: "Canvas", x: int, y: int) -> None:
        for j in range(src.h):
            for i in range(src.w):
                c = src.get(i, j)
                if c[3]:
                    self.px(x + i, y + j, c)

    def flipped(self) -> "Canvas":
        out = Canvas(self.w, self.h)
        for j in range(self.h):
            for i in range(self.w):
                out.p[j * self.w + i] = list(self.get(self.w - 1 - i, j))
        return out

    def scaled(self, k: int) -> "Canvas":
        out = Canvas(self.w * k, self.h * k)
        for j in range(out.h):
            for i in range(out.w):
                out.p[j * out.w + i] = list(self.p[(j // k) * self.w + i // k])
        return out

    # --- output -----------------------------------------------------------
    def save(self, path: str, scale: int = 1) -> None:
        img = self.scaled(scale) if scale > 1 else self
        write_png(path, img.w, img.h, img.p)


def write_png(path: str, w: int, h: int, pixels) -> None:
    """pixels: w*h [r, g, b, a] lists, row-major."""
    raw = bytearray()
    for j in range(h):
        raw.append(0)
        for i in range(w):
            raw.extend(pixels[j * w + i])

    def chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
    png += chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(png)


class Atlas:
    """Packs named canvases into one PNG (shelf packing, 1 art-pixel padding)
    and writes a Rust module of source rects: `pub const HERO: [f32; 4]`."""

    def __init__(self, scale: int = 4, width: int = 256):
        self.scale, self.width = scale, width
        self.items: list[tuple[str, Canvas]] = []

    def add(self, name: str, c: Canvas) -> None:
        self.items.append((name, c))

    def save(self, png_path: str, rust_path: str | None = None) -> dict:
        x = y = shelf = 0
        rects = {}
        for name, c in sorted(self.items, key=lambda it: -it[1].h):
            if x + c.w > self.width:
                x, y, shelf = 0, y + shelf + 1, 0
            rects[name] = (x, y, c.w, c.h)
            x += c.w + 1
            shelf = max(shelf, c.h)
        h = y + shelf
        h = 1 << max(0, (h - 1).bit_length())  # power-of-two height
        sheet = Canvas(self.width, h)
        for name, c in self.items:
            sheet.blit(c, rects[name][0], rects[name][1])
        sheet.save(png_path, self.scale)
        if rust_path:
            s = self.scale
            lines = [
                "//! Generated by tools/ (pixel.py Atlas). Source rects in atlas texels: [x, y, w, h].",
                "#![allow(dead_code)]",
                f"//! Each art pixel is {s}x{s} texels; art size = rect size / {s}.",
                f"pub const SCALE: f32 = {s}.0;",
            ]
            for name, (rx, ry, rw, rh) in rects.items():
                lines.append(f"pub const {name.upper()}: [f32; 4] = [{rx * s}.0, {ry * s}.0, {rw * s}.0, {rh * s}.0];")
            with open(rust_path, "w") as f:
                f.write("\n".join(lines) + "\n")
        return rects
