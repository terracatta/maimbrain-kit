"""Renders a game's sound events with its real assets and measures the mix,
since you can't listen to it. assets.md §5 "Mix the sound" has the rules.

    python3 <skill dir>/scripts/mix_check.py games/<name>/assets mix.log
    python3 .../mix_check.py games/<name>/assets mix.log --out mix.wav --json mix.json

The event log is plain text, one event per line (`#` starts a comment),
whitespace- or comma-separated:

    <seconds> <sound> <vol> <pitch> [play|loop|set|stop]

    0.10 music 0.70 1.0 loop     # a looping voice (music, stems, ambience)
    1.25 pop   0.45 1.12         # a one-shot ("play" is the default)
    3.00 music 0.28 1.0 set      # new vol/pitch for that sound's loop voice
    9.80 music 0 1 stop          # stops the loop (or, for a one-shot, its oldest voice)

`sound` is the asset's file name without `.ogg`; vol and pitch are what the
game passes to `Sound::play` (vol 1 = the file's own level, pitch = rate).
Get a log from a bot-played round: the kits' sound modules record one in
tests (`MB_MIX_LOG=/tmp/mix.log cargo test -p <crate> mix_log`), or write
one by hand from the game's cue rates.

It mixes in mono like the platform (master gain 0.8 into a -9 dB 12:1
limiter, 32 voices, oldest one-shot stolen first), measures loudness
K-weighted (LUFS-like: 400 ms momentary, 3 s short-term) per bus and per
sound, and reports: how far the music sits above the summed effects, how
hard the limiter works, each sound's rate, level relative to the music,
share of the effects, overlap, variation, brightness, attack and, for
tonal sounds, whether the notes played sit in the music's key (measured:
generators ignore the key you ask for). Loop voices count as music unless
named with --fx-loop. Exit code 1 if anything is flagged. Needs oggdec
(brew install vorbis-tools); numpy makes it faster but isn't required.
"""

from __future__ import annotations

import argparse
import array
import json
import math
import os
import subprocess
import sys
import tempfile
import wave

try:
    import numpy as np
except ImportError:  # stdlib fallback
    np = None

MASTER = 0.8  # runtime/src/audio.ts MASTER_GAIN
LIMITER = (-9.0, 6.0, 12.0, 0.003, 0.15)  # threshold dB, knee dB, ratio, attack s, release s
VOICES = 32
BLOCK = 0.1  # loudness blocks; momentary = 4 blocks, short-term = 30
FREQUENT = 40.0  # plays per minute: at or above this a sound is "frequent"
RARE = 6.0  # below this, "rare"
# Rules (assets.md §5), in dB relative to the music's typical short-term level.
RULE_SFX_BUS = 3.0  # music minus all effects (momentary), median, at least
RULE_SFX_P10 = -6.0  # ... and in 90% of the windows
RULE_FREQUENT = -8.0  # a frequent sound's own short-term level (p90), at most
RULE_REGULAR = -4.0
RULE_REG_HIT = 4.0
RULE_RARE_HIT = 8.0  # a rare sound's loudest moment, at most
RULE_FREQ_HIT = 0.0  # a frequent sound's loudest moment, at most
RULE_SHARE = 0.5  # share of the effects' energy one frequent sound may take
RULE_BRIGHT = 3500.0  # Hz (rms frequency) for frequent sounds
RULE_HARSH = 6500.0  # Hz for any sound
RULE_OVERLAP = 4  # same-sound voices at once, before it needs a cap
NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
MAJOR = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88]
MINOR = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17]
SCALE = {"major": {0, 2, 4, 5, 7, 9, 11}, "minor": {0, 2, 3, 5, 7, 8, 10}}


def db(x: float) -> float:
    return 20 * math.log10(max(x, 1e-12))


def lufs(ms: float) -> float:
    return -0.691 + 10 * math.log10(max(ms, 1e-12))


# ---- Assets ----------------------------------------------------------------


def decode(path: str) -> tuple[list[float], int]:
    with tempfile.TemporaryDirectory() as d:
        out = os.path.join(d, "x.wav")
        if path.endswith(".wav"):
            out = path
        else:
            subprocess.run(["oggdec", "-Q", "-o", out, path], check=True)
        with wave.open(out) as w:
            n, rate, ch = w.getnframes(), w.getframerate(), w.getnchannels()
            a = array.array("h")
            a.frombytes(w.readframes(n))
    if sys.byteorder == "big":
        a.byteswap()
    if ch == 1:
        return [v / 32768 for v in a], rate
    return [sum(a[i : i + ch]) / (32768 * ch) for i in range(0, len(a), ch)], rate


def biquad(x: list[float], b: tuple, a: tuple) -> list[float]:
    b0, b1, b2 = (v / a[0] for v in b)
    a1, a2 = a[1] / a[0], a[2] / a[0]
    y, x1, x2, y1, y2 = [0.0] * len(x), 0.0, 0.0, 0.0, 0.0
    for i, v in enumerate(x):
        o = b0 * v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2
        x2, x1, y2, y1 = x1, v, y1, o
        y[i] = o
    return y


def k_weight(x: list[float], rate: int) -> list[float]:
    """BS.1770 K-weighting at any rate (the analog prototypes, bilinear)."""
    w = 2 * math.pi * 1681.974450955533 / rate
    A, al = 10 ** (4.0 / 40), math.sin(w) / (2 * 0.7071752369554196)
    c, s = math.cos(w), 2 * math.sqrt(A) * al
    shelf = ((A * ((A + 1) + (A - 1) * c + s), -2 * A * ((A - 1) + (A + 1) * c), A * ((A + 1) + (A - 1) * c - s)),
             ((A + 1) - (A - 1) * c + s, 2 * ((A - 1) - (A + 1) * c), (A + 1) - (A - 1) * c - s))
    w = 2 * math.pi * 38.13547087602444 / rate
    c, al = math.cos(w), math.sin(w) / (2 * 0.5003270373238773)
    hp = (((1 + c) / 2, -(1 + c), (1 + c) / 2), (1 + al, -2 * c, 1 - al))
    return biquad(biquad(x, *shelf), *hp)


def bright_hz(x: list[float], rate: int) -> float:
    """RMS frequency: a sine of f Hz has rms(diff)/rms = 2 sin(pi f / rate),
    so inverting that gives a rate-independent brightness (white noise reads
    rate/4)."""
    full = math.sqrt(sum(v * v for v in x) / max(1, len(x)))
    if full < 1e-6:
        return 0.0
    diff = math.sqrt(sum((x[i] - x[i - 1]) ** 2 for i in range(1, len(x))) / max(1, len(x)))
    return rate / math.pi * math.asin(min(1.0, diff / full / 2))


def envelope(x: list[float], rate: int, win: float = 0.005) -> list[float]:
    h = max(1, int(rate * win))
    return [math.sqrt(sum(v * v for v in x[i : i + h]) / h) for i in range(0, len(x), h)]


def power(frame: list[float]) -> list[float]:
    """|FFT|^2 of a power-of-two frame, bins 0..n/2 (iterative radix-2; numpy if present)."""
    if np is not None:
        return list(np.abs(np.fft.rfft(frame)) ** 2)
    n = len(frame)
    a = [complex(v) for v in frame]
    j = 0
    for i in range(1, n):  # bit reversal
        bit = n >> 1
        while j & bit:
            j ^= bit
            bit >>= 1
        j |= bit
        if i < j:
            a[i], a[j] = a[j], a[i]
    size = 2
    while size <= n:
        w = complex(math.cos(2 * math.pi / size), -math.sin(2 * math.pi / size))
        half = size // 2
        tw = [w**k for k in range(half)]
        for start in range(0, n, size):
            for k in range(half):
                u, t = a[start + k], a[start + k + half] * tw[k]
                a[start + k], a[start + k + half] = u + t, u - t
        size *= 2
    return [abs(c) ** 2 for c in a[: n // 2 + 1]]


def chroma(x: list[float], rate: int, secs: float) -> list[float]:
    """Power per pitch class (55 Hz-2 kHz), from Hann frames of 0.37 s over
    the first `secs` (at most 30 s with numpy, 12 s without)."""
    d = max(1, rate // 5512)
    tri = [sum(x[i : i + d]) / d for i in range(len(x) - d + 1)]  # two box passes: less aliasing
    y = [sum(tri[i : i + d]) / d for i in range(0, len(tri) - d + 1, d)]
    r = rate / d
    y = y[: int(min(secs, 30 if np is not None else 12) * r)]
    out = [0.0] * 12
    f = 2048 if len(y) >= 2048 else 1 << max(7, len(y).bit_length() - 1)
    if len(y) < f:
        return out
    win = [0.5 - 0.5 * math.cos(2 * math.pi * k / (f - 1)) for k in range(f)]
    bins = [(k, (round(12 * math.log2(k * r / f / 440)) + 9) % 12) for k in range(1, f // 2) if 55 < k * r / f < 2000]
    for i in range(0, len(y) - f + 1, f // 2):
        p = power([y[i + k] * win[k] for k in range(f)])
        for k, pc in bins:
            out[pc] += p[k]
    return out


def in_scale(ch: list[float], scale, semis: int = 0) -> float:
    """Share of the chroma's power on the scale's notes, played `semis` up
    (`scale` a set of pitch classes, or a list of sets: the best fit)."""
    if isinstance(scale, list):
        return max(in_scale(ch, s, semis) for s in scale)
    tot = sum(ch) or 1e-12
    return sum(ch[i] for i in range(12) if (i + semis) % 12 in scale) / tot


def keys_of(ch: list[float]) -> list[tuple[int, str, float]]:
    """(root, mode, fit) for all 24 keys, best first: the share of power on
    the key's scale (Lyria and friends ignore the key you ask for, so
    measure it), ties between relative keys broken by Krumhansl profiles."""
    def corr(a, b):
        ma, mb = sum(a) / 12, sum(b) / 12
        num = sum((a[i] - ma) * (b[i] - mb) for i in range(12))
        den = math.sqrt(sum((v - ma) ** 2 for v in a) * sum((v - mb) ** 2 for v in b)) or 1e-9
        return num / den

    cand = []
    for r in range(12):
        for m, prof in (("major", MAJOR), ("minor", MINOR)):
            fit = in_scale(ch, {(r + d) % 12 for d in SCALE[m]})
            cand.append((r, m, fit, corr([ch[(i + r) % 12] for i in range(12)], prof)))
    cand.sort(key=lambda t: -(round(t[2], 2) + 0.001 * t[3]))
    return [(r, m, fit) for r, m, fit, _ in cand]


class Asset:
    def __init__(self, path: str):
        self.x, self.rate = decode(path)
        x = self.x
        self.secs = len(x) / self.rate
        self.peak = max((abs(v) for v in x), default=0.0)
        last = max((i for i in range(len(x) - 1, -1, -64) if abs(x[i]) > self.peak * 0.01), default=0)
        self.audible = last / self.rate + 0.01
        self.k = k_weight(x, self.rate)
        self.bright = bright_hz(x, self.rate)
        env = envelope(x[: int(self.rate * 1.0)], self.rate)
        top = max(env, default=0.0)
        i10 = next((i for i, e in enumerate(env) if e >= 0.1 * top), 0)
        i90 = next((i for i, e in enumerate(env) if e >= 0.9 * top), 0)
        self.attack = (i90 - i10) * 5.0
        self.ch = chroma(x, self.rate, 30.0 if self.secs > 3 else min(self.secs, 0.6))
        tot = sum(self.ch) or 1e-9
        # Tonal: one pitch class well above the rest (noise and thuds are flat).
        self.pc = max(range(12), key=lambda i: self.ch[i])
        self.tonal = self.ch[self.pc] / tot >= 0.4 and self.ch[self.pc] >= 3 * sorted(self.ch)[-3]
        if np is not None:
            self.xa, self.ka = np.asarray(self.x, dtype=np.float64), np.asarray(self.k, dtype=np.float64)


# ---- Events and voices -----------------------------------------------------


class Voice:
    def __init__(self, sound: str, t: float, vol: float, pitch: float, loop: bool):
        self.sound, self.start, self.loop = sound, t, loop
        self.segs = [(t, vol, pitch)]
        self.end = math.inf
        self.stolen = False


def parse(path: str) -> list[tuple[float, str, float, float, str]]:
    ev = []
    with open(path) as f:
        for ln, line in enumerate(f, 1):
            line = line.split("#")[0].replace(",", " ").split()
            if not line:
                continue
            try:
                op = line[4] if len(line) > 4 else "play"
                ev.append((float(line[0]), os.path.splitext(os.path.basename(line[1]))[0], float(line[2]), float(line[3]), op))
            except (ValueError, IndexError):
                sys.exit(f"{path}:{ln}: expected '<seconds> <sound> <vol> <pitch> [play|loop|set|stop]'")
            if op not in ("play", "loop", "set", "stop"):
                sys.exit(f"{path}:{ln}: unknown op {op!r}")
    ev.sort(key=lambda e: e[0])
    return ev


def schedule(ev, assets: dict, end: float) -> tuple[list[Voice], int]:
    voices: list[Voice] = []
    active: list[Voice] = []
    loops: dict[str, Voice] = {}
    steals = 0
    for t, s, vol, pitch, op in ev:
        if op in ("set", "stop"):
            v = loops.get(s)
            if v is None:
                if op == "stop":  # no loop: stops the oldest one-shot of it still sounding (a voice cap)
                    v = next((v for v in voices if v.sound == s and not v.loop and v.end > t), None)
                    if v is not None:
                        v.end = t
                continue
            if op == "set":
                v.segs.append((t, vol, pitch))
            else:
                v.end = t
                del loops[s]
            continue
        a = assets[s]
        active = [v for v in active if v.end > t]
        if len(active) >= VOICES:
            victim = next((v for v in active if not v.loop), active[0])
            victim.end, victim.stolen = t, True
            active.remove(victim)
            steals += 1
        v = Voice(s, t, vol, pitch, op == "loop")
        if v.loop:
            if s in loops:
                loops[s].end = t
            loops[s] = v
        else:
            v.end = t + a.secs / max(pitch, 1e-3)
        voices.append(v)
        active.append(v)
    for v in voices:
        v.end = min(v.end, end)
    return voices, steals


def render(v: Voice, a: Asset, rate: int, n_out: int):
    """The voice's raw and K-weighted samples at `rate`, from sample index i0."""
    i0 = int(round(v.start * rate))
    i1 = min(n_out, int(round(v.end * rate)))
    if i1 <= i0:
        return i0, [], []
    bounds = [(max(i0, int(round(t * rate))), vol, p) for t, vol, p in v.segs] + [(i1, 0, 0)]
    pos, n_src = 0.0, len(a.x)
    raws, ks = [], []
    for (s0, vol, p), (s1, _, _) in zip(bounds, bounds[1:]):
        m = s1 - s0
        if m <= 0:
            continue
        step = p * a.rate / rate
        if np is not None:
            idx = pos + step * np.arange(m)
            if v.loop:
                idx = np.mod(idx, n_src)
            else:
                ok = idx < n_src - 1
                idx = idx[ok]
            raws.append(vol * np.interp(idx, np.arange(n_src), a.xa))
            ks.append(vol * np.interp(idx, np.arange(n_src), a.ka))
        else:
            r, k = [], []
            x, kx = a.x, a.k
            for j in range(m):
                q = pos + step * j
                if v.loop:
                    q %= n_src
                elif q >= n_src - 1:
                    break
                i = int(q)
                f = q - i
                i2 = (i + 1) % n_src
                r.append(vol * (x[i] + (x[i2] - x[i]) * f))
                k.append(vol * (kx[i] + (kx[i2] - kx[i]) * f))
            raws.append(r)
            ks.append(k)
        pos += step * m
        if not v.loop and pos >= n_src:
            break
    if np is not None:
        return i0, np.concatenate(raws) if raws else np.zeros(0), np.concatenate(ks) if ks else np.zeros(0)
    return i0, [y for r in raws for y in r], [y for k in ks for y in k]


def zeros(n: int):
    return np.zeros(n) if np is not None else array.array("d", bytes(8 * n))


def add(dst, i0: int, src) -> None:
    n = min(len(src), len(dst) - i0)
    if n <= 0:
        return
    if np is not None:
        dst[i0 : i0 + n] += src[:n]
    else:
        for j in range(n):
            dst[i0 + j] += src[j]


def blocks(x, rate: int) -> list[float]:
    h = int(rate * BLOCK)
    if np is not None:
        n = len(x) // h
        return list((np.asarray(x[: n * h]) ** 2).reshape(n, h).mean(axis=1))
    return [sum(v * v for v in x[i : i + h]) / h for i in range(0, len(x) - h + 1, h)]


def windows(ms: list[float], n: int) -> list[float]:
    """Loudness of each n-block window ending at each block."""
    out, acc = [], 0.0
    for i, v in enumerate(ms):
        acc += v
        if i >= n:
            acc -= ms[i - n]
        out.append(lufs(acc / min(i + 1, n)))
    return out


def limiter(x, rate: int) -> tuple[list[float], float]:
    """WebAudio-style compressor on the master: gain reduction (dB) per block, and the peak (dBFS) going in."""
    thr, knee, ratio, att, rel = LIMITER
    ca, cr = math.exp(-1 / (att * rate)), math.exp(-1 / (rel * rate))
    gr, out, h, acc, peak = 0.0, [], int(rate * BLOCK), 0.0, 0.0
    lo = thr - knee / 2
    xs = x.tolist() if np is not None else x
    for i, v in enumerate(xs):
        a = abs(v)
        peak = max(peak, a)
        lvl = 20 * math.log10(a) if a > 1e-6 else -120.0
        if lvl <= lo:
            want = 0.0
        elif lvl < thr + knee / 2:
            want = (1 / ratio - 1) * (lvl - lo) ** 2 / (2 * knee)
        else:
            want = (1 / ratio - 1) * (lvl - thr)
        want = -want
        gr = want + (gr - want) * (ca if want > gr else cr)
        acc += gr
        if (i + 1) % h == 0:
            out.append(acc / h)
            acc = 0.0
    return out, db(peak)


def pct(v: list[float], p: float) -> float:
    s = sorted(v)
    return s[min(len(s) - 1, max(0, int(round((len(s) - 1) * p))))] if s else float("nan")


# ---- Report ----------------------------------------------------------------


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("assets", help="the game's assets/ directory")
    ap.add_argument("log", help="event log (see the module docstring)")
    ap.add_argument("--out", help="write the mixed round (after the limiter) as a WAV")
    ap.add_argument("--json", help="write the measurements as JSON")
    ap.add_argument("--rate", type=int, default=22050, help="render rate (default 22050)")
    ap.add_argument("--tail", type=float, default=1.5, help="seconds rendered after the last event")
    ap.add_argument("--fx-loop", default="", help="comma-separated loops that are effects, not music (a ticking clock, an engine, rain)")
    args = ap.parse_args(argv)
    ev = parse(args.log)
    if not ev:
        sys.exit("no events")
    names = sorted({e[1] for e in ev})
    assets = {}
    for s in names:
        for ext in (".ogg", ".wav"):
            p = os.path.join(args.assets, s + ext)
            if os.path.exists(p):
                assets[s] = Asset(p)
                break
        else:
            sys.exit(f"no asset for sound {s!r} in {args.assets}")
    t0, t1 = ev[0][0], ev[-1][0] + args.tail
    rate = args.rate
    n = int(math.ceil(t1 * rate))
    voices, steals = schedule(ev, assets, t1)
    fx_loops = {x for x in args.fx_loop.split(",") if x}
    music = {v.sound for v in voices if v.loop and v.sound not in fx_loops}
    mix, mus_k, sfx_k = zeros(n), zeros(n), zeros(n)
    per: dict[str, object] = {}
    for s in names:
        own = zeros(n)
        for v in (v for v in voices if v.sound == s):
            i0, raw, kw = render(v, assets[s], rate, n)
            add(mix, i0, raw)
            add(own, i0, kw)
        add(mus_k if s in music else sfx_k, 0, own)
        per[s] = blocks(own, rate)
    if np is not None:  # master gain
        mix *= MASTER
    else:
        for i in range(n):
            mix[i] *= MASTER
    gr, peak_in = limiter(mix, rate)
    mb, sb = blocks(mus_k, rate), blocks(sfx_k, rate)
    b0 = int(t0 / BLOCK)
    mm, sm = windows(mb, 4)[b0:], windows(sb, 4)[b0:]
    mst = windows(mb, 30)[b0:]
    has_music = any(v > -70 for v in mm)
    span_min = max(1e-3, (ev[-1][0] - t0) / 60)
    flags: list[str] = []
    rep: dict = {"seconds": round(t1 - t0, 1), "voice_steals": steals, "peak_into_limiter_db": round(peak_in, 1)}

    gr_play = gr[b0:]
    rep["limiter"] = {"over_1db": round(sum(g > 1 for g in gr_play) / max(1, len(gr_play)), 3),
                      "over_3db": round(sum(g > 3 for g in gr_play) / max(1, len(gr_play)), 3), "max_db": round(max(gr_play, default=0), 1)}
    if rep["limiter"]["over_3db"] > 0.05:
        flags.append(f"master limiter pulls >3 dB {100 * rep['limiter']['over_3db']:.0f}% of the time: everything (music too) pumps; turn effects down")
    if steals:
        flags.append(f"{steals} voice(s) stolen at the 32-voice limit: cap simultaneous voices per sound")

    if has_music:
        act = [i for i, v in enumerate(mm) if v > -70]
        margin = [mm[i] - sm[i] for i in act]
        ref = pct([mst[i] for i in act], 0.5)
        rep["music"] = {"sounds": sorted(music), "short_term_lufs_median": round(ref, 1), "p10": round(pct([mst[i] for i in act], 0.1), 1),
                        "over_sfx_db_median": round(pct(margin, 0.5), 1), "over_sfx_db_p10": round(pct(margin, 0.1), 1),
                        "buried_pct": round(100 * sum(m < 0 for m in margin) / len(margin), 1),
                        "ducked_pct": round(100 * sum(mm[i] < ref - 4 for i in act) / len(act), 1)}
        if rep["music"]["over_sfx_db_median"] < RULE_SFX_BUS:
            flags.append(f"music is under the summed effects most of the time (median {rep['music']['over_sfx_db_median']:+.1f} dB; want >= {RULE_SFX_BUS:+.0f})")
        if rep["music"]["over_sfx_db_p10"] < RULE_SFX_P10:
            flags.append(f"music is buried by effects in 10% of the round (p10 {rep['music']['over_sfx_db_p10']:+.1f} dB; want >= {RULE_SFX_P10:+.0f})")
    else:
        flags.append("no music (no loop events): levels below are relative to -23 LUFS")
        ref = -23.0
    # Effects are judged against the music's typical level (ducking under a
    # big moment is allowed and shouldn't make that moment look louder).

    sfx_total = sum(sb) or 1e-12
    mus_now = windows(mb, 4)
    if has_music:
        main = max(music, key=lambda s: sum(per[s]))
        # The key of everything musical, weighted by how loud each loop plays.
        mch = [0.0] * 12
        for s in music:
            v = next(v for v in voices if v.sound == s)
            d, ch = round(12 * math.log2(max(v.segs[0][2], 1e-3))), assets[s].ch
            w = sum(per[s]) / (sum(ch) or 1e-12)
            for i in range(12):
                mch[(i + d) % 12] += w * ch[i]
        ks = keys_of(mch)
        (kr, km, kc), (kr2, km2, kc2) = ks[:2]
        # A chromatic track can fit two scales about equally: then a note is
        # in key if either has it.
        scale, key_names = [], []
        for r, mo, f in ks:
            sc = {(r + d) % 12 for d in SCALE[mo]}
            if f >= kc - 0.02 and sc not in scale and len(scale) < 2:
                scale.append(sc)
                key_names.append(f"{NAMES[r]} {mo}")
        guess = key_names[0] + (f" or {key_names[1]}" if len(key_names) > 1 else "")
        rep["key"] = {"guess": guess, "confidence": round(kc, 2), "runner_up": f"{NAMES[kr2]} {km2} ({kc2:.2f})", "from": "+".join(sorted(music))}
        base = in_scale(assets[main].ch, scale)
        for s in sorted(music - {main}):
            v = next(v for v in voices if v.sound == s)
            semis = round(12 * math.log2(max(v.segs[0][2], 1e-3)))
            if keys_of(assets[s].ch)[0][2] < 0.75:
                continue  # unpitched (a ticking clock, wind): no key to clash
            share = in_scale(assets[s].ch, scale, semis)
            rep.setdefault("layers", {})[s] = {"in_key": round(share, 2), "main_in_key": round(base, 2), "semitones": semis}
            best = max(range(-6, 6), key=lambda d: in_scale(assets[s].ch, scale, d))
            if share < 0.7 or share < in_scale(assets[s].ch, scale, best) - 0.08:
                flags.append(f"{s}: music layer clashes with {main} ({100 * share:.0f}% of it in key at {semis:+d} semitones vs {100 * base:.0f}%; {best:+d} fits best)")
    rows = []
    for s in names:
        if s in music:
            continue
        vs = [v for v in voices if v.sound == s]
        a = assets[s]
        st = windows(per[s], 30)
        mo = windows(per[s], 4)
        on = [i for i in range(b0, len(st)) if st[i] > -70]
        rel_st = pct([st[i] - ref for i in on], 0.9) if on else float("nan")
        rel_hit = max((mo[i] - ref for i in on), default=float("nan"))
        gains = [db(v.segs[0][1]) for v in vs]
        pitches = [v.segs[0][2] for v in vs]
        g_mean = sum(gains) / len(gains)
        g_sd = math.sqrt(sum((g - g_mean) ** 2 for g in gains) / len(gains))
        lp = [math.log2(p) for p in pitches]
        p_sd = 100 * (2 ** math.sqrt(sum((x - sum(lp) / len(lp)) ** 2 for x in lp) / len(lp)) - 1)
        spans = sorted((v.start, min(v.end, v.start + a.audible / max(v.segs[0][2], 1e-3))) for v in vs)
        overlap, ends = 0, []
        for st0, en in spans:
            ends = [e for e in ends if e > st0] + [en]
            overlap = max(overlap, len(ends))
        rate_min = len(vs) / span_min
        cls = "loop" if s in fx_loops else "frequent" if rate_min >= FREQUENT else "rare" if rate_min < RARE else "regular"
        med_p = sorted(pitches)[len(pitches) // 2]
        row = {"sound": s, "class": cls, "plays": len(vs), "per_min": round(rate_min, 1), "share_pct": round(100 * sum(per[s]) / sfx_total, 1),
               "st_vs_music_db": round(rel_st, 1), "hit_vs_music_db": round(rel_hit, 1), "vol_db_spread": round(g_sd, 2), "pitch_spread_pct": round(p_sd, 1),
               "pitch_range": [round(min(pitches), 3), round(max(pitches), 3)], "overlap": overlap, "bright_hz": round(a.bright * med_p),
               "attack_ms": round(a.attack), "stolen": sum(v.stolen for v in vs)}
        notes = []
        if cls == "frequent":
            if rel_st > RULE_FREQUENT:
                notes.append(f"too loud for how often it plays ({rel_st:+.1f} dB vs music; want <= {RULE_FREQUENT:+.0f})")
            if rel_hit > RULE_FREQ_HIT:
                notes.append(f"its loudest moment is {rel_hit:+.1f} dB over the music")
            if row["share_pct"] > 100 * RULE_SHARE and len(names) - len(music) > 2:
                notes.append(f"dominates the effects ({row['share_pct']:.0f}% of their energy)")
            if row["bright_hz"] > RULE_BRIGHT:
                notes.append(f"bright for a frequent sound ({row['bright_hz']} Hz; want <= {RULE_BRIGHT:.0f}: lowpass or a softer take)")
            if a.attack < 2 and row["bright_hz"] > 2500:
                notes.append("hard, bright attack")
            if g_sd < 0.4 and p_sd < 1.0:
                notes.append("no variation: randomize pitch ±3-5% and vol ±1-2 dB")
            if overlap > RULE_OVERLAP:
                notes.append(f"{overlap} voices at once: cap it (steal the oldest)")
        elif cls in ("regular", "loop"):
            if rel_st > RULE_REGULAR:
                notes.append(f"loud for a sound heard every few seconds ({rel_st:+.1f} dB vs music; want <= {RULE_REGULAR:+.0f})")
            if rel_hit > RULE_REG_HIT:
                notes.append(f"its loudest moment is {rel_hit:+.1f} dB over the music (want <= {RULE_REG_HIT:+.0f})")
            if overlap > RULE_OVERLAP + 2 and cls == "regular":
                notes.append(f"{overlap} voices at once: cap it")
        elif rel_hit > RULE_RARE_HIT:
            notes.append(f"very loud ({rel_hit:+.1f} dB over the music): turn it down or duck the music under it")
        if a.bright * med_p > RULE_HARSH:
            notes.append(f"harsh ({row['bright_hz']} Hz)")
        if max(pitches) > 1.6 or min(pitches) < 0.6:
            notes.append(f"pitched {min(pitches):.2f}-{max(pitches):.2f}x: samples sound chipmunked/sludgy past ~±7 semitones")
        # Key only matters while the music is heard.
        with_music = [v.segs[0][2] for v in vs if mus_now[min(len(mus_now) - 1, int(v.start / BLOCK))] > ref - 20]
        if has_music and a.tonal and with_music:
            pitches = with_music
            offs = sorted({round(12 * math.log2(p)) for p in pitches})
            row["notes_played"] = [NAMES[(a.pc + d) % 12] for d in offs]
            bad = [(d, in_scale(a.ch, scale, d)) for d in offs if in_scale(a.ch, scale, d) < 0.7]
            if bad:
                notes.append(f"tonal (mostly {NAMES[a.pc]}) and clashes with {rep['key']['guess']} at " + ", ".join(f"{d:+d} st ({100 * k:.0f}% in key)" for d, k in bad))
            detuned = sum(abs(12 * math.log2(p) - round(12 * math.log2(p))) > 0.3 for p in pitches)
            if detuned > len(pitches) * 0.25:
                notes.append(f"tonal and {detuned} of {len(pitches)} plays fall between semitones: jitter tonal sounds ≤1% or in scale steps")
        row["notes"] = notes
        flags += [f"{s}: {x}" for x in notes]
        rows.append(row)
    rep["sounds"] = rows
    rep["flags"] = flags

    print(f"{rep['seconds']} s, {len(voices)} voices, peak into limiter {peak_in:+.1f} dBFS, limiter >1 dB {100 * rep['limiter']['over_1db']:.0f}% / >3 dB {100 * rep['limiter']['over_3db']:.0f}% (max {rep['limiter']['max_db']} dB)")
    if has_music:
        m = rep["music"]
        k = rep["key"]
        print(f"music {'+'.join(m['sounds'])}: {m['short_term_lufs_median']} LUFS short-term (p10 {m['p10']}), ducked {m['ducked_pct']}% of the time")
        print(f"key of {k['from']} ~{k['guess']} ({k['confidence']}; next {k['runner_up']})" + "".join(f"; {s} {100 * l['in_key']:.0f}% in key at {l['semitones']:+d} st" for s, l in rep.get("layers", {}).items()))
        print(f"music over all effects: median {m['over_sfx_db_median']:+.1f} dB, p10 {m['over_sfx_db_p10']:+.1f} dB, effects louder {m['buried_pct']}% of the time")
    print(f"\n{'sound':12} {'class':8} {'/min':>6} {'share':>6} {'st/mus':>7} {'hit/mus':>7} {'±vol':>5} {'±pitch':>6} {'ovl':>3} {'bright':>6} {'atk':>4}")
    for r in sorted(rows, key=lambda r: -r["share_pct"]):
        print(f"{r['sound'][:12]:12} {r['class']:8} {r['per_min']:6.1f} {r['share_pct']:5.1f}% {r['st_vs_music_db']:+7.1f} {r['hit_vs_music_db']:+7.1f} "
              f"{r['vol_db_spread']:5.1f} {r['pitch_spread_pct']:5.1f}% {r['overlap']:3d} {r['bright_hz']:6d} {r['attack_ms']:4d}"
              + (f"  notes {' '.join(r['notes_played'])}" if r.get("notes_played") else ""))
    print("\nst/mus: the sound's own 3 s loudness (p90 while it plays) minus the music's; hit/mus: its loudest 400 ms minus the music's;")
    print("±vol dB and ±pitch % are the spread across plays; ovl: most voices of it at once; bright: rms frequency (Hz); atk: attack ms.")
    print("\n" + ("\n".join("FLAG " + f for f in flags) if flags else "mix looks sane"))
    if args.json:
        with open(args.json, "w") as f:
            json.dump(rep, f, indent=1)
    if args.out:
        h = int(rate * BLOCK)
        with wave.open(args.out, "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(rate)
            out = array.array("h")
            for i in range(n):
                g = 10 ** (-gr[min(i // h, len(gr) - 1)] / 20) if gr else 1.0
                out.append(int(max(-1.0, min(1.0, float(mix[i]) * g)) * 32767))
            w.writeframes(out.tobytes())
    return 1 if flags else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
