//! Device sensors (SPEC §5.7). Declare each one you use in the manifest's
//! `sensors`; list the ones the game can't be played without in `needs`.
//! Sensor readings are recorded per frame, so replays reproduce them.

use crate::ffi;

/// Gravity in the game's axes, in g: +x right, +y down the screen, +z out of
/// the screen toward the player. Held upright in portrait it reads about
/// (0, 1, 0); lying flat face-up, (0, 0, −1). Its length is about 1 when the
/// phone is still.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Tilt {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Tilt {
    /// Sideways tip in radians, −π/2…π/2: positive when the right edge is
    /// lower. Works the same held upright, slumped or lying flat, so it's the
    /// usual control angle for "tip left/right". Compare against the value
    /// when play started (or each level began) to make it relative.
    pub fn roll(self) -> f32 {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len < 1e-3 { 0.0 } else { (self.x / len).clamp(-1.0, 1.0).asin() }
    }

    /// Forward/back tip in radians, 0 upright to π/2 lying flat face-up
    /// (negative past upright, face toward the floor), ignoring sideways tip.
    pub fn pitch(self) -> f32 {
        (-self.z).atan2(self.y)
    }
}

/// This frame's tilt (the same value for every call within a frame), or
/// `None` before the first sample has arrived. Needs `sensors = ["tilt"]`.
pub fn tilt() -> Option<Tilt> {
    let mut t = Tilt::default();
    let r = unsafe { ffi::mb_tilt(&mut t as *mut Tilt as *mut u8) };
    (r >= 0).then_some(t)
}

/// Acceleration without gravity, in g, in the same axes as [`Tilt`]: what the
/// player's hand is doing (shakes, jabs, flicks). About zero when the phone
/// is still, ±1–3 g in a vigorous shake, briefly more on a hard jolt.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Motion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Motion {
    /// Length of the acceleration vector, in g.
    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

/// This frame's motion (one snapshot per frame), or `None` before the first
/// sample. Needs `sensors = ["motion"]`. A frame is ~1/60 s, so sum or peak-
/// track readings over time to measure a shake rather than reacting to one.
pub fn motion() -> Option<Motion> {
    let mut m = Motion::default();
    let r = unsafe { ffi::mb_motion(&mut m as *mut Motion as *mut u8) };
    (r >= 0).then_some(m)
}

/// Mic loudness this frame, 0 (silence) to 1 (shouting or blowing straight
/// into the mic), on a −60…0 dBFS scale with fast attack and slower
/// release. `None` while the mic is unavailable: before permission is
/// granted, if the player denied it, or before the first reading. Needs
/// `sensors = ["loudness"]`; show the player what to do when it's `None`.
pub fn loudness() -> Option<f32> {
    let v = unsafe { ffi::mb_loudness() };
    (v >= 0.0).then_some(v)
}

/// A haptic effect. Needs `sensors = ["haptics"]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Haptic {
    /// A light tick.
    Tap = 0,
    Success = 1,
    Fail = 2,
    /// A strong thud.
    Heavy = 3,
}

/// Plays a haptic effect. Output only (like audio, it never affects the game);
/// the platform may drop it, e.g. while the game is a feed card rather than
/// being played, or when requests come faster than about 30 per second.
pub fn haptic(kind: Haptic) {
    unsafe { ffi::mb_haptic(kind as u32) }
}
