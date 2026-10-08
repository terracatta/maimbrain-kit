"""Writes games/gallery/icon.png: a plum tile with a pink button and a gold star.

    python3 games/gallery/tools/icon.py
"""
import math
import os
import struct
import zlib

N = 256


def write_png(path, w, h, rows):
    raw = b"".join(b"\0" + bytes(r) for r in rows)

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    open(path, "wb").write(png)


def rrect_d(x, y, cx, cy, hw, hh, r):
    qx, qy = abs(x - cx) - hw + r, abs(y - cy) - hh + r
    return math.hypot(max(qx, 0), max(qy, 0)) + min(max(qx, qy), 0) - r


def star_d(x, y, cx, cy, r):
    # Inside test via polygon winding, distance approximated by edges.
    pts = []
    for i in range(10):
        a = i * math.pi / 5
        rr = r if i % 2 == 0 else r * 0.48
        pts.append((cx + rr * math.sin(a), cy - rr * math.cos(a)))
    inside = False
    j = len(pts) - 1
    for i in range(len(pts)):
        xi, yi = pts[i]
        xj, yj = pts[j]
        if (yi > y) != (yj > y) and x < (xj - xi) * (y - yi) / (yj - yi) + xi:
            inside = not inside
        j = i
    d = min(seg_d(x, y, pts[i], pts[i - 1]) for i in range(len(pts)))
    return -d if inside else d


def seg_d(x, y, a, b):
    ax, ay = a
    bx, by = b
    dx, dy = bx - ax, by - ay
    t = max(0, min(1, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)))
    return math.hypot(x - ax - t * dx, y - ay - t * dy)


def over(dst, src, a):
    return tuple(int(dst[i] * (1 - a) + src[i] * a) for i in range(3))


def main():
    rows = []
    for y in range(N):
        row = []
        for x in range(N):
            t = y / N
            c = (int(0x3A * (1 - t) + 0x1A * t), int(0x1C * (1 - t) + 0x0B * t), int(0x71 * (1 - t) + 0x3A * t))
            # Button shadow, edge, face.
            sd = rrect_d(x, y, 128, 150, 92, 58, 40)
            c = over(c, (10, 0, 26), max(0, min(1, 0.5 - sd / 18)) * 0.5)
            ed = rrect_d(x, y, 128, 140, 92, 58, 40)
            c = over(c, (0xA0, 0x2A, 0x5E), max(0, min(1, 0.5 - ed)))
            fd = rrect_d(x, y, 128, 128, 92, 58, 40)
            k = (y - 70) / 116
            face = (int(0xFF * 1), int(0x7A * (1 - k) + 0x46 * k), int(0xB4 * (1 - k) + 0x92 * k))
            c = over(c, face, max(0, min(1, 0.5 - fd)))
            sd2 = star_d(x, y, 128, 130, 46)
            c = over(c, (0x1D, 0x0B, 0x3D), max(0, min(1, 0.5 - (sd2 - 6))))
            c = over(c, (0xFF, 0xD2, 0x3F), max(0, min(1, 0.5 - sd2)))
            row.extend([*c, 255])
        rows.append(row)
    out = os.path.join(os.path.dirname(__file__), "..", "icon.png")
    write_png(out, N, N, rows)


if __name__ == "__main__":
    main()
