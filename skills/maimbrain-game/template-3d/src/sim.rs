//! The rules, with no host calls: `cargo test -p NAME` runs them natively.
//! The camera and the tap test live here too: `Camera::project` is the same
//! projection the host uses (SPEC §5.4), in plain Rust.

use maimbrain::Rng;
use maimbrain::gfx3d::{Camera, Vec3, look_at, vec3};

/// The logical screen (manifest.toml's `logical_size`).
pub const W: f32 = 360.0;
pub const H: f32 = 640.0;
/// Seconds on the clock at the start; each catch adds some back.
pub const START_CLOCK: f32 = 6.0;
pub const CATCH_BONUS: f32 = 1.5;
/// How close to the drone (logical units) a tap must land.
pub const TAP_RADIUS: f32 = 46.0;
/// Radians per second along the flight loop, and the speed-up per catch.
const START_SPEED: f32 = 0.9;
const SPEED_UP: f32 = 0.12;

/// A drone loops over the platform; tap it before the clock runs out.
pub struct Sim {
    /// Seconds since the round started.
    pub t: f32,
    /// How far along its loop the drone is (radians).
    pub phase: f32,
    pub speed: f32,
    pub clock: f32,
    pub score: u32,
    pub over: bool,
    rng: Rng,
}

impl Sim {
    pub fn new(seed: u64) -> Sim {
        Sim { t: 0.0, phase: 0.0, speed: START_SPEED, clock: START_CLOCK, score: 0, over: false, rng: Rng::new(seed) }
    }

    /// Where the drone is: a figure-eight over the platform.
    pub fn drone(&self) -> Vec3 {
        let a = self.phase;
        vec3(1.7 * a.sin(), 1.5 + 0.5 * (a * 2.0).sin(), 1.2 * (a * 2.0).sin() - 0.4)
    }

    /// The drone's velocity (m/s), e.g. for `Emitter::emit_moving`.
    pub fn drone_velocity(&self) -> Vec3 {
        let a = self.phase;
        vec3(1.7 * a.cos(), 1.0 * (a * 2.0).cos(), 2.4 * (a * 2.0).cos()) * self.speed
    }

    /// The camera: a slow sway, a function of time only.
    pub fn camera(&self) -> Camera {
        let pos = vec3(0.6 * (self.t * 0.2).sin(), 4.0, 8.5);
        let target = vec3(0.0, 1.0, 0.0);
        Camera { pos, rot: look_at(pos, target, Vec3::Y), fov_y: 0.9, near: 0.1, far: 100.0 }
    }

    pub fn step(&mut self, dt: f32) {
        if self.over {
            return;
        }
        self.t += dt;
        self.phase += self.speed * dt;
        self.clock -= dt;
        if self.clock <= 0.0 {
            self.clock = 0.0;
            self.over = true;
        }
    }

    /// A tap at logical (x, y). On a catch the drone jumps to a new spot on
    /// its loop and speeds up; returns where it was caught.
    pub fn tap(&mut self, x: f32, y: f32) -> Option<Vec3> {
        let at = self.drone();
        let p = self.camera().project(at, W, H);
        if self.over || !p.on_screen || (p.x - x).hypot(p.y - y) > TAP_RADIUS {
            return None;
        }
        self.score += 1;
        self.speed += SPEED_UP;
        self.clock = (self.clock + CATCH_BONUS).min(START_CLOCK);
        self.phase += self.rng.range(1.5, 3.5);
        Some(at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen_pos(s: &Sim) -> (f32, f32) {
        let p = s.camera().project(s.drone(), W, H);
        assert!(p.on_screen, "the drone is always in view");
        (p.x, p.y)
    }

    #[test]
    fn first_seconds_cannot_fail() {
        let mut s = Sim::new(1);
        for _ in 0..(60 * 3) {
            s.step(1.0 / 60.0);
        }
        assert!(!s.over);
    }

    #[test]
    fn tapping_the_drone_scores_and_missing_does_not() {
        let mut s = Sim::new(1);
        s.step(0.5);
        let (x, y) = screen_pos(&s);
        assert!(s.tap(x + 300.0, y).is_none());
        let before = s.drone();
        assert_eq!(s.tap(x + 10.0, y - 10.0), Some(before));
        assert_eq!(s.score, 1);
        assert!(s.drone().distance(before) > 0.5, "it jumps away");
    }

    #[test]
    fn the_drone_stays_in_view_and_the_clock_ends_the_round() {
        let mut s = Sim::new(7);
        while !s.over {
            screen_pos(&s);
            s.step(1.0 / 60.0);
        }
        assert!((s.t - START_CLOCK).abs() < 0.05);
    }

    #[test]
    fn same_seed_same_game() {
        let play = |seed| {
            let mut s = Sim::new(seed);
            for i in 0..600 {
                s.step(1.0 / 60.0);
                if i % 50 == 0 {
                    let (x, y) = screen_pos(&s);
                    s.tap(x, y);
                }
            }
            (s.score, s.phase.to_bits())
        };
        assert_eq!(play(3), play(3));
    }
}
