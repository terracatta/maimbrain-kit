#!/usr/bin/env python3
"""Draws art/ref/map_sketch.png: the road layout (PATH, ROAD_W and CAKE from
src/sim.rs, copied below) at 2 px per unit. `mb art background --ref` gets it
so the generated map's road lands where the jellies walk. Change the map in
sim.rs, copy the numbers here, rerun, then regenerate the map (tools/regen.sh map).

    python3 tools/map_sketch.py art/ref/map_sketch.png
"""
import zlib, struct, sys, math
W, H = 720, 1280
PATH = [(196,-24),(196,160),(84,160),(84,284),(276,284),(276,404),(180,404),(180,492)]
ROAD_W = 36; CAKE = (180, 522)
grass = (120, 200, 120); road = (243, 217, 164); clear = (200, 230, 170)
px = bytearray(W*H*3)
for i in range(W*H): px[i*3:i*3+3] = bytes(grass)
def seg_dist(x, y, a, b):
    ax, ay = a; bx, by = b
    dx, dy = bx-ax, by-ay; L = dx*dx+dy*dy
    t = max(0, min(1, ((x-ax)*dx+(y-ay)*dy)/L)) if L else 0
    return math.hypot(x-ax-t*dx, y-ay-t*dy)
for py in range(H):
    for pxx in range(W):
        x, y = (pxx+0.5)/2, (py+0.5)/2
        if math.hypot(x-CAKE[0], (y-CAKE[1]-14)*1.6) < 52: c = clear
        else: c = None
        for a, b in zip(PATH, PATH[1:]):
            if seg_dist(x, y, a, b) <= ROAD_W/2: c = road; break
        if c: i = (py*W+pxx)*3; px[i:i+3] = bytes(c)
raw = b''.join(b'\0' + bytes(px[r*W*3:(r+1)*W*3]) for r in range(H))
def chunk(t, d): return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t+d) & 0xffffffff)
png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', W, H, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b'')
open(sys.argv[1] if len(sys.argv) > 1 else 'art/ref/map_sketch.png', 'wb').write(png)
