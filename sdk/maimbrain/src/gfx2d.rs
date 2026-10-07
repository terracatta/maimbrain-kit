//! The host 2D engine, `mb2d` (SPEC §5.3). Declare `stdlib = { mb2d = 1 }`
//! in manifest.toml. Colors are `0xRRGGBBAA`; coordinates are logical units,
//! origin top-left, +y down.

use crate::ffi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Blend {
    Alpha = 0,
    Add = 1,
    Multiply = 2,
    Screen = 3,
}

/// Host fonts (SPEC §5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Font {
    Sans = 0,
    SansBold = 1,
    /// 5×7 pixel font; crisp at sizes that are multiples of 8.
    Pixel = 2,
}

/// An image decoded from a ready PNG asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Image(pub u32);

impl Image {
    /// None until the asset is ready, or if it isn't a valid PNG.
    pub fn new(asset: crate::sys::Asset) -> Option<Image> {
        let h = unsafe { ffi::mb2d_image(asset.0) };
        (h > 0).then_some(Image(h as u32))
    }
}

pub fn clear(rgba: u32) {
    unsafe { ffi::mb2d_clear(rgba) }
}

pub fn push() {
    unsafe { ffi::mb2d_push() }
}

pub fn pop() {
    unsafe { ffi::mb2d_pop() }
}

pub fn translate(x: f32, y: f32) {
    unsafe { ffi::mb2d_translate(x, y) }
}

pub fn rotate(radians: f32) {
    unsafe { ffi::mb2d_rotate(radians) }
}

pub fn scale(sx: f32, sy: f32) {
    unsafe { ffi::mb2d_scale(sx, sy) }
}

pub fn blend(mode: Blend) {
    unsafe { ffi::mb2d_blend(mode as u32) }
}

pub fn rect(x: f32, y: f32, w: f32, h: f32, rgba: u32) {
    unsafe { ffi::mb2d_rect(x, y, w, h, rgba) }
}

/// A rectangle shaded from `top` to `bottom`.
pub fn rect_gradient(x: f32, y: f32, w: f32, h: f32, top: u32, bottom: u32) {
    unsafe { ffi::mb2d_rect_gradient(x, y, w, h, top, bottom) }
}

pub fn circle(x: f32, y: f32, r: f32, rgba: u32) {
    unsafe { ffi::mb2d_circle(x, y, r, rgba) }
}

pub fn line(x0: f32, y0: f32, x1: f32, y1: f32, width: f32, rgba: u32) {
    unsafe { ffi::mb2d_line(x0, y0, x1, y1, width, rgba) }
}

/// A filled polygon; `points` is `[x0, y0, x1, y1, ...]`.
pub fn poly(points: &[f32], rgba: u32) {
    unsafe { ffi::mb2d_poly(points.as_ptr(), (points.len() / 2) as u32, rgba) }
}

/// Draws `src` (pixels in the image) into `dst` (logical units), tinted.
pub fn sprite(img: Image, src: [f32; 4], dst: [f32; 4], tint: u32) {
    unsafe { ffi::mb2d_sprite(img.0, src[0], src[1], src[2], src[3], dst[0], dst[1], dst[2], dst[3], tint) }
}

/// Text with its first line's top at `y`; `size` is the em height in logical
/// units. `\n` starts a new line.
pub fn text(font: Font, size: f32, x: f32, y: f32, rgba: u32, s: &str) {
    unsafe { ffi::mb2d_text(font as u32, size, x, y, rgba, s.as_ptr(), s.len() as u32) }
}

/// Width of the widest line of `s`, in logical units.
pub fn measure(font: Font, size: f32, s: &str) -> f32 {
    unsafe { ffi::mb2d_measure(font as u32, size, s.as_ptr(), s.len() as u32) }
}

/// Text centered horizontally on `cx`.
pub fn text_centered(font: Font, size: f32, cx: f32, y: f32, rgba: u32, s: &str) {
    text(font, size, cx - measure(font, size, s) / 2.0, y, rgba, s);
}
