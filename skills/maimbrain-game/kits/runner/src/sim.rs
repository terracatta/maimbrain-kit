//! The rules of the run: the hero's jump, the course generator, collisions,
//! scoring and the tumble when it goes wrong. No host calls (only
//! `maimbrain::Rng` and math), so `cargo test -p kit_runner` runs it natively
//! and a bot can play it.
//!
//! Units: world x in px (the hero runs toward +x and sits at `dist`), heights
//! in px above the ground (+ up), time in seconds. The host draws a world x at
//! screen x `HERO_X + (x - dist)` and a height h at `GROUND_Y - h`.
//!
//! The sim never draws or plays anything: it pushes `Cue`s (jumps, coins,
//! close calls, hits…) that the host turns into sound, haptics and juice.

use maimbrain::Rng;
use std::f32::consts::{PI, TAU};

// ---- Tuning knobs -----------------------------------------------------
/// Running speed at the start, px/s. Raise it and the first obstacles come
/// sooner and need earlier taps.
pub const START_SPEED: f32 = 235.0;
/// Speed added every `SPEED_EVERY` seconds, px/s ("FASTER!").
pub const SPEED_STEP: f32 = 15.0;
/// Seconds between speed-ups.
pub const SPEED_EVERY: f32 = 12.0;
/// Top speed, px/s. Past ~400 the 250 px of look-ahead is under 0.65 s.
pub const MAX_SPEED: f32 = 385.0;
/// A new obstacle family joins every this many seconds (see `Kind::ORDER`).
pub const FAMILY_EVERY: f32 = 15.0;
/// Seconds of open ground between obstacles (min, max) at the start and at
/// full difficulty (`RAMP_TIME`). Lower = denser course.
pub const GAP_START: (f32, f32) = (1.15, 1.85);
pub const GAP_END: (f32, f32) = (0.6, 1.05);
/// Seconds until gaps are at GAP_END and the press window at WINDOW_END.
pub const RAMP_TIME: f32 = 90.0;
/// Seconds of open ground (with a row of coins) before the first obstacle.
pub const WARMUP: f32 = 3.3;
/// Launch speed of a jump, px/s. With `GRAVITY_HOLD` it sets the full jump's
/// height (v² / 2g ≈ 160 px).
pub const JUMP_VY: f32 = 760.0;
/// Gravity while the finger is down and the hero is rising, px/s². Lower =
/// floatier, higher held jumps.
pub const GRAVITY_HOLD: f32 = 1800.0;
/// Gravity otherwise (falling, or rising after release), px/s².
pub const GRAVITY: f32 = 2600.0;
/// Letting go cuts the upward speed to this, px/s: the lower, the shorter a
/// quick tap's hop. Holding longer than `MAX_HOLD` s does nothing more.
pub const CUT_VY: f32 = 430.0;
pub const MAX_HOLD: f32 = 0.42;
/// The quickest tap counts as held this long, s, so it always clears a crate.
pub const MIN_HOLD: f32 = 0.11;
/// One extra jump in the air (and its launch speed, px/s).
pub const DOUBLE_JUMP: bool = true;
pub const DOUBLE_VY: f32 = 560.0;
/// A pass this close (px) counts as a CLOSE! call (bonus + slow-mo).
pub const CLOSE_PX: f32 = 9.0;
/// Points: one per meter run, plus these.
pub const COIN_POINTS: u32 = 5;
pub const CLOSE_POINTS: u32 = 10;
pub const PX_PER_M: f32 = 30.0;
/// The difficulty curve, in the one number that matters: how long (s) the
/// window is in which a press clears an obstacle. The generator resizes each
/// obstacle (within `Obstacle::resize`'s limits) toward WINDOW_START at the
/// start and WINDOW_END at `RAMP_TIME`, then on toward MIN_WINDOW, never
/// below it. Wider = gentler; this is the knob for "too hard / too easy".
pub const WINDOW_START: f32 = 0.34;
pub const WINDOW_END: f32 = 0.17;
pub const MIN_WINDOW: f32 = 0.12;
/// Where the hero runs on screen (px from the left edge): everything to its
/// right (360 - HERO_X px) is the player's look-ahead.
pub const HERO_X: f32 = 110.0;
// ---- End of tuning knobs ----------------------------------------------

/// The sim advances in fixed ticks (no tunnelling through thin crates at any dt).
pub const STEP: f32 = 1.0 / 120.0;
/// The hero's hitbox: half-width and height (px). Drawn a bit bigger: near
/// misses look closer than they are, which is the fair direction.
pub const HERO_HW: f32 = 12.0;
pub const HERO_H: f32 = 40.0;
/// After running off an edge, a press still counts as a ground jump this long (s).
pub const COYOTE: f32 = 0.08;
/// A press just before landing jumps on landing (s).
pub const BUFFER: f32 = 0.12;
/// The ends of a pit you can still stand on (px), and how far below the
/// ground the hero can be over a pit and still pop back up at its far edge.
pub const PIT_GRACE: f32 = 6.0;
pub const PIT_FALL: f32 = 14.0;
/// From the hit to the game-over card (s): the tumble plays while still Playing.
pub const DEATH_TIME: f32 = 1.5;
/// Run-cycle stride (px per step of the legs).
pub const STRIDE: f32 = 46.0;
/// Bees bob at this rate (rad/s).
pub const BOB_RATE: f32 = 4.2;
/// How far ahead of the hero the course is generated (px).
const SPAWN_AHEAD: f32 = 820.0;
/// Speed changes glide at this rate (px/s²).
const SPEED_ACCEL: f32 = 30.0;

/// Obstacle families, in the order they join the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A low crate: a tap clears it.
    Crate,
    /// A gap in the ground: jump it (hold for wide ones).
    Pit,
    /// A tall stack of crates: hold for a high jump (or double jump).
    Stack,
    /// A gull swooping in at head height + a bit: stay low, don't jump into it.
    Gull,
    /// A bee bobbing up and down: time it.
    Bee,
    /// A gull skimming the ground, fast: jump it.
    Skimmer,
    /// A long low log: hold for a long jump.
    Log,
}

impl Kind {
    pub const ORDER: [Kind; 7] = [Kind::Crate, Kind::Pit, Kind::Stack, Kind::Gull, Kind::Bee, Kind::Skimmer, Kind::Log];

    /// How often it's picked once unlocked (relative).
    fn weight(self) -> f32 {
        match self {
            Kind::Crate => 3.0,
            Kind::Pit => 2.2,
            Kind::Stack => 2.0,
            _ => 1.6,
        }
    }

    /// Hold times worth trying, in order of preference (the plan takes the
    /// first with a comfortable window).
    fn holds(self) -> &'static [f32] {
        match self {
            Kind::Crate | Kind::Skimmer => &[MIN_HOLD, 0.2],
            Kind::Pit => &[MIN_HOLD, 0.2, 0.3, MAX_HOLD],
            Kind::Stack => &[MAX_HOLD],
            Kind::Gull => &[],
            Kind::Bee => &[MIN_HOLD, 0.2, MAX_HOLD],
            Kind::Log => &[0.2, 0.3, MAX_HOLD],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Obstacle {
    pub id: u32,
    pub kind: Kind,
    /// Left edge (world x) at sim time `t0`; it moves at `vx`.
    pub x: f32,
    pub t0: f32,
    pub w: f32,
    /// Height of the thing (unused for pits).
    pub h: f32,
    /// Bottom above the ground at its lowest (0 for things on the ground).
    pub y: f32,
    /// Own horizontal speed, px/s (negative = flying toward the hero).
    pub vx: f32,
    /// Bobbing: how far it rises at the top of its bob (px) and its phase.
    pub bob: f32,
    pub phase: f32,
    /// Hard ones: never two in a row.
    pub nasty: bool,
    /// The first of its family this round (the host names it and hints).
    pub intro: bool,
    /// The host has been told it's coming (`Cue::NewKind`, when it's nearly on screen).
    pub announced: bool,
    pub passed: bool,
    /// The press window the generator measured for it (s; 1 = just run under it).
    pub window: f32,
    /// Smallest clearance seen while passing it (px), for CLOSE! calls.
    pub min_gap: f32,
}

impl Obstacle {
    pub fn left(&self, t: f32) -> f32 {
        self.x + self.vx * (t - self.t0)
    }
    pub fn right(&self, t: f32) -> f32 {
        self.left(t) + self.w
    }
    pub fn bottom(&self, t: f32) -> f32 {
        self.y + self.bob * (0.5 - 0.5 * (BOB_RATE * (t - self.t0) + self.phase).cos())
    }
    pub fn top(&self, t: f32) -> f32 {
        self.bottom(t) + self.h
    }

    /// Would a hero centred at world x `hx`, feet `h` above the ground, be in
    /// trouble at time `t`? (For a pit: standing over its middle.)
    pub fn hits(&self, t: f32, hx: f32, h: f32) -> bool {
        if self.kind == Kind::Pit {
            return h <= 0.0 && self.over_pit(hx);
        }
        if hx + HERO_HW <= self.left(t) || hx - HERO_HW >= self.right(t) {
            return false;
        }
        h < self.top(t) && h + HERO_H > self.bottom(t)
    }

    /// One step bigger/harder (`grow`) or smaller/easier, within this kind's
    /// limits; false if it's already at the limit. `v`: the speed it's met at.
    fn resize(&mut self, grow: bool, v: f32) -> bool {
        let k = if grow { 1.08 } else { 1.0 / 1.08 };
        let fit = |x: f32, lo: f32, hi: f32| (x * k).clamp(lo, hi);
        let before = (self.w, self.h, self.vx, self.phase);
        match self.kind {
            Kind::Crate => {
                self.h = fit(self.h, 18.0, 62.0);
                self.w = fit(self.w, 22.0, 52.0);
            }
            Kind::Stack => self.h = fit(self.h, 66.0, 134.0),
            Kind::Log => self.w = fit(self.w, 70.0, 170.0),
            Kind::Pit => self.w = fit(self.w, v * 0.22, v * 0.62),
            Kind::Gull => self.vx = -fit(-self.vx, 50.0, 260.0),
            Kind::Skimmer => {
                self.vx = -fit(-self.vx, 50.0, 260.0);
                self.h = fit(self.h, 22.0, 44.0);
            }
            Kind::Bee => self.phase += 0.6,
        }
        before != (self.w, self.h, self.vx, self.phase)
    }

    fn over_pit(&self, hx: f32) -> bool {
        hx > self.x + PIT_GRACE && hx < self.x + self.w - PIT_GRACE
    }
}

/// A coin, in world coordinates (`y` is its centre's height above the ground).
#[derive(Clone, Debug)]
pub struct Coin {
    pub x: f32,
    pub y: f32,
    pub arc: u32,
    pub taken: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Hero {
    /// Feet height above the ground and vertical speed (px, px/s; + up).
    pub h: f32,
    pub vy: f32,
    pub grounded: bool,
    /// The finger is down.
    pub held: bool,
    /// Seconds since this jump's launch.
    pub jump_t: f32,
    /// Jumps used since leaving the ground (0, 1, 2).
    pub jumps: u32,
    cut: bool,
    /// Seconds since leaving the ground.
    pub air_t: f32,
    buffer: f32,
    /// Seconds since landing, and how hard (px/s): squash.
    pub land_t: f32,
    pub land_speed: f32,
    /// Double-jump flip: radians turned so far (0 when not flipping).
    pub flip: f32,
    /// Leg cycle, in strides.
    pub run_phase: f32,
}

impl Hero {
    fn launch(&mut self, vy: f32) {
        self.vy = vy;
        self.jump_t = 0.0;
        self.cut = false;
        self.grounded = false;
        self.buffer = 0.0;
    }

    /// One tick of the jump's vertical motion: low gravity while held and
    /// rising (the first `MIN_HOLD` always counts as held), a cut on release.
    fn fall(&mut self, dt: f32) {
        let holding = self.held || self.jump_t < MIN_HOLD;
        if !holding && !self.cut {
            self.cut = true;
            self.vy = self.vy.min(CUT_VY);
        }
        let g = if holding && self.vy > 0.0 && self.jump_t < MAX_HOLD { GRAVITY_HOLD } else { GRAVITY };
        self.vy -= g * dt;
        self.h += self.vy * dt;
        self.jump_t += dt;
    }
}

/// Heights above the ground, one per tick after a press, for a jump held
/// `hold` seconds (until it lands).
pub fn jump_curve(hold: f32) -> Vec<f32> {
    let mut b = Hero::default();
    b.launch(JUMP_VY);
    b.held = true;
    let mut out = Vec::new();
    while out.len() < 400 {
        if b.jump_t >= hold {
            b.held = false;
        }
        b.fall(STEP);
        if b.h <= 0.0 {
            break;
        }
        out.push(b.h);
    }
    out
}

/// What to do about one obstacle: press `press` seconds from now and hold
/// for `hold` s (anywhere within ±window/2 of it works), or None: keep running.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub press: Option<f32>,
    pub hold: f32,
    pub window: f32,
}

/// True if a hero at `hero_x` at time `now`, running at `speed`, gets past
/// `o` with this press (seconds from now, and the jump's curve), or none.
pub fn clears(o: &Obstacle, now: f32, hero_x: f32, speed: f32, press: Option<(f32, &[f32])>) -> bool {
    // Only the ticks near the obstacle can touch it.
    let closing = (speed - o.vx).max(1.0);
    let gap = o.left(now) - hero_x - HERO_HW - 4.0;
    let k0 = ((gap / closing / STEP) as i64 - 2).max(1) as usize;
    for k in k0..k0 + 900 {
        let t = k as f32 * STEP;
        let hx = hero_x + speed * t;
        let h = match press {
            Some((tau, curve)) if t > tau => {
                let j = ((t - tau) / STEP).round() as usize;
                if j >= 1 && j <= curve.len() { curve[j - 1] } else { 0.0 }
            }
            _ => 0.0,
        };
        if o.hits(now + t, hx, h) {
            return false;
        }
        let past = if o.kind == Kind::Pit { hx > o.x + o.w } else { hx - HERO_HW > o.right(now + t) };
        if past {
            return true;
        }
    }
    true
}

/// The best way past `o` from here: a no-jump if that's safe (when the
/// family allows it), else the first hold (in the family's order) with a
/// window of at least `MIN_WINDOW`, else the widest window found.
pub fn plan(o: &Obstacle, now: f32, hero_x: f32, speed: f32) -> Option<Plan> {
    if matches!(o.kind, Kind::Gull | Kind::Bee) {
        // Safe to just run under it, even a little off in time?
        let ok = [-0.1f32, 0.0, 0.1].iter().all(|dt| clears(o, now + dt, hero_x, speed, None));
        if ok {
            return Some(Plan { press: None, hold: 0.0, window: 1.0 });
        }
    }
    let closing = (speed - o.vx).max(1.0);
    let arrive = (o.left(now) - hero_x - HERO_HW) / closing;
    let span = arrive.clamp(0.0, 1.3) + 0.05;
    let mut best: Option<Plan> = None;
    for &hold in o.kind.holds() {
        let curve = jump_curve(hold);
        // Scan press times every 1/120 s; keep the longest run that clears.
        let n = (span / STEP) as usize;
        let (mut run, mut best_run, mut best_end) = (0usize, 0usize, 0usize);
        for i in 0..=n {
            let tau = i as f32 * STEP;
            if clears(o, now, hero_x, speed, Some((tau, &curve))) {
                run += 1;
                if run > best_run {
                    best_run = run;
                    best_end = i;
                }
            } else {
                run = 0;
            }
        }
        if best_run == 0 {
            continue;
        }
        let window = best_run as f32 * STEP;
        let centre = (best_end as f32 - (best_run as f32 - 1.0) / 2.0) * STEP;
        let p = Plan { press: Some(centre), hold, window };
        if window >= MIN_WINDOW {
            return Some(p);
        }
        if best.is_none_or(|b| window > b.window) {
            best = Some(p);
        }
    }
    best
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    Bonk(Kind),
    Pit,
}

/// The tumble after a mistake (world coordinates, like the hero).
#[derive(Clone, Debug)]
pub struct Dying {
    pub t: f32,
    pub cause: Cause,
    pub x: f32,
    pub h: f32,
    pub vx: f32,
    pub vy: f32,
    pub rot: f32,
    pub spin: f32,
    pub resting: bool,
    /// For a pit: its far wall (world x), where the hero ends up jammed.
    wall: f32,
}

/// Things that happened this tick, for sound, haptics and juice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    Jump,
    DoubleJump,
    /// Landed, falling this fast (px/s).
    Land { speed: f32 },
    /// A coin at (x, height); `streak` coins in a row (climbs the scale).
    Coin { x: f32, y: f32, streak: u32 },
    /// Every coin of an arc taken.
    ArcDone { x: f32, y: f32 },
    /// A narrow escape at (x, height).
    Close { x: f32, y: f32 },
    SpeedUp,
    /// A new obstacle family is about to come on screen.
    NewKind(Kind),
    /// Every 100 m.
    Milestone(u32),
    Hit { kind: Kind, x: f32, y: f32 },
    Fall,
    Bounce,
    /// Jammed upside down in the pit, legs kicking.
    Stuck,
    Over,
}

#[derive(Clone, Debug)]
pub struct Sim {
    pub t: f32,
    /// World x of the hero's centre.
    pub dist: f32,
    pub speed: f32,
    pub hero: Hero,
    pub obstacles: Vec<Obstacle>,
    pub coins: Vec<Coin>,
    pub coins_got: u32,
    pub closes: u32,
    pub dying: Option<Dying>,
    pub over: bool,
    pub cues: Vec<Cue>,
    /// Speed-up count so far.
    pub tier: u32,
    /// Longest a jump has been held (s): the host teaches holding until it's done.
    pub longest_hold: f32,
    pub jumps: u32,
    pub double_jumps: u32,
    /// Seconds since the last coin and the last close call (the face reacts).
    pub coin_t: f32,
    pub close_t: f32,
    rng: Rng,
    acc: f32,
    meters: u32,
    cursor: f32,
    next_id: u32,
    next_arc: u32,
    introduced: usize,
    last_kind: Option<Kind>,
    repeat: u32,
    last_nasty: bool,
    streak: u32,
}

impl Sim {
    pub fn new(seed: u64) -> Sim {
        let mut s = Sim {
            t: 0.0,
            dist: 0.0,
            speed: START_SPEED,
            hero: Hero { grounded: true, jump_t: 9.0, land_t: 9.0, air_t: 0.0, ..Default::default() },
            obstacles: Vec::new(),
            coins: Vec::new(),
            coins_got: 0,
            closes: 0,
            dying: None,
            over: false,
            cues: Vec::new(),
            tier: 0,
            longest_hold: 0.0,
            jumps: 0,
            double_jumps: 0,
            coin_t: 9.0,
            close_t: 9.0,
            rng: Rng::new(seed),
            acc: 0.0,
            meters: 0,
            cursor: START_SPEED * WARMUP,
            next_id: 1,
            next_arc: 1,
            introduced: 0,
            last_kind: None,
            repeat: 0,
            last_nasty: false,
            streak: 0,
        };
        // The warm-up: a row of coins at running height, so the first win
        // comes in under a second whatever the player does.
        let arc = s.new_arc();
        for i in 0..9 {
            s.coins.push(Coin { x: 120.0 + i as f32 * 34.0, y: HERO_H * 0.5, arc, taken: false });
        }
        s.spawn();
        s
    }

    /// Distance run so far, in meters (frozen at the mistake).
    pub fn meters(&self) -> u32 {
        self.meters
    }

    pub fn score(&self) -> u32 {
        self.meters + self.coins_got * COIN_POINTS + self.closes * CLOSE_POINTS
    }

    pub fn alive(&self) -> bool {
        self.dying.is_none()
    }

    /// The finger went down: jump (from the ground, or once more in the air).
    pub fn press(&mut self) {
        if self.dying.is_some() || self.over {
            return;
        }
        let hero = &mut self.hero;
        hero.held = true;
        if hero.grounded || (hero.jumps == 0 && hero.air_t < COYOTE) {
            hero.launch(JUMP_VY);
            hero.jumps = 1;
            self.jumps += 1;
            self.cues.push(Cue::Jump);
        } else if DOUBLE_JUMP && hero.jumps < 2 {
            hero.launch(DOUBLE_VY);
            hero.jumps = 2;
            hero.flip = 0.001;
            self.double_jumps += 1;
            self.cues.push(Cue::DoubleJump);
        } else {
            hero.buffer = BUFFER;
        }
    }

    /// The finger came up: a jump still rising is cut short.
    pub fn release(&mut self) {
        self.hero.held = false;
    }

    /// Advances by `dt` seconds in fixed ticks.
    pub fn step(&mut self, dt: f32) {
        if self.over {
            return;
        }
        self.acc += dt;
        while self.acc >= STEP * 0.999 && !self.over {
            self.acc -= STEP;
            self.tick(STEP);
        }
    }

    /// The first obstacle the hero hasn't got past yet.
    pub fn next_obstacle(&self) -> Option<&Obstacle> {
        self.obstacles.iter().find(|o| !o.passed)
    }

    /// Seconds until the hero reaches `o` at the current speeds.
    pub fn time_to(&self, o: &Obstacle) -> f32 {
        (o.left(self.t) - self.dist - HERO_HW) / (self.speed - o.vx).max(1.0)
    }

    /// The best way past `o` from the hero's current state on the ground.
    pub fn plan_for(&self, o: &Obstacle) -> Option<Plan> {
        plan(o, self.t, self.dist, self.speed)
    }

    fn ground_under(&self, x: f32) -> bool {
        !self.obstacles.iter().any(|o| o.kind == Kind::Pit && o.over_pit(x))
    }

    fn new_arc(&mut self) -> u32 {
        self.next_arc += 1;
        self.next_arc
    }

    fn tick(&mut self, dt: f32) {
        self.t += dt;
        if self.dying.is_some() {
            self.tumble(dt);
            return;
        }
        // Faster every SPEED_EVERY seconds, gliding to the new speed.
        let tier = (self.t / SPEED_EVERY) as u32;
        if tier > self.tier {
            self.tier = tier;
            if START_SPEED + SPEED_STEP * tier as f32 <= MAX_SPEED + SPEED_STEP * 0.5 {
                self.cues.push(Cue::SpeedUp);
            }
        }
        let target = (START_SPEED + SPEED_STEP * self.tier as f32).min(MAX_SPEED);
        self.speed = if self.speed < target { (self.speed + SPEED_ACCEL * dt).min(target) } else { target };
        self.dist += self.speed * dt;
        let m = (self.dist / PX_PER_M) as u32;
        if m / 100 > self.meters / 100 {
            self.cues.push(Cue::Milestone(m / 100 * 100));
        }
        self.meters = m;
        self.coin_t += dt;
        self.close_t += dt;

        // The hero.
        let over_ground = self.ground_under(self.dist);
        let hero = &mut self.hero;
        hero.land_t += dt;
        if hero.held && !hero.grounded {
            self.longest_hold = self.longest_hold.max(hero.jump_t.min(MAX_HOLD));
        }
        if hero.buffer > 0.0 {
            hero.buffer -= dt;
        }
        if hero.grounded {
            hero.run_phase += self.speed * dt / STRIDE;
            if !over_ground {
                // Ran off an edge.
                hero.grounded = false;
                hero.jumps = 0;
                hero.air_t = 0.0;
                hero.vy = 0.0;
                hero.jump_t = 9.0;
                hero.cut = true;
            }
        }
        if !hero.grounded {
            hero.air_t += dt;
            hero.fall(dt);
            if hero.flip > 0.0 {
                hero.flip += dt * TAU / 0.42;
                if hero.flip >= TAU {
                    hero.flip = 0.0;
                }
            }
            if hero.h <= 0.0 && hero.vy <= 0.0 {
                if over_ground {
                    let speed = -hero.vy;
                    hero.h = 0.0;
                    hero.vy = 0.0;
                    hero.grounded = true;
                    hero.jumps = 0;
                    hero.flip = 0.0;
                    hero.land_t = 0.0;
                    hero.land_speed = speed;
                    self.cues.push(Cue::Land { speed });
                    if hero.buffer > 0.0 {
                        // A press just before landing: jump now.
                        hero.launch(JUMP_VY);
                        hero.jumps = 1;
                        self.jumps += 1;
                        self.cues.push(Cue::Jump);
                    }
                } else if hero.h < -PIT_FALL {
                    self.die(Cause::Pit);
                    return;
                }
            }
        }

        // Obstacles: hits, near misses, passing.
        let (t, hx, h) = (self.t, self.dist, self.hero.h);
        let mut hit = None;
        let mut closes = Vec::new();
        for o in &mut self.obstacles {
            if o.intro && !o.announced && o.left(t) - hx < 255.0 {
                o.announced = true;
                if o.kind != Kind::Crate {
                    self.cues.push(Cue::NewKind(o.kind));
                }
            }
            if o.passed {
                continue;
            }
            if o.kind == Kind::Pit {
                if hx >= o.x + o.w - PIT_GRACE {
                    o.passed = true;
                    // How high the hero was crossing the far edge.
                    if h > 0.0 && h < CLOSE_PX * 1.6 {
                        closes.push((o.x + o.w, 0.0));
                    }
                }
                continue;
            }
            if o.hits(t, hx, h) {
                hit = Some((o.kind, o.left(t).max(hx - HERO_HW), h + HERO_H * 0.5));
                break;
            }
            let (l, r) = (o.left(t), o.right(t));
            if hx + HERO_HW > l && hx - HERO_HW < r {
                let b = o.bottom(t);
                let gap = if h >= b + o.h { h - (b + o.h) } else { b - (h + HERO_H) };
                o.min_gap = o.min_gap.min(gap);
            }
            if hx - HERO_HW > r {
                o.passed = true;
                if o.min_gap < CLOSE_PX {
                    closes.push((r, h + HERO_H * 0.5));
                }
            }
        }
        if let Some((kind, x, y)) = hit {
            self.cues.push(Cue::Hit { kind, x, y });
            self.die(Cause::Bonk(kind));
            return;
        }
        for (x, y) in closes {
            self.closes += 1;
            self.close_t = 0.0;
            self.cues.push(Cue::Close { x, y });
        }

        // Coins.
        let cy = self.hero.h + HERO_H * 0.5;
        let mut got = Vec::new();
        for (i, c) in self.coins.iter_mut().enumerate() {
            if !c.taken && (c.x - hx).abs() < 22.0 && (c.y - cy).abs() < 30.0 {
                c.taken = true;
                got.push(i);
            }
        }
        for i in got {
            self.streak = if self.coin_t < 0.45 { self.streak + 1 } else { 1 };
            self.coin_t = 0.0;
            self.coins_got += 1;
            let (x, y, arc) = (self.coins[i].x, self.coins[i].y, self.coins[i].arc);
            self.cues.push(Cue::Coin { x, y, streak: self.streak });
            if self.coins.iter().filter(|c| c.arc == arc).all(|c| c.taken) {
                self.cues.push(Cue::ArcDone { x, y });
            }
        }

        // The course ahead, and forgetting what's behind.
        self.spawn();
        let behind = self.dist - 300.0;
        let t = self.t;
        self.obstacles.retain(|o| o.right(t).max(o.x + o.w) > behind);
        self.coins.retain(|c| c.x > behind);
    }

    fn die(&mut self, cause: Cause) {
        let (vx, vy, spin) = match cause {
            Cause::Bonk(_) => (-45.0, 540.0, -11.0),
            Cause::Pit => (0.4 * self.speed, self.hero.vy.min(0.0) - 60.0, 9.0),
        };
        if cause == Cause::Pit {
            self.cues.push(Cue::Fall);
        }
        let x = self.dist;
        let wall = self.obstacles.iter().find(|o| o.kind == Kind::Pit && x > o.x && x < o.x + o.w).map_or(x + 40.0, |o| o.x + o.w);
        self.dying = Some(Dying { t: 0.0, cause, x, h: self.hero.h, vx, vy, rot: 0.0, spin, resting: false, wall });
    }

    fn tumble(&mut self, dt: f32) {
        // The world stops (fast) while the hero tumbles, so it stays in view.
        self.speed *= (-14.0 * dt).exp();
        self.dist += self.speed * dt;
        let Some(mut d) = self.dying.take() else { return };
        d.t += dt;
        if !d.resting {
            d.vy -= GRAVITY * 0.75 * dt;
            d.h += d.vy * dt;
            d.x += d.vx * dt;
            d.rot += d.spin * dt;
            if d.cause == Cause::Pit {
                // Head first into the hole, jammed against the far wall.
                if d.x > d.wall - 20.0 {
                    d.x = d.wall - 20.0;
                    d.vx = 0.0;
                }
                if d.h <= -16.0 {
                    d.h = -16.0;
                    d.resting = true;
                    self.cues.push(Cue::Stuck);
                }
            } else if self.ground_under(d.x) && d.h <= 0.0 && d.vy < 0.0 {
                d.h = 0.0;
                if -d.vy > 170.0 {
                    d.vy = -d.vy * 0.45;
                    d.vx *= 0.55;
                    d.spin *= 0.5;
                    self.cues.push(Cue::Bounce);
                } else {
                    d.resting = true;
                    d.vy = 0.0;
                }
            }
            d.h = d.h.max(-400.0);
        } else {
            // Slide to a stop and roll onto its back, feet in the air.
            d.vx *= (-9.0 * dt).exp();
            d.x += d.vx * dt;
            let rest = ((d.rot / PI - 1.0) / 2.0).round() * 2.0 * PI + PI;
            d.rot += (rest - d.rot) * (1.0 - (-14.0 * dt).exp());
        }
        if d.t >= DEATH_TIME && !self.over {
            self.over = true;
            self.cues.push(Cue::Over);
        }
        self.dying = Some(d);
    }

    // ---- The course generator ----------------------------------------

    fn spawn(&mut self) {
        while self.cursor < self.dist + SPAWN_AHEAD {
            self.spawn_one();
        }
    }

    fn pick_kind(&mut self, unlocked: usize) -> Kind {
        if self.last_nasty {
            // Never two hard ones in a row: a crate (or a narrow pit) to breathe.
            return if unlocked >= 2 && self.rng.f32() < 0.3 { Kind::Pit } else { Kind::Crate };
        }
        let kinds = &Kind::ORDER[..unlocked];
        for _ in 0..3 {
            let total: f32 = kinds.iter().map(|k| k.weight()).sum();
            let mut r = self.rng.f32() * total;
            let mut pick = kinds[0];
            for k in kinds {
                r -= k.weight();
                if r <= 0.0 {
                    pick = *k;
                    break;
                }
            }
            // The same family three times running is boring: roll again.
            if !(Some(pick) == self.last_kind && self.repeat >= 2) {
                return pick;
            }
        }
        Kind::Crate
    }

    fn spawn_one(&mut self) {
        let speed = self.speed.max(1.0);
        // When (and how fast) the hero will get there.
        let eta = self.t + (self.cursor - self.dist) / speed;
        let tier = (eta / SPEED_EVERY) as u32;
        let v = (START_SPEED + SPEED_STEP * tier as f32).min(MAX_SPEED).max(self.speed);
        let d = (eta / RAMP_TIME).clamp(0.0, 1.0);
        let unlocked = (1 + (eta / FAMILY_EVERY) as usize).min(Kind::ORDER.len());
        let (kind, intro) = if unlocked > self.introduced && !self.last_nasty {
            self.introduced = unlocked;
            (Kind::ORDER[unlocked - 1], true)
        } else {
            (self.pick_kind(unlocked), false)
        };
        // The first of a family, and the breather after a hard one, come easy.
        let easy = intro || self.last_nasty;
        // Room to take off: big jumps start further back.
        self.cursor += match kind {
            Kind::Stack | Kind::Log => 0.3,
            Kind::Pit => 0.1,
            _ => 0.0,
        } * v;
        let lerp = |a: f32, b: f32| a + (b - a) * d;
        let r = &mut self.rng;
        let mut o = Obstacle {
            id: self.next_id,
            kind,
            x: self.cursor,
            t0: self.t + (self.cursor - self.dist) / speed,
            w: 30.0,
            h: 36.0,
            y: 0.0,
            vx: 0.0,
            bob: 0.0,
            phase: 0.0,
            nasty: kind != Kind::Crate,
            intro,
            announced: false,
            passed: false,
            window: 1.0,
            min_gap: f32::MAX,
        };
        self.next_id += 1;
        match kind {
            Kind::Crate => {
                o.w = r.range(28.0, 38.0);
                o.h = if easy { 30.0 } else { lerp(30.0, 44.0) * r.range(0.9, 1.05) };
            }
            Kind::Pit => {
                // Widths in seconds of running: wide ones need a held jump.
                o.w = v * if easy { 0.3 } else { lerp(0.32, 0.5) * r.range(0.9, 1.08) };
                o.nasty = o.w > v * 0.4;
            }
            Kind::Stack => {
                o.w = r.range(38.0, 46.0);
                o.h = if intro { 84.0 } else { lerp(84.0, 114.0) * r.range(0.93, 1.0) };
            }
            Kind::Gull => {
                o.w = 56.0;
                o.h = 26.0;
                o.y = HERO_H + 22.0;
                o.vx = -lerp(70.0, 120.0);
            }
            Kind::Bee => {
                o.w = 38.0;
                o.h = 34.0;
                o.bob = 104.0;
                o.phase = r.range(0.0, TAU);
            }
            Kind::Skimmer => {
                o.w = 52.0;
                o.h = 28.0;
                o.y = 4.0;
                o.vx = -lerp(110.0, 160.0);
            }
            Kind::Log => {
                o.w = if intro { 86.0 } else { lerp(86.0, 128.0) * r.range(0.92, 1.05) };
                o.h = 28.0;
            }
        }
        // Fair by construction, and the difficulty curve: resize (or re-time)
        // it until the press window that clears it is about the target for
        // this point of the run (wide early, tight late), never under MIN_WINDOW.
        const LOOK: f32 = 1.4;
        // (It keeps tightening past RAMP_TIME, down to MIN_WINDOW: everyone falls eventually.)
        let k = (eta / RAMP_TIME).min(2.0);
        let target = (WINDOW_START + (WINDOW_END - WINDOW_START) * k).max(MIN_WINDOW);
        let target = if intro { WINDOW_START } else if easy { (target + 0.04).min(WINDOW_START) } else { target };
        let mut best = None;
        for _ in 0..24 {
            let (now, hx) = (o.t0 - LOOK, o.x - v * LOOK);
            best = plan(&o, now, hx, v);
            let w = match best {
                Some(Plan { press: None, .. }) => break, // run under it
                Some(p) => p.window,
                None => 0.0,
            };
            o.window = w;
            let grow = if w < MIN_WINDOW || w < target * 0.9 {
                false
            } else if w > target * 1.25 {
                true
            } else {
                break;
            };
            if !o.resize(grow, v) && w >= MIN_WINDOW {
                break; // as big (or small) as this kind goes
            }
        }
        // Coins trace the ideal path: a reward for a good jump.
        let hx = o.x - v * LOOK;
        let mut end = o.x + o.w;
        let coins = intro || self.rng.f32() < 0.7;
        match best {
            Some(Plan { press: Some(tau), hold, .. }) => {
                let curve = jump_curve(hold);
                let px = hx + v * tau;
                end = end.max(px + v * curve.len() as f32 * STEP);
                if coins {
                    let arc = self.new_arc();
                    let n = 5 + (curve.len() > 70) as usize * 2;
                    for i in 0..n {
                        let j = ((i as f32 + 0.5) / n as f32 * curve.len() as f32) as usize;
                        let j = j.min(curve.len() - 1);
                        self.coins.push(Coin { x: px + v * (j + 1) as f32 * STEP, y: curve[j] + HERO_H * 0.5, arc, taken: false });
                    }
                }
            }
            _ => {
                // Run under it: a line of coins on the ground where you meet it.
                if coins {
                    let arc = self.new_arc();
                    for i in 0..5 {
                        self.coins.push(Coin { x: o.x - 50.0 + i as f32 * 30.0, y: HERO_H * 0.5, arc, taken: false });
                    }
                }
            }
        }
        let gap = self.rng.range(lerp(GAP_START.0, GAP_END.0), lerp(GAP_START.1, GAP_END.1)) + if o.nasty { 0.3 } else { 0.0 } + if intro { 0.35 } else { 0.0 };
        self.cursor = end + gap * v;
        self.repeat = if Some(kind) == self.last_kind { self.repeat + 1 } else { 1 };
        self.last_kind = Some(kind);
        self.last_nasty = o.nasty;
        self.obstacles.push(o);
    }
}

/// Plays from the plans, nearly perfectly: the title card's attract mode and
/// the fairness test's oracle. `sloppy` (s) adds timing error.
#[derive(Clone, Debug)]
pub struct Autopilot {
    rng: Rng,
    pub sloppy: f32,
    done: u32,
    press_at: Option<f32>,
    release_at: Option<f32>,
    hold: f32,
}

impl Autopilot {
    pub fn new(seed: u64, sloppy: f32) -> Autopilot {
        Autopilot { rng: Rng::new(seed), sloppy, done: 0, press_at: None, release_at: None, hold: 0.0 }
    }

    /// Call once per frame before `sim.step`.
    pub fn drive(&mut self, sim: &mut Sim) {
        if let Some(at) = self.release_at
            && sim.t >= at
        {
            sim.release();
            self.release_at = None;
        }
        if let Some(at) = self.press_at {
            if sim.t >= at {
                sim.press();
                self.release_at = Some(sim.t + self.hold);
                self.press_at = None;
            }
            return;
        }
        if !sim.hero.grounded || self.release_at.is_some() {
            return;
        }
        let Some(o) = sim.obstacles.iter().find(|o| !o.passed && o.id > self.done) else { return };
        if sim.time_to(o) > 1.1 {
            return;
        }
        self.done = o.id;
        if let Some(Plan { press: Some(tau), hold, window }) = sim.plan_for(o) {
            let err = self.rng.range(-1.0, 1.0) * self.sloppy.min(window * 0.5);
            self.press_at = Some(sim.t + (tau + err).max(0.0));
            self.hold = hold;
        }
    }
}
