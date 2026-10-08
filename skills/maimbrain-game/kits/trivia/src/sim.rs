//! The rules, with no host calls: `cargo test -p kit_trivia` runs them
//! natively (src/tests.rs has rule tests and a bot that plays like a person).
//!
//! A round is a run of questions. Each one is dealt (the card flips in),
//! asked (the countdown runs) and revealed (right: points; wrong or out of
//! time: a life). Three lives; the first question is a warm-up that can't
//! cost one. The sim never draws or plays: it pushes `Cue`s that the host
//! turns into sound, haptics and juice.

use maimbrain::Rng;

use crate::questions::{Question, bank};

// ---- Tuning knobs -----------------------------------------------------
/// Lives (hearts) per round. Raise it and every round lasts longer.
pub const LIVES: u32 = 3;
/// The clock for each question is TIME_FIXED + pace × (TIME_BASE +
/// TIME_PER_WORD × words), where words counts the question plus half the
/// answers' words (they are scanned, not read): fair for long and short
/// questions alike, and the pace squeezes everything but the fixed part.
/// Seconds every question gets to react and tap, never squeezed.
pub const TIME_FIXED: f32 = 1.2;
/// Seconds to decide, squeezed by the pace.
pub const TIME_BASE: f32 = 1.5;
/// Seconds per word to read. Raise it for slower readers.
pub const TIME_PER_WORD: f32 = 0.42;
/// The pace starts at 1 and loses this much per question dealt: the main
/// escalation (the clock gets tighter every question).
pub const PACE_DROP: f32 = 0.12;
/// The pace never drops below this.
pub const PACE_MIN: f32 = 0.45;
/// Questions dealt before the bank moves up a tier regardless of streak
/// (index of the first medium and first hard question). Lower = harder sooner.
pub const TIER2_FROM: u32 = 2;
pub const TIER3_FROM: u32 = 5;
/// A streak this long jumps up a tier early (difficulty ramps with the streak).
pub const TIER2_STREAK: u32 = 2;
pub const TIER3_STREAK: u32 = 4;
/// Daily mode deals the same questions to everyone, so its tiers follow
/// the question number only.
pub const DAILY_TIER2_FROM: u32 = 3;
pub const DAILY_TIER3_FROM: u32 = 7;
/// Points for a right answer per tier, at full speed (before multipliers).
pub const POINTS: [u32; 3] = [100, 200, 300];
/// Fraction of the points an answer gets at the buzzer (1.0 = speed doesn't matter).
pub const SPEED_FLOOR: f32 = 0.35;
/// Streak lengths at which the multiplier becomes ×2, ×3, ×4.
pub const MULT_STEPS: [u32; 3] = [3, 6, 10];
/// Every Nth question is golden: double points. 0 turns them off.
pub const GOLDEN_EVERY: u32 = 5;
/// A streak of this many (8, 16, …) gives back a lost heart. 0 turns it off.
pub const HEART_EVERY: u32 = 8;
/// The first N questions are a warm-up: a miss costs no life.
pub const WARMUP: u32 = 1;
/// Phase lengths, seconds: the card flipping in, the reveal after a right
/// and a wrong answer, and the bulb blowing at the end.
pub const DEAL_TIME: f32 = 0.5;
pub const REVEAL_RIGHT: f32 = 0.75;
pub const REVEAL_WRONG: f32 = 1.5;
pub const ENDING_TIME: f32 = 1.8;
// -----------------------------------------------------------------------

/// Where the current question is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The card flips in; answers aren't live yet.
    Deal,
    /// The clock runs; answers are live.
    Ask,
    /// The answer is shown. `picked` is the slot tapped (None: time ran out).
    Reveal { picked: Option<u8>, right: bool },
    /// Out of lives: the bulb flickers and pops (the fail animation).
    Ending,
    Over,
}

/// Something the host should make felt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    /// A new card. `tier_up`: harder than the last one.
    Deal { golden: bool, tier: u8, tier_up: bool },
    /// Answers are live.
    Ask,
    /// Whole seconds left on the clock, from 3 down.
    Tick { left: u32 },
    Right { slot: u8, points: u32, streak: u32, fast: bool },
    Wrong { slot: u8 },
    Timeout,
    /// A heart was lost (`left` remain); none on the warm-up.
    LifeLost { left: u32 },
    /// The streak bumped the multiplier.
    MultUp { mult: u32 },
    /// A long streak gave back a heart.
    HeartBack,
    /// The last life went: the fail animation starts.
    Ending,
    Over,
}

/// The question on screen, its answers in screen order.
#[derive(Clone, Copy, Debug)]
pub struct Dealt {
    pub q: Question,
    /// Screen slot → index into `q.answers` (0 is the right one).
    pub order: [u8; 4],
    pub tier: u8,
    pub golden: bool,
    /// Seconds on the clock.
    pub limit: f32,
    /// 0-based question number in this round.
    pub index: u32,
}

impl Dealt {
    pub fn answer(&self, slot: usize) -> &'static str {
        self.q.answers[self.order[slot] as usize]
    }
    pub fn right_slot(&self) -> u8 {
        self.order.iter().position(|&i| i == 0).unwrap_or(0) as u8
    }
    pub fn warmup(&self) -> bool {
        self.index < WARMUP
    }
}

pub struct Sim {
    pub daily: bool,
    pub phase: Phase,
    /// Seconds in the current phase.
    pub phase_t: f32,
    /// Seconds since the round began.
    pub t: f32,
    pub cur: Dealt,
    pub score: u32,
    pub streak: u32,
    pub best_streak: u32,
    /// Right answers this round.
    pub right: u32,
    pub lives: u32,
    pub over: bool,
    /// Seconds left on the clock when the answer came in.
    pub answered_left: f32,
    pub last_points: u32,
    pub cues: Vec<Cue>,
    seed: u64,
    /// Each tier's questions (bank indices), shuffled once per round.
    decks: [Vec<u16>; 3],
    next: [usize; 3],
    last_tick: u32,
}

/// A whole-question shuffle (Fisher–Yates) from the game's `Rng`.
fn shuffle<T>(v: &mut [T], rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        let j = (rng.next_u32() as usize) % (i + 1);
        v.swap(i, j);
    }
}

/// The streak multiplier for a streak this long.
pub fn multiplier(streak: u32) -> u32 {
    1 + MULT_STEPS.iter().filter(|&&s| streak >= s).count() as u32
}

/// Words to read: the question plus half the answers (they're scanned).
pub fn read_words(q: &Question) -> f32 {
    let w = |s: &str| s.split_whitespace().count() as f32;
    w(q.text) + 0.5 * q.answers.iter().map(|a| w(a)).sum::<f32>()
}

/// How tight the clock is for question `index`: 1 at first, down to PACE_MIN.
pub fn pace(index: u32) -> f32 {
    (1.0 - PACE_DROP * index as f32).max(PACE_MIN)
}

/// Seconds on the clock for question `q`, dealt as number `index`.
pub fn time_limit(q: &Question, index: u32) -> f32 {
    TIME_FIXED + (TIME_BASE + TIME_PER_WORD * read_words(q)) * pace(index)
}

/// Which tier question `index` comes from.
pub fn tier_for(index: u32, streak: u32, daily: bool) -> u8 {
    if daily {
        return 1 + (index >= DAILY_TIER2_FROM) as u8 + (index >= DAILY_TIER3_FROM) as u8;
    }
    let by_index = 1 + (index >= TIER2_FROM) as u8 + (index >= TIER3_FROM) as u8;
    let by_streak = 1 + (streak >= TIER2_STREAK) as u8 + (streak >= TIER3_STREAK) as u8;
    // The warm-up is always easy.
    if index < WARMUP { 1 } else { by_index.max(by_streak) }
}

pub fn golden(index: u32) -> bool {
    GOLDEN_EVERY > 0 && index % GOLDEN_EVERY == GOLDEN_EVERY - 1
}

impl Sim {
    /// A new round. `daily`: everyone gets the same questions in the same
    /// order today (seed it with `sys::daily_seed()`).
    pub fn new(seed: u64, daily: bool) -> Sim {
        let mut rng = Rng::new(seed);
        let mut decks: [Vec<u16>; 3] = Default::default();
        for (i, q) in bank().iter().enumerate() {
            decks[(q.tier - 1) as usize].push(i as u16);
        }
        for d in &mut decks {
            shuffle(d, &mut rng);
        }
        let placeholder = Dealt { q: bank()[0], order: [0, 1, 2, 3], tier: 1, golden: false, limit: 10.0, index: 0 };
        let mut s = Sim {
            daily,
            phase: Phase::Deal,
            phase_t: 0.0,
            t: 0.0,
            cur: placeholder,
            score: 0,
            streak: 0,
            best_streak: 0,
            right: 0,
            lives: LIVES,
            over: false,
            answered_left: 0.0,
            last_points: 0,
            cues: Vec::new(),
            seed,
            decks,
            next: [0; 3],
            last_tick: 0,
        };
        s.deal(0);
        s
    }

    /// Draws the next question of `tier` (another tier if that one ran out;
    /// a fresh shuffle only once the whole bank has been used).
    fn draw(&mut self, tier: u8) -> u16 {
        let t = (tier - 1) as usize;
        for k in [t, t.saturating_sub(1), (t + 1).min(2), 0, 1, 2] {
            if self.next[k] < self.decks[k].len() {
                self.next[k] += 1;
                return self.decks[k][self.next[k] - 1];
            }
        }
        // Everything asked: start the decks over (a very long round).
        self.next = [0; 3];
        self.next[t] = 1;
        self.decks[t][0]
    }

    fn deal(&mut self, index: u32) {
        let tier = tier_for(index, self.streak, self.daily);
        let q = bank()[self.draw(tier) as usize];
        // The answer order comes from the question number alone, so it is
        // the same for everyone in daily mode whatever they answered before.
        let mut rng = Rng::new(self.seed ^ (index as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut order = [0u8, 1, 2, 3];
        shuffle(&mut order, &mut rng);
        let tier_up = index > 0 && tier > self.cur.tier;
        self.cur = Dealt { q, order, tier, golden: golden(index), limit: time_limit(&q, index), index };
        self.phase = Phase::Deal;
        self.phase_t = 0.0;
        self.last_tick = 4;
        self.cues.push(Cue::Deal { golden: self.cur.golden, tier, tier_up });
    }

    /// Seconds left on the clock (frozen once answered).
    pub fn time_left(&self) -> f32 {
        match self.phase {
            Phase::Deal => self.cur.limit,
            Phase::Ask => (self.cur.limit - self.phase_t).max(0.0),
            _ => self.answered_left,
        }
    }

    /// 0…1 over the clock's last 3 seconds (0 with time to spare): the
    /// host's sweat, the red edge and the music's tension layer follow it.
    pub fn danger(&self) -> f32 {
        if self.phase != Phase::Ask {
            return 0.0;
        }
        (1.0 - self.time_left() / 3.0).clamp(0.0, 1.0)
    }

    pub fn multiplier(&self) -> u32 {
        multiplier(self.streak)
    }

    /// The player taps answer `slot` (0–3, screen order). False if answers
    /// aren't live.
    pub fn answer(&mut self, slot: u8) -> bool {
        if self.phase != Phase::Ask || slot > 3 {
            return false;
        }
        self.resolve(Some(slot));
        true
    }

    fn resolve(&mut self, picked: Option<u8>) {
        self.answered_left = self.time_left();
        let right = picked == Some(self.cur.right_slot());
        if right {
            let before = self.multiplier();
            self.streak += 1;
            self.best_streak = self.best_streak.max(self.streak);
            self.right += 1;
            let frac = self.answered_left / self.cur.limit;
            let base = POINTS[(self.cur.tier - 1) as usize] as f32 * (SPEED_FLOOR + (1.0 - SPEED_FLOOR) * frac);
            let base = ((base / 10.0).round() * 10.0) as u32;
            let points = base * self.multiplier() * if self.cur.golden { 2 } else { 1 };
            self.score += points;
            self.last_points = points;
            self.cues.push(Cue::Right { slot: picked.unwrap_or(0), points, streak: self.streak, fast: frac > 0.6 });
            if self.multiplier() > before {
                self.cues.push(Cue::MultUp { mult: self.multiplier() });
            }
            if HEART_EVERY > 0 && self.streak.is_multiple_of(HEART_EVERY) && self.lives < LIVES {
                self.lives += 1;
                self.cues.push(Cue::HeartBack);
            }
        } else {
            self.streak = 0;
            self.last_points = 0;
            self.cues.push(match picked {
                Some(slot) => Cue::Wrong { slot },
                None => Cue::Timeout,
            });
            if !self.cur.warmup() {
                self.lives = self.lives.saturating_sub(1);
                self.cues.push(Cue::LifeLost { left: self.lives });
            }
        }
        self.phase = Phase::Reveal { picked, right };
        self.phase_t = 0.0;
    }

    pub fn step(&mut self, dt: f32) {
        if self.over {
            return;
        }
        self.t += dt;
        self.phase_t += dt;
        match self.phase {
            Phase::Deal => {
                if self.phase_t >= DEAL_TIME {
                    self.phase = Phase::Ask;
                    self.phase_t -= DEAL_TIME;
                    self.cues.push(Cue::Ask);
                }
            }
            Phase::Ask => {
                let left = self.time_left();
                let whole = left.ceil() as u32;
                if whole < self.last_tick && whole <= 3 && left > 0.0 {
                    self.last_tick = whole;
                    self.cues.push(Cue::Tick { left: whole });
                }
                if left <= 0.0 {
                    self.resolve(None);
                }
            }
            Phase::Reveal { right, .. } => {
                let hold = if right { REVEAL_RIGHT } else { REVEAL_WRONG };
                if self.phase_t >= hold {
                    if self.lives == 0 {
                        self.phase = Phase::Ending;
                        self.phase_t = 0.0;
                        self.cues.push(Cue::Ending);
                    } else {
                        self.deal(self.cur.index + 1);
                    }
                }
            }
            Phase::Ending => {
                if self.phase_t >= ENDING_TIME {
                    self.phase = Phase::Over;
                    self.over = true;
                    self.cues.push(Cue::Over);
                }
            }
            Phase::Over => {}
        }
    }
}
