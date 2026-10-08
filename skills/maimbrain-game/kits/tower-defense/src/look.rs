//! The look: everything a reskin changes first. Words, the UI theme and the
//! palette live here; shapes are drawn in src/draw.rs from these colors.

use maimbrain::gfx2d::Font;
use maimbrain::ui::Theme;

// ---- Words ------------------------------------------------------------
pub const TITLE: &str = "CAKE KEEP";
pub const TAGLINE: &str = "they want the cake";
/// Results card heading when the cake is devoured, and what the score counts.
pub const OVER_HEADING: &str = "DEVOURED!";
pub const SCORE_LABEL: &str = "JELLIES POPPED";
/// In-play hints (≤ 5 words each).
pub const HINT_BUILD: &str = "BUILD HERE";
pub const HINT_BUY: &str = "PICK A TOWER";
pub const HINT_UPGRADE: &str = "TAP TO UPGRADE";
/// Banners for a king wave and a rush wave.
pub const BOSS_BANNER: &str = "THE KING RETURNS!";
pub const RUSH_BANNER: &str = "ZIPPER RUSH!";
/// Banner when a new jelly kind first appears (indexed by `CreepKind`).
pub const NEW_KIND_BANNER: [&str; 6] = ["", "ZIPPERS!", "HELMETS!", "SPLITTERS!", "", "KING JELLY!"];

// ---- Palette ----------------------------------------------------------
// Picked from the generated clay art (art/style.toml has the same colours),
// so code-drawn things (shots, rings, the HUD) sit in the same world.
pub const INK: u32 = 0x3a1f35ff;
pub const WHITE: u32 = 0xffffffff;
/// The map's lawn, for screens taller than the map.
pub const GROUND: u32 = 0x9fd49cff;
/// Soft shadows under jellies, the cake and bombs, on the lawn and the road.
pub const SHADOW: u32 = 0x24502c48;
pub const PAD_SHADOW: u32 = 0x2a5a3050;
/// The "+" on an empty pad.
pub const PAD_PLUS: u32 = 0xa8743eff;
/// The dimples that drift down the road.
pub const ROAD_DOT: u32 = 0xe8cf9cff;
pub const ROAD_DOT_SHADE: u32 = 0xc99a6280;
pub const GOLD: u32 = 0xffd23fff;
pub const DANGER: u32 = 0xff4f6dff;
pub const FROST: u32 = 0x8fe3ffff;
/// Added over a slowed jelly (a cold blue sheen that keeps its own colour).
pub const FROST_GLOW: u32 = 0x3c8cd000;
/// POP's gumballs (the colours in its dome).
pub const GUMBALLS: [u32; 4] = [0xff7fa8ff, 0xffd23fff, 0x7fcfffff, 0xfff4e6ff];
/// Jelly colors for particles, indexed by `CreepKind` (gummy, zipper, helmet, splitter, mini, king).
pub const JELLY: [u32; 6] = [0xff5d73ff, 0xffc93cff, 0x9b4f9bff, 0xff9f43ff, 0xff9f43ff, 0xe8439bff];
/// Tower colors for particles, indexed by `TowerKind` (pop, chill, boom).
pub const TOWER: [u32; 3] = [0xff7fa0ff, 0x9fe0ffff, 0xe0484bff];
/// Crumbs and bite bursts.
pub const CAKE_SPONGE: u32 = 0xf0c27aff;
pub const CAKE_FROSTING: u32 = 0xff7f9cff;

/// The UI kit's theme (title card, HUD, results card, buttons): warm
/// plum-chocolate clay panels, a strawberry accent, extra-round corners.
pub fn theme() -> Theme {
    let mut t = Theme::candy();
    t.bg_top = 0x7a3c5cff;
    t.bg_bottom = 0x45203aff;
    t.panel_top = 0x8a4862ff;
    t.panel_bottom = 0x5a2842ff;
    t.panel_border = 0xffe6d040;
    t.text_dim = 0xffe8dcc0;
    t.accent = 0xff5d7cff;
    t.good = 0x7be38aff;
    t.gold = GOLD;
    t.bad = DANGER;
    t.outline = INK;
    t.shadow = 0x2a102860;
    t.radius = 26.0;
    t.title_font = Font::SansBold;
    t.number_font = Font::SansBold;
    t.confetti = [0xff5d73ff, GOLD, 0x8fe3ffff, 0xff9f43ff, WHITE];
    t
}
