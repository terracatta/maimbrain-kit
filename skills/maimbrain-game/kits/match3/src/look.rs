//! Everything a reskin changes first: names, words, colors, the board's
//! place on screen, which atlas sprite each poppet uses. The art itself is
//! generated from `art/style.toml` (see `tools/regen.sh`); faces, specials and
//! effects are drawn in code in `draw.rs`.

use maimbrain::ui::{Theme, hex};

// ---- Words ------------------------------------------------------------------
pub const TITLE: &str = "POPPETS";
/// The title card's one line of desire (never an instruction).
pub const TAGLINE: &str = "don't let them doze off";
/// The game-over card's heading and score label.
pub const HEADING: &str = "NAP TIME!";
pub const SCORE_LABEL: &str = "POINTS";
pub const DAILY_LABEL: &str = "DAILY POINTS";
/// The in-play hint that teaches the swipe (≤ 5 words).
pub const HINT: &str = "SWIPE TO SWAP";

// ---- Board geometry -----------------------------------------------------------
/// Cell size in logical units (anything a thumb must hit wants ≥ 44).
pub const CELL: f32 = 46.0;
/// Top-left of the board.
pub const BOARD_X: f32 = (360.0 - CELL * crate::sim::COLS as f32) / 2.0;
pub const BOARD_Y: f32 = 176.0;

// ---- Art --------------------------------------------------------------------
/// The atlas sprite for each poppet kind (`src/cast.rs`, from `mb art atlas`):
/// coral puffball, apricot marshmallow square, butter rice-ball triangle, sage
/// sprout hexagon, powder-blue cloud, lavender star, bubblegum heart. Shape AND
/// color differ, so it reads colorblind.
pub const SPRITES: [[f32; 4]; 7] = [
    crate::cast::P_CORAL,
    crate::cast::P_ORANGE,
    crate::cast::P_YELLOW,
    crate::cast::P_GREEN,
    crate::cast::P_BLUE,
    crate::cast::P_PURPLE,
    crate::cast::P_PINK,
];
/// A sprite is drawn this many piece radii wide (the painted body fills ~84 %
/// of its 128 px cell, so ~2.3 makes the body about one radius from centre to edge).
pub const SPRITE_SPAN: f32 = 2.3;
/// Where each kind's face sits (piece radii below centre): the triangle's is
/// low in its wide bottom, the heart's high between its lobes. Index 7 is the
/// rock, 8 the rainbow.
pub const FACE_Y: [f32; 9] = [0.02, 0.0, 0.2, 0.1, 0.02, 0.1, -0.06, 0.08, 0.04];
/// Faces are drawn this much bigger than their unit layout (bigger reads sleepier).
pub const FACE_SCALE: f32 = 1.15;

// ---- Colors -----------------------------------------------------------------
/// One color per poppet kind, matched to its painted body: particles, the
/// fallback shapes drawn before the atlas loads, banners.
pub const PIECES: [u32; 7] = [
    hex(0xe8695f), // coral puffball
    hex(0xf0a050), // apricot square
    hex(0xf2c95e), // butter triangle
    hex(0x7fb88a), // sage hexagon
    hex(0x86b2e8), // powder-blue cloud
    hex(0xb294e0), // lavender star
    hex(0xee86c4), // bubblegum heart
];
/// Pencil ink for faces, outlines and text outlines (a warm dark plum).
pub const INK: u32 = hex(0x34203f);
pub const ROCK: u32 = hex(0x9a9088);
/// Rosy cheeks.
pub const BLUSH: u32 = hex(0xff8a9a);
/// Before the painted window loads (and the first frame): the night sky as a
/// plain gradient (top, bottom), awake and asleep.
pub const SKY_AWAKE: (u32, u32) = (hex(0x2c2c66), hex(0x6a5f9e));
pub const SKY_ASLEEP: (u32, u32) = (hex(0x0d0f2a), hex(0x261d50));
/// Drowsiness tints the painted room (multiply): awake → no tint, asleep → this.
pub const NIGHT_TINT: u32 = hex(0x6a72b8);
/// The bedside lamp's warm glow while everyone's awake (additive).
pub const LAMP: u32 = hex(0xffc884);
/// The board: an indigo wash on paper, paler cells, a pencil border.
pub const BOARD_WASH: u32 = 0x231d4ad8;
pub const CELL_WASH: u32 = 0xfff3dc1c;
pub const PENCIL: u32 = hex(0xf6eedd);
/// The wake meter: full, half, nearly empty (sage, butter, coral).
pub const METER: [u32; 3] = [hex(0x8fd09a), hex(0xf2d37a), hex(0xe8695f)];
pub const GOLD: u32 = hex(0xf6d77e);

/// The UI kit's theme (title card, HUD, results card, buttons): plum-indigo
/// panels like a wash on dark paper, a coral accent, butter bests.
pub fn theme() -> Theme {
    let mut t = Theme::candy();
    t.outline = INK;
    t.accent = hex(0xe2706f);
    t.gold = GOLD;
    t.panel_top = hex(0x3d3566);
    t.panel_bottom = hex(0x241f48);
    t.panel_border = 0xf6eedd40;
    t.text = hex(0xfff8ea);
    t.text_dim = 0xf6eeddb8;
    t.secondary = 0xf6eedd26;
    t.good = METER[0];
    t.shadow = 0x0d0a2060;
    t.radius = 26.0;
    t.confetti = [PIECES[0], PIECES[2], PIECES[3], PIECES[4], PIECES[6]];
    t
}
