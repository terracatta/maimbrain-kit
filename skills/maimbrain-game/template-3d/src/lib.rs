//! 3D template: copy this directory to games/<name>, replace NAME in
//! Cargo.toml and manifest.toml (`mb new --3d <name>` does both), and keep
//! the shape: pure rules in `sim`, the scene built once in `scene`, and a
//! thin host layer here (input, round state, saving, moving nodes, the HUD).
//!
//! The HUD, title card and game-over card are the SDK's UI kit
//! (`maimbrain::ui`, docs/UI.md) drawn with mb2d over the 3D image, in the
//! look of IDENTITY below: replace the one `mb new` picked at random with the
//! game's own (the skill's art-direction step) and match the scene to it.

mod scene;
mod sim;

use maimbrain::gfx3d;
use maimbrain::juice::Popups;
use maimbrain::sys::{self, Round};
use maimbrain::ui::{Hud, Layout, ResultsAction, ResultsCard, Theme, TitleCard, shape, text, with_alpha};
use maimbrain::{Game, Rng, export_game, gfx2d, input, store};

use sim::{H, Sim, W};

/// The game's identity: a kit identity name or a few words composing one
/// (`Theme::from_identity`, docs/UI.md); `mb font add --identity` bakes its fonts.
const IDENTITY: &str = "__IDENTITY__";
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
    layout: Layout,
    theme: Theme,
    title: TitleCard,
    hud: Hud,
    results: Option<ResultsCard>,
    popups: Popups,
}

impl Drone {
    fn start(&mut self) {
        self.sim = Sim::new(self.rng.next_u32() as u64);
        self.hud.reset(self.best as i64);
        self.results = None;
        self.mode = Mode::Play;
        sys::round(Round::Playing);
    }
}

impl Game for Drone {
    fn init() -> Self {
        sys::round(Round::Idle);
        let mut rng = Rng::from_host();
        let sim = Sim::new(rng.next_u32() as u64);
        let best = store::get_u64("best").unwrap_or(0) as u32;
        let theme = Theme::from_identity(IDENTITY).load_fonts();
        Drone {
            mode: Mode::Title,
            sim,
            scene: scene::build(),
            rng,
            best,
            time: 0.0,
            caught_at: None,
            layout: Layout::new(),
            theme,
            // The title is a feed card, an ad: a hook and a brag, never
            // instructions (it can't take input). How to play comes after the tap.
            title: TitleCard::new("DRONE", theme).tagline("catch it if you can").best(best as i64),
            hud: Hud::new(theme, best as i64),
            results: None,
            popups: Popups::new(),
        }
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        for e in input::poll() {
            if let (Mode::Over(t), Some(card)) = (&self.mode, &mut self.results) {
                let retry = *t > RETRY_GUARD && card.handle(&e) == Some(ResultsAction::Retry);
                if retry {
                    self.start();
                    continue;
                }
            }
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
                Mode::Over(t) if t > RETRY_GUARD && !self.results.as_ref().is_some_and(|c| c.claims(e.x, e.y)) => self.start(),
                Mode::Over(_) => {}
            }
        }
        match &mut self.mode {
            Mode::Play => {
                self.sim.step(dt);
                self.hud.progress = Some(self.sim.clock / sim::START_CLOCK);
                if self.sim.over {
                    // Submit before reporting Over: the platform saves the replay on Over.
                    store::submit_score(BOARD, self.sim.score as i64);
                    let prev = self.best;
                    if self.sim.score > self.best {
                        self.best = self.sim.score;
                        store::set_u64("best", self.best as u64);
                    }
                    self.title.set_best(self.best as i64);
                    let seed = self.rng.next_u32() as u64;
                    self.results = Some(ResultsCard::new(self.sim.score as i64, prev as i64, self.theme, &self.layout, seed).heading("ESCAPED!").label("CAUGHT"));
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
        self.title.update(dt);
        self.hud.update(dt);
        self.popups.update(dt);
        if let Some(card) = &mut self.results {
            card.update(dt);
        }
    }

    fn render(&self) {
        gfx3d::camera(&self.sim.camera());
        gfx3d::render();

        // The HUD: mb2d draws on top of the 3D image, in the same logical
        // coordinates `gfx3d::project` returns.
        let s = &self.sim;
        let p = gfx3d::project(s.drone());
        match self.mode {
            Mode::Title => self.title.draw(&self.layout),
            Mode::Play => {
                // The clock is the HUD's progress bar; red when it runs low.
                self.hud.draw(&self.layout);
                if p.on_screen {
                    // A reticle on the drone, placed by projecting its world position.
                    let r = sim::TAP_RADIUS * (0.95 + 0.08 * (self.time * 8.0).sin());
                    let col = if s.clock / sim::START_CLOCK < 0.3 { self.theme.bad } else { 0xffffffe0 };
                    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                        let (cx, cy) = (p.x + dx * r, p.y + dy * r);
                        shape::capsule(cx, cy, cx - dx * 14.0, cy, 3.5, col);
                        shape::capsule(cx, cy, cx, cy - dy * 14.0, 3.5, col);
                    }
                    // Play teaches: a hint until the first catch.
                    if s.score == 0 {
                        text("TAP IT!").size(20.0).outline(0.1, self.theme.outline).middle().draw(p.x, p.y + r + 22.0);
                    }
                }
            }
            Mode::Over(_) => {
                gfx2d::rect(0.0, 0.0, W, H, with_alpha(self.theme.outline, 0.35));
                if let Some(card) = &self.results {
                    card.draw();
                }
            }
        }
        self.popups.draw();
    }
}

impl Drone {
    fn try_catch(&mut self, x: f32, y: f32) {
        let velocity = self.sim.drone_velocity();
        if let Some(at) = self.sim.tap(x, y) {
            self.scene.caught(at, velocity);
            self.caught_at = Some(self.time);
            self.hud.set_score(self.sim.score as i64);
            // `Camera::project` is plain Rust, so this works in update (and in tests).
            let q = self.sim.camera().project(at, W, H);
            self.popups.score(q.x, q.y - 30.0, 1, self.theme.gold);
        }
    }
}

export_game!(Drone);
