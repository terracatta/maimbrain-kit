//! The look: every name, word and color a reskin changes, in one place.
//! Shapes and physics are in `sim.rs`; how things are drawn is `draw.rs`.

use maimbrain::ui::{Theme, hex};

use crate::sim::Kind;
use crate::sprites as sp;

// ---- Words ---------------------------------------------------------------

pub const TITLE: &str = "PICNIC PILE";
/// The card's one line of desire (not an instruction: the card can't be touched).
pub const TAGLINE: &str = "how high can lunch go?";
pub const OVER_HEADING: &str = "SPLAT!";
pub const SCORE_LABEL: &str = "SNACKS STACKED";
pub const DAILY_LABEL: &str = "TODAY'S SNACKS";
/// The in-play hint (≤ 5 words), shown until the player has dropped a few.
pub const HINT: &str = "TAP TO DROP";
pub const NEAT: &str = "NEAT!";
/// A near miss: it only just stayed on.
pub const CLOSE: &str = "PHEW!";
/// Popups when a heart goes: a snack that missed, a tower that tumbled.
pub const MISS: &str = "OOPS!";
pub const TUMBLE: &str = "TUMBLE!";
pub const GUST: &str = "WIND!";
pub const TILT: &str = "WOBBLE!";
pub const LAST_HEART: &str = "LAST HEART!";
/// A streak won a heart back.
pub const HEART_BACK: &str = "+1 HEART";

/// Each snack's name (shown when it first arrives) and its three looks: a
/// picture in `assets/sprites.png` (`src/sprites.rs`, made by tools/regen.sh)
/// and a tint multiplied over it (white keeps the picture's own colors).
/// Pictures are drawn stretched over the snack's collider, so a new picture
/// should fill its `Kind::size()` box the same way (toast: a 3:2 rectangle,
/// cheese: a trapezoid, tomato: a circle).
pub struct Snack {
    pub name: &'static str,
    pub sprites: [([f32; 4], u32); 3],
}

pub fn snack(kind: Kind) -> Snack {
    match kind {
        Kind::Box => Snack { name: "TOAST", sprites: [(sp::TOAST, WHITE), (sp::TOAST, 0xf3e3d2ff), (sp::TOAST, 0xfff3e2ff)] },
        Kind::Cube => Snack { name: "JELLY", sprites: [(sp::JELLY_0, WHITE), (sp::JELLY_1, WHITE), (sp::JELLY_2, WHITE)] },
        Kind::Plank => Snack { name: "BAGUETTE", sprites: [(sp::BAGUETTE, WHITE), (sp::BAGUETTE, 0xf6e6d6ff), (sp::BAGUETTE, 0xfff1dcff)] },
        Kind::Wedge => Snack { name: "CHEESE", sprites: [(sp::CHEESE, WHITE), (sp::CHEESE, 0xfff2d4ff), (sp::CHEESE, 0xffe7bcff)] },
        Kind::Bun => Snack { name: "MACARON", sprites: [(sp::MACARON_0, WHITE), (sp::MACARON_1, WHITE), (sp::MACARON_2, WHITE)] },
        Kind::Ball => Snack { name: "TOMATO", sprites: [(sp::TOMATO_0, WHITE), (sp::TOMATO_1, WHITE), (sp::TOMATO_2, WHITE)] },
    }
}

// ---- Colors ----------------------------------------------------------------

/// Plum ink: the darkest paper (text edges, the ghost's shade).
pub const INK: u32 = 0x3a2433ff;
pub const WHITE: u32 = 0xffffffff;
/// Paper drop shadows (alpha is set where they're drawn).
pub const SHADOW: u32 = 0x3a2433ff;
/// Paper strips (wind streaks, tilt arrows).
pub const PAPER_WHITE: u32 = 0xfff8ecff;
pub const STAR: u32 = 0xfff4c8ff;
/// Sky (top, bottom) by the tower's height in px: noon, golden hour, sunset,
/// night. Flat, papery colors: the paper grain is multiplied over them.
pub const SKY: [(f32, u32, u32); 4] = [(0.0, 0x86cdeeff, 0xd9f1f8ff), (500.0, 0x79b6e8ff, 0xffe0a6ff), (1000.0, 0x8a6fd0ff, 0xffa47cff), (1600.0, 0x1c2350ff, 0x4b3b80ff)];
/// The ground below the hills and below the meadow (matched to the art's edges).
pub const FIELD: u32 = 0x527237ff;
pub const FIELD_NEAR: u32 = 0x4f7a3aff;
/// The landing ghost and its dotted path.
pub const GHOST: u32 = 0xffffffff;
pub const DANGER: u32 = 0xe8564fff;

/// The UI kit's theme (title card, HUD, results card, buttons).
pub fn theme() -> Theme {
    // Cut-paper colors from art/style.toml's palette: cream paper, plum ink,
    // tomato red, cheese yellow, meadow green.
    let mut t = Theme::candy();
    t.text = 0xfff8ecff;
    t.text_dim = 0xfff4e0d8;
    t.outline = hex(0x3a2433);
    t.shadow = 0x3a243366;
    t.accent = hex(0xe8564f);
    t.on_accent = 0xfff8ecff;
    t.secondary = 0xfff4e040;
    t.gold = hex(0xffd84d);
    t.bad = hex(0xe8564f);
    t.panel_top = hex(0x5a3a4c);
    t.panel_bottom = hex(0x3a2433);
    t.confetti = [hex(0xe8564f), hex(0xffd84d), hex(0x8cc56a), hex(0x6fc3e8), hex(0xfff4e0)];
    t
}
