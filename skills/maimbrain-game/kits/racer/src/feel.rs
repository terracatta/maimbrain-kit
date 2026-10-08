//! The sense of speed: one number, `Feel::k`, says how fast the game should
//! *feel* right now (0 = standing still, ~0.35 at the start speed, 1 at top
//! cruise, ~1.7 boosting), and everything that sells speed reads it: the
//! camera's field of view, height, distance and vibration (`sim.rs`
//! `camera`), the wind streaks, rumble strips and post effects (`scene.rs`),
//! the screen-space speed lines and the speedo (`lib.rs`), the engine pitch
//! and the wind roar (`sound.rs`). DESIGN.md "Sense of speed" says why.
//!
//! Lives in the sim (stepped with it, no randomness), so it replays exactly.
//! It never feeds back into the rules: hitboxes, timing and the generator
//! don't read it.

use maimbrain::gfx3d::{Vec3, vec3};

use crate::sim::{MAX_SPEED, START_SPEED};

// ---- Speed-feel knobs ---------------------------------------------------
/// Feel at the start speed: above 0 so the first seconds already feel quick.
pub const FEEL_START: f32 = 0.35;
/// Feel added at full boost (on top of the speed's own share).
pub const FEEL_BOOST: f32 = 0.55;
/// Kicks (decaying extra feel) for a speed-up, a boost pad and a near miss.
pub const KICK_SPEEDUP: f32 = 0.6;
pub const KICK_BOOST: f32 = 0.9;
pub const KICK_NEAR: f32 = 0.25;
/// How fast a kick dies away (per second).
pub const KICK_DECAY: f32 = 2.2;

/// Vertical field of view (radians) at feel 0 and feel 1; boost and kicks
/// widen it further. Portrait screens are narrow, so the horizontal FOV is
/// ~56% of this: 0.92 rad vertical is only ~31° across.
pub const FOV_SLOW: f32 = 0.98;
pub const FOV_FAST: f32 = 1.2;
pub const FOV_BOOST: f32 = 0.2;
pub const FOV_KICK: f32 = 0.1;
/// Chase camera: distance behind and height above the car at feel 0 and 1.
/// It comes in closer and lower as the FOV widens, so the car stays the
/// same size while the road around it stretches and streams past faster.
pub const CAM_BACK_SLOW: f32 = 7.4;
pub const CAM_BACK_FAST: f32 = 6.0;
pub const CAM_HIGH_SLOW: f32 = 3.5;
pub const CAM_HIGH_FAST: f32 = 2.85;
/// The car surges away from the camera when it accelerates (boost) and the
/// camera catches up: meters of pull-back per m/s² of acceleration, the most
/// it pulls back, and how long it takes to catch up (s).
pub const SURGE_PER_ACCEL: f32 = 0.03;
pub const SURGE_MAX: f32 = 1.8;
pub const SURGE_TIME: f32 = 0.35;
/// Road vibration: meters of camera buzz at feel 1 (grows with feel²), and
/// its base frequency (Hz). Smooth noise, not per-frame random jitter.
pub const RUMBLE: f32 = 0.035;
pub const RUMBLE_HZ: f32 = 9.0;
/// Screen-space speed lines: they start above this feel and are fully on at 1.5.
pub const LINES_FROM: f32 = 0.75;
// -------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Feel {
    /// How fast it should feel (0 … ~2).
    pub k: f32,
    /// Decaying extra feel from events.
    pub kick: f32,
    /// Camera pull-back from acceleration (m).
    pub surge: f32,
    /// A clock for the vibration.
    pub clock: f32,
    last_speed: f32,
}

fn smooth(x: f32) -> f32 {
    let k = x.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

impl Feel {
    pub fn new() -> Feel {
        Feel { last_speed: START_SPEED, ..Default::default() }
    }

    /// A speed-up, a boost or a near miss: a burst of extra speed feel.
    pub fn kick(&mut self, amount: f32) {
        self.kick = (self.kick + amount).min(1.2);
    }

    pub fn step(&mut self, speed: f32, boost_k: f32, running: bool, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        self.clock += dt;
        self.kick *= (-KICK_DECAY * dt).exp();
        let frac = ((speed - START_SPEED) / (MAX_SPEED - START_SPEED)).clamp(0.0, 1.0);
        let base = if speed <= START_SPEED { FEEL_START * (speed / START_SPEED).clamp(0.0, 1.0) } else { FEEL_START + (1.0 - FEEL_START) * frac };
        self.k = if running { base + FEEL_BOOST * boost_k + self.kick } else { base * 0.5 };
        let accel = (speed - self.last_speed) / dt;
        self.last_speed = speed;
        let want = if running { (accel * SURGE_PER_ACCEL).clamp(-SURGE_MAX * 0.5, SURGE_MAX) } else { 0.0 };
        // Pulls back quickly, catches up slowly.
        let tau = if want > self.surge { 0.06 } else { SURGE_TIME };
        self.surge += (want - self.surge) * (1.0 - (-dt / tau).exp());
    }

    /// Feel for things capped at 1 (FOV, camera placement).
    pub fn cruise(&self) -> f32 {
        smooth(self.k.min(1.0))
    }

    pub fn fov(&self, boost_k: f32) -> f32 {
        FOV_SLOW + (FOV_FAST - FOV_SLOW) * self.cruise() + FOV_BOOST * boost_k + FOV_KICK * self.kick.min(1.0)
    }

    /// Camera offset from the car: (back, high).
    pub fn chase(&self) -> (f32, f32) {
        let c = self.cruise();
        (CAM_BACK_SLOW + (CAM_BACK_FAST - CAM_BACK_SLOW) * c + self.surge, CAM_HIGH_SLOW + (CAM_HIGH_FAST - CAM_HIGH_SLOW) * c)
    }

    /// The camera's road buzz this frame: a sum of out-of-step sines (smooth,
    /// deterministic), mostly vertical.
    pub fn rumble(&self) -> Vec3 {
        let t = self.clock * RUMBLE_HZ * std::f32::consts::TAU;
        let a = RUMBLE * self.k.min(1.8).powi(2);
        let y = (t).sin() * 0.6 + (t * 2.37 + 1.3).sin() * 0.3 + (t * 5.11 + 0.4).sin() * 0.1;
        let x = (t * 0.71 + 2.1).sin() * 0.6 + (t * 1.93).sin() * 0.4;
        vec3(x * a * 0.5, y * a, 0.0)
    }

    /// 0 … 1: how strong the screen-space speed lines are.
    pub fn lines(&self) -> f32 {
        ((self.k - LINES_FROM) / (1.5 - LINES_FROM)).clamp(0.0, 1.0)
    }
}
