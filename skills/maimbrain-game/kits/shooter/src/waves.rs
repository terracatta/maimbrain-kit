//! Formations and the wave schedule: which enemies come when, and in what
//! shape. Pure data and math (no host calls), driven by the sim's wave `Rng`
//! only, so a daily seed gives everyone the same sequence of waves.
//!
//! To add a formation: add a variant to `Formation`, build its spawns in
//! `build`, and put it in `pool` from the phase it should appear.

use maimbrain::Rng;

use crate::sim::{Kind, Path, H, W};

/// One enemy to spawn `delay` seconds after its wave starts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spawn {
    pub delay: f32,
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    pub path: Path,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Formation {
    /// A row of jellies drifting down.
    Row,
    /// A V of jellies, point first.
    Vee,
    /// Two columns of jellies, one on each side.
    Columns,
    /// Jellies scattered across the sky, one after another.
    Scatter,
    /// A train of swoopers crossing from one side, bobbing.
    SwoopTrain,
    /// Two swooper trains crossing from both sides at once.
    SwoopCross,
    /// Darters that stop, aim at the ship and dash.
    Darters,
    /// One big bulb that hovers and fires fans.
    Bulb,
    /// Two bulbs, one each side.
    BulbPair,
    /// A spinner that hovers and sprays a spiral.
    Spinner,
    /// Jellies in a row with a bulb behind them.
    Escort,
}

impl Formation {
    /// The "two nasty surprises in a row" rule (engagement.md §4) avoids
    /// following one of these with another.
    pub fn nasty(self) -> bool {
        matches!(self, Formation::Darters | Formation::BulbPair | Formation::Spinner | Formation::SwoopCross)
    }
}

/// The opening, the same every round: easy jellies right where an idle
/// ship's shots go, so the first kill lands within about two seconds.
pub const OPENING: [(f32, Formation); 3] = [(0.4, Formation::Row), (3.6, Formation::Vee), (6.8, Formation::Columns)];

/// When each new enemy type joins (seconds into a loop) with a wave of its
/// own and a banner: something new every ~10–15 s until the boss.
pub const INTROS: [(u32, f32, Formation, Kind); 4] = [
    (0, 13.0, Formation::SwoopTrain, Kind::Swooper),
    (0, 24.0, Formation::Darters, Kind::Darter),
    (0, 35.0, Formation::Bulb, Kind::Bulb),
    (1, 10.0, Formation::Spinner, Kind::Spinner),
];

/// The formations that can come up `phase` seconds into loop `loop_n`.
pub fn pool(loop_n: u32, phase: f32) -> Vec<Formation> {
    use Formation::*;
    let mut p = vec![Row, Vee, Columns, Scatter];
    let after = |l: u32, t: f32| loop_n > l || (loop_n == l && phase >= t);
    if after(0, 13.0) {
        p.extend([SwoopTrain, SwoopTrain]);
    }
    if after(0, 20.0) {
        p.push(SwoopCross);
    }
    if after(0, 24.0) {
        p.push(Darters);
    }
    if after(0, 35.0) {
        p.extend([Bulb, Escort]);
    }
    if loop_n >= 1 {
        p.push(BulbPair);
    }
    if after(1, 10.0) {
        p.push(Spinner);
    }
    p
}

/// Spawns for one wave. `rng` is the wave generator (never the gameplay
/// one), so the same seed lays out the same waves whatever the player does.
pub fn build(f: Formation, rng: &mut Rng) -> Vec<Spawn> {
    use Formation::*;
    let mut out = Vec::new();
    let jelly = |x: f32, delay: f32, vy: f32, sway: f32| Spawn { delay, kind: Kind::Jelly, x, y: -24.0, path: Path::Drift { vy, sway } };
    match f {
        Row => {
            let n = 5;
            let cx = rng.range(150.0, 210.0);
            for i in 0..n {
                let x = cx + (i as f32 - 2.0) * 52.0;
                out.push(jelly(x, i as f32 * 0.05, 62.0, 10.0));
            }
        }
        Vee => {
            let cx = rng.range(130.0, 230.0);
            for i in 0..7 {
                let k = i as f32 - 3.0;
                out.push(jelly(cx + k * 34.0, k.abs() * 0.22, 74.0, 0.0));
            }
        }
        Columns => {
            let inset = rng.range(60.0, 90.0);
            for i in 0..4 {
                out.push(jelly(inset, i as f32 * 0.45, 82.0, 14.0));
                out.push(jelly(W - inset, i as f32 * 0.45 + 0.2, 82.0, 14.0));
            }
        }
        Scatter => {
            for i in 0..7 {
                out.push(jelly(rng.range(36.0, W - 36.0), i as f32 * 0.32, rng.range(70.0, 100.0), rng.range(0.0, 24.0)));
            }
        }
        SwoopTrain | SwoopCross => {
            let sides: &[f32] = if f == SwoopCross { &[-1.0, 1.0] } else if rng.f32() < 0.5 { &[-1.0] } else { &[1.0] };
            for &side in sides {
                let y0 = rng.range(70.0, 150.0);
                for i in 0..5 {
                    let x = if side < 0.0 { -30.0 } else { W + 30.0 };
                    let path = Path::Swoop { vx: -side * 120.0, vy: 26.0, amp: 46.0, freq: 2.6 };
                    out.push(Spawn { delay: i as f32 * 0.32, kind: Kind::Swooper, x, y: y0, path });
                }
            }
        }
        Darters => {
            let n = 2 + (rng.f32() < 0.5) as usize;
            for i in 0..n {
                let x = (i as f32 + 0.5) * W / n as f32 + rng.range(-20.0, 20.0);
                out.push(Spawn { delay: i as f32 * 0.5, kind: Kind::Darter, x, y: -24.0, path: Path::Dive { stop_y: rng.range(90.0, 150.0) } });
            }
        }
        Bulb => {
            let x = rng.range(110.0, 250.0);
            out.push(Spawn { delay: 0.0, kind: Kind::Bulb, x, y: -40.0, path: Path::Hover { stop_y: rng.range(120.0, 170.0), hold: 6.0 } });
        }
        BulbPair => {
            for (i, x) in [95.0, W - 95.0].into_iter().enumerate() {
                out.push(Spawn { delay: i as f32 * 0.6, kind: Kind::Bulb, x, y: -40.0, path: Path::Hover { stop_y: 140.0, hold: 5.5 } });
            }
        }
        Spinner => {
            let x = rng.range(120.0, 240.0);
            out.push(Spawn { delay: 0.0, kind: Kind::Spinner, x, y: -36.0, path: Path::Hover { stop_y: 150.0, hold: 5.0 } });
        }
        Escort => {
            let x = rng.range(130.0, 230.0);
            out.push(Spawn { delay: 0.6, kind: Kind::Bulb, x, y: -40.0, path: Path::Hover { stop_y: 120.0, hold: 5.0 } });
            for i in 0..4 {
                out.push(jelly(x + (i as f32 - 1.5) * 50.0, 0.0, 70.0, 8.0));
            }
        }
    }
    // Keep every spawn on screen horizontally (formations near an edge).
    for s in &mut out {
        if !matches!(s.path, Path::Swoop { .. }) {
            s.x = s.x.clamp(26.0, W - 26.0);
        }
        debug_assert!(s.y < H);
    }
    out
}
