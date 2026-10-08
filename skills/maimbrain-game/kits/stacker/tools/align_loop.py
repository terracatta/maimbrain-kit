#!/usr/bin/env python3
"""Locks a second music loop onto the first so the two can play as stems.

`mb music` cuts each track to whole bars on its own, so two tracks asked for
at the same tempo come out a few milliseconds apart in length and start at
different points of the beat. This stretches the second loop to exactly the
first's length, or a whole fraction of it (a 4-bar layer under an 8-bar loop
plays twice per loop; the stretch is a fraction of a percent: inaudible), and
rotates it (a seamless loop stays seamless from any start point) to where its
onsets line up best with the first's. Needs oggdec/oggenc; stdlib only.

    python3 tools/align_loop.py assets/music.ogg assets/tense.ogg [-q 3]
"""

import array
import math
import os
import subprocess
import sys
import tempfile
import wave


def read(path):
    with tempfile.TemporaryDirectory() as d:
        wav = os.path.join(d, "a.wav")
        subprocess.run(["oggdec", "-Q", "-o", wav, path], check=True)
        with wave.open(wav) as w:
            assert w.getsampwidth() == 2 and w.getnchannels() == 1, "expects mono 16-bit"
            rate = w.getframerate()
            s = array.array("h", w.readframes(w.getnframes()))
    return rate, [v / 32768.0 for v in s]


def write(path, rate, samples, quality):
    with tempfile.TemporaryDirectory() as d:
        wav = os.path.join(d, "a.wav")
        with wave.open(wav, "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(rate)
            w.writeframes(array.array("h", (max(-32767, min(32767, round(v * 32767))) for v in samples)).tobytes())
        subprocess.run(["oggenc", "-Q", "-q", str(quality), "-o", path, wav], check=True)


def onsets(s, hop):
    # Positive change of loudness per hop: where notes and hits start.
    env = []
    for i in range(0, len(s) - hop + 1, hop):
        e = math.sqrt(sum(v * v for v in s[i : i + hop]) / hop)
        env.append(e)
    return [max(0.0, env[i] - env[i - 1]) for i in range(len(env))]


def main():
    args = sys.argv[1:]
    q = 3
    if "-q" in args:
        i = args.index("-q")
        q = args[i + 1]
        del args[i : i + 2]
    lead, follow = args
    rate, a = read(lead)
    rate_b, b = read(follow)
    assert rate == rate_b, "both loops need the same sample rate"
    reps = max(1, round(len(a) / len(b)))
    n = len(a) // reps
    # Stretch: linear interpolation over the loop (wrapping at the seam).
    k = len(b) / n
    b = [b[int(i * k) % len(b)] * (1 - (i * k) % 1) + b[(int(i * k) + 1) % len(b)] * ((i * k) % 1) for i in range(n)]
    hop = 256
    # The lead's onsets folded onto the follower's length.
    ea = onsets(a[: n * reps], hop)
    per = len(ea) // reps
    ea = [sum(ea[i + r * per] for r in range(reps)) for i in range(per)]
    eb = onsets(b, hop)
    m = min(len(ea), len(eb))
    best, lag = -1.0, 0
    for l in range(m):
        c = sum(ea[i] * eb[(i + l) % m] for i in range(0, m))
        if c > best:
            best, lag = c, l
    shift = (lag * hop) % n
    b = b[shift:] + b[:shift]
    write(follow, rate, b, q)
    print(f"{follow}: stretched {k:.5f}x to {n / rate:.3f} s ({reps} per loop) and rotated {shift / rate * 1000:.0f} ms onto {lead}")


if __name__ == "__main__":
    main()
