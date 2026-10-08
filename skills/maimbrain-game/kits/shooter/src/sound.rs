//! Sounds, music and haptics, from the sim's cues. Output only: nothing
//! here is read back by the game, and the jitter comes from this module's
//! own fixed-seed generator, never the game's, so sound can't change a replay.
//!
//! The music is generated (`tools/regen.sh music`): a synthwave loop split
//! into a low and a high layer (base + high = the full mix; the high layer
//! opens up with the action), and a boss loop that takes over while the
//! Gloom Queen is alive. The effects are synthesized to match it
//! (`tools/synth_sfx.py`: detuned saws, FM bells, filtered noise, gated
//! reverb) and tuned to its key: the main loop is in G minor, the boss loop
//! sits on A, so while the boss loop plays every tonal effect moves up a
//! whole tone ([`KEY_BOSS`]).
//!
//! The mix: the music is the reference. Frequent effects (the auto-fire,
//! hits, pops, gems, grazes: several a second) are the quietest, vary in
//! pitch and level, and each has a cap on simultaneous voices (the oldest
//! is stopped) and a minimum gap between starts. Rare effects (a hit on
//! Pip, the boss, the death, a new best) are loud, and only they duck the
//! music. The levels are in [`MIX`]; `tools/mixsim.py` renders a bot round
//! with them and measures the effects against the music.

use maimbrain::Rng;
use maimbrain::audio::{Sound as Snd, Voice};
use maimbrain::sensors::{self, Haptic};
use maimbrain::sys::{Asset, AssetState};

use crate::sim::{Cue, Kind, W};

pub const FILES: [&str; 25] = [
    "shot", "pop", "hit", "gem", "start", "music_base", "music_hi", "boom", "power", "hurt", "shield", "graze", "lock", "new", "warn",
    "boss_boom", "die", "over", "best", "ui", "music_boss", "shield_hum", "bhit", "charge", "rage",
];
const SHOT: usize = 0;
const POP: usize = 1;
const HIT: usize = 2;
const GEM: usize = 3;
const START: usize = 4;
const MUSIC_BASE: usize = 5;
const MUSIC_HI: usize = 6;
const BOOM: usize = 7;
const POWER: usize = 8;
const HURT: usize = 9;
const SHIELD: usize = 10;
const GRAZE: usize = 11;
const LOCK: usize = 12;
const NEW: usize = 13;
const WARN: usize = 14;
const BOSS_BOOM: usize = 15;
const DIE: usize = 16;
const OVER: usize = 17;
const BEST: usize = 18;
const UI: usize = 19;
const MUSIC_BOSS: usize = 20;
const SHIELD_HUM: usize = 21;
const BHIT: usize = 22;
const CHARGE: usize = 23;
const RAGE: usize = 24;

/// How each effect sits under the music: `vol` its level (the files are
/// all normalized alike), `cap` the most voices of it at once (0: no cap;
/// a new one stops the oldest), `gap` the least time between two starts,
/// and `len` about how long it rings (to know when a voice is done).
#[derive(Clone, Copy)]
struct Mix {
    vol: f32,
    cap: usize,
    gap: f32,
    len: f32,
}

const fn m(vol: f32, cap: usize, gap: f32, len: f32) -> Mix {
    Mix { vol, cap, gap, len }
}

const MIX: [Mix; 25] = [
    m(0.075, 2, 0.12, 0.08), // shot: every other volley (~4.5/s), the softest thing in the game
    m(0.18, 3, 0.05, 0.32),  // pop: every kill (~1–3/s in waves)
    m(0.07, 2, 0.08, 0.06),  // hit: a shot lands but doesn't kill (up to ~10/s)
    m(0.1, 3, 0.04, 0.32),   // gem
    m(0.4, 1, 0.0, 1.5),     // start
    m(0.0, 0, 0.0, 0.0),     // music_base
    m(0.0, 0, 0.0, 0.0),     // music_hi
    m(0.36, 3, 0.05, 0.9),   // boom: big kills and the explosion chains
    m(0.3, 1, 0.0, 1.8),     // power
    m(0.7, 1, 0.0, 0.35),    // hurt
    m(0.6, 1, 0.0, 0.7),     // shield (break)
    m(0.13, 2, 0.12, 0.3),   // graze
    m(0.22, 2, 0.1, 0.9),    // lock (darters)
    m(0.32, 1, 0.0, 1.4),    // new enemy
    m(0.5, 1, 0.0, 3.1),     // warn
    m(0.85, 1, 0.0, 3.2),    // boss_boom
    m(0.75, 1, 0.0, 2.6),    // die
    m(0.6, 1, 0.0, 2.1),     // over
    m(0.65, 1, 0.0, 2.6),    // best
    m(0.35, 2, 0.0, 0.09),   // ui
    m(0.0, 0, 0.0, 0.0),     // music_boss
    m(0.0, 0, 0.0, 0.0),     // shield_hum (looped at HUM_VOL)
    m(0.09, 2, 0.08, 0.13),  // bhit: a shot lands on the queen
    m(0.3, 1, 0.0, 0.8),     // charge: the queen's attack telegraph
    m(0.6, 1, 0.0, 1.9),     // rage: the queen enrages
];

/// The G minor pentatonic (G Bb C D F), in semitones: kills and gems climb
/// it with the chain.
const PENTA: [f32; 5] = [0.0, 3.0, 5.0, 7.0, 10.0];
/// While the boss loop (on A) plays, tonal effects move up a whole tone.
const KEY_BOSS: f32 = 2.0;
/// Music level: the reference everything else is mixed under.
const MUSIC_VOL: f32 = 0.7;
/// The high layer's share at rest: enough sparkle for calm play, the rest
/// comes in with bullets, heat and the last heart.
const HI_FLOOR: f32 = 0.45;
/// Quiet before the queen: the main loop sinks this low under the warning.
const PRE_BOSS: f32 = 0.25;
/// After the queen falls, the main loop comes back after this long.
const BOSS_TO_MAIN: f32 = 1.6;
const HUM_VOL: f32 = 0.09;
/// A sound that hasn't finished decoding plays silently, so music waits
/// this long after its sounds exist before starting.
const DECODE_GRACE: f32 = 0.6;

fn semis(s: f32) -> f32 {
    (s / 12.0).exp2()
}

/// Pentatonic step `n` up from the root, climbing octaves.
fn scale(n: u32) -> f32 {
    let n = n as usize;
    semis(PENTA[n % 5] + 12.0 * (n / 5) as f32)
}

/// 0, 1, … top, top−1, … 1, 0, 1, …: a ladder that climbs gently, turns
/// round at the top (one octave, never chipmunked further) and comes back.
fn bounce(n: u32, top: u32) -> u32 {
    if top == 0 {
        return 0;
    }
    let k = n % (2 * top);
    if k <= top { k } else { 2 * top - k }
}

fn pan(x: f32) -> f32 {
    ((x / W) * 2.0 - 1.0).clamp(-1.0, 1.0) * 0.35
}

pub struct Sound {
    assets: Vec<Asset>,
    sounds: Vec<Option<Snd>>,
    age: Vec<f32>,
    rng: Rng,
    /// Seconds since this module started (for gaps and voice ages).
    clock: f32,
    /// Live one-shot voices per sound, oldest first: (voice, started at).
    voices: Vec<Vec<(Voice, f32)>>,
    last: Vec<f32>,
    /// The main loop's two layers, or the boss loop (`.1` None).
    music: Option<(Voice, Option<Voice>)>,
    music_boss: bool,
    music_wanted: bool,
    /// Seconds before the next track may start (after the boss falls).
    music_wait: f32,
    /// The warning is up: the main loop sinks (eased).
    pre_boss: bool,
    pre: f32,
    /// The boss is alive and being played: tonal effects in its key.
    boss_key: bool,
    hum: Option<Voice>,
    /// The tension layer's level (eased) and the duck (dips on big moments).
    hi: f32,
    duck: f32,
    /// Fade-out after the round (1 → 0).
    fade: f32,
    fading: bool,
    volley: u32,
    booms: u32,
    /// Delayed one-shots: (seconds, sound, vol, pitch).
    later: Vec<(f32, usize, f32, f32)>,
    /// Test builds: every play, stop, loop start and track level, for the
    /// offline mix (`mix_trace` in tests.rs, `tools/mixsim.py`).
    #[cfg(test)]
    pub log: Vec<(f32, u8, usize, f32, f32, u32)>,
    #[cfg(test)]
    next_voice: u32,
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            assets: FILES.iter().map(|f| Asset::load(&format!("assets/{f}.ogg"))).collect(),
            sounds: vec![None; FILES.len()],
            age: vec![0.0; FILES.len()],
            rng: Rng::new(0x0054_07e7),
            clock: 0.0,
            voices: vec![Vec::new(); FILES.len()],
            last: vec![-1.0; FILES.len()],
            music: None,
            music_boss: false,
            music_wanted: false,
            music_wait: 0.0,
            pre_boss: false,
            pre: 1.0,
            boss_key: false,
            hum: None,
            hi: 0.0,
            duck: 1.0,
            fade: 1.0,
            fading: false,
            volley: 0,
            booms: 0,
            later: Vec::new(),
            #[cfg(test)]
            log: Vec::new(),
            #[cfg(test)]
            next_voice: 0,
        }
    }

    /// Starts effect `i` at its mix level × `vol`, honouring its voice cap
    /// and gap. `pitch` is a playback rate (tonal callers pass key × step).
    fn fx(&mut self, i: usize, vol: f32, pitch: f32, pan: f32) {
        let Some(s) = self.sounds[i] else { return };
        let mx = MIX[i];
        if mx.gap > 0.0 && self.last[i] >= 0.0 && self.clock - self.last[i] < mx.gap {
            return;
        }
        let now = self.clock;
        let live = &mut self.voices[i];
        live.retain(|&(_, t)| now - t < mx.len / pitch.max(0.25));
        if mx.cap > 0 && live.len() >= mx.cap {
            let (v, _) = live.remove(0);
            v.stop();
            #[cfg(test)]
            self.log.push((now, 3, i, 0.0, 0.0, v.0));
        }
        #[allow(unused_variables)]
        let v = s.play(mx.vol * vol, pan, pitch);
        #[cfg(test)]
        let v = {
            self.next_voice += 1;
            self.log.push((now, 0, i, mx.vol * vol, pitch, self.next_voice));
            Voice(self.next_voice)
        };
        self.voices[i].push((v, now));
        self.last[i] = now;
    }

    /// Level jitter of ±`db` decibels.
    fn vary(&mut self, db: f32) -> f32 {
        (self.rng.range(-db, db) / 20.0 * std::f32::consts::LOG2_10).exp2()
    }

    fn jitter(&mut self, amount: f32) -> f32 {
        1.0 + self.rng.range(-amount, amount)
    }

    /// The pitch that puts a tonal effect in the key of the music playing.
    fn key(&self) -> f32 {
        if self.boss_key { semis(KEY_BOSS) } else { 1.0 }
    }

    /// A round starts: the start sound and the music from the top.
    pub fn start(&mut self) {
        self.stop_music();
        self.later.clear();
        self.fx(START, 1.0, 1.0, 0.0);
        self.music_wanted = true;
        self.fade = 1.0;
        self.fading = false;
        self.hi = 0.0;
        self.music_wait = 0.0;
        self.pre_boss = false;
        self.pre = 1.0;
        self.duck = 1.0;
    }

    pub fn stop_music(&mut self) {
        self.stop_track();
        self.music_wanted = false;
        if let Some(h) = self.hum.take() {
            h.stop();
        }
    }

    fn stop_track(&mut self) {
        if let Some((a, b)) = self.music.take() {
            a.stop();
            if let Some(b) = b {
                b.stop();
            }
        }
    }

    /// Back from a pause mid-round: the music again, from the top.
    pub fn resume_music(&mut self) {
        self.music_wanted = true;
        self.fade = 1.0;
        self.fading = false;
    }

    /// The round's verdict, a beat after the card appears.
    pub fn verdict(&mut self, new_best: bool) {
        self.later.push((0.35, if new_best { BEST } else { OVER }, 1.0, 1.0));
    }

    pub fn new_best(&mut self) {
        sensors::haptic(Haptic::Success);
    }

    /// A button: up a fourth when it turns something on.
    pub fn ui(&mut self, on: bool) {
        self.fx(UI, 1.0, if on { semis(5.0) } else { 1.0 }, 0.0);
    }

    /// Plays this frame's cues. On the title card (`playing` false) the
    /// attract mode stays nearly silent: only quiet pops. `boss` picks the
    /// boss loop; `shield` hums while Pip's shield is up.
    pub fn update(&mut self, dt: f32, cues: &[Cue], playing: bool, intensity: f32, boss: bool, shield: bool) {
        self.clock += dt;
        for i in 0..FILES.len() {
            if self.sounds[i].is_none() {
                if cfg!(test) {
                    self.sounds[i] = Some(Snd(i as u32 + 1));
                } else if self.assets[i].state() == AssetState::Ready {
                    self.sounds[i] = Snd::new(self.assets[i]);
                }
            } else {
                self.age[i] += dt;
            }
        }
        let mut due = Vec::new();
        self.later.retain_mut(|q| {
            q.0 -= dt;
            if q.0 <= 0.0 {
                due.push((q.1, q.2, q.3));
            }
            q.0 > 0.0
        });
        for (i, v, p) in due {
            self.fx(i, v, p, 0.0);
        }
        self.boss_key = boss && playing;
        self.booms = 0;
        for &cue in cues {
            if playing {
                self.cue(cue);
            } else if let Cue::Kill { x, kind, .. } = cue {
                // The attract mode: a few soft pops, nothing else.
                let p = self.type_pitch(kind, 1) * self.jitter(0.004);
                self.fx(POP, 0.3, p, pan(x));
            }
        }
        self.music_update(dt, intensity, boss && playing);
        self.hum_update(playing && shield && !self.fading);
    }

    /// The pop's pitch: up the pentatonic with the combo, each enemy type
    /// starting higher the faster it is (the big ones an octave down).
    fn type_pitch(&self, kind: Kind, combo: u32) -> f32 {
        let (first, octave) = match kind {
            Kind::Jelly => (0, 1.0),
            Kind::Swooper => (1, 1.0),
            Kind::Darter => (2, 1.0),
            Kind::Bulb | Kind::Spinner => (0, 0.5),
        };
        let step = first + bounce(combo.saturating_sub(1), 5 - first);
        self.key() * octave * scale(step)
    }

    fn hum_update(&mut self, on: bool) {
        match (on, self.hum) {
            (true, None) => {
                if let Some(s) = self.sounds[SHIELD_HUM] {
                    self.hum = Some(s.play_looped(HUM_VOL, 0.0, self.key()));
                    #[cfg(test)]
                    self.log.push((self.clock, 1, SHIELD_HUM, 0.0, self.key(), 0));
                }
            }
            (false, Some(h)) => {
                h.stop();
                self.hum = None;
            }
            (true, Some(h)) => h.set(HUM_VOL, 0.0, self.key()),
            _ => {}
        }
        #[cfg(test)]
        self.log.push((self.clock, 2, SHIELD_HUM, if self.hum.is_some() { HUM_VOL } else { 0.0 }, 1.0, 0));
    }

    fn cue(&mut self, cue: Cue) {
        match cue {
            Cue::Shot => {
                // Every other volley: a soft patter, not a machine gun.
                self.volley += 1;
                if self.volley.is_multiple_of(2) {
                    let (v, p) = (self.vary(1.5), self.key() * self.jitter(0.03));
                    self.fx(SHOT, v, p, 0.0);
                }
            }
            Cue::Hit { x, .. } => {
                let (v, p) = (self.vary(2.0), self.jitter(0.08));
                self.fx(HIT, v, p, pan(x));
            }
            Cue::BossHit { x, .. } => {
                let (v, p) = (self.vary(2.0), self.jitter(0.05));
                self.fx(BHIT, v, p, pan(x));
            }
            Cue::Kill { x, kind, combo, .. } => {
                let p = self.type_pitch(kind, combo) * self.jitter(0.004);
                let v = self.vary(1.0);
                self.fx(POP, v, p, pan(x));
                if matches!(kind, Kind::Bulb | Kind::Spinner) {
                    let p = self.jitter(0.04);
                    self.fx(BOOM, 1.0, p, pan(x));
                    sensors::haptic(Haptic::Heavy);
                } else {
                    sensors::haptic(Haptic::Tap);
                }
            }
            Cue::Gem { x, streak, .. } => {
                let p = self.key() * scale(bounce(streak.saturating_sub(1), 5)) * self.jitter(0.003);
                let v = self.vary(1.0);
                self.fx(GEM, v, p, pan(x));
            }
            Cue::Power { .. } | Cue::Heart { .. } => {
                // A heart: the same arpeggio a fourth up (C minor, still in G minor).
                let p = self.key() * if matches!(cue, Cue::Heart { .. }) { semis(5.0) } else { 1.0 };
                self.fx(POWER, 1.0, p, 0.0);
                sensors::haptic(Haptic::Success);
            }
            Cue::Graze { x, .. } => {
                let (v, p) = (self.vary(1.5), self.key() * self.jitter(0.004));
                self.fx(GRAZE, v, p, pan(x));
            }
            Cue::DarterLock { x, .. } => {
                let p = self.key();
                self.fx(LOCK, 1.0, p, pan(x));
            }
            Cue::BossCharge(_) => self.fx(CHARGE, 1.0, 1.0, 0.0),
            Cue::BossEnrage => {
                self.fx(RAGE, 1.0, 1.0, 0.0);
                self.duck = self.duck.min(0.6);
                sensors::haptic(Haptic::Heavy);
            }
            Cue::ShieldBreak { .. } => {
                let p = self.key();
                self.fx(SHIELD, 1.0, p, 0.0);
                sensors::haptic(Haptic::Heavy);
            }
            Cue::Hurt { .. } => {
                self.fx(HURT, 1.0, 1.0, 0.0);
                self.duck = self.duck.min(0.4);
                sensors::haptic(Haptic::Heavy);
            }
            Cue::NewEnemy(_) => self.fx(NEW, 1.0, 1.0, 0.0),
            Cue::Warning => {
                self.fx(WARN, 1.0, 1.0, 0.0);
                self.duck = self.duck.min(0.5);
                self.pre_boss = true;
            }
            Cue::BossArrive => {
                // Her arrival boom, down to A (the boss loop's key) and long.
                self.fx(BOOM, 1.8, semis(-10.0), 0.0);
                sensors::haptic(Haptic::Heavy);
            }
            Cue::BossBeaten { .. } => {
                self.fx(BOOM, 1.8, semis(-5.0), 0.0);
                self.duck = self.duck.min(0.5);
                sensors::haptic(Haptic::Heavy);
            }
            Cue::BossDown { .. } => {
                self.fx(BOSS_BOOM, 1.0, 1.0, 0.0);
                self.music_wait = BOSS_TO_MAIN;
                self.duck = self.duck.min(0.2);
                sensors::haptic(Haptic::Success);
            }
            Cue::Boom { x, size, .. } => {
                if self.booms < 2 {
                    self.booms += 1;
                    let p = self.jitter(0.05) * (1.15 - 0.35 * size);
                    self.fx(BOOM, 0.6 + 0.9 * size, p, pan(x));
                    if size > 0.8 {
                        sensors::haptic(Haptic::Heavy);
                    }
                }
            }
            Cue::Dying { .. } => {
                self.fx(DIE, 1.0, 1.0, 0.0);
                // The music drains away under the slow-motion death.
                self.fading = true;
                sensors::haptic(Haptic::Fail);
            }
            Cue::Over => {}
            Cue::EnemyFire { .. } | Cue::ComboEnd { .. } => {}
        }
    }

    fn music_update(&mut self, dt: f32, intensity: f32, boss: bool) {
        // The boss loop while she's alive, the main loop otherwise: a hard
        // switch, hidden under her arrival boom and her death explosion.
        if self.music.is_some() && self.music_boss != boss {
            self.stop_track();
            if boss {
                self.music_wait = 0.0;
            }
        }
        self.music_wait = (self.music_wait - dt).max(0.0);
        if self.music_wanted && self.music.is_none() && self.music_wait <= 0.0 && !self.fading {
            let ready = |i: usize| self.sounds[i].is_some() && self.age[i] >= DECODE_GRACE;
            let at = maimbrain::sys::time() + 0.1;
            if boss && ready(MUSIC_BOSS) {
                let b = self.sounds[MUSIC_BOSS].unwrap();
                self.music = Some((b.play_looped_at(MUSIC_VOL, 0.0, 1.0, at), None));
                self.music_boss = true;
                self.pre_boss = false;
                self.pre = 1.0;
                #[cfg(test)]
                self.log.push((self.clock + 0.1, 1, MUSIC_BOSS, 0.0, 1.0, 0));
            } else if !boss && ready(MUSIC_BASE) && ready(MUSIC_HI) {
                let (b, h) = (self.sounds[MUSIC_BASE].unwrap(), self.sounds[MUSIC_HI].unwrap());
                self.music = Some((b.play_looped_at(MUSIC_VOL, 0.0, 1.0, at), Some(h.play_looped_at(0.0, 0.0, 1.0, at))));
                self.music_boss = false;
                #[cfg(test)]
                {
                    self.log.push((self.clock + 0.1, 1, MUSIC_BASE, 0.0, 1.0, 0));
                    self.log.push((self.clock + 0.1, 1, MUSIC_HI, 0.0, 1.0, 0));
                }
            }
        }
        // The high layer swells in over ~0.7 s and leaves over ~1.5 s.
        let target = HI_FLOOR + (1.0 - HI_FLOOR) * intensity;
        let tau = if target > self.hi { 0.7 } else { 1.5 };
        self.hi += (target - self.hi) * (1.0 - (-dt / tau).exp());
        self.duck += (1.0 - self.duck) * (1.0 - (-dt / 0.5).exp());
        let pre = if self.pre_boss && !self.music_boss { PRE_BOSS } else { 1.0 };
        self.pre += (pre - self.pre) * (1.0 - (-dt / 0.8).exp());
        if self.fading {
            self.fade = (self.fade - dt / 1.2).max(0.0);
        }
        let v = MUSIC_VOL * self.duck * self.fade * if self.music_boss { 1.0 } else { self.pre };
        #[cfg(test)]
        {
            let (main, boss) = match self.music {
                Some(_) if self.music_boss => (0.0, v),
                Some(_) => (v, 0.0),
                None => (0.0, 0.0),
            };
            self.log.push((self.clock, 2, MUSIC_BASE, main, 1.0, 0));
            self.log.push((self.clock, 2, MUSIC_HI, main * self.hi, 1.0, 0));
            self.log.push((self.clock, 2, MUSIC_BOSS, boss, 1.0, 0));
        }
        if let Some((b, h)) = self.music {
            b.set(v, 0.0, 1.0);
            if let Some(h) = h {
                h.set(v * self.hi, 0.0, 1.0);
            }
            if self.fading && self.fade <= 0.0 {
                self.stop_track();
                self.music_wanted = false;
            }
        }
    }
}
