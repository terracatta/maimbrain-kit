//! Drawing the world: a cut-paper meadow, the plate on its stick, snacks
//! with faces, the paper bird, the landing ghost, wind. The pictures are
//! generated art (`assets/sprites.png`, `bird.png`, `faces.png`, `paper.png`,
//! see tools/regen.sh); motion, shadows, the sky's colors and the effects are
//! code. Reads the sim, changes nothing. Colors and names come from `look.rs`.

use maimbrain::gfx2d::{self, Blend, Image};
use maimbrain::physics2d::{Vec2, vec2};
use maimbrain::sys::{Asset, AssetState};
use maimbrain::ui::{Rect, mix, shape, with_alpha};

use crate::look::{self, INK, WHITE};
use crate::sim::{GROUND_Y, Kind, PLATE_H, PLATE_TOP, PLATE_W, Piece, Sim, State, Twist, TwistKind, W};
use crate::sprites as spr;

/// How a face looks.
#[derive(Clone, Copy, PartialEq)]
pub enum Mood {
    /// On the bird: can't wait.
    Excited,
    /// In the air.
    Scared,
    Happy,
    /// The tower sways or it was hit.
    Worried,
    /// Settled deep in the tower: dozing.
    Sleepy,
    /// Fell off.
    Dizzy,
}

// ---- The pictures ---------------------------------------------------------

const SHEET: usize = 0;
const BIRD: usize = 1;
const FACES: usize = 2;
const PAPER: usize = 3;
const FILES: [&str; 4] = ["assets/sprites.png", "assets/bird.png", "assets/faces.png", "assets/paper.png"];
/// `bird.png`: four flapping frames, each cell this many texels (feet at the bottom).
const BIRD_CELL: (f32, f32) = (200.0, 140.0);
const BIRD_FRAMES: usize = 4;
/// `faces.png`: eight expressions in a row (see `face_frame`), cells this big.
const FACE_CELL: (f32, f32) = (96.0, 64.0);
/// `paper.png`: one seamless tile.
const PAPER_SRC: [f32; 4] = [0.0, 0.0, 128.0, 128.0];

/// The loaded images (they decode in the background; until then nothing draws).
pub struct Art {
    assets: [Asset; 4],
    img: [Option<Image>; 4],
}

impl Art {
    pub fn new() -> Art {
        Art { assets: FILES.map(Asset::load), img: [None; 4] }
    }

    pub fn update(&mut self) {
        for i in 0..FILES.len() {
            if self.img[i].is_none() && self.assets[i].state() == AssetState::Ready {
                self.img[i] = Image::new(self.assets[i]);
            }
        }
    }

    /// `src` texels of image `which` into `dst` ([x, y, w, h]), tinted.
    fn draw(&self, which: usize, src: [f32; 4], dst: [f32; 4], tint: u32) {
        if let Some(img) = self.img[which] {
            gfx2d::sprite(img, src, dst, tint);
        }
    }

    /// A piece of the atlas with a soft paper drop shadow (`off`: which way
    /// the shadow falls, in the current frame).
    fn paper(&self, src: [f32; 4], dst: [f32; 4], tint: u32, off: Vec2) {
        let [x, y, w, h] = dst;
        let a = (tint & 0xff) as f32 / 255.0;
        for (k, sa) in [(0.55, 0.16), (1.0, 0.11)] {
            self.draw(SHEET, src, [x + off.x * k, y + off.y * k, w, h], with_alpha(look::SHADOW, sa * a));
        }
        self.draw(SHEET, src, [x, y, w, h], tint);
    }
}

/// Where paper shadows fall (down and a little right, like the sun at noon).
pub const SHADOW_OFF: Vec2 = vec2(1.6, 3.2);

// ---- The sky ------------------------------------------------------------------

/// The sky behind everything (screen space): the color of the hour, the sun,
/// paper clouds, stars at night, all over a sheet of paper.
pub fn sky(art: &Art, sim: &Sim, time: f32) {
    let (top, bottom) = sky_colors(sim.height);
    gfx2d::rect_gradient(0.0, 0.0, W, 640.0, top, bottom);
    let night = ((sim.height - 1100.0) / 500.0).clamp(0.0, 1.0);
    if night > 0.0 {
        for i in 0..36u32 {
            let x = (i.wrapping_mul(97) % 360) as f32;
            let y = ((i.wrapping_mul(53) % 600) as f32 - sim.cam_y * 0.04).rem_euclid(640.0);
            let tw = 0.6 + 0.4 * (time * 2.0 + i as f32).sin();
            shape::disc(x, y, 1.0 + (i % 3) as f32 * 0.5, with_alpha(look::STAR, night * tw));
        }
    }
    let day = 1.0 - ((sim.height - 900.0) / 600.0).clamp(0.0, 1.0);
    if day > 0.0 {
        // The sun sinks a little and turns slowly.
        let (sx, sy) = (296.0, 84.0 - sim.cam_y * 0.06 + (1.0 - day) * 60.0);
        gfx2d::push();
        gfx2d::translate(sx, sy);
        gfx2d::rotate(time * 0.08);
        art.draw(SHEET, spr::SUN, [-38.0, -38.0, 76.0, 76.0], with_alpha(WHITE, day));
        gfx2d::pop();
    }
    let cloud = with_alpha(WHITE, 0.35 + 0.65 * day);
    for i in 0..4 {
        let fi = i as f32;
        let (src, w) = if i % 2 == 0 { (spr::CLOUD_0, 84.0 + fi * 6.0) } else { (spr::CLOUD_1, 112.0 - fi * 4.0) };
        let h = w * src[3] / src[2];
        let x = ((fi * 131.0 + time * (5.0 + fi * 2.0)) % 560.0) - 140.0;
        let y = (60.0 + fi * 150.0 - sim.cam_y * 0.25).rem_euclid(760.0) - 70.0;
        art.paper(src, [x, y, w, h], cloud, SHADOW_OFF);
    }
    // Paper grain over the whole sky (multiplied, so it only darkens the fibers).
    gfx2d::blend(Blend::Multiply);
    let t = 112.0;
    let oy = (-sim.cam_y * 0.25).rem_euclid(t);
    for j in -1..6 {
        for i in 0..4 {
            art.draw(PAPER, PAPER_SRC, [i as f32 * t, j as f32 * t + oy, t, t], WHITE);
        }
    }
    gfx2d::blend(Blend::Alpha);
}

fn sky_colors(height: f32) -> (u32, u32) {
    let s = look::SKY;
    for w in s.windows(2) {
        if height <= w[1].0 {
            let t = (height - w[0].0) / (w[1].0 - w[0].0);
            return (mix(w[0].1, w[1].1, t), mix(w[0].2, w[1].2, t));
        }
    }
    (s[3].1, s[3].2)
}

// ---- The world ----------------------------------------------------------------

/// The world, in world coordinates (call inside the camera transform).
pub fn world(art: &Art, sim: &Sim, time: f32, ghost: bool) {
    ground(art, sim);
    for p in sim.pieces.iter().filter(|p| matches!(p.state, State::Lost { .. })) {
        piece(art, sim, p, time);
    }
    pole_and_plate(art, sim);
    for p in sim.pieces.iter().filter(|p| !matches!(p.state, State::Lost { .. })) {
        piece(art, sim, p, time);
    }
    if ghost {
        landing_ghost(sim, time);
    }
    bird(art, sim, time);
    wind(art, sim, time);
}

fn ground(art: &Art, sim: &Sim) {
    // Far hills scroll a little slower than the world (parallax), with a
    // field below them so no sky shows through as they rise.
    let hy = 474.0 + sim.cam_y * 0.3;
    let hw = 392.0;
    let hh = hw * spr::HILLS[3] / spr::HILLS[2];
    gfx2d::rect(-20.0, hy + hh - 4.0, W + 40.0, 400.0, look::FIELD);
    art.draw(SHEET, spr::HILLS, [(W - hw) / 2.0, hy, hw, hh], WHITE);
    // The near meadow and the blanket the stick stands on (fallen snacks land on it).
    gfx2d::rect(-20.0, GROUND_Y + 30.0, W + 40.0, 400.0, look::FIELD_NEAR);
    let mw = 392.0;
    let mh = mw * spr::MEADOW[3] / spr::MEADOW[2];
    art.draw(SHEET, spr::MEADOW, [(W - mw) / 2.0, GROUND_Y - 14.0, mw, mh], WHITE);
    let bw = 236.0;
    let bh = bw * spr::BLANKET[3] / spr::BLANKET[2];
    art.paper(spr::BLANKET, [W / 2.0 - bw / 2.0, GROUND_Y - 30.0, bw, bh], WHITE, SHADOW_OFF);
}

fn pole_and_plate(art: &Art, sim: &Sim) {
    let cx = W / 2.0;
    let pole_top = PLATE_TOP + PLATE_H;
    art.paper(spr::POLE, [cx - 7.0, pole_top - 4.0, 14.0, GROUND_Y - pole_top + 10.0], WHITE, SHADOW_OFF);
    let angle = sim.plate_angle;
    sim.world.with_transform(sim.plate, || {
        let (w, h) = (PLATE_W, PLATE_H);
        // The plate's top edge sits on the collider's top: snacks rest on it.
        art.paper(spr::PLATE, [-w / 2.0 - 3.0, -h / 2.0 - 1.0, w + 6.0, h + 4.0], WHITE, SHADOW_OFF.rotate(-angle));
    });
}

/// The face a snack makes right now.
pub fn mood(p: &Piece, wobble: f32, ending: bool) -> Mood {
    match p.state {
        State::Lost { .. } => Mood::Dizzy,
        // The plate tipped: whoever's still on it is not happy about it.
        _ if ending => Mood::Worried,
        State::Falling { .. } if p.fear > 0.2 || p.age < 0.6 => Mood::Scared,
        _ if p.fear > 0.4 => Mood::Worried,
        State::Landed if p.solid.is_some() && wobble < 0.3 => Mood::Sleepy,
        _ => Mood::Happy,
    }
}

fn piece(art: &Art, sim: &Sim, p: &Piece, time: f32) {
    let mood = mood(p, sim.wobble, sim.ending.is_some());
    let lost = matches!(p.state, State::Lost { .. });
    let pose = sim.world.draw_pose(p.id);
    let seed = p.id.to_bits() as f32 * 0.37;
    // The shadow falls the same way on screen however the snack is turned.
    let off = SHADOW_OFF.rotate(-pose.angle);
    sim.world.with_transform(p.id, || {
        snack(art, p.kind, p.variant, if lost { 0.8 } else { 1.0 }, off);
        face(art, p.kind, mood, p.fear, time + seed);
    });
}

/// A snack drawn centered at (0, 0) in its own frame, its picture stretched
/// exactly over its collider (`Kind::size`), with a paper shadow toward `off`.
/// `a` fades it.
pub fn snack(art: &Art, kind: Kind, variant: u32, a: f32, off: Vec2) {
    let s = kind.size();
    let (src, tint) = look::snack(kind).sprites[variant as usize % 3];
    art.paper(src, [-s.x / 2.0, -s.y / 2.0, s.x, s.y], with_alpha(tint, a), off);
}

/// Which cell of `faces.png` shows a mood.
fn face_frame(mood: Mood, fear: f32, blink: bool) -> f32 {
    match mood {
        Mood::Happy if blink => 1.0,
        Mood::Happy => 0.0,
        Mood::Excited => 2.0,
        Mood::Scared => 3.0,
        Mood::Worried if fear > 0.6 => 5.0,
        Mood::Worried => 4.0,
        Mood::Sleepy => 6.0,
        Mood::Dizzy => 7.0,
    }
}

/// A face in the snack's frame. `t` animates blinks and the worried shiver.
pub fn face(art: &Art, kind: Kind, mood: Mood, fear: f32, t: f32) {
    let s = kind.size();
    // Faces sit on the visual middle (a wedge's is low, where it's wide).
    // `tall`: how much of the snack's height the face may take (a baguette's
    // face spills a little past its thin crust, or it would be too small to read).
    let (cx, cy, wide, tall) = match kind {
        Kind::Wedge => (0.0, 5.0, 0.62, 0.8),
        Kind::Ball => (0.0, 3.0, 1.0, 0.8),
        Kind::Plank => (0.0, 0.0, 1.0, 1.1),
        _ => (0.0, 0.0, 1.0, 0.8),
    };
    let k = (s.x * wide * 0.68 / 48.0).min(s.y * tall / 32.0).min(0.85);
    let (fw, fh) = (48.0 * k, 32.0 * k);
    let shake = if mood == Mood::Worried { (t * 40.0).sin() * fear * 0.8 } else { 0.0 };
    let blink = matches!(mood, Mood::Happy | Mood::Excited) && (t * 0.31).fract() < 0.04;
    let f = face_frame(mood, fear, blink);
    // Scared faces swell a little with fear.
    let grow = if mood == Mood::Scared { 1.0 + fear * 0.12 } else { 1.0 };
    let (fw, fh) = (fw * grow, fh * grow);
    art.draw(FACES, [f * FACE_CELL.0, 0.0, FACE_CELL.0, FACE_CELL.1], [cx - fw / 2.0 + shake, cy - fh / 2.0, fw, fh], WHITE);
}

/// Where the carried snack would land: a dotted path and a ghost outline.
fn landing_ghost(sim: &Sim, time: f32) {
    let (Some(held), Some(pred)) = (sim.held, sim.predict()) else { return };
    if held.age < 0.25 {
        return;
    }
    let a = 0.75 + 0.2 * (time * 6.0).sin();
    let col = if pred.hits { look::GHOST } else { look::DANGER };
    let shade = with_alpha(INK, 0.22);
    let n = pred.arc.len();
    for (i, p) in pred.arc.iter().enumerate().skip(1) {
        if i + 1 < n {
            shape::disc(p.x, p.y, 3.4, shade);
            shape::disc(p.x, p.y, 2.2, with_alpha(col, a));
        }
    }
    // A shadow of the snack where it would land, with a bright edge.
    let s = held.kind.size();
    let q = pred.land;
    gfx2d::push();
    gfx2d::translate(q.x, q.y);
    let r = match held.kind {
        Kind::Box | Kind::Cube => 6.0,
        k => k.radius().max(s.y / 2.0),
    };
    match held.kind {
        Kind::Ball => {
            shape::disc(0.0, 0.0, s.x / 2.0, shade);
            shape::ring(0.0, 0.0, s.x / 2.0, 2.5, with_alpha(col, a));
        }
        Kind::Wedge => {
            let pts = Kind::wedge_points();
            let mut flat: Vec<f32> = pts.iter().flat_map(|p| [p.x, p.y]).collect();
            shape::poly(&flat, shade);
            flat.extend([pts[0].x, pts[0].y]);
            shape::polyline(&flat, 2.5, with_alpha(col, a));
        }
        _ => {
            let rect = Rect::new(-s.x / 2.0, -s.y / 2.0, s.x, s.y);
            shape::rrect(rect, r.min(s.y / 2.0), shade);
            shape::rrect_stroke(rect, r.min(s.y / 2.0), 2.5, with_alpha(col, a));
        }
    }
    gfx2d::pop();
}

/// The paper bird, flapping, with the snack in its feet.
fn bird(art: &Art, sim: &Sim, time: f32) {
    // After the last heart it flies off, up and away.
    let away = sim.ending.map_or(0.0, |e| e * e * 260.0);
    let b = sim.bird() - vec2(-away * 0.6, away);
    let vx = sim.bird_vx();
    let dir = if vx < -1.0 { -1.0 } else { 1.0 };
    let bob = (time * 4.0).sin() * 3.0;
    let (x, y) = (b.x, b.y + bob);
    if let (Some(h), Some(hp)) = (sim.held, sim.held_pos()) {
        // It swoops in with each new snack.
        let k = (h.age / 0.25).min(1.0);
        let hy = hp.y + bob - (1.0 - k) * 30.0;
        gfx2d::push();
        gfx2d::translate(hp.x, hy);
        let sway = (time * 3.0).sin() * 0.04 - vx * 0.0006;
        gfx2d::rotate(sway);
        gfx2d::scale(k, k);
        snack(art, h.kind, h.variant, 1.0, SHADOW_OFF.rotate(-sway));
        face(art, h.kind, Mood::Excited, 0.0, time);
        gfx2d::pop();
    }
    // The sheet's feet are at the bottom of each cell: they grip the snack's top.
    let frame = ((time * 11.0) as usize % BIRD_FRAMES) as f32;
    let src = [frame * BIRD_CELL.0, 0.0, BIRD_CELL.0, BIRD_CELL.1];
    let (w, h) = (BIRD_CELL.0 / 2.0, BIRD_CELL.1 / 2.0);
    gfx2d::push();
    gfx2d::translate(x, y);
    gfx2d::scale(dir, 1.0);
    for (k, sa) in [(1.4, 0.12), (2.6, 0.08)] {
        art.draw(BIRD, src, [-w / 2.0 + SHADOW_OFF.x * k * dir, 22.0 - h + SHADOW_OFF.y * k, w, h], with_alpha(look::SHADOW, sa));
    }
    art.draw(BIRD, src, [-w / 2.0, 22.0 - h, w, h], WHITE);
    gfx2d::pop();
}

/// Wind: paper streaks and leaves while a gust is announced or blowing;
/// tilt arrows at the plate.
fn wind(art: &Art, sim: &Sim, time: f32) {
    let (kind, dir, k) = match sim.twist {
        Twist::Warn { kind, dir, t } => (kind, dir, 0.45 + 0.2 * (t * 8.0).sin()),
        Twist::On { kind, dir, .. } => (kind, dir, 1.0),
        _ => return,
    };
    match kind {
        TwistKind::Gust => {
            let strength = if k >= 1.0 { (sim.wind.abs() / 70.0).clamp(0.4, 1.0) } else { k };
            let span = W + 160.0;
            for i in 0..12 {
                let fi = i as f32;
                let x = ((fi * 71.0 + time * 460.0 * (0.8 + (i % 3) as f32 * 0.2)) % span) - 80.0;
                let x = if dir > 0.0 { x } else { W - x };
                let y = sim.cam_y + 110.0 + (fi * 43.0) % 420.0;
                let len = 34.0 + (i % 4) as f32 * 14.0;
                shape::capsule(x, y + 2.5, x - dir * len, y + 2.5, 3.0, with_alpha(look::SHADOW, 0.18 * strength));
                shape::capsule(x, y, x - dir * len, y, 3.0, with_alpha(look::PAPER_WHITE, 0.85 * strength));
            }
            for i in 0..6 {
                let fi = i as f32;
                let x = ((fi * 97.0 + time * 300.0) % span) - 80.0;
                let x = if dir > 0.0 { x } else { W - x };
                let y = sim.cam_y + 160.0 + (fi * 83.0) % 360.0 + (time * 5.0 + fi).sin() * 12.0;
                gfx2d::push();
                gfx2d::translate(x, y);
                gfx2d::rotate(time * 6.0 * dir + fi);
                art.paper(spr::LEAF, [-6.0, -10.0, 12.0, 20.0], with_alpha(WHITE, strength), SHADOW_OFF);
                gfx2d::pop();
            }
        }
        TwistKind::Tilt => {
            // Arrows at the plate's ends (kept on screen when the plate is below it).
            let y = (PLATE_TOP + PLATE_H / 2.0).clamp(sim.cam_y + 140.0, sim.cam_y + 600.0);
            let bounce = (time * 7.0).sin() * 4.0;
            for sx in [-1.0, 1.0] {
                let x = W / 2.0 + sx * (PLATE_W / 2.0 + 22.0);
                // A clockwise tilt (dir > 0) takes the right end down.
                let up = sx * dir < 0.0;
                let (y0, y1) = if up { (y + 14.0, y - 16.0 + bounce) } else { (y - 14.0, y + 16.0 + bounce) };
                let c = with_alpha(look::PAPER_WHITE, 0.6 * k + 0.3);
                let d = if y1 < y0 { -1.0 } else { 1.0 };
                for (col, off) in [(with_alpha(INK, 0.3), 2.0), (c, 0.0)] {
                    shape::capsule(x, y0 + off, x, y1 + off, 5.0, col);
                    shape::poly(&[x - 9.0, y1 + off, x + 9.0, y1 + off, x, y1 + d * 11.0 + off], col);
                }
            }
        }
    }
}
