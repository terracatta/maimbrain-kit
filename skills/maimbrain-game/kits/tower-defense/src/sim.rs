//! The rules, with no host calls: `cargo test -p kit_tower_defense` runs them
//! natively. Jellies march down a winding road toward the cake; towers on
//! build pads beside the road pop them for coins; every jelly that reaches
//! the cake takes a bite (a heart). Out of hearts, the cake is devoured.
//!
//! The sim emits [`Cue`]s (fired, popped, bitten, wave started…) that the
//! host turns into sound, haptics and juice. It never draws or plays.

use maimbrain::Rng;

// ---- Tuning knobs -----------------------------------------------------
/// Coins at the start of a round: enough for exactly one POP or CHILL
/// tower (BOOM costs more). Raise it and the opening gets easier.
pub const START_COINS: u32 = 60;
/// Hearts (cake slices). Each jelly that reaches the cake bites one off (a
/// king bites `KING_BITE`). Raise for longer, more forgiving rounds.
pub const HEARTS: u32 = 5;
/// Seconds between wave starts once the first wave has begun. The main
/// pacing knob: lower means waves overlap sooner and rounds end faster.
pub const WAVE_GAP: f32 = 10.0;
/// Wave 1 waits for the first tower (then starts `FIRST_WAVE_DELAY` later),
/// or starts by itself after this many seconds.
pub const FIRST_WAVE_WAIT: f32 = 9.0;
pub const FIRST_WAVE_DELAY: f32 = 1.5;
/// Coins paid at the start of each wave: `PAYDAY + PAYDAY_GROWTH × wave`.
pub const PAYDAY: f32 = 8.0;
pub const PAYDAY_GROWTH: f32 = 2.0;
/// Calling the next wave early pays this many coins per second skipped.
pub const EARLY_BONUS_PER_SEC: f32 = 2.5;
/// Jelly hit points are multiplied by `HP_BASE × HP_GROWTH^(wave − 1)`.
/// HP_GROWTH is the difficulty curve: 1.10 is gentle, 1.18 is brutal.
pub const HP_BASE: f32 = 0.8;
/// Wave 1's jellies are this much softer still: one POP on any pad pops
/// them all (tested), so the first wave is always a win.
pub const FIRST_WAVE_HP: f32 = 0.6;
pub const HP_GROWTH: f32 = 1.2;
/// Jellies also speed up a little each wave (fraction per wave, capped).
pub const SPEED_GROWTH: f32 = 0.012;
pub const SPEED_GROWTH_MAX: f32 = 0.3;
/// A king (boss) leads every 5th wave from this wave on.
pub const FIRST_BOSS_WAVE: u32 = 10;
/// Rush waves (8, 13, 18…) pack their jellies this much tighter.
pub const RUSH_SPACING: f32 = 0.55;
/// Seconds the devouring plays before the round is over (still Playing).
pub const ENDING: f32 = 1.9;

/// One tower kind's numbers, per level (index 0 = as built, 2 = maxed).
pub struct TowerSpec {
    pub name: &'static str,
    /// [build, upgrade to level 2, upgrade to level 3].
    pub cost: [u32; 3],
    pub range: [f32; 3],
    /// Seconds between shots (or pulses).
    pub interval: [f32; 3],
    pub damage: [f32; 3],
    /// BOOM: splash radius. CHILL: the slow factor (speed multiplier).
    pub special: [f32; 3],
}

/// POP: fast single-target gumball shooter. CHILL: a frost pulse that slows
/// everything in range. BOOM: a lobbed cherry bomb with splash damage.
pub const TOWERS: [TowerSpec; 3] = [
    TowerSpec { name: "POP", cost: [50, 40, 70], range: [88.0, 96.0, 106.0], interval: [0.5, 0.38, 0.28], damage: [1.0, 1.5, 2.2], special: [0.0; 3] },
    TowerSpec { name: "CHILL", cost: [60, 45, 75], range: [72.0, 82.0, 92.0], interval: [1.3, 1.15, 1.0], damage: [0.4, 0.7, 1.1], special: [0.55, 0.45, 0.35] },
    TowerSpec { name: "BOOM", cost: [80, 60, 95], range: [104.0, 114.0, 126.0], interval: [1.7, 1.45, 1.2], damage: [2.2, 3.4, 5.0], special: [38.0, 46.0, 56.0] },
];
/// POP's gumball speed (units/s) and BOOM's flight time (s).
pub const POP_SHOT_SPEED: f32 = 460.0;
pub const BOOM_FLIGHT: f32 = 0.55;
/// How long a CHILL pulse keeps a jelly slowed (s).
pub const SLOW_TIME: f32 = 1.5;

/// One jelly kind's numbers.
pub struct CreepSpec {
    pub name: &'static str,
    pub hp: f32,
    /// Units per second along the road.
    pub speed: f32,
    /// Subtracted from every hit (a hit always does at least a quarter).
    pub armor: f32,
    pub bounty: u32,
    pub radius: f32,
    /// Seconds between two of these in a wave.
    pub spacing: f32,
}

pub const CREEPS: [CreepSpec; 6] = [
    CreepSpec { name: "GUMMY", hp: 3.0, speed: 42.0, armor: 0.0, bounty: 5, radius: 13.0, spacing: 0.85 },
    CreepSpec { name: "ZIPPER", hp: 2.0, speed: 78.0, armor: 0.0, bounty: 4, radius: 10.0, spacing: 0.45 },
    CreepSpec { name: "HELMET", hp: 5.0, speed: 33.0, armor: 1.0, bounty: 8, radius: 14.0, spacing: 1.1 },
    CreepSpec { name: "SPLITTER", hp: 7.0, speed: 37.0, armor: 0.0, bounty: 5, radius: 17.0, spacing: 1.5 },
    CreepSpec { name: "MINI", hp: 1.5, speed: 58.0, armor: 0.0, bounty: 2, radius: 8.0, spacing: 0.3 },
    CreepSpec { name: "KING", hp: 70.0, speed: 24.0, armor: 0.4, bounty: 50, radius: 24.0, spacing: 2.0 },
];
/// Hearts a king bites off.
pub const KING_BITE: u32 = 3;
/// A splitter pops into this many minis.
pub const SPLIT_INTO: usize = 3;

// ---- The map ------------------------------------------------------------
/// The road, top (off screen) to the cake. Jellies walk it at constant speed.
pub const PATH: [(f32, f32); 8] = [(196.0, -24.0), (196.0, 160.0), (84.0, 160.0), (84.0, 284.0), (276.0, 284.0), (276.0, 404.0), (180.0, 404.0), (180.0, 492.0)];
pub const ROAD_W: f32 = 36.0;
pub const CAKE: (f32, f32) = (180.0, 522.0);
pub const CAKE_R: f32 = 34.0;
/// Build pads beside the road (centres). Each is `PAD` units square.
pub const PADS: [(f32, f32); 11] = [
    (128.0, 100.0),
    (258.0, 124.0),
    (140.0, 222.0),
    (226.0, 222.0),
    (326.0, 300.0),
    (32.0, 222.0),
    (180.0, 344.0),
    (84.0, 352.0),
    (326.0, 410.0),
    (124.0, 448.0),
    (248.0, 470.0),
];
pub const PAD: f32 = 52.0;
/// The pad the first-build hint points at (the one covering the most road).
pub const HINT_PAD: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TowerKind {
    Pop = 0,
    Chill = 1,
    Boom = 2,
}

impl TowerKind {
    pub const ALL: [TowerKind; 3] = [TowerKind::Pop, TowerKind::Chill, TowerKind::Boom];
    pub fn spec(self) -> &'static TowerSpec {
        &TOWERS[self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreepKind {
    Gummy = 0,
    Zipper = 1,
    Helmet = 2,
    Splitter = 3,
    Mini = 4,
    King = 5,
}

impl CreepKind {
    pub fn spec(self) -> &'static CreepSpec {
        &CREEPS[self as usize]
    }
}

#[derive(Clone, Debug)]
pub struct Creep {
    pub id: u32,
    pub kind: CreepKind,
    /// Distance walked along the road.
    pub d: f32,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    /// Seconds of slow left, and the speed multiplier while slowed.
    pub slow: f32,
    pub slow_k: f32,
    /// Hit flash (1 → 0, for drawing).
    pub flash: f32,
    pub age: f32,
}

impl Creep {
    pub fn cur_speed(&self) -> f32 {
        self.speed * if self.slow > 0.0 { self.slow_k } else { 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Tower {
    pub kind: TowerKind,
    /// 0, 1 or 2.
    pub level: usize,
    pub cd: f32,
    /// Where the barrel points (radians, screen space) and the recoil kick
    /// (1 → 0), for drawing.
    pub aim: f32,
    pub recoil: f32,
    /// Seconds since built or upgraded (for a pop-in).
    pub age: f32,
}

impl Tower {
    pub fn range(&self) -> f32 {
        self.kind.spec().range[self.level]
    }
}

#[derive(Clone, Debug)]
pub struct Shot {
    pub kind: TowerKind,
    pub x: f32,
    pub y: f32,
    /// Start (BOOM's arc), target point and the target jelly (POP homes).
    pub sx: f32,
    pub sy: f32,
    pub tx: f32,
    pub ty: f32,
    pub target: Option<u32>,
    pub damage: f32,
    pub splash: f32,
    pub t: f32,
}

impl Shot {
    /// BOOM's height above the ground along its arc (for drawing).
    pub fn lift(&self) -> f32 {
        if self.kind == TowerKind::Boom {
            let k = (self.t / BOOM_FLIGHT).clamp(0.0, 1.0);
            4.0 * k * (1.0 - k) * 70.0
        } else {
            0.0
        }
    }
}

/// What the next wave brings (shown before it comes).
#[derive(Clone, Debug, PartialEq)]
pub struct WavePlan {
    pub n: u32,
    pub groups: Vec<(CreepKind, u32)>,
    pub boss: bool,
    /// A kind making its first appearance this wave.
    pub new_kind: Option<CreepKind>,
    /// A rush: one fast kind, packed tight (every fifth wave from 8, between kings).
    pub rush: bool,
}

/// Things the host shows and plays. The sim never draws or plays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    Built { pad: usize, kind: TowerKind },
    Upgraded { pad: usize, level: usize },
    /// Couldn't afford it.
    Denied,
    Fire { kind: TowerKind, x: f32, y: f32 },
    Frost { x: f32, y: f32, r: f32 },
    Boom { x: f32, y: f32, r: f32 },
    Hit { x: f32, y: f32 },
    /// A jelly popped: coins earned, and how many popped in quick succession.
    Pop { x: f32, y: f32, kind: CreepKind, coins: u32, chain: u32 },
    Split { x: f32, y: f32 },
    /// A jelly reached the cake.
    Bite { x: f32, y: f32, hearts: u32 },
    Wave { n: u32, boss: bool, rush: bool, new_kind: Option<CreepKind> },
    Payday { coins: u32 },
    Early { bonus: u32 },
    /// Out of hearts: the devouring starts (still playing).
    Ending,
    /// A jelly dives into the cake during the devouring.
    Chomp { x: f32, y: f32 },
    Over,
}

pub struct Sim {
    pub t: f32,
    pub coins: u32,
    pub hearts: u32,
    /// Jellies popped: the score.
    pub score: u32,
    /// Waves started so far (the one on screen).
    pub wave: u32,
    /// Seconds until the next wave starts by itself.
    pub next_in: f32,
    pub next: WavePlan,
    pub creeps: Vec<Creep>,
    /// One slot per pad.
    pub towers: Vec<Option<Tower>>,
    pub shots: Vec<Shot>,
    pub cues: Vec<Cue>,
    /// Seconds since the hearts ran out (the devouring), then `over`.
    pub ending: Option<f32>,
    pub over: bool,
    pub built: u32,
    pub upgrades: u32,
    /// Jellies still to come: (seconds until, kind).
    spawns: Vec<(f32, CreepKind)>,
    hp_mult: f32,
    speed_mult: f32,
    chain: u32,
    chain_t: f32,
    next_id: u32,
    rng: Rng,
    path_len: Vec<f32>,
}

impl Sim {
    pub fn new(seed: u64) -> Sim {
        let mut rng = Rng::new(seed);
        let next = plan_wave(1, &mut rng);
        let mut path_len = vec![0.0];
        for w in PATH.windows(2) {
            let l = path_len.last().unwrap() + (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
            path_len.push(l);
        }
        Sim {
            t: 0.0,
            coins: START_COINS,
            hearts: HEARTS,
            score: 0,
            wave: 0,
            next_in: FIRST_WAVE_WAIT,
            next,
            creeps: Vec::new(),
            towers: vec![None; PADS.len()],
            shots: Vec::new(),
            cues: Vec::new(),
            ending: None,
            over: false,
            built: 0,
            upgrades: 0,
            spawns: Vec::new(),
            hp_mult: 1.0,
            speed_mult: 1.0,
            chain: 0,
            chain_t: 0.0,
            next_id: 1,
            rng,
            path_len,
        }
    }

    /// Total length of the road.
    pub fn road_len(&self) -> f32 {
        *self.path_len.last().unwrap()
    }

    /// The point `d` units along the road (clamped to its ends).
    pub fn point_at(&self, d: f32) -> (f32, f32) {
        let d = d.clamp(0.0, self.road_len());
        for i in 0..PATH.len() - 1 {
            let (l0, l1) = (self.path_len[i], self.path_len[i + 1]);
            if d <= l1 || i == PATH.len() - 2 {
                let k = if l1 > l0 { (d - l0) / (l1 - l0) } else { 0.0 };
                let (a, b) = (PATH[i], PATH[i + 1]);
                return (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k);
            }
        }
        PATH[PATH.len() - 1]
    }

    /// The pad under a tap, with a generous touch area.
    pub fn pad_at(x: f32, y: f32) -> Option<usize> {
        let half = PAD / 2.0 + 4.0;
        PADS.iter().position(|&(px, py)| (x - px).abs() <= half && (y - py).abs() <= half)
    }

    pub fn can_afford(&self, cost: u32) -> bool {
        self.coins >= cost
    }

    /// Cost of upgrading the tower on `pad`, None if empty or maxed.
    pub fn upgrade_cost(&self, pad: usize) -> Option<u32> {
        let t = self.towers.get(pad)?.as_ref()?;
        (t.level < 2).then(|| t.kind.spec().cost[t.level + 1])
    }

    pub fn playing(&self) -> bool {
        !self.over && self.ending.is_none()
    }

    /// Builds a tower on an empty pad. False (and a `Denied` cue if it was
    /// the price) when it can't.
    pub fn build(&mut self, pad: usize, kind: TowerKind) -> bool {
        if !self.playing() || pad >= PADS.len() || self.towers[pad].is_some() {
            return false;
        }
        let cost = kind.spec().cost[0];
        if self.coins < cost {
            self.cues.push(Cue::Denied);
            return false;
        }
        self.coins -= cost;
        self.towers[pad] = Some(Tower { kind, level: 0, cd: 0.25, aim: -std::f32::consts::FRAC_PI_2, recoil: 0.0, age: 0.0 });
        self.built += 1;
        self.cues.push(Cue::Built { pad, kind });
        // The first tower calls wave 1 (soon, not instantly: a beat to enjoy it).
        if self.wave == 0 {
            self.next_in = self.next_in.min(FIRST_WAVE_DELAY);
        }
        true
    }

    /// Upgrades the tower on `pad` one level.
    pub fn upgrade(&mut self, pad: usize) -> bool {
        if !self.playing() {
            return false;
        }
        let Some(cost) = self.upgrade_cost(pad) else { return false };
        if self.coins < cost {
            self.cues.push(Cue::Denied);
            return false;
        }
        self.coins -= cost;
        let t = self.towers[pad].as_mut().unwrap();
        t.level += 1;
        t.age = 0.0;
        self.upgrades += 1;
        self.cues.push(Cue::Upgraded { pad, level: t.level });
        true
    }

    /// Coins for calling the next wave now.
    pub fn early_bonus(&self) -> u32 {
        if self.wave == 0 { 0 } else { (self.next_in * EARLY_BONUS_PER_SEC) as u32 }
    }

    /// Starts the next wave now, for a bonus. Returns the bonus paid.
    pub fn call_wave(&mut self) -> Option<u32> {
        if !self.playing() {
            return None;
        }
        let bonus = self.early_bonus();
        self.coins += bonus;
        self.cues.push(Cue::Early { bonus });
        self.start_wave();
        Some(bonus)
    }

    fn start_wave(&mut self) {
        let plan = std::mem::replace(&mut self.next, WavePlan { n: 0, groups: Vec::new(), boss: false, new_kind: None, rush: false });
        self.wave = plan.n;
        let w = (self.wave - 1) as f32;
        self.hp_mult = HP_BASE * HP_GROWTH.powf(w) * if self.wave == 1 { FIRST_WAVE_HP } else { 1.0 };
        self.speed_mult = 1.0 + (SPEED_GROWTH * w).min(SPEED_GROWTH_MAX);
        // Interleave the groups so a wave reads as a mixed parade.
        let mut left: Vec<(CreepKind, u32)> = plan.groups.clone();
        let mut at = 0.0;
        let last_spawn = self.spawns.last().map_or(0.0, |s| s.0);
        let base = last_spawn.max(0.0);
        while left.iter().any(|g| g.1 > 0) {
            for g in left.iter_mut() {
                if g.1 > 0 {
                    self.spawns.push((base + at, g.0));
                    at += g.0.spec().spacing * self.rng.range(0.85, 1.15) * if plan.rush { RUSH_SPACING } else { 1.0 };
                    g.1 -= 1;
                }
            }
        }
        let pay = (PAYDAY + PAYDAY_GROWTH * self.wave as f32) as u32;
        self.coins += pay;
        self.cues.push(Cue::Payday { coins: pay });
        self.cues.push(Cue::Wave { n: self.wave, boss: plan.boss, rush: plan.rush, new_kind: plan.new_kind });
        self.next = plan_wave(self.wave + 1, &mut self.rng);
        self.next_in = WAVE_GAP;
    }

    fn spawn(&mut self, kind: CreepKind, d: f32) {
        let s = kind.spec();
        let hp = s.hp * if kind == CreepKind::Mini { self.hp_mult.sqrt() } else { self.hp_mult };
        let (x, y) = self.point_at(d);
        self.creeps.push(Creep {
            id: self.next_id,
            kind,
            d,
            x,
            y,
            hp,
            max_hp: hp,
            speed: s.speed * self.speed_mult * self.rng.range(0.96, 1.04),
            slow: 0.0,
            slow_k: 1.0,
            flash: 0.0,
            age: 0.0,
        });
        self.next_id += 1;
    }

    /// Hits jelly `i` for `dmg` (armor applies). Pops it if it runs out.
    fn hurt(&mut self, i: usize, dmg: f32) {
        let c = &mut self.creeps[i];
        let armor = c.kind.spec().armor;
        c.hp -= (dmg - armor).max(dmg * 0.25);
        c.flash = 1.0;
    }

    /// Removes jellies at 0 hp: coins, score, cues, splitting.
    fn reap(&mut self) {
        let mut i = 0;
        while i < self.creeps.len() {
            if self.creeps[i].hp > 0.0 {
                i += 1;
                continue;
            }
            let c = self.creeps.remove(i);
            let coins = c.kind.spec().bounty;
            self.coins += coins;
            self.score += 1;
            self.chain = if self.chain_t > 0.0 { self.chain + 1 } else { 1 };
            self.chain_t = 0.6;
            self.cues.push(Cue::Pop { x: c.x, y: c.y, kind: c.kind, coins, chain: self.chain });
            if c.kind == CreepKind::Splitter {
                self.cues.push(Cue::Split { x: c.x, y: c.y });
                for k in 0..SPLIT_INTO {
                    self.spawn(CreepKind::Mini, (c.d + (k as f32 - 1.0) * 10.0).max(0.0));
                }
            }
        }
    }

    pub fn step(&mut self, dt: f32) {
        if self.over {
            return;
        }
        self.t += dt;
        self.chain_t -= dt;
        if let Some(e) = &mut self.ending {
            *e += dt;
            let e = *e;
            // The devouring: every jelly rushes the cake.
            let end = self.road_len();
            for c in &mut self.creeps {
                c.d += c.speed * 6.0 * dt;
                c.age += dt;
            }
            for i in 0..self.creeps.len() {
                let (x, y) = self.point_at(self.creeps[i].d);
                self.creeps[i].x = x;
                self.creeps[i].y = y;
            }
            for c in self.creeps.iter().filter(|c| c.d >= end) {
                self.cues.push(Cue::Chomp { x: c.x, y: c.y });
            }
            self.creeps.retain(|c| c.d < end);
            self.shots.clear();
            if e >= ENDING {
                self.over = true;
                self.cues.push(Cue::Over);
            }
            return;
        }

        // Waves.
        self.next_in -= dt;
        if self.next_in <= 0.0 {
            self.start_wave();
        }
        let mut k = 0;
        while k < self.spawns.len() {
            self.spawns[k].0 -= dt;
            if self.spawns[k].0 <= 0.0 {
                let kind = self.spawns.remove(k).1;
                self.spawn(kind, 0.0);
            } else {
                k += 1;
            }
        }

        // Jellies walk; the ones that reach the cake bite.
        let end = self.road_len();
        for i in 0..self.creeps.len() {
            let c = &mut self.creeps[i];
            c.d += c.cur_speed() * dt;
            c.slow = (c.slow - dt).max(0.0);
            c.flash = (c.flash - dt * 6.0).max(0.0);
            c.age += dt;
            let (x, y) = self.point_at(self.creeps[i].d);
            self.creeps[i].x = x;
            self.creeps[i].y = y;
        }
        let mut i = 0;
        while i < self.creeps.len() {
            if self.creeps[i].d >= end {
                let c = self.creeps.remove(i);
                let bite = if c.kind == CreepKind::King { KING_BITE } else { 1 };
                self.hearts = self.hearts.saturating_sub(bite);
                self.cues.push(Cue::Bite { x: c.x, y: c.y, hearts: self.hearts });
                if self.hearts == 0 {
                    self.ending = Some(0.0);
                    self.cues.push(Cue::Ending);
                    return;
                }
            } else {
                i += 1;
            }
        }

        // Towers.
        for p in 0..PADS.len() {
            let Some(t) = &mut self.towers[p] else { continue };
            t.age += dt;
            t.recoil = (t.recoil - dt * 5.0).max(0.0);
            t.cd -= dt;
            if t.cd > 0.0 {
                continue;
            }
            let (px, py) = PADS[p];
            let (kind, level, range) = (t.kind, t.level, t.range());
            let spec = kind.spec();
            // Target: the jelly furthest along the road that's in range.
            let mut best: Option<usize> = None;
            for (j, c) in self.creeps.iter().enumerate() {
                if (c.x - px).hypot(c.y - py) <= range && best.is_none_or(|b| c.d > self.creeps[b].d) {
                    best = Some(j);
                }
            }
            let Some(j) = best else { continue };
            let t = self.towers[p].as_mut().unwrap();
            t.cd = spec.interval[level];
            t.recoil = 1.0;
            let (cx, cy) = (self.creeps[j].x, self.creeps[j].y);
            t.aim = (cy - py).atan2(cx - px);
            match kind {
                TowerKind::Pop => {
                    let id = self.creeps[j].id;
                    self.shots.push(Shot { kind, x: px, y: py - 10.0, sx: px, sy: py - 10.0, tx: cx, ty: cy, target: Some(id), damage: spec.damage[level], splash: 0.0, t: 0.0 });
                    self.cues.push(Cue::Fire { kind, x: px, y: py });
                }
                TowerKind::Boom => {
                    // Lob to where the jelly will be when the bomb lands.
                    let c = &self.creeps[j];
                    let (lx, ly) = self.point_at(c.d + c.cur_speed() * BOOM_FLIGHT);
                    self.shots.push(Shot { kind, x: px, y: py, sx: px, sy: py - 8.0, tx: lx, ty: ly, target: None, damage: spec.damage[level], splash: spec.special[level], t: 0.0 });
                    self.cues.push(Cue::Fire { kind, x: px, y: py });
                }
                TowerKind::Chill => {
                    let (dmg, k) = (spec.damage[level], spec.special[level]);
                    for q in 0..self.creeps.len() {
                        let c = &mut self.creeps[q];
                        if (c.x - px).hypot(c.y - py) <= range {
                            c.slow_k = if c.slow > 0.0 { c.slow_k.min(k) } else { k };
                            c.slow = SLOW_TIME;
                            self.hurt(q, dmg);
                        }
                    }
                    self.cues.push(Cue::Frost { x: px, y: py, r: range });
                }
            }
        }

        // Shots.
        let mut s = 0;
        while s < self.shots.len() {
            self.shots[s].t += dt;
            let shot = self.shots[s].clone();
            let mut done = false;
            match shot.kind {
                TowerKind::Pop => {
                    // Home on the target while it lives; else fly to its last spot.
                    let target = shot.target.and_then(|id| self.creeps.iter().position(|c| c.id == id));
                    let (tx, ty) = target.map_or((shot.tx, shot.ty), |q| (self.creeps[q].x, self.creeps[q].y));
                    let (dx, dy) = (tx - shot.x, ty - shot.y);
                    let dist = dx.hypot(dy);
                    let step = POP_SHOT_SPEED * dt;
                    if dist <= step.max(8.0) {
                        done = true;
                        if let Some(q) = target {
                            self.hurt(q, shot.damage);
                            self.cues.push(Cue::Hit { x: tx, y: ty });
                        }
                    } else {
                        let sh = &mut self.shots[s];
                        sh.x += dx / dist * step;
                        sh.y += dy / dist * step;
                        sh.tx = tx;
                        sh.ty = ty;
                    }
                    if shot.t > 1.5 {
                        done = true;
                    }
                }
                TowerKind::Boom => {
                    let k = (shot.t / BOOM_FLIGHT).min(1.0);
                    let sh = &mut self.shots[s];
                    sh.x = shot.sx + (shot.tx - shot.sx) * k;
                    sh.y = shot.sy + (shot.ty - shot.sy) * k;
                    if k >= 1.0 {
                        done = true;
                        for q in 0..self.creeps.len() {
                            let c = &self.creeps[q];
                            if (c.x - shot.tx).hypot(c.y - shot.ty) <= shot.splash + c.kind.spec().radius * 0.5 {
                                self.hurt(q, shot.damage);
                            }
                        }
                        self.cues.push(Cue::Boom { x: shot.tx, y: shot.ty, r: shot.splash });
                    }
                }
                TowerKind::Chill => done = true,
            }
            if done {
                self.shots.remove(s);
            } else {
                s += 1;
            }
        }
        self.reap();
    }

    /// 0 (calm) … 1 (a jelly at the cake, or one heart left): drives the
    /// tension music, the cake's face and the warning vignette.
    pub fn danger(&self) -> f32 {
        let end = self.road_len();
        let near = self.creeps.iter().map(|c| c.d / end).fold(0.0f32, f32::max);
        let road = ((near - 0.6) / 0.4).clamp(0.0, 1.0);
        let hearts = 1.0 - (self.hearts as f32 - 1.0) / (HEARTS as f32 - 1.0);
        road.max(hearts * 0.85).max(if self.creeps.iter().any(|c| c.kind == CreepKind::King) { 0.7 } else { 0.0 })
    }

    /// Jellies still to come in waves already started.
    pub fn pending_spawns(&self) -> usize {
        self.spawns.len()
    }
}

/// What wave `n` brings. The first ten are written by hand (a new jelly
/// every couple of waves); after that it's a budget spent on a random mix,
/// with a king every fifth wave.
pub fn plan_wave(n: u32, rng: &mut Rng) -> WavePlan {
    use CreepKind::*;
    let (groups, new_kind): (Vec<(CreepKind, u32)>, Option<CreepKind>) = match n {
        1 => (vec![(Gummy, 4)], None),
        2 => (vec![(Gummy, 7)], None),
        3 => (vec![(Gummy, 6), (Zipper, 6)], Some(Zipper)),
        4 => (vec![(Helmet, 3), (Gummy, 8)], Some(Helmet)),
        5 => (vec![(Gummy, 10), (Zipper, 6), (Helmet, 2)], None),
        6 => (vec![(Zipper, 10), (Helmet, 3), (Gummy, 4)], None),
        7 => (vec![(Splitter, 4), (Gummy, 8)], Some(Splitter)),
        8 => (vec![(Zipper, 18), (Helmet, 3)], None),
        9 => (vec![(Splitter, 5), (Helmet, 5), (Zipper, 8)], None),
        10 => (vec![(King, 1), (Gummy, 8)], Some(King)),
        _ => {
            let mut budget = 14 + 3 * n as i32;
            let mut g = Vec::new();
            if n % 5 == 3 {
                return WavePlan { n, groups: vec![(Zipper, (budget / 2) as u32), (Mini, (budget / 3) as u32)], boss: false, new_kind: None, rush: true };
            }
            if n.is_multiple_of(5) && n >= FIRST_BOSS_WAVE {
                g.push((King, 1 + (n - FIRST_BOSS_WAVE) / 10));
                budget /= 2;
            }
            // Two or three kinds, never all the hard ones at once.
            let pool = [(Gummy, 2), (Zipper, 2), (Helmet, 4), (Splitter, 5)];
            let a = (rng.f32() * 4.0) as usize % 4;
            let b = (a + 1 + (rng.f32() * 3.0) as usize % 3) % 4;
            for &(k, cost) in [pool[a], pool[b]].iter() {
                let share = budget / 2;
                g.push((k, (share / cost).max(2) as u32));
            }
            (g, None)
        }
    };
    let boss = groups.iter().any(|g| g.0 == King);
    WavePlan { n, groups, boss, new_kind, rush: n == 8 }
}

/// Road length inside a circle: how much a tower at `pad` with `range` sees.
pub fn coverage(sim: &Sim, pad: usize, range: f32) -> f32 {
    let (px, py) = PADS[pad];
    let step = 4.0;
    let mut d = 0.0;
    let mut n = 0;
    while d < sim.road_len() {
        let (x, y) = sim.point_at(d);
        if (x - px).hypot(y - py) <= range {
            n += 1;
        }
        d += step;
    }
    n as f32 * step
}
