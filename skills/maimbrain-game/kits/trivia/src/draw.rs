//! Drawing: the host (Watt, a lightbulb with a face), the stage, the
//! marquee, the question card, the answer buttons, the stopwatch and the
//! streak badge. Watt, the stage, the emblems, the marquee and the starburst
//! are generated art (`art`); cards, buttons, the clock and all text are
//! code-drawn in the same print style (thick ink rims, hard offset shadows,
//! the poster palette), so they stay crisp and remixable. Nothing here
//! changes game state.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use maimbrain::gfx2d::{self, Blend, Image};
use maimbrain::motion::{Ease, Spring};
use maimbrain::ui::screens::bouncy_title;
use maimbrain::ui::widgets::rays;
use maimbrain::ui::{Icon, Layout, Rect, darken, fade, lighten, mix, pill, shape, text, with_alpha};

use crate::art::{self, Frame};
use crate::look::*;
use crate::questions::Question;

// ---- the host ------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Idle,
    /// Reading along with the player.
    Think,
    /// The clock is nearly out: wide eyes, sweat.
    Nervous,
    Happy,
    /// A big moment (golden question, long streak): star eyes.
    Excited,
    Sad,
    /// Blown: dark glass, cracked, X eyes.
    Dead,
}

/// Watt's animation state. Advanced in `update`, drawn in `render`.
#[derive(Clone, Debug)]
pub struct Host {
    pub mood: Mood,
    /// Seconds in this mood.
    pub mood_t: f32,
    /// Seconds the mood holds before falling back to `rest`.
    hold: f32,
    pub rest: Mood,
    /// Squash and stretch (kicked on reactions).
    pub bounce: Spring,
    /// Side-to-side lean (kicked on reactions, wobbles back).
    pub lean: Spring,
    /// 0 dark … 1 fully lit.
    pub lit: f32,
    /// Extra glow from the streak, eased.
    pub glow: f32,
    pub glow_target: f32,
    /// Where the pupils point, −1…1 each way, eased toward `look_target`.
    pub look: (f32, f32),
    pub look_target: (f32, f32),
    /// Seconds left of a flicker (a wrong answer makes the bulb stutter).
    pub flicker: f32,
    pub cracked: bool,
    pub t: f32,
}

impl Host {
    pub fn new() -> Host {
        Host {
            mood: Mood::Idle,
            mood_t: 0.0,
            hold: 0.0,
            rest: Mood::Idle,
            bounce: Spring::new(0.0, 5.0, 0.3),
            lean: Spring::new(0.0, 3.0, 0.35),
            lit: 1.0,
            glow: 0.2,
            glow_target: 0.2,
            look: (0.0, 0.0),
            look_target: (0.0, 0.0),
            flicker: 0.0,
            cracked: false,
            t: 0.0,
        }
    }

    /// Reacts with `mood` for `hold` seconds, then goes back to `rest`.
    pub fn react(&mut self, mood: Mood, hold: f32) {
        self.mood = mood;
        self.mood_t = 0.0;
        self.hold = hold;
        match mood {
            Mood::Happy | Mood::Excited => {
                self.bounce.kick(-5.0);
                self.look_target = (0.0, -0.8);
            }
            Mood::Sad => {
                self.bounce.kick(3.5);
                self.lean.kick(6.0);
                self.flicker = 0.45;
                self.look_target = (0.0, 0.9);
            }
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.t += dt;
        self.mood_t += dt;
        if self.mood != self.rest && self.mood != Mood::Dead && self.hold > 0.0 && self.mood_t > self.hold {
            self.mood = self.rest;
            self.mood_t = 0.0;
            self.hold = 0.0;
        } else if self.hold <= 0.0 && self.mood != Mood::Dead {
            self.mood = self.rest;
        }
        self.bounce.update(dt);
        self.lean.update(dt);
        self.flicker = (self.flicker - dt).max(0.0);
        let k = 1.0 - (-dt * 4.0).exp();
        self.glow += (self.glow_target - self.glow) * k;
        let k = 1.0 - (-dt * 9.0).exp();
        self.look.0 += (self.look_target.0 - self.look.0) * k;
        self.look.1 += (self.look_target.1 - self.look.1) * k;
    }

    /// The brightness this frame (flickers stutter it).
    fn brightness(&self) -> f32 {
        if self.flicker > 0.0 {
            let on = ((self.flicker * 31.0).sin() + (self.flicker * 13.0).cos()) > -0.2;
            self.lit * if on { 1.0 } else { 0.25 }
        } else {
            self.lit
        }
    }

    /// Draws Watt with the glass centred at (cx, cy); `s` = 1 is a glass of
    /// radius 34 (~110 units tall). Uses his generated expression sheet once
    /// it has loaded, the code-drawn fallback before.
    pub fn draw(&self, sheet: Option<Image>, cx: f32, cy: f32, s: f32) {
        let r = 34.0 * s;
        let lit = self.brightness();
        let bob = (self.t * 2.2).sin() * 3.0 * s * if self.mood == Mood::Dead { 0.0 } else { 1.0 };
        let nervous = self.mood == Mood::Nervous;
        let wobble = if nervous { (self.t * 38.0).sin() * 0.03 } else { 0.0 };
        // Behind him: a turning sunburst of rays when he's thrilled, and the glow.
        let thrill = match self.mood {
            Mood::Excited => 1.0,
            Mood::Happy => 0.6,
            _ => 0.0,
        } * (self.mood_t / 0.15).min(1.0);
        if thrill > 0.0 {
            rays(cx, cy + bob, r * 2.3, 14, self.t * 0.6, fade(CREAM, 0.35 * thrill));
        }
        if lit > 0.02 {
            gfx2d::blend(Blend::Add);
            let g = (0.12 + 0.4 * self.glow) * lit;
            shape::soft_disc(cx, cy + bob, r * (1.45 + 0.5 * self.glow), r * 1.1, fade(GLOW, g));
            gfx2d::blend(Blend::Alpha);
        }
        gfx2d::push();
        gfx2d::translate(cx, cy + bob + r * 1.4);
        gfx2d::rotate(self.lean.value * 0.02 + wobble);
        let sq = self.bounce.value * 0.025;
        gfx2d::scale(1.0 - sq, 1.0 + sq);
        gfx2d::translate(0.0, -r * 1.4);
        let frame = match self.mood {
            Mood::Idle => Frame::Idle,
            Mood::Think => Frame::Think,
            Mood::Nervous => Frame::Nervous,
            Mood::Happy => Frame::Happy,
            Mood::Excited => Frame::Star,
            Mood::Sad => Frame::Flicker,
            Mood::Dead => Frame::Cracked,
        };
        // A stuttering bulb dims the whole drawing (ink included, like a light going out).
        let tint = if self.mood == Mood::Dead { 0xffffffff } else { mix(0x8a8a9aff, 0xffffffff, lit) };
        if art::host_frame(sheet, frame, r, tint) {
            // The grey "flicker" glass catches the light again on the stutter's on-beats.
            if self.mood == Mood::Sad && self.flicker > 0.0 && self.brightness() > 0.5 {
                gfx2d::blend(Blend::Add);
                shape::soft_disc(0.0, 0.0, r * 0.9, r * 0.25, fade(GLOW, 0.45));
                gfx2d::blend(Blend::Alpha);
            }
        } else {
            self.body(r, lit);
            self.face(r, lit);
        }
        gfx2d::pop();
    }

    fn body(&self, r: f32, lit: f32) {
        let glass_top = mix(GLASS_OFF, lighten(GLASS_LIT, 0.4), lit);
        let glass_bot = mix(darken(GLASS_OFF, 0.2), GLASS_LIT, lit);
        let neck = |grow: f32| [-0.62 * r - grow, 0.5 * r, 0.62 * r + grow, 0.5 * r, 0.42 * r + grow, 1.12 * r + grow, -0.42 * r - grow, 1.12 * r + grow];
        // Ink outline: the same shapes, a little bigger, underneath.
        let o = 3.0;
        shape::disc(0.0, 0.0, r + o, INK);
        shape::poly(&neck(o), INK);
        // The screw base and its bow tie.
        let bw = 0.9 * r;
        shape::rrect(Rect::new(-bw / 2.0 - o, 1.05 * r, bw + 2.0 * o, 0.72 * r + o), 6.0, INK);
        for i in 0..3 {
            let y = 1.08 * r + i as f32 * 0.2 * r;
            gfx2d::rrect(-bw / 2.0, y, bw, 0.2 * r, 0.1 * r, 0.0, 0.0, lighten(METAL, 0.25), darken(METAL, 0.25));
        }
        shape::rrect(Rect::new(-0.2 * r, 1.68 * r, 0.4 * r, 0.16 * r), 0.08 * r, INK);
        // Glass.
        gfx2d::rrect(-r, -r, 2.0 * r, 2.0 * r, r, 0.0, 0.0, glass_top, glass_bot);
        shape::poly(&neck(0.0), glass_bot);
        // Bow tie at the collar.
        let (bx, by, bs) = (0.0, 1.08 * r, 0.32 * r);
        shape::poly(&[bx, by, bx - bs * 1.3, by - bs * 0.7, bx - bs * 1.3, by + bs * 0.7], INK);
        shape::poly(&[bx, by, bx + bs * 1.3, by - bs * 0.7, bx + bs * 1.3, by + bs * 0.7], INK);
        shape::poly(&[bx, by, bx - bs * 1.1, by - bs * 0.5, bx - bs * 1.1, by + bs * 0.5], BOWTIE);
        shape::poly(&[bx, by, bx + bs * 1.1, by - bs * 0.5, bx + bs * 1.1, by + bs * 0.5], BOWTIE);
        shape::disc(bx, by, bs * 0.38, darken(BOWTIE, 0.2));
        // A shine on the glass.
        shape::soft_disc(-0.45 * r, -0.5 * r, 0.2 * r, 0.12 * r, fade(0xffffffff, 0.35 + 0.4 * lit));
        shape::capsule(0.52 * r, -0.62 * r, 0.66 * r, -0.36 * r, 0.09 * r, fade(0xffffffff, 0.25 + 0.3 * lit));
        if self.cracked {
            let c = fade(INK, 0.85);
            shape::polyline(&[-0.2 * r, -0.98 * r, -0.05 * r, -0.62 * r, -0.32 * r, -0.38 * r, -0.1 * r, -0.08 * r], 2.2, c);
            shape::polyline(&[-0.05 * r, -0.62 * r, 0.3 * r, -0.55 * r, 0.42 * r, -0.8 * r], 2.0, c);
        }
    }

    fn face(&self, r: f32, lit: f32) {
        let ink = mix(darken(INK, 0.1), INK, lit);
        let ey = -0.1 * r;
        let ex = 0.37 * r;
        let (lx, ly) = self.look;
        // Blinks: two quick ones now and then (a function of time, so it replays).
        let ph = (self.t * 0.31).fract();
        let blink = ph < 0.025 || (ph > 0.06 && ph < 0.085);
        match self.mood {
            Mood::Dead => {
                for sx in [-1.0, 1.0] {
                    let (x, y, d) = (sx * ex, ey, 0.13 * r);
                    shape::capsule(x - d, y - d, x + d, y + d, 0.08 * r, ink);
                    shape::capsule(x - d, y + d, x + d, y - d, 0.08 * r, ink);
                }
            }
            Mood::Happy => {
                for sx in [-1.0, 1.0] {
                    shape::arc(sx * ex, ey + 0.06 * r, 0.15 * r, 0.075 * r, -1.25, 1.25, ink);
                }
            }
            Mood::Excited => {
                for sx in [-1.0, 1.0] {
                    shape::star(sx * ex, ey, 0.2 * r, self.t * 2.0, darken(GLOW, 0.15));
                    shape::star(sx * ex, ey, 0.13 * r, self.t * 2.0, lighten(GLOW, 0.5));
                }
            }
            _ => {
                let wide = if self.mood == Mood::Nervous { 1.18 } else { 1.0 };
                let pr = if self.mood == Mood::Nervous { 0.07 } else { 0.1 } * r;
                for sx in [-1.0, 1.0] {
                    let (x, y) = (sx * ex, ey);
                    if blink {
                        shape::capsule(x - 0.13 * r, y, x + 0.13 * r, y, 0.06 * r, ink);
                    } else {
                        shape::disc(x, y, 0.19 * r * wide, 0xffffffff);
                        shape::ring(x, y, 0.19 * r * wide, 0.035 * r, fade(ink, 0.5));
                        shape::disc(x + lx * 0.07 * r, y + ly * 0.07 * r, pr, ink);
                        shape::disc(x + lx * 0.07 * r - pr * 0.35, y + ly * 0.07 * r - pr * 0.35, pr * 0.32, 0xffffffff);
                    }
                }
            }
        }
        // Brows: their tilt is most of the expression.
        let (tilt, lift) = match self.mood {
            Mood::Sad => (-0.35, 0.0),
            Mood::Nervous => (-0.25, 0.06),
            Mood::Think => (0.12, 0.02),
            Mood::Happy | Mood::Excited => (0.0, 0.08),
            Mood::Dead => (0.0, -0.4),
            Mood::Idle => (0.05, 0.0),
        };
        if self.mood != Mood::Dead {
            for sx in [-1.0f32, 1.0] {
                let (x, y) = (sx * ex, ey - 0.36 * r - lift * r);
                // tilt > 0 pulls the inner ends down (focus), < 0 up (worry).
                let (dx, d) = (0.14 * r, tilt * 0.14 * r);
                shape::capsule(x - dx, y + sx * d, x + dx, y - sx * d, 0.065 * r, ink);
            }
        }
        // Cheeks when pleased.
        if matches!(self.mood, Mood::Happy | Mood::Excited | Mood::Idle) {
            let a = if self.mood == Mood::Idle { 0.25 } else { 0.55 };
            for sx in [-1.0, 1.0] {
                shape::soft_disc(sx * 0.58 * r, 0.2 * r, 0.12 * r, 0.08 * r, fade(BOWTIE, a * lit.max(0.3)));
            }
        }
        // Mouth.
        let my = 0.3 * r;
        match self.mood {
            Mood::Happy | Mood::Excited => {
                // An open grin: a half disc with a tongue.
                let mut pts = Vec::with_capacity(36);
                for k in 0..=16 {
                    let a = PI * k as f32 / 16.0;
                    pts.extend([0.3 * r * a.cos(), my - 0.04 * r + 0.28 * r * a.sin()]);
                }
                shape::poly(&pts, ink);
                shape::disc(0.0, my + 0.15 * r, 0.12 * r, BOWTIE);
            }
            Mood::Sad => shape::arc(0.0, my + 0.22 * r, 0.2 * r, 0.065 * r, -1.0, 1.0, ink),
            Mood::Nervous => {
                let w = 0.26 * r;
                let mut pts = Vec::new();
                for k in 0..=6 {
                    let x = -w + 2.0 * w * k as f32 / 6.0;
                    pts.extend([x, my + if k % 2 == 0 { 0.0 } else { 0.06 * r }]);
                }
                shape::polyline(&pts, 0.05 * r, ink);
                // A sweat drop sliding down the glass.
                let k = (self.mood_t * 0.9).fract();
                let (sx, sy) = (0.74 * r, -0.55 * r + k * 0.5 * r);
                shape::disc(sx, sy, 0.1 * r, 0x8fd8ffff);
                shape::poly(&[sx - 0.087 * r, sy - 0.05 * r, sx, sy - 0.24 * r, sx + 0.087 * r, sy - 0.05 * r], 0x8fd8ffff);
            }
            Mood::Think => shape::disc(0.12 * r, my + 0.02 * r, 0.07 * r, ink),
            Mood::Dead => {
                let mut pts = Vec::new();
                for k in 0..=8 {
                    let x = -0.24 * r + 0.48 * r * k as f32 / 8.0;
                    pts.extend([x, my + 0.05 * r + (k as f32 * 1.6).sin() * 0.04 * r]);
                }
                shape::polyline(&pts, 0.05 * r, ink);
            }
            Mood::Idle => shape::arc(0.0, my - 0.12 * r, 0.24 * r, 0.06 * r, PI - 0.95, PI + 0.95, ink),
        }
    }
}


// ---- print helpers -----------------------------------------------------------

/// A printed panel: a hard offset shadow, a thick ink rim, a paper face
/// (`top` → `bottom`) and a thin inner rule, like a card on a 1950s set.
fn print_panel(r: Rect, radius: f32, top: u32, bottom: u32, alpha: f32) {
    let f = |c: u32| fade(c, alpha);
    shape::rrect(r.offset(DROP.0, DROP.1).inset(-3.0), radius + 3.0, f(with_alpha(INK, 0.55)));
    shape::rrect(r.inset(-3.0), radius + 3.0, f(INK));
    shape::rrect_gradient(r, radius, f(top), f(bottom));
}

/// The inner rule a printed card has, `inset` inside its edge.
fn inner_rule(r: Rect, radius: f32, inset: f32, color: u32) {
    shape::rrect_stroke(r.inset(inset), (radius - inset).max(2.0), 1.5, color);
}

// ---- the stage ---------------------------------------------------------------

/// The studio: the generated stage (curtains, teal wall, starbursts), cover-
/// fitted to the screen, with two warm spotlights that sway. `heat` (0…1)
/// brightens them with the streak; `alarm` (0…1) reddens the edges.
pub fn backdrop(l: &Layout, stage: Option<Image>, t: f32, heat: f32, alarm: f32) {
    let s = l.screen;
    match stage {
        Some(img) => {
            // Cover: scale to fill, centred, cropping whichever side is long.
            let k = (s.w / 360.0).max(s.h / 640.0);
            let (w, h) = (360.0 * k, 640.0 * k);
            art::whole(img, (720.0, 1280.0), Rect::centered(s.cx(), s.cy(), w, h), 0xffffffff);
        }
        None => shape::gradient(s, BG_TOP, BG_BOTTOM),
    }
    gfx2d::blend(Blend::Add);
    for (i, side) in [-1.0f32, 1.0].iter().enumerate() {
        let sway = (t * 0.5 + i as f32 * 2.1).sin() * 0.16;
        let (ox, oy) = (s.cx() + side * s.w * 0.45, s.y - 20.0);
        let ang = side * -0.42 + sway + FRAC_PI_2;
        let len = s.h * 1.05;
        let spread = 0.15;
        let p = |a: f32| [ox + len * a.cos(), oy + len * a.sin()];
        let (a, b) = (p(ang - spread), p(ang + spread));
        shape::poly(&[ox, oy, a[0], a[1], b[0], b[1]], fade(0xffe9a8ff, 0.05 + 0.1 * heat));
    }
    gfx2d::blend(Blend::Alpha);
    if alarm > 0.0 {
        shape::vignette(s, 120.0, alarm * 0.55, WRONG);
    }
}

/// The marquee sign with `lines` lettered on its panel, centred at (cx, y
/// top), `w` units wide. Its bulbs shimmer with a light sweeping across.
/// Returns the sign's rect. Code-drawn letters (crisp, and a reskin is a
/// string change); before the image loads, a code-drawn sign stands in.
pub fn marquee(img: Option<Image>, lines: &[&str], cx: f32, y: f32, w: f32, t: f32, theme: &maimbrain::ui::Theme) -> Rect {
    let k = w / art::MARQUEE.0;
    let sign = Rect::new(cx - w / 2.0, y, w, art::MARQUEE.1 * k);
    let [px, py, pw, ph] = art::MARQUEE_PANEL;
    let panel = Rect::new(sign.x + px * k, sign.y + py * k, pw * k, ph * k);
    // A gentle swing on its chains.
    gfx2d::push();
    gfx2d::translate(sign.cx(), sign.y);
    gfx2d::rotate((t * 1.3).sin() * 0.012);
    gfx2d::translate(-sign.cx(), -sign.y);
    match img {
        Some(img) => {
            art::whole(img, art::MARQUEE, sign, 0xffffffff);
            // A band of light sweeping across the bulbs every few seconds.
            let ph = (t * 0.35).fract() * 1.6 - 0.3;
            gfx2d::blend(Blend::Add);
            for side in [sign.y + sign.h * 0.12, sign.bottom() - sign.h * 0.12] {
                shape::soft_disc(sign.x + sign.w * ph, side, sign.w * 0.07, sign.h * 0.06, fade(CREAM, 0.28));
            }
            gfx2d::blend(Blend::Alpha);
        }
        None => {
            shape::rrect(sign.inset(-3.0), 26.0, INK);
            shape::rrect(sign, 23.0, TOMATO);
            shape::rrect(panel.inset(-3.0), 15.0, INK);
            shape::rrect(panel, 12.0, TEAL);
        }
    }
    // The lettering: cream, ink-outlined, with a hard offset print shadow.
    let n = lines.len().max(1) as f32;
    let size = (panel.h / n) * 0.92;
    for (i, line) in lines.iter().enumerate() {
        let ly = panel.y + panel.h * (i as f32 + 0.5) / n + 2.0;
        bouncy_title(line, panel.cx(), ly, size, t - i as f32 * 0.25, theme, ON_STAGE, panel.w - 16.0);
    }
    gfx2d::pop();
    sign
}

// ---- the card ---------------------------------------------------------------

/// The largest of `sizes` at which `s` wraps into `max_lines` within `w`.
pub fn fit(s: &str, w: f32, max_lines: usize, sizes: &[f32]) -> f32 {
    for &sz in sizes {
        if text(s).size(sz).wrap(w).lines().len() <= max_lines {
            return sz;
        }
    }
    *sizes.last().unwrap_or(&16.0)
}

const CARD_RADIUS: f32 = 12.0;

/// The question card. `tag` goes top-right (the question number); `flip`
/// 0…1 is the card turning (edge-on at 0). Golden cards sit on a turning
/// starburst.
#[allow(clippy::too_many_arguments)]
pub fn card(r: Rect, q: &Question, tag: &str, golden: bool, flip: f32, t: f32, cats: Option<Image>, burst: Option<Image>) {
    if flip <= 0.01 {
        return;
    }
    gfx2d::push();
    gfx2d::translate(r.cx(), r.cy());
    gfx2d::scale(flip, 1.0);
    gfx2d::translate(-r.cx(), -r.cy());
    if golden {
        // Behind the stopwatch's spot (top right), rays out around it.
        starburst(burst, r.right() - 40.0, r.y - 50.0, 112.0, t * 0.5);
    }
    let (top, bot) = if golden { (GOLD_TOP, GOLD_BOTTOM) } else { (PAPER, PAPER_SHADE) };
    print_panel(r, CARD_RADIUS, top, bot, 1.0);
    inner_rule(r, CARD_RADIUS, 6.0, with_alpha(INK, if golden { 0.3 } else { 0.18 }));
    if golden {
        for i in 0..6 {
            let a = t * 0.8 + i as f32 * TAU / 6.0;
            let (x, y) = (r.cx() + a.cos() * (r.w * 0.5 + 2.0), r.cy() + a.sin() * (r.h * 0.5 + 2.0));
            shape::sparkle(x, y, 5.0 + 3.0 * (t * 3.0 + i as f32).sin().abs(), a, 0xfff4dce0);
        }
    }
    // Category: an emblem medallion on the top-left corner and its name beside it.
    let (cell, col) = category(q.cat);
    let pw = maimbrain::ui::pill_width(q.cat, 24.0, gfx2d::Font::SansBold, false);
    let es = 42.0;
    // Centred a little right of the middle: Watt stands at the top-left corner.
    let ex = r.cx() + 24.0 - (pw + es) / 2.0;
    let ey = r.y + 1.0;
    let (lx, ly) = (ex + 14.0 + pw / 2.0 + 4.0, r.y);
    shape::rrect(Rect::centered(lx + 2.0, ly + 3.0, pw + 22.0, 29.0), 14.5, with_alpha(INK, 0.55));
    shape::rrect(Rect::centered(lx, ly, pw + 22.0, 29.0), 14.5, INK);
    pill(q.cat, lx + 4.0, ly, 24.0, col, ON_STAGE, gfx2d::Font::SansBold, None);
    let drawn = cell.is_some_and(|c| art::emblem(cats, c, ex, ey, es));
    if !drawn {
        question_emblem(ex, ey, es);
    }
    let tag_col = if golden { hex_rgb(0x7a4200) } else { fade(INK, 0.6) };
    text(tag).size(14.0).color(tag_col).right().draw(r.right() - 12.0, r.y + 22.0);
    // The question: big on two lines, a little smaller when it needs three.
    let w = r.w - 32.0;
    let two = fit(q.text, w, 2, &[24.0, 22.0]);
    let size = if text(q.text).size(two).wrap(w).lines().len() <= 2 { two } else { fit(q.text, w, 3, &[21.0, 19.0, 17.0, 16.0]) };
    let body = Rect::new(r.x, r.y + 26.0, r.w, r.h - 34.0);
    let block = text(q.text).size(size).color(INK).wrap(w).center().leading(1.02);
    block.draw(body.cx(), body.cy() - block.measure().1 / 2.0);
    gfx2d::pop();
}

/// The card's back: the right answer, big, on green (the title card's reveal).
pub fn card_back(r: Rect, answer: &str, flip: f32) {
    if flip <= 0.01 {
        return;
    }
    gfx2d::push();
    gfx2d::translate(r.cx(), r.cy());
    gfx2d::scale(flip, 1.0);
    gfx2d::translate(-r.cx(), -r.cy());
    print_panel(r, CARD_RADIUS, lighten(RIGHT, 0.12), darken(RIGHT, 0.08), 1.0);
    inner_rule(r, CARD_RADIUS, 6.0, with_alpha(ON_STAGE, 0.45));
    shape::disc(r.cx(), r.y, 19.0, INK);
    shape::disc(r.cx(), r.y, 16.0, MUSTARD);
    Icon::CHECK.draw(r.cx(), r.y, 20.0, INK);
    let w = r.w - 32.0;
    let size = fit(answer, w, 2, &[34.0, 30.0, 26.0, 22.0]);
    let block = text(answer).size(size).color(ON_STAGE).outline(0.1, INK).shadow(0.0, size * 0.08, with_alpha(INK, 0.8)).wrap(w).center();
    block.draw(r.cx(), r.cy() + 6.0 - block.measure().1 / 2.0);
    gfx2d::pop();
}

/// The generated starburst (a code-drawn star before it loads), `size`
/// across, turned by `rot`.
pub fn starburst(img: Option<Image>, cx: f32, cy: f32, size: f32, rot: f32) {
    gfx2d::push();
    gfx2d::translate(cx, cy);
    gfx2d::rotate(rot);
    match img {
        Some(img) => art::whole(img, (art::BURST, art::BURST), Rect::centered(0.0, 0.0, size, size), 0xffffffff),
        None => {
            let pts = shape::star_points(0.0, 0.0, size * 0.5, size * 0.33, 12, 0.0);
            shape::poly(&pts, INK);
            let pts = shape::star_points(0.0, 0.0, size * 0.46, size * 0.3, 12, 0.0);
            shape::poly(&pts, MUSTARD);
        }
    }
    gfx2d::pop();
}

const fn hex_rgb(rgb: u32) -> u32 {
    (rgb << 8) | 0xff
}

// ---- answers ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ans {
    /// Live (or waiting to be), plain.
    Plain,
    /// The player's right answer.
    Right,
    /// The player's wrong answer.
    Wrong,
    /// The right answer, revealed after a miss.
    Revealed,
    /// Not involved in the reveal.
    Dim,
}

/// One answer button: a paper tile with a thick ink rim on an ink base (a
/// contestant's push button). `k` 0…1: how far into its reveal; `scale` its
/// press punch; `dx` a slide or shake; `alpha` fades it in.
#[allow(clippy::too_many_arguments)]
pub fn answer(r: Rect, slot: usize, label: &str, st: Ans, k: f32, scale: f32, dx: f32, alpha: f32) {
    if alpha <= 0.01 {
        return;
    }
    let f = |c: u32| fade(c, alpha);
    let r = r.offset(dx, 0.0);
    gfx2d::push();
    gfx2d::translate(r.cx(), r.cy());
    gfx2d::scale(scale, scale);
    gfx2d::translate(-r.cx(), -r.cy());
    let pulse = 0.55 + 0.45 * (k * TAU * 2.0).cos().abs();
    let (top, bot, ink) = match st {
        Ans::Right => (lighten(RIGHT, 0.1), darken(RIGHT, 0.08), ON_STAGE),
        Ans::Wrong => (lighten(WRONG, 0.08), darken(WRONG, 0.1), ON_STAGE),
        Ans::Revealed => (mix(PAPER, lighten(RIGHT, 0.35), pulse), mix(PAPER_SHADE, lighten(RIGHT, 0.15), pulse), INK),
        Ans::Dim => (mix(PAPER, 0x8a8078ff, 0.35), mix(PAPER_SHADE, 0x8a8078ff, 0.4), with_alpha(INK, 0.45)),
        _ => (PAPER, PAPER_SHADE, INK),
    };
    let g = |c: u32| f(c);
    let rad = 12.0;
    let base = 5.0;
    shape::rrect(r.offset(DROP.0, base + 3.0).inset(-2.5), rad + 2.5, g(with_alpha(INK, 0.45)));
    shape::rrect(r.offset(0.0, base).inset(-3.0), rad + 3.0, g(INK));
    shape::rrect(r.inset(-3.0), rad + 3.0, g(INK));
    shape::rrect_gradient(r, rad, g(top), g(bot));
    if st == Ans::Revealed {
        shape::rrect_stroke(r.inset(-6.0), rad + 6.0, 3.0, f(fade(RIGHT, pulse)));
    }
    // The letter badge (a check or a cross once judged).
    let (bx, by) = (r.x + 25.0, r.cy());
    let badge = match st {
        Ans::Dim => mix(BADGE, 0x8a8078ff, 0.6),
        Ans::Right => darken(RIGHT, 0.35),
        Ans::Wrong => darken(WRONG, 0.35),
        Ans::Revealed => RIGHT,
        _ => BADGE,
    };
    shape::disc(bx, by, 16.5, g(INK));
    shape::disc(bx, by, 14.0, g(badge));
    match st {
        Ans::Right | Ans::Revealed => Icon::CHECK.draw(bx, by, 18.0, g(ON_STAGE)),
        Ans::Wrong => Icon::CLOSE.draw(bx, by, 16.0, g(ON_STAGE)),
        _ => {
            text(["A", "B", "C", "D"][slot.min(3)]).size(17.0).color(g(ON_STAGE)).middle().draw(bx, by + 0.5);
        }
    }
    let w = r.w - 62.0;
    let size = fit(label, w, 1, &[20.0, 18.0, 17.0]);
    let size = if text(label).size(size).wrap(w).lines().len() > 1 { 16.0 } else { size };
    text(label).size(size).color(g(ink)).wrap(w).leading(0.95).valign(maimbrain::ui::VAlign::Middle).draw(r.x + 50.0, r.cy() - (text(label).size(size).wrap(w).lines().len() as f32 - 1.0) * size * 0.58);
    gfx2d::pop();
}

// ---- clock and streak ------------------------------------------------------------

/// The countdown: a studio stopwatch whose ring drains clockwise, teal →
/// mustard → tomato, and pulses on the last ticks. `pulse` 0…1.
pub fn timer(cx: f32, cy: f32, left: f32, limit: f32, pulse: f32, alpha: f32) {
    let frac = if limit > 0.0 { (left / limit).clamp(0.0, 1.0) } else { 0.0 };
    let col = if frac > 0.5 { mix(MUSTARD, TEAL, (frac - 0.5) * 2.0) } else { mix(TOMATO, MUSTARD, frac * 2.0) };
    let s = 1.0 + 0.22 * pulse;
    gfx2d::push();
    gfx2d::translate(cx, cy);
    gfx2d::scale(s, s);
    let f = |c: u32| fade(c, alpha);
    // The crown and its button, then the case with a hard shadow.
    shape::rrect(Rect::centered(0.0, -37.0, 14.0, 10.0), 3.0, f(INK));
    shape::rrect(Rect::centered(0.0, -37.0, 9.0, 6.0), 2.0, f(METAL));
    shape::capsule(20.0, -27.0, 26.0, -33.0, 7.0, f(INK));
    shape::disc(DROP.0 * 0.7, DROP.1 * 0.7, 33.0, f(with_alpha(INK, 0.55)));
    shape::disc(0.0, 0.0, 33.0, f(INK));
    shape::disc(0.0, 0.0, 29.5, f(PAPER));
    maimbrain::ui::ring_meter(0.0, 0.0, 25.0, 7.0, frac, f(col), f(with_alpha(INK, 0.12)));
    let n = left.ceil().max(0.0) as u32;
    text(&n.to_string()).size(23.0).color(f(if frac < 0.3 { WRONG } else { INK })).middle().draw(0.0, 1.0);
    gfx2d::pop();
}

/// The streak: a starburst with the multiplier once it's hot, and pips
/// toward the next one.
#[allow(clippy::too_many_arguments)]
pub fn streak(burst: Option<Image>, cx: f32, cy: f32, streak: u32, mult: u32, next_at: Option<u32>, prev_at: u32, scale: f32, t: f32) {
    gfx2d::push();
    gfx2d::translate(cx, cy);
    gfx2d::scale(scale, scale);
    let hot = mult >= 2;
    let label = format!("×{mult}");
    if hot {
        let size = 72.0 + 8.0 * (mult - 2).min(2) as f32 + 3.0 * (t * 6.0).sin();
        starburst(burst, 0.0, -3.0, size, t * 0.8);
        shape::disc(0.0, -3.0, 20.0, INK);
        shape::disc(0.0, -3.0, 17.5, if mult >= 4 { WRONG } else { TOMATO });
        text(&label).size(22.0).color(ON_STAGE).middle().draw(0.0, -3.0);
    } else {
        shape::disc(DROP.0 * 0.5, -3.0 + DROP.1 * 0.5, 21.0, with_alpha(INK, 0.55));
        shape::disc(0.0, -3.0, 21.0, INK);
        shape::disc(0.0, -3.0, 18.5, PAPER);
        text(&label).size(20.0).color(with_alpha(INK, 0.7)).middle().draw(0.0, -3.0);
    }
    // Pips: progress to the next multiplier.
    if let Some(next) = next_at {
        let n = next - prev_at;
        let have = streak.saturating_sub(prev_at).min(n);
        let w = 12.0;
        let x0 = -(n as f32 - 1.0) * w / 2.0;
        for i in 0..n {
            let on = i < have;
            let x = x0 + i as f32 * w;
            shape::disc(x, 30.0, 5.5, INK);
            shape::disc(x, 30.0, 3.8, if on { MUSTARD } else { PAPER_SHADE });
        }
    }
    gfx2d::pop();
}

/// A white-gloved finger tapping (the in-play hint), its tip at (x, y).
/// `press` 0…1.
pub fn finger(x: f32, y: f32, press: f32, alpha: f32) {
    let f = |c: u32| fade(c, alpha);
    let y = y + press * 6.0;
    // A ring where it taps.
    if press > 0.6 {
        let k = (press - 0.6) / 0.4;
        shape::ring(x, y - 2.0, 10.0 + 16.0 * k, 3.0, f(fade(INK, 1.0 - k)));
    }
    gfx2d::push();
    gfx2d::translate(x, y);
    gfx2d::rotate(-0.35);
    let glove = 0xfffaf0ff;
    shape::capsule(0.0, 4.0, 0.0, 40.0, 19.0, f(INK));
    shape::rrect(Rect::new(-17.0, 30.0, 34.0, 40.0), 14.0, f(INK));
    shape::capsule(0.0, 4.0, 0.0, 40.0, 14.0, f(glove));
    shape::rrect(Rect::new(-14.5, 32.5, 29.0, 35.0), 12.0, f(glove));
    // The glove's stitching: two lines down the back.
    shape::capsule(-5.0, 46.0, -5.0, 60.0, 2.0, f(with_alpha(INK, 0.5)));
    shape::capsule(5.0, 46.0, 5.0, 60.0, 2.0, f(with_alpha(INK, 0.5)));
    // The cuff.
    shape::rrect(Rect::new(-19.0, 62.0, 38.0, 12.0), 5.0, f(INK));
    shape::rrect(Rect::new(-16.5, 64.5, 33.0, 7.0), 3.5, f(glove));
    gfx2d::pop();
}

/// Smoke from a blown bulb: one puff, by age.
pub fn puff(x: f32, y: f32, age: f32, life: f32, size: f32) {
    let k = (age / life).clamp(0.0, 1.0);
    let a = (1.0 - k) * 0.6;
    let r = size * (0.5 + Ease::QuadOut.at(k));
    shape::soft_disc(x, y, r, r * 0.8, fade(0x5a504aff, a));
}
