//! The look. Everything a reskin changes lives in this file: the words, the
//! sky that drifts day → dusk → night → dawn with distance, the UI theme,
//! and how the generated pixel art is drawn: the hero, the obstacles, coins,
//! the parallax scenery and the ground.
//!
//! The art itself is generated (`art/style.toml`, `tools/regen.sh`): sprites
//! packed into `assets/sprites.png` (rects in `sprites.rs`) plus three
//! parallax layers. Every art pixel is 1.5 units = 3 canvas pixels, so it
//! stays crisp. Code still draws the sky bands, stars, the grass fringe,
//! sweat and dizzy stars, and the particles.
//!
//! The rules (`sim.rs`) know none of it: to the sim the hero is a 24 × 40 box
//! and a gull is a box that flies. Swap the bun for a robot, the crates for
//! cacti and the gulls for drones here, and the game plays the same.

use std::f32::consts::TAU;

use maimbrain::gfx2d::{self, Font, Image};
use maimbrain::sys::{Asset, AssetState};
use maimbrain::ui::{Rect, Theme, hex, mix, shape, with_alpha};

use crate::sim::Kind;
use crate::sprites as S;

// ---- Words ---------------------------------------------------------------
pub const TITLE: &str = "BUN RUN";
pub const TAGLINE: &str = "how far can a bun go?";
/// Game-over headings: hit something / fell in a pit.
pub const HEADING_BONK: &str = "BONK!";
pub const HEADING_PIT: &str = "WHOOPS!";
pub const SCORE_LABEL: &str = "SCORE";
pub const DAILY_LABEL: &str = "DAILY SCORE";
pub const HINT_TAP: &str = "TAP TO JUMP";
pub const HINT_AGAIN: &str = "TAP AGAIN!";
pub const FASTER: &str = "FASTER!";
pub const CLOSE: &str = "CLOSE!";
/// Every coin of an arc collected.
pub const NICE: &str = "NICE!";

/// The banner when a family first shows up.
pub fn kind_name(k: Kind) -> &'static str {
    match k {
        Kind::Crate => "CRATES!",
        Kind::Pit => "PITS!",
        Kind::Stack => "TALL STACKS!",
        Kind::Gull => "HUNGRY GULLS!",
        Kind::Bee => "BUSY BEES!",
        Kind::Skimmer => "LOW GULLS!",
        Kind::Log => "LOGS!",
    }
}

/// The hint while the first of a family comes at you (≤ 5 words).
pub fn kind_hint(k: Kind) -> Option<&'static str> {
    match k {
        Kind::Crate => None,
        Kind::Pit => Some("JUMP THE GAP"),
        Kind::Stack => Some("HOLD TO JUMP HIGHER"),
        Kind::Gull => Some("STAY LOW!"),
        Kind::Bee => Some("TIME IT!"),
        Kind::Skimmer => Some("JUMP IT!"),
        Kind::Log => Some("HOLD FOR A LONG JUMP"),
    }
}

// ---- Layout --------------------------------------------------------------
/// The ground line on screen: lower-middle, above the thumb.
pub const GROUND_Y: f32 = 430.0;
pub const W: f32 = 360.0;
pub const H: f32 = 640.0;

// ---- Palette (art/style.toml's, for what code draws) ----------------------
const INK: u32 = hex(0x1a1230);
const GOLD: u32 = hex(0xffe23a);
const RED: u32 = hex(0xec3b35);
const LEAF: u32 = hex(0x2f9a3e);
const LEAF_DARK: u32 = hex(0x1c5a3a);
const LEAF_LIGHT: u32 = hex(0xb6f075);
const SKY_PALE: u32 = hex(0xc4ecff);
const SKY_DEEP: u32 = hex(0x6fc3ff);

/// The UI kit's theme (title card, HUD, results): the art's ink, red and
/// gold, deep-blue panels, small radii and the pixel font for big words.
pub fn theme() -> Theme {
    let mut t = Theme::candy();
    t.accent = RED;
    t.gold = GOLD;
    t.outline = INK;
    t.shadow = 0x1a123070;
    t.text_dim = 0xfff1c2d8;
    t.secondary = 0xffffff30;
    t.panel_top = hex(0x3d7be0);
    t.panel_bottom = hex(0x24418f);
    t.panel_border = 0x1a1230ff;
    t.radius = 6.0;
    t.outline_em = 0.125;
    t.title_font = Font::Pixel;
    t.number_font = Font::Pixel;
    t.confetti = [RED, GOLD, hex(0x6bd04a), hex(0x6fc3ff), 0xffffffff];
    t
}

// ---- The art ---------------------------------------------------------------

/// Every image: the sprite atlas (`mb art atlas`, rects in `sprites.rs`) and
/// the three parallax layers (`mb art layers`), far to near.
const ART_FILES: [&str; 4] = ["assets/sprites.png", "assets/hills_1.png", "assets/hills_2.png", "assets/hills_3.png"];

/// The loaded images. Until one is ready its things just aren't drawn (the
/// atlas and layers are startup assets, so that's the first frame or two).
pub struct Art {
    files: [Asset; 4],
    imgs: [Option<Image>; 4],
}

impl Art {
    pub fn load() -> Art {
        Art { files: ART_FILES.map(Asset::load), imgs: [None; 4] }
    }

    /// Call every update: picks up images as they finish loading.
    pub fn poll(&mut self) {
        for i in 0..ART_FILES.len() {
            if self.imgs[i].is_none() && self.files[i].state() == AssetState::Ready {
                self.imgs[i] = Image::new(self.files[i]);
            }
        }
    }

    fn atlas(&self) -> Option<Image> {
        self.imgs[0]
    }
}

/// Logical units per art pixel. The atlas stores 3 texels per art pixel
/// (`pixel_scale = 3`), so a texel is half a unit and one canvas pixel:
/// sprites draw at exactly their size, never resampled.
const PX: f32 = 1.5;

/// Rounds to the canvas pixel grid (half units), so pixels stay crisp.
fn snap(v: f32) -> f32 {
    (v * 2.0).round() * 0.5
}

/// Frame `i` of an `n`-frame strip.
fn cell(r: [f32; 4], i: usize, n: usize) -> [f32; 4] {
    let w = r[2] / n as f32;
    [r[0] + w * (i % n) as f32, r[1], w, r[3]]
}

/// A texel rect at its natural size, top-left at (x, y).
fn blit(img: Image, src: [f32; 4], x: f32, y: f32, tint: u32) {
    gfx2d::sprite(img, src, [snap(x), snap(y), src[2] * 0.5, src[3] * 0.5], tint);
}

/// One axis of a sliced sprite, in art pixels: the first part, the middle
/// (repeated, never stretched) and the last part.
type Slices = [(f32, f32); 3];

/// Where each piece of one axis goes: (texel offset in the sprite, start, length in units).
fn spans(s: Slices, at: f32, len: f32) -> Vec<(f32, f32, f32)> {
    let (a, c) = ((s[0].1 - s[0].0) * PX, (s[2].1 - s[2].0) * PX);
    let k = if a + c > len { len / (a + c) } else { 1.0 };
    let (a, c) = (snap(a * k), snap(c * k));
    let mut out = vec![(s[0].0 * 3.0, at, a)];
    let tile = (s[1].1 - s[1].0) * PX;
    let (mut x, end) = (at + a, at + len - c);
    while tile > 0.0 && x < end - 0.01 {
        let d = (end - x).min(tile);
        out.push((s[1].0 * 3.0, x, d));
        x += d;
    }
    out.push((s[2].1 * 3.0 - c * 2.0, end, c));
    out
}

/// Draws sprite `base` into `dst` at any size: the edges and corners at
/// their size, the middles repeated (pixel art can't stretch).
fn sliced(img: Image, base: [f32; 4], xs: Slices, ys: Slices, dst: Rect, tint: u32) {
    let (x, y) = (snap(dst.x), snap(dst.y));
    let (cx, cy) = (spans(xs, x, snap(dst.w)), spans(ys, y, snap(dst.h)));
    for &(u, x, w) in &cx {
        for &(v, y, h) in &cy {
            if w > 0.01 && h > 0.01 {
                gfx2d::sprite(img, [base[0] + u, base[1] + v, w * 2.0, h * 2.0], [x, y, w, h], tint);
            }
        }
    }
}

/// Fills `dst` with copies of `src` (a tile) starting at (ox, oy), clipped.
fn tiled(img: Image, src: [f32; 4], dst: Rect, ox: f32, oy: f32, tint: u32) {
    let (tw, th) = (src[2] * 0.5, src[3] * 0.5);
    let mut y = oy;
    while y < dst.bottom() {
        let (y0, y1) = (y.max(dst.y), (y + th).min(dst.bottom()));
        let mut x = ox;
        while x < dst.right() {
            let (x0, x1) = (x.max(dst.x), (x + tw).min(dst.right()));
            if x1 > x0 && y1 > y0 {
                let u = src[0] + (x0 - x) * 2.0;
                let v = src[1] + (y0 - y) * 2.0;
                gfx2d::sprite(img, [u, v, (x1 - x0) * 2.0, (y1 - y0) * 2.0], [x0, y0, x1 - x0, y1 - y0], tint);
            }
            x += tw;
        }
        y += th;
    }
}

/// A square art pixel (or a block of them) in code.
fn pixel(x: f32, y: f32, w: f32, h: f32, c: u32) {
    gfx2d::rect(snap(x), snap(y), w * PX, h * PX, c);
}

// ---- The sky ---------------------------------------------------------------

/// One moment of the day: the sky's colors, how each depth is lit (sprite
/// tints: white = as drawn), the sun or moon, the stars.
#[derive(Clone, Copy, Debug)]
pub struct Sky {
    pub top: u32,
    pub bottom: u32,
    /// Tints for the far mountains, the hills, the near trees, the clouds,
    /// the ground and the obstacles.
    pub far: u32,
    pub mid: u32,
    pub near: u32,
    pub cloud: u32,
    pub ground: u32,
    pub prop: u32,
    /// The sun/moon's height (0 = horizon, 1 = high) and the stars' alpha.
    pub sun_up: f32,
    pub stars: f32,
    /// How dark the world is (0 day … 1 night).
    pub dim: f32,
}

const DAY: Sky = Sky {
    top: hex(0x3d8fe8),
    bottom: hex(0xc4ecff),
    far: 0xffffffff,
    mid: 0xffffffff,
    near: hex(0xb8d4c4),
    cloud: 0xffffffff,
    ground: 0xffffffff,
    prop: 0xffffffff,
    sun_up: 1.0,
    stars: 0.0,
    dim: 0.0,
};
const DUSK: Sky = Sky {
    top: hex(0x5a4a9c),
    bottom: hex(0xffa86b),
    far: hex(0xe0a0c0),
    mid: hex(0xd8a8a0),
    near: hex(0xc09090),
    cloud: hex(0xffc0b0),
    ground: hex(0xe0b8a8),
    prop: hex(0xf0d0c0),
    sun_up: 0.25,
    stars: 0.15,
    dim: 0.25,
};
const NIGHT: Sky = Sky {
    top: hex(0x0b1236),
    bottom: hex(0x24418f),
    far: hex(0x3a4a8a),
    mid: hex(0x34477c),
    near: hex(0x2a3a68),
    cloud: hex(0x5a6aa0),
    ground: hex(0x5a6aa8),
    prop: hex(0x9aa4d8),
    sun_up: 0.8,
    stars: 1.0,
    dim: 0.55,
};
const DAWN: Sky = Sky {
    top: hex(0x6a8ad8),
    bottom: hex(0xffc7a0),
    far: hex(0xc8c0e8),
    mid: hex(0xd0d8d0),
    near: hex(0xb8c8c0),
    cloud: hex(0xffe0d0),
    ground: hex(0xe8dcd0),
    prop: hex(0xf8f0e8),
    sun_up: 0.2,
    stars: 0.2,
    dim: 0.15,
};

/// Meters from one time of day to the next (day → dusk → night → dawn → day).
pub const CYCLE_M: f32 = 260.0;

/// The sky after running `meters`.
pub fn sky_at(meters: f32) -> Sky {
    let keys = [DAY, DUSK, NIGHT, DAWN];
    let p = (meters / CYCLE_M).rem_euclid(4.0);
    let i = p as usize % 4;
    // Hold each look for a while, then blend to the next.
    let k = ((p.fract() - 0.55) / 0.45).clamp(0.0, 1.0);
    let k = k * k * (3.0 - 2.0 * k);
    let (a, b) = (keys[i], keys[(i + 1) % 4]);
    let l = |x: f32, y: f32| x + (y - x) * k;
    Sky {
        top: mix(a.top, b.top, k),
        bottom: mix(a.bottom, b.bottom, k),
        far: mix(a.far, b.far, k),
        mid: mix(a.mid, b.mid, k),
        near: mix(a.near, b.near, k),
        cloud: mix(a.cloud, b.cloud, k),
        ground: mix(a.ground, b.ground, k),
        prop: mix(a.prop, b.prop, k),
        sun_up: l(a.sun_up, b.sun_up),
        stars: l(a.stars, b.stars),
        dim: l(a.dim, b.dim),
    }
}

/// A repeatable pseudo-random number in 0…1 for an integer (scenery layout).
fn hash(i: i32, salt: u32) -> f32 {
    let mut x = (i as u32).wrapping_mul(0x9e37_79b1) ^ salt.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    (x & 0xffff) as f32 / 65535.0
}

/// The sky and the parallax layers behind the play (`scroll` = world px run).
pub fn draw_backdrop(art: &Art, sky: &Sky, scroll: f32, t: f32) {
    // A banded sky, 16-bit style: flat steps from top to horizon. (A margin
    // all round: screen shake rotates the world.)
    let (top, bottom, n) = (-40.0, GROUND_Y + 10.0, 16);
    let band = ((bottom - top) / n as f32 / PX).ceil() * PX;
    for i in 0..n {
        let k = i as f32 / (n - 1) as f32;
        gfx2d::rect(-40.0, top + band * i as f32, W + 80.0, band, mix(sky.top, sky.bottom, k * k));
    }
    // Stars: single pixels, a few bigger twinkling crosses.
    if sky.stars > 0.01 {
        for i in 0..46 {
            let x = (hash(i, 1) * W - scroll * 0.01).rem_euclid(W);
            let y = hash(i, 2) * (GROUND_Y - 170.0);
            let tw = 0.55 + 0.45 * (t * (1.0 + hash(i, 3) * 2.0) + i as f32).sin();
            let c = with_alpha(0xffffffff, sky.stars * tw);
            pixel(x, y, 1.0, 1.0, c);
            if hash(i, 4) > 0.8 {
                pixel(x - PX, y, 1.0, 1.0, with_alpha(c, 0.5 * sky.stars * tw));
                pixel(x + PX, y, 1.0, 1.0, with_alpha(c, 0.5 * sky.stars * tw));
                pixel(x, y - PX, 1.0, 1.0, with_alpha(c, 0.5 * sky.stars * tw));
                pixel(x, y + PX, 1.0, 1.0, with_alpha(c, 0.5 * sky.stars * tw));
            }
        }
    }
    let Some(img) = art.atlas() else { return };
    // The sun by day, the moon by night (low at dusk and dawn).
    let sy = GROUND_Y - 100.0 - 140.0 * sky.sun_up;
    let moon = ((sky.stars - 0.4) / 0.4).clamp(0.0, 1.0);
    if moon < 1.0 {
        blit(img, S::SUN, 272.0 - 21.0, sy - 21.0, with_alpha(0xffffffff, 1.0 - moon));
    }
    if moon > 0.0 {
        blit(img, S::MOON, 272.0 - 16.5, sy - 16.5, with_alpha(0xffffffff, moon));
    }
    // Clouds drift and scroll a little.
    for i in 0..4 {
        let x = (i as f32 * 151.0 - scroll * 0.06 - t * 5.0).rem_euclid(W + 160.0) - 80.0;
        let y = 70.0 + hash(i, 7) * 130.0;
        blit(img, cell(S::CLOUD, i as usize % 3, 3), x - 36.0, y - 18.0, sky.cloud);
    }
    // Mountains, hills, trees: further = slower. Each layer is 720 units
    // wide and wraps; the near trees sit low so their crowns read as a hedge.
    for (i, par, drop, tint, haze) in [(1usize, 0.06, 120.0, sky.far, 0.45), (2, 0.2, 20.0, sky.mid, 0.2), (3, 0.45, 175.0, sky.near, 0.0)] {
        let Some(layer) = art.imgs[i] else { continue };
        let off = snap((scroll * par).rem_euclid(720.0));
        let y = GROUND_Y + 10.0 + drop - 480.0;
        for k in -1..2 {
            gfx2d::sprite(layer, [0.0, 0.0, 1440.0, 960.0], [k as f32 * 720.0 - off, y, 720.0, 480.0], tint);
        }
        // Aerial haze: further layers fade toward the horizon's color.
        if haze > 0.0 {
            let top = GROUND_Y - 190.0;
            gfx2d::rect(-40.0, top, W + 80.0, GROUND_Y + 10.0 - top, with_alpha(sky.bottom, haze * (1.0 - sky.dim)));
        }
    }
}

/// A stretch of solid ground from screen x0 to x1: a grass lip over soil.
pub fn draw_ground(art: &Art, sky: &Sky, x0: f32, x1: f32, scroll: f32) {
    let (x0, x1) = (snap(x0), snap(x1));
    if x1 <= x0 {
        return;
    }
    let Some(img) = art.atlas() else {
        gfx2d::rect(x0, GROUND_Y, x1 - x0, H + 40.0 - GROUND_Y, hex(0x7a4122));
        return;
    };
    let ox = snap(-scroll.rem_euclid(48.0)) - 48.0;
    tiled(img, S::DIRT, Rect::new(x0, GROUND_Y, x1 - x0, H + 40.0 - GROUND_Y), ox, GROUND_Y + 6.0, mix(sky.ground | 0xff, INK, 0.3));
    // Deeper is darker, in flat bands.
    for i in 0..4 {
        let y = GROUND_Y + 48.0 + 42.0 * i as f32;
        gfx2d::rect(x0, y, x1 - x0, H + 40.0 - y, with_alpha(INK, 0.13));
    }
    // The grass: the top rows of the grass tile, an ink line over it, a dark
    // fringe and drips under it, blades poking up.
    let grass = [S::GRASS[0], S::GRASS[1], S::GRASS[2], 21.0];
    let gx = snap(-scroll.rem_euclid(24.0)) - 24.0;
    tiled(img, grass, Rect::new(x0, GROUND_Y - 3.0, x1 - x0, 10.5), gx, GROUND_Y - 3.0, sky.ground);
    let lit = |c: u32| mix(c, sky.ground | 0xff, sky.dim * 0.8);
    gfx2d::rect(x0, GROUND_Y - 4.5, x1 - x0, PX, INK);
    gfx2d::rect(x0, GROUND_Y + 7.5, x1 - x0, PX, lit(LEAF_DARK));
    let step = 9.0;
    let i0 = ((scroll + x0) / step).floor() as i32;
    let i1 = ((scroll + x1) / step).ceil() as i32;
    for i in i0..i1 {
        let x = i as f32 * step - scroll;
        if x < x0 || x + 4.5 > x1 {
            continue;
        }
        let r = hash(i, 34);
        if r < 0.5 {
            pixel(x, GROUND_Y + 9.0, 3.0, 1.0, lit(LEAF_DARK));
            pixel(x + PX, GROUND_Y + 10.5, 1.0, 1.0, lit(LEAF_DARK));
        }
        if r > 0.7 {
            // A blade of grass.
            pixel(x + PX, GROUND_Y - 7.5, 1.0, 2.0, INK);
            pixel(x + PX, GROUND_Y - 6.0, 1.0, 1.0, lit(LEAF_LIGHT));
        }
        if r > 0.94 {
            // A flower.
            pixel(x + 3.0, GROUND_Y - 9.0, 1.0, 1.0, if hash(i, 35) > 0.5 { lit(RED) } else { lit(GOLD) });
        }
    }
    // Ink edges where a pit cuts the ground.
    gfx2d::rect(x0, GROUND_Y - 4.5, PX, H + 40.0 - GROUND_Y, INK);
    gfx2d::rect(x1 - PX, GROUND_Y - 4.5, PX, H + 40.0 - GROUND_Y, INK);
    let _ = LEAF;
}

/// A pit's inside (drawn before the hero, so a falling hero is seen in it).
pub fn draw_pit(art: &Art, sky: &Sky, x0: f32, x1: f32) {
    let (x0, x1) = (snap(x0), snap(x1));
    let r = Rect::new(x0, GROUND_Y, x1 - x0, H + 40.0 - GROUND_Y);
    if let Some(img) = art.atlas() {
        tiled(img, S::DIRT, r, x0, GROUND_Y, mix(sky.ground | 0xff, INK, 0.72));
    }
    for i in 0..5 {
        let y = GROUND_Y + 24.0 + 30.0 * i as f32;
        gfx2d::rect(x0, y, x1 - x0, H + 40.0 - y, with_alpha(INK, 0.3));
    }
    // Water glinting far below.
    gfx2d::rect(x0 + 3.0, H - 33.0, x1 - x0 - 6.0, 3.0, with_alpha(SKY_DEEP, 0.55));
    gfx2d::rect(x0 + 9.0, H - 33.0, 6.0, 1.5, with_alpha(SKY_PALE, 0.7));
}

// ---- The hero ------------------------------------------------------------

/// What the hero is doing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Face {
    Run,
    Jump,
    Spin,
    Worried,
    Phew,
    Happy,
    Dizzy,
    Scared,
}

/// Everything needed to draw the hero this frame.
pub struct Pose {
    /// Feet centre on screen.
    pub x: f32,
    pub y: f32,
    /// Squash and stretch (1 = none), and rotation (radians, clockwise).
    pub sx: f32,
    pub sy: f32,
    pub rot: f32,
    /// Leg cycle in strides; whether the feet are on the ground; going up.
    pub run: f32,
    pub grounded: bool,
    pub rising: bool,
    pub face: Face,
    pub t: f32,
}

/// Where the hero turns and squashes: the middle of the bun, above its feet.
const HERO_MID: f32 = 30.0;

/// The hero: a bread bun in red sneakers, 4 run frames (`hero_run`) and
/// jump / fall / tuck / bonked (`hero_air`), 66 × 66 units with the feet at
/// the bottom, around a 24 × 40 hitbox. Overlays (sweat, dizzy stars) in code.
pub fn draw_hero(art: &Art, p: &Pose) {
    let Some(img) = art.atlas() else { return };
    let run = cell(S::HERO_RUN, (p.run * 4.0).rem_euclid(4.0) as usize, 4);
    let air = |i: usize| cell(S::HERO_AIR, i, 4);
    let src = match p.face {
        Face::Spin => air(2),
        Face::Dizzy => air(3),
        // Jammed in a pit: the run cycle upside down is legs kicking.
        Face::Scared if p.grounded => run,
        Face::Scared => air(1),
        _ if p.grounded => run,
        _ if p.rising => air(0),
        _ => air(1),
    };
    gfx2d::push();
    gfx2d::translate(snap(p.x), snap(p.y - HERO_MID * p.sy));
    gfx2d::rotate(p.rot);
    gfx2d::scale(p.sx, p.sy);
    gfx2d::sprite(img, src, [-33.0, HERO_MID - 64.5, 66.0, 66.0], 0xffffffff);
    match p.face {
        Face::Phew | Face::Worried => {
            // A sweat drop by the brow.
            let (x, y) = (18.0, -24.0 + ((p.t * 6.0).sin() * 1.5).round());
            pixel(x, y, 1.0, 1.0, SKY_PALE);
            pixel(x - PX * 0.5, y + PX, 2.0, 2.0, SKY_DEEP);
            pixel(x - PX * 0.5, y + PX * 3.0, 2.0, 1.0, INK);
        }
        Face::Dizzy => {
            // Stars circling the head.
            for i in 0..3 {
                let a = p.t * 4.0 + i as f32 * TAU / 3.0;
                let (x, y) = (a.cos() * 22.0, -34.0 + a.sin() * 6.0);
                pixel(x - PX, y, 3.0, 1.0, GOLD);
                pixel(x, y - PX, 1.0, 3.0, GOLD);
            }
        }
        _ => {}
    }
    gfx2d::pop();
}

/// The hero's shadow on the ground, shrinking as it rises: a stepped pixel oval.
pub fn draw_shadow(x: f32, h: f32) {
    let k = (1.0 - h / 260.0).clamp(0.25, 1.0);
    let w = (12.0 * k).round();
    let c = with_alpha(INK, 0.3 * k);
    pixel(x - w * PX * 0.5, GROUND_Y - 4.5, w, 1.0, c);
    pixel(x - (w - 4.0) * PX * 0.5, GROUND_Y - 6.0, w - 4.0, 1.0, c);
}

// ---- Obstacles and coins -------------------------------------------------

/// The crate sprite's slices (art pixels): frame and corners kept, the
/// planks repeated.
const CRATE_X: Slices = [(0.0, 5.0), (5.0, 19.0), (19.0, 24.0)];
const CRATE_Y: Slices = [(0.0, 5.0), (10.0, 15.0), (18.0, 24.0)];
/// The log: its cut end, a stretch of bark repeated, the far end.
const LOG_X: Slices = [(5.0, 13.0), (13.0, 38.0), (40.0, 59.0)];
const LOG_Y: Slices = [(1.0, 19.0), (0.0, 0.0), (19.0, 19.0)];

fn crate_box(img: Image, sky: &Sky, x: f32, y: f32, w: f32, h: f32) {
    // The sprite has one clear pixel all round: the wood fills the hitbox.
    sliced(img, S::CRATE, CRATE_X, CRATE_Y, Rect::new(x - PX, y - PX, w + 2.0 * PX, h + 2.0 * PX), sky.prop);
}

/// An obstacle at screen x `sx` (its left edge), bottom `bottom` px above the ground.
pub fn draw_obstacle(art: &Art, sky: &Sky, kind: Kind, sx: f32, w: f32, h: f32, bottom: f32, t: f32) {
    let Some(img) = art.atlas() else { return };
    let y = GROUND_Y - bottom - h; // top on screen
    let (cx, cy) = (sx + w * 0.5, y + h * 0.5);
    match kind {
        Kind::Crate => crate_box(img, sky, sx, y, w, h),
        Kind::Stack => {
            let n = if h > 100.0 { 3 } else { 2 };
            let ch = h / n as f32;
            for i in 0..n {
                let wob = if i % 2 == 1 { 3.0 } else { 0.0 };
                crate_box(img, sky, sx + wob, GROUND_Y - ch * (i + 1) as f32, w, ch);
            }
        }
        Kind::Log => sliced(img, S::LOG, LOG_X, LOG_Y, Rect::new(sx, GROUND_Y - bottom - 27.0, w, 27.0), sky.prop),
        // Birds and the bee: frames bottom-anchored in their cells; the
        // offsets put the body (not the raised wings) on the hitbox.
        Kind::Gull => blit(img, cell(S::GULL, (t * 9.0) as usize, 4), cx - 39.0, cy - 30.0, sky.prop),
        Kind::Skimmer => {
            // Speed lines behind it.
            for i in 0..3 {
                let yy = cy - 6.0 + i as f32 * 6.0;
                pixel(cx + 30.0 + i as f32 * 3.0, yy, 6.0 + i as f32 * 2.0, 1.0, with_alpha(0xffffffff, 0.6));
            }
            blit(img, cell(S::SKIMMER, (t * 16.0) as usize, 4), cx - 39.0, cy - 37.5, sky.prop);
        }
        Kind::Bee => blit(img, cell(S::BEE, (t * 24.0) as usize, 2), cx - 24.0, cy - 28.5, sky.prop),
        Kind::Pit => {}
    }
}

/// A bee's up-and-down track (a dotted line), so its rhythm reads ahead of time.
pub fn draw_bee_track(cx: f32, top: f32, bottom: f32) {
    let mut y = top;
    while y < bottom {
        pixel(cx - 0.75, y, 1.0, 1.0, 0xffffff90);
        y += 9.0;
    }
}

/// A coin at (x, y) on screen; `spin` (radians) turns it.
pub fn draw_coin(art: &Art, x: f32, y: f32, spin: f32) {
    let Some(img) = art.atlas() else { return };
    let i = (spin / TAU * 6.0).rem_euclid(6.0) as usize;
    blit(img, cell(S::COIN, i, 6), x - 12.0, y - 12.0, 0xffffffff);
}

/// A warning at the right edge for something flying in from off screen.
pub fn draw_incoming(y: f32, t: f32) {
    let pulse = 1.0 + 0.12 * (t * 14.0).sin();
    gfx2d::push();
    gfx2d::translate(W - 22.0, snap(y));
    gfx2d::scale(pulse, pulse);
    shape::poly(&[10.0, 0.0, -2.0, -12.0, -2.0, 12.0], INK);
    shape::disc(-8.0, 0.0, 13.0, INK);
    shape::disc(-8.0, 0.0, 11.0, RED);
    maimbrain::ui::text("!").font(Font::SansBold).size(18.0).color(0xffffffff).middle().draw(-8.0, 0.5);
    gfx2d::pop();
}
