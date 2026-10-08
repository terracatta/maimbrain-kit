//! Template: copy this directory to games/<name>, replace NAME in Cargo.toml
//! and manifest.toml, and keep the shape — pure rules in `sim`, a thin host
//! layer here (input, round state, saving, sound, drawing).
//!
//! The title card, HUD and game-over card come from the SDK's UI kit
//! (`maimbrain::ui`, docs/UI.md), the pops' juice from `maimbrain::juice`.
//! Everything takes its look from one `Theme`, composed from IDENTITY below:
//! don't ship the identity `mb new` picked at random. Choose the game's own
//! (the skill's art-direction step), bake its fonts with `mb font add`, and
//! draw the scene in its palette.

mod sim;

use maimbrain::juice::{Particles, Popups};
use maimbrain::motion::Shake;
use maimbrain::sys::{self, Round};
use maimbrain::ui::{Hud, Layout, ResultsAction, ResultsCard, Theme, TitleCard, shape, with_alpha};
use maimbrain::{Game, Rng, export_game, gfx2d, input, store};

use sim::Sim;

/// The game's identity: a kit identity (`riso`, `crt`, `neon`, `storybook`,
/// `saloon`, `swiss`, `brutalist`, `comic`, `gothic`, `bubblegum`, `terminal`,
/// `field`, `orbital`, `gala`, `notebook`, `stadium`, `cozy-pixel`) or a few
/// words ("1970s diner menu, mustard and teal, snappy") that compose one
/// (`Theme::from_identity`, docs/UI.md). Its fonts come from `mb font add`.
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

struct Bubble {
    mode: Mode,
    sim: Sim,
    rng: Rng,
    best: u32,
    time: f32,
    layout: Layout,
    theme: Theme,
    title: TitleCard,
    hud: Hud,
    results: Option<ResultsCard>,
    fx: Particles,
    popups: Popups,
    shake: Shake,
}

impl Bubble {
    fn start(&mut self) {
        self.sim = Sim::new(self.rng.next_u32() as u64);
        self.hud.reset(self.best as i64);
        self.results = None;
        self.mode = Mode::Play;
        sys::round(Round::Playing);
    }

    fn tap(&mut self, x: f32, y: f32) {
        let (bx, by) = (self.sim.x, self.sim.y);
        if self.sim.tap(x, y) {
            // Juice: a puff and a ring where it popped, "+1" floating up, a little shake.
            self.fx.burst(bx, by, 18, self.theme.accent);
            self.fx.ring(bx, by, 70.0, 0xffffffc0);
            self.popups.score(bx, by - 20.0, 1, self.theme.gold);
            self.hud.set_score(self.sim.score as i64);
            self.shake.add(0.25);
        }
    }
}

impl Game for Bubble {
    fn init() -> Self {
        sys::round(Round::Idle);
        let mut rng = Rng::from_host();
        let sim = Sim::new(rng.next_u32() as u64);
        let best = store::get_u64("best").unwrap_or(0) as u32;
        // The identity's look, in its library fonts (baked into assets/fonts/ by
        // `mb new` or `mb font add --identity`; until they load it draws in Inter).
        // Adjust any field after: theme.accent, theme.style.motion, theme.title_font…
        let theme = Theme::from_identity(IDENTITY).load_fonts();
        Bubble {
            mode: Mode::Title,
            sim,
            best,
            time: 0.0,
            layout: Layout::new(),
            theme,
            // The title is a feed card, an ad: a hook and a brag, never
            // instructions (it can't take input). How to play comes after the tap.
            title: TitleCard::new("BUBBLE", theme).tagline("don't let it pop").best(best as i64),
            hud: Hud::new(theme, best as i64),
            results: None,
            fx: Particles::new(rng.next_u32() as u64),
            popups: Popups::new(),
            shake: Shake::new(),
            rng,
        }
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        for e in input::poll() {
            if let (Mode::Over(t), Some(card)) = (&self.mode, &mut self.results) {
                // The card's retry button; any other tap retries too, after the guard.
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
                    self.tap(e.x, e.y);
                }
                Mode::Play => self.tap(e.x, e.y),
                Mode::Over(t) if t > RETRY_GUARD && !self.results.as_ref().is_some_and(|c| c.claims(e.x, e.y)) => self.start(),
                Mode::Over(_) => {}
            }
        }
        match &mut self.mode {
            Mode::Play => {
                self.sim.step(dt);
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
                    self.results = Some(ResultsCard::new(self.sim.score as i64, prev as i64, self.theme, &self.layout, seed).heading("POP!").label("SCORE"));
                    self.fx.burst(self.sim.x, self.sim.y, 30, self.theme.bad);
                    self.shake.add(0.6);
                    self.mode = Mode::Over(0.0);
                    sys::round(Round::Over);
                }
            }
            Mode::Over(t) => *t += dt,
            Mode::Title => {}
        }
        self.title.update(dt);
        self.hud.update(dt);
        if let Some(card) = &mut self.results {
            // Returns Tick / NewBest / Landed: hook sounds and haptics here.
            card.update(dt);
        }
        self.fx.update(dt);
        self.popups.update(dt);
        self.shake.update(dt);
    }

    fn render(&self) {
        let l = &self.layout;
        gfx2d::push();
        self.shake.apply(l.screen.cx(), l.screen.cy());
        let th = &self.theme;
        th.backdrop(l.screen, self.time);
        let s = &self.sim;
        let pulse = if self.mode == Mode::Title { 1.0 + 0.06 * (self.time * 6.0).sin() } else { 1.0 };
        if !s.over {
            // The scene in the identity's palette (draw your own world here).
            shape::soft_disc(s.x, s.y, s.r * pulse + 6.0, 14.0, with_alpha(th.accent, 0.3));
            shape::disc(s.x, s.y, s.r * pulse, th.accent);
            shape::soft_disc(s.x - s.r * 0.35, s.y - s.r * 0.35, s.r * 0.16, s.r * 0.12, 0xffffffb0);
        }
        self.fx.draw();
        gfx2d::pop();
        self.popups.draw();
        match self.mode {
            Mode::Title => self.title.draw(l),
            Mode::Play => {
                self.hud.draw(l);
                // Play teaches: an animated hint on the bubble until the first pop.
                if s.score == 0 {
                    let bob = (self.time * 5.0).sin() * 6.0;
                    maimbrain::ui::text(&th.case("Pop it!")).font(th.body_font).size(th.fit(th.body_font, 20.0)).color(th.text).outline(0.1, th.outline).middle().draw(s.x, s.y + s.r + 24.0 + bob);
                }
            }
            Mode::Over(_) => {
                if let Some(card) = &self.results {
                    card.draw();
                }
            }
        }
    }
}

export_game!(Bubble);
