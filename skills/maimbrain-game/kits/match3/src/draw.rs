//! Drawing: poppets (painted bodies from the atlas, faces, specials), the
//! painted window and its night, the board, the wake meter, blasts and the
//! swipe hint. All read-only. Every painted asset has a code-drawn fallback
//! (the first frames draw before the PNGs arrive).

use std::f32::consts::{PI, TAU};

use maimbrain::gfx2d::{self, Blend, Image};
use maimbrain::motion::Ease;
use maimbrain::sys::{Asset, AssetState};
use maimbrain::ui::{Rect, darken, fade, lighten, mix, shape, text, with_alpha};

use crate::look::*;
use crate::sim::{Blast, BlastKind, COLS, Kind, Piece, ROWS, Special};

/// How a face looks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mood {
    Happy,
    /// Picked up, swapped, about to pop: big eyes, open mouth.
    Excited,
    /// A swap that didn't work.
    Huh,
    /// Eyelids drooping (0 awake … 1 nearly shut).
    Sleepy(f32),
    Asleep,
    /// Popping: squeezed shut, mouth wide.
    Squeeze,
}

/// The generated art: the cast atlas, the painted window, the paper grain.
pub struct Art {
    assets: [Asset; 3],
    pub cast: Option<Image>,
    pub bg: Option<Image>,
    pub paper: Option<Image>,
}

/// Texel size of the painted window (as generated).
const BG_TEXELS: [f32; 4] = [0.0, 0.0, 540.0, 960.0];

impl Art {
    pub fn new() -> Art {
        Art {
            assets: [Asset::load("assets/cast.png"), Asset::load("assets/bg.png"), Asset::load("assets/paper.png")],
            cast: None,
            bg: None,
            paper: None,
        }
    }

    /// Per update: picks up images as they finish loading.
    pub fn poll(&mut self) {
        let slots = [&mut self.cast, &mut self.bg, &mut self.paper];
        for (slot, a) in slots.into_iter().zip(self.assets) {
            if slot.is_none() && a.state() == AssetState::Ready {
                *slot = Image::new(a);
            }
        }
    }
}

/// Unit-radius outlines for each poppet shape (the fallback before the atlas
/// loads), and the art.
pub struct Shapes {
    pub piece: [Vec<f32>; 7],
    pub rock: Vec<f32>,
    pub art: Art,
}

fn regular(n: usize, r: f32, rot: f32) -> Vec<f32> {
    (0..n).flat_map(|i| {
        let a = rot + i as f32 * TAU / n as f32;
        [r * a.sin(), -r * a.cos()]
    }).collect()
}

impl Shapes {
    pub fn new() -> Shapes {
        let round = shape::rounded_points;
        let heart: Vec<f32> = (0..40)
            .flat_map(|i| {
                let t = i as f32 / 40.0 * TAU;
                let x = 16.0 * t.sin().powi(3);
                let y = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
                [x / 16.5, y / 16.5 + 0.1]
            })
            .collect();
        let cloud: Vec<f32> = (0..48)
            .flat_map(|i| {
                let a = i as f32 / 48.0 * TAU;
                let r = 0.9 + 0.1 * (a * 6.0).cos().abs();
                [r * a.sin(), -r * a.cos()]
            })
            .collect();
        Shapes {
            piece: [
                regular(32, 0.98, 0.0),
                round(&[-0.84, -0.84, 0.84, -0.84, 0.84, 0.84, -0.84, 0.84], 0.32, 4),
                round(&[0.0, -1.08, 1.0, 0.8, -1.0, 0.8], 0.3, 4),
                round(&regular(6, 1.0, 0.0), 0.2, 3),
                cloud,
                round(&shape::star_points(0.0, 0.08, 1.12, 0.56, 5, 0.0), 0.16, 3),
                heart,
            ],
            rock: round(&[-0.9, -0.5, -0.4, -0.95, 0.45, -0.88, 0.95, -0.3, 0.85, 0.6, 0.3, 0.92, -0.5, 0.9, -0.95, 0.35], 0.25, 3),
            art: Art::new(),
        }
    }
}

/// Draws unit points at (x, y), scaled by s.
fn at(pts: &[f32], x: f32, y: f32, s: f32, color: u32) {
    gfx2d::push();
    gfx2d::translate(x, y);
    gfx2d::scale(s, s);
    shape::poly(pts, color);
    gfx2d::pop();
}

/// A painted body from the atlas, in unit coordinates (the piece's transform is current).
fn body(img: Image, src: [f32; 4], alpha: f32) {
    let h = SPRITE_SPAN / 2.0;
    gfx2d::sprite(img, src, [-h, -h, SPRITE_SPAN, SPRITE_SPAN], fade(0xffffffff, alpha));
}

/// A poppet at (x, y), `r` from centre to edge, squashed by (sx, sy).
#[allow(clippy::too_many_arguments)]
pub fn piece(sh: &Shapes, p: &Piece, x: f32, y: f32, r: f32, sx: f32, sy: f32, mood: Mood, look: (f32, f32), t: f32, alpha: f32) {
    if alpha <= 0.01 || r <= 0.5 {
        return;
    }
    let a = |c: u32| fade(c, alpha);
    gfx2d::push();
    gfx2d::translate(x, y);
    gfx2d::scale(r * sx, r * sy);
    let blink = ((t * 0.31 + p.id as f32 * 0.6180339).fract() < 0.035) && !matches!(mood, Mood::Asleep | Mood::Squeeze);
    let cast = sh.art.cast;
    match p.kind {
        Kind::Color(k) => {
            let col = PIECES[k as usize];
            if matches!(p.special, Special::LineH | Special::LineV) {
                let pulse = 0.5 + 0.5 * (t * 7.0).sin();
                gfx2d::blend(Blend::Add);
                shape::soft_disc(0.0, 0.0, 1.25, 0.55, a(with_alpha(0x9fd0ffff, 0.3 + 0.25 * pulse)));
                gfx2d::blend(Blend::Alpha);
            }
            if p.special == Special::Bomb {
                let pulse = 0.5 + 0.5 * (t * 6.0).sin();
                gfx2d::blend(Blend::Add);
                shape::soft_disc(0.0, 0.0, 1.3 + 0.1 * pulse, 0.55, a(with_alpha(LAMP, 0.4 + 0.25 * pulse)));
                gfx2d::blend(Blend::Alpha);
                shape::ring(0.0, 0.0, 1.24, 0.1, a(with_alpha(INK, 0.8)));
                shape::ring(0.0, 0.0, 1.16, 0.06, a(GOLD));
            }
            shape::soft_disc(0.0, 0.2, 0.9, 0.35, a(0x0d0a2048));
            if let Some(img) = cast {
                body(img, SPRITES[k as usize], alpha);
            } else {
                let pts = &sh.piece[k as usize];
                at(pts, 0.0, 0.0, 1.0, a(darken(col, 0.45)));
                at(pts, 0.0, 0.0, 0.88, a(col));
                shape::soft_disc(-0.3, -0.38, 0.3, 0.3, a(with_alpha(lighten(col, 0.5), 0.7)));
            }
            match p.special {
                Special::LineH | Special::LineV => {
                    let h = p.special == Special::LineH;
                    let push = 1.18 + 0.08 * (t * 7.0).sin().abs();
                    for s in [-1.0f32, 1.0] {
                        let (cx, cy) = if h { (s * push, 0.0) } else { (0.0, s * push) };
                        let tri = if h {
                            [cx + s * 0.34, cy, cx - s * 0.12, cy - 0.32, cx - s * 0.12, cy + 0.32]
                        } else {
                            [cx, cy + s * 0.34, cx - 0.32, cy - s * 0.12, cx + 0.32, cy - s * 0.12]
                        };
                        let out: Vec<f32> = tri.chunks(2).flat_map(|p| [cx + (p[0] - cx) * 1.45, cy + (p[1] - cy) * 1.45]).collect();
                        shape::poly(&out, a(INK));
                        shape::poly(&tri, a(PENCIL));
                        // A painted stripe across the body, the way it will blast.
                        if h {
                            shape::capsule(-0.62, s * 0.5, 0.62, s * 0.5, 0.13, a(with_alpha(PENCIL, 0.85)));
                        } else {
                            shape::capsule(s * 0.5, -0.62, s * 0.5, 0.62, 0.13, a(with_alpha(PENCIL, 0.85)));
                        }
                    }
                }
                Special::Bomb => {
                    let k = t * 3.0;
                    shape::sparkle(0.0, -1.2, 0.34 + 0.08 * (t * 13.0).sin(), k, a(0xfff3c0ff));
                }
                Special::None => {}
            }
            face(mood, look, blink, FACE_Y[k as usize], alpha);
            if p.star {
                let tw = 1.0 + 0.12 * (t * 5.0 + p.id as f32).sin();
                shape::star(0.68, -0.68, 0.44 * tw, 0.0, a(INK));
                shape::star(0.68, -0.68, 0.34 * tw, 0.0, a(GOLD));
            }
        }
        Kind::Rainbow => {
            shape::soft_disc(0.0, 0.2, 0.9, 0.35, a(0x0d0a2048));
            let spin = t * 1.6;
            gfx2d::blend(Blend::Add);
            shape::soft_disc(0.0, 0.0, 1.3, 0.6, a(with_alpha(0xfff0d0ff, 0.25 + 0.1 * (t * 4.0).sin())));
            gfx2d::blend(Blend::Alpha);
            if let Some(img) = cast {
                gfx2d::push();
                gfx2d::rotate(0.08 * (spin * 1.3).sin());
                body(img, crate::cast::RAINBOW, alpha);
                gfx2d::pop();
            } else {
                shape::disc(0.0, 0.0, 1.0, a(INK));
                for i in 0..6 {
                    let a0 = spin + i as f32 * TAU / 6.0;
                    let mut pts = vec![0.0, 0.0];
                    for k in 0..=6 {
                        let ang = a0 + k as f32 * TAU / 36.0;
                        pts.extend([0.88 * ang.sin(), -0.88 * ang.cos()]);
                    }
                    shape::poly(&pts, a(PIECES[i]));
                }
                shape::disc(0.0, 0.0, 0.62, a(0xfff6e8e8));
            }
            face(mood, look, blink, FACE_Y[8], alpha);
        }
        Kind::Rock => {
            shape::soft_disc(0.0, 0.2, 0.9, 0.35, a(0x0d0a2048));
            if let Some(img) = cast {
                body(img, crate::cast::ROCK, alpha);
            } else {
                at(&sh.rock, 0.0, 0.0, 1.0, a(darken(ROCK, 0.55)));
                at(&sh.rock, 0.0, 0.0, 0.86, a(ROCK));
                at(&sh.rock, -0.08, -0.16, 0.55, a(lighten(ROCK, 0.25)));
            }
            // A grumpy face: beady eyes under flat, frowning brows.
            let fy = FACE_Y[7];
            let ink = a(INK);
            for s in [-1.0f32, 1.0] {
                shape::disc(s * 0.3, fy + 0.02, 0.085, ink);
                shape::capsule(s * 0.14, fy - 0.12, s * 0.46, fy - 0.2, 0.065, ink);
            }
            shape::arc(0.0, fy + 0.5, 0.18, 0.06, -PI * 0.28, PI * 0.28, ink);
        }
    }
    gfx2d::pop();
}

/// An ellipse (rx, ry) at (x, y).
fn oval(x: f32, y: f32, rx: f32, ry: f32, color: u32) {
    gfx2d::push();
    gfx2d::translate(x, y);
    gfx2d::scale(rx, ry);
    shape::disc(0.0, 0.0, 1.0, color);
    gfx2d::pop();
}

/// A storybook face in unit coordinates (the piece's transform is current):
/// ink-oval eyes that close from the top as the poppet gets sleepy, a lid line
/// with a lash, rosy cheeks, a small pencil mouth. `fy` moves it down.
fn face(mood: Mood, look: (f32, f32), blink: bool, fy: f32, alpha: f32) {
    gfx2d::push();
    gfx2d::translate(0.0, fy);
    gfx2d::scale(FACE_SCALE, FACE_SCALE);
    face_at_origin(mood, look, blink, alpha);
    gfx2d::pop();
}

fn face_at_origin(mood: Mood, look: (f32, f32), blink: bool, alpha: f32) {
    let fy = 0.0;
    let a = |c: u32| fade(c, alpha);
    let ink = a(INK);
    let (lx, ly) = (look.0 * 0.05, look.1 * 0.04);
    let ey = fy - 0.04;
    let big = if mood == Mood::Excited { 1.2 } else { 1.0 };
    let (rx, ry) = (0.095 * big, 0.13 * big);
    // Cheeks first, under everything.
    let blush = match mood {
        Mood::Asleep => 0.55,
        Mood::Sleepy(d) => 0.4 + 0.2 * d,
        _ => 0.42,
    };
    for s in [-1.0f32, 1.0] {
        shape::soft_disc(s * 0.5, ey + 0.24, 0.17, 0.12, a(with_alpha(BLUSH, blush)));
    }
    for s in [-1.0f32, 1.0] {
        let ex = s * 0.3;
        match mood {
            Mood::Asleep => shape::arc(ex, ey - 0.08, 0.12, 0.055, PI * 0.68, PI * 1.32, ink),
            Mood::Squeeze => shape::polyline(&[ex - s * 0.12, ey - 0.1, ex + s * 0.05, ey, ex - s * 0.12, ey + 0.1], 0.065, ink),
            _ if blink => shape::arc(ex, ey - 0.07, 0.11, 0.05, PI * 0.7, PI * 1.3, ink),
            _ => {
                let d = if let Mood::Sleepy(d) = mood { d.clamp(0.0, 1.0) } else { 0.0 };
                // The eye shuts from the top: what's left of it hangs under the lid.
                let open = 1.0 - 0.82 * d;
                let h = ry * open;
                let cy = ey + ry - h;
                oval(ex + lx * open, cy + ly * open, rx, h, ink);
                if open > 0.45 {
                    shape::disc(ex + lx - 0.03, cy + ly - h * 0.45, 0.035 * big, a(0xfffaf0ff));
                }
                if d > 0.05 {
                    // The lid: a heavy line across the top, drooping at the outer end, with a lash.
                    let top = cy - h;
                    let droop = 0.05 * d;
                    shape::capsule(ex - s * 0.13, top + 0.005, ex + s * 0.14, top + droop, 0.06, ink);
                    shape::capsule(ex + s * 0.14, top + droop, ex + s * 0.2, top + droop - 0.05, 0.04, ink);
                }
                if mood == Mood::Huh {
                    shape::capsule(ex - 0.13, ey - 0.27 - s * 0.04, ex + 0.13, ey - 0.27 + s * 0.04, 0.05, ink);
                }
            }
        }
    }
    let my = ey + 0.3;
    match mood {
        Mood::Happy => shape::arc(0.0, my - 0.13, 0.14, 0.055, PI * 0.7, PI * 1.3, ink),
        Mood::Excited | Mood::Squeeze => {
            let h = if mood == Mood::Squeeze { 0.15 } else { 0.12 };
            gfx2d::rrect(-0.12, my - 0.06, 0.24, h * 2.0, h, 0.0, 0.0, ink, ink);
            gfx2d::rrect(-0.07, my - 0.06 + h * 1.15, 0.14, h * 0.7, h * 0.35, 0.0, 0.0, a(0xf07a8eff), a(0xf07a8eff));
        }
        Mood::Huh => shape::capsule(-0.08, my, 0.1, my - 0.03, 0.055, ink),
        Mood::Sleepy(d) => {
            if d > 0.55 {
                // A yawn, slowly opening.
                let o = 0.05 + 0.06 * (d - 0.55) / 0.45;
                oval(0.0, my, o, o * 1.25, ink);
            } else {
                shape::arc(0.0, my - 0.13 + d * 0.06, 0.13, 0.05, PI * (0.74 + d * 0.12), PI * (1.26 - d * 0.12), ink);
            }
        }
        Mood::Asleep => oval(0.0, my, 0.045, 0.055, ink),
    }
}

/// The background: the painted bedroom window. As the poppets get sleepy the
/// lamp's warm glow fades, the room sinks into blue night and stars come out.
pub fn sky(art: &Art, sleep: f32, t: f32) {
    if let Some(img) = art.bg {
        gfx2d::sprite(img, BG_TEXELS, [0.0, 0.0, 360.0, 640.0], 0xffffffff);
        if sleep > 0.005 {
            gfx2d::blend(Blend::Multiply);
            gfx2d::rect(0.0, 0.0, 360.0, 640.0, mix(0xffffffff, NIGHT_TINT, sleep));
            gfx2d::blend(Blend::Alpha);
        }
    } else {
        let top = mix(SKY_AWAKE.0, SKY_ASLEEP.0, sleep);
        let bottom = mix(SKY_AWAKE.1, SKY_ASLEEP.1, sleep);
        gfx2d::rect_gradient(0.0, 0.0, 360.0, 640.0, top, bottom);
    }
    // The bedside lamp, off to the lower left, glowing while they're awake.
    gfx2d::blend(Blend::Add);
    let flicker = 1.0 + 0.04 * (t * 1.7).sin();
    shape::soft_disc(-20.0, 600.0, 300.0 * flicker, 300.0, with_alpha(LAMP, 0.2 * (1.0 - sleep)));
    shape::soft_disc(380.0, 80.0, 160.0, 160.0, with_alpha(0xfff2c8ff, 0.08 * (1.0 - sleep)));
    gfx2d::blend(Blend::Alpha);
    if sleep > 0.05 {
        // Extra stars in the window, twinkling (the moon is painted, top right).
        for i in 0..14 {
            let fi = i as f32;
            let x = 60.0 + (fi * 97.31).rem_euclid(170.0);
            let y = 64.0 + (fi * 53.7 + 11.0).rem_euclid(100.0);
            let tw = 0.5 + 0.5 * (t * 2.0 + fi * 1.3).sin();
            shape::sparkle(x, y, 1.6 + 2.4 * tw, 0.0, with_alpha(0xfff6d0ff, sleep * (0.3 + 0.55 * tw)));
        }
    }
    // A soft shade along the bottom, so the feed's title reads over the windowsill.
    gfx2d::rect_gradient(0.0, 530.0, 360.0, 110.0, with_alpha(INK, 0.0), with_alpha(INK, 0.8));
}

/// A wobbly pencil line around a rounded rect (two light passes).
fn pencil_rrect(r: Rect, radius: f32, color: u32) {
    let pts = shape::rounded_points(&[r.x, r.y, r.right(), r.y, r.right(), r.bottom(), r.x, r.bottom()], radius, 8);
    for pass in 0..2 {
        let k = pass as f32;
        let mut wob: Vec<f32> = pts
            .chunks(2)
            .enumerate()
            .flat_map(|(i, p)| {
                let fi = i as f32;
                [p[0] + 0.9 * (fi * 1.71 + k * 2.0).sin(), p[1] + 0.9 * (fi * 2.13 + k * 1.3).cos()]
            })
            .collect();
        wob.extend_from_slice(&wob[..2].to_vec());
        shape::polyline(&wob, 1.4 - 0.4 * k, fade(color, 0.6 - 0.25 * k));
    }
}

/// The board: an indigo wash on watercolor paper, paler cells, a pencil border.
pub fn board_bg(art: &Art, sleep: f32) {
    let r = Rect::new(BOARD_X - 8.0, BOARD_Y - 8.0, CELL * COLS as f32 + 16.0, CELL * ROWS as f32 + 16.0);
    shape::shadow(r, 22.0, 6.0, 16.0, 0x0a001860);
    shape::rrect(r, 22.0, BOARD_WASH);
    for row in 0..ROWS {
        for c in 0..COLS {
            let cell = Rect::new(BOARD_X + c as f32 * CELL + 2.0, BOARD_Y + row as f32 * CELL + 2.0, CELL - 4.0, CELL - 4.0);
            if (row + c) % 2 == 0 {
                shape::rrect(cell, 12.0, CELL_WASH);
            }
        }
    }
    if let Some(img) = art.paper {
        // Paper grain over the wash: one 64-unit tile per 128-texel paper.
        let inner = r.inset(6.0);
        let mut y = inner.y;
        while y < inner.bottom() {
            let h = (inner.bottom() - y).min(64.0);
            let mut x = inner.x;
            while x < inner.right() {
                let w = (inner.right() - x).min(64.0);
                gfx2d::blend(Blend::Multiply);
                gfx2d::sprite(img, [0.0, 0.0, w * 2.0, h * 2.0], [x, y, w, h], 0xffffffff);
                gfx2d::blend(Blend::Add);
                gfx2d::sprite(img, [0.0, 0.0, w * 2.0, h * 2.0], [x, y, w, h], 0xffffff12);
                gfx2d::blend(Blend::Alpha);
                x += 64.0;
            }
            y += 64.0;
        }
    }
    pencil_rrect(r.inset(2.0), 20.0, with_alpha(PENCIL, 1.0 - sleep * 0.5));
}

pub fn cell_center(c: f32, r: f32) -> (f32, f32) {
    (BOARD_X + (c + 0.5) * CELL, BOARD_Y + (r + 0.5) * CELL)
}

/// The wake meter: a painted capsule that drains, with a moon and a sun at its
/// ends. `shown` trails `energy` (the refill grows in); `flash` 0…1 after a refill.
pub fn meter(cx: f32, cy: f32, w: f32, energy: f32, shown: f32, flash: f32, t: f32) {
    let h = 18.0;
    let r = Rect::centered(cx, cy, w, h);
    shape::rrect(r.inset(-3.0), h, with_alpha(INK, 0.75));
    pencil_rrect(r.inset(-3.5), h / 2.0 + 3.5, with_alpha(PENCIL, 0.7));
    let col = if energy > 0.5 { mix(METER[1], METER[0], (energy - 0.5) * 2.0) } else { mix(METER[2], METER[1], (energy - 0.25).max(0.0) * 4.0) };
    let danger = energy < crate::sim::DROWSY;
    let blink = if danger { 0.55 + 0.45 * (t * 9.0).sin() } else { 1.0 };
    let gain = shown.max(energy);
    if gain > 0.005 {
        shape::rrect(Rect::new(r.x, r.y, (r.w * gain).max(h), h), h, with_alpha(PENCIL, 0.3 + 0.5 * flash));
    }
    let lo = energy.min(shown);
    if lo > 0.005 {
        let fw = (r.w * lo).max(h);
        gfx2d::rrect(r.x, r.y, fw, h, h / 2.0, 0.0, 0.0, fade(lighten(col, 0.2), blink), fade(darken(col, 0.12), blink));
        // A watercolor bloom along the top edge.
        shape::capsule(r.x + 7.0, r.y + 5.0, r.x + fw - 7.0, r.y + 5.0, 3.0, fade(0xfffaf060, blink));
    }
    // Moon at the empty end, sun at the full end.
    let mx = r.x - 17.0;
    shape::disc(mx, cy, 12.0, INK);
    shape::disc(mx, cy, 9.0, 0xfff1c8ff);
    shape::disc(mx + 4.5, cy - 3.0, 7.5, INK);
    let sx = r.right() + 17.0;
    shape::disc(sx, cy, 13.0, INK);
    let spin = t * 0.8;
    for i in 0..8 {
        let a = spin + i as f32 * TAU / 8.0;
        shape::capsule(sx + 6.0 * a.sin(), cy - 6.0 * a.cos(), sx + 10.5 * a.sin(), cy - 10.5 * a.cos(), 2.5, GOLD);
    }
    shape::disc(sx, cy, 6.0, GOLD);
}

/// A special going off, `age` seconds after its own start.
pub fn blast(b: &Blast, age: f32) {
    if age < 0.0 {
        return;
    }
    let (x, y) = cell_center(b.at.0 as f32, b.at.1 as f32);
    let col = b.color.map_or(0xffffffff, |k| PIECES[k as usize]);
    let k = (age / 0.35).clamp(0.0, 1.0);
    if k >= 1.0 && !matches!(b.kind, BlastKind::Zap | BlastKind::Board) {
        return;
    }
    let left = BOARD_X - 6.0;
    let right = BOARD_X + CELL * COLS as f32 + 6.0;
    let top = BOARD_Y - 6.0;
    let bottom = BOARD_Y + CELL * ROWS as f32 + 6.0;
    let beam = |horiz: bool, at: f32| {
        let w = 30.0 * (1.0 - k);
        gfx2d::blend(Blend::Add);
        if horiz {
            shape::capsule(left, at, right, at, w * 1.6, fade(col, 0.5 * (1.0 - k)));
            shape::capsule(left, at, right, at, w * 0.5, fade(PENCIL, 1.0 - k));
        } else {
            shape::capsule(at, top, at, bottom, w * 1.6, fade(col, 0.5 * (1.0 - k)));
            shape::capsule(at, top, at, bottom, w * 0.5, fade(PENCIL, 1.0 - k));
        }
        gfx2d::blend(Blend::Alpha);
    };
    match b.kind {
        BlastKind::Row => beam(true, y),
        BlastKind::Col => beam(false, x),
        BlastKind::Cross(n) => {
            for d in -(n as i32)..=n as i32 {
                beam(true, y + d as f32 * CELL);
                beam(false, x + d as f32 * CELL);
            }
        }
        BlastKind::Area(n) => {
            let rr = (n as f32 + 0.8) * CELL * (0.3 + 0.7 * Ease::QuadOut.at(k));
            gfx2d::blend(Blend::Add);
            shape::soft_disc(x, y, rr, rr * 0.6, fade(0xffe0a0ff, 0.6 * (1.0 - k)));
            gfx2d::blend(Blend::Alpha);
            shape::ring(x, y, rr, 10.0 * (1.0 - k) + 1.0, fade(PENCIL, 1.0 - k));
        }
        BlastKind::Zap => {
            let k = (age / 0.45).clamp(0.0, 1.0);
            if k >= 1.0 {
                return;
            }
            gfx2d::blend(Blend::Add);
            for (i, c) in b.targets.iter().enumerate() {
                let (tx, ty) = cell_center(c.0 as f32, c.1 as f32);
                let (mx, my) = ((x + tx) / 2.0 + 10.0 * ((i as f32) * 2.1 + age * 40.0).sin(), (y + ty) / 2.0 + 10.0 * ((i as f32) * 1.3 + age * 37.0).cos());
                let a = 1.0 - k;
                shape::polyline(&[x, y, mx, my, tx, ty], 5.0 * a + 1.0, fade(col, 0.7 * a));
                shape::polyline(&[x, y, mx, my, tx, ty], 1.6, fade(0xffffffff, a));
            }
            shape::soft_disc(x, y, 40.0, 30.0, fade(0xffffffff, 0.7 * (1.0 - k)));
            gfx2d::blend(Blend::Alpha);
        }
        BlastKind::Board => {
            let k = (age / 0.6).clamp(0.0, 1.0);
            if k >= 1.0 {
                return;
            }
            gfx2d::blend(Blend::Add);
            gfx2d::rect(left, top, right - left, bottom - top, fade(0xffffffff, 0.6 * (1.0 - k)));
            gfx2d::blend(Blend::Alpha);
            shape::ring(x, y, 40.0 + 400.0 * k, 14.0 * (1.0 - k) + 1.0, fade(PENCIL, 1.0 - k));
        }
    }
}

/// The swipe hint: both cells glow and a ghost thumb slides from `a` to `b`.
pub fn swipe_hint(a: (f32, f32), b: (f32, f32), t: f32, strength: f32) {
    let pulse = 0.5 + 0.5 * (t * 5.0).sin();
    for (x, y) in [a, b] {
        gfx2d::blend(Blend::Add);
        shape::soft_disc(x, y, CELL * 0.55, 14.0, fade(0xffe6b0ff, (0.25 + 0.3 * pulse) * strength));
        gfx2d::blend(Blend::Alpha);
    }
    let k = (t * 0.9).fract();
    let move_k = Ease::CubicInOut.at(((k - 0.2) / 0.5).clamp(0.0, 1.0));
    let alpha = strength * if k < 0.1 { k / 0.1 } else if k > 0.85 { (1.0 - k) / 0.15 } else { 1.0 };
    let (fx, fy) = (a.0 + (b.0 - a.0) * move_k, a.1 + (b.1 - a.1) * move_k);
    // An arrow along the way.
    let (dx, dy) = ((b.0 - a.0) / CELL, (b.1 - a.1) / CELL);
    let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let tip = (mx + dx * 9.0, my + dy * 9.0);
    let (px, py) = (-dy, dx);
    shape::poly(&[tip.0, tip.1, mx - dx * 5.0 + px * 9.0, my - dy * 5.0 + py * 9.0, mx - dx * 5.0 - px * 9.0, my - dy * 5.0 - py * 9.0], fade(PENCIL, alpha * 0.9));
    let press = if (0.12..0.8).contains(&k) { 0.85 } else { 1.0 };
    shape::disc(fx + 3.0, fy + 5.0, 17.0 * press, fade(0x0d0a2050, alpha));
    shape::disc(fx, fy, 17.0 * press, fade(0xfff6eee0, alpha));
    shape::ring(fx, fy, 17.0 * press, 3.0, fade(INK, alpha * 0.8));
}

/// "z"s floating up off sleeping poppets.
pub fn zzz(t: f32, alpha: f32) {
    for i in 0..7 {
        let fi = i as f32;
        let k = (t * 0.45 + fi * 0.37).fract();
        let c = (i * 3 + 1) % COLS;
        let r = (i * 5 + 2) % ROWS;
        let (x, y) = cell_center(c as f32, r as f32);
        let a = alpha * if k < 0.2 { k / 0.2 } else { 1.0 - (k - 0.2) / 0.8 };
        let size = 14.0 + 12.0 * k;
        text("z").size(size).color(fade(PENCIL, a)).outline(0.12, fade(INK, a)).middle().draw(x + 12.0 + 14.0 * k + 5.0 * (k * 9.0).sin(), y - 14.0 - 50.0 * k);
    }
}
