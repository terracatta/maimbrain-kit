//! Everything a reskin changes first: names, words, the UI theme and the
//! palette. Colors for mb2d are sRGB `0xRRGGBBAA`; colors for the 3D scene
//! are sRGB hex too, converted with `srgb()` where they're used.

use maimbrain::ui::Theme;

// ---- Words -------------------------------------------------------------
pub const TITLE: &str = "SWERVE";
pub const TAGLINE: &str = "the sunset road never ends";
/// The game-over heading and the score's label.
pub const HEADING: &str = "WRECKED!";
pub const SCORE_LABEL: &str = "SCORE";
pub const DAILY_LABEL: &str = "DAILY SCORE";
/// The in-play hint (≤ 5 words) and the near-miss popup.
pub const HINT: &str = "TAP LEFT · RIGHT";
pub const NEAR_MISS: &str = "CLOSE!";
pub const SPEED_UP: &str = "SPEED UP!";
pub const BOOST: &str = "BOOST!";
/// Announcements when something new joins the road.
pub fn new_hazard(kind: crate::sim::Kind, first_truck: bool) -> &'static str {
    use crate::sim::Kind;
    match kind {
        Kind::Truck if first_truck => "TRUCKS!",
        Kind::Truck => "WATCH THE BLINKERS",
        Kind::Cones => "ROADWORKS!",
        Kind::Oil => "OIL SLICKS!",
        Kind::Car => "TRAFFIC!",
    }
}

// ---- UI theme ----------------------------------------------------------
/// The UI kit's theme: night's shapes in sunset colors.
pub fn theme() -> Theme {
    let mut t = Theme::night();
    t.panel_top = 0x3a1f3ce8;
    t.panel_bottom = 0x22122ae8;
    t.panel_border = 0xffb38a50;
    t.text = 0xfff4e8ff;
    t.text_dim = 0xffe2cdc0;
    t.accent = PAINT;
    t.on_accent = 0x0b2422ff;
    t.secondary = 0xffd9c030;
    t.gold = 0xffd24aff;
    t.bad = 0xff4d5eff;
    t.outline = 0x1d0b1eff;
    t.shadow = 0x1d0b1e90;
    t.confetti = [PAINT, 0xffd24aff, 0xff6f91ff, 0xff9a4aff, 0xfff4e8ff];
    t
}

// ---- Palette (3D) ------------------------------------------------------
/// The player's car: paint, trim and the glass.
pub const PAINT: u32 = 0x22d3c5ff;
pub const PAINT_DARK: u32 = 0x128a86ff;
pub const TRIM: u32 = 0x2a2233ff;
pub const GLASS: u32 = 0x2b4470ff;
pub const STRIPE: u32 = 0xfff4e8ff;
/// Traffic body tints (instance colors multiply a white body), muted so the hero pops.
pub const TRAFFIC: [u32; 8] = [0xf2e6d0ff, 0x9fb7d9ff, 0xc9b6e4ff, 0xa8c9a0ff, 0xe8a9a0ff, 0xd8d4ccff, 0x8f9bb0ff, 0xf0c98aff];
/// Truck trailers.
pub const TRUCKS: [u32; 8] = [0xf4efe6ff, 0xe8d2b0ff, 0xc0d4e8ff, 0xf4efe6ff, 0xd8c8e8ff, 0xeee4d4ff, 0xb8d8c8ff, 0xf4efe6ff];
pub const ASPHALT: u32 = 0x3b3245ff;
/// Bands across the road every few meters (a touch lighter): optic flow.
pub const ASPHALT_BAND: u32 = 0x473d54ff;
/// Rumble strips (kerbs) along both edges, alternating with `LINES`.
pub const KERB: u32 = 0xd8463cff;
/// Overhead light gantries and their glowing strips (linear, above 1 blooms).
pub const GANTRY: u32 = 0x4a4252ff;
pub const GANTRY_GLOW: [f32; 3] = [3.4, 2.1, 1.2];
pub const SHOULDER: u32 = 0x5a4a52ff;
pub const LINES: u32 = 0xfff1d8ff;
pub const SAND: u32 = 0xd99a6cff;
pub const SAND_DARK: u32 = 0xc07f5aff;
pub const RAIL: u32 = 0xc8c0c8ff;
pub const MESA: u32 = 0xa65a4cff;
pub const PALM_TRUNK: u32 = 0x7a5238ff;
pub const PALM_LEAF: u32 = 0x2f6b4aff;
pub const CONE: u32 = 0xff7a2aff;
pub const COIN: u32 = 0xffc23aff;
/// Glows (linear, above 1 blooms).
pub const BOOST_GLOW: [f32; 3] = [0.4, 2.6, 3.2];
pub const BLINKER: [f32; 3] = [4.0, 1.6, 0.2];

/// The sky (linear): background, two nebula colors. It's also the ambient light.
pub const SKY: [[f32; 3]; 3] = [[0.2, 0.09, 0.19], [0.9, 0.34, 0.14], [0.24, 0.11, 0.4]];
/// Fog matches the horizon so the road melts into it.
pub const FOG: [f32; 3] = [0.66, 0.3, 0.27];
/// Outline ink: the darkest plum.
pub const INK: [f32; 3] = [0.03, 0.012, 0.03];
