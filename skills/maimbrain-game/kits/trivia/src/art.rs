//! The generated art (tools/regen.sh makes every file from art/style.toml):
//! the stage backdrop, Watt's expression sheet, the category emblems, the
//! marquee sign and the starburst. Images load asynchronously; until one is
//! ready, `draw` falls back to code-drawn shapes, so the first frame always
//! looks right. Polled in `update` (output only: nothing here changes the game).

use maimbrain::gfx2d::{self, Image};
use maimbrain::sys::{Asset, AssetState};
use maimbrain::ui::Rect;

/// Watt's sheet: 4 columns of 320×352 cells, frames in `Frame` order. The
/// glass's centre sits at (160, 165) in every cell and its radius is ~90 px.
pub const HOST_CELL: (f32, f32) = (320.0, 352.0);
pub const HOST_COLS: usize = 4;
pub const HOST_GLASS: (f32, f32, f32) = (160.0, 165.0, 90.0);
/// The emblems: 5 columns of 112×112 cells (`look::category` order); the
/// round badge fills ~98 px of each.
pub const CAT_CELL: f32 = 112.0;
pub const CAT_COLS: usize = 5;
pub const CAT_BADGE: f32 = 98.0;
/// The marquee sign (680×300) and the teal panel inside it, in its pixels.
pub const MARQUEE: (f32, f32) = (680.0, 300.0);
pub const MARQUEE_PANEL: [f32; 4] = [95.0, 65.0, 492.0, 166.0];
/// The starburst (256×256).
pub const BURST: f32 = 256.0;

/// Watt's frames, in sheet order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    Idle = 0,
    Think = 1,
    Nervous = 2,
    Happy = 3,
    Star = 4,
    Flicker = 5,
    Cracked = 6,
}

const FILES: [&str; 5] = ["assets/stage.png", "assets/host.png", "assets/cats.png", "assets/marquee.png", "assets/burst.png"];

pub struct Art {
    assets: [Asset; 5],
    images: [Option<Image>; 5],
}

impl Art {
    pub fn new() -> Art {
        Art { assets: FILES.map(Asset::load), images: [None; 5] }
    }

    pub fn update(&mut self) {
        for (i, a) in self.assets.iter().enumerate() {
            if self.images[i].is_none() && a.state() == AssetState::Ready {
                self.images[i] = Image::new(*a);
            }
        }
    }

    pub fn stage(&self) -> Option<Image> {
        self.images[0]
    }
    pub fn host(&self) -> Option<Image> {
        self.images[1]
    }
    pub fn cats(&self) -> Option<Image> {
        self.images[2]
    }
    pub fn marquee(&self) -> Option<Image> {
        self.images[3]
    }
    pub fn burst(&self) -> Option<Image> {
        self.images[4]
    }
}

/// Draws Watt's `frame` with the glass centred at the origin, glass radius
/// `r` units. False when the sheet isn't loaded yet.
pub fn host_frame(img: Option<Image>, frame: Frame, r: f32, tint: u32) -> bool {
    let Some(img) = img else { return false };
    let i = frame as usize;
    let (cw, ch) = HOST_CELL;
    let src = [(i % HOST_COLS) as f32 * cw, (i / HOST_COLS) as f32 * ch, cw, ch];
    let k = r / HOST_GLASS.2;
    gfx2d::sprite(img, src, [-HOST_GLASS.0 * k, -HOST_GLASS.1 * k, cw * k, ch * k], tint);
    true
}

/// Draws category emblem `cell` with its badge `size` units across.
pub fn emblem(img: Option<Image>, cell: usize, cx: f32, cy: f32, size: f32) -> bool {
    let Some(img) = img else { return false };
    let src = [(cell % CAT_COLS) as f32 * CAT_CELL, (cell / CAT_COLS) as f32 * CAT_CELL, CAT_CELL, CAT_CELL];
    let w = size * CAT_CELL / CAT_BADGE;
    gfx2d::sprite(img, src, [cx - w / 2.0, cy - w / 2.0, w, w], 0xffffffff);
    true
}

/// Draws a whole image into `dst`.
pub fn whole(img: Image, size: (f32, f32), dst: Rect, tint: u32) {
    gfx2d::sprite(img, [0.0, 0.0, size.0, size.1], [dst.x, dst.y, dst.w, dst.h], tint);
}
