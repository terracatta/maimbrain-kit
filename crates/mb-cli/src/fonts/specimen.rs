//! `mb fonts --preview`: a specimen sheet of library fonts, rasterized here
//! (anti-aliased, 4× supersampled) with the baker's outline code.

use ttf_parser::{Face, Tag};

use super::Entry;
use super::bake::{fill, outline};

const COLS: usize = 2;
const CARD_W: usize = 820;
const CARD_H: usize = 138;
const SS: usize = 4;

struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[f32; 3]>,
}

impl Canvas {
    fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, c: [f32; 3]) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                self.px[yy * self.w + xx] = c;
            }
        }
    }

    /// Draws `text` with its baseline at `y`; stops at `max_x`. Returns the pen's end.
    fn text(&mut self, face: &Face, text: &str, size: f32, x: f32, y: f32, color: [f32; 3], max_x: f32) -> f32 {
        let s = size / face.units_per_em() as f32;
        let mut pen = x;
        for c in text.chars() {
            let Some(gid) = face.glyph_index(c) else { continue };
            let adv = face.glyph_hor_advance(gid).unwrap_or(0) as f32 * s;
            if pen + adv > max_x {
                break;
            }
            let segs = outline(face, gid, s);
            if !segs.is_empty() {
                let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for q in &segs {
                    x0 = x0.min(q[0]).min(q[2]);
                    x1 = x1.max(q[0]).max(q[2]);
                    y0 = y0.min(q[1]).min(q[3]);
                    y1 = y1.max(q[1]).max(q[3]);
                }
                let (gx, gy) = ((pen + x0).floor(), (y + y0).floor());
                let (w, h) = (((pen + x1).ceil() - gx) as usize + 1, ((y + y1).ceil() - gy) as usize + 1);
                let k = SS as f32;
                let hi: Vec<[f32; 4]> = segs.iter().map(|q| [(q[0] + pen - gx) * k, (q[1] + y - gy) * k, (q[2] + pen - gx) * k, (q[3] + y - gy) * k]).collect();
                let inside = fill(&hi, w * SS, h * SS);
                for oy in 0..h {
                    for ox in 0..w {
                        let mut n = 0;
                        for sy in 0..SS {
                            for sx in 0..SS {
                                n += inside[(oy * SS + sy) * w * SS + ox * SS + sx] as usize;
                            }
                        }
                        let (cx, cy) = (gx as i64 + ox as i64, gy as i64 + oy as i64);
                        if n == 0 || cx < 0 || cy < 0 || cx as usize >= self.w || cy as usize >= self.h {
                            continue;
                        }
                        let a = n as f32 / (SS * SS) as f32;
                        let p = &mut self.px[cy as usize * self.w + cx as usize];
                        for i in 0..3 {
                            p[i] = p[i] * (1.0 - a) + color[i] * a;
                        }
                    }
                }
            }
            pen += adv;
        }
        pen
    }
}

pub fn sheet(fonts: &[(&Entry, Vec<u8>, Option<f32>)], label: &[u8]) -> Result<Vec<u8>, String> {
    let rows = fonts.len().div_ceil(COLS);
    let (w, h) = (COLS * CARD_W, rows * CARD_H + 16);
    let mut c = Canvas { w, h, px: vec![[0.97, 0.96, 0.94]; w * h] };
    let label = Face::parse(label, 0).map_err(|e| format!("label font: {e}"))?;
    for (i, (e, data, wght)) in fonts.iter().enumerate() {
        let (x0, y0) = ((i % COLS) * CARD_W, (i / COLS) * CARD_H + 8);
        c.rect(x0 + 8, y0, CARD_W - 16, CARD_H - 10, [1.0, 1.0, 1.0]);
        let mut face = Face::parse(data, 0).map_err(|err| format!("{}: {err}", e.id))?;
        if let Some(wv) = wght {
            face.set_variation(Tag::from_bytes(b"wght"), *wv);
        }
        let (x, right) = (x0 as f32 + 22.0, (x0 + CARD_W - 22) as f32);
        let tag = format!("{}   ·   {}   ·   {}", e.id, e.category, e.mood.join(", "));
        c.text(&label, &tag, 13.0, x, y0 as f32 + 20.0, [0.45, 0.42, 0.40], right);
        // Pixel fonts at a whole multiple of their grid.
        let big = e.pixel_em.map_or(44.0, |p| (44.0 / p as f32).round().max(1.0) * p as f32);
        let small = e.pixel_em.map_or(21.0, |p| (21.0 / p as f32).round().max(1.0) * p as f32);
        let title = if e.caps_only == Some(true) { e.family.to_uppercase() } else { e.family.clone() };
        let end = c.text(&face, &title, big, x, y0 as f32 + 72.0, [0.10, 0.09, 0.12], right);
        c.text(&face, "  1,234,567", big * 0.8, end, y0 as f32 + 72.0, [0.85, 0.30, 0.20], right);
        let sample = if e.caps_only == Some(true) { "THE QUICK BROWN FOX JUMPS OVER 3 LAZY DOGS! ★" } else { "The quick brown fox jumps over 3 lazy dogs! NEW BEST ★" };
        c.text(&face, sample, small, x, y0 as f32 + 112.0, [0.20, 0.20, 0.25], right);
    }
    let mut rgb = Vec::with_capacity(w * h * 3);
    for p in &c.px {
        rgb.extend(p.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_deflate_compression(png::DeflateCompression::Level(9));
        enc.set_filter(png::Filter::Adaptive);
        let mut wr = enc.write_header().map_err(|e| e.to_string())?;
        wr.write_image_data(&rgb).map_err(|e| e.to_string())?;
    }
    Ok(out)
}
