//! Sounds, music and haptics from the sim's cues. Output only: nothing here
//! feeds back into the game, and pitch jitter comes from this module's own
//! fixed-seed generator, so playing sound can't change a replay.
//!
//! Everything is generated (tools/regen.sh): a big-band swing loop split
//! into a calm base and a bright high layer (`mb music --layers`), a ticking
//! stopwatch loop for the clock's tension, and designed effects
//! (`mb sfx`). The right-answer ding climbs with the streak on notes that
//! fit the band (a chromatic swing tune: E minor or A minor, measured), at
//! most a fifth up so the recorded brass stab never chipmunks.
//!
//! The mix (measured with the skill's scripts/mix_check.py on a bot round,
//! tests.rs `mix_log`): the band is the reference; the clock's ticks and the
//! stopwatch loop, the most frequent sounds, sit well under it, and the
//! stings (right, wrong, golden) peak a few dB over it at most.

use maimbrain::Rng;
use maimbrain::audio::{Sound as Snd, Voice};
use maimbrain::sensors::{self, Haptic};
use maimbrain::sys::{self, Asset, AssetState};

use crate::sim::Cue;

const FILES: [&str; 18] = [
    "start", "flip", "tap", "right", "wrong", "timeup", "tick", "multup", "golden", "heart", "pop", "over", "best", "ui", "music_base", "music_hi", "fizzle", "tension",
];
pub const START: usize = 0;
pub const FLIP: usize = 1;
pub const TAP: usize = 2;
pub const RIGHT: usize = 3;
const WRONG: usize = 4;
const TIMEUP: usize = 5;
const TICK: usize = 6;
const MULTUP: usize = 7;
const GOLDEN: usize = 8;
const HEART: usize = 9;
pub const POP: usize = 10;
const OVER: usize = 11;
const BEST: usize = 12;
pub const UI: usize = 13;
const MUSIC: usize = 14;
const MUSIC_HI: usize = 15;
const FIZZLE: usize = 16;
const TENSION: usize = 17;
/// The right-answer ding (a G and F dyad) by streak: semitones that keep it in
/// the band's key, then it holds at the top.
const RIGHT_STEPS: [f32; 4] = [-3.0, 2.0, 4.0, 7.0];
/// The clock tick (a woodblock on A#) for 3, 2 and 1 seconds left: A, B, E,
/// rising into the band's key.
const TICK_STEPS: [f32; 3] = [-1.0, 1.0, 6.0];
/// Music level: the reference everything else is mixed against (the base
/// stem is saved quieter than most loops, at -23 dBFS RMS, so it plays above 1).
const MUSIC_VOL: f32 = 1.3;
/// The high layer (brass, cymbals) never drops below this share, so the
/// band always swings; intensity brings it up to full.
const HI_FLOOR: f32 = 0.5;
/// The ticking stopwatch loop, played a touch fast so its ticks fall on the
/// band's quarter notes: (5.95 s loop / 15 ticks) / (12.048 s / 32 beats).
/// Re-derive it when you regenerate either loop (tools/regen.sh prints both lengths).
const TENSION_PITCH: f32 = 1.0536;
const TENSION_VOL: f32 = 0.65;

pub struct Sound {
    assets: Vec<Asset>,
    sounds: Vec<Option<Snd>>,
    rng: Rng,
    music: Option<(Voice, Voice, Option<Voice>)>,
    /// Asked to start the music; it starts once both stems exist.
    music_at: Option<f64>,
    /// The tension layer's level, eased toward `hi_target`.
    hi: f32,
    pub hi_target: f32,
    /// The stopwatch layer's level (0…1, from how close the clock is), eased.
    tension: f32,
    pub tension_target: f32,
    /// Ducks the whole loop for a moment (a miss, a fanfare).
    duck: f32,
    /// Delayed one-shots: (seconds left, sound, vol, pitch).
    later: Vec<(f32, usize, f32, f32)>,
    /// Test builds: every sound event, for scripts/mix_check.py (tests.rs `mix_log`).
    #[cfg(test)]
    pub log: Option<String>,
    #[cfg(test)]
    clock: f64,
    #[cfg(test)]
    logged: [f32; 3],
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            assets: FILES.iter().map(|f| Asset::load(&format!("assets/{f}.ogg"))).collect(),
            sounds: vec![None; FILES.len()],
            rng: Rng::new(0x5_9a_7c),
            music: None,
            music_at: None,
            hi: 0.0,
            hi_target: 0.0,
            tension: 0.0,
            tension_target: 0.0,
            duck: 1.0,
            later: Vec::new(),
            #[cfg(test)]
            log: None,
            #[cfg(test)]
            clock: 0.0,
            #[cfg(test)]
            logged: [-1.0; 3],
        }
    }

    pub fn play(&mut self, i: usize, vol: f32, pitch: f32) {
        self.note(i, vol, pitch, "play");
        if let Some(s) = self.sounds[i] {
            s.play(vol, 0.0, pitch);
        }
    }

    #[cfg(test)]
    fn note(&mut self, i: usize, vol: f32, pitch: f32, op: &str) {
        if let Some(l) = &mut self.log {
            l.push_str(&format!("{:.3} {} {:.3} {:.4} {op}\n", self.clock, FILES[i], vol, pitch));
        }
    }

    #[cfg(not(test))]
    fn note(&mut self, _: usize, _: f32, _: f32, _: &str) {}

    /// Logs the three loops' levels in tests (a loop when they start, then changes).
    #[cfg(test)]
    fn note_music(&mut self, on: bool, levels: [(f32, f32); 3]) {
        if self.log.is_none() {
            return;
        }
        for (k, &(v, p)) in levels.iter().enumerate() {
            let i = [MUSIC, MUSIC_HI, TENSION][k];
            let was = self.logged[k];
            if !on {
                if was >= 0.0 {
                    self.note(i, 0.0, 1.0, "stop");
                }
                self.logged[k] = -1.0;
            } else if was < 0.0 || (v - was).abs() > 0.004 {
                self.note(i, v, p, if was < 0.0 { "loop" } else { "set" });
                self.logged[k] = v;
            }
        }
    }

    fn jitter(&mut self) -> f32 {
        self.rng.range(0.96, 1.04)
    }

    /// A level change of ±`db` dB, so a sound heard often never repeats exactly.
    fn vary(&mut self, db: f32) -> f32 {
        (self.rng.range(-db, db) / 20.0 * std::f32::consts::LN_10).exp()
    }

    /// A round starts: a jingle and the music from the top.
    pub fn start(&mut self) {
        self.stop_music();
        self.later.clear();
        self.play(START, 0.6, 1.0);
        self.hi = 0.0;
        self.start_music();
    }

    /// The loop from the top (a round start, or coming back from a pause).
    pub fn start_music(&mut self) {
        self.stop_music();
        // Both stems scheduled for the same moment stay sample-aligned.
        self.music_at = Some(sys::time() + 0.15);
    }

    pub fn stop_music(&mut self) {
        #[cfg(test)]
        self.note_music(false, [(0.0, 1.0); 3]);
        self.music_at = None;
        if let Some((a, b, c)) = self.music.take() {
            a.stop();
            b.stop();
            if let Some(c) = c {
                c.stop();
            }
        }
    }

    /// The round's verdict after a beat of quiet.
    pub fn verdict(&mut self, new_best: bool) {
        self.later.push((0.55, if new_best { BEST } else { OVER }, 0.65, 1.0));
    }

    pub fn new_best(&mut self) {
        sensors::haptic(Haptic::Success);
    }

    /// Plays this frame's cues, with their haptics.
    pub fn cues(&mut self, cues: &[Cue]) {
        for &cue in cues {
            match cue {
                Cue::Deal { golden, tier_up, .. } => {
                    let (v, p) = (0.32 * self.vary(1.5), self.jitter());
                    self.play(FLIP, v, p);
                    if golden {
                        self.play(GOLDEN, 0.6, 1.0);
                        self.duck = 0.6;
                    } else if tier_up {
                        self.play(TICK, 0.22, 2f32.powf(TICK_STEPS[2] / 12.0));
                    }
                }
                Cue::Tick { left } => {
                    // Higher and louder as it runs out.
                    let k = (3 - left.clamp(1, 3)) as usize;
                    self.play(TICK, 0.32 + 0.08 * k as f32, 2f32.powf(TICK_STEPS[k] / 12.0));
                    sensors::haptic(Haptic::Tap);
                }
                Cue::Right { streak, fast, .. } => {
                    let semis = RIGHT_STEPS[((streak.max(1) - 1) as usize).min(RIGHT_STEPS.len() - 1)];
                    self.play(RIGHT, if fast { 0.6 } else { 0.5 }, 2f32.powf(semis / 12.0));
                    sensors::haptic(Haptic::Success);
                }
                Cue::Wrong { .. } => {
                    let p = self.jitter();
                    self.play(WRONG, 0.62, p);
                    // Watt's bulb stutters as the buzzer goes.
                    self.later.push((0.06, FIZZLE, 0.38, p));
                    sensors::haptic(Haptic::Fail);
                    self.duck = 0.6;
                }
                Cue::Timeout => {
                    self.play(TIMEUP, 0.62, 1.0);
                    self.later.push((0.1, FIZZLE, 0.34, 1.0));
                    sensors::haptic(Haptic::Fail);
                    self.duck = 0.6;
                }
                Cue::MultUp { mult } => {
                    self.later.push((0.18, MULTUP, 0.55, 1.0 + 0.06 * (mult - 2) as f32));
                }
                Cue::HeartBack => self.later.push((0.3, HEART, 0.6, 1.0)),
                Cue::Ending => {
                    self.stop_music();
                    // The flicker, then the pop (the host layer times the pop).
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        #[cfg(test)]
        {
            self.clock += dt as f64;
            if self.log.is_some() && self.music_at.take().is_some() {
                self.logged = [-1.0; 3];
                self.note_music(true, [(MUSIC_VOL, 1.0), (MUSIC_VOL * HI_FLOOR, 1.0), (0.0, TENSION_PITCH)]);
            }
        }
        for i in 0..FILES.len() {
            if self.sounds[i].is_none() && self.assets[i].state() == AssetState::Ready {
                self.sounds[i] = Snd::new(self.assets[i]);
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
        if let (Some(at), Some(m), Some(h)) = (self.music_at, self.sounds[MUSIC], self.sounds[MUSIC_HI]) {
            // Late stems join where the loop would be by now (SPEC §5.6).
            let tick = self.sounds[TENSION].map(|t| t.play_looped_at(0.0, 0.0, TENSION_PITCH, at));
            self.music = Some((m.play_looped_at(MUSIC_VOL, 0.0, 1.0, at), h.play_looped_at(MUSIC_VOL * HI_FLOOR, 0.0, 1.0, at), tick));
            self.music_at = None;
        }
        self.duck += (1.0 - self.duck) * (1.0 - (-dt / 0.5).exp());
        let tau = if self.hi_target > self.hi { 0.4 } else { 1.2 };
        self.hi += (self.hi_target - self.hi) * (1.0 - (-dt / tau).exp());
        let tau = if self.tension_target > self.tension { 0.25 } else { 0.5 };
        self.tension += (self.tension_target - self.tension) * (1.0 - (-dt / tau).exp());
        // The band steps back a little as the stopwatch takes over.
        let band = self.duck * (1.0 - 0.35 * self.tension);
        let levels = [MUSIC_VOL * band, MUSIC_VOL * band * (HI_FLOOR + (1.0 - HI_FLOOR) * self.hi), TENSION_VOL * self.tension * self.tension];
        #[cfg(test)]
        if self.logged[0] >= 0.0 {
            self.note_music(true, [(levels[0], 1.0), (levels[1], 1.0), (levels[2], TENSION_PITCH)]);
        }
        if let Some((m, h, tick)) = self.music {
            m.set(levels[0], 0.0, 1.0);
            h.set(levels[1], 0.0, 1.0);
            if let Some(tick) = tick {
                tick.set(levels[2], 0.0, TENSION_PITCH);
            }
        }
    }
}
