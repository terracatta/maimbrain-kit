//! A vector icon set drawn with mb2d (original drawings): anti-aliased,
//! one color (details in darker or lighter shades of it), any size.
//!
//! ```ignore
//! use maimbrain::ui::Icon;
//! Icon::TROPHY.draw(180.0, 300.0, 48.0, 0xffd23fff);
//! ```
//!
//! An [`Icon`] is a plain function pointer, so a game links only the icons
//! it names (size matters, SPEC §1). [`Icon::ALL`] lists every one.

#![allow(clippy::too_many_arguments)]

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use super::color::{darken, lighten};
use super::shape::{arc, capsule, disc, poly, ring, rounded_points, star};
use crate::gfx2d;

/// Draws at (cx, cy) fitting a `size` × `size` box, in `color`.
#[derive(Clone, Copy)]
pub struct Icon(pub fn(cx: f32, cy: f32, size: f32, color: u32));

impl std::fmt::Debug for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Icon")
    }
}

impl Icon {
    pub fn draw(self, cx: f32, cy: f32, size: f32, color: u32) {
        (self.0)(cx, cy, size, color)
    }

    pub const PLAY: Icon = Icon(play);
    pub const PAUSE: Icon = Icon(pause);
    pub const RESTART: Icon = Icon(restart);
    pub const STAR: Icon = Icon(star_icon);
    pub const HEART: Icon = Icon(heart);
    pub const TROPHY: Icon = Icon(trophy);
    pub const CROWN: Icon = Icon(crown);
    pub const LOCK: Icon = Icon(lock);
    pub const UNLOCK: Icon = Icon(unlock);
    pub const CHECK: Icon = Icon(check);
    pub const CLOSE: Icon = Icon(close);
    pub const PLUS: Icon = Icon(plus);
    pub const MINUS: Icon = Icon(minus);
    pub const ARROW_UP: Icon = Icon(arrow_up);
    pub const ARROW_DOWN: Icon = Icon(arrow_down);
    pub const ARROW_LEFT: Icon = Icon(arrow_left);
    pub const ARROW_RIGHT: Icon = Icon(arrow_right);
    pub const HOME: Icon = Icon(home);
    pub const GEAR: Icon = Icon(gear);
    pub const SOUND: Icon = Icon(sound);
    pub const MUTE: Icon = Icon(mute);
    pub const MUSIC: Icon = Icon(music);
    pub const COIN: Icon = Icon(coin);
    pub const BOLT: Icon = Icon(bolt);
    pub const FLAG: Icon = Icon(flag);
    pub const CLOCK: Icon = Icon(clock);
    pub const MEDAL: Icon = Icon(medal);
    pub const FIRE: Icon = Icon(fire);
    pub const GEM: Icon = Icon(gem);
    pub const SHIELD: Icon = Icon(shield);
    pub const TARGET: Icon = Icon(target);
    pub const PERSON: Icon = Icon(person);
    pub const BOMB: Icon = Icon(bomb);

    /// Every icon with its name (galleries; linking this links them all).
    pub const ALL: [(&'static str, Icon); 33] = [
        ("play", Icon::PLAY), ("pause", Icon::PAUSE), ("restart", Icon::RESTART), ("star", Icon::STAR),
        ("heart", Icon::HEART), ("trophy", Icon::TROPHY), ("crown", Icon::CROWN), ("lock", Icon::LOCK),
        ("unlock", Icon::UNLOCK), ("check", Icon::CHECK), ("close", Icon::CLOSE), ("plus", Icon::PLUS),
        ("minus", Icon::MINUS), ("up", Icon::ARROW_UP), ("down", Icon::ARROW_DOWN), ("left", Icon::ARROW_LEFT),
        ("right", Icon::ARROW_RIGHT), ("home", Icon::HOME), ("gear", Icon::GEAR), ("sound", Icon::SOUND),
        ("mute", Icon::MUTE), ("music", Icon::MUSIC), ("coin", Icon::COIN), ("bolt", Icon::BOLT),
        ("flag", Icon::FLAG), ("clock", Icon::CLOCK), ("medal", Icon::MEDAL), ("fire", Icon::FIRE),
        ("gem", Icon::GEM), ("shield", Icon::SHIELD), ("target", Icon::TARGET), ("person", Icon::PERSON),
        ("bomb", Icon::BOMB),
    ];
}

/// Unit-box points ([-1, 1], y down) scaled to the icon's box.
fn pts(cx: f32, cy: f32, u: f32, unit: &[f32]) -> Vec<f32> {
    unit.as_chunks::<2>().0.iter().flat_map(|p| [cx + p[0] * u, cy + p[1] * u]).collect()
}

/// A rounded polygon from unit-box points.
fn shape(cx: f32, cy: f32, u: f32, unit: &[f32], round: f32, color: u32) {
    poly(&rounded_points(&pts(cx, cy, u, unit), round * u, 4), color);
}

/// A rounded rect from unit-box corners.
fn bx(cx: f32, cy: f32, u: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, color: u32) {
    let (x, y) = (cx + x0 * u, cy + y0 * u);
    gfx2d::rrect(x, y, (x1 - x0) * u, (y1 - y0) * u, r * u, 0.0, 0.0, color, color);
}

fn cap(cx: f32, cy: f32, u: f32, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, color: u32) {
    capsule(cx + x0 * u, cy + y0 * u, cx + x1 * u, cy + y1 * u, w * u, color);
}

fn play(cx: f32, cy: f32, s: f32, c: u32) {
    shape(cx, cy, s / 2.0, &[-0.5, -0.82, 0.86, 0.0, -0.5, 0.82], 0.2, c);
}

fn pause(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    bx(cx, cy, u, -0.64, -0.78, -0.18, 0.78, 0.15, c);
    bx(cx, cy, u, 0.18, -0.78, 0.64, 0.78, 0.15, c);
}

fn restart(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    let (r, w) = (0.78 * u, 0.24 * u);
    let end = TAU - 0.55;
    arc(cx, cy + 0.04 * u, r, w, 0.75, end, c);
    // The arrowhead at the arc's end, pointing on round (clockwise).
    let rm = r - w * 0.5;
    let a = end + 0.12;
    let (px, py) = (cx + rm * a.sin(), cy + 0.04 * u - rm * a.cos());
    let (tx, ty) = (a.cos(), a.sin());
    let (nx, ny) = (a.sin(), -a.cos());
    let head = [
        px + tx * 0.42 * u, py + ty * 0.42 * u,
        px - nx * 0.36 * u - tx * 0.12 * u, py - ny * 0.36 * u - ty * 0.12 * u,
        px + nx * 0.36 * u - tx * 0.12 * u, py + ny * 0.36 * u - ty * 0.12 * u,
    ];
    poly(&rounded_points(&head, 0.06 * u, 3), c);
}

fn star_icon(cx: f32, cy: f32, s: f32, c: u32) {
    star(cx, cy + s * 0.04, s * 0.5, 0.0, c);
}

fn heart(cx: f32, cy: f32, s: f32, c: u32) {
    let k = s * 0.5 * 0.94 / 16.0;
    let mut v = Vec::with_capacity(96);
    for i in 0..48 {
        let t = i as f32 / 48.0 * TAU;
        let x = 16.0 * t.sin().powi(3);
        let y = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
        v.extend([cx + x * k, cy + (y - 2.65) * k]);
    }
    poly(&v, c);
}

fn trophy(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    ring(cx - 0.6 * u, cy - 0.42 * u, 0.3 * u, 0.12 * u, c);
    ring(cx + 0.6 * u, cy - 0.42 * u, 0.3 * u, 0.12 * u, c);
    shape(cx, cy, u, &[-0.62, -0.86, 0.62, -0.86, 0.58, -0.3, 0.42, 0.02, 0.16, 0.2, -0.16, 0.2, -0.42, 0.02, -0.58, -0.3], 0.1, c);
    bx(cx, cy, u, -0.11, 0.1, 0.11, 0.56, 0.0, c);
    bx(cx, cy, u, -0.32, 0.44, 0.32, 0.6, 0.06, c);
    bx(cx, cy, u, -0.5, 0.6, 0.5, 0.88, 0.1, c);
    // A shine on the cup.
    cap(cx, cy, u, -0.36, -0.66, -0.3, -0.24, 0.1, lighten(c, 0.45));
}

fn crown(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    shape(cx, cy, u, &[-0.86, 0.5, -0.86, -0.42, -0.44, 0.02, 0.0, -0.66, 0.44, 0.02, 0.86, -0.42, 0.86, 0.5], 0.08, c);
    for (x, y) in [(-0.86, -0.56), (0.0, -0.8), (0.86, -0.56)] {
        disc(cx + x * u, cy + y * u, 0.14 * u, c);
    }
    bx(cx, cy, u, -0.86, 0.58, 0.86, 0.84, 0.07, c);
    disc(cx, cy + 0.16 * u, 0.12 * u, darken(c, 0.3));
}

fn lock_body(cx: f32, cy: f32, u: f32, c: u32) {
    bx(cx, cy, u, -0.72, -0.16, 0.72, 0.88, 0.2, c);
    let d = darken(c, 0.45);
    disc(cx, cy + 0.26 * u, 0.15 * u, d);
    bx(cx, cy, u, -0.06, 0.26, 0.06, 0.6, 0.05, d);
}

fn lock(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    arc(cx, cy - 0.36 * u, 0.5 * u, 0.2 * u, -FRAC_PI_2, FRAC_PI_2, c);
    bx(cx, cy, u, -0.5, -0.38, -0.3, -0.1, 0.0, c);
    bx(cx, cy, u, 0.3, -0.38, 0.5, -0.1, 0.0, c);
    lock_body(cx, cy, u, c);
}

fn unlock(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    arc(cx + 0.36 * u, cy - 0.6 * u, 0.42 * u, 0.2 * u, -FRAC_PI_2, FRAC_PI_2, c);
    bx(cx, cy, u, -0.06, -0.62, 0.14, -0.1, 0.0, c);
    bx(cx, cy, u, 0.58, -0.62, 0.78, -0.44, 0.0, c);
    lock_body(cx, cy, u, c);
}

fn check(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    cap(cx, cy, u, -0.62, 0.04, -0.2, 0.48, 0.28, c);
    cap(cx, cy, u, -0.2, 0.48, 0.66, -0.5, 0.28, c);
}

fn close(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    cap(cx, cy, u, -0.56, -0.56, 0.56, 0.56, 0.26, c);
    cap(cx, cy, u, 0.56, -0.56, -0.56, 0.56, 0.26, c);
}

fn plus(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    cap(cx, cy, u, -0.68, 0.0, 0.68, 0.0, 0.28, c);
    cap(cx, cy, u, 0.0, -0.68, 0.0, 0.68, 0.28, c);
}

fn minus(cx: f32, cy: f32, s: f32, c: u32) {
    cap(cx, cy, s / 2.0, -0.68, 0.0, 0.68, 0.0, 0.28, c);
}

/// An arrow pointing right, turned by `angle` (radians, clockwise).
fn arrow(cx: f32, cy: f32, s: f32, c: u32, angle: f32) {
    let u = s / 2.0;
    gfx2d::push();
    gfx2d::translate(cx, cy);
    gfx2d::rotate(angle);
    cap(0.0, 0.0, u, -0.7, 0.0, 0.3, 0.0, 0.26, c);
    shape(0.0, 0.0, u, &[0.02, -0.62, 0.84, 0.0, 0.02, 0.62], 0.1, c);
    gfx2d::pop();
}

fn arrow_right(cx: f32, cy: f32, s: f32, c: u32) {
    arrow(cx, cy, s, c, 0.0);
}
fn arrow_down(cx: f32, cy: f32, s: f32, c: u32) {
    arrow(cx, cy, s, c, FRAC_PI_2);
}
fn arrow_left(cx: f32, cy: f32, s: f32, c: u32) {
    arrow(cx, cy, s, c, PI);
}
fn arrow_up(cx: f32, cy: f32, s: f32, c: u32) {
    arrow(cx, cy, s, c, -FRAC_PI_2);
}

fn home(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    shape(cx, cy, u, &[-0.92, -0.02, 0.0, -0.86, 0.92, -0.02], 0.12, c);
    bx(cx, cy, u, -0.62, -0.2, 0.62, 0.84, 0.12, c);
    bx(cx, cy, u, -0.18, 0.26, 0.18, 0.84, 0.08, darken(c, 0.4));
}

fn gear(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    let mut v = Vec::with_capacity(64);
    for k in 0..8 {
        let a = k as f32 * TAU / 8.0;
        for (da, r) in [(-0.27, 0.7), (-0.16, 0.96), (0.16, 0.96), (0.27, 0.7)] {
            let t = a + da;
            v.extend([cx + r * u * t.sin(), cy - r * u * t.cos()]);
        }
    }
    poly(&rounded_points(&v, 0.06 * u, 2), c);
    disc(cx, cy, 0.3 * u, darken(c, 0.45));
}

fn speaker(cx: f32, cy: f32, u: f32, c: u32) {
    shape(cx, cy, u, &[-0.88, -0.3, -0.46, -0.3, 0.04, -0.76, 0.04, 0.76, -0.46, 0.3, -0.88, 0.3], 0.1, c);
}

fn sound(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    speaker(cx, cy, u, c);
    arc(cx + 0.04 * u, cy, 0.5 * u, 0.15 * u, PI / 4.0, 3.0 * PI / 4.0, c);
    arc(cx + 0.04 * u, cy, 0.86 * u, 0.15 * u, PI / 4.0 + 0.1, 3.0 * PI / 4.0 - 0.1, c);
}

fn mute(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    speaker(cx, cy, u, c);
    cap(cx, cy, u, 0.32, -0.28, 0.86, 0.28, 0.17, c);
    cap(cx, cy, u, 0.86, -0.28, 0.32, 0.28, 0.17, c);
}

fn music(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    disc(cx - 0.46 * u, cy + 0.56 * u, 0.27 * u, c);
    disc(cx + 0.52 * u, cy + 0.4 * u, 0.27 * u, c);
    bx(cx, cy, u, -0.31, -0.58, -0.17, 0.56, 0.0, c);
    bx(cx, cy, u, 0.67, -0.74, 0.81, 0.4, 0.0, c);
    shape(cx, cy, u, &[-0.31, -0.52, 0.81, -0.68, 0.81, -0.96, -0.31, -0.8], 0.04, c);
}

fn coin(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    disc(cx, cy, 0.92 * u, c);
    ring(cx, cy, 0.7 * u, 0.09 * u, darken(c, 0.25));
    star(cx, cy + 0.03 * u, 0.4 * u, 0.0, darken(c, 0.25));
    arc(cx, cy, 0.82 * u, 0.08 * u, -1.2, -0.3, lighten(c, 0.5));
}

fn bolt(cx: f32, cy: f32, s: f32, c: u32) {
    shape(cx, cy, s / 2.0, &[0.18, -0.96, -0.62, 0.14, -0.06, 0.14, -0.22, 0.96, 0.62, -0.2, 0.06, -0.2], 0.06, c);
}

fn flag(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    cap(cx, cy, u, -0.66, -0.84, -0.66, 0.9, 0.17, c);
    shape(cx, cy, u, &[-0.6, -0.86, 0.0, -0.7, 0.78, -0.86, 0.78, 0.06, 0.0, 0.22, -0.6, 0.06], 0.07, c);
}

fn clock(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    ring(cx, cy, 0.92 * u, 0.17 * u, c);
    cap(cx, cy, u, 0.0, 0.0, 0.0, -0.52, 0.15, c);
    cap(cx, cy, u, 0.0, 0.0, 0.36, 0.14, 0.15, c);
}

fn medal(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    let d = darken(c, 0.3);
    shape(cx, cy, u, &[-0.56, -0.96, -0.18, -0.96, 0.12, -0.2, -0.24, -0.2], 0.04, d);
    shape(cx, cy, u, &[0.56, -0.96, 0.18, -0.96, -0.12, -0.2, 0.24, -0.2], 0.04, d);
    disc(cx, cy + 0.32 * u, 0.62 * u, c);
    star(cx, cy + 0.35 * u, 0.36 * u, 0.0, d);
}

fn flame(cx: f32, cy: f32, a: f32, b: f32, c: u32) {
    let mut v = Vec::with_capacity(64);
    for i in 0..32 {
        let t = i as f32 / 32.0 * TAU;
        v.extend([cx + a * t.sin() * (t / 2.0).sin(), cy - b * t.cos()]);
    }
    poly(&v, c);
}

fn fire(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    // A flame with a lick either side, and a bright core.
    shape(cx, cy, u, &[0.06, -0.98, 0.34, -0.5, 0.52, -0.76, 0.76, -0.26, 0.8, 0.28, 0.56, 0.74, 0.0, 0.94, -0.56, 0.74, -0.8, 0.28, -0.68, -0.2, -0.44, 0.0, -0.36, -0.46], 0.14, c);
    flame(cx, cy + 0.44 * u, 0.48 * u, 0.48 * u, lighten(c, 0.55));
}

fn gem(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    shape(cx, cy, u, &[-0.9, -0.3, -0.46, -0.78, 0.46, -0.78, 0.9, -0.3, 0.0, 0.88], 0.06, c);
    poly(&pts(cx, cy, u, &[-0.42, -0.72, 0.42, -0.72, 0.24, -0.3, -0.24, -0.3]), lighten(c, 0.35));
    poly(&pts(cx, cy, u, &[0.24, -0.3, 0.84, -0.3, 0.0, 0.8]), darken(c, 0.2));
}

fn shield(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    shape(cx, cy, u, &[-0.78, -0.72, 0.0, -0.96, 0.78, -0.72, 0.72, 0.12, 0.0, 0.96, -0.72, 0.12], 0.14, c);
    shape(cx, cy, u, &[0.0, -0.74, 0.56, -0.56, 0.52, 0.08, 0.0, 0.72], 0.08, lighten(c, 0.3));
}

fn target(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    ring(cx, cy, 0.92 * u, 0.17 * u, c);
    ring(cx, cy, 0.56 * u, 0.17 * u, c);
    disc(cx, cy, 0.2 * u, c);
}

fn person(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    disc(cx, cy - 0.42 * u, 0.4 * u, c);
    shape(cx, cy, u, &[-0.82, 0.92, -0.72, 0.36, -0.36, 0.08, 0.36, 0.08, 0.72, 0.36, 0.82, 0.92], 0.24, c);
}

fn bomb(cx: f32, cy: f32, s: f32, c: u32) {
    let u = s / 2.0;
    disc(cx - 0.1 * u, cy + 0.16 * u, 0.74 * u, c);
    bx(cx, cy, u, 0.12, -0.72, 0.48, -0.42, 0.06, c);
    arc(cx + 0.62 * u, cy - 0.58 * u, 0.32 * u, 0.1 * u, -PI, -FRAC_PI_2 + 0.2, c);
    super::shape::sparkle(cx + 0.66 * u, cy - 0.9 * u, 0.22 * u, 0.3, lighten(c, 0.6));
    disc(cx - 0.34 * u, cy - 0.08 * u, 0.13 * u, lighten(c, 0.35));
}
