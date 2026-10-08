//! Motion: easing curves, tweens, springs, sequences and staggers, screen
//! shake, scale punches, rolling number counters, hit-stop and pulses.
//!
//! Everything here advances only by the `dt` you pass to `update` (never a
//! clock of its own), so it replays exactly (SPEC §3): update motion state in
//! `Game::update`, read it in `Game::render`.
//!
//! ```
//! use maimbrain::motion::{Ease, Spring, Tween};
//! let mut fade = Tween::new(0.0, 1.0, 0.4, Ease::CubicOut);
//! let mut pos = Spring::bouncy(0.0, 3.0);
//! pos.target = 100.0;
//! for _ in 0..60 { fade.update(1.0 / 60.0); pos.update(1.0 / 60.0); }
//! assert_eq!(fade.value(), 1.0);
//! assert!((pos.value - 100.0).abs() < 5.0);
//! ```

use std::f32::consts::PI;

/// Easing curves (easings.net names): `t` 0→1 maps to 0→1. Back and
/// elastic overshoot past 1 (and under 0); bounce stays within 0…1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ease {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    QuintIn,
    QuintOut,
    QuintInOut,
    SineIn,
    SineOut,
    SineInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    CircIn,
    CircOut,
    CircInOut,
    BackIn,
    BackOut,
    BackInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl Ease {
    /// Every curve, for galleries and tests.
    pub const ALL: [Ease; 31] = [
        Ease::Linear, Ease::QuadIn, Ease::QuadOut, Ease::QuadInOut, Ease::CubicIn, Ease::CubicOut, Ease::CubicInOut,
        Ease::QuartIn, Ease::QuartOut, Ease::QuartInOut, Ease::QuintIn, Ease::QuintOut, Ease::QuintInOut,
        Ease::SineIn, Ease::SineOut, Ease::SineInOut, Ease::ExpoIn, Ease::ExpoOut, Ease::ExpoInOut,
        Ease::CircIn, Ease::CircOut, Ease::CircInOut, Ease::BackIn, Ease::BackOut, Ease::BackInOut,
        Ease::ElasticIn, Ease::ElasticOut, Ease::ElasticInOut, Ease::BounceIn, Ease::BounceOut, Ease::BounceInOut,
    ];

    /// The curve at `t` (clamped to 0…1).
    pub fn at(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let inout = |f: fn(f32) -> f32, t: f32| if t < 0.5 { f(2.0 * t) / 2.0 } else { 1.0 - f(2.0 - 2.0 * t) / 2.0 };
        let out = |f: fn(f32) -> f32, t: f32| 1.0 - f(1.0 - t);
        use Ease::*;
        match self {
            Linear => t,
            QuadIn => t * t,
            QuadOut => out(|t| t * t, t),
            QuadInOut => inout(|t| t * t, t),
            CubicIn => t * t * t,
            CubicOut => out(|t| t * t * t, t),
            CubicInOut => inout(|t| t * t * t, t),
            QuartIn => t.powi(4),
            QuartOut => out(|t| t.powi(4), t),
            QuartInOut => inout(|t| t.powi(4), t),
            QuintIn => t.powi(5),
            QuintOut => out(|t| t.powi(5), t),
            QuintInOut => inout(|t| t.powi(5), t),
            SineIn => sine_in(t),
            SineOut => out(sine_in, t),
            SineInOut => inout(sine_in, t),
            ExpoIn => expo_in(t),
            ExpoOut => out(expo_in, t),
            ExpoInOut => inout(expo_in, t),
            CircIn => circ_in(t),
            CircOut => out(circ_in, t),
            CircInOut => inout(circ_in, t),
            BackIn => back_in(t),
            BackOut => out(back_in, t),
            BackInOut => inout(back_in, t),
            ElasticIn => elastic_in(t),
            ElasticOut => out(elastic_in, t),
            ElasticInOut => inout(elastic_in, t),
            BounceIn => out(bounce_out, t),
            BounceOut => bounce_out(t),
            BounceInOut => inout(|t| 1.0 - bounce_out(1.0 - t), t),
        }
    }
}

fn sine_in(t: f32) -> f32 {
    1.0 - (t * PI / 2.0).cos()
}
fn expo_in(t: f32) -> f32 {
    if t <= 0.0 { 0.0 } else { (10.0 * t - 10.0).exp2() }
}
fn circ_in(t: f32) -> f32 {
    1.0 - (1.0 - t * t).max(0.0).sqrt()
}
fn back_in(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    (C1 + 1.0) * t * t * t - C1 * t * t
}
fn elastic_in(t: f32) -> f32 {
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    -(10.0 * t - 10.0).exp2() * ((10.0 * t - 10.75) * (2.0 * PI / 3.0)).sin()
}
fn bounce_out(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let t = t - 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        let t = t - 2.25 / D;
        N * t * t + 0.9375
    } else {
        let t = t - 2.625 / D;
        N * t * t + 0.984375
    }
}

/// `ease(e, t)` = `e.at(t)`.
pub fn ease(e: Ease, t: f32) -> f32 {
    e.at(t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Where `v` sits between `a` and `b` (0 at a, 1 at b; not clamped).
pub fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    if a == b { 0.0 } else { (v - a) / (b - a) }
}

/// Maps `v` from `a0..a1` to `b0..b1`, clamped.
pub fn remap(v: f32, a0: f32, a1: f32, b0: f32, b1: f32) -> f32 {
    lerp(b0, b1, inv_lerp(a0, a1, v).clamp(0.0, 1.0))
}

/// Frame-rate independent smoothing: moves `current` toward `target`,
/// closing a fraction `1 − e^(−rate·dt)` of the gap (rate ≈ 1 / time constant).
pub fn damp(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    target + (current - target) * (-rate * dt).exp()
}

/// Moves `current` toward `target` by at most `max_step`.
pub fn approach(current: f32, target: f32, max_step: f32) -> f32 {
    if current < target { (current + max_step).min(target) } else { (current - max_step).max(target) }
}

/// Values a [`Tween`] can animate.
pub trait Lerp: Copy {
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

impl<const N: usize> Lerp for [f32; N] {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Repeat {
    #[default]
    Once,
    /// Starts over at the end.
    Loop,
    /// Plays forward, then backward, forever (a yo-yo).
    PingPong,
}

/// A value easing from `from` to `to` over `duration` seconds, after `delay`.
#[derive(Clone, Copy, Debug)]
pub struct Tween<T: Lerp = f32> {
    pub from: T,
    pub to: T,
    pub duration: f32,
    pub delay: f32,
    pub ease: Ease,
    pub repeat: Repeat,
    t: f32,
}

impl<T: Lerp> Tween<T> {
    pub fn new(from: T, to: T, duration: f32, ease: Ease) -> Self {
        Tween { from, to, duration: duration.max(1e-6), delay: 0.0, ease, repeat: Repeat::Once, t: 0.0 }
    }

    /// Already at `v`, with nothing to do (retarget it later).
    pub fn at_rest(v: T) -> Self {
        let mut t = Tween::new(v, v, 1e-6, Ease::Linear);
        t.t = 1.0;
        t
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds;
        self
    }

    pub fn repeat(mut self, r: Repeat) -> Self {
        self.repeat = r;
        self
    }

    pub fn update(&mut self, dt: f32) {
        self.t += dt;
    }

    /// Seconds since the start, including the delay.
    pub fn elapsed(&self) -> f32 {
        self.t
    }

    /// Linear progress 0…1 within one play (after repeats are applied).
    pub fn progress(&self) -> f32 {
        let t = ((self.t - self.delay) / self.duration).max(0.0);
        match self.repeat {
            Repeat::Once => t.min(1.0),
            Repeat::Loop => t.fract(),
            Repeat::PingPong => {
                let k = t % 2.0;
                if k > 1.0 { 2.0 - k } else { k }
            }
        }
    }

    pub fn value(&self) -> T {
        T::lerp(self.from, self.to, self.ease.at(self.progress()))
    }

    /// True once a `Once` tween has reached `to` (never for repeating ones).
    pub fn done(&self) -> bool {
        self.repeat == Repeat::Once && self.t >= self.delay + self.duration
    }

    pub fn restart(&mut self) {
        self.t = 0.0;
    }

    /// Starts a new tween from the current value to `to` (no delay).
    pub fn retarget(&mut self, to: T, duration: f32) {
        self.from = self.value();
        self.to = to;
        self.duration = duration.max(1e-6);
        self.delay = 0.0;
        self.t = 0.0;
    }
}

/// A damped spring pulling `value` toward `target` (framer-motion style).
/// Integrated exactly for any `dt`, so it's stable at any frame rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    /// Natural frequency in Hz: how fast it moves (2–4 snappy UI, 1 lazy).
    pub frequency: f32,
    /// 1 = critically damped (fastest without overshoot), < 1 bouncy, > 1 sluggish.
    pub damping: f32,
}

impl Spring {
    pub fn new(value: f32, frequency: f32, damping: f32) -> Spring {
        Spring { value, velocity: 0.0, target: value, frequency, damping }
    }

    /// No overshoot: smooth follow, camera moves, sliders.
    pub fn critical(value: f32, frequency: f32) -> Spring {
        Spring::new(value, frequency, 1.0)
    }

    /// A lively overshoot or two: popping UI, landing, scale punches.
    pub fn bouncy(value: f32, frequency: f32) -> Spring {
        Spring::new(value, frequency, 0.4)
    }

    /// Lots of wobble: jelly, springy hinges.
    pub fn wobbly(value: f32, frequency: f32) -> Spring {
        Spring::new(value, frequency, 0.2)
    }

    pub fn to(mut self, target: f32) -> Spring {
        self.target = target;
        self
    }

    /// Adds velocity (units per second): a flick.
    pub fn kick(&mut self, velocity: f32) {
        self.velocity += velocity;
    }

    /// Jumps to `v` at rest.
    pub fn snap(&mut self, v: f32) {
        self.value = v;
        self.target = v;
        self.velocity = 0.0;
    }

    pub fn settled(&self) -> bool {
        (self.value - self.target).abs() < 1e-3 && self.velocity.abs() < 1e-2
    }

    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        if self.settled() {
            self.value = self.target;
            self.velocity = 0.0;
            return;
        }
        let w = (self.frequency * 2.0 * PI).max(1e-3);
        let z = self.damping.max(0.0);
        let (x0, v0) = (self.value - self.target, self.velocity);
        let (x, v) = if (z - 1.0).abs() < 1e-3 {
            let e = (-w * dt).exp();
            let b = v0 + w * x0;
            ((x0 + b * dt) * e, (b - w * (x0 + b * dt)) * e)
        } else if z < 1.0 {
            let wd = w * (1.0 - z * z).sqrt();
            let e = (-z * w * dt).exp();
            let (s, c) = (wd * dt).sin_cos();
            let b = (v0 + z * w * x0) / wd;
            (e * (x0 * c + b * s), e * ((b * wd - z * w * x0) * c - (x0 * wd + z * w * b) * s))
        } else {
            let q = (z * z - 1.0).sqrt();
            let (r1, r2) = (-w * (z - q), -w * (z + q));
            let c1 = (v0 - r2 * x0) / (r1 - r2);
            let c2 = x0 - c1;
            let (e1, e2) = ((r1 * dt).exp(), (r2 * dt).exp());
            (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        };
        self.value = self.target + x;
        self.velocity = v;
    }
}

/// One value moved by a list of steps: `to` (ease there), `wait`, `set`.
/// ```
/// use maimbrain::motion::{Ease, Seq};
/// let mut s = Seq::new(0.0).to(1.0, 0.2, Ease::BackOut).wait(1.0).to(0.0, 0.3, Ease::QuadIn);
/// s.update(0.6);
/// assert_eq!(s.value(), 1.0);
/// ```
#[derive(Clone, Debug)]
pub struct Seq {
    start: f32,
    steps: Vec<(f32, f32, Ease)>,
    t: f32,
    pub looping: bool,
}

impl Seq {
    pub fn new(start: f32) -> Seq {
        Seq { start, steps: Vec::new(), t: 0.0, looping: false }
    }
    pub fn to(mut self, value: f32, duration: f32, ease: Ease) -> Seq {
        self.steps.push((value, duration.max(0.0), ease));
        self
    }
    pub fn wait(self, duration: f32) -> Seq {
        let v = self.end_value();
        self.to(v, duration, Ease::Linear)
    }
    /// Jumps to `value` instantly.
    pub fn set(self, value: f32) -> Seq {
        self.to(value, 0.0, Ease::Linear)
    }
    pub fn looped(mut self) -> Seq {
        self.looping = true;
        self
    }
    fn end_value(&self) -> f32 {
        self.steps.last().map_or(self.start, |s| s.0)
    }
    pub fn duration(&self) -> f32 {
        self.steps.iter().map(|s| s.1).sum()
    }
    pub fn update(&mut self, dt: f32) {
        self.t += dt;
    }
    pub fn restart(&mut self) {
        self.t = 0.0;
    }
    pub fn done(&self) -> bool {
        !self.looping && self.t >= self.duration()
    }
    pub fn value(&self) -> f32 {
        let total = self.duration();
        let mut t = if self.looping && total > 0.0 { self.t % total } else { self.t };
        let mut from = self.start;
        for &(to, d, e) in &self.steps {
            if t < d {
                return lerp(from, to, e.at(t / d));
            }
            t -= d;
            from = to;
        }
        from
    }
}

/// Progress 0…1 of item `index` in a staggered group: item i starts
/// `i × each` seconds after `t` = 0 and takes `duration`.
pub fn stagger(t: f32, index: usize, each: f32, duration: f32) -> f32 {
    ((t - index as f32 * each) / duration.max(1e-6)).clamp(0.0, 1.0)
}

/// Counts up from 0; `fired` when it passes `period` (and wraps).
#[derive(Clone, Copy, Debug, Default)]
pub struct Timer {
    pub t: f32,
    pub period: f32,
}

impl Timer {
    pub fn new(period: f32) -> Timer {
        Timer { t: 0.0, period }
    }
    /// True on the update that crosses the period (once per period).
    pub fn update(&mut self, dt: f32) -> bool {
        self.t += dt;
        if self.period > 0.0 && self.t >= self.period {
            self.t %= self.period;
            true
        } else {
            false
        }
    }
    pub fn progress(&self) -> f32 {
        if self.period > 0.0 { (self.t / self.period).min(1.0) } else { 0.0 }
    }
}

/// A smooth pseudo-random signal in −1…1 (value noise on a seeded lattice).
pub fn noise1(seed: u32, x: f32) -> f32 {
    let i = x.floor();
    let f = x - i;
    let h = |n: i32| {
        let mut v = (n as u32).wrapping_mul(0x9E37_79B1) ^ seed.wrapping_mul(0x85EB_CA6B);
        v ^= v >> 15;
        v = v.wrapping_mul(0x2C1B_3C6D);
        v ^= v >> 12;
        (v & 0xffff) as f32 / 32767.5 - 1.0
    };
    let u = f * f * (3.0 - 2.0 * f);
    lerp(h(i as i32), h(i as i32 + 1), u)
}

/// Screen shake with "trauma": `add` some (0…1) on a hit; the shake grows
/// with trauma² and decays on its own. Apply it around a frame with
/// [`Shake::apply`] (inside `gfx2d::push`/`pop`).
#[derive(Clone, Copy, Debug)]
pub struct Shake {
    pub trauma: f32,
    /// Offset at full trauma, in logical units.
    pub max_offset: f32,
    /// Rotation at full trauma, in radians.
    pub max_angle: f32,
    /// Trauma lost per second.
    pub decay: f32,
    /// Shake frequency in Hz.
    pub frequency: f32,
    pub seed: u32,
    t: f32,
}

impl Default for Shake {
    fn default() -> Self {
        Shake::new()
    }
}

impl Shake {
    pub fn new() -> Shake {
        Shake { trauma: 0.0, max_offset: 14.0, max_angle: 0.06, decay: 1.8, frequency: 22.0, seed: 7, t: 0.0 }
    }
    /// Small hit ≈ 0.25, big hit ≈ 0.5, death ≈ 0.8.
    pub fn add(&mut self, trauma: f32) {
        self.trauma = (self.trauma + trauma).clamp(0.0, 1.0);
    }
    pub fn update(&mut self, dt: f32) {
        self.t += dt;
        self.trauma = (self.trauma - self.decay * dt).max(0.0);
    }
    fn amount(&self) -> f32 {
        self.trauma * self.trauma
    }
    /// The current offset (dx, dy).
    pub fn offset(&self) -> (f32, f32) {
        let (k, x) = (self.amount() * self.max_offset, self.t * self.frequency);
        (k * noise1(self.seed, x), k * noise1(self.seed ^ 0x5bd1, x))
    }
    pub fn angle(&self) -> f32 {
        self.amount() * self.max_angle * noise1(self.seed ^ 0x27d4, self.t * self.frequency)
    }
    /// Translates and rotates about (cx, cy). Call between push and pop.
    pub fn apply(&self, cx: f32, cy: f32) {
        if self.trauma <= 0.0 {
            return;
        }
        let (dx, dy) = self.offset();
        crate::gfx2d::translate(cx + dx, cy + dy);
        crate::gfx2d::rotate(self.angle());
        crate::gfx2d::translate(-cx, -cy);
    }
}

/// A scale punch: `kick` makes `scale()` jump above 1 and spring back with
/// a wobble. Score labels, buttons, anything that just changed.
#[derive(Clone, Copy, Debug)]
pub struct Punch {
    pub spring: Spring,
}

impl Default for Punch {
    fn default() -> Self {
        Punch::new()
    }
}

impl Punch {
    pub fn new() -> Punch {
        Punch { spring: Spring::new(0.0, 4.5, 0.32) }
    }
    /// 0.15 subtle, 0.3 lively, 0.6 huge.
    pub fn kick(&mut self, amount: f32) {
        let cap = amount.abs() * 2.0;
        self.spring.value = (self.spring.value + amount).max(-cap).min(cap);
        self.spring.velocity = 0.0;
    }
    pub fn update(&mut self, dt: f32) {
        self.spring.update(dt);
    }
    pub fn scale(&self) -> f32 {
        1.0 + self.spring.value
    }
}

/// A number that rolls toward its target (scores, coins), with a punch each
/// time it's set. `update` says when the shown value changed (tick sounds).
#[derive(Clone, Copy, Debug)]
pub struct Counter {
    from: f64,
    target: i64,
    t: f32,
    duration: f32,
    pub ease: Ease,
    /// Longest roll, in seconds (bigger jumps roll longer, up to this).
    pub max_duration: f32,
    pub punch: Punch,
    last_shown: i64,
}

impl Counter {
    pub fn new(value: i64) -> Counter {
        Counter { from: value as f64, target: value, t: 1.0, duration: 1.0, ease: Ease::QuartOut, max_duration: 1.2, punch: Punch::new(), last_shown: value }
    }
    /// Rolls from what's shown now to `target`.
    pub fn set(&mut self, target: i64) {
        if target == self.target {
            return;
        }
        self.from = self.shown_f();
        let delta = (target as f64 - self.from).abs().max(1.0);
        self.duration = (0.25 + 0.22 * delta.log10() as f32).min(self.max_duration).max(0.2);
        self.target = target;
        self.t = 0.0;
        self.punch.kick(0.25);
    }
    /// Rolls with a fixed duration (e.g. a results screen's big roll-up).
    pub fn roll_to(&mut self, target: i64, duration: f32) {
        self.from = self.shown_f();
        self.target = target;
        self.duration = duration.max(1e-3);
        self.t = 0.0;
    }
    /// Shows `v` at once.
    pub fn snap(&mut self, v: i64) {
        self.from = v as f64;
        self.target = v;
        self.t = self.duration;
        self.last_shown = v;
    }
    /// Advances; true if the shown number changed.
    pub fn update(&mut self, dt: f32) -> bool {
        self.t += dt;
        self.punch.update(dt);
        let v = self.value();
        let changed = v != self.last_shown;
        self.last_shown = v;
        changed
    }
    fn shown_f(&self) -> f64 {
        let k = self.ease.at(self.t / self.duration) as f64;
        self.from + (self.target as f64 - self.from) * k
    }
    /// The number to draw.
    pub fn value(&self) -> i64 {
        if self.done() { self.target } else { self.shown_f().round() as i64 }
    }
    pub fn target(&self) -> i64 {
        self.target
    }
    pub fn done(&self) -> bool {
        self.t >= self.duration
    }
    pub fn scale(&self) -> f32 {
        self.punch.scale()
    }
}

/// Hit-stop (freeze frame): `freeze` for 50–80 ms on a big impact, and run
/// the simulation with `step(dt)`, which is 0 while frozen.
#[derive(Clone, Copy, Debug, Default)]
pub struct HitStop {
    left: f32,
}

impl HitStop {
    pub fn freeze(&mut self, seconds: f32) {
        self.left = self.left.max(seconds);
    }
    /// The dt the simulation should use this frame (what's left after the freeze ends).
    pub fn step(&mut self, dt: f32) -> f32 {
        let used = self.left.min(dt);
        self.left -= used;
        dt - used
    }
    pub fn frozen(&self) -> bool {
        self.left > 0.0
    }
}

/// A one-shot 1 → 0 fade (flashes, glows, "hit" tints): `fire`, `update`,
/// read `value()`.
#[derive(Clone, Copy, Debug)]
pub struct Pulse {
    pub duration: f32,
    pub ease: Ease,
    t: f32,
}

impl Pulse {
    pub fn new(duration: f32) -> Pulse {
        Pulse { duration: duration.max(1e-6), ease: Ease::QuadOut, t: f32::INFINITY }
    }
    pub fn fire(&mut self) {
        self.t = 0.0;
    }
    pub fn update(&mut self, dt: f32) {
        self.t += dt;
    }
    pub fn active(&self) -> bool {
        self.t < self.duration
    }
    /// 1 when fired, easing to 0 over `duration`.
    pub fn value(&self) -> f32 {
        if self.t >= self.duration { 0.0 } else { 1.0 - self.ease.at(self.t / self.duration) }
    }
    /// Seconds since `fire`.
    pub fn age(&self) -> f32 {
        self.t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ease_starts_at_0_and_ends_at_1() {
        for e in Ease::ALL {
            assert!(e.at(0.0).abs() < 1e-5, "{e:?}(0) = {}", e.at(0.0));
            assert!((e.at(1.0) - 1.0).abs() < 1e-5, "{e:?}(1) = {}", e.at(1.0));
            assert!(e.at(0.5).is_finite());
        }
        // Symmetric in/outs pass through the middle.
        for e in [Ease::QuadInOut, Ease::CubicInOut, Ease::SineInOut, Ease::ExpoInOut, Ease::BackInOut, Ease::ElasticInOut, Ease::BounceInOut] {
            assert!((e.at(0.5) - 0.5).abs() < 1e-4, "{e:?}");
        }
        assert!(Ease::BackOut.at(0.6) > 1.0, "back overshoots");
        assert!(Ease::BackIn.at(0.2) < 0.0, "back pulls back first");
        assert!((0..=100).all(|i| (0.0..=1.0001).contains(&Ease::BounceOut.at(i as f32 / 100.0))));
    }

    #[test]
    fn tweens_delay_repeat_and_retarget() {
        let mut t = Tween::new(10.0, 20.0, 1.0, Ease::Linear).delay(0.5);
        t.update(0.5);
        assert_eq!(t.value(), 10.0);
        t.update(0.5);
        assert!((t.value() - 15.0).abs() < 1e-5);
        t.update(1.0);
        assert!(t.done() && t.value() == 20.0);
        t.retarget(0.0, 2.0);
        t.update(1.0);
        assert!((t.value() - 10.0).abs() < 1e-5);
        let mut p = Tween::new(0.0, 1.0, 1.0, Ease::Linear).repeat(Repeat::PingPong);
        p.update(1.5);
        assert!((p.value() - 0.5).abs() < 1e-5 && !p.done());
        let mut v = Tween::new([0.0, 0.0], [2.0, 4.0], 1.0, Ease::Linear);
        v.update(0.5);
        assert_eq!(v.value(), [1.0, 2.0]);
    }

    #[test]
    fn springs_settle_without_blowing_up_at_any_dt() {
        for damping in [0.2, 0.4, 1.0, 2.0] {
            for dt in [1.0 / 120.0, 1.0 / 60.0, 0.1] {
                let mut s = Spring::new(0.0, 3.0, damping).to(100.0);
                let mut peak: f32 = 0.0;
                for _ in 0..(4.0 / dt) as usize {
                    s.update(dt);
                    peak = peak.max(s.value);
                }
                assert!((s.value - 100.0).abs() < 1.0, "damping {damping} dt {dt}: {}", s.value);
                if damping >= 1.0 {
                    assert!(peak <= 100.0 + 1e-3, "critical/over-damped doesn't overshoot ({peak})");
                } else {
                    assert!(peak > 100.0, "bouncy overshoots");
                }
            }
        }
        // Same result whether stepped in one go or in pieces of the same total (exact integration).
        let (mut a, mut b) = (Spring::bouncy(0.0, 2.0).to(1.0), Spring::bouncy(0.0, 2.0).to(1.0));
        a.update(0.1);
        b.update(0.05);
        b.update(0.05);
        assert!((a.value - b.value).abs() < 1e-4);
    }

    #[test]
    fn sequences_and_staggers() {
        let mut s = Seq::new(0.0).to(1.0, 1.0, Ease::Linear).wait(1.0).set(5.0).to(0.0, 1.0, Ease::Linear);
        assert_eq!(s.duration(), 3.0);
        s.update(0.5);
        assert!((s.value() - 0.5).abs() < 1e-5);
        s.update(1.0);
        assert_eq!(s.value(), 1.0);
        s.update(1.0);
        assert!((s.value() - 2.5).abs() < 1e-5);
        assert!((stagger(0.3, 2, 0.1, 0.2) - 0.5).abs() < 1e-5);
        assert_eq!(stagger(0.0, 2, 0.1, 0.2), 0.0);
    }

    #[test]
    fn counters_roll_and_report_ticks() {
        let mut c = Counter::new(0);
        c.set(1000);
        let mut ticks = 0;
        let mut last = 0;
        for _ in 0..120 {
            if c.update(1.0 / 60.0) {
                ticks += 1;
            }
            assert!(c.value() >= last, "rolls up monotonically");
            last = c.value();
        }
        assert_eq!(c.value(), 1000);
        assert!(ticks > 10);
        assert!(c.done());
    }

    #[test]
    fn hit_stop_eats_dt_then_releases_it() {
        let mut h = HitStop::default();
        h.freeze(0.05);
        assert_eq!(h.step(1.0 / 60.0), 0.0);
        assert_eq!(h.step(1.0 / 60.0), 0.0);
        let rest = h.step(1.0 / 60.0);
        assert!(rest > 0.0 && rest < 1.0 / 60.0);
        assert_eq!(h.step(0.016), 0.016);
    }

    #[test]
    fn shake_decays_and_noise_is_bounded() {
        let mut s = Shake::new();
        s.add(1.0);
        s.update(0.1);
        let (dx, dy) = s.offset();
        assert!(dx.abs() <= s.max_offset && dy.abs() <= s.max_offset);
        for _ in 0..60 {
            s.update(1.0 / 60.0);
        }
        assert_eq!(s.trauma, 0.0);
        assert_eq!(s.offset(), (0.0, 0.0));
        assert!((0..1000).all(|i| noise1(3, i as f32 * 0.37).abs() <= 1.0));
    }
}
