//! 3D template: copy this directory to games/<name>, replace NAME in
//! Cargo.toml and manifest.toml (`mb new --3d <name>` does both), and keep
//! the shape: pure rules in `sim`, the scene built once in `scene`, and a
//! thin host layer here (input, round state, saving, moving nodes, the HUD).

mod scene;
mod sim;

use maimbrain::gfx2d::Font;
use maimbrain::gfx3d;
use maimbrain::sys::{self, Round};
use maimbrain::{Game, Rng, export_game, gfx2d, input, store};

use sim::{H, Sim, W};

const BOARD: u32 = 0;
/// Taps right after a round ends don't restart it (a frantic tap during the failure).
const RETRY_GUARD: f32 = 0.35;

#[derive(PartialEq)]
enum Mode {
    Title,
    Play,
    Over(f32),
}

struct Drone {
    mode: Mode,
    sim: Sim,
    scene: scene::Scene,
    rng: Rng,
    best: u32,
    /// Seconds since init: drives idle animation (a function of dt only).
    time: f32,
    /// When the last catch happened (game time), for the flash.
    caught_at: Option<f32>,
}

impl Drone {
    fn start(&mut self) {
        self.sim = Sim::new(self.rng.next_u32() as u64);
        self.mode = Mode::Play;
        sys::round(Round::Playing);
    }
}

impl Game for Drone {
    fn init() -> Self {
        sys::round(Round::Idle);
        let mut rng = Rng::from_host();
        let sim = Sim::new(rng.next_u32() as u64);
        Drone { mode: Mode::Title, sim, scene: scene::build(), rng, best: store::get_u64("best").unwrap_or(0) as u32, time: 0.0, caught_at: None }
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        for e in input::poll() {
            if !e.is_press() {
                continue;
            }
            match self.mode {
                // The tap that enters play is also the first move (SPEC §5.2).
                Mode::Title => {
                    self.start();
                    self.try_catch(e.x, e.y);
                }
                Mode::Play => self.try_catch(e.x, e.y),
                Mode::Over(t) if t > RETRY_GUARD => self.start(),
                Mode::Over(_) => {}
            }
        }
        match &mut self.mode {
            Mode::Play => {
                self.sim.step(dt);
                if self.sim.over {
                    // Submit before reporting Over: the platform saves the replay on Over.
                    store::submit_score(BOARD, self.sim.score as i64);
                    if self.sim.score > self.best {
                        self.best = self.sim.score;
                        store::set_u64("best", self.best as u64);
                    }
                    self.mode = Mode::Over(0.0);
                    sys::round(Round::Over);
                }
            }
            Mode::Over(t) => *t += dt,
            // The title card is live: let the drone fly its loop.
            Mode::Title => self.sim.phase += 0.6 * dt,
        }
        self.scene.place_drone(self.sim.drone(), self.time * 3.0);
        self.scene.flash_light(self.caught_at.map(|t| self.time - t));
    }

    fn render(&self) {
        gfx3d::camera(&self.sim.camera());
        gfx3d::render();

        // The HUD: mb2d draws on top of the 3D image, in the same logical
        // coordinates `gfx3d::project` returns.
        let s = &self.sim;
        let p = gfx3d::project(s.drone());
        match self.mode {
            Mode::Title => {
                // The title is a feed card, an ad: a hook and a brag, never
                // instructions (it can't take input). How to play comes after the tap.
                gfx2d::text_centered(Font::Pixel, 24.0, W / 2.0, 80.0, 0xffffffff, "CATCH THE DRONE");
                if self.best > 0 {
                    gfx2d::text_centered(Font::Pixel, 16.0, W / 2.0, 116.0, 0xffd23fff, &format!("BEST {}", self.best));
                }
            }
            _ => {
                gfx2d::text_centered(Font::Pixel, 32.0, W / 2.0, 60.0, 0xffffffff, &s.score.to_string());
                // The clock as a bar under the score.
                let k = s.clock / sim::START_CLOCK;
                gfx2d::rect(100.0, 104.0, 160.0, 6.0, 0xffffff30);
                gfx2d::rect(100.0, 104.0, 160.0 * k, 6.0, if k < 0.3 { 0xff5a5aff } else { 0x6fe0ffff });
                if self.mode == Mode::Play && p.on_screen {
                    // A reticle on the drone, placed by projecting its world position.
                    let r = sim::TAP_RADIUS * (0.95 + 0.08 * (self.time * 8.0).sin());
                    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                        let (cx, cy) = (p.x + dx * r, p.y + dy * r);
                        gfx2d::line(cx, cy, cx - dx * 14.0, cy, 3.0, 0xffffffe0);
                        gfx2d::line(cx, cy, cx, cy - dy * 14.0, 3.0, 0xffffffe0);
                    }
                    // Play teaches: a hint until the first catch.
                    if s.score == 0 {
                        gfx2d::text_centered(Font::Pixel, 16.0, p.x, p.y + r + 12.0, 0xffffffff, "TAP IT!");
                    }
                }
            }
        }
        if let Mode::Over(_) = self.mode {
            gfx2d::rect(0.0, 0.0, W, H, 0x00000070);
            gfx2d::text_centered(Font::Pixel, 32.0, W / 2.0, 250.0, 0xffffffff, "ESCAPED!");
            gfx2d::text_centered(Font::Pixel, 16.0, W / 2.0, 300.0, 0xffd23fff, &format!("BEST {}", self.best));
        }
    }
}

impl Drone {
    fn try_catch(&mut self, x: f32, y: f32) {
        let velocity = self.sim.drone_velocity();
        if let Some(at) = self.sim.tap(x, y) {
            self.scene.caught(at, velocity);
            self.caught_at = Some(self.time);
        }
    }
}

export_game!(Drone);
