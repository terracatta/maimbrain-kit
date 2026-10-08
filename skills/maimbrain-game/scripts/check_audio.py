"""Checks generated audio without listening: length, peak, loudness,
brightness (a harshness proxy) and, for loops, the jump at the loop point.
Flags silent, clipping and harsh files. You can't hear your sounds, so run
this after every audio change, and scripts/mix_check.py for how they sit
together in the game.

Brightness is the RMS frequency in Hz (a sine of f Hz reads f, white noise
a quarter of the sample rate), so it means the same at any sample rate;
above ~6500 Hz reads as harsh. The loop jump compares the step from the
last sample to the first with the steps right around the seam.

    python3 <skill dir>/scripts/check_audio.py games/<name>/assets/*.ogg
    python3 .../check_audio.py --loops music,wind games/<name>/assets/*.ogg

Files whose name contains "music" or "loop" (or any --loops word) are
checked as loops. Needs oggdec (brew install vorbis-tools).
"""

from __future__ import annotations

import math
import os
import struct
import subprocess
import sys
import tempfile
import wave


def decode(path: str) -> tuple[list[float], int]:
    with tempfile.TemporaryDirectory() as d:
        out = os.path.join(d, "x.wav")
        subprocess.run(["oggdec", "-Q", "-o", out, path], check=True)
        with wave.open(out) as w:
            n, rate, ch = w.getnframes(), w.getframerate(), w.getnchannels()
            raw = w.readframes(n)
    s = struct.unpack(f"<{n * ch}h", raw)[::ch]
    return [v / 32768 for v in s], rate


def db(x: float) -> float:
    return 20 * math.log10(max(x, 1e-9))


def main(argv: list[str]) -> int:
    loops = {"music", "loop"}
    files = []
    it = iter(argv)
    for a in it:
        if a == "--loops":
            loops |= set(next(it).split(","))
        else:
            files.append(a)
    if not files:
        print(__doc__)
        return 2
    bad = 0
    print(f"{'file':22} {'secs':>6} {'peak dB':>8} {'rms dB':>7} {'bright Hz':>9} {'loop jump':>9}  notes")
    for p in sorted(files):
        x, rate = decode(p)
        name = os.path.basename(p)
        peak = max((abs(v) for v in x), default=0.0)
        loud = sorted(v * v for v in x)[len(x) // 3:]
        rms = math.sqrt(sum(loud) / max(1, len(loud)))
        full = math.sqrt(sum(v * v for v in x) / max(1, len(x))) or 1e-9
        diff = math.sqrt(sum((x[i] - x[i - 1]) ** 2 for i in range(1, len(x))) / max(1, len(x)))
        # rms(diff)/rms = 2 sin(pi f / rate) for a sine: invert it, so the
        # number doesn't depend on the sample rate.
        bright = rate / math.pi * math.asin(min(1.0, diff / full / 2))
        notes, jump = [], ""
        if any(w in name for w in loops):
            seam = abs(x[0] - x[-1])
            # The steps on either side of the seam, not the file's average
            # (a quiet intro made loud seams look like clicks).
            k = min(32, len(x) // 4)
            near = [abs(x[i] - x[i - 1]) for i in range(1, k + 1)] + [abs(x[-i] - x[-i - 1]) for i in range(1, k + 1)]
            ratio = seam / max(sum(near) / max(1, len(near)), 1e-4)
            jump = f"{ratio:8.1f}x"
            if ratio > 4 and seam > 0.003:
                notes.append("CLICK at loop seam")
        if peak < 0.01:
            notes.append("SILENT")
        elif db(peak) > -0.5:
            notes.append("clips (peak > -0.5 dB)")
        if peak >= 0.01 and db(rms) < -30 and not jump:  # stems are quiet by design
            notes.append("very quiet")
        if bright > 6500:
            notes.append("harsh/bright (try a lower cutoff)")
        bad += bool(notes)
        print(f"{name:22} {len(x) / rate:6.2f} {db(peak):8.1f} {db(rms):7.1f} {bright:9.0f} {jump:>9}  {', '.join(notes)}")
    print(f"\n{bad} file(s) flagged" if bad else "\nall files look sane")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
