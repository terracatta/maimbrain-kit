#!/usr/bin/env python3
"""Swerve's icon -> icon.png (256x256): the teal car from behind, boost
flames out, tearing down a sunset road. Drawn at 3x and box-filtered down
for smooth edges. Stdlib only (pixel.py's canvas and PNG writer).

    python3 tools/gen_icon.py
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
for _p in (HERE, os.path.join(HERE, "..", "..", "..", "scripts")):  # scaffolded games get copies in tools/
    if os.path.exists(os.path.join(_p, "sfx.py")):
        sys.path.insert(0, _p)
        break
from pixel import Canvas, write_png  # noqa: E402

S = 3  # supersampling
N = 256 * S
INK = "#1d0b1e"
PAINT = "#22d3c5"
PAINT_DARK = "#128a86"
GLASS = "#1b2a44"


def rrect(x, y, w, h, r, n=8):
    """A rounded rectangle as polygon points (in 256-space, scaled to the canvas)."""
    pts = []
    for cx, cy, a0 in ((x + w - r, y + r, -90), (x + w - r, y + h - r, 0), (x + r, y + h - r, 90), (x + r, y + r, 180)):
        for k in range(n + 1):
            a = math.radians(a0 + 90 * k / n)
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def sc(pts):
    return [(x * S, y * S) for x, y in pts]


def grow(pts, d):
    """Pushes a convex-ish polygon out by d from its centroid (outlines)."""
    cx = sum(p[0] for p in pts) / len(pts)
    cy = sum(p[1] for p in pts) / len(pts)
    out = []
    for x, y in pts:
        dx, dy = x - cx, y - cy
        L = math.hypot(dx, dy) or 1.0
        out.append((x + dx / L * d, y + dy / L * d))
    return out


def shape(c, pts, color, ink=3.0):
    if ink:
        c.poly(sc(grow(pts, ink)), INK)
    c.poly(sc(pts), color)


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def main():
    c = Canvas(N, N)
    horizon = 112
    # Sky: plum to orange to peach.
    stops = [(0, (52, 24, 62)), (60, (150, 52, 84)), (100, (240, 120, 80)), (horizon, (255, 186, 130))]
    for j in range(horizon * S):
        y = j / S
        for (y0, c0), (y1, c1) in zip(stops, stops[1:]):
            if y0 <= y <= y1:
                col = lerp(c0, c1, (y - y0) / (y1 - y0))
        c.rect(0, j, N, 1, col + (255,))
    # The sun on the horizon, cut by bands.
    sun_r = 54
    for j in range((horizon - sun_r) * S, horizon * S):
        y = j / S
        dy = horizon - 8 - y
        if abs(dy) > sun_r:
            continue
        band = (y - (horizon - sun_r)) / sun_r
        if y > horizon - 30 and int((y - (horizon - 30)) / 5) % 2 == 1:
            continue
        hw = math.sqrt(max(0.0, sun_r**2 - dy**2))
        col = lerp((255, 236, 150), (255, 120, 90), band)
        c.rect(int((128 - hw) * S), j, int(2 * hw * S), 1, col + (255,))
    # Mesas.
    for x0, w, h in ((0, 60, 18), (40, 34, 10), (196, 60, 22), (170, 30, 12)):
        c.poly(sc([(x0, horizon), (x0 + 8, horizon - h), (x0 + w - 8, horizon - h), (x0 + w, horizon)]), (166, 74, 70, 255))
    # Sand and road.
    c.rect(0, horizon * S, N, N - horizon * S, (217, 154, 108, 255))
    c.poly(sc([(118, horizon), (138, horizon), (290, 256), (-34, 256)]), (59, 50, 69, 255))
    for side in (-1, 1):
        x_far, x_near = 128 + side * 8.5, 128 + side * 136
        c.poly(sc([(x_far - 0.6, horizon), (x_far + 0.6, horizon), (x_near + 4 * side + 3, 256), (x_near + 4 * side - 3, 256)]), (255, 241, 216, 255))
    for k in range(6):
        t0 = (k / 6) ** 2
        t1 = ((k + 0.45) / 6) ** 2
        for side in (-1, 1):
            pts = []
            for t, s in ((t0, -1), (t0, 1), (t1, 1), (t1, -1)):
                y = horizon + t * (256 - horizon)
                x = 128 + side * (3 + t * 44) + s * (0.4 + 2.5 * t)
                pts.append((x, y))
            c.poly(sc(pts), (255, 241, 216, 255))
    # Speed streaks.
    for x, y, L in ((18, 150, 40), (10, 176, 52), (226, 156, 36), (232, 184, 48), (30, 204, 30), (214, 210, 34)):
        c.poly(sc([(x, y), (x + L, y - 1), (x + L, y + 1.5), (x, y + 2.5)]), (255, 244, 232, 150))

    # The car from behind.
    cx = 128
    # Shadow.
    c.ellipse(int((cx - 92) * S), int(222 * S), int(184 * S), int(22 * S), (29, 11, 30, 120))
    # Wheels.
    for side in (-1, 1):
        shape(c, rrect(cx + side * 70 - 15, 196, 30, 36, 7), "#231e29", 0)
    # Body.
    shape(c, rrect(cx - 86, 150, 172, 64, 18), PAINT, 4)
    shape(c, rrect(cx - 82, 192, 164, 20, 8), PAINT_DARK, 0)
    # Cabin and roof.
    shape(c, [(cx - 58, 152), (cx + 58, 152), (cx + 44, 108), (cx - 44, 108)], GLASS, 4)
    shape(c, rrect(cx - 46, 102, 92, 10, 4), PAINT, 0)
    c.poly(sc([(cx - 12, 102), (cx + 12, 102), (cx + 12, 112), (cx - 12, 112)]), "#fff4e8")
    # A glint on the glass.
    c.poly(sc([(cx - 30, 146), (cx - 16, 146), (cx + 4, 114), (cx - 10, 114)]), (120, 160, 220, 110))
    # Stripe down the trunk.
    c.poly(sc([(cx - 12, 152), (cx + 12, 152), (cx + 12, 190), (cx - 12, 190)]), "#fff4e8")
    # Tail lights, glowing.
    for side in (-1, 1):
        x0 = cx + side * 52 - 22
        c.ellipse(int((x0 - 10) * S), int(158 * S), int(64 * S), int(32 * S), (255, 60, 70, 70))
        shape(c, rrect(x0, 166, 44, 13, 5), "#ff2038", 2)
        c.poly(sc(rrect(x0 + 4, 168, 36, 4, 2)), "#ffb0b8")
    # Spoiler on struts.
    for side in (-1, 1):
        shape(c, rrect(cx + side * 40 - 4, 136, 8, 18, 2), "#2a2233", 2)
    shape(c, rrect(cx - 90, 128, 180, 12, 5), "#2a2233", 3)
    # Bumper and exhausts with boost flames.
    shape(c, rrect(cx - 80, 200, 160, 12, 5), "#2a2233", 0)
    for side in (-1, 1):
        x = cx + side * 30
        c.ellipse(int((x - 16) * S), int(206 * S), int(32 * S), int(40 * S), (120, 240, 255, 90))
        c.poly(sc([(x - 8, 210), (x + 8, 210), (x + 3, 246), (x - 3, 246)]), (122, 244, 255, 230))
        c.poly(sc([(x - 4, 210), (x + 4, 210), (x + 1, 232), (x - 1, 232)]), (240, 255, 255, 255))
        shape(c, rrect(x - 7, 202, 14, 9, 4), "#8a8494", 2)

    # Box-filter down to 256.
    out = []
    for j in range(256):
        for i in range(256):
            acc = [0, 0, 0, 0]
            for dj in range(S):
                row = (j * S + dj) * N
                for di in range(S):
                    p = c.p[row + i * S + di]
                    for k in range(4):
                        acc[k] += p[k]
            out.append([v // (S * S) for v in acc])
    write_png(os.path.join(HERE, "..", "icon.png"), 256, 256, out)
    print("wrote icon.png")


if __name__ == "__main__":
    main()
