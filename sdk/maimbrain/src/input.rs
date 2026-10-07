//! Raw input events (SPEC §5.2). Events arrive at frame boundaries; poll them
//! in `update`. Coordinates are logical units. `time` is the game time the
//! event itself happened: within the span since the previous update
//! (`sys::time() - dt ..= sys::time()`), in order, never ahead of `sys::time()`.

use crate::ffi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    TouchDown = 1,
    TouchMove = 2,
    TouchUp = 3,
    TouchCancel = 4,
    KeyDown = 5,
    KeyUp = 6,
    MouseDown = 7,
    MouseMove = 8,
    MouseUp = 9,
    MouseWheel = 10,
    PadButton = 11,
    PadAxis = 12,
    Text = 13,
    Unknown = 255,
}

/// One input event, exactly as the host writes it (32 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Event {
    pub kind: u8,
    /// Touch/pointer id, or mouse button.
    pub id: u8,
    /// Gamepad index.
    pub pad: u16,
    pub x: f32,
    pub y: f32,
    /// Kind-specific: wheel dx / axis value / pressure.
    pub a: f32,
    /// Kind-specific: wheel dy.
    pub b: f32,
    /// Key code, gamepad button/axis, or a Unicode scalar for `Text`.
    pub code: u32,
    /// Game time of the event itself (finer than a frame for touches,
    /// mouse and keys; recorded, so replays match). Judge timing with this.
    pub time: f64,
}

const _: () = assert!(size_of::<Event>() == 32);

impl Event {
    pub fn kind(&self) -> Kind {
        match self.kind {
            1 => Kind::TouchDown,
            2 => Kind::TouchMove,
            3 => Kind::TouchUp,
            4 => Kind::TouchCancel,
            5 => Kind::KeyDown,
            6 => Kind::KeyUp,
            7 => Kind::MouseDown,
            8 => Kind::MouseMove,
            9 => Kind::MouseUp,
            10 => Kind::MouseWheel,
            11 => Kind::PadButton,
            12 => Kind::PadAxis,
            13 => Kind::Text,
            _ => Kind::Unknown,
        }
    }

    /// True for a touch or mouse press.
    pub fn is_press(&self) -> bool {
        matches!(self.kind(), Kind::TouchDown | Kind::MouseDown)
    }
}

/// All events queued for this frame.
pub fn poll() -> Vec<Event> {
    let mut out = Vec::new();
    let mut buf = [Event::default(); 32];
    loop {
        let n = unsafe { ffi::mb_input_poll(buf.as_mut_ptr() as *mut u8, buf.len() as u32) };
        if n <= 0 {
            break;
        }
        out.extend_from_slice(&buf[..n as usize]);
        if (n as usize) < buf.len() {
            break;
        }
    }
    out
}
