#!/usr/bin/env python3
"""Quantizes PNGs to an indexed palette (median cut, alpha included), in place.

`mb art atlas` packs sprites but doesn't quantize the sheet, and an RGBA atlas
of paper-textured sprites is ~3x the size of an indexed one. tools/regen.sh
runs this on assets/sprites.png after packing. Stdlib only.

    python3 tools/quantize.py assets/sprites.png [--colors 256]
"""

import struct
import sys
import zlib


def read_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", f"{path}: not a PNG"
    o, idat, plte, trns = 8, [], None, None
    while o < len(data):
        n, kind = struct.unpack(">I4s", data[o : o + 8])
        body = data[o + 8 : o + 8 + n]
        if kind == b"IHDR":
            w, h, depth, ct = struct.unpack(">IIBB", body[:10])
        elif kind == b"PLTE":
            plte = body
        elif kind == b"tRNS":
            trns = body
        elif kind == b"IDAT":
            idat.append(body)
        o += 12 + n
    raw = zlib.decompress(b"".join(idat))
    ch = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[ct]
    bits = ch * depth
    bpp = max(1, bits // 8)
    stride = (w * bits + 7) // 8
    rows, prev = [], bytearray(stride)
    for y in range(h):
        f = raw[y * (stride + 1)]
        line = bytearray(raw[y * (stride + 1) + 1 : (y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b = prev[i]
            c = prev[i - bpp] if i >= bpp else 0
            if f == 1:
                line[i] = (line[i] + a) & 255
            elif f == 2:
                line[i] = (line[i] + b) & 255
            elif f == 3:
                line[i] = (line[i] + ((a + b) >> 1)) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line)
        prev = line
    px = []
    for line in rows:
        for x in range(w):
            if ct == 6:
                px.append(tuple(line[x * 4 : x * 4 + 4]))
            elif ct == 2:
                px.append((line[x * 3], line[x * 3 + 1], line[x * 3 + 2], 255))
            elif ct == 3:
                per = 8 // depth
                idx = line[x] if depth == 8 else (line[x // per] >> (8 - depth * (x % per + 1))) & ((1 << depth) - 1)
                a = trns[idx] if trns and idx < len(trns) else 255
                px.append((plte[idx * 3], plte[idx * 3 + 1], plte[idx * 3 + 2], a))
            elif ct == 0:
                px.append((line[x],) * 3 + (255,))
            else:
                px.append((line[x * 2],) * 3 + (line[x * 2 + 1],))
    return w, h, px


def median_cut(counts, n):
    boxes = [list(counts.items())]
    while len(boxes) < n:
        # Split the box with the largest weighted spread.
        best, bi, bc = -1, -1, 0
        for i, box in enumerate(boxes):
            if len(box) < 2:
                continue
            for c in range(4):
                vals = [k[c] for k, _ in box]
                spread = (max(vals) - min(vals)) * (2 if c == 3 else 1) * sum(v for _, v in box) ** 0.5
                if spread > best:
                    best, bi, bc = spread, i, c
        if bi < 0:
            break
        box = sorted(boxes.pop(bi), key=lambda kv: kv[0][bc])
        total, acc, cut = sum(v for _, v in box), 0, 1
        for j, (_, v) in enumerate(box):
            acc += v
            if acc >= total / 2:
                cut = max(1, min(len(box) - 1, j))
                break
        boxes += [box[:cut], box[cut:]]
    pal = []
    for box in boxes:
        t = sum(v for _, v in box)
        pal.append(tuple(round(sum(k[c] * v for k, v in box) / t) for c in range(4)))
    return pal


def write_png(path, w, h, pal, idx):
    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)

    raw = bytearray()
    for y in range(h):
        raw.append(0)
        raw += bytes(idx[y * w : (y + 1) * w])
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 3, 0, 0, 0))
    png += chunk(b"PLTE", bytes(c for p in pal for c in p[:3]))
    alphas = [p[3] for p in pal]
    last = max((i for i, a in enumerate(alphas) if a < 255), default=-1)
    if last >= 0:
        png += chunk(b"tRNS", bytes(alphas[: last + 1]))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    open(path, "wb").write(png)
    return len(png)


def quantize(path, n):
    w, h, px = read_png(path)
    # Fully transparent texels are one color; the rest keep their alpha.
    px = [p if p[3] > 0 else (0, 0, 0, 0) for p in px]
    counts = {}
    for p in px:
        counts[p] = counts.get(p, 0) + 1
    opaque = {k: v for k, v in counts.items() if k[3] > 0}
    pal = [(0, 0, 0, 0)] + (median_cut(opaque, n - 1) if len(opaque) > n - 1 else list(opaque))
    # Transparent first so tRNS stays short; then by alpha (translucent first).
    pal = [pal[0]] + sorted(pal[1:], key=lambda p: p[3])
    lookup = {(0, 0, 0, 0): 0}
    for k in opaque:
        best, bd = 1, 1 << 30
        for i in range(1, len(pal)):
            q = pal[i]
            d = (k[0] - q[0]) ** 2 * 3 + (k[1] - q[1]) ** 2 * 4 + (k[2] - q[2]) ** 2 * 2 + (k[3] - q[3]) ** 2 * 6
            if d < bd:
                best, bd = i, d
        lookup[k] = best
    size = write_png(path, w, h, pal, [lookup[p] for p in px])
    print(f"{path}: {w}x{h}, {len(counts)} colors -> {len(pal)}, {size // 1024} KB")


if __name__ == "__main__":
    args = sys.argv[1:]
    n = 256
    if "--colors" in args:
        i = args.index("--colors")
        n = int(args[i + 1])
        del args[i : i + 2]
    for f in args:
        quantize(f, max(2, min(256, n)))
