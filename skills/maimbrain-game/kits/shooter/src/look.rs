//! The look: every color, the UI theme and every string a reskin changes.
//! Shapes live in `draw.rs`; this file is the palette and the words.

use maimbrain::ui::{Theme, hex};

use crate::sim::{BannerKind, Kind, Power};

// ---- Words ----------------------------------------------------------------
pub const TITLE: &str = "NOVA PIP";
pub const TAGLINE: &str = "one tiny pod. one big swarm.";
/// The game-over card's heading.
pub const HEADING: &str = "SHOT DOWN!";
pub const SCORE_LABEL: &str = "SCORE";
pub const DAILY_LABEL: &str = "DAILY SCORE";
/// The in-play hint, with the animated thumb under the ship.
pub const HINT: &str = "DRAG TO FLY";
pub const BOSS_NAME: &str = "GLOOM QUEEN";

/// A banner's big line and small line.
pub fn banner(kind: BannerKind) -> (&'static str, &'static str) {
    match kind {
        BannerKind::Warning => ("WARNING", "THE GLOOM QUEEN"),
        BannerKind::New(k) => (
            match k {
                Kind::Jelly => "JELLIES!",
                Kind::Swooper => "SWOOPERS!",
                Kind::Darter => "DARTERS!",
                Kind::Bulb => "BULBS!",
                Kind::Spinner => "SPINNERS!",
            },
            "NEW ENEMY",
        ),
        BannerKind::Clear => ("QUEEN DOWN!", "AGAIN, FASTER"),
    }
}

pub fn power_name(p: Power) -> &'static str {
    match p {
        Power::Spread => "SPREAD!",
        Power::Shield => "SHIELD!",
        Power::Magnet => "MAGNET!",
    }
}

// ---- Palette ----------------------------------------------------------------
// Neon synthwave: deep indigo night, magenta and cyan neon, a gold sun.
// The art itself is generated (`assets/sprites.png`, `assets/backdrop.png`,
// see art/style.toml); these colors are for everything drawn in code
// (the grid, bullets, glows, particles, the UI) so it matches the art.

/// The backdrop is drawn tinted by this (a little dimmer than the art, so
/// bullets pop), warming toward `BG_HOT_TINT` as the heat rises.
pub const BG_TINT: u32 = 0xc4bcdcff;
pub const BG_HOT_TINT: u32 = 0xe4b4d2ff;
/// Behind the backdrop before it loads.
pub const BG_TOP: u32 = hex(0x140c3c);
pub const BG_BOTTOM: u32 = hex(0x0c0826);
/// The scrolling floor grid below the horizon, and the horizon line.
pub const GRID: u32 = hex(0xff2fb4);
pub const GRID_HOT: u32 = hex(0xff7a3a);
pub const HORIZON_GLOW: u32 = hex(0xff5ad0);
pub const STAR: u32 = 0xdff4ffff;
/// Speed streaks falling past (the near star layer).
pub const STREAK: u32 = hex(0x7ff3ff);

/// Pip's engine and face (the pod itself is the `PIP` sprite).
pub const SHIP_TRIM: u32 = hex(0x22e6ff);
pub const SHIP_TRIM_DARK: u32 = hex(0x1a8fb8);
pub const INK: u32 = hex(0x0a0820);
pub const EYE: u32 = hex(0xf4f0ff);
pub const FLAME_IN: u32 = hex(0xe8ffff);
pub const FLAME_OUT: u32 = hex(0x22c8ff);
pub const FLAME_EDGE: u32 = hex(0xff2fb4);
pub const SHIELD: u32 = 0x5f9bffff;
/// Pip's wreck after the round: the sprite multiplied by this.
pub const WRECK_TINT: u32 = 0x6a5e86ff;

/// The ship's shots: cool cyan neon, nothing like the enemy bullets.
pub const SHOT: u32 = hex(0x8ffcff);

/// Enemy neon colors, by `Kind::index` (glows, particles, banners, brows):
/// jelly magenta, swooper orange, darter violet, bulb gold, spinner coral.
pub const ENEMY: [u32; 5] = [hex(0xff4fd0), hex(0xff9a3c), hex(0xb46bff), hex(0xffc23a), hex(0xff5a6e)];

/// Enemy bullets: hot pink rims around white-hot cores, inside a dark
/// outline so they read on the sun and the grid alike. Big ones orange.
pub const BULLET_RIM: u32 = hex(0xff2f86);
pub const BULLET_BIG_RIM: u32 = hex(0xff7a1a);
pub const BULLET_CORE: u32 = hex(0xffffff);
pub const BULLET_OUTLINE: u32 = hex(0x0a0418);

pub const GEM: u32 = hex(0xffc23a);
pub const HEART: u32 = hex(0xff4f6b);
pub fn power_color(p: Power) -> u32 {
    match p {
        Power::Spread => hex(0x22e6ff),
        Power::Shield => hex(0x5f9bff),
        Power::Magnet => hex(0xff2fb4),
    }
}

pub const BOSS: u32 = hex(0xc04dff);
/// The queen's tentacles and arms (neon tubes drawn in code).
pub const BOSS_NEON: u32 = hex(0xff3fc8);
pub const BOSS_NEON_2: u32 = hex(0x22e6ff);
pub const DANGER: u32 = hex(0xff3355);

/// The UI kit's theme (title card, HUD, results card, buttons): indigo
/// glass panels with a cyan neon edge, a hot magenta accent.
pub fn theme() -> Theme {
    let mut t = Theme::night();
    t.accent = hex(0xff2fb4);
    t.on_accent = hex(0xffffff);
    t.secondary = 0x22e6ff30;
    t.on_secondary = hex(0xe8fdff);
    t.gold = hex(0xffc23a);
    t.good = SHIP_TRIM;
    t.bad = HEART;
    t.outline = INK;
    t.shadow = 0x05021480;
    t.panel_top = 0x24186af0;
    t.panel_bottom = 0x0e0a30f0;
    t.panel_border = 0x22e6ff70;
    t.radius = 14.0;
    t.text = hex(0xf4f0ff);
    t.text_dim = 0xc8c0ffc0;
    t.confetti = [SHIP_TRIM, GEM, hex(0xff2fb4), hex(0xb46bff), hex(0xffffff)];
    t
}
