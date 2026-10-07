//! Sound playback (SPEC §5.6). Audio is output-only: it never feeds back
//! into game state, so it doesn't affect replays.

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

    /// `vol` 0–4, `pan` −1…1, `pitch` as a playback-rate multiplier.
    pub fn play(self, vol: f32, pan: f32, pitch: f32) -> Voice {
        Voice(unsafe { ffi::mb_play(self.0, vol, pan, pitch, 0) })
    }

    pub fn play_looped(self, vol: f32, pan: f32, pitch: f32) -> Voice {
        Voice(unsafe { ffi::mb_play(self.0, vol, pan, pitch, 1) })
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
