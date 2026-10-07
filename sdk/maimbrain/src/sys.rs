//! Always-available host services (SPEC §5.1).

use crate::ffi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

pub fn log(level: Level, msg: &str) {
    unsafe { ffi::mb_log(level as u32, msg.as_ptr(), msg.len() as u32) }
}

/// Game time in seconds: the sum of every `dt` passed to `update` so far.
/// Not wall-clock time (SPEC §3).
pub fn time() -> f64 {
    unsafe { ffi::mb_time() }
}

/// Wall-clock seconds this frame that game time did *not* get: what the
/// `dt ≤ 0.1` clamp dropped after a hitch or stall (SPEC §5.1). Usually 0.
/// Music scheduled earlier is now that far ahead of game time, so a rhythm
/// game re-anchors when this is > 0. Recorded, so replays see the same
/// values. 0 across a suspend/resume (the platform pauses audio with it).
pub fn time_lost() -> f32 {
    unsafe { ffi::mb_time_lost() }
}

/// Per-session seed. Seed your own PRNG from it, e.g. [`crate::Rng`].
pub fn rand_seed() -> u64 {
    unsafe { ffi::mb_rand_seed() }
}

/// The same for every player of this game on a given UTC day: use it for
/// daily challenges everyone plays identically.
pub fn daily_seed() -> u64 {
    unsafe { ffi::mb_daily_seed() }
}

/// Where the game is in its round (SPEC §5.1). The platform lets the player
/// swipe between games unless a round is being played, and turns a finished
/// round's screen into a feed card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Round {
    /// Title or menu: nothing at stake.
    Idle = 0,
    /// A round is in progress.
    Playing = 1,
    /// The round just ended (game-over screen).
    Over = 2,
}

pub fn round(state: Round) {
    unsafe { ffi::mb_round(state as u32) }
}

/// Screen geometry in logical units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Screen {
    pub width: f32,
    pub height: f32,
    /// Safe-area insets. `inset_bottom` includes the platform's reserved
    /// feed strip; keep interactive elements out of it (SPEC §5.2).
    pub inset_top: f32,
    pub inset_right: f32,
    pub inset_bottom: f32,
    pub inset_left: f32,
    /// Device pixels per logical unit.
    pub dpr: f32,
    pub _reserved: f32,
}

/// Assets load asynchronously; readiness changes only at frame boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asset(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetState {
    Pending,
    Ready,
    Failed,
}

impl Asset {
    /// Starts loading `path` (relative to the bundle root, e.g. "assets/hit.ogg").
    pub fn load(path: &str) -> Asset {
        Asset(unsafe { ffi::mb_asset_load(path.as_ptr(), path.len() as u32) })
    }

    pub fn state(self) -> AssetState {
        match unsafe { ffi::mb_asset_state(self.0) } {
            0 => AssetState::Pending,
            1 => AssetState::Ready,
            _ => AssetState::Failed,
        }
    }

    /// The raw bytes, once ready (for .json/.bin/.txt).
    pub fn read(self) -> Option<Vec<u8>> {
        let n = unsafe { ffi::mb_asset_read(self.0, std::ptr::null_mut(), 0) };
        if n < 0 {
            return None;
        }
        let mut v = vec![0u8; n as usize];
        unsafe { ffi::mb_asset_read(self.0, v.as_mut_ptr(), n as u32) };
        Some(v)
    }
}

/// Asks the platform to open an http(s) URL. It always shows a "leaving
/// Maimbrain" interstitial first. Needs `capabilities = ["links"]`.
pub fn open_link(url: &str) -> bool {
    unsafe { ffi::mb_link_open(url.as_ptr(), url.len() as u32) == 0 }
}

pub fn screen() -> Screen {
    let mut s = Screen::default();
    unsafe { ffi::mb_screen(&mut s as *mut Screen as *mut u8) };
    s
}
