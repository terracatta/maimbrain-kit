//! Sounds, music and haptics from the sim's cues.
//!
//! Output only: nothing here is read back by the game, and pitch jitter comes
//! from this module's own fixed-seed generator, never the game's RNG, so
//! playing sound can't change a replay.
//!
//! The mix (measured with the skill's scripts/mix_check.py on a bot round,
//! tests.rs `mix_log`): the march is the reference. A full board fires a
//! gumball, a hit and a splat several times a second, so those are the
//! quietest sounds, varied in pitch and level, rate-limited and held to
//! `VOICES` voices each (the oldest fades out); the music came out in B♭
//! major, and the tonal effects are pitched onto its notes.

use maimbrain::Rng;
use maimbrain::audio::{Sound as Snd, Voice};
use maimbrain::sensors::{self, Haptic};
use maimbrain::sys::{Asset, AssetState};

use crate::sim::{CreepKind, Cue, TowerKind};

/// The files, in load order. All generated (`mb music`, `mb sfx`; each has its
/// `.gen.json` recipe, tools/regen.sh remakes them): a bouncy circus march and
/// its urgent boss-wave twin, and clay-foley effects.
const FILES: [&str; 19] = [
    "music_base", "music_tension", "start", "build", "upgrade", "pop", "lob", "frost", "boom", "hit", "splat", "coin", "bite", "wave", "boss", "deny",
    "ui", "over", "best",
];
const MUSIC_BASE: usize = 0;
const MUSIC_TENSION: usize = 1;
const START: usize = 2;
const BUILD: usize = 3;
const UPGRADE: usize = 4;
const POP: usize = 5;
const LOB: usize = 6;
const FROST: usize = 7;
const BOOM: usize = 8;
const HIT: usize = 9;
const SPLAT: usize = 10;
const COIN: usize = 11;
const BITE: usize = 12;
const WAVE: usize = 13;
const BOSS: usize = 14;
const DENY: usize = 15;
const UI: usize = 16;
const OVER: usize = 17;
const BEST: usize = 18;

/// Music level: the reference everything else is mixed against (the loops
/// are saved at -20 dBFS RMS, effects at -14, so effects play well under 1).
const MUSIC_VOL: f32 = 1.0;
/// Voices at once for each rapid-fire sound (gumball, hit, splat, boom).
const VOICES: usize = 3;
/// The mortar's thoomp rings on B: a semitone down puts it on B♭.
const LOB_PITCH: f32 = 0.943_874;
/// Danger at which the music turns urgent (a king on the road, the last
/// heart, a jelly almost at the cake), and below which it calms again.
const URGENT_ON: f32 = 0.8;
const URGENT_OFF: f32 = 0.55;
/// A sound that hasn't finished decoding plays silently, and a silent loop
/// would stay silent: music waits this long after its sounds exist.
const DECODE_GRACE: f32 = 0.6;
/// Semitones a splat rises per step of a quick chain, and the most steps:
/// it climbs gently (±7 semitones at most with the jelly's size) and holds.
const CHAIN_STEP: f32 = 1.5;
const CHAIN_STEPS: f32 = 4.0;

pub struct Sound {
    assets: Vec<Asset>,
    sounds: Vec<Option<Snd>>,
    age: Vec<f32>,
    rng: Rng,
    music: Option<(Voice, Voice)>,
    want_music: bool,
    /// How far the music has crossed from the march to the urgent loop (eased),
    /// master fade and duck.
    tension: f32,
    /// Whether the urgent loop is wanted (with hysteresis, so it doesn't flicker).
    urgent: bool,
    fade: f32,
    duck: f32,
    /// Per-sound cooldowns so many towers don't machine-gun.
    cool: Vec<f32>,
    /// Delayed one-shots: (seconds left, sound, vol).
    later: Vec<(f32, usize, f32)>,
    /// Voices of each capped sound still sounding: (age, voice), oldest first.
    pool: Vec<Vec<(f32, Option<Voice>)>>,
    /// Stolen voices fading out (the 10 ms glide), stopped when this runs out.
    fading: Vec<(f32, Voice)>,
    /// Test builds: every sound event, for scripts/mix_check.py (tests.rs `mix_log`).
    #[cfg(test)]
    pub log: Option<String>,
    #[cfg(test)]
    clock: f64,
    #[cfg(test)]
    logged: [f32; 2],
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            assets: FILES.iter().map(|f| Asset::load(&format!("assets/{f}.ogg"))).collect(),
            sounds: vec![None; FILES.len()],
            age: vec![0.0; FILES.len()],
            rng: Rng::new(0x7d_ca4e),
            music: None,
            want_music: false,
            tension: 0.0,
            urgent: false,
            fade: 1.0,
            duck: 1.0,
            cool: vec![0.0; FILES.len()],
            later: Vec::new(),
            pool: vec![Vec::new(); FILES.len()],
            fading: Vec::new(),
            #[cfg(test)]
            log: None,
            #[cfg(test)]
            clock: 0.0,
            #[cfg(test)]
            logged: [-1.0; 2],
        }
    }

    fn play(&mut self, i: usize, vol: f32, pitch: f32) {
        self.voice(i, vol, pitch);
    }

    fn voice(&mut self, i: usize, vol: f32, pitch: f32) -> Option<Voice> {
        self.note(i, vol, pitch, "play");
        self.sounds[i].map(|s| s.play(vol, 0.0, pitch))
    }

    /// For the rapid-fire sounds: at most once per `gap` seconds, ±1.5 dB,
    /// and at most `VOICES` at once (the oldest fades out).
    fn play_capped(&mut self, i: usize, vol: f32, pitch: f32, gap: f32) {
        if self.cool[i] > 0.0 {
            return;
        }
        self.cool[i] = gap;
        if self.pool[i].len() >= VOICES {
            let (_, old) = self.pool[i].remove(0);
            if let Some(old) = old {
                old.set(0.0, 0.0, 1.0);
                self.fading.push((0.03, old));
            }
            self.note(i, 0.0, 1.0, "stop");
        }
        let vol = vol * (self.rng.range(-1.5, 1.5) / 20.0 * std::f32::consts::LN_10).exp();
        let v = self.voice(i, vol, pitch);
        self.pool[i].push((0.0, v));
    }

    #[cfg(test)]
    fn note(&mut self, i: usize, vol: f32, pitch: f32, op: &str) {
        if let Some(l) = &mut self.log {
            l.push_str(&format!("{:.3} {} {:.3} {:.4} {op}\n", self.clock, FILES[i], vol, pitch));
        }
    }

    #[cfg(not(test))]
    fn note(&mut self, _: usize, _: f32, _: f32, _: &str) {}

    /// Logs the two loops' levels in tests (a loop when they start, then changes).
    #[cfg(test)]
    fn note_music(&mut self, on: bool, levels: [f32; 2]) {
        if self.log.is_none() {
            return;
        }
        for (k, &v) in levels.iter().enumerate() {
            let i = [MUSIC_BASE, MUSIC_TENSION][k];
            let was = self.logged[k];
            if !on {
                if was >= 0.0 {
                    self.note(i, 0.0, 1.0, "stop");
                }
                self.logged[k] = -1.0;
            } else if was < 0.0 || (v - was).abs() > 0.004 {
                self.note(i, v, 1.0, if was < 0.0 { "loop" } else { "set" });
                self.logged[k] = v;
            }
        }
    }

    /// Plays unless the same sound played within `gap` seconds.
    fn play_cool(&mut self, i: usize, vol: f32, pitch: f32, gap: f32) {
        if self.cool[i] <= 0.0 {
            self.cool[i] = gap;
            self.play(i, vol, pitch);
        }
    }

    fn jitter(&mut self, amount: f32) -> f32 {
        1.0 + self.rng.range(-amount, amount)
    }

    /// A round starts: the start blip and the music from the top.
    pub fn start(&mut self) {
        self.stop_music();
        self.later.clear();
        self.play(START, 0.6, 1.0);
        self.want_music = true;
        self.fade = 1.0;
        self.tension = 0.0;
        self.urgent = false;
    }

    pub fn stop_music(&mut self) {
        if let Some((a, b)) = self.music.take() {
            a.stop();
            b.stop();
        }
        self.want_music = false;
        #[cfg(test)]
        self.note_music(false, [0.0; 2]);
    }

    /// Back from a pause mid-round: the music comes back.
    pub fn resume_music(&mut self) {
        self.want_music = true;
        self.fade = 1.0;
    }

    /// Taps on UI (opening the picker, closing it).
    pub fn ui(&mut self, up: bool) {
        let p = if up { 1.12 } else { 0.9 };
        self.play(UI, 0.4, p);
    }

    pub fn deny(&mut self) {
        self.play(DENY, 0.55, 1.0);
        sensors::haptic(Haptic::Tap);
    }

    pub fn new_best(&mut self) {
        self.play(BEST, 0.8, 1.0);
        sensors::haptic(Haptic::Success);
    }

    /// The results card's score ticking up.
    pub fn tick(&mut self) {
        self.play_cool(COIN, 0.18, 1.5, 0.05);
    }

    /// Plays one frame's cues. On the title card (`playing` false) the game
    /// plays itself, quietly, and buzzes nothing.
    pub fn cues(&mut self, cues: &[Cue], playing: bool) {
        let card = if playing { 1.0 } else { 0.3 };
        for &cue in cues {
            match cue {
                Cue::Built { kind, .. } => {
                    self.play(BUILD, 0.75 * card, [1.0, 1.12, 0.89][kind as usize]);
                    if playing {
                        sensors::haptic(Haptic::Tap);
                    }
                }
                Cue::Upgraded { level, .. } => {
                    self.play(UPGRADE, 0.6 * card, if level == 1 { 1.0 } else { 1.19 });
                    if playing {
                        sensors::haptic(Haptic::Success);
                    }
                }
                Cue::Denied => {
                    if playing {
                        self.deny();
                    }
                }
                Cue::Fire { kind, .. } => match kind {
                    TowerKind::Pop => {
                        let p = self.jitter(0.06);
                        self.play_capped(POP, 0.15 * card, p, 0.1);
                    }
                    TowerKind::Boom => {
                        // Tonal: vary the level, barely the pitch.
                        let p = self.jitter(0.008) * LOB_PITCH;
                        self.play_capped(LOB, 0.34 * card, p, 0.12);
                    }
                    TowerKind::Chill => {}
                },
                Cue::Frost { .. } => {
                    let p = self.jitter(0.04);
                    self.play_cool(FROST, 0.3 * card, p, 0.12);
                }
                Cue::Boom { .. } => {
                    let p = self.jitter(0.06);
                    self.play_capped(BOOM, 0.24 * card, p, 0.12);
                }
                Cue::Hit { .. } => {
                    let p = self.jitter(0.1);
                    self.play_capped(HIT, 0.17 * card, p, 0.08);
                }
                Cue::Pop { kind, chain, .. } => {
                    // Quick pops climb gently; big jellies pop lower.
                    let step = ((chain.max(1) - 1) as f32).min(CHAIN_STEPS);
                    let size = match kind {
                        CreepKind::King => -5.0,
                        CreepKind::Splitter | CreepKind::Helmet => -2.0,
                        CreepKind::Zipper | CreepKind::Mini => 1.5,
                        CreepKind::Gummy => 0.0,
                    };
                    let p = 2f32.powf((step * CHAIN_STEP + size) / 12.0) * self.jitter(0.03);
                    let v = if kind == CreepKind::King { 0.5 } else { 0.22 };
                    self.play_capped(SPLAT, v * card, p, 0.06);
                    if playing && (chain >= 3 || kind == CreepKind::King) {
                        sensors::haptic(if kind == CreepKind::King { Haptic::Heavy } else { Haptic::Tap });
                    }
                }
                Cue::Split { .. } => {}
                Cue::Chomp { .. } => {
                    let p = self.jitter(0.12);
                    self.play_cool(BITE, 0.5 * card, p * 1.15, 0.09);
                }
                Cue::Bite { hearts, .. } => {
                    self.play(BITE, 0.9 * card, if hearts <= 1 { 0.85 } else { 1.0 });
                    self.duck = 0.4;
                    if playing {
                        sensors::haptic(Haptic::Heavy);
                    }
                }
                Cue::Wave { boss, .. } => {
                    if boss {
                        self.play(BOSS, 0.8 * card, 1.0);
                        self.duck = 0.3;
                        if playing {
                            sensors::haptic(Haptic::Heavy);
                        }
                    } else {
                        self.play(WAVE, 0.42 * card, 1.0);
                    }
                }
                Cue::Payday { .. } => self.later.push((0.25, COIN, 0.35 * card)),
                Cue::Early { bonus } => {
                    if bonus > 0 {
                        self.play(COIN, 0.7, 1.0);
                        self.later.push((0.08, COIN, 0.6));
                    }
                }
                Cue::Ending => {
                    self.fade_out();
                    if playing {
                        sensors::haptic(Haptic::Fail);
                    }
                }
                Cue::Over => {
                    if playing {
                        self.play(OVER, 0.85, 1.0);
                    }
                }
            }
        }
    }

    /// Music out under the devouring (it's the punchline's beat of quiet).
    fn fade_out(&mut self) {
        self.want_music = false;
    }

    /// Advances: loads sounds, plays delayed ones, mixes the music.
    /// `danger` (0…1) crosses the music from the march to the urgent loop.
    pub fn update(&mut self, dt: f32, danger: f32) {
        #[cfg(test)]
        {
            self.clock += dt as f64;
        }
        for i in 0..FILES.len() {
            if self.sounds[i].is_none() {
                if self.assets[i].state() == AssetState::Ready {
                    self.sounds[i] = Snd::new(self.assets[i]);
                }
            } else {
                self.age[i] += dt;
            }
            self.cool[i] -= dt;
            for v in &mut self.pool[i] {
                v.0 += dt;
            }
            self.pool[i].retain(|v| v.0 < 0.6);
        }
        self.fading.retain_mut(|f| {
            f.0 -= dt;
            if f.0 <= 0.0 {
                f.1.stop();
            }
            f.0 > 0.0
        });
        let mut due = Vec::new();
        self.later.retain_mut(|l| {
            l.0 -= dt;
            if l.0 <= 0.0 {
                due.push((l.1, l.2));
            }
            l.0 > 0.0
        });
        for (i, v) in due {
            self.play(i, v, 1.0);
        }
        // The urgent loop swells in over ~0.7 s and leaves over ~1.5 s. The two
        // loops share a tempo but not a downbeat, so they cross rather than stack.
        if danger >= URGENT_ON {
            self.urgent = true;
        } else if danger < URGENT_OFF {
            self.urgent = false;
        }
        let want = if self.urgent { 1.0 } else { 0.0 };
        let tau = if want > self.tension { 0.5 } else { 1.2 };
        self.tension += (want - self.tension) * (1.0 - (-dt / tau).exp());
        self.duck += (1.0 - self.duck) * (1.0 - (-dt / 0.5).exp());
        #[cfg(test)]
        let logging = self.log.is_some() && self.logged[0] >= 0.0;
        #[cfg(not(test))]
        let logging = false;
        if !self.want_music && (self.music.is_some() || logging) {
            self.fade -= dt / 0.4;
            if self.fade <= 0.0 {
                if let Some((a, b)) = self.music.take() {
                    a.stop();
                    b.stop();
                }
                #[cfg(test)]
                self.note_music(false, [0.0; 2]);
            }
        }
        let ready = |i: usize| self.sounds[i].is_some() && self.age[i] >= DECODE_GRACE;
        if self.want_music && self.music.is_none() && ready(MUSIC_BASE) && ready(MUSIC_TENSION) {
            let at = maimbrain::sys::time() + 0.1;
            let (a, b) = (self.sounds[MUSIC_BASE].unwrap(), self.sounds[MUSIC_TENSION].unwrap());
            self.music = Some((a.play_looped_at(MUSIC_VOL, 0.0, 1.0, at), b.play_looped_at(0.0, 0.0, 1.0, at)));
        }
        let m = MUSIC_VOL * self.fade.clamp(0.0, 1.0) * self.duck;
        let levels = [m * (1.0 - 0.9 * self.tension), m * self.tension];
        #[cfg(test)]
        if self.want_music || self.logged[0] >= 0.0 {
            self.note_music(true, levels);
        }
        if let Some((a, b)) = self.music {
            a.set(levels[0], 0.0, 1.0);
            b.set(levels[1], 0.0, 1.0);
        }
    }
}
