//! A thumb that plays like a person. The title card's attract mode flies
//! the ship with it, and the difficulty tests (`tests.rs`) measure rounds
//! with its three presets.
//!
//! How it models a player steering by relative drag:
//! - **Reaction delay**: it re-plans every 150–350 ms, from where bullets
//!   were that long ago (positions wound back by their velocity).
//! - **Anticipation**: skilled players lead bullets (extrapolate forward).
//! - **Awareness**: first-timers only notice bullets close to the ship.
//! - **Motor noise**: a slowly wandering thumb tremor, multiplied by the
//!   drag gain on the ship, and a thumb speed limit.
//! - **Overshoot, then caution**: a big new move goes past the goal; the
//!   next correction is damped.
//! - **Lapses**: now and then it doesn't re-plan at all (looking at the score).
//! - **Greed**: it lines up under enemies and chases gems and power-ups,
//!   sometimes into trouble.

use maimbrain::Rng;

use crate::sim::{BOSS_RX, BOSS_RY, DRAG_GAIN, DropKind, H, Kind, SHIP_R, SHIP_X, SHIP_Y, Sim, W};

#[derive(Clone, Copy, Debug)]
pub struct Skill {
    /// For the tests' report.
    #[cfg_attr(not(test), allow(dead_code))]
    pub name: &'static str,
    /// Reaction delay range (s): acts on what it saw this long ago, and re-plans this often.
    pub react: (f32, f32),
    /// How far ahead it extrapolates bullets (s); 0 = not at all.
    pub anticipate: f32,
    /// Thumb tremor (units of thumb travel); the ship moves `DRAG_GAIN`× that.
    pub tremor: f32,
    /// Fastest thumb movement (units/s).
    pub thumb_speed: f32,
    /// Bullets farther from the ship than this go unnoticed (units).
    pub awareness: f32,
    /// Clearance it tries to keep from bullets, beyond the hitboxes (units).
    pub margin: f32,
    /// Overshoot on a big new move (0.3 = 30 % past the goal).
    pub overshoot: f32,
    /// Chance per decision of not re-planning.
    pub lapse: f32,
    /// How much it chases pickups and lines up under enemies (0–1).
    pub greed: f32,
}

// The attract mode flies with GOOD; the tests measure all three.
#[cfg_attr(not(test), allow(dead_code))]
pub const FIRST_TIMER: Skill = Skill {
    name: "first-timer",
    react: (0.24, 0.38),
    anticipate: 0.0,
    tremor: 6.0,
    thumb_speed: 420.0,
    awareness: 75.0,
    margin: 2.0,
    overshoot: 0.35,
    lapse: 0.30,
    greed: 0.9,
};
#[cfg_attr(not(test), allow(dead_code))]
pub const DECENT: Skill = Skill {
    name: "decent",
    react: (0.19, 0.29),
    anticipate: 0.12,
    tremor: 4.0,
    thumb_speed: 650.0,
    awareness: 105.0,
    margin: 5.0,
    overshoot: 0.22,
    lapse: 0.14,
    greed: 0.6,
};
pub const GOOD: Skill = Skill {
    name: "good",
    react: (0.15, 0.22),
    anticipate: 0.25,
    tremor: 2.5,
    thumb_speed: 950.0,
    awareness: 160.0,
    margin: 9.0,
    overshoot: 0.1,
    lapse: 0.04,
    greed: 0.45,
};

pub struct Bot {
    pub skill: Skill,
    rng: Rng,
    /// Where the thumb is (None: not touching).
    finger: Option<(f32, f32)>,
    /// Where the thumb is heading.
    goal: (f32, f32),
    think: f32,
    /// The last planned ship position (to spot big new moves).
    last_plan: (f32, f32),
    cautious: bool,
    tremor: (f32, f32),
    tremor_goal: (f32, f32),
}

/// Candidate offsets from the ship the bot considers moving to.
const DX: [f32; 11] = [-130.0, -90.0, -60.0, -36.0, -18.0, 0.0, 18.0, 36.0, 60.0, 90.0, 130.0];
const DY: [f32; 7] = [-80.0, -50.0, -24.0, 0.0, 24.0, 50.0, 80.0];
/// Times (s) ahead at which it checks where bullets will be.
const LOOK: [f32; 5] = [0.06, 0.16, 0.28, 0.42, 0.6];
/// Where it likes to sit when nothing's happening.
const HOME_Y: f32 = 470.0;

impl Bot {
    pub fn new(skill: Skill, seed: u64) -> Bot {
        Bot {
            skill,
            rng: Rng::new(seed),
            finger: None,
            goal: (0.0, 0.0),
            think: 0.0,
            last_plan: (0.0, 0.0),
            cautious: false,
            tremor: (0.0, 0.0),
            tremor_goal: (0.0, 0.0),
        }
    }

    fn gauss(&mut self) -> f32 {
        (0..4).map(|_| self.rng.f32()).sum::<f32>() - 2.0
    }

    /// Plays one frame: moves the thumb and feeds `sim` the touch events.
    pub fn drive(&mut self, sim: &mut Sim, dt: f32) {
        if !sim.alive() {
            if self.finger.take().is_some() {
                sim.touch_up();
            }
            return;
        }
        let Some(mut f) = self.finger else {
            // Thumb down under the ship (wherever is comfortable).
            let f = (sim.ship.x + self.rng.range(-20.0, 20.0), (sim.ship.y + 110.0).min(H - 6.0));
            sim.touch_down(f.0, f.1);
            self.finger = Some(f);
            self.goal = f;
            self.last_plan = (sim.ship.x, sim.ship.y);
            return;
        };
        self.think -= dt;
        if self.think <= 0.0 {
            let delay = self.rng.range(self.skill.react.0, self.skill.react.1);
            self.think = delay;
            if self.rng.f32() >= self.skill.lapse {
                self.plan(sim, delay);
            }
            self.tremor_goal = (self.gauss() * self.skill.tremor, self.gauss() * self.skill.tremor);
        }
        // The thumb moves toward its goal no faster than it can.
        let (dx, dy) = (self.goal.0 - f.0, self.goal.1 - f.1);
        let d = (dx * dx + dy * dy).sqrt();
        let step = self.skill.thumb_speed * dt;
        if d > step {
            f.0 += dx / d * step;
            f.1 += dy / d * step;
        } else {
            f = self.goal;
        }
        let k = 1.0 - (-6.0 * dt).exp();
        self.tremor.0 += (self.tremor_goal.0 - self.tremor.0) * k;
        self.tremor.1 += (self.tremor_goal.1 - self.tremor.1) * k;
        self.finger = Some(f);
        let (tx, ty) = ((f.0 + self.tremor.0).clamp(0.0, W), (f.1 + self.tremor.1).clamp(0.0, H));
        sim.touch_move(tx, ty);
    }

    /// Picks where the ship should go, from what it saw `delay` ago.
    fn plan(&mut self, sim: &Sim, delay: f32) {
        let s = &sim.ship;
        let sk = self.skill;
        let wind = sk.anticipate - delay;
        // Threats it noticed: (x, y, vx, vy, radius).
        let mut threats: Vec<(f32, f32, f32, f32, f32)> = Vec::new();
        for b in &sim.bullets {
            let (x, y) = (b.x + b.vx * wind, b.y + b.vy * wind);
            if (x - s.x).abs() < sk.awareness * 1.2 && (y - s.y).abs() < sk.awareness * 1.6 && (x - s.x).hypot(y - s.y) < sk.awareness * 1.6 {
                threats.push((x, y, b.vx, b.vy, b.r));
            }
        }
        for e in &sim.enemies {
            let (x, y) = (e.x + e.vx * wind, e.y + e.vy * wind);
            if e.kind == Kind::Darter && e.stage >= 1 && e.stage < 3 {
                // A darter about to dash: its lane is dangerous (good players read the line).
                let (dx, dy) = (e.aim.0 - e.x, e.aim.1 - e.y);
                let d = dx.hypot(dy).max(1.0);
                let v = 460.0 * sim.heat().sqrt();
                if sk.anticipate > 0.05 || e.stage == 2 {
                    threats.push((x, y, dx / d * v, dy / d * v, e.kind.radius()));
                }
            } else if (x - s.x).hypot(y - s.y) < sk.awareness * 2.0 {
                threats.push((x, y, e.vx, e.vy, e.kind.radius() * 0.8));
            }
        }
        // What it wants: an enemy to shoot (the lowest one), pickups.
        let target = sim.enemies.iter().filter(|e| e.y > 0.0 && e.y < s.y - 60.0).max_by(|a, b| a.y.total_cmp(&b.y)).map(|e| e.x).or(sim.boss.as_ref().map(|b| b.x));
        let mut best = (f32::MAX, s.x, s.y);
        for &dy in &DY {
            for &dx in &DX {
                let (cx, cy) = ((s.x + dx).clamp(SHIP_X.0, SHIP_X.1), (s.y + dy).clamp(SHIP_Y.0, SHIP_Y.1));
                let travel = (cx - s.x).hypot(cy - s.y);
                // Time to get there, roughly (so far-off spots are judged later in the future).
                let eta = travel / (sk.thumb_speed * DRAG_GAIN);
                let mut cost = travel * 0.0025 + ((cy - HOME_Y) / 160.0).powi(2) * 0.4;
                for &(x, y, vx, vy, r) in &threats {
                    let clear = r + SHIP_R + sk.margin;
                    for &t in &LOOK {
                        // Between here and the candidate along the way, then at it.
                        let k = if eta > 0.0 { (t / eta).min(1.0) } else { 1.0 };
                        let (px, py) = (s.x + (cx - s.x) * k, s.y + (cy - s.y) * k);
                        let (bx, by) = (x + vx * t, y + vy * t);
                        let d = (bx - px).hypot(by - py);
                        if d < clear * 2.2 {
                            let w = 1.0 - t * 0.9;
                            cost += ((clear * 2.2 - d) / clear).powi(2) * 4.0 * w;
                        }
                    }
                }
                if let Some(b) = &sim.boss {
                    let (ex, ey) = ((cx - b.x) / (BOSS_RX + 40.0), (cy - b.y) / (BOSS_RY + 60.0));
                    if ex * ex + ey * ey < 1.0 {
                        cost += 6.0;
                    }
                }
                if let Some(tx) = target {
                    cost -= sk.greed * 0.8 * (-(cx - tx).abs() / 40.0).exp();
                }
                for d in &sim.drops {
                    let want = match d.kind {
                        DropKind::Gem => 0.15,
                        DropKind::Power(_) | DropKind::Heart => 1.2,
                    };
                    let dist = (d.x - cx).hypot(d.y + 30.0 - cy);
                    cost -= sk.greed * want * (-dist / 45.0).exp();
                }
                if cost < best.0 {
                    best = (cost, cx, cy);
                }
            }
        }
        let (_, gx, gy) = best;
        // A big new move overshoots; the correction after it is cautious.
        let jump = (gx - self.last_plan.0).hypot(gy - self.last_plan.1);
        let (mut ox, mut oy) = (gx, gy);
        if jump > 45.0 && !self.cautious {
            ox = s.x + (gx - s.x) * (1.0 + sk.overshoot);
            oy = s.y + (gy - s.y) * (1.0 + sk.overshoot);
            self.cautious = true;
        } else if self.cautious {
            ox = s.x + (gx - s.x) * 0.8;
            oy = s.y + (gy - s.y) * 0.8;
            self.cautious = false;
        }
        self.last_plan = (gx, gy);
        if let Some(f) = sim.finger_for(ox, oy) {
            self.goal = (f.0.clamp(0.0, W), f.1.clamp(0.0, H));
        }
    }
}
