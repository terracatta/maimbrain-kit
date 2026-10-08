//! Sounds, music and haptics from the sim's cues. Output only: nothing here
//! feeds back into the game, and pitch jitter comes from this module's own
//! fixed-seed generator, never the game's RNG, so sound can't change a replay.
//!
//! The assets are generated (`tools/regen.sh`: `mb music` for the lullaby and
//! the tension layer, `mb sfx` for the effects). The lullaby came out in F
//! major; the tension drone is an E minor 7 chord, so it plays two semitones
//! down (`TENSION_PITCH`: D minor 7, F major's relative minor). As the
//! poppets get drowsy the drone fades in and the lullaby winds down like a
//! music box (`WIND_DOWN`, the drone follows it).
//!
//! The mix (measured with the skill's scripts/mix_check.py on a bot round,
//! tests.rs `mix_log`): the lullaby is the reference; pops, the most frequent
//! sound, sit well under it with varied pitch and level and at most
//! `POP_VOICES` at once; every tonal effect is pitched onto F major notes.

use maimbrain::Rng;
use maimbrain::audio::{Sound, Voice};
use maimbrain::sensors::{self, Haptic};
use maimbrain::sys::{Asset, AssetState};

use crate::sim::{BlastKind, Cue, News};

const FILES: [&str; 21] = [
    "music", "tension", "start", "select", "swap", "bounce", "pop", "chime", "line", "bomb", "rainbow", "made", "rock",
    "level", "shuffle", "drowsy", "phew", "over", "best", "ui", "thud",
];
const MUSIC: usize = 0;
const TENSION: usize = 1;
const START: usize = 2;
const SELECT: usize = 3;
const SWAP: usize = 4;
const BOUNCE: usize = 5;
const POP: usize = 6;
const CHIME: usize = 7;
const LINE: usize = 8;
const BOMB: usize = 9;
const RAINBOW: usize = 10;
const MADE: usize = 11;
const ROCK: usize = 12;
const LEVEL: usize = 13;
const SHUFFLE: usize = 14;
const DROWSY: usize = 15;
const PHEW: usize = 16;
const OVER: usize = 17;
const BEST: usize = 18;
const UI: usize = 19;
const THUD: usize = 20;
/// The cascade chime's notes from chain 2 up (semitones from its F): the F
/// major pentatonic C D F G B♭ C, within ±7 so the bell never chipmunks.
const CHIME_STEPS: [f32; 6] = [-5.0, -3.0, 0.0, 2.0, 5.0, 7.0];
/// Pops rise a semitone per cascade step, up to this many.
const POP_RISE: f32 = 4.0;
/// Pop voices at once (a cascade stacks them); the oldest fades out.
const POP_VOICES: usize = 3;
/// Music level: the reference everything else is mixed against (the loop is
/// saved at -20 dBFS RMS, effects at -14, so effects play well under 1).
const MUSIC_VOL: f32 = 0.85;
/// The tension drone at full drowsiness, relative to the lullaby.
const TENSION_VOL: f32 = 0.9;
/// The drone is an E minor 7 chord: two semitones down is D minor 7, in F.
const TENSION_PITCH: f32 = 0.890_899;
/// How far the lullaby slows (playback rate) when everyone is nearly asleep.
const WIND_DOWN: f32 = 0.955;
/// The select tick is a low marimba E; a fourth up (A, in F major) makes a light tap.
const SELECT_PITCH: f32 = 1.334_84;
/// The swap swish rings on C#; a minor third up puts it on E.
const SWAP_PITCH: f32 = 1.189_207;
/// A sound that hasn't decoded plays silently, and a silent loop stays silent:
/// music waits this long after its sounds exist.
const DECODE_GRACE: f32 = 0.5;

fn semis(s: f32) -> f32 {
    (s / 12.0).exp2()
}

pub struct Sfx {
    assets: Vec<Asset>,
    sounds: Vec<Option<Sound>>,
    age: Vec<f32>,
    rng: Rng,
    music: Option<(Voice, Voice)>,
    music_wanted: bool,
    music_fade: f32,
    hi: f32,
    hi_target: f32,
    duck: f32,
    /// Seconds to the next drowsy heartbeat tick.
    tick: f32,
    /// Delayed one-shots: (seconds left, sound, vol, pitch).
    later: Vec<(f32, usize, f32, f32)>,
    /// Pop voices still sounding: (age, voice), oldest first.
    pops: Vec<(f32, Option<Voice>)>,
    /// Stolen voices fading out (the 10 ms glide), stopped when this runs out.
    fading: Vec<(f32, Voice)>,
    /// Test builds: every sound event, for scripts/mix_check.py (tests.rs `mix_log`).
    #[cfg(test)]
    pub log: Option<String>,
    #[cfg(test)]
    clock: f64,
    #[cfg(test)]
    sent: [(f32, f32); 2],
}

impl Sfx {
    pub fn new() -> Sfx {
        Sfx {
            assets: FILES.iter().map(|f| Asset::load(&format!("assets/{f}.ogg"))).collect(),
            sounds: vec![None; FILES.len()],
            age: vec![0.0; FILES.len()],
            rng: Rng::new(0x9099_e75),
            music: None,
            music_wanted: false,
            music_fade: 1.0,
            hi: 0.0,
            hi_target: 0.0,
            duck: 1.0,
            tick: 0.0,
            later: Vec::new(),
            pops: Vec::new(),
            fading: Vec::new(),
            #[cfg(test)]
            log: None,
            #[cfg(test)]
            clock: 0.0,
            #[cfg(test)]
            sent: [(-1.0, 0.0); 2],
        }
    }

    pub fn play(&mut self, i: usize, vol: f32, pitch: f32) {
        self.voice(i, vol, pitch);
    }

    fn voice(&mut self, i: usize, vol: f32, pitch: f32) -> Option<Voice> {
        self.note(i, vol, pitch, "play");
        self.sounds[i].map(|s| s.play(vol, 0.0, pitch))
    }

    /// Level and pitch varied a little, so a sound heard often never repeats exactly.
    fn vary(&mut self, vol: f32, pitch: f32, db: f32, pct: f32) -> (f32, f32) {
        (vol * (self.rng.range(-db, db) / 20.0 * std::f32::consts::LN_10).exp(), pitch * self.jitter(pct))
    }

    #[cfg(test)]
    fn note(&mut self, i: usize, vol: f32, pitch: f32, op: &str) {
        if let Some(l) = &mut self.log {
            l.push_str(&format!("{:.3} {} {:.3} {:.4} {op}\n", self.clock, FILES[i], vol, pitch));
        }
    }

    #[cfg(not(test))]
    fn note(&mut self, _: usize, _: f32, _: f32, _: &str) {}

    /// The music stems' levels: sent to the voices, and logged in tests.
    fn send_music(&mut self, levels: [(f32, f32); 2]) {
        #[cfg(test)]
        if self.log.is_some() {
            for (k, &(v, p)) in levels.iter().enumerate() {
                let (sv, sp) = self.sent[k];
                if sv < 0.0 {
                    self.note([MUSIC, TENSION][k], v, p, "loop");
                } else if (v - sv).abs() > 0.004 || (p - sp).abs() > 0.002 {
                    self.note([MUSIC, TENSION][k], v, p, "set");
                } else {
                    continue;
                }
                self.sent[k] = (v, p);
            }
        }
        if let Some((m, h)) = self.music {
            m.set(levels[0].0, 0.0, levels[0].1);
            h.set(levels[1].0, 0.0, levels[1].1);
        }
    }

    /// Whether the music is (or, in a logged test, would be) playing.
    fn music_on(&self) -> bool {
        #[cfg(test)]
        if self.log.is_some() {
            return self.sent[0].0 >= 0.0;
        }
        self.music.is_some()
    }

    fn jitter(&mut self, amount: f32) -> f32 {
        1.0 + self.rng.range(-amount, amount)
    }

    /// A round starts: the start sting and the music from the top.
    pub fn start(&mut self) {
        self.stop_music();
        self.later.clear();
        self.play(START, 0.6, 1.0);
        self.music_wanted = true;
        self.music_fade = 1.0;
        self.hi = 0.0;
    }

    pub fn stop_music(&mut self) {
        self.kill_music();
        self.music_wanted = false;
    }

    fn kill_music(&mut self) {
        if let Some((a, b)) = self.music.take() {
            a.stop();
            b.stop();
        }
        #[cfg(test)]
        if self.log.is_some() && self.sent[0].0 >= 0.0 {
            self.note(MUSIC, 0.0, 1.0, "stop");
            self.note(TENSION, 0.0, 1.0, "stop");
            self.sent = [(-1.0, 0.0); 2];
        }
    }

    pub fn select(&mut self) {
        // Tonal: vary the level, keep the note.
        let (v, p) = self.vary(0.4, SELECT_PITCH, 1.5, 0.005);
        self.play(SELECT, v, p);
        sensors::haptic(Haptic::Tap);
    }

    /// A rock poked (rocks don't move).
    pub fn play_rock_poke(&mut self) {
        let (v, p) = self.vary(0.4, 0.7, 1.5, 0.05);
        self.play(ROCK, v, p);
        sensors::haptic(Haptic::Tap);
    }

    pub fn ui(&mut self) {
        self.play(UI, 0.45, 1.0);
    }

    pub fn best(&mut self) {
        self.play(BEST, 0.7, 1.0);
        sensors::haptic(Haptic::Success);
    }

    /// How much tension music to mix in (0…1): danger and level.
    pub fn tension(&mut self, k: f32) {
        self.hi_target = k.clamp(0.0, 1.0);
    }

    /// Per frame: load sounds, start and mix the music, the drowsy heartbeat.
    pub fn update(&mut self, dt: f32, drowsy: bool) {
        #[cfg(test)]
        {
            self.clock += dt as f64;
        }
        for i in 0..FILES.len() {
            if self.sounds[i].is_none() {
                if self.assets[i].state() == AssetState::Ready {
                    self.sounds[i] = Sound::new(self.assets[i]);
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
            self.play(i, v, p);
        }
        for p in &mut self.pops {
            p.0 += dt;
        }
        self.pops.retain(|p| p.0 < 0.6);
        self.fading.retain_mut(|f| {
            f.0 -= dt;
            if f.0 <= 0.0 {
                f.1.stop();
            }
            f.0 > 0.0
        });
        if self.music_wanted && self.music.is_none() && self.age[MUSIC] >= DECODE_GRACE && self.age[TENSION] >= DECODE_GRACE {
            if let (Some(m), Some(h)) = (self.sounds[MUSIC], self.sounds[TENSION]) {
                let at = maimbrain::sys::time() + 0.1;
                self.music = Some((m.play_looped_at(MUSIC_VOL, 0.0, 1.0, at), h.play_looped_at(0.0, 0.0, TENSION_PITCH, at)));
            }
        }
        #[cfg(test)]
        if self.music_wanted && self.log.is_some() && self.sent[0].0 < 0.0 {
            self.send_music([(MUSIC_VOL, 1.0), (0.0, TENSION_PITCH)]);
        }
        let tau = if self.hi_target > self.hi { 0.6 } else { 1.5 };
        self.hi += (self.hi_target - self.hi) * (1.0 - (-dt / tau).exp());
        self.duck += (1.0 - self.duck) * (1.0 - (-dt / 0.4).exp());
        if !self.music_wanted && self.music_on() {
            self.music_fade -= dt / 0.35;
            if self.music_fade <= 0.0 {
                self.kill_music();
            }
        }
        if self.music_on() {
            let v = MUSIC_VOL * self.duck * self.music_fade.clamp(0.0, 1.0);
            // The lullaby winds down as the drone comes in.
            let rate = 1.0 + (WIND_DOWN - 1.0) * self.hi;
            self.send_music([(v * (1.0 - 0.35 * self.hi), rate), (v * TENSION_VOL * self.hi, TENSION_PITCH * rate)]);
        }
        // A soft heartbeat (two plush thuds) while they're nodding off.
        if drowsy {
            self.tick -= dt;
            if self.tick <= 0.0 {
                self.tick = 0.7;
                self.play(THUD, 0.4, 1.0);
                self.later.push((0.18, THUD, 0.28, 0.9));
            }
        } else {
            self.tick = 0.0;
        }
    }

    /// This frame's cues. Off the card (`playing` false) nothing sounds.
    pub fn cues(&mut self, cues: &[Cue], playing: bool) {
        if !playing {
            return;
        }
        let mut fires = 0;
        for &cue in cues {
            match cue {
                Cue::Swap => {
                    let (v, p) = self.vary(0.45, SWAP_PITCH, 1.5, 0.008);
                    self.play(SWAP, v, p);
                }
                Cue::Bounce => {
                    self.play(BOUNCE, 0.5, 1.0);
                    sensors::haptic(Haptic::Tap);
                }
                Cue::Pop { chain, pieces } => {
                    // The most frequent sound: quiet, varied, capped. It rises
                    // gently with the cascade; the chime carries the melody.
                    let k = (chain.max(1) - 1) as f32;
                    let (v, p) = self.vary(0.28 + 0.01 * pieces.min(8) as f32, semis(k.min(POP_RISE)), 1.5, 0.03);
                    if self.pops.len() >= POP_VOICES {
                        let (_, old) = self.pops.remove(0);
                        if let Some(old) = old {
                            old.set(0.0, 0.0, 1.0);
                            self.fading.push((0.03, old));
                        }
                        self.note(POP, 0.0, 1.0, "stop");
                    }
                    let voice = self.voice(POP, v, p);
                    self.pops.push((0.0, voice));
                    if chain >= 2 {
                        let s = CHIME_STEPS[(chain as usize - 2).min(CHIME_STEPS.len() - 1)];
                        let (v, _) = self.vary(0.27, 1.0, 1.0, 0.0);
                        self.play(CHIME, v, semis(s));
                    }
                    sensors::haptic(if chain >= 3 { Haptic::Success } else { Haptic::Tap });
                }
                Cue::Made { .. } => {
                    let (v, p) = self.vary(0.26, 1.0, 1.0, 0.02);
                    self.play(MADE, v, p);
                }
                Cue::Fire { kind } => {
                    fires += 1;
                    if fires > 2 {
                        continue;
                    }
                    let p = self.jitter(0.04);
                    match kind {
                        BlastKind::Row | BlastKind::Col | BlastKind::Cross(_) => self.play(LINE, 0.55, p),
                        BlastKind::Area(_) => self.play(BOMB, 0.75, p),
                        BlastKind::Zap | BlastKind::Board => self.play(RAINBOW, 0.65, 1.0),
                    }
                    sensors::haptic(Haptic::Heavy);
                }
                Cue::Combo => {
                    self.play(BOMB, 0.9, 0.8);
                    self.duck = 0.4;
                }
                Cue::Rocks { .. } => {
                    let p = self.jitter(0.06);
                    self.play(ROCK, 0.75, p);
                }
                Cue::Star { .. } => self.play(PHEW, 0.6, 1.5),
                Cue::Cascade { .. } => {}
                Cue::Level { news, .. } => {
                    // Other news a fourth up (both in F major).
                    self.play(LEVEL, 0.55, if news == News::Faster { 1.0 } else { semis(5.0) });
                    sensors::haptic(Haptic::Success);
                }
                Cue::Shuffle => self.play(SHUFFLE, 0.6, 1.0),
                Cue::Drowsy => {
                    self.play(DROWSY, 0.55, 1.0);
                    sensors::haptic(Haptic::Fail);
                }
                Cue::Phew => self.play(PHEW, 0.7, 1.0),
                Cue::Asleep => {
                    self.music_wanted = false;
                    self.play(OVER, 0.75, 1.0);
                    sensors::haptic(Haptic::Fail);
                }
            }
        }
    }
}
