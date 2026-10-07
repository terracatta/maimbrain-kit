//! Sound playback (SPEC §5.6). Audio is output-only: it never feeds back
//! into game state, so it doesn't affect replays.
//!
//! [`Sound::play`] starts a voice as soon as it can. [`Sound::play_at`] makes
//! it *heard* when game time ([`crate::sys::time`]) reaches `at`: the platform
//! accounts for output latency and for when the frame is on screen, and a
//! voice that can only start late (its sound still decoding, `at` already
//! past) starts part-way into the sound so it stays on time. Use it for
//! anything that must land on a beat (music, rhythm cues).

use crate::ffi;
use crate::sys::Asset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sound(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Voice(pub u32);

impl Sound {
    /// None until the asset (.ogg) is ready.
    pub fn new(asset: Asset) -> Option<Sound> {
        let h = unsafe { ffi::mb_sound(asset.0) };
        (h > 0).then_some(Sound(h as u32))
    }

    /// Starts now. `vol` 0–4, `pan` −1…1, `pitch` as a playback-rate
    /// multiplier. Skipped if the sound hasn't finished decoding.
    pub fn play(self, vol: f32, pan: f32, pitch: f32) -> Voice {
        Voice(unsafe { ffi::mb_play(self.0, vol, pan, pitch, 0) })
    }

    /// Loops from now (or from when the sound has decoded, if later).
    pub fn play_looped(self, vol: f32, pan: f32, pitch: f32) -> Voice {
        Voice(unsafe { ffi::mb_play(self.0, vol, pan, pitch, 1) })
    }

    /// Heard from game time `at` (seconds, like [`crate::sys::time`]). If it
    /// can only start after `at`, it starts that far into the sound; skipped
    /// if that's more than half of it. Schedule ~0.1 s or more ahead so it
    /// starts from the top.
    pub fn play_at(self, vol: f32, pan: f32, pitch: f32, at: f64) -> Voice {
        Voice(unsafe { ffi::mb_play_at(self.0, vol, pan, pitch, 0, at) })
    }

    /// Loops with its start heard at game time `at`. Started late, it joins
    /// where the loop would be by then, so loops started with the same `at`
    /// (stems) stay sample-aligned with each other and with the game's grid.
    pub fn play_looped_at(self, vol: f32, pan: f32, pitch: f32, at: f64) -> Voice {
        Voice(unsafe { ffi::mb_play_at(self.0, vol, pan, pitch, 1, at) })
    }
}

impl Voice {
    pub fn set(self, vol: f32, pan: f32, pitch: f32) {
        unsafe { ffi::mb_voice_set(self.0, vol, pan, pitch) }
    }

    pub fn stop(self) {
        unsafe { ffi::mb_voice_stop(self.0) }
    }
}
