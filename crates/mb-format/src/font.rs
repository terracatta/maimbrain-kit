//! Game fonts (`.mbf`, SPEC §5.3 "Game fonts"): a font baked at build time
//! (`mb font add`) into a glyph atlas plus metrics and kerning, which a game
//! loads with `mb2d_font_load`. Signed-distance-field glyphs (`Kind::Sdf`)
//! scale cleanly and take the host's outlines and glows; 1-bit glyphs
//! (`Kind::Bitmap`) are for pixel fonts, drawn with nearest sampling.
//!
//! Layout (little endian):
//!
//! ```text
//!  0  "MBFG"
//!  4  u16 version (1)
//!  6  u8  kind (0 SDF, 1 bitmap)
//!  7  u8  reserved (0)
//!  8  u16 glyph count
//! 10  u16 kerning pair count
//! 12  f32 em_px         pixels per em the glyphs are stored at
//! 16  f32 ascent        px at em_px, above the baseline
//! 20  f32 descent       px at em_px, below the baseline (positive)
//! 24  f32 line_height   px at em_px, baseline to baseline
//! 28  f32 spread        SDF distance range (px at em_px) each side of the edge; 0 for bitmaps
//! 32  f32 cap_height    px at em_px
//! 36  f32 x_height      px at em_px
//! 40  u16 atlas width, u16 atlas height
//! 44  u16 meta length, u16 reserved (0)
//! 48  u32 PNG length
//! 52  meta: UTF-8 `key=value` lines (family, weight, license, source, recipe)
//!     glyphs: count × (u32 codepoint, u16 x, y, w, h, f32 ox, oy, advance)
//!             atlas rect in texels; quad top-left = pen + (ox, oy) px at em_px, y down
//!     kerning: count × (u32 left, u32 right, f32 adjust px at em_px), sorted by (left, right)
//!     PNG: the atlas, 8-bit greyscale, atlas width × height
//! ```

pub const MAGIC: &[u8; 4] = b"MBFG";
pub const VERSION: u16 = 1;
/// Largest `.mbf` file (SPEC §1).
pub const MAX_BYTES: usize = 512 * 1024;
/// Largest atlas side.
pub const MAX_ATLAS: u16 = 1024;
pub const MAX_GLYPHS: usize = 1024;
pub const MAX_KERNS: usize = 16384;
pub const MAX_META: usize = 2048;
/// Fonts a game can load (and `.mbf` files a bundle may hold).
pub const MAX_FONTS: usize = 16;
/// The first font id `mb2d_font_load` hands out; 0–15 are host fonts.
pub const FIRST_GAME_FONT: u32 = 16;
const HEADER: usize = 52;
const GLYPH: usize = 24;
const KERN: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Sdf = 0,
    Bitmap = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub codepoint: u32,
    /// Atlas rect in texels.
    pub rect: [u16; 4],
    /// Quad top-left relative to the pen on the baseline (px at em_px, y down).
    pub offset: [f32; 2],
    pub advance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kern {
    pub left: u32,
    pub right: u32,
    pub adjust: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MbFont {
    pub kind: Kind,
    pub em_px: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_height: f32,
    pub spread: f32,
    pub cap_height: f32,
    pub x_height: f32,
    pub atlas: [u16; 2],
    pub meta: String,
    pub glyphs: Vec<Glyph>,
    pub kerns: Vec<Kern>,
    pub png: Vec<u8>,
}

impl MbFont {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(HEADER + self.meta.len() + self.glyphs.len() * GLYPH + self.kerns.len() * KERN + self.png.len());
        o.extend(MAGIC);
        o.extend(VERSION.to_le_bytes());
        o.extend([self.kind as u8, 0]);
        o.extend((self.glyphs.len() as u16).to_le_bytes());
        o.extend((self.kerns.len() as u16).to_le_bytes());
        for v in [self.em_px, self.ascent, self.descent, self.line_height, self.spread, self.cap_height, self.x_height] {
            o.extend(v.to_le_bytes());
        }
        o.extend(self.atlas[0].to_le_bytes());
        o.extend(self.atlas[1].to_le_bytes());
        o.extend((self.meta.len() as u16).to_le_bytes());
        o.extend(0u16.to_le_bytes());
        o.extend((self.png.len() as u32).to_le_bytes());
        o.extend(self.meta.as_bytes());
        for g in &self.glyphs {
            o.extend(g.codepoint.to_le_bytes());
            for v in g.rect {
                o.extend(v.to_le_bytes());
            }
            for v in [g.offset[0], g.offset[1], g.advance] {
                o.extend(v.to_le_bytes());
            }
        }
        for k in &self.kerns {
            o.extend(k.left.to_le_bytes());
            o.extend(k.right.to_le_bytes());
            o.extend(k.adjust.to_le_bytes());
        }
        o.extend(&self.png);
        o
    }

    /// Parses and checks a `.mbf`: structure, limits, finite metrics, glyph
    /// rects inside the atlas, and a PNG header of the atlas's size.
    pub fn parse(b: &[u8]) -> Result<MbFont, String> {
        if b.len() > MAX_BYTES {
            return Err(format!("{} bytes; a font may be at most {MAX_BYTES}", b.len()));
        }
        if b.len() < HEADER || &b[..4] != MAGIC {
            return Err("not a Maimbrain font (bake fonts with `mb font add`)".into());
        }
        let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        let u32_at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let f32_at = |o: usize| f32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        if u16_at(4) != VERSION {
            return Err(format!("font version {} is not supported (this validator reads {VERSION})", u16_at(4)));
        }
        let kind = match b[6] {
            0 => Kind::Sdf,
            1 => Kind::Bitmap,
            k => return Err(format!("unknown font kind {k}")),
        };
        let (n, nk) = (u16_at(8) as usize, u16_at(10) as usize);
        let m: Vec<f32> = (0..7).map(|i| f32_at(12 + i * 4)).collect();
        let atlas = [u16_at(40), u16_at(42)];
        let (meta_len, png_len) = (u16_at(44) as usize, u32_at(48) as usize);
        if n == 0 || n > MAX_GLYPHS {
            return Err(format!("{n} glyphs; a font has 1–{MAX_GLYPHS}"));
        }
        if nk > MAX_KERNS {
            return Err(format!("{nk} kerning pairs; the limit is {MAX_KERNS}"));
        }
        if meta_len > MAX_META {
            return Err(format!("metadata is {meta_len} bytes; the limit is {MAX_META}"));
        }
        if !m.iter().all(|v| v.is_finite()) {
            return Err("metrics must be finite".into());
        }
        let [em_px, ascent, descent, line_height, spread, cap_height, x_height] = m[..] else { unreachable!() };
        if !(4.0..=128.0).contains(&em_px) || line_height <= 0.0 || spread < 0.0 || spread > em_px {
            return Err("metrics out of range (em_px 4–128, line height > 0, spread 0–em_px)".into());
        }
        if atlas[0] == 0 || atlas[1] == 0 || atlas[0] > MAX_ATLAS || atlas[1] > MAX_ATLAS {
            return Err(format!("atlas {}×{} is outside 1–{MAX_ATLAS} on a side", atlas[0], atlas[1]));
        }
        let need = HEADER + meta_len + n * GLYPH + nk * KERN + png_len;
        if b.len() != need {
            return Err(format!("{} bytes, but the header describes {need}", b.len()));
        }
        let meta = std::str::from_utf8(&b[HEADER..HEADER + meta_len]).map_err(|_| "metadata is not UTF-8")?.to_string();
        let mut at = HEADER + meta_len;
        let mut glyphs = Vec::with_capacity(n);
        for _ in 0..n {
            let g = Glyph {
                codepoint: u32_at(at),
                rect: [u16_at(at + 4), u16_at(at + 6), u16_at(at + 8), u16_at(at + 10)],
                offset: [f32_at(at + 12), f32_at(at + 16)],
                advance: f32_at(at + 20),
            };
            let [x, y, w, h] = g.rect.map(u32::from);
            if x + w > atlas[0] as u32 || y + h > atlas[1] as u32 {
                return Err(format!("glyph U+{:04X} lies outside the atlas", g.codepoint));
            }
            if char::from_u32(g.codepoint).is_none() || ![g.offset[0], g.offset[1], g.advance].iter().all(|v| v.is_finite()) {
                return Err(format!("glyph U+{:04X} is invalid", g.codepoint));
            }
            glyphs.push(g);
            at += GLYPH;
        }
        let mut kerns = Vec::with_capacity(nk);
        for _ in 0..nk {
            let k = Kern { left: u32_at(at), right: u32_at(at + 4), adjust: f32_at(at + 8) };
            if !k.adjust.is_finite() {
                return Err("kerning must be finite".into());
            }
            kerns.push(k);
            at += KERN;
        }
        let png = b[at..].to_vec();
        if png.len() < 24 || &png[..8] != b"\x89PNG\r\n\x1a\n" || &png[12..16] != b"IHDR" {
            return Err("atlas is not a PNG".into());
        }
        let be = |o: usize| u32::from_be_bytes(png[o..o + 4].try_into().unwrap());
        if (be(16), be(20)) != (atlas[0] as u32, atlas[1] as u32) {
            return Err(format!("atlas PNG is {}×{}, the header says {}×{}", be(16), be(20), atlas[0], atlas[1]));
        }
        Ok(MbFont { kind, em_px, ascent, descent, line_height, spread, cap_height, x_height, atlas, meta, glyphs, kerns, png })
    }

    /// A `key=value` line of the metadata.
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        self.meta.lines().find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn sample() -> MbFont {
        // A 2×1 greyscale PNG would need an encoder; a header-only stand-in is
        // enough for the structural checks here.
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend(16u32.to_be_bytes());
        png.extend(8u32.to_be_bytes());
        MbFont {
            kind: Kind::Sdf,
            em_px: 32.0,
            ascent: 30.0,
            descent: 8.0,
            line_height: 40.0,
            spread: 4.0,
            cap_height: 22.0,
            x_height: 16.0,
            atlas: [16, 8],
            meta: "family=Test\nlicense=OFL-1.1".into(),
            glyphs: vec![Glyph { codepoint: 'A' as u32, rect: [0, 0, 8, 8], offset: [-1.0, -24.0], advance: 20.0 }],
            kerns: vec![Kern { left: 'A' as u32, right: 'V' as u32, adjust: -2.0 }],
            png,
        }
    }

    #[test]
    fn round_trips_and_checks() {
        let f = sample();
        let b = f.encode();
        assert_eq!(MbFont::parse(&b).unwrap(), f);
        assert_eq!(f.meta_value("license"), Some("OFL-1.1"));
        assert!(MbFont::parse(&b[..b.len() - 1]).is_err(), "truncated");
        let mut bad = f.clone();
        bad.glyphs[0].rect = [10, 0, 8, 8];
        assert!(MbFont::parse(&bad.encode()).unwrap_err().contains("outside the atlas"));
        let mut bad = f.clone();
        bad.atlas = [2048, 8];
        assert!(MbFont::parse(&bad.encode()).is_err());
        let mut bad = f;
        bad.em_px = f32::NAN;
        assert!(MbFont::parse(&bad.encode()).is_err());
        assert!(MbFont::parse(b"MBF1....").is_err());
    }
}
