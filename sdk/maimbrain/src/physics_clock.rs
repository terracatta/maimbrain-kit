//! The fixed-step clock both physics worlds share (docs/PHYSICS.md).
//!
//! `update(dt)` gets a varying, recorded `dt`. Physics always advances in
//! equal steps of `h`: the clock adds each `dt` to an accumulator and runs as
//! many whole steps as fit. Same `dt` sequence → same step count every
//! time, so replays reproduce the simulation exactly. What's left over (less
//! than one step) is the interpolation factor used for drawing.

/// Steps per second unless a game changes it.
pub const DEFAULT_STEP_RATE: f32 = 60.0;
/// The most steps one `step(dt)` call runs. `dt` is at most 0.1 s, i.e. 6
/// steps at 60 Hz; time beyond this is dropped rather than letting a slow
/// frame snowball into ever more work (the "spiral of death").
pub const MAX_STEPS_PER_CALL: u32 = 8;

const EPS: f64 = 1e-6;

#[derive(Clone, Debug)]
pub(crate) struct Clock {
    /// Seconds not yet simulated, always in `[0, h)` between calls.
    acc: f64,
    h: f32,
}

impl Clock {
    pub fn new() -> Clock {
        Clock { acc: 0.0, h: 1.0 / DEFAULT_STEP_RATE }
    }

    /// Step length in seconds.
    pub fn h(&self) -> f32 {
        self.h
    }

    pub fn set_rate(&mut self, hz: f32) {
        // 15–480 Hz; anything else (or NaN) keeps the current rate.
        if (15.0..=480.0).contains(&hz) {
            self.h = 1.0 / hz;
            self.acc = self.acc.min(self.h as f64 * 0.999);
        }
    }

    /// Adds `dt` and returns how many whole steps to run now.
    pub fn advance(&mut self, dt: f32) -> u32 {
        if dt.is_finite() && dt > 0.0 {
            self.acc += dt as f64;
        }
        let h = self.h as f64;
        let mut n = 0;
        // EPS absorbs f32 rounding, so dt = 0.1 is 6 steps (not 5 and a
        // sliver) and exact 1/60 frames step once each.
        while self.acc + EPS >= h && n < MAX_STEPS_PER_CALL {
            self.acc -= h;
            n += 1;
        }
        self.acc = self.acc.max(0.0);
        if self.acc + EPS >= h {
            // Over the cap: drop the backlog, keep the phase.
            self.acc %= h;
            if self.acc + EPS >= h {
                self.acc = 0.0;
            }
        }
        n
    }

    /// How far into the next step game time is, 0–1: where to draw between
    /// the last two simulated states.
    pub fn alpha(&self) -> f32 {
        ((self.acc / self.h as f64) as f32).clamp(0.0, 1.0)
    }
}

/// Collision events are collected from inside Rapier's step through this
/// (Rapier wants `Send + Sync`; there is only ever one thread).
pub(crate) struct Inbox<T>(pub std::sync::Mutex<Vec<T>>);

impl<T> Inbox<T> {
    pub fn new() -> Inbox<T> {
        Inbox(std::sync::Mutex::new(Vec::new()))
    }
    pub fn push(&self, v: T) {
        if let Ok(mut q) = self.0.lock() {
            q.push(v);
        }
    }
    pub fn take(&self) -> Vec<T> {
        self.0.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default()
    }
}

/// The shortest signed angle from `a` to `b`, in `(-π, π]`.
#[cfg(feature = "physics2d")]
pub(crate) fn angle_delta(a: f32, b: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut d = (b - a) % tau;
    if d > std::f32::consts::PI {
        d -= tau;
    } else if d <= -std::f32::consts::PI {
        d += tau;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_steps_from_uneven_frames() {
        let mut c = Clock::new();
        // 120 Hz frames: a step every other frame.
        let steps: Vec<u32> = (0..6).map(|_| c.advance(1.0 / 120.0)).collect();
        assert_eq!(steps.iter().sum::<u32>(), 3, "{steps:?}");
        // A jittery 60 Hz: the total is what the time adds up to.
        let mut c = Clock::new();
        let dts = [0.0166, 0.0168, 0.0159, 0.0175, 0.0166, 0.0167];
        let n: u32 = dts.iter().map(|&dt| c.advance(dt)).sum();
        let total: f32 = dts.iter().sum();
        assert_eq!(n, (total * 60.0) as u32);
        assert!((0.0..1.0).contains(&c.alpha()));
    }

    #[test]
    fn a_hitch_is_capped_and_bad_dt_ignored() {
        let mut c = Clock::new();
        assert_eq!(c.advance(0.1), 6);
        assert_eq!(c.advance(1.0), MAX_STEPS_PER_CALL);
        assert!(c.alpha() < 1.0);
        assert_eq!(c.advance(f32::NAN), 0);
        assert_eq!(c.advance(-1.0), 0);
    }

    #[test]
    fn exact_sixtieths_step_once_each() {
        let mut c = Clock::new();
        for _ in 0..1000 {
            assert_eq!(c.advance(1.0 / 60.0), 1);
        }
    }

    #[test]
    #[cfg(feature = "physics2d")]
    fn angle_delta_wraps() {
        assert!((angle_delta(3.0, -3.0) - (std::f32::consts::TAU - 6.0)).abs() < 1e-5);
        assert!((angle_delta(0.1, 0.3) - 0.2).abs() < 1e-6);
    }
}
