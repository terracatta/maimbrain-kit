//! Sounds, music and haptics from the sim's cues. Output only: nothing here
//! feeds back into the game, and pitch jitter comes from this module's own
//! fixed-seed generator, so playing sound can't change a replay.
//!
//! The files are generated (`mb music`, `mb sfx`; tools/regen.sh lists every
//! prompt): a jaunty acoustic loop, a tense layer locked to it
//! (tools/align_loop.py) that rises in twists and danger, and designed
//! effects. Landings step a soft marimba note up the G major pentatonic
//! (the music came out in G major; measured, since generators ignore keys).
//!
//! The mix (measured with the skill's scripts/mix_check.py on a bot round,
//! tests.rs `mix_log`): the music is the reference; thuds, the most frequent
//! sound, sit well under it, vary, and share `THUD_VOICES` voices (a
//! collapse would otherwise stack twenty); tonal effects play in G major.

use maimbrain::Rng;
use maimbrain::audio::{Sound as Snd, Voice};
use maimbrain::sensors::{self, Haptic};
use maimbrain::sys::{Asset, AssetState};

use crate::sim::{Cue, LIVES, Sim, Twist, TwistKind};

const FILES: [&str; 17] =
    ["start", "drop", "thud", "thud2", "land", "neat", "lost", "new", "wind", "wobble", "over", "best", "ui", "ooh", "collapse", "music", "tense"];
const START: usize = 0;
const DROP: usize = 1;
const THUD: usize = 2;
const THUD2: usize = 3;
const LAND: usize = 4;
const NEAT: usize = 5;
const LOST: usize = 6;
const NEW: usize = 7;
const WIND: usize = 8;
const WOBBLE: usize = 9;
const OVER: usize = 10;
const BEST: usize = 11;
const UI: usize = 12;
/// The crowd gasps at a near-tumble.
const OOH: usize = 13;
/// Everything slides off the plate.
const COLLAPSE: usize = 14;
const MUSIC: usize = 15;
/// The tension layer: 4 bars, locked to the music's 8 (it plays twice per loop).
const TENSE: usize = 16;
/// The landing marimba (a G) climbs G major pentatonic with the streak, D E
/// G A B D, within ±7 semitones so it never chipmunks, then starts over.
const LAND_STEPS: [f32; 6] = [-5.0, -3.0, 0.0, 2.0, 4.0, 7.0];
/// The neat sparkle (a D chord) only fits G major at these shifts.
const NEAT_STEPS: [f32; 3] = [-2.0, 0.0, 5.0];
/// Thuds share this many voices; a new one fades the oldest out.
const THUD_VOICES: usize = 4;
/// The impacts' level at full speed (the sim's `vol` scales it).
const THUD_VOL: f32 = 0.55;
/// The new-best sting rings on G#: a semitone down puts it on G.
const BEST_PITCH: f32 = 0.943_874;
/// Music level: the reference everything else is mixed against (the loop is
/// saved at -20 dBFS RMS, effects at -14, so effects play well under 1).
const MUSIC_VOL: f32 = 0.85;
/// The tower sways this hard: the crowd goes "ooh" (at most every `OOH_GAP` s).
const OOH_WOBBLE: f32 = 0.7;
const OOH_GAP: f32 = 4.0;
/// A sound that has only just been created may still be decoding (a silent
/// loop would stay silent): music waits this long before starting.
const DECODE_GRACE: f32 = 0.5;

pub struct Sound {
    assets: Vec<Asset>,
    sounds: Vec<Option<Snd>>,
    age: Vec<f32>,
    rng: Rng,
    music: Option<(Voice, Voice)>,
    want_music: bool,
    /// The tension layer's level (0–1), eased.
    hi: f32,
    /// Ducks the music under big moments; recovers by itself.
    duck: f32,
    /// A delayed sting: (seconds left, sound).
    later: Option<(f32, usize)>,
    /// The stem volumes last sent (only changes are sent).
    sent: (f32, f32),
    /// Seconds since the last "ooh"; the wobble last frame.
    ooh: f32,
    wobble: f32,
    /// Thud voices still sounding: (age, voice), oldest first.
    thuds: Vec<(f32, Option<Voice>)>,
    /// Stolen voices fading out (the 10 ms glide), stopped when this runs out.
    fading: Vec<(f32, Voice)>,
    /// Test builds: every sound event, for scripts/mix_check.py (tests.rs `mix_log`).
    #[cfg(test)]
    pub log: Option<String>,
    #[cfg(test)]
    clock: f64,
    #[cfg(test)]
    logged: [(f32, f32); 2],
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            assets: FILES.iter().map(|f| Asset::load(&format!("assets/{f}.ogg"))).collect(),
            sounds: vec![None; FILES.len()],
            age: vec![0.0; FILES.len()],
            rng: Rng::new(0x5a1ad),
            music: None,
            want_music: false,
            hi: 0.0,
            duck: 1.0,
            later: None,
            sent: (-1.0, -1.0),
            ooh: 0.0,
            wobble: 0.0,
            thuds: Vec::new(),
            fading: Vec::new(),
            #[cfg(test)]
            log: None,
            #[cfg(test)]
            clock: 0.0,
            #[cfg(test)]
            logged: [(-1.0, 0.0); 2],
        }
    }

    fn play(&mut self, i: usize, vol: f32, pitch: f32) {
        self.voice(i, vol, pitch);
    }

    fn voice(&mut self, i: usize, vol: f32, pitch: f32) -> Option<Voice> {
        self.note(i, vol, pitch, "play");
        self.sounds[i].map(|s| s.play(vol, 0.0, pitch))
    }

    /// A level change of ±`db` dB, so a sound heard often never repeats exactly.
    fn vary(&mut self, db: f32) -> f32 {
        (self.rng.range(-db, db) / 20.0 * std::f32::consts::LN_10).exp()
    }

    #[cfg(test)]
    fn note(&mut self, i: usize, vol: f32, pitch: f32, op: &str) {
        if let Some(l) = &mut self.log {
            l.push_str(&format!("{:.3} {} {:.3} {:.4} {op}\n", self.clock, FILES[i], vol, pitch));
        }
    }

    #[cfg(not(test))]
    fn note(&mut self, _: usize, _: f32, _: f32, _: &str) {}

    /// Logs the stems' levels in tests (a loop when they start, then changes).
    #[cfg(test)]
    fn note_music(&mut self, on: bool, levels: [f32; 2]) {
        if self.log.is_none() {
            return;
        }
        for (k, &v) in levels.iter().enumerate() {
            let i = [MUSIC, TENSE][k];
            let was = self.logged[k].0;
            if !on {
                if was >= 0.0 {
                    self.note(i, 0.0, 1.0, "stop");
                }
                self.logged[k] = (-1.0, 0.0);
            } else if was < 0.0 {
                self.note(i, v, 1.0, "loop");
                self.logged[k] = (v, 1.0);
            } else if (v - was).abs() > 0.004 {
                self.note(i, v, 1.0, "set");
                self.logged[k] = (v, 1.0);
            }
        }
    }

    fn jitter(&mut self) -> f32 {
        self.rng.range(0.95, 1.05)
    }

    /// A round starts: a sting and the music from the top.
    pub fn start(&mut self) {
        self.stop_music();
        self.later = None;
        self.play(START, 0.6, 1.0);
        self.want_music = true;
        self.hi = 0.0;
    }

    pub fn stop_music(&mut self) {
        self.want_music = false;
        self.pause();
    }

    /// Going off screen: silence the music; it starts again (from the top)
    /// with the next update if a round is still on.
    pub fn pause(&mut self) {
        if let Some((a, b)) = self.music.take() {
            a.stop();
            b.stop();
        }
        self.sent = (-1.0, -1.0);
        #[cfg(test)]
        self.note_music(false, [0.0; 2]);
    }

    pub fn ui(&mut self, on: bool) {
        self.play(UI, 0.5, if on { 1.12 } else { 1.0 });
    }

    /// The results card landed on a new best.
    pub fn best(&mut self) {
        self.play(BEST, 0.75, BEST_PITCH);
        sensors::haptic(Haptic::Success);
    }

    /// Plays this frame's cues. On the title card (`playing` false) only the
    /// impacts sound, quietly, and nothing buzzes.
    pub fn update(&mut self, dt: f32, cues: &[Cue], playing: bool, sim: &Sim) {
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
        }
        for t in &mut self.thuds {
            t.0 += dt;
        }
        self.thuds.retain(|t| t.0 < 0.5);
        self.fading.retain_mut(|f| {
            f.0 -= dt;
            if f.0 <= 0.0 {
                f.1.stop();
            }
            f.0 > 0.0
        });
        let card = if playing { 1.0 } else { 0.3 };
        let (mut thuds, mut lost) = (0, 0);
        for &cue in cues {
            match cue {
                Cue::Thud { vol, heavy, .. } if thuds < 2 => {
                    thuds += 1;
                    let pitch = self.jitter() * if heavy { 0.8 } else { 1.0 };
                    let which = if self.rng.f32() < 0.5 { THUD } else { THUD2 };
                    if self.thuds.len() >= THUD_VOICES {
                        let (_, old) = self.thuds.remove(0);
                        if let Some(old) = old {
                            old.set(0.0, 0.0, 1.0);
                            self.fading.push((0.03, old));
                        }
                        self.note(which, 0.0, 1.0, "stop");
                    }
                    let v = THUD_VOL * vol * card * self.vary(1.0);
                    let voice = self.voice(which, v, pitch);
                    self.thuds.push((0.0, voice));
                    if playing && vol > 0.3 {
                        sensors::haptic(if heavy { Haptic::Heavy } else { Haptic::Tap });
                    }
                }
                Cue::Drop if playing => {
                    // Tonal (a B): vary the level, barely the pitch.
                    let (v, p) = (0.3 * self.vary(1.5), self.rng.range(0.992, 1.008));
                    self.play(DROP, v, p);
                }
                Cue::Land { close: true, .. } if playing && self.ooh > 1.0 => {
                    // It only just stayed on.
                    let v = 0.5 * self.vary(1.0);
                    self.play(OOH, v, 1.0);
                    self.play(LAND, 0.42, 1.0);
                    self.ooh = 0.0;
                    sensors::haptic(Haptic::Tap);
                }
                Cue::Land { streak, neat, .. } if playing => {
                    let step = (streak.max(1) - 1) as usize;
                    let semis = LAND_STEPS[step % LAND_STEPS.len()];
                    let v = 0.42 * self.vary(1.0);
                    self.play(LAND, v, 2f32.powf(semis / 12.0));
                    if neat {
                        let semis = NEAT_STEPS[(step / 2).min(NEAT_STEPS.len() - 1)];
                        self.play(NEAT, 0.4, 2f32.powf(semis / 12.0));
                    }
                    sensors::haptic(Haptic::Tap);
                }
                Cue::Lost { counted, .. } if lost < 1 => {
                    lost += 1;
                    if playing && counted {
                        self.play(LOST, 0.45, 1.0);
                        self.duck = 0.35;
                        sensors::haptic(Haptic::Fail);
                    } else {
                        let p = self.jitter() * 1.15;
                        self.play(LOST, 0.1 * card, p);
                    }
                }
                Cue::NewKind(_) if playing => self.play(NEW, 0.5, 1.0),
                Cue::HeartBack if playing => {
                    self.play(NEW, 0.6, 1.335);
                    sensors::haptic(Haptic::Success);
                }
                Cue::Warn(kind, _) if playing => self.play(if kind == TwistKind::Gust { WIND } else { WOBBLE }, 0.6, 1.0),
                // And again, lower, as it really starts.
                Cue::Start(kind) if playing => self.play(if kind == TwistKind::Gust { WIND } else { WOBBLE }, 0.48, 0.88),
                Cue::Collapse if playing => {
                    self.play(COLLAPSE, 0.7, 1.0);
                    self.duck = 0.2;
                    sensors::haptic(Haptic::Heavy);
                }
                Cue::Over if playing => {
                    self.stop_music();
                    // A beat of quiet after the crash, then the sting.
                    self.later = Some((0.15, OVER));
                    sensors::haptic(Haptic::Fail);
                }
                _ => {}
            }
        }
        // A tower that sways hard (but hasn't gone yet) gets a gasp.
        self.ooh += dt;
        if playing && sim.ending.is_none() && sim.wobble > OOH_WOBBLE && self.wobble <= OOH_WOBBLE && self.ooh > OOH_GAP {
            let v = 0.45 * self.vary(1.0);
            self.play(OOH, v, 1.0);
            self.ooh = 0.0;
        }
        self.wobble = sim.wobble;
        if let Some((t, s)) = self.later {
            self.later = Some((t - dt, s));
            if t - dt <= 0.0 {
                self.play(s, 0.75, 1.0);
                self.later = None;
            }
        }
        self.music(dt, playing, sim);
    }

    /// Both loops start together (sample-locked: the tense one is exactly
    /// half the music's length); the tension layer rises with danger (a
    /// twist, the tower swaying, the last heart) and a little with height.
    fn music(&mut self, dt: f32, playing: bool, sim: &Sim) {
        if self.want_music && self.music.is_none() {
            let ready = [MUSIC, TENSE].iter().all(|&i| self.sounds[i].is_some() && self.age[i] >= DECODE_GRACE);
            if let (true, Some(m), Some(h)) = (ready, self.sounds[MUSIC], self.sounds[TENSE]) {
                let at = maimbrain::sys::time() + 0.1;
                self.music = Some((m.play_looped_at(MUSIC_VOL, 0.0, 1.0, at), h.play_looped_at(0.0, 0.0, 1.0, at)));
            }
        }
        let danger = sim.wobble.max(if matches!(sim.twist, Twist::Calm { .. }) { 0.0 } else { 0.8 }).max(if sim.lives == 1 && LIVES > 1 { 1.0 } else { 0.0 });
        let want = if playing && sim.ending.is_none() { danger.max((sim.height / 1600.0).min(0.3)) } else { 0.0 };
        let rate = if want > self.hi { 2.0 } else { 0.6 };
        self.hi += (want - self.hi) * (1.0 - (-dt * rate).exp());
        self.duck += (1.0 - self.duck) * (1.0 - (-dt * 1.5).exp());
        #[cfg(test)]
        if self.want_music {
            self.note_music(true, [MUSIC_VOL * self.duck, MUSIC_VOL * self.hi * self.duck]);
        }
        if let Some((m, h)) = self.music {
            let want = (MUSIC_VOL * self.duck, MUSIC_VOL * self.hi * self.duck);
            if (want.0 - self.sent.0).abs() > 0.003 || (want.1 - self.sent.1).abs() > 0.003 {
                m.set(want.0, 0.0, 1.0);
                h.set(want.1, 0.0, 1.0);
                self.sent = want;
            }
        }
    }
}
