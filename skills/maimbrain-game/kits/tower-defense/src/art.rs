//! The generated art: one PNG per sheet in assets/ (each made by `mb art`,
//! with its `.gen.json` recipe beside it; tools/regen.sh remakes them all).
//! A sheet is one row of equal cells, bottom-anchored, so frame `i` of a
//! sheet is the cell at `i × cell width`. Everything is loaded at startup
//! (manifest `startup_assets`), so it's ready on the first frame.

use maimbrain::gfx2d::{self, Blend, Image};
use maimbrain::sys::{Asset, AssetState};

use crate::sim::{CreepKind, TowerKind};

/// One sheet: its file, its cell in texels and how many frames it has.
pub struct Sheet {
    pub file: &'static str,
    pub cell: (f32, f32),
    pub frames: u32,
}

pub const MAP: usize = 0;
pub const PAD: usize = 1;
pub const CAKE: usize = 2;
pub const BOMB: usize = 3;
/// Towers: `TOWER + kind`, three frames (levels 1–3).
pub const TOWER: usize = 4;
/// Jellies: `JELLY + kind`, walk A, walk B, pop (the gummy also has a hit
/// squash and a smug, full face: `GUMMY_HIT`, `GUMMY_SMUG`).
pub const JELLY: usize = 7;

pub const SHEETS: [Sheet; 13] = [
    Sheet { file: "assets/map.png", cell: (540.0, 960.0), frames: 1 },
    Sheet { file: "assets/pad.png", cell: (112.0, 112.0), frames: 1 },
    Sheet { file: "assets/cake.png", cell: (176.0, 176.0), frames: 5 },
    Sheet { file: "assets/bomb.png", cell: (48.0, 48.0), frames: 1 },
    Sheet { file: "assets/tower_pop.png", cell: (128.0, 128.0), frames: 3 },
    Sheet { file: "assets/tower_chill.png", cell: (128.0, 128.0), frames: 3 },
    Sheet { file: "assets/tower_boom.png", cell: (128.0, 128.0), frames: 3 },
    Sheet { file: "assets/jelly_gummy.png", cell: (96.0, 96.0), frames: 5 },
    Sheet { file: "assets/jelly_zipper.png", cell: (80.0, 80.0), frames: 3 },
    Sheet { file: "assets/jelly_helmet.png", cell: (96.0, 96.0), frames: 3 },
    Sheet { file: "assets/jelly_splitter.png", cell: (112.0, 112.0), frames: 3 },
    Sheet { file: "assets/jelly_mini.png", cell: (64.0, 64.0), frames: 3 },
    Sheet { file: "assets/jelly_king.png", cell: (144.0, 144.0), frames: 3 },
];

/// Which frame of a tower sheet shows each level, per `TowerKind`. `mb art
/// frames` sorts frames by rows it expects, and a tall last frame can land
/// first: look at art/preview/tower_*.png after regenerating and fix these.
pub const TOWER_LEVEL_FRAME: [[u32; 3]; 3] = [[1, 2, 0], [0, 1, 2], [0, 1, 2]];

/// Cake moods: frames of the cake sheet.
pub const CAKE_HAPPY: u32 = 0;
pub const CAKE_NERVOUS: u32 = 1;
pub const CAKE_PANIC: u32 = 2;
pub const CAKE_WINCE: u32 = 3;
pub const CAKE_GONE: u32 = 4;
pub const GUMMY_HIT: u32 = 2;
pub const GUMMY_SMUG: u32 = 4;

/// The pop frame of a jelly kind's sheet.
pub fn pop_frame(kind: CreepKind) -> u32 {
    if kind == CreepKind::Gummy { 3 } else { 2 }
}

/// How wide a jelly's cell is drawn, in units per unit of its radius: the
/// walking body fills about 60% of the cell (the pop frame sets the scale).
pub const JELLY_CELL_PER_R: [f32; 6] = [4.0, 4.4, 4.0, 3.8, 4.4, 3.6];

pub fn tower_sheet(kind: TowerKind) -> usize {
    TOWER + kind as usize
}

pub fn jelly_sheet(kind: CreepKind) -> usize {
    JELLY + kind as usize
}

pub struct Art {
    assets: Vec<Asset>,
    images: Vec<Option<Image>>,
}

impl Art {
    pub fn new() -> Art {
        Art { assets: SHEETS.iter().map(|s| Asset::load(s.file)).collect(), images: vec![None; SHEETS.len()] }
    }

    /// Picks up sheets as they finish loading (call once per update).
    pub fn poll(&mut self) {
        for (i, a) in self.assets.iter().enumerate() {
            if self.images[i].is_none() && a.state() == AssetState::Ready {
                self.images[i] = Image::new(*a);
            }
        }
    }

    pub fn ready(&self, sheet: usize) -> bool {
        self.images[sheet].is_some()
    }

    /// Draws frame `i` of `sheet` with its bottom centre at (x, y), `w` units
    /// wide (height follows the cell), mirrored when `flip`, tinted.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, sheet: usize, i: u32, x: f32, y: f32, w: f32, flip: bool, tint: u32) {
        let Some(img) = self.images[sheet] else { return };
        let s = &SHEETS[sheet];
        let h = w * s.cell.1 / s.cell.0;
        let src = [(i % s.frames) as f32 * s.cell.0, 0.0, s.cell.0, s.cell.1];
        if flip {
            gfx2d::push();
            gfx2d::translate(x, y);
            gfx2d::scale(-1.0, 1.0);
            gfx2d::sprite(img, src, [-w / 2.0, -h, w, h], tint);
            gfx2d::pop();
        } else {
            gfx2d::sprite(img, src, [x - w / 2.0, y - h, w, h], tint);
        }
    }

    /// The same frame again, added on top in `rgb` (0xRRGGBB00) at `amount`
    /// (0…1): a white hit flash, a frosty blue sheen.
    #[allow(clippy::too_many_arguments)]
    pub fn glow(&self, sheet: usize, i: u32, x: f32, y: f32, w: f32, flip: bool, rgb: u32, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        gfx2d::blend(Blend::Add);
        let a = ((amount.clamp(0.0, 1.0) * 200.0) as u32).min(255);
        self.draw(sheet, i, x, y, w, flip, (rgb & 0xffffff00) | a);
        gfx2d::blend(Blend::Alpha);
    }

    /// The whole map, stretched over the 360 × 640 board.
    pub fn map(&self) {
        let Some(img) = self.images[MAP] else { return };
        let s = &SHEETS[MAP];
        gfx2d::sprite(img, [0.0, 0.0, s.cell.0, s.cell.1], [0.0, 0.0, 360.0, 640.0], 0xffffffff);
    }
}
