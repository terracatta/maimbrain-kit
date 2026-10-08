//! `0xRRGGBBAA` color helpers.

/// From 0–255 channels.
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | a as u32
}

/// `0xRRGGBB` → opaque `0xRRGGBBAA`.
pub const fn hex(rgb: u32) -> u32 {
    (rgb << 8) | 0xff
}

/// Replaces the alpha with `a` (0…1).
pub fn with_alpha(c: u32, a: f32) -> u32 {
    (c & 0xffff_ff00) | (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u32
}

/// Multiplies the alpha by `a` (0…1): fading something already translucent.
pub fn fade(c: u32, a: f32) -> u32 {
    (c & 0xffff_ff00) | (((c & 0xff) as f32) * a.clamp(0.0, 1.0) + 0.5) as u32
}

fn ch(c: u32, shift: u32) -> f32 {
    ((c >> shift) & 0xff) as f32
}

/// Linear blend from `a` (t = 0) to `b` (t = 1), alpha included.
pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let m = |s| ((ch(a, s) + (ch(b, s) - ch(a, s)) * t + 0.5) as u32) << s;
    m(24) | m(16) | m(8) | m(0)
}

/// Toward white by `t`, keeping alpha.
pub fn lighten(c: u32, t: f32) -> u32 {
    (mix(c, 0xffffffff, t) & 0xffff_ff00) | (c & 0xff)
}

/// Toward black by `t`, keeping alpha.
pub fn darken(c: u32, t: f32) -> u32 {
    (mix(c, 0x000000ff, t) & 0xffff_ff00) | (c & 0xff)
}

/// Hue in degrees, saturation and value 0…1, opaque.
pub fn hsv(h: f32, s: f32, v: f32) -> u32 {
    let h = h.rem_euclid(360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    let to = |f: f32| (((f + m) * 255.0).round() as u32).min(255);
    (to(r) << 24) | (to(g) << 16) | (to(b) << 8) | 0xff
}

/// Relative brightness 0…1 (for picking dark or light text on a color).
pub fn luma(c: u32) -> f32 {
    (0.299 * ch(c, 24) + 0.587 * ch(c, 16) + 0.114 * ch(c, 8)) / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_math() {
        assert_eq!(rgba(0x12, 0x34, 0x56, 0x78), 0x12345678);
        assert_eq!(hex(0xff8800), 0xff8800ff);
        assert_eq!(with_alpha(0x112233ff, 0.0), 0x11223300);
        assert_eq!(fade(0x11223380, 0.5), 0x11223340);
        assert_eq!(mix(0x000000ff, 0xffffffff, 0.5), 0x808080ff);
        assert_eq!(lighten(0x00000080, 1.0), 0xffffff80);
        assert_eq!(hsv(0.0, 1.0, 1.0), 0xff0000ff);
        assert_eq!(hsv(120.0, 1.0, 1.0), 0x00ff00ff);
        assert!(luma(0xffffffff) > 0.99 && luma(0x000000ff) < 0.01);
    }
}
