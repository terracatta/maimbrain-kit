//! The rules, with no host calls: `cargo test -p kit_racer` runs them natively.
//!
//! A floating frame, as in games/mistwood: the player's car stays at z = 0
//! facing −z and the road moves past it. Something `d` meters along the road
//! is at world z = −(d − dist); lane `l` (0, 1, 2) is at x = (l − 1) × LANE_W.
//! The camera lives here too, so its shake comes from the sim's seeded `Rng`.

use maimbrain::Rng;
use maimbrain::gfx3d::{Camera, Quat, Vec3, look_at, vec3};

use crate::feel::{self, Feel};

pub const W: f32 = 360.0;
pub const H: f32 = 640.0;

// ---- Tuning knobs -----------------------------------------------------
/// Speed at the start, m/s (16 m/s ≈ 58 km/h). Raise it and the first seconds get hard.
pub const START_SPEED: f32 = 16.0;
/// Speed added at each speed-up, m/s.
pub const SPEED_STEP: f32 = 2.2;
/// Seconds between speed-ups (engagement.md: something gets faster every 10–15 s).
pub const SPEED_EVERY: f32 = 12.0;
/// Top cruising speed, m/s (boost goes past it).
pub const MAX_SPEED: f32 = 44.0;
/// Traffic's own forward speed, m/s. You're faster, so you catch up with it.
pub const TRAFFIC_SPEED: f32 = 7.0;
/// Seconds of clear road between rows of obstacles at the start, and the
/// floor it shrinks to. Lower = denser traffic.
pub const GAP_START: f32 = 1.5;
pub const GAP_MIN: f32 = 0.66;
/// How much the gap shrinks per second of play.
pub const GAP_SHRINK: f32 = 0.011;
/// The lane change spring: stiffness (1/s²) and damping ratio (< 1 overshoots,
/// which is the springy feel; 1 is a plain slide).
pub const LANE_STIFFNESS: f32 = 170.0;
pub const LANE_DAMPING: f32 = 0.62;
/// A boost pad: seconds of boost, and the speed multiplier. Boosting smashes through everything.
pub const BOOST_TIME: f32 = 2.6;
pub const BOOST_MULT: f32 = 1.45;
/// A dodge is a near miss if you were still in the way this many seconds before impact.
pub const NEAR_TTC: f32 = 0.55;
/// Points per coin, per near miss (× the near-miss chain) and per smash; distance scores 1 per meter.
pub const COIN_POINTS: u32 = 10;
pub const NEAR_POINTS: u32 = 50;
pub const SMASH_POINTS: u32 = 100;
/// Seconds into the round when each new thing first shows up (something new every ~15 s).
pub const UNLOCK_WALLS: f32 = 9.0;
pub const UNLOCK_BOOST: f32 = 11.0;
pub const UNLOCK_TRUCKS: f32 = 17.0;
pub const UNLOCK_CHANGERS: f32 = 31.0;
pub const UNLOCK_CONES: f32 = 46.0;
pub const UNLOCK_OIL: f32 = 62.0;
/// No obstacle reaches you before this many seconds (the warm-up can't be failed).
pub const SAFE_START: f32 = 3.5;
/// After this many seconds, nasty rows (walls, lane-changers, roadworks) can come back to back.
pub const RELENTLESS: f32 = 90.0;
// -----------------------------------------------------------------------

pub const LANES: i32 = 3;
pub const LANE_W: f32 = 2.4;
/// The player's car: half width and half length (m).
pub const CAR_HALF: (f32, f32) = (0.8, 1.9);
/// Rows are laid out this far ahead (the fog hides the rest).
pub const AHEAD: f32 = 150.0;
/// A lane-changing truck starts blinking this many seconds before you'd reach it,
/// waits `BLINK_LEAD`, then slides over in `SHIFT_TIME`.
pub const BLINK_TTC: f32 = 2.1;
pub const BLINK_LEAD: f32 = 0.85;
pub const SHIFT_TIME: f32 = 0.6;
/// Oil: how long steering is gone after the slide.
pub const OIL_LOCK: f32 = 0.45;
/// The crash: real seconds of slow motion, its rate, and the whole crash before the card.
pub const SLOWMO_TIME: f32 = 0.9;
pub const SLOWMO: f32 = 0.25;
pub const CRASH_TIME: f32 = 1.8;
/// A car or truck in the next lane gets its pass-by whoosh this many seconds
/// before it's alongside (the sound peaks about then), if it's within `PASS_NEAR` m sideways.
pub const PASS_LEAD: f32 = 0.45;
pub const PASS_NEAR: f32 = 4.0;
/// Near misses within this many seconds of each other chain (× 2, × 3 …).
pub const CHAIN_WINDOW: f32 = 3.0;
/// Seconds a person needs to start a lane change, and to make one: the
/// generator only lays out rows a player can get through.
const REACT: f32 = 0.33;
const LANE_T: f32 = 0.3;

/// Where lane `l` is.
pub fn lane_x(l: i32) -> f32 {
    (l - 1) as f32 * LANE_W
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Car,
    Truck,
    /// A closed lane: a barrier and cones.
    Cones,
    Oil,
}

impl Kind {
    /// Half width and half length (m).
    pub fn half(self) -> (f32, f32) {
        match self {
            Kind::Car => (0.85, 2.0),
            Kind::Truck => (1.0, 4.2),
            Kind::Cones => (0.95, 7.0),
            Kind::Oil => (0.9, 1.6),
        }
    }
    pub fn moves(self) -> bool {
        matches!(self, Kind::Car | Kind::Truck)
    }
}

/// A smashed or crashed-into obstacle flying off: offsets from where it was hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Knock {
    pub t: f32,
    pub vel: Vec3,
    pub off: Vec3,
    pub spin: f32,
    pub angle: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub kind: Kind,
    /// Road position of its center (m).
    pub d: f32,
    /// Lateral position (m).
    pub x: f32,
    /// The lane it's in (or heading for).
    pub lane: i32,
    pub from_lane: i32,
    /// Own forward speed, m/s.
    pub v: f32,
    /// Index into the look's tint table.
    pub tint: u32,
    /// A planned lane change (trucks): the lane it will move to.
    pub change: Option<i32>,
    /// Seconds since its blinker came on.
    pub blink: Option<f32>,
    /// It was in your way close enough to count a dodge as a near miss.
    pub near: bool,
    pub passed: bool,
    /// Its pass-by whoosh has been cued.
    pub whooshed: bool,
    pub knock: Option<Knock>,
}

impl Obstacle {
    /// Lane-change progress, 0…1.
    pub fn shift(&self) -> f32 {
        self.blink.map_or(0.0, |b| ((b - BLINK_LEAD) / SHIFT_TIME).clamp(0.0, 1.0))
    }
    /// The blinker is on and it hasn't finished moving over.
    pub fn signalling(&self) -> bool {
        self.change.is_some() && self.shift() < 1.0
    }
    /// Lanes it covers now (both while sliding over).
    pub fn covers(&self, lane: i32) -> bool {
        (self.x - lane_x(lane)).abs() < self.kind.half().0 + CAR_HALF.0 - 0.2
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coin {
    pub d: f32,
    pub lane: i32,
    pub taken: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pad {
    pub d: f32,
    pub lane: i32,
    pub used: bool,
}

/// Something that happened this step, for sound, haptics and juice (the host
/// turns these into output; the sim never draws or plays).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    /// A lane change (dir −1 left, +1 right); `quick` when it reversed a change in progress.
    Lane { dir: i32, quick: bool },
    /// Steered into the edge of the road.
    Bump { dir: i32 },
    /// `n`: coins in a row (pitch climbs a scale).
    Coin { n: u32, at: Vec3 },
    /// `chain`: near misses in a row; `points` scored.
    NearMiss { chain: u32, points: u32, at: Vec3 },
    Boost { at: Vec3 },
    Smash { at: Vec3 },
    Oil { dir: i32 },
    /// A car or truck is about to go past close by: `side` is its offset
    /// from you (m, + = right), `closing` the speed you pass it at (m/s).
    Pass { side: f32, closing: f32 },
    /// A truck's blinker came on.
    Blinker,
    SpeedUp { tier: u32 },
    /// Something new joins the road.
    New(Kind),
    Crash { at: Vec3 },
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Running,
    /// Real seconds since the hit, and world seconds (slowed).
    Crashing { t: f32, world: f32 },
    Over { t: f32 },
}

/// The car's pose: lateral position, lift, and rotations (rad).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub lift: f32,
    pub yaw: f32,
    pub roll: f32,
    pub pitch: f32,
}

pub struct Sim {
    /// Seconds of play.
    pub t: f32,
    /// Meters driven.
    pub dist: f32,
    /// Cruising speed (m/s), without boost.
    pub speed: f32,
    pub tier: u32,
    /// The lane the car is steering for, and where it is now.
    pub lane: i32,
    pub x: f32,
    pub vx: f32,
    /// Seconds of boost left, and how boosted it looks (0…1, eased).
    pub boost: f32,
    pub boost_k: f32,
    /// Seconds of no steering left after oil.
    pub oil: f32,
    pub coins: u32,
    pub coin_run: u32,
    pub near_chain: u32,
    pub chain_t: f32,
    pub bonus: u32,
    pub phase: Phase,
    pub over: bool,
    pub obstacles: Vec<Obstacle>,
    pub coin_list: Vec<Coin>,
    pub pads: Vec<Pad>,
    pub cues: Vec<Cue>,
    /// Lane changes made (the hint goes once you've steered).
    pub steers: u32,
    /// Seconds since the last press (the hint comes back when you stall).
    pub idle: f32,
    /// How fast it feels (camera, effects, sound; never the rules): `feel.rs`.
    pub feel: Feel,
    /// Camera shake ("trauma", 0…1) and this frame's offset.
    pub shake: f32,
    pub jitter: Vec3,
    // The crash.
    crash_x: f32,
    crash_dir: f32,
    crash_speed: f32,
    // Steering: the lane before this touch's press (a swipe re-aims from it).
    touch_from: i32,
    // The generator.
    /// Where (in meeting space) you're clear of the last row laid out.
    clear_at: f32,
    prev_free: [bool; 3],
    last_static: [f32; 3],
    nasty: bool,
    announced: u32,
    rng: Rng,
    shake_rng: Rng,
}

/// How far ahead (seconds) the attract mode plans its lane.
const AUTOPILOT_LOOK: f32 = 1.4;

/// Hazards in the order they unlock, with when.
const UNLOCKS: [(f32, Option<Kind>); 6] = [
    (UNLOCK_WALLS, None),
    (UNLOCK_BOOST, None),
    (UNLOCK_TRUCKS, Some(Kind::Truck)),
    (UNLOCK_CHANGERS, Some(Kind::Truck)),
    (UNLOCK_CONES, Some(Kind::Cones)),
    (UNLOCK_OIL, Some(Kind::Oil)),
];

fn smooth(x: f32) -> f32 {
    let k = x.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

impl Sim {
    pub fn new(seed: u64) -> Sim {
        let mut s = Sim {
            t: 0.0,
            dist: 0.0,
            speed: START_SPEED,
            tier: 0,
            lane: 1,
            x: 0.0,
            vx: 0.0,
            boost: 0.0,
            boost_k: 0.0,
            oil: 0.0,
            coins: 0,
            coin_run: 0,
            near_chain: 0,
            chain_t: 0.0,
            bonus: 0,
            phase: Phase::Running,
            over: false,
            obstacles: Vec::new(),
            coin_list: Vec::new(),
            pads: Vec::new(),
            cues: Vec::new(),
            steers: 0,
            idle: 0.0,
            feel: Feel::new(),
            shake: 0.0,
            jitter: Vec3::ZERO,
            crash_x: 0.0,
            crash_dir: 1.0,
            crash_speed: 0.0,
            touch_from: 1,
            // So that the first row is touched after SAFE_START.
            clear_at: START_SPEED * (SAFE_START - GAP_START),
            prev_free: [true; 3],
            last_static: [-1000.0; 3],
            nasty: false,
            announced: 0,
            rng: Rng::new(seed),
            shake_rng: Rng::new(seed ^ 0x5ca1_ab1e),
        };
        // The warm-up: a carpet of coins in every lane, so whichever way the
        // first tap goes, it pays (an early win in the first two seconds).
        for k in 0..7 {
            for lane in 0..LANES {
                s.coin_list.push(Coin { d: 9.0 + k as f32 * 4.5, lane, taken: false });
            }
        }
        s.spawn();
        s
    }

    /// Score: a point per meter plus coins, near misses and smashes.
    pub fn score(&self) -> u32 {
        self.dist as u32 + self.coins * COIN_POINTS + self.bonus
    }

    /// Current speed with boost, m/s.
    pub fn speed_now(&self) -> f32 {
        match self.phase {
            Phase::Running => self.speed * (1.0 + (BOOST_MULT - 1.0) * self.boost_k),
            Phase::Crashing { world, .. } => self.crash_speed * (-3.0 * world).exp(),
            Phase::Over { .. } => 0.0,
        }
    }

    /// Cruising speed at round time `t` (the speed-ups).
    pub fn cruise_at(t: f32) -> f32 {
        (START_SPEED + SPEED_STEP * (t / SPEED_EVERY).floor()).min(MAX_SPEED)
    }

    pub fn crashed(&self) -> bool {
        self.phase != Phase::Running
    }

    // --- steering ---

    /// A press on the left (−1) or right (+1) half of the screen.
    pub fn press(&mut self, dir: i32) {
        self.touch_from = self.lane;
        self.steer(dir);
    }

    /// A swipe during the same touch: aims from the lane the touch started in,
    /// so a swipe left that began on the right half still goes left.
    pub fn swipe(&mut self, dir: i32) {
        let want = (self.touch_from + dir).clamp(0, LANES - 1);
        if want != self.lane {
            let d = (want - self.lane).signum();
            self.lane = want;
            self.lane_cue(d);
        }
    }

    fn steer(&mut self, dir: i32) {
        self.idle = 0.0;
        if self.crashed() || self.oil > 0.0 {
            return;
        }
        let want = (self.lane + dir).clamp(0, LANES - 1);
        if want == self.lane {
            self.vx += dir as f32 * 2.5;
            self.cues.push(Cue::Bump { dir });
            return;
        }
        self.lane = want;
        self.lane_cue(dir);
    }

    fn lane_cue(&mut self, dir: i32) {
        self.steers += 1;
        let quick = self.vx * (dir as f32) < -2.0;
        self.cues.push(Cue::Lane { dir, quick });
    }

    // --- the step ---

    pub fn step(&mut self, dt: f32) {
        match self.phase {
            Phase::Running => self.run(dt),
            Phase::Crashing { t, world } => {
                let wdt = dt * if t < SLOWMO_TIME { SLOWMO } else { 1.0 };
                self.phase = Phase::Crashing { t: t + dt, world: world + wdt };
                self.dist += self.speed_now() * wdt;
                // Traffic brakes for the wreck.
                for o in &mut self.obstacles {
                    o.v *= (-2.5 * wdt).exp();
                }
                self.move_obstacles(wdt);
                if t + dt >= CRASH_TIME {
                    self.phase = Phase::Over { t: 0.0 };
                    self.over = true;
                    self.cues.push(Cue::Over);
                }
            }
            Phase::Over { t } => {
                self.phase = Phase::Over { t: t + dt };
                self.move_obstacles(dt);
            }
        }
        self.feel.step(self.speed_now(), self.boost_k, self.phase == Phase::Running, dt);
        self.shake = (self.shake - 1.6 * dt).max(0.0);
        let k = self.shake * self.shake;
        self.jitter = if k > 0.0 {
            let r = &mut self.shake_rng;
            vec3(r.range(-1.0, 1.0), r.range(-1.0, 1.0), r.range(-0.5, 0.5)) * (0.45 * k)
        } else {
            Vec3::ZERO
        };
    }

    fn run(&mut self, dt: f32) {
        self.t += dt;
        self.idle += dt;
        // Speed-ups, announced; the cruise eases up to the new speed.
        let tier = (self.t / SPEED_EVERY) as u32;
        if tier > self.tier && Sim::cruise_at(self.t) > self.speed + 0.1 {
            self.tier = tier;
            self.feel.kick(feel::KICK_SPEEDUP);
            self.cues.push(Cue::SpeedUp { tier });
        }
        let target = Sim::cruise_at(self.t);
        self.speed = (self.speed + 3.0 * dt).min(target).max(self.speed.min(target));
        while self.announced < UNLOCKS.len() as u32 && self.t >= UNLOCKS[self.announced as usize].0 {
            if let Some(k) = UNLOCKS[self.announced as usize].1 {
                self.cues.push(Cue::New(k));
            }
            self.announced += 1;
        }
        self.boost = (self.boost - dt).max(0.0);
        let want = if self.boost > 0.0 { 1.0 } else { 0.0 };
        self.boost_k += (want - self.boost_k) * (1.0 - (-dt * if want > self.boost_k { 9.0 } else { 3.0 }).exp());
        self.oil = (self.oil - dt).max(0.0);
        self.chain_t = (self.chain_t - dt).max(0.0);
        if self.chain_t <= 0.0 {
            self.near_chain = 0;
        }
        let p = self.speed_now();
        self.dist += p * dt;

        // The lane spring, in small steps so a long frame stays stable.
        let n = (dt * 240.0).ceil().max(1.0) as u32;
        let h = dt / n as f32;
        let damp = 2.0 * LANE_DAMPING * LANE_STIFFNESS.sqrt();
        for _ in 0..n {
            let a = LANE_STIFFNESS * (lane_x(self.lane) - self.x) - damp * self.vx;
            self.vx += a * h;
            self.x += self.vx * h;
        }

        self.move_obstacles(dt);
        self.collide(p);
        self.pickups();
        self.spawn();
        let behind = self.dist - 16.0;
        self.obstacles.retain(|o| o.d + o.kind.half().1 > behind || o.knock.is_some_and(|k| k.t < 2.5));
        self.coin_list.retain(|c| c.d > behind);
        self.pads.retain(|p| p.d > behind);
    }

    fn move_obstacles(&mut self, dt: f32) {
        let p = self.speed_now();
        let dist = self.dist;
        let mut blinked = false;
        for o in &mut self.obstacles {
            if let Some(k) = &mut o.knock {
                k.t += dt;
                k.vel.y -= 22.0 * dt;
                let drag = (-1.6 * dt).exp();
                k.vel.x *= drag;
                k.vel.z *= drag;
                k.off += k.vel * dt;
                if k.off.y < 0.0 {
                    k.off.y = 0.0;
                    k.vel.y = -k.vel.y * 0.35;
                    k.spin *= 0.6;
                }
                k.angle += k.spin * dt;
                continue;
            }
            o.d += o.v * dt;
            if let Some(to) = o.change {
                let gap = o.d - dist - o.kind.half().1 - CAR_HALF.1;
                let ttc = gap / (p - o.v).max(1.0);
                match &mut o.blink {
                    None if ttc < BLINK_TTC && ttc > 0.0 && p > o.v => {
                        o.blink = Some(0.0);
                        blinked = true;
                    }
                    Some(b) => *b += dt,
                    None => {}
                }
                let s = smooth(o.shift());
                o.x = lane_x(o.from_lane) + (lane_x(to) - lane_x(o.from_lane)) * s;
                if s >= 1.0 {
                    o.lane = to;
                }
            }
        }
        if blinked {
            self.cues.push(Cue::Blinker);
        }
    }

    fn collide(&mut self, p: f32) {
        let (cw, cl) = CAR_HALF;
        let mut crash: Option<usize> = None;
        for i in 0..self.obstacles.len() {
            let o = self.obstacles[i];
            if o.knock.is_some() || o.passed {
                continue;
            }
            let (ow, ol) = o.kind.half();
            let rel = o.d - self.dist;
            let lateral = (o.x - self.x).abs() < ow + cw - 0.22;
            let ttc = (rel - ol - cl) / (p - o.v).max(1.0);
            if o.kind.moves() && !o.whooshed && !lateral && (o.x - self.x).abs() < PASS_NEAR && rel > 0.0 && rel / (p - o.v).max(1.0) < PASS_LEAD {
                self.obstacles[i].whooshed = true;
                self.cues.push(Cue::Pass { side: o.x - self.x, closing: p - o.v });
            }
            // Still in the way just before impact: dodging now is a near miss.
            if lateral && ttc > 0.0 && ttc < NEAR_TTC && o.kind != Kind::Oil {
                self.obstacles[i].near = true;
            }
            if lateral && rel.abs() < ol + cl - 0.3 {
                match o.kind {
                    Kind::Oil if self.boost <= 0.0 => {
                        self.obstacles[i].passed = true;
                        if self.oil <= 0.0 {
                            let mut dir = if self.rng.f32() < 0.5 { -1 } else { 1 };
                            if !(0..LANES).contains(&(self.lane + dir)) {
                                dir = -dir;
                            }
                            self.lane += dir;
                            self.oil = OIL_LOCK;
                            self.vx += dir as f32 * 3.0;
                            self.cues.push(Cue::Oil { dir });
                        }
                    }
                    Kind::Oil => self.obstacles[i].passed = true,
                    _ if self.boost > 0.0 => {
                        let side = if o.x > self.x { 1.0 } else if o.x < self.x { -1.0 } else if self.rng.f32() < 0.5 { 1.0 } else { -1.0 };
                        let vel = vec3(side * self.rng.range(4.0, 7.0), self.rng.range(5.0, 8.0), -self.rng.range(4.0, 8.0));
                        let spin = side * self.rng.range(5.0, 9.0);
                        self.obstacles[i].knock = Some(Knock { t: 0.0, vel, off: Vec3::ZERO, spin, angle: 0.0 });
                        self.bonus += SMASH_POINTS;
                        self.shake = (self.shake + 0.35).min(1.0);
                        self.cues.push(Cue::Smash { at: vec3(o.x, 0.8, -(rel)) });
                    }
                    _ => {
                        crash = Some(i);
                        break;
                    }
                }
                continue;
            }
            // Passed it.
            if rel + ol < -cl {
                self.obstacles[i].passed = true;
                if o.near {
                    self.near_chain += 1;
                    self.chain_t = CHAIN_WINDOW;
                    let points = NEAR_POINTS * self.near_chain.min(5);
                    self.bonus += points;
                    self.shake = (self.shake + 0.18).min(1.0);
                    self.feel.kick(feel::KICK_NEAR);
                    self.cues.push(Cue::NearMiss { chain: self.near_chain, points, at: vec3(o.x, 1.2, -rel) });
                }
            }
        }
        if let Some(i) = crash {
            let o = self.obstacles[i];
            let rel = o.d - self.dist;
            // The wreck slides away from what it hit, toward clear road if it can
            // (the camera swings out that way too).
            let pref = if self.x > o.x + 0.05 {
                1.0
            } else if self.x < o.x - 0.05 {
                -1.0
            } else if self.rng.f32() < 0.5 {
                1.0
            } else {
                -1.0
            };
            let busy = |side: f32| self.obstacles.iter().any(|q| q.knock.is_none() && q.kind.moves() && (q.d - self.dist).abs() < 10.0 && (q.x - self.x) * side > 0.6);
            self.crash_dir = if busy(pref) && !busy(-pref) { -pref } else { pref };
            self.crash_x = self.x;
            self.crash_speed = p;
            // The other vehicle gets shoved on ahead and spun.
            let vel = vec3(-self.crash_dir * 3.0, 2.5, -(p - o.v) * 0.4);
            self.obstacles[i].knock = Some(Knock { t: 0.0, vel, off: Vec3::ZERO, spin: -self.crash_dir * 2.0, angle: 0.0 });
            self.phase = Phase::Crashing { t: 0.0, world: 0.0 };
            self.boost = 0.0;
            self.shake = 1.0;
            self.cues.push(Cue::Crash { at: vec3((self.x + o.x) / 2.0, 0.7, -(rel - o.kind.half().1).max(0.0)) });
        }
    }

    fn pickups(&mut self) {
        let mut got = 0;
        for c in &mut self.coin_list {
            if !c.taken && (c.d - self.dist).abs() < 1.7 && (lane_x(c.lane) - self.x).abs() < 1.15 {
                c.taken = true;
                got += 1;
            }
        }
        for _ in 0..got {
            self.coins += 1;
            self.coin_run += 1;
            self.cues.push(Cue::Coin { n: self.coin_run, at: vec3(self.x, 0.9, -1.0) });
        }
        // A run of coins ends when the next one in reach was missed.
        if got == 0 && self.coin_list.iter().any(|c| !c.taken && c.d < self.dist - 2.0 && c.d > self.dist - 3.0) {
            self.coin_run = 0;
        }
        for i in 0..self.pads.len() {
            let pd = self.pads[i];
            if !pd.used && (pd.d - self.dist).abs() < 2.0 && (lane_x(pd.lane) - self.x).abs() < 1.2 {
                self.pads[i].used = true;
                self.boost = BOOST_TIME;
                self.shake = (self.shake + 0.25).min(1.0);
                self.feel.kick(feel::KICK_BOOST);
                self.cues.push(Cue::Boost { at: vec3(self.x, 0.3, 0.0) });
            }
        }
    }

    // --- the road ahead ---

    /// Lays out rows of obstacles up to `AHEAD` meters ahead, in time: each
    /// row starts `gap` seconds after you're clear of the last one. Rows are
    /// placed by where you'll meet them: something moving at `v` starts back
    /// by `v × (time until you get there)`, so when you reach a row it is
    /// exactly the pattern designed (at a steady speed).
    fn spawn(&mut self) {
        let p = self.speed.max(1.0);
        while self.clear_at < self.dist + AHEAD {
            let t_clear = self.t + (self.clear_at - self.dist) / p;
            let gap = (GAP_START - GAP_SHRINK * t_clear).max(GAP_MIN);
            let pm = Sim::cruise_at(t_clear);
            let contact = self.clear_at + gap * pm;
            self.row(contact, t_clear + gap, gap, pm, p);
        }
    }

    /// How far you drive (meeting-space meters) from touching something of
    /// `kind` to being level with its middle: longer for traffic, which you
    /// only gain on at the difference in speed.
    fn reach(kind: Kind, pm: f32) -> f32 {
        let v = if kind.moves() { TRAFFIC_SPEED } else { 0.0 };
        (kind.half().1 + CAR_HALF.1) * pm / (pm - v).max(1.0)
    }

    fn row(&mut self, front: f32, t_meet: f32, gap: f32, pm: f32, p: f32) {
        let unlocked = |at: f32| t_meet >= at;
        // Pick a pattern: (weight, id). Nasty ones never come twice in a row,
        // until the late game (RELENTLESS), when they can.
        let calm = self.nasty && t_meet < RELENTLESS;
        let mut menu: Vec<(f32, u32)> = vec![(3.0, 0), (0.5, 7)];
        if unlocked(UNLOCK_WALLS) && !calm {
            menu.push((2.2, 1));
        }
        if unlocked(UNLOCK_TRUCKS) {
            menu.push((2.0, 2));
            if !calm {
                menu.push((1.0, 3));
            }
        }
        if unlocked(UNLOCK_CHANGERS) && !calm {
            menu.push((1.6, 4));
        }
        if unlocked(UNLOCK_CONES) && !calm {
            menu.push((1.3, 5));
        }
        if unlocked(UNLOCK_OIL) {
            menu.push((1.1, 6));
        }
        for _attempt in 0..10 {
            let total: f32 = menu.iter().map(|m| m.0).sum();
            let mut pick = self.rng.range(0.0, total);
            let mut id = 0;
            for &(w, i) in &menu {
                if pick < w {
                    id = i;
                    break;
                }
                pick -= w;
            }
            if let Some(plan) = self.plan_row(id, front, gap) {
                self.place(plan, front, t_meet, pm, p);
                return;
            }
        }
        // Fallback: one car in a lane that's always fair.
        let lane = (self.rng.next_u32() % 3) as i32;
        let plan = RowPlan { items: vec![(Kind::Car, lane, None)], blocked: lane_bit(lane), nasty: false };
        self.place(plan, front, t_meet, pm, p);
    }

    /// A row's obstacles for pattern `id`, or None if it isn't fair or legal here.
    fn plan_row(&mut self, id: u32, front: f32, gap: f32) -> Option<RowPlan> {
        let r = |s: &mut Sim| (s.rng.next_u32() % 3) as i32;
        let other = |s: &mut Sim, a: i32| (a + 1 + (s.rng.next_u32() % 2) as i32) % 3;
        let plan = match id {
            0 => {
                let a = r(self);
                RowPlan { items: vec![(Kind::Car, a, None)], blocked: lane_bit(a), nasty: false }
            }
            1 => {
                let a = r(self);
                let b = other(self, a);
                RowPlan { items: vec![(Kind::Car, a, None), (Kind::Car, b, None)], blocked: lane_bit(a) | lane_bit(b), nasty: true }
            }
            2 => {
                let a = r(self);
                RowPlan { items: vec![(Kind::Truck, a, None)], blocked: lane_bit(a), nasty: false }
            }
            3 => {
                let a = r(self);
                let b = other(self, a);
                RowPlan { items: vec![(Kind::Truck, a, None), (Kind::Car, b, None)], blocked: lane_bit(a) | lane_bit(b), nasty: true }
            }
            4 => {
                // A truck that moves over (blinker first) into a free lane.
                let a = r(self);
                let b = if a == 1 { if self.rng.f32() < 0.5 { 0 } else { 2 } } else { 1 };
                RowPlan { items: vec![(Kind::Truck, a, Some(b))], blocked: lane_bit(a) | lane_bit(b), nasty: true }
            }
            5 => {
                let a = r(self);
                RowPlan { items: vec![(Kind::Cones, a, None)], blocked: lane_bit(a), nasty: true }
            }
            6 => {
                let a = r(self);
                let mut items = vec![(Kind::Oil, a, None)];
                let mut blocked = lane_bit(a);
                if self.rng.f32() < 0.5 {
                    let b = other(self, a);
                    items.push((Kind::Car, b, None));
                    blocked |= lane_bit(b);
                }
                RowPlan { items, blocked, nasty: false }
            }
            _ => RowPlan { items: Vec::new(), blocked: 0, nasty: false },
        };
        // Moving things can't start behind something static in their lane
        // (they'd drive through it before you got there).
        let p = self.speed.max(1.0);
        for &(k, lane, to) in &plan.items {
            if k.moves() {
                let clear = AHEAD * TRAFFIC_SPEED / p + 12.0;
                if front - self.last_static[lane as usize] < clear {
                    return None;
                }
                if let Some(b) = to
                    && front - self.last_static[b as usize] < clear + 15.0
                {
                    return None;
                }
            }
        }
        // Fair: from every lane that was free in the last row, a free lane
        // here is reachable in time.
        let free = |l: i32| plan.blocked & lane_bit(l) == 0;
        if (0..LANES).all(|l| !free(l)) {
            return None;
        }
        for f in 0..LANES {
            if !self.prev_free[f as usize] {
                continue;
            }
            let ok = (0..LANES).any(|g| free(g) && (f - g).abs() as f32 * LANE_T + REACT <= gap);
            if !ok {
                return None;
            }
        }
        Some(plan)
    }

    /// Places a planned row: `front` is where (in meeting space) you first
    /// touch it, `t_meet` when.
    fn place(&mut self, plan: RowPlan, front: f32, t_meet: f32, pm: f32, p: f32) {
        let mut clear = front + 4.0;
        for &(kind, lane, to) in &plan.items {
            let (_, half) = kind.half();
            let reach = Sim::reach(kind, pm);
            let center = front + reach + self.rng.range(0.0, 0.8);
            clear = clear.max(center + reach);
            let v = if kind.moves() { TRAFFIC_SPEED } else { 0.0 };
            let d = center - v * (center - self.dist) / p;
            let tint = self.rng.next_u32() % 8;
            self.obstacles.push(Obstacle {
                kind,
                d,
                x: lane_x(lane),
                lane,
                from_lane: lane,
                v,
                tint,
                change: to,
                blink: None,
                near: false,
                passed: false,
                whooshed: false,
                knock: None,
            });
            if !kind.moves() {
                self.last_static[lane as usize] = center + half;
            }
        }
        let free: Vec<i32> = (0..LANES).filter(|l| plan.blocked & lane_bit(*l) == 0).collect();
        // Coins in a free lane: the reward for picking the right one.
        if !free.is_empty() && (plan.items.is_empty() || self.rng.f32() < 0.45) {
            let lane = free[(self.rng.next_u32() as usize) % free.len()];
            for k in 0..5 {
                self.coin_list.push(Coin { d: front + 4.0 + (k as f32 - 2.0) * 2.6, lane, taken: false });
            }
        }
        // A boost pad just before the row now and then.
        if t_meet >= UNLOCK_BOOST && !free.is_empty() && self.rng.f32() < 0.13 {
            let lane = free[(self.rng.next_u32() as usize) % free.len()];
            self.pads.push(Pad { d: front - 5.0, lane, used: false });
        }
        for l in 0..LANES {
            self.prev_free[l as usize] = plan.blocked & lane_bit(l) == 0;
        }
        self.nasty = plan.nasty;
        self.clear_at = clear;
    }

    // --- planning (the attract mode and the tests' bot) ---

    /// Seconds until something in `lane` reaches the car (0 if it's alongside
    /// now; `f32::MAX` if nothing within `horizon`). With `anticipate`, a
    /// blinking truck counts in the lane it's about to take.
    pub fn threat(&self, lane: i32, horizon: f32, anticipate: bool) -> f32 {
        let p = self.speed_now().max(1.0);
        let mut best = f32::MAX;
        for o in &self.obstacles {
            if o.knock.is_some() || o.passed {
                continue;
            }
            let covers = o.covers(lane) || (anticipate && o.signalling() && o.change == Some(lane));
            if !covers {
                continue;
            }
            let (_, ol) = o.kind.half();
            let rel = o.d - self.dist;
            if rel + ol < -CAR_HALF.1 {
                continue;
            }
            let gap = rel - ol - CAR_HALF.1;
            let ttc = if gap <= 0.0 { 0.0 } else { gap / (p - o.v).max(1.0) };
            if ttc < horizon {
                best = best.min(ttc);
            }
        }
        best
    }

    /// The lane a careful driver would head for now, looking `horizon`
    /// seconds ahead: stay if it's clear, else the clearest lane reachable
    /// without driving through something on the way.
    pub fn plan(&self, horizon: f32, anticipate: bool) -> i32 {
        let here = self.lane;
        let th: Vec<f32> = (0..LANES).map(|l| self.threat(l, horizon, anticipate)).collect();
        if th[here as usize] == f32::MAX {
            return here;
        }
        let mut best = here;
        let mut best_score = th[here as usize];
        for l in 0..LANES {
            if l == here {
                continue;
            }
            // Passing through the middle lane needs it clear while you cross.
            if (l - here).abs() == 2 && th[1] < 0.8 {
                continue;
            }
            let score = th[l as usize] - 0.12 * (l - here).abs() as f32;
            if score > best_score + 0.05 {
                best = l;
                best_score = score;
            }
        }
        best
    }

    /// The attract mode: drives itself, dodging late enough for drama.
    pub fn autopilot(&mut self) {
        if self.crashed() || self.oil > 0.0 {
            return;
        }
        // Plan well ahead, but leave one-lane dodges late: near misses sell the card.
        let want = self.plan(AUTOPILOT_LOOK, true);
        if want == self.lane || (self.x - lane_x(self.lane)).abs() > 0.5 {
            return;
        }
        if (want - self.lane).abs() > 1 || self.threat(self.lane, 2.0, true) < 0.5 {
            self.steer((want - self.lane).signum());
        }
    }

    // --- what the host draws ---

    /// The car's pose (the crash tumble is a function of the crash's world time).
    pub fn pose(&self) -> Pose {
        match self.phase {
            Phase::Running => {
                let wobble = if self.oil > 0.0 { (self.oil * 40.0).sin() * 0.25 * (self.oil / OIL_LOCK) } else { 0.0 };
                // The nose turns into the lane change and the body rolls out of it.
                let yaw = (-self.vx * 0.03).clamp(-0.3, 0.3) + wobble;
                let roll = (self.vx * 0.014).clamp(-0.12, 0.12);
                Pose { x: self.x, lift: 0.0, yaw, roll, pitch: 0.02 * self.boost_k }
            }
            Phase::Crashing { world, .. } => self.crash_pose(world),
            Phase::Over { .. } => self.crash_pose(2.0),
        }
    }

    fn crash_pose(&self, w: f32) -> Pose {
        let dir = self.crash_dir;
        let air = (5.0 * w - 11.0 * w * w).max(0.0);
        let k = smooth(w / 0.5);
        Pose {
            x: self.crash_x + dir * 1.1 * smooth(w / 0.8),
            lift: air + 0.05 * k,
            yaw: dir * 0.9 * smooth(w / 0.8),
            roll: -dir * std::f32::consts::PI * k,
            pitch: 0.25 * (std::f32::consts::PI * k).sin(),
        }
    }

    /// Wheel spin angle (rad) for the distance driven.
    pub fn wheel_angle(&self) -> f32 {
        -(self.dist / 0.36) % std::f32::consts::TAU
    }

    /// The camera. `title` (0…1) blends to the attract-mode view; `time` sways it.
    pub fn camera(&self, title: f32, time: f32) -> Camera {
        let pose = self.pose();
        let car = vec3(pose.x, 0.6 + pose.lift, 0.0);
        // The chase camera comes in lower and closer as it gets faster (and
        // the FOV widens), and drops back when the car surges ahead (feel.rs).
        let (back, high) = self.feel.chase();
        let mut pos = vec3(pose.x * 0.5, high, back);
        let mut target = vec3(pose.x * 0.7, 0.5 + 0.3 * (high - feel::CAM_HIGH_SLOW), -8.0);
        // The crash: swing round the wreck.
        let crash = match self.phase {
            Phase::Running => None,
            Phase::Crashing { t, .. } => Some(t),
            Phase::Over { t } => Some(CRASH_TIME + t),
        };
        if let Some(t) = crash {
            // Swing out to the side the car slid to (away from what it hit), and up.
            let k = smooth(t / 1.2);
            let a = self.crash_dir * (0.5 + 0.3 * smooth(t / 4.0)) + 0.04 * (t * 0.6).sin();
            let orbit = car + vec3(a.sin() * 8.2, 5.4, a.cos() * 8.2);
            pos = pos.lerp(orbit, k);
            target = target.lerp(car + vec3(0.0, 2.2, 0.0), k);
        }
        let title = smooth(title);
        if title > 0.0 {
            let tpos = vec3(pose.x + 2.1 + 0.35 * (time * 0.31).sin(), 1.9 + 0.15 * (time * 0.23).sin(), 7.4);
            let ttarget = vec3(pose.x - 0.9, 0.9, -4.0);
            pos = pos.lerp(tpos, title);
            target = target.lerp(ttarget, title);
        }
        pos += self.jitter + self.feel.rumble() * (1.0 - title);
        let lean = if crash.is_none() { (-self.vx * 0.01).clamp(-0.06, 0.06) * (1.0 - title) } else { 0.0 };
        let rot = look_at(pos, target, Vec3::Y) * Quat::from_rotation_z(lean);
        let fov = self.feel.fov(self.boost_k) * (1.0 - title) + 0.92 * title;
        Camera { pos, rot, fov_y: fov, near: 0.1, far: 300.0 }
    }

    /// Meters from the camera to the car (depth of field focus).
    pub fn focus(&self, title: f32, time: f32) -> f32 {
        let c = self.camera(title, time);
        let p = self.pose();
        c.pos.distance(vec3(p.x, 0.6 + p.lift, 0.0))
    }
}

struct RowPlan {
    items: Vec<(Kind, i32, Option<i32>)>,
    blocked: u32,
    nasty: bool,
}

fn lane_bit(l: i32) -> u32 {
    1 << l
}
