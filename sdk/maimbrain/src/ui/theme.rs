//! Themes: one struct of colors, fonts and shape settings every widget and
//! screen reads, so a game restyles the whole kit in one place.

use super::color::{darken, hex, lighten};
use crate::gfx2d::Font;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    /// Backdrop gradient for screens that draw their own background.
    pub bg_top: u32,
    pub bg_bottom: u32,
    /// Panels and cards.
    pub panel_top: u32,
    pub panel_bottom: u32,
    pub panel_border: u32,
    /// Body text and quieter text.
    pub text: u32,
    pub text_dim: u32,
    /// Primary buttons, highlights, progress fills.
    pub accent: u32,
    /// Text and icons on `accent`.
    pub on_accent: u32,
    /// Secondary buttons.
    pub secondary: u32,
    pub on_secondary: u32,
    pub good: u32,
    pub bad: u32,
    /// Bests, stars, coins.
    pub gold: u32,
    /// Outline around titles and big numbers.
    pub outline: u32,
    pub shadow: u32,
    /// Corner radius of panels (buttons use about half of it).
    pub radius: f32,
    /// Width of title outlines, in ems.
    pub outline_em: f32,
    pub title_font: Font,
    pub body_font: Font,
    /// Scores and counters.
    pub number_font: Font,
    /// Confetti colors.
    pub confetti: [u32; 5],
}

impl Default for Theme {
    fn default() -> Self {
        Theme::candy()
    }
}

impl Theme {
    /// Bright and friendly: plum panels, pink accent, gold bests.
    pub fn candy() -> Theme {
        Theme {
            bg_top: hex(0x3a1c71),
            bg_bottom: hex(0x1a0b3a),
            panel_top: hex(0x4a2a8a),
            panel_bottom: hex(0x2c1660),
            panel_border: 0xffffff30,
            text: 0xffffffff,
            text_dim: 0xe8dcffb0,
            accent: hex(0xff4f9a),
            on_accent: 0xffffffff,
            secondary: 0xffffff26,
            on_secondary: 0xffffffff,
            good: hex(0x4be3a0),
            bad: hex(0xff5a5a),
            gold: hex(0xffd23f),
            outline: hex(0x1d0b3d),
            shadow: 0x0a001a70,
            radius: 22.0,
            outline_em: 0.07,
            title_font: Font::SansBold,
            body_font: Font::SansBold,
            number_font: Font::SansBold,
            confetti: [hex(0xff4f9a), hex(0xffd23f), hex(0x4be3a0), hex(0x5ab8ff), hex(0xffffff)],
        }
    }

    /// Deep navy with an electric cyan accent.
    pub fn night() -> Theme {
        Theme {
            bg_top: hex(0x0b1a33),
            bg_bottom: hex(0x050b18),
            panel_top: hex(0x16294d),
            panel_bottom: hex(0x0d1a33),
            panel_border: 0x6fd3ff40,
            text: hex(0xeaf6ff),
            text_dim: 0xb5cde6b0,
            accent: hex(0x2ee6d6),
            on_accent: hex(0x04211f),
            secondary: 0x6fd3ff24,
            on_secondary: hex(0xeaf6ff),
            good: hex(0x6cf28a),
            bad: hex(0xff5d73),
            gold: hex(0xffcf4a),
            outline: hex(0x020814),
            shadow: 0x00000080,
            radius: 18.0,
            outline_em: 0.06,
            title_font: Font::SansBold,
            body_font: Font::Sans,
            number_font: Font::SansBold,
            confetti: [hex(0x2ee6d6), hex(0xffcf4a), hex(0xff5d73), hex(0x8f7bff), hex(0xeaf6ff)],
        }
    }

    /// Chunky pixel-font arcade: black panels, neon green, square corners.
    pub fn arcade() -> Theme {
        Theme {
            bg_top: hex(0x10101a),
            bg_bottom: hex(0x000000),
            panel_top: hex(0x1b1b2b),
            panel_bottom: hex(0x101018),
            panel_border: hex(0x39ff88),
            text: hex(0xffffff),
            text_dim: 0xffffffa0,
            accent: hex(0x39ff88),
            on_accent: hex(0x06200f),
            secondary: hex(0x2a2a40),
            on_secondary: hex(0xffffff),
            good: hex(0x39ff88),
            bad: hex(0xff3b5c),
            gold: hex(0xffe14a),
            outline: hex(0x000000),
            shadow: 0x000000a0,
            radius: 4.0,
            outline_em: 0.125,
            title_font: Font::Pixel,
            body_font: Font::Pixel,
            number_font: Font::Pixel,
            confetti: [hex(0x39ff88), hex(0xffe14a), hex(0xff3b5c), hex(0x3bc9ff), hex(0xffffff)],
        }
    }

    /// Warm paper: cream panels, ink text, tomato accent.
    pub fn paper() -> Theme {
        Theme {
            bg_top: hex(0xfff3df),
            bg_bottom: hex(0xf6dcc0),
            panel_top: hex(0xfffaf0),
            panel_bottom: hex(0xfbecd6),
            panel_border: 0x3a2a1a30,
            text: hex(0x2b1d14),
            text_dim: 0x2b1d14a0,
            accent: hex(0xf2542d),
            on_accent: hex(0xffffff),
            secondary: 0x2b1d1418,
            on_secondary: hex(0x2b1d14),
            good: hex(0x2f9e5b),
            bad: hex(0xd63b2f),
            gold: hex(0xf0a400),
            outline: hex(0xffffff),
            shadow: 0x5a3a1a40,
            radius: 20.0,
            outline_em: 0.06,
            title_font: Font::SansBold,
            body_font: Font::SansBold,
            number_font: Font::SansBold,
            confetti: [hex(0xf2542d), hex(0xf0a400), hex(0x2f9e5b), hex(0x3a86ff), hex(0x8338ec)],
        }
    }

    /// Lime and teal on forest green.
    pub fn jungle() -> Theme {
        Theme {
            bg_top: hex(0x0f3d2e),
            bg_bottom: hex(0x06201a),
            panel_top: hex(0x1d5c45),
            panel_bottom: hex(0x0f3b2c),
            panel_border: 0xc9ff7a40,
            text: hex(0xf3ffe8),
            text_dim: 0xd6f5c8b0,
            accent: hex(0xc9ff3a),
            on_accent: hex(0x15300a),
            secondary: 0xffffff22,
            on_secondary: hex(0xf3ffe8),
            good: hex(0x7dff9b),
            bad: hex(0xff6b4a),
            gold: hex(0xffd84a),
            outline: hex(0x062014),
            shadow: 0x00100a80,
            radius: 26.0,
            outline_em: 0.08,
            title_font: Font::SansBold,
            body_font: Font::SansBold,
            number_font: Font::SansBold,
            confetti: [hex(0xc9ff3a), hex(0xffd84a), hex(0xff6b4a), hex(0x4ad8ff), hex(0xffffff)],
        }
    }

    /// All presets with their names.
    pub fn presets() -> [(&'static str, Theme); 5] {
        [("candy", Theme::candy()), ("night", Theme::night()), ("arcade", Theme::arcade()), ("paper", Theme::paper()), ("jungle", Theme::jungle())]
    }

    /// Same theme, another accent (and readable text on it).
    pub fn with_accent(mut self, accent: u32) -> Theme {
        self.accent = accent;
        self.on_accent = if super::color::luma(accent) > 0.6 { darken(accent, 0.8) } else { 0xffffffff };
        self
    }

    /// Same theme, panels in shades of `color`.
    pub fn with_panel(mut self, color: u32) -> Theme {
        self.panel_top = lighten(color, 0.08);
        self.panel_bottom = darken(color, 0.25);
        self
    }

    pub fn with_radius(mut self, r: f32) -> Theme {
        self.radius = r;
        self
    }

    /// Every text role in one font.
    pub fn with_font(mut self, f: crate::gfx2d::Font) -> Theme {
        self.title_font = f;
        self.body_font = f;
        self.number_font = f;
        if f == crate::gfx2d::Font::Pixel {
            self.outline_em = 0.125;
        }
        self
    }

    /// A text size fitted to the font: the pixel font only looks crisp at
    /// multiples of 8, so sizes round to those for it.
    pub fn fit(&self, font: Font, size: f32) -> f32 {
        if font == Font::Pixel { ((size / 8.0).round() * 8.0).max(8.0) } else { size }
    }
}
