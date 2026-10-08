//! Everything a reskin changes first: the name and strings, the palette, the
//! UI kit `Theme`, and each category's emblem and color. The generated art
//! (`assets/*.png`, made from `art/style.toml` by `tools/regen.sh`) uses the
//! same palette, so code-drawn things (cards, buttons, the clock, text)
//! belong with it.

use maimbrain::gfx2d::Font;
use maimbrain::ui::{Theme, hex, text};

// ---- Strings ----------------------------------------------------------
pub const TITLE: &str = "BRIGHT SPARK";
/// The title on the marquee: two lines.
pub const TITLE_LINES: [&str; 2] = ["BRIGHT", "SPARK"];
pub const TAGLINE: &str = "how bright are you?";
/// The title's tagline in daily mode.
pub const DAILY_TAGLINE: &str = "today's quiz for all";
/// The results card's heading when the last heart goes.
pub const HEADING: &str = "LIGHTS OUT!";
pub const SCORE_LABEL: &str = "POINTS";
pub const DAILY_LABEL: &str = "DAILY POINTS";
/// The in-play hint, until the first answer (≤ 5 words).
pub const HINT: &str = "TAP THE ANSWER";
pub const WARMUP: &str = "WARM-UP";
pub const GOLDEN: &str = "GOLDEN ×2";
pub const TIME_UP: &str = "TIME!";

// ---- Palette: 1950s game-show print -------------------------------------
/// Ink: outlines, card text, hard print shadows.
pub const INK: u32 = hex(0x2a1e1a);
/// Paper: cards and buttons.
pub const PAPER: u32 = hex(0xfbf1dc);
pub const PAPER_SHADE: u32 = hex(0xefdcb8);
/// Poster colors.
pub const TEAL: u32 = hex(0x1f7a7a);
pub const TOMATO: u32 = hex(0xe2412f);
pub const MUSTARD: u32 = hex(0xf2a922);
pub const CREAM: u32 = hex(0xffe9a8);
/// Text on the stage (cream with an ink outline reads on teal and red).
pub const ON_STAGE: u32 = hex(0xfff4dc);
/// The fallback backdrop before the stage image loads.
pub const BG_TOP: u32 = hex(0x1f6e6e);
pub const BG_BOTTOM: u32 = hex(0x14504f);
/// The golden question's card.
pub const GOLD_TOP: u32 = hex(0xffd769);
pub const GOLD_BOTTOM: u32 = hex(0xf2a922);
/// Answer letter badges.
pub const BADGE: u32 = TOMATO;
/// Right and wrong: a print green and the tomato red.
pub const RIGHT: u32 = hex(0x3d9a4b);
pub const WRONG: u32 = hex(0xd8352a);
/// The host's glow (and the fallback drawing's glass, metal and tie).
pub const GLOW: u32 = hex(0xffd75a);
pub const GLASS_LIT: u32 = hex(0xfff0c0);
pub const GLASS_OFF: u32 = hex(0x968a80);
pub const METAL: u32 = hex(0xb8b0a6);
pub const BOWTIE: u32 = TOMATO;
/// How far a hard print shadow sits from what casts it.
pub const DROP: (f32, f32) = (4.0, 5.0);

/// The UI kit's theme (title, HUD, results card, buttons, juice).
pub fn theme() -> Theme {
    let mut t = Theme::paper();
    t.bg_top = BG_TOP;
    t.bg_bottom = BG_BOTTOM;
    t.panel_top = PAPER;
    t.panel_bottom = PAPER_SHADE;
    t.panel_border = INK;
    t.text = ON_STAGE;
    t.text_dim = 0xfff4dcd0;
    t.accent = TOMATO;
    t.on_accent = ON_STAGE;
    t.secondary = PAPER;
    t.on_secondary = INK;
    t.gold = MUSTARD;
    t.good = RIGHT;
    t.bad = WRONG;
    t.outline = INK;
    t.shadow = 0x2a1e1a90;
    t.radius = 14.0;
    t.outline_em = 0.1;
    t.title_font = Font::SansBold;
    t.confetti = [MUSTARD, TOMATO, TEAL, CREAM, RIGHT];
    t
}

// ---- Categories -------------------------------------------------------
/// The emblem (a cell of `assets/cats.png`, in this order) and color for a
/// category name from questions.txt. Add a line here (and an emblem to the
/// `cats` prompt in tools/regen.sh) when you add a category; unknown ones
/// get a code-drawn question mark.
pub fn category(name: &str) -> (Option<usize>, u32) {
    match name {
        "SCIENCE" => (Some(0), hex(0x1f7a7a)),
        "SPACE" => (Some(1), hex(0x34507e)),
        "WORLD" => (Some(2), hex(0x2c8a68)),
        "NATURE" => (Some(3), hex(0x5c8a32)),
        "FOOD" => (Some(4), hex(0xd2691e)),
        "NUMBERS" => (Some(5), hex(0xc8382c)),
        "WORDS" => (Some(6), hex(0x8a4a78)),
        "INVENTIONS" => (Some(7), hex(0x7a6656)),
        "BODY" => (Some(8), hex(0xb83040)),
        "TIME" => (Some(9), hex(0xb87a12)),
        _ => (None, TOMATO),
    }
}

/// The emblem for a category with no cell: a question mark on paper.
pub fn question_emblem(cx: f32, cy: f32, size: f32) {
    use maimbrain::ui::shape;
    shape::disc(cx, cy, size * 0.5, INK);
    shape::disc(cx, cy, size * 0.5 - 3.0, PAPER);
    text("?").size(size * 0.62).color(TOMATO).middle().draw(cx, cy);
}
