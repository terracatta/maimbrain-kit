//! The host import table (SPEC §5): every function a guest may import from
//! the `mb` namespace, its exact Wasm signature, and what the manifest must
//! declare to use it. Imports not listed here are rejected, so modules are
//! added as the host implements them.

use crate::manifest::{Capability, Manifest, Sensor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    I32,
    I64,
    F32,
    F64,
}

use Ty::*;

#[derive(Clone, Copy, Debug)]
pub enum Requires {
    Always,
    Stdlib(&'static str),
    Sensor(Sensor),
    Capability(Capability),
}

#[derive(Clone, Copy, Debug)]
pub struct Import {
    pub name: &'static str,
    pub params: &'static [Ty],
    pub results: &'static [Ty],
    pub requires: Requires,
}

const fn f(name: &'static str, params: &'static [Ty], results: &'static [Ty], requires: Requires) -> Import {
    Import { name, params, results, requires }
}

const A: Requires = Requires::Always;
const MB2D: Requires = Requires::Stdlib("mb2d");

pub const IMPORTS: &[Import] = &[
    // 5.1 sys
    f("mb_log", &[I32, I32, I32], &[], A),
    f("mb_time", &[], &[F64], A),
    f("mb_rand_seed", &[], &[I64], A),
    f("mb_daily_seed", &[], &[I64], A),
    f("mb_round", &[I32], &[], A),
    f("mb_player_id", &[I32], &[I32], A),
    f("mb_locale", &[I32, I32], &[I32], A),
    f("mb_screen", &[I32], &[], A),
    f("mb_asset_load", &[I32, I32], &[I32], A),
    f("mb_asset_state", &[I32], &[I32], A),
    f("mb_asset_read", &[I32, I32, I32], &[I32], A),
    // 5.2 input
    f("mb_input_poll", &[I32, I32], &[I32], A),
    // 5.3 mb2d
    f("mb2d_image", &[I32], &[I32], MB2D),
    f("mb2d_font", &[I32], &[I32], MB2D),
    f("mb2d_clear", &[I32], &[], MB2D),
    f("mb2d_push", &[], &[], MB2D),
    f("mb2d_pop", &[], &[], MB2D),
    f("mb2d_translate", &[F32, F32], &[], MB2D),
    f("mb2d_rotate", &[F32], &[], MB2D),
    f("mb2d_scale", &[F32, F32], &[], MB2D),
    f("mb2d_blend", &[I32], &[], MB2D),
    f("mb2d_rect", &[F32, F32, F32, F32, I32], &[], MB2D),
    f("mb2d_rect_gradient", &[F32, F32, F32, F32, I32, I32], &[], MB2D),
    f("mb2d_circle", &[F32, F32, F32, I32], &[], MB2D),
    f("mb2d_line", &[F32, F32, F32, F32, F32, I32], &[], MB2D),
    f("mb2d_poly", &[I32, I32, I32], &[], MB2D),
    f("mb2d_sprite", &[I32, F32, F32, F32, F32, F32, F32, F32, F32, I32], &[], MB2D),
    f("mb2d_text", &[I32, F32, F32, F32, I32, I32, I32], &[], MB2D),
    f("mb2d_measure", &[I32, F32, I32, I32], &[F32], MB2D),
    // 5.6 audio (output only, so it never affects determinism)
    f("mb_sound", &[I32], &[I32], A),
    f("mb_play", &[I32, F32, F32, F32, I32], &[I32], A),
    f("mb_voice_set", &[I32, F32, F32, F32], &[], A),
    f("mb_voice_stop", &[I32], &[], A),
    // 5.7 sensors
    f("mb_tilt", &[I32], &[I32], Requires::Sensor(Sensor::Tilt)),
    f("mb_motion", &[I32], &[I32], Requires::Sensor(Sensor::Motion)),
    f("mb_loudness", &[], &[F32], Requires::Sensor(Sensor::Loudness)),
    f("mb_light", &[], &[F32], Requires::Sensor(Sensor::Light)),
    f("mb_haptic", &[I32], &[], Requires::Sensor(Sensor::Haptics)),
    // 5.8 store / score
    f("mb_store_get", &[I32, I32, I32, I32], &[I32], Requires::Capability(Capability::Store)),
    f("mb_store_set", &[I32, I32, I32, I32], &[I32], Requires::Capability(Capability::Store)),
    f("mb_score_submit", &[I32, I64], &[I32], Requires::Capability(Capability::Score)),
    f("mb_score_show", &[I32], &[], Requires::Capability(Capability::Score)),
    // 5.9 text input / links
    f("mb_text_begin", &[I32, I32, I32], &[], Requires::Capability(Capability::TextInput)),
    f("mb_text_end", &[], &[], Requires::Capability(Capability::TextInput)),
    f("mb_link_open", &[I32, I32], &[I32], Requires::Capability(Capability::Links)),
];

/// Required and optional guest exports (SPEC §3).
pub struct Export {
    pub name: &'static str,
    pub params: &'static [Ty],
    pub results: &'static [Ty],
    pub required: bool,
}

pub const EXPORTS: &[Export] = &[
    Export { name: "mb_init", params: &[], results: &[], required: true },
    Export { name: "mb_update", params: &[F32], results: &[], required: true },
    Export { name: "mb_render", params: &[], results: &[], required: true },
    Export { name: "mb_alloc", params: &[I32], results: &[I32], required: true },
    Export { name: "mb_suspend", params: &[], results: &[], required: false },
    Export { name: "mb_resume", params: &[], results: &[], required: false },
];

pub fn lookup(name: &str) -> Option<&'static Import> {
    IMPORTS.iter().find(|i| i.name == name)
}

impl Requires {
    /// Whether `m` declares what this import needs; Err names what's missing.
    pub fn satisfied_by(self, m: &Manifest) -> Result<(), String> {
        match self {
            Requires::Always => Ok(()),
            Requires::Stdlib(s) if m.has_stdlib(s) => Ok(()),
            Requires::Stdlib(s) => Err(format!("stdlib {{ {s} = 1 }}")),
            Requires::Sensor(s) if m.sensors.contains(&s) => Ok(()),
            Requires::Sensor(s) => Err(format!("sensors = [\"{}\"]", s.as_str())),
            Requires::Capability(c) if m.capabilities.contains(&c) => Ok(()),
            Requires::Capability(c) => Err(format!("capabilities = [\"{}\"]", c.as_str())),
        }
    }
}
