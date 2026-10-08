//! The rules: the ship and its auto-fire, enemies and their bullets, the
//! waves and the boss, power-ups, hits, scoring and the combo. No host calls
//! (only `maimbrain::Rng` and math), so `cargo test -p kit_shooter` runs it
//! natively and the bot (`bot.rs`) can play thousands of rounds.
//!
//! The sim never draws or plays anything: it pushes `Cue`s that the host
//! (`lib.rs`, `sound.rs`) turns into juice, sound and haptics, and the
//! drawing code (`draw.rs`) reads its public state.

use maimbrain::Rng;

use crate::waves::{self, Formation, Spawn};

// ---- Tuning knobs -----------------------------------------------------
/// Thumb-to-ship gain: the ship moves this many units per unit of thumb
/// travel. Above 1 crosses the screen with a shorter drag; much higher feels twitchy.
pub const DRAG_GAIN: f32 = 1.25;
/// How fast the ship catches up with where the thumb puts it (1/s). Lower feels floaty.
pub const FOLLOW_RATE: f32 = 26.0;
/// The ship keeps at least this far above the thumb (units), easing up, so the thumb never covers it.
pub const MIN_LIFT: f32 = 88.0;
/// Seconds between auto-fire volleys. Lower = more damage (and more shots on screen).
pub const FIRE_INTERVAL: f32 = 0.11;
/// Hits the ship can take (the HUD's hearts).
pub const LIVES: u32 = 3;
/// Seconds of blinking invulnerability after a hit.
pub const INVULN_TIME: f32 = 1.8;
/// Opening seconds with no enemy fire: the first seconds can't be failed.
pub const GRACE: f32 = 4.0;
/// Difficulty steps up every this many seconds: faster, denser bullets and waves.
pub const HEAT_STEP: f32 = 11.0;
/// What each step adds to the heat multiplier (1.0 at the start; capped at `HEAT_MAX`).
pub const HEAT_GAIN: f32 = 0.14;
pub const HEAT_MAX: f32 = 2.2;
/// Seconds between waves at heat 1 (divided by heat^0.8 later).
pub const WAVE_GAP: f32 = 2.7;
/// Seconds into each loop when the boss arrives (its WARNING shows `WARNING_TIME` before).
pub const BOSS_AT: f32 = 48.0;
pub const WARNING_TIME: f32 = 2.5;
/// The first boss's health in shots; each later boss has 50 % more.
pub const BOSS_HP: f32 = 140.0;
/// Base speed of enemy bullets (units/s) at heat 1.
pub const BULLET_SPEED: f32 = 185.0;
/// Seconds a kill keeps the combo alive. Longer = easier multipliers.
pub const COMBO_WINDOW: f32 = 1.6;
/// A power-up drops at least once every this many kills.
pub const POWER_EVERY: u32 = 20;
/// Seconds a Spread or Magnet power-up lasts.
pub const POWER_TIME: f32 = 10.0;
/// Ship hit radius (units). Smaller is more forgiving; the art is ~2.5× bigger.
pub const SHIP_R: f32 = 7.0;
/// Bullets passing this close (units, edge to edge) without hitting count as a graze: bonus points.
pub const GRAZE: f32 = 14.0;
/// Jellies start firing (single slow aimed shots) after this many seconds.
pub const JELLY_FIRE_AT: f32 = 7.0;
/// Seconds of the slow-motion death before the game-over card.
pub const DEATH_TIME: f32 = 1.7;

// ---- Geometry -----------------------------------------------------------
pub const W: f32 = 360.0;
pub const H: f32 = 640.0;
/// Where the ship may go: below the HUD, above the thumb.
pub const SHIP_X: (f32, f32) = (18.0, W - 18.0);
pub const SHIP_Y: (f32, f32) = (200.0, 560.0);
pub const SHIP_START: (f32, f32) = (180.0, 470.0);
const DEMO_Y_MAX: f32 = 420.0;
const SHOT_SPEED: f32 = 780.0;
/// Enemies only fire from above this line, and only this far above the ship.
const FIRE_LINE: f32 = 430.0;
const FIRE_CLEARANCE: f32 = 130.0;
/// A telegraph glow shows this long before an enemy fires.
pub const TELL: f32 = 0.35;
const DEATH_SLOWMO: f32 = 0.3;
const GEM_VALUE: u64 = 25;
const GRAZE_VALUE: u64 = 10;

// ---- Types --------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Small, slow, comes in formations. The bread and butter.
    Jelly,
    /// Crosses the screen bobbing, fires aimed shots.
    Swooper,
    /// Stops, aims at the ship (a line shows where), then dashes.
    Darter,
    /// Big and tough; hovers and fires fans of big bullets.
    Bulb,
    /// Hovers and sprays a spiral.
    Spinner,
}

impl Kind {
    pub const ALL: [Kind; 5] = [Kind::Jelly, Kind::Swooper, Kind::Darter, Kind::Bulb, Kind::Spinner];
    /// Hit points at loop 0.
    pub fn hp(self) -> f32 {
        match self {
            Kind::Jelly => 1.0,
            Kind::Swooper => 2.0,
            Kind::Darter => 3.0,
            Kind::Bulb => 14.0,
            Kind::Spinner => 10.0,
        }
    }
    /// Body radius (hit circle) in units.
    pub fn radius(self) -> f32 {
        match self {
            Kind::Jelly => 15.0,
            Kind::Swooper => 15.0,
            Kind::Darter => 14.0,
            Kind::Bulb => 26.0,
            Kind::Spinner => 22.0,
        }
    }
    /// Points for a kill, before the combo multiplier.
    pub fn points(self) -> u64 {
        match self {
            Kind::Jelly => 100,
            Kind::Swooper => 150,
            Kind::Darter => 250,
            Kind::Bulb => 600,
            Kind::Spinner => 500,
        }
    }
    /// Gems it drops.
    pub fn gems(self) -> u32 {
        match self {
            Kind::Jelly => 1,
            Kind::Swooper => 1,
            Kind::Darter => 2,
            Kind::Bulb => 5,
            Kind::Spinner => 4,
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

/// How an enemy moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Path {
    /// Straight down at `vy`, swaying `sway` units side to side.
    Drift { vy: f32, sway: f32 },
    /// Across the screen at `vx`, sinking at `vy`, bobbing `amp` units at `freq` rad/s.
    Swoop { vx: f32, vy: f32, amp: f32, freq: f32 },
    /// Down to `stop_y`, hover for `hold` seconds (firing), then leave upward.
    Hover { stop_y: f32, hold: f32 },
    /// Down to `stop_y`, aim at the ship, then dash where it was.
    Dive { stop_y: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    /// Velocity last frame (for the bot and for drawing lean).
    pub vx: f32,
    pub vy: f32,
    pub x0: f32,
    pub y0: f32,
    pub age: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub path: Path,
    /// Seconds until it fires (counts down only where it may fire).
    pub fire_cd: f32,
    /// Hit flash, 1 → 0.
    pub flash: f32,
    /// Darters: 0 arriving, 1 aiming, 2 locked (line flashes), 3 dashing.
    pub stage: u8,
    pub aim: (f32, f32),
    /// Spinners: spray angle.
    pub spin: f32,
}

impl Enemy {
    fn new(s: &Spawn, hp_k: f32, rng: &mut Rng) -> Enemy {
        let hp = (s.kind.hp() * hp_k).round().max(1.0);
        Enemy {
            kind: s.kind,
            x: s.x,
            y: s.y,
            vx: 0.0,
            vy: 0.0,
            x0: s.x,
            y0: s.y,
            age: 0.0,
            hp,
            max_hp: hp,
            path: s.path,
            fire_cd: first_fire(s.kind) * rng.range(0.8, 1.25),
            flash: 0.0,
            stage: 0,
            aim: (s.x, H),
            spin: rng.range(0.0, std::f32::consts::TAU),
        }
    }

    /// True while it may fire now and its telegraph should show.
    pub fn telegraph(&self) -> bool {
        self.fire_cd < TELL && fires(self.kind)
    }
}

fn fires(k: Kind) -> bool {
    !matches!(k, Kind::Darter)
}

/// Seconds after arriving before an enemy's first shot.
fn first_fire(k: Kind) -> f32 {
    match k {
        Kind::Jelly => 2.6,
        Kind::Swooper => 0.8,
        Kind::Darter => 99.0,
        Kind::Bulb => 1.1,
        Kind::Spinner => 1.2,
    }
}

/// An enemy bullet: big and bright in the art; `r` is its hit radius.
#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub r: f32,
    /// Big bullets (bulbs, the boss) draw orange and larger.
    pub big: bool,
    pub grazed: bool,
    pub age: f32,
}

/// The ship's shot.
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Power {
    /// Three shots, five if picked again while active.
    Spread,
    /// A bubble that takes one hit.
    Shield,
    /// Gems and power-ups fly to the ship.
    Magnet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropKind {
    Gem,
    Power(Power),
    /// One heart back (the boss drops one).
    Heart,
}

#[derive(Clone, Copy, Debug)]
pub struct Drop {
    pub kind: DropKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub age: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    /// Rings of bullets bursting outward.
    Bloom,
    /// Aimed fans from alternating hands.
    Rain,
    /// A two-armed spiral while swaying.
    Sweep,
}

#[derive(Clone, Copy, Debug)]
pub struct Boss {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub age: f32,
    pub attack: Attack,
    /// Seconds into the current attack (the first `CHARGE` are its telegraph).
    pub attack_t: f32,
    shot_t: f32,
    pub spin: f32,
    pub flash: f32,
    /// Seconds since it was beaten (it shakes apart for `BOSS_DEATH`).
    pub dying: Option<f32>,
    booms: u32,
    hand: bool,
}

/// Boss attack timing: charge (telegraph), fire, rest.
pub const CHARGE: f32 = 0.8;
pub const ATTACK_FIRE: f32 = 4.6;
const ATTACK_REST: f32 = 0.9;
pub const BOSS_DEATH: f32 = 1.6;
/// The boss's hit ellipse (half-width, half-height) and where it hovers.
pub const BOSS_RX: f32 = 62.0;
pub const BOSS_RY: f32 = 44.0;
const BOSS_Y: f32 = 196.0;
/// Where the boss's hands are, from its centre (the Rain attack fires from them).
pub const BOSS_HAND: (f32, f32) = (84.0, 18.0);

impl Boss {
    /// 0…1 while charging up the current attack (for the telegraph).
    pub fn charge(&self) -> f32 {
        if self.entering() || self.dying.is_some() { 0.0 } else { (self.attack_t / CHARGE).min(1.0) * (self.attack_t < CHARGE) as u8 as f32 }
    }
    pub fn entering(&self) -> bool {
        self.age < 2.4
    }
    /// Below 40 % health it attacks faster.
    pub fn enraged(&self) -> bool {
        self.hp < self.max_hp * 0.4
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BannerKind {
    /// The boss is coming.
    Warning,
    /// A new enemy type joins.
    New(Kind),
    /// Boss beaten: the loop starts again, harder.
    Clear,
}

#[derive(Clone, Copy, Debug)]
pub struct Banner {
    pub kind: BannerKind,
    pub age: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Ship {
    pub x: f32,
    pub y: f32,
    /// Where the thumb wants it.
    pub tx: f32,
    pub ty: f32,
    pub vx: f32,
    pub hp: u32,
    /// Seconds of invulnerability left (it blinks).
    pub invuln: f32,
    pub shield: bool,
    /// 0 single, 1 three-way, 2 five-way; `spread_t` seconds left.
    pub spread: u8,
    pub spread_t: f32,
    pub magnet_t: f32,
    fire_cd: f32,
    side: f32,
    /// 0…1: how many bullets are close (the face worries).
    pub danger: f32,
    /// Seconds of a happy face (after a pickup).
    pub happy: f32,
    /// Spin while dying (radians).
    pub spin: f32,
}

/// The thumb's drag: where it touched down and where the ship was then.
#[derive(Clone, Copy, Debug)]
pub struct Grab {
    fx: f32,
    fy: f32,
    sx: f32,
    sy: f32,
    pub x: f32,
    pub y: f32,
}

/// Something happened: the host plays juice, sound and haptics for it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    /// The ship fired a volley.
    Shot,
    /// An enemy took a hit and lived.
    Hit { x: f32, y: f32 },
    /// An enemy died: `points` already multiplied by `mult`; `combo` is the chain length.
    Kill { x: f32, y: f32, kind: Kind, points: u64, mult: u32, combo: u32 },
    /// A gem collected; `streak` counts gems in quick succession (for rising pitch).
    Gem { x: f32, y: f32, streak: u32 },
    Power { x: f32, y: f32, power: Power },
    Heart { x: f32, y: f32 },
    /// A bullet passed close without hitting.
    Graze { x: f32, y: f32 },
    /// An enemy fired (`big` for bulbs, spinners and the boss).
    EnemyFire { big: bool },
    /// A darter locked on and is about to dash.
    DarterLock { x: f32, y: f32 },
    ShieldBreak { x: f32, y: f32 },
    /// The ship was hit; `hp` left.
    Hurt { x: f32, y: f32, hp: u32 },
    /// A combo of at least 5 ran out (or was broken by a hit).
    ComboEnd { count: u32 },
    NewEnemy(Kind),
    Warning,
    BossArrive,
    BossHit { x: f32, y: f32 },
    /// The boss dropped below 40 % health: it attacks faster from now on.
    BossEnrage,
    /// The boss starts charging an attack.
    BossCharge(Attack),
    /// The boss is beaten (it shakes apart, then `BossDown`).
    BossBeaten { x: f32, y: f32 },
    BossDown { x: f32, y: f32, points: u64 },
    /// An explosion with no other meaning (the death and boss chains); `size` 0…1.
    Boom { x: f32, y: f32, size: f32 },
    /// The last heart is gone: the slow-motion death starts.
    Dying { x: f32, y: f32 },
    /// The death animation finished: the round is over.
    Over,
}

pub struct Sim {
    /// Seconds of play (real time, including the slow-motion death).
    pub t: f32,
    /// Seconds into the current loop (world time; the boss comes at `BOSS_AT`).
    pub phase: f32,
    /// Bosses beaten.
    pub loop_n: u32,
    pub ship: Ship,
    pub grab: Option<Grab>,
    pub shots: Vec<Shot>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub drops: Vec<Drop>,
    pub boss: Option<Boss>,
    pub banner: Option<Banner>,
    pub score: u64,
    pub kills: u32,
    pub combo: u32,
    pub combo_left: f32,
    pub best_combo: u32,
    pub grazes: u32,
    /// Seconds since the slow-motion death began.
    pub dying: Option<f32>,
    /// Where the fatal hit landed (drawn as a ring: why you died).
    pub killer: Option<(f32, f32)>,
    pub over: bool,
    /// Title-card autopilot: hits only flash, nothing is scored for real.
    pub demo: bool,
    pub cues: Vec<Cue>,
    pending: Vec<Spawn>,
    next_wave: f32,
    opening: usize,
    intros_done: usize,
    last_nasty: bool,
    warned: bool,
    seen: [bool; 5],
    kills_since_power: u32,
    last_power: Option<Power>,
    gem_streak: u32,
    gem_streak_t: f32,
    death_booms: u32,
    /// Waves come from this one alone (a daily seed gives everyone the same waves).
    wave_rng: Rng,
    /// Everything else random (drops, aim jitter).
    rng: Rng,
}

impl Sim {
    pub fn new(seed: u64) -> Sim {
        let mut wave_rng = Rng::new(seed);
        let rng = Rng::new(seed ^ 0x5eed_5eed_d00d);
        let _ = wave_rng.next_u32();
        Sim {
            t: 0.0,
            phase: 0.0,
            loop_n: 0,
            ship: Ship {
                x: SHIP_START.0,
                y: SHIP_START.1,
                tx: SHIP_START.0,
                ty: SHIP_START.1,
                vx: 0.0,
                hp: LIVES,
                invuln: 0.0,
                shield: false,
                spread: 0,
                spread_t: 0.0,
                magnet_t: 0.0,
                fire_cd: 0.05,
                side: 1.0,
                danger: 0.0,
                happy: 0.0,
                spin: 0.0,
            },
            grab: None,
            shots: Vec::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            drops: Vec::new(),
            boss: None,
            banner: None,
            score: 0,
            kills: 0,
            combo: 0,
            combo_left: 0.0,
            best_combo: 0,
            grazes: 0,
            dying: None,
            killer: None,
            over: false,
            demo: false,
            cues: Vec::new(),
            pending: Vec::new(),
            next_wave: 0.0,
            opening: 0,
            intros_done: 0,
            last_nasty: false,
            warned: false,
            seen: [false; 5],
            kills_since_power: 0,
            last_power: None,
            gem_streak: 0,
            gem_streak_t: 0.0,
            death_booms: 0,
            wave_rng,
            rng,
        }
    }

    /// A sim for the title card: the same game, but hits only flash.
    pub fn demo(seed: u64) -> Sim {
        Sim { demo: true, ..Sim::new(seed) }
    }

    // ---- Input --------------------------------------------------------

    /// The thumb touches down anywhere: steering starts from here. The ship
    /// doesn't jump to the thumb; it moves by how far the thumb moves.
    pub fn touch_down(&mut self, x: f32, y: f32) {
        if self.dying.is_some() || self.over {
            return;
        }
        self.grab = Some(Grab { fx: x, fy: y, sx: self.ship.tx, sy: self.ship.ty, x, y });
    }

    pub fn touch_move(&mut self, x: f32, y: f32) {
        if let Some(g) = &mut self.grab {
            g.x = x;
            g.y = y;
        }
    }

    pub fn touch_up(&mut self) {
        self.grab = None;
    }

    /// Where the thumb would have to be for the ship to head to (x, y), if
    /// it's touching (the bot steers with this).
    pub fn finger_for(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        self.grab.map(|g| (g.fx + (x - g.sx) / DRAG_GAIN, g.fy + (y - g.sy) / DRAG_GAIN))
    }

    // ---- Derived state --------------------------------------------------

    /// Difficulty multiplier: 1 at the start, +`HEAT_GAIN` every `HEAT_STEP` seconds.
    pub fn heat(&self) -> f32 {
        (1.0 + HEAT_GAIN * (self.t / HEAT_STEP).floor() + 0.15 * self.loop_n as f32).min(HEAT_MAX)
    }

    /// The difficulty level shown to the player (1, 2, 3…).
    pub fn level(&self) -> u32 {
        (self.t / HEAT_STEP) as u32 + 1
    }

    /// Score multiplier from the combo: ×1, ×2 at 5 kills, ×3 at 10… up to ×8.
    pub fn multiplier(&self) -> u32 {
        (1 + self.combo / 5).min(8)
    }

    /// 0…1 for the music's tension layer: the boss, bullets on screen, low health.
    pub fn intensity(&self) -> f32 {
        if self.dying.is_some() || self.over {
            return 0.0;
        }
        let boss: f32 = if self.boss.is_some() || self.warned && self.phase > BOSS_AT - WARNING_TIME { 1.0 } else { 0.0 };
        let bullets = (self.bullets.len() as f32 / 30.0).min(1.0) * 0.6;
        let low = if self.ship.hp == 1 { 0.7 } else { 0.0 };
        let heat = ((self.heat() - 1.0) / 0.6).clamp(0.0, 0.5);
        boss.max(bullets).max(low).max(heat)
    }

    pub fn alive(&self) -> bool {
        self.dying.is_none() && !self.over
    }

    // ---- The step -------------------------------------------------------

    pub fn step(&mut self, dt: f32) {
        if self.over {
            // The aftermath under the results card: the world drifts on,
            // quietly, and Pip's wreck tumbles (draw.rs floats it into view).
            self.ship.spin += dt * 1.1;
            self.move_world(dt, false);
            self.banner_age(dt);
            return;
        }
        self.t += dt;
        let wdt = if let Some(d) = &mut self.dying {
            *d += dt;
            dt * DEATH_SLOWMO
        } else {
            dt
        };
        if self.dying.is_some() {
            self.dying_step(dt);
        } else {
            self.steer(dt);
            self.fire(dt);
        }
        self.schedule(wdt);
        self.move_world(wdt, self.alive());
        self.boss_step(wdt);
        self.collide_shots();
        if self.alive() {
            self.collide_ship();
            self.pickups(wdt);
        }
        self.timers(wdt);
        self.banner_age(dt);
    }

    fn banner_age(&mut self, dt: f32) {
        if let Some(b) = &mut self.banner {
            b.age += dt;
            if b.age > 2.2 {
                self.banner = None;
            }
        }
    }

    fn show(&mut self, kind: BannerKind) {
        self.banner = Some(Banner { kind, age: 0.0 });
    }

    fn steer(&mut self, dt: f32) {
        let s = &mut self.ship;
        if let Some(g) = &mut self.grab {
            let mut tx = g.sx + (g.x - g.fx) * DRAG_GAIN;
            let mut ty = g.sy + (g.y - g.fy) * DRAG_GAIN;
            // Ease the ship up when the thumb is right under it.
            let gap = g.y - ty;
            if gap < MIN_LIFT && (g.x - tx).abs() < 70.0 {
                let lift = (MIN_LIFT - gap) * (1.0 - (-8.0 * dt).exp());
                g.sy -= lift;
                ty -= lift;
            }
            // At an edge, move the grab with the thumb so coming back responds at once.
            let cx = tx.clamp(SHIP_X.0, SHIP_X.1);
            // On the title card the autopilot stays above the card's DAILY button.
            let cy = ty.clamp(SHIP_Y.0, if self.demo { DEMO_Y_MAX } else { SHIP_Y.1 });
            g.sx += cx - tx;
            g.sy += cy - ty;
            tx = cx;
            ty = cy;
            s.tx = tx;
            s.ty = ty;
        }
        let k = 1.0 - (-FOLLOW_RATE * dt).exp();
        let nx = s.x + (s.tx - s.x) * k;
        s.vx = if dt > 0.0 { (nx - s.x) / dt } else { 0.0 };
        s.x = nx;
        s.y += (s.ty - s.y) * k;
    }

    fn fire(&mut self, dt: f32) {
        let s = &mut self.ship;
        s.fire_cd -= dt;
        while s.fire_cd <= 0.0 {
            s.fire_cd += FIRE_INTERVAL;
            let angles: &[f32] = match s.spread {
                0 => &[0.0],
                1 => &[-0.17, 0.0, 0.17],
                _ => &[-0.32, -0.16, 0.0, 0.16, 0.32],
            };
            s.side = -s.side;
            let ox = if s.spread == 0 { s.side * 5.0 } else { 0.0 };
            for &a in angles {
                self.shots.push(Shot { x: s.x + ox + a * 30.0, y: s.y - 18.0, vx: a.sin() * SHOT_SPEED, vy: -a.cos() * SHOT_SPEED });
            }
            self.cues.push(Cue::Shot);
        }
    }

    /// Waves, intros, the boss's warning and arrival.
    fn schedule(&mut self, dt: f32) {
        self.phase += dt;
        // Pending spawns from waves already started.
        let hp_k = 1.0 + 0.3 * self.loop_n as f32;
        let mut i = 0;
        while i < self.pending.len() {
            self.pending[i].delay -= dt;
            if self.pending[i].delay <= 0.0 {
                let s = self.pending.remove(i);
                let e = Enemy::new(&s, hp_k, &mut self.rng);
                self.enemies.push(e);
            } else {
                i += 1;
            }
        }
        if !self.alive() {
            return;
        }
        // The boss.
        if self.boss.is_none() && !self.warned && self.phase >= BOSS_AT - WARNING_TIME {
            self.warned = true;
            self.show(BannerKind::Warning);
            self.cues.push(Cue::Warning);
        }
        if self.boss.is_none() && self.warned && self.phase >= BOSS_AT {
            let hp = BOSS_HP * (1.0 + 0.5 * self.loop_n as f32);
            self.boss = Some(Boss {
                x: W / 2.0,
                y: -90.0,
                hp,
                max_hp: hp,
                age: 0.0,
                attack: Attack::Bloom,
                attack_t: 0.0,
                shot_t: 0.0,
                spin: 0.0,
                flash: 0.0,
                dying: None,
                booms: 0,
                hand: false,
            });
            self.cues.push(Cue::BossArrive);
        }
        if self.warned {
            // Waves pause from the warning until the boss is beaten, except
            // a few jellies now and then to keep the combo going.
            if self.boss.as_ref().is_some_and(|b| !b.entering() && b.dying.is_none()) && self.phase >= self.next_wave {
                self.next_wave = self.phase + 7.0;
                self.start_wave(Formation::Row);
            }
            return;
        }
        // The opening (fixed), then the intros of new enemy types, then random waves.
        if self.loop_n == 0 && self.opening < waves::OPENING.len() {
            let (at, f) = waves::OPENING[self.opening];
            if self.phase >= at {
                self.opening += 1;
                self.start_wave(f);
                self.next_wave = self.phase + WAVE_GAP;
            }
            return;
        }
        if let Some(&(l, at, f, kind)) = waves::INTROS.get(self.intros_done)
            && self.loop_n >= l
            && (self.loop_n > l || self.phase >= at)
        {
            self.intros_done += 1;
            if !self.seen[kind.index()] {
                self.start_wave(f);
                self.next_wave = self.phase + WAVE_GAP * 1.2;
                return;
            }
        }
        if self.phase >= self.next_wave {
            let pool = waves::pool(self.loop_n, self.phase);
            let mut f = pool[(self.wave_rng.next_u32() as usize) % pool.len()];
            if self.last_nasty && f.nasty() {
                f = Formation::Row;
            }
            self.last_nasty = f.nasty();
            self.start_wave(f);
            let gap = (WAVE_GAP / self.heat().powf(0.8)).max(1.5);
            let gap = if matches!(f, Formation::Bulb | Formation::BulbPair | Formation::Spinner) { gap * 1.4 } else { gap };
            self.next_wave = self.phase + gap;
        }
    }

    fn start_wave(&mut self, f: Formation) {
        for s in waves::build(f, &mut self.wave_rng) {
            if !self.seen[s.kind.index()] {
                self.seen[s.kind.index()] = true;
                if s.kind != Kind::Jelly {
                    self.cues.push(Cue::NewEnemy(s.kind));
                    self.show(BannerKind::New(s.kind));
                }
            }
            self.pending.push(s);
        }
    }

    fn move_world(&mut self, dt: f32, can_fire: bool) {
        let heat = self.heat();
        let speed_k = heat.sqrt();
        let (sx, sy) = (self.ship.x, self.ship.y);
        let mut fired: Vec<(f32, f32, Kind, f32)> = Vec::new();
        for e in &mut self.enemies {
            let (px, py) = (e.x, e.y);
            e.age += dt;
            e.flash = (e.flash - dt * 6.0).max(0.0);
            match e.path {
                Path::Drift { vy, sway } => {
                    e.y += vy * speed_k * dt;
                    e.x = e.x0 + sway * (e.age * 1.7 + e.x0 * 0.05).sin();
                }
                Path::Swoop { vx, vy, amp, freq } => {
                    e.x = e.x0 + vx * speed_k * e.age;
                    e.y = e.y0 + vy * e.age + amp * (e.age * freq).sin();
                }
                Path::Hover { stop_y, hold } => {
                    if e.age < hold + 1.4 {
                        e.y += (stop_y - e.y) * (1.0 - (-2.6 * dt).exp());
                        e.x = e.x0 + 22.0 * (e.age * 0.9).sin();
                    } else {
                        // Done: leave upward, out of the way.
                        e.y -= 70.0 * (e.age - hold - 1.4).min(1.5) * dt * 2.0;
                        e.fire_cd = e.fire_cd.max(1.0);
                    }
                }
                Path::Dive { stop_y } => match e.stage {
                    0 => {
                        e.y += (stop_y - e.y) * (1.0 - (-3.0 * dt).exp());
                        if e.age > 1.1 {
                            e.stage = 1;
                            e.age = 0.0;
                        }
                    }
                    1 => {
                        e.aim = (sx, sy);
                        if e.age > 0.55 {
                            e.stage = 2;
                            e.age = 0.0;
                            self.cues.push(Cue::DarterLock { x: e.x, y: e.y });
                        }
                    }
                    2 => {
                        if e.age > 0.4 {
                            e.stage = 3;
                            e.age = 0.0;
                            let (dx, dy) = (e.aim.0 - e.x, e.aim.1 - e.y);
                            let d = (dx * dx + dy * dy).sqrt().max(1.0);
                            let v = 460.0 * speed_k;
                            e.vx = dx / d * v;
                            e.vy = dy / d * v;
                        }
                    }
                    _ => {
                        e.x += e.vx * dt;
                        e.y += e.vy * dt;
                    }
                },
            }
            if !(matches!(e.path, Path::Dive { .. }) && e.stage == 3) && dt > 0.0 {
                e.vx = (e.x - px) / dt;
                e.vy = (e.y - py) / dt;
            }
            // Firing, with a telegraph glow for `TELL` seconds first.
            let may = can_fire && fires(e.kind) && e.y > 10.0 && e.y < FIRE_LINE.min(sy - FIRE_CLEARANCE) && e.x > 8.0 && e.x < W - 8.0;
            let jelly_ok = e.kind != Kind::Jelly || self.t > JELLY_FIRE_AT || self.loop_n > 0;
            if may && jelly_ok && self.t > GRACE {
                e.fire_cd -= dt;
                if e.fire_cd <= 0.0 {
                    fired.push((e.x, e.y, e.kind, e.spin));
                    e.fire_cd = match e.kind {
                        Kind::Jelly => 4.0,
                        Kind::Swooper => 1.6,
                        Kind::Bulb => 1.7,
                        Kind::Spinner => 0.12,
                        Kind::Darter => 99.0,
                    } / heat.powf(1.4);
                    if e.kind == Kind::Spinner {
                        e.spin += 0.42;
                        // Bursts of 14 shots, then a breather.
                        if (e.spin / 0.42) as u32 % 14 == 13 {
                            e.fire_cd = 1.1 / heat;
                        }
                    }
                }
            }
        }
        for (x, y, kind, spin) in fired {
            self.enemy_fire(x, y, kind, spin);
        }
        self.enemies.retain(|e| e.y < H + 50.0 && e.y > -80.0 && e.x > -60.0 && e.x < W + 60.0);
        for s in &mut self.shots {
            s.x += s.vx * dt;
            s.y += s.vy * dt;
        }
        self.shots.retain(|s| s.y > -20.0 && s.x > -20.0 && s.x < W + 20.0);
        for b in &mut self.bullets {
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.age += dt;
        }
        self.bullets.retain(|b| b.y < H + 20.0 && b.y > -40.0 && b.x > -20.0 && b.x < W + 20.0);
        let magnet = self.ship.magnet_t > 0.0 && self.alive();
        for d in &mut self.drops {
            d.age += dt;
            let (dx, dy) = (sx - d.x, sy - d.y);
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            if magnet && dist < 170.0 {
                let pull = 620.0;
                d.vx += (dx / dist * pull - d.vx) * (1.0 - (-8.0 * dt).exp());
                d.vy += (dy / dist * pull - d.vy) * (1.0 - (-8.0 * dt).exp());
            } else {
                // Pop out, then sink gently with a little sway.
                let fall = if d.kind == DropKind::Gem { 70.0 } else { 55.0 };
                d.vx *= (-3.0 * dt).exp();
                d.vy += (fall - d.vy) * (1.0 - (-2.5 * dt).exp());
            }
            d.x += d.vx * dt;
            d.y += d.vy * dt;
            d.x = d.x.clamp(10.0, W - 10.0);
        }
        self.drops.retain(|d| d.y < H + 20.0);
    }

    fn aimed(&self, x: f32, y: f32) -> f32 {
        (self.ship.y - y).atan2(self.ship.x - x)
    }

    fn bullet(&mut self, x: f32, y: f32, angle: f32, speed: f32, big: bool) {
        let r = if big { 8.0 } else { 6.0 };
        self.bullets.push(Bullet { x, y, vx: angle.cos() * speed, vy: angle.sin() * speed, r, big, grazed: false, age: 0.0 });
    }

    fn enemy_fire(&mut self, x: f32, y: f32, kind: Kind, spin: f32) {
        let v = BULLET_SPEED * (0.75 + 0.25 * self.heat());
        let a = self.aimed(x, y);
        match kind {
            Kind::Jelly => self.bullet(x, y + 10.0, a, v * 0.8, false),
            Kind::Swooper => self.bullet(x, y + 10.0, a, v, false),
            Kind::Bulb => {
                let n = if self.loop_n == 0 { 3 } else { 5 };
                for i in 0..n {
                    let k = i as f32 - (n - 1) as f32 / 2.0;
                    self.bullet(x, y + 18.0, a + k * 0.24, v * 0.85, true);
                }
            }
            Kind::Spinner => {
                for arm in 0..2 {
                    self.bullet(x, y, spin + arm as f32 * std::f32::consts::PI, v * 0.8, false);
                }
            }
            Kind::Darter => {}
        }
        self.cues.push(Cue::EnemyFire { big: matches!(kind, Kind::Bulb | Kind::Spinner) });
    }

    fn boss_step(&mut self, dt: f32) {
        let Some(mut b) = self.boss else { return };
        b.age += dt;
        b.flash = (b.flash - dt * 6.0).max(0.0);
        if let Some(d) = &mut b.dying {
            *d += dt;
            let d = *d;
            b.x += (d * 40.0).sin() * 2.0;
            // A chain of explosions across its body, then the big one.
            let due = (d / 0.18) as u32;
            while b.booms < due.min(8) {
                b.booms += 1;
                let (ox, oy) = (self.rng.range(-BOSS_RX, BOSS_RX), self.rng.range(-BOSS_RY, BOSS_RY));
                self.cues.push(Cue::Boom { x: b.x + ox, y: b.y + oy, size: 0.5 });
            }
            if d >= BOSS_DEATH {
                self.boss = None;
                self.boss_down(b.x, b.y);
                return;
            }
            self.boss = Some(b);
            return;
        }
        if b.entering() {
            b.y += (BOSS_Y - b.y) * (1.0 - (-2.2 * dt).exp());
            self.boss = Some(b);
            return;
        }
        b.y += (BOSS_Y - b.y) * (1.0 - (-2.2 * dt).exp());
        let sway = if b.attack == Attack::Sweep { 95.0 } else { 55.0 };
        b.x = W / 2.0 + sway * (b.age * 0.7).sin();
        let speed = if b.enraged() { 1.35 } else { 1.0 } * (1.0 + 0.2 * self.loop_n as f32);
        let prev = b.attack_t;
        b.attack_t += dt;
        if prev == 0.0 {
            self.cues.push(Cue::BossCharge(b.attack));
        }
        if b.attack_t > CHARGE && b.attack_t < CHARGE + ATTACK_FIRE && self.alive() {
            b.shot_t -= dt * speed;
            let v = BULLET_SPEED * (0.75 + 0.25 * self.heat());
            while b.shot_t <= 0.0 {
                match b.attack {
                    Attack::Bloom => {
                        b.shot_t += 0.95;
                        let n = 16 + 2 * self.loop_n.min(4) as usize;
                        b.spin += std::f32::consts::PI / n as f32;
                        for i in 0..n {
                            let a = b.spin + i as f32 * std::f32::consts::TAU / n as f32;
                            if a.sin() > -0.5 {
                                self.bullet(b.x, b.y + 10.0, a, v * 0.85, true);
                            }
                        }
                    }
                    Attack::Rain => {
                        b.shot_t += 0.5;
                        b.hand = !b.hand;
                        let (hx, hy) = (b.x + if b.hand { BOSS_HAND.0 } else { -BOSS_HAND.0 }, b.y + BOSS_HAND.1);
                        let a = self.aimed(hx, hy);
                        for k in [-1.0, 0.0, 1.0] {
                            self.bullet(hx, hy, a + k * 0.2, v * 1.15, false);
                        }
                    }
                    Attack::Sweep => {
                        b.shot_t += 0.1;
                        b.spin += 0.29;
                        for arm in 0..2 {
                            let a = b.spin + arm as f32 * std::f32::consts::PI;
                            self.bullet(b.x, b.y + 10.0, a, v * 0.8, false);
                        }
                    }
                }
                self.cues.push(Cue::EnemyFire { big: true });
            }
        }
        if b.attack_t > CHARGE + ATTACK_FIRE + ATTACK_REST {
            b.attack_t = 0.0;
            b.shot_t = 0.0;
            b.attack = match b.attack {
                Attack::Bloom => Attack::Rain,
                Attack::Rain => Attack::Sweep,
                Attack::Sweep => Attack::Bloom,
            };
        }
        self.boss = Some(b);
    }

    fn boss_down(&mut self, x: f32, y: f32) {
        let points = 5000 * (self.loop_n as u64 + 1);
        self.add_score(points);
        self.cues.push(Cue::BossDown { x, y, points });
        self.cues.push(Cue::Boom { x, y, size: 1.0 });
        self.drops.push(Drop { kind: DropKind::Heart, x, y, vx: 0.0, vy: -60.0, age: 0.0 });
        for i in 0..14 {
            let a = i as f32 / 14.0 * std::f32::consts::TAU;
            self.drops.push(Drop { kind: DropKind::Gem, x, y, vx: a.cos() * 160.0, vy: a.sin() * 160.0 - 40.0, age: 0.0 });
        }
        self.loop_n += 1;
        self.phase = 0.0;
        self.next_wave = 3.0;
        self.warned = false;
        self.last_nasty = false;
        self.show(BannerKind::Clear);
    }

    fn collide_shots(&mut self) {
        let mut i = 0;
        while i < self.shots.len() {
            let s = self.shots[i];
            let mut hit = false;
            for j in 0..self.enemies.len() {
                let e = &mut self.enemies[j];
                let r = e.kind.radius() + 5.0;
                if (s.x - e.x).powi(2) + (s.y - e.y).powi(2) < r * r && e.y > -10.0 {
                    hit = true;
                    e.hp -= 1.0;
                    e.flash = 1.0;
                    if e.hp <= 0.0 {
                        let e = self.enemies.remove(j);
                        self.kill(e);
                    } else {
                        self.cues.push(Cue::Hit { x: s.x, y: s.y });
                    }
                    break;
                }
            }
            let mut beaten = None;
            if !hit
                && let Some(b) = &mut self.boss
                && b.dying.is_none()
                && b.y > 40.0
            {
                let (dx, dy) = ((s.x - b.x) / BOSS_RX, (s.y - b.y) / BOSS_RY);
                if dx * dx + dy * dy < 1.0 {
                    hit = true;
                    let calm = !b.enraged();
                    b.hp -= 1.0;
                    b.flash = 1.0;
                    self.cues.push(Cue::BossHit { x: s.x, y: s.y });
                    if calm && b.enraged() && b.hp > 0.0 {
                        self.cues.push(Cue::BossEnrage);
                    }
                    if b.hp <= 0.0 {
                        b.dying = Some(0.0);
                        beaten = Some((b.x, b.y));
                    }
                }
            }
            if let Some((x, y)) = beaten {
                self.cues.push(Cue::BossBeaten { x, y });
                // Every bullet on screen turns into a gem, and every enemy pops.
                for bl in std::mem::take(&mut self.bullets) {
                    self.drops.push(Drop { kind: DropKind::Gem, x: bl.x, y: bl.y, vx: 0.0, vy: -30.0, age: 0.0 });
                }
                for e in std::mem::take(&mut self.enemies) {
                    self.kill(e);
                }
            }
            if hit {
                self.shots.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn add_score(&mut self, points: u64) {
        if !self.demo {
            self.score += points;
        }
    }

    fn kill(&mut self, e: Enemy) {
        self.kills += 1;
        self.combo += 1;
        self.combo_left = COMBO_WINDOW;
        self.best_combo = self.best_combo.max(self.combo);
        let mult = self.multiplier();
        let points = e.kind.points() * mult as u64;
        self.add_score(points);
        self.cues.push(Cue::Kill { x: e.x, y: e.y, kind: e.kind, points, mult, combo: self.combo });
        for k in 0..e.kind.gems() {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32 - (e.kind.gems() - 1) as f32 / 2.0) * 0.7 + self.rng.range(-0.3, 0.3);
            let sp = self.rng.range(70.0, 130.0);
            self.drops.push(Drop { kind: DropKind::Gem, x: e.x, y: e.y, vx: a.cos() * sp, vy: a.sin() * sp, age: 0.0 });
        }
        self.kills_since_power += 1;
        let bonus = e.kind == Kind::Bulb && self.rng.f32() < 0.6;
        if self.kills_since_power >= POWER_EVERY || bonus {
            self.kills_since_power = 0;
            let choices: Vec<Power> = [Power::Spread, Power::Shield, Power::Magnet].into_iter().filter(|p| Some(*p) != self.last_power).collect();
            let p = choices[(self.rng.next_u32() as usize) % choices.len()];
            self.last_power = Some(p);
            self.drops.push(Drop { kind: DropKind::Power(p), x: e.x, y: e.y, vx: 0.0, vy: -50.0, age: 0.0 });
        }
    }

    fn collide_ship(&mut self) {
        if self.ship.invuln > 0.0 {
            return; // blinking: bullets pass through
        }
        let (sx, sy) = (self.ship.x, self.ship.y);
        let mut hit_at = None;
        let mut grazes = Vec::new();
        for b in &mut self.bullets {
            let d = ((b.x - sx).powi(2) + (b.y - sy).powi(2)).sqrt();
            if d < b.r + SHIP_R && hit_at.is_none() {
                hit_at = Some((b.x, b.y));
                b.age = -1.0; // marked: removed below
            } else if !b.grazed && d < b.r + SHIP_R + GRAZE {
                b.grazed = true;
                grazes.push((b.x, b.y));
            }
        }
        if hit_at.is_some() {
            self.bullets.retain(|b| b.age >= 0.0);
        }
        // Ramming an enemy hurts both (a darter dies of it).
        let mut rammed = None;
        if hit_at.is_none() {
            for (i, e) in self.enemies.iter_mut().enumerate() {
                let r = e.kind.radius() * 0.8 + SHIP_R;
                if (e.x - sx).powi(2) + (e.y - sy).powi(2) < r * r {
                    hit_at = Some((e.x, e.y));
                    e.hp -= 3.0;
                    e.flash = 1.0;
                    rammed = Some(i);
                    break;
                }
            }
        }
        if let Some(i) = rammed
            && self.enemies[i].hp <= 0.0
        {
            let e = self.enemies.remove(i);
            self.kill(e);
        }
        if let Some(b) = &self.boss
            && hit_at.is_none()
            && b.dying.is_none()
        {
            let (dx, dy) = ((sx - b.x) / (BOSS_RX + SHIP_R), (sy - b.y) / (BOSS_RY + SHIP_R));
            if dx * dx + dy * dy < 1.0 {
                hit_at = Some((sx, sy - 10.0));
            }
        }
        for (x, y) in grazes {
            self.grazes += 1;
            self.add_score(GRAZE_VALUE);
            self.cues.push(Cue::Graze { x, y });
        }
        if let Some((x, y)) = hit_at {
            self.hurt(x, y);
        }
    }

    fn hurt(&mut self, x: f32, y: f32) {
        // Mercy: clear the bullets right around the ship.
        let (sx, sy) = (self.ship.x, self.ship.y);
        self.bullets.retain(|b| (b.x - sx).powi(2) + (b.y - sy).powi(2) > 80.0 * 80.0);
        if self.ship.shield {
            self.ship.shield = false;
            self.ship.invuln = 1.0;
            self.cues.push(Cue::ShieldBreak { x: sx, y: sy });
            return;
        }
        if self.combo >= 5 {
            self.cues.push(Cue::ComboEnd { count: self.combo });
        }
        self.combo = 0;
        self.combo_left = 0.0;
        if self.demo {
            self.ship.invuln = INVULN_TIME;
            self.cues.push(Cue::Hurt { x, y, hp: self.ship.hp });
            return;
        }
        self.ship.hp = self.ship.hp.saturating_sub(1);
        self.ship.spread = 0;
        self.ship.spread_t = 0.0;
        self.cues.push(Cue::Hurt { x, y, hp: self.ship.hp });
        if self.ship.hp == 0 {
            self.dying = Some(0.0);
            self.killer = Some((x, y));
            self.grab = None;
            self.cues.push(Cue::Dying { x: sx, y: sy });
        } else {
            self.ship.invuln = INVULN_TIME;
        }
    }

    fn dying_step(&mut self, dt: f32) {
        let d = self.dying.unwrap_or(0.0);
        self.ship.spin += dt * (4.0 + d * 6.0);
        self.ship.y += 26.0 * dt;
        // Little pops on the ship, then the big one just before the card.
        let due = ((d / 0.28) as u32).min(5);
        while self.death_booms < due {
            self.death_booms += 1;
            let (ox, oy) = (self.rng.range(-14.0, 14.0), self.rng.range(-14.0, 14.0));
            self.cues.push(Cue::Boom { x: self.ship.x + ox, y: self.ship.y + oy, size: 0.35 });
        }
        if d >= DEATH_TIME - 0.15 && self.death_booms < 6 {
            self.death_booms = 6;
            self.cues.push(Cue::Boom { x: self.ship.x, y: self.ship.y, size: 1.0 });
        }
        if d >= DEATH_TIME {
            self.over = true;
            self.cues.push(Cue::Over);
        }
    }

    fn pickups(&mut self, dt: f32) {
        let (sx, sy) = (self.ship.x, self.ship.y);
        self.gem_streak_t -= dt;
        if self.gem_streak_t <= 0.0 {
            self.gem_streak = 0;
        }
        let mut i = 0;
        while i < self.drops.len() {
            let d = self.drops[i];
            let reach = if d.kind == DropKind::Gem { 26.0 } else { 30.0 };
            if d.age > 0.25 && (d.x - sx).powi(2) + (d.y - sy).powi(2) < reach * reach {
                self.drops.swap_remove(i);
                self.collect(d);
            } else {
                i += 1;
            }
        }
    }

    fn collect(&mut self, d: Drop) {
        let s = &mut self.ship;
        match d.kind {
            DropKind::Gem => {
                self.gem_streak += 1;
                self.gem_streak_t = 0.6;
                self.add_score(GEM_VALUE);
                self.cues.push(Cue::Gem { x: d.x, y: d.y, streak: self.gem_streak });
            }
            DropKind::Power(p) => {
                s.happy = 1.2;
                match p {
                    Power::Spread => {
                        s.spread = (s.spread + 1).min(2);
                        s.spread_t = POWER_TIME;
                    }
                    Power::Shield => s.shield = true,
                    Power::Magnet => s.magnet_t = POWER_TIME,
                }
                self.add_score(200);
                self.cues.push(Cue::Power { x: d.x, y: d.y, power: p });
            }
            DropKind::Heart => {
                s.happy = 1.5;
                if s.hp < LIVES {
                    s.hp += 1;
                } else {
                    self.add_score(1000);
                }
                self.cues.push(Cue::Heart { x: d.x, y: d.y });
            }
        }
    }

    fn timers(&mut self, dt: f32) {
        let s = &mut self.ship;
        s.invuln = (s.invuln - dt).max(0.0);
        s.happy = (s.happy - dt).max(0.0);
        if s.spread_t > 0.0 {
            s.spread_t -= dt;
            if s.spread_t <= 0.0 {
                s.spread = 0;
            }
        }
        s.magnet_t = (s.magnet_t - dt).max(0.0);
        if self.combo > 0 {
            self.combo_left -= dt;
            if self.combo_left <= 0.0 {
                if self.combo >= 5 {
                    self.cues.push(Cue::ComboEnd { count: self.combo });
                }
                self.combo = 0;
            }
        }
        // How worried the face is: bullets close by.
        let mut near = 0.0f32;
        for b in &self.bullets {
            let d = ((b.x - s.x).powi(2) + (b.y - s.y).powi(2)).sqrt();
            near += (1.0 - d / 90.0).max(0.0);
        }
        let target = near.min(1.0);
        s.danger += (target - s.danger) * (1.0 - (-10.0 * dt).exp());
    }

    // ---- Test and debug helpers ------------------------------------------

    /// Skips ahead to `phase` seconds into the current loop (debugging the boss).
    pub fn skip_to(&mut self, phase: f32) {
        self.phase = phase;
        self.t = phase;
        self.opening = waves::OPENING.len();
        self.intros_done = waves::INTROS.iter().filter(|i| i.0 == 0 && i.1 <= phase).count();
        self.next_wave = phase;
    }

    /// Stops new waves (tests that set up their own enemies).
    pub fn hold_waves(&mut self) {
        self.opening = waves::OPENING.len();
        self.intros_done = waves::INTROS.len();
        self.next_wave = f32::MAX;
        self.pending.clear();
    }

    /// Puts an enemy bullet at (x, y) moving at (vx, vy) (tests).
    pub fn add_bullet(&mut self, x: f32, y: f32, vx: f32, vy: f32) {
        self.bullets.push(Bullet { x, y, vx, vy, r: 6.0, big: false, grazed: false, age: 0.0 });
    }

    /// Puts an enemy at (x, y) that just drifts (tests).
    pub fn add_enemy(&mut self, kind: Kind, x: f32, y: f32) {
        let s = Spawn { delay: 0.0, kind, x, y, path: Path::Drift { vy: 0.0, sway: 0.0 } };
        let e = Enemy::new(&s, 1.0, &mut self.rng);
        self.enemies.push(e);
    }

    pub fn add_drop(&mut self, kind: DropKind, x: f32, y: f32) {
        self.drops.push(Drop { kind, x, y, vx: 0.0, vy: 0.0, age: 1.0 });
    }
}
