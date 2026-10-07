//! Template: copy this directory to games/<name>, replace NAME in Cargo.toml
//! and manifest.toml, and keep the shape — pure rules in `sim`, a thin host
//! layer here (input, round state, saving, sound, drawing).

mod sim;

use maimbrain::gfx2d::Font;
use maimbrain::sys::{self, Round};
use maimbrain::{Game, Rng, export_game, gfx2d, input, store};

use sim::Sim;

const BOARD: u32 = 0;
/// Taps right after a round ends don't restart it (a frantic tap during the failure).
const RETRY_GUARD: f32 = 0.35;

#[derive(PartialEq)]
enum Mode {
    Title,
    Play,
    Over(f32),
}

struct Bubble {
    mode: Mode,
    sim: Sim,
    rng: Rng,
    best: u32,
    time: f32,
}

impl Bubble {
    fn start(&mut self) {
        self.sim = Sim::new(self.rng.next_u32() as u64);
        self.mode = Mode::Play;
        sys::round(Round::Playing);
    }
}

impl Game for Bubble {
    fn init() -> Self {
        sys::round(Round::Idle);
        let mut rng = Rng::from_host();
        let sim = Sim::new(rng.next_u32() as u64);
        Bubble { mode: Mode::Title, sim, rng, best: store::get_u64("best").unwrap_or(0) as u32, time: 0.0 }
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
                    self.sim.tap(e.x, e.y);
                }
                Mode::Play => {
                    self.sim.tap(e.x, e.y);
                }
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
            Mode::Title => {}
        }
    }

    fn render(&self) {
        gfx2d::rect_gradient(0.0, 0.0, 360.0, 640.0, 0x1b2a49ff, 0x0d1424ff);
        let s = &self.sim;
        let pulse = 1.0 + 0.06 * (self.time * 6.0).sin();
        gfx2d::circle(s.x, s.y, s.r * if self.mode == Mode::Title { pulse } else { 1.0 }, 0x6fd3ffd0);
        gfx2d::circle(s.x - s.r * 0.35, s.y - s.r * 0.35, s.r * 0.18, 0xffffff90);
        match self.mode {
            Mode::Title => {
                // The title is a feed card, an ad: a hook and a brag, never
                // instructions (it can't take input). How to play comes after the tap.
                gfx2d::text_centered(Font::Pixel, 24.0, 180.0, 80.0, 0xffffffff, "DON'T LET IT POP");
                if self.best > 0 {
                    gfx2d::text_centered(Font::Pixel, 16.0, 180.0, 116.0, 0xffd23fff, &format!("BEST {}", self.best));
                }
            }
            _ => {
                gfx2d::text_centered(Font::Pixel, 32.0, 180.0, 60.0, 0xffffffff, &s.score.to_string());
                // Play teaches: an animated hint on the bubble until the first pop.
                if self.mode == Mode::Play && s.score == 0 {
                    let bob = (self.time * 5.0).sin() * 6.0;
                    gfx2d::text_centered(Font::Pixel, 16.0, s.x, s.y + s.r + 20.0 + bob, 0xffffffff, "POP IT!");
                }
            }
        }
        if let Mode::Over(_) = self.mode {
            gfx2d::rect(0.0, 0.0, 360.0, 640.0, 0x00000080);
            gfx2d::text_centered(Font::Pixel, 32.0, 180.0, 250.0, 0xffffffff, "POP!");
            gfx2d::text_centered(Font::Pixel, 16.0, 180.0, 300.0, 0xffd23fff, &format!("BEST {}", self.best));
        }
    }
}

export_game!(Bubble);
