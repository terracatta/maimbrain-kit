//! Picnic Pile: a bird flies back and forth over a plate balanced on a pole,
//! carrying snacks with faces. Tap to let go. Everything is real rigid-body
//! physics (`maimbrain::physics2d`): snacks land, wobble, wedge and tumble.
//! Three tumbles and lunch is over. Wind gusts and a tilting plate join in.
//!
//! The stacker starter kit. Layout:
//! - `sim.rs`: rules + physics + tuning knobs (tested, no host calls)
//! - `look.rs`: names, words, colors, the UI theme (what a reskin changes)
//! - `draw.rs`: the world (snacks, faces, bird, plate, ghost) from the art
//! - `sprites.rs`: source rects in assets/sprites.png (made by `mb art atlas`)
//! - `sound.rs`: sounds, music stems and haptics from the sim's cues
//! - this file: rounds, input, cards, HUD and juice
//!
//! Daily mode plays the same snacks and twists for everyone today.

mod draw;
mod look;
mod sim;
mod sound;
mod sprites;
#[cfg(test)]
mod tests;

use maimbrain::input::{self, Event};
use maimbrain::juice::{Particles, Popups, VignettePulse};
use maimbrain::motion::{Ease, HitStop, Shake};
use maimbrain::physics2d::vec2;
use maimbrain::sys::{self, Round};
use maimbrain::ui::{self, Button, ButtonKind, Hud, Icon, Layout, Rect, ResultsAction, ResultsCard, ResultsEvent, Tap, Theme, TitleCard, text, with_alpha};
use maimbrain::{Game, Rng, export_game, gfx2d, store};

use sim::{Cue, LIVES, Sim, TwistKind};

const BOARD: u32 = 0;
const BOARD_DAILY: u32 = 1;
/// Taps right after a round ends don't restart it (a frantic tap during the fall).
const RETRY_GUARD: f32 = 0.35;
/// The how-to hint comes back after this long without a tap.
const HINT_IDLE: f32 = 3.0;
/// The title card's own tower starts over after this long (or when it falls).
const ATTRACT_ROUND: f32 = 40.0;
/// Debug: the autopilot plays the rounds too (to reach late states for screenshots).
const DEBUG_AUTOPLAY: bool = false;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// The feed card: the bird stacks by itself, a little sloppily.
    Title,
    Play,
    /// Seconds since the round ended.
    Over(f32),
}

/// A big word in the middle of the screen for a moment ("WIND!", "NEW: JELLY!").
struct Banner {
    text: String,
    color: u32,
    icon: Option<Icon>,
    t: f32,
}

struct Picnic {
    mode: Mode,
    sim: Sim,
    rng: Rng,
    daily: bool,
    best: u64,
    best_daily: u64,
    time: f32,
    /// Seconds since the player last dropped (for the hint).
    idle: f32,
    layout: Layout,
    theme: Theme,
    title: TitleCard,
    hud: Hud,
    results: Option<ResultsCard>,
    daily_btn: Button,
    scores_btn: Button,
    fx: Particles,
    popups: Popups,
    shake: Shake,
    hitstop: HitStop,
    hurt: VignettePulse,
    banner: Option<Banner>,
    art: draw::Art,
    sound: sound::Sound,
}

impl Game for Picnic {
    fn init() -> Self {
        sys::round(Round::Idle);
        let mut rng = Rng::from_host();
        let layout = Layout::new();
        let theme = look::theme();
        // DAILY (and SCORES after a round) sit low in the card area, clear of
        // the feed's overlays, at its edges so the tower stays in view.
        let row = [Rect::new(layout.card.x, layout.card.bottom() - 46.0, 112.0, 44.0), Rect::new(layout.card.right() - 112.0, layout.card.bottom() - 46.0, 112.0, 44.0)];
        let mut g = Picnic {
            mode: Mode::Title,
            sim: Sim::new(rng.next_u32() as u64),
            daily: false,
            best: store::get_u64("best").unwrap_or(0),
            best_daily: 0,
            time: 0.0,
            idle: 0.0,
            title: TitleCard::new(look::TITLE, theme).tagline(look::TAGLINE).size(52.0),
            hud: Hud::new(theme, 0),
            results: None,
            daily_btn: Button::new(row[1], "DAILY").kind(ButtonKind::Secondary).with_icon(Icon::STAR),
            scores_btn: Button::new(row[0], "SCORES").kind(ButtonKind::Secondary).with_icon(Icon::TROPHY),
            fx: Particles::new(rng.next_u32() as u64),
            popups: Popups::new(),
            shake: Shake::new(),
            hitstop: HitStop::default(),
            hurt: VignettePulse::new(look::DANGER),
            banner: None,
            art: draw::Art::new(),
            sound: sound::Sound::new(),
            layout,
            theme,
            rng,
        };
        g.best_daily = store::get_u64(&g.daily_key()).unwrap_or(0);
        g.title.set_best(g.current_best() as i64);
        g.hud.lives = Some((LIVES, LIVES));
        g
    }

    fn update(&mut self, dt: f32) {
        self.art.update();
        self.time += dt;
        self.idle += dt;
        for e in input::poll() {
            if !self.ui_event(&e) && e.is_press() {
                self.tap();
            }
        }
        if self.mode == Mode::Play && DEBUG_AUTOPLAY {
            self.autopilot();
        }
        if self.mode == Mode::Title {
            self.autopilot();
            if self.sim.over || self.sim.t > ATTRACT_ROUND {
                self.sim = Sim::new(self.rng.next_u32() as u64);
            }
        }
        // A hit-stop freezes the physics for a moment when a heart goes.
        self.sim.step(self.hitstop.step(dt));
        let cues = std::mem::take(&mut self.sim.cues);
        self.juice(&cues);
        self.sound.update(dt, &cues, self.mode != Mode::Title, &self.sim);
        match &mut self.mode {
            Mode::Play if self.sim.over => self.game_over(),
            Mode::Over(t) => *t += dt,
            _ => {}
        }
        self.title.update(dt);
        self.hud.lives = Some((self.sim.lives, LIVES));
        self.hud.update(dt);
        if let Some(card) = &mut self.results
            && card.update(dt) == Some(ResultsEvent::NewBest)
        {
            self.sound.best();
        }
        self.daily_btn.kind = if self.daily { ButtonKind::Primary } else { ButtonKind::Secondary };
        self.daily_btn.update(dt);
        self.scores_btn.update(dt);
        self.fx.update(dt);
        self.popups.update(dt);
        self.shake.update(dt);
        self.hurt.level = if self.mode == Mode::Play && self.sim.lives == 1 { 0.22 + 0.08 * (self.time * 4.0).sin() } else { 0.0 };
        self.hurt.update(dt);
        if let Some(b) = &mut self.banner {
            b.t += dt;
            if b.t > 1.6 {
                self.banner = None;
            }
        }
    }

    fn render(&self) {
        let l = &self.layout;
        draw::sky(&self.art, &self.sim, self.time);
        gfx2d::push();
        self.shake.apply(l.screen.cx(), l.screen.cy());
        gfx2d::translate(0.0, -self.sim.cam_y);
        draw::world(&self.art, &self.sim, self.time, !matches!(self.mode, Mode::Over(_)) && self.sim.ending.is_none());
        if self.show_hint() {
            self.hint_ring();
        }
        self.fx.draw();
        self.popups.draw();
        gfx2d::pop();
        self.hurt.draw(l.screen);
        match self.mode {
            Mode::Title => {
                // The card sells (a name, a hook, the best); how to play waits for play.
                self.title.draw(l);
                self.daily_btn.draw(&self.theme);
            }
            Mode::Play => {
                self.hud.draw(l);
                if self.daily {
                    // Under the pause pill, clear of the score.
                    let z = l.pill_zone();
                    ui::pill("DAILY", z.x + 52.0, z.bottom() + 18.0, 22.0, with_alpha(self.theme.outline, 0.5), 0xffffffff, self.theme.body_font, Some(Icon::STAR));
                }
                self.draw_banner();
                if self.show_hint() {
                    let b = self.sim.bird();
                    let a = 0.75 + 0.25 * (self.time * 6.0).sin();
                    text(look::HINT).size(22.0).color(0xffffffff).outline(0.1, self.theme.outline).alpha(a).middle().draw(l.screen.cx(), b.y - self.sim.cam_y - 50.0);
                }
            }
            Mode::Over(_) => {
                if let Some(card) = &self.results {
                    card.draw();
                }
                self.scores_btn.draw(&self.theme);
                self.daily_btn.draw(&self.theme);
            }
        }
    }

    fn suspend(&mut self) {
        self.sound.pause();
    }
}

impl Picnic {
    fn daily_key(&self) -> String {
        format!("daily:{:016x}", sys::daily_seed())
    }

    fn current_best(&self) -> u64 {
        if self.daily { self.best_daily } else { self.best }
    }

    fn start(&mut self) {
        let seed = if self.daily { sys::daily_seed() } else { self.rng.next_u32() as u64 };
        self.sim = Sim::new(seed);
        self.hud.reset(self.current_best() as i64);
        self.results = None;
        self.banner = None;
        self.popups.list.clear();
        self.mode = Mode::Play;
        self.idle = 0.0;
        sys::round(Round::Playing);
        self.sound.start();
        // The tap that starts the round is the first drop: the first snack
        // is a slice of toast over the middle of a still bird, so it lands.
        self.sim.drop_piece();
    }

    fn tap(&mut self) {
        match self.mode {
            Mode::Title => self.start(),
            Mode::Play => {
                if self.sim.drop_piece() {
                    self.idle = 0.0;
                }
            }
            Mode::Over(t) if t > RETRY_GUARD => self.start(),
            Mode::Over(_) => {}
        }
    }

    /// Offers an event to the on-screen buttons; true if one took it.
    fn ui_event(&mut self, e: &Event) -> bool {
        match self.mode {
            Mode::Play => false,
            Mode::Title => match self.daily_btn.handle(e) {
                Some(Tap::Clicked) => {
                    self.toggle_daily();
                    true
                }
                Some(_) => true,
                None => false,
            },
            Mode::Over(t) => {
                if t <= RETRY_GUARD {
                    return true; // a frantic tap during the fall
                }
                if let Some(tap) = self.daily_btn.handle(e) {
                    if tap == Tap::Clicked {
                        self.toggle_daily();
                    }
                    return true;
                }
                if let Some(tap) = self.scores_btn.handle(e) {
                    if tap == Tap::Clicked {
                        self.sound.ui(false);
                        store::show_scores(if self.daily { BOARD_DAILY } else { BOARD });
                    }
                    return true;
                }
                let action = self.results.as_mut().and_then(|r| r.handle(e));
                if action == Some(ResultsAction::Retry) {
                    self.start();
                    return true;
                }
                self.results.as_ref().is_some_and(|r| r.claims(e.x, e.y))
            }
        }
    }

    fn toggle_daily(&mut self) {
        self.daily = !self.daily;
        self.sound.ui(self.daily);
        self.title.set_best(self.current_best() as i64);
        if self.mode != Mode::Title {
            self.mode = Mode::Title;
            self.results = None;
            self.sim = Sim::new(self.rng.next_u32() as u64);
            self.sound.stop_music();
            sys::round(Round::Idle);
        }
    }

    fn game_over(&mut self) {
        let score = self.sim.score as u64;
        let prev = self.current_best();
        // Submit before reporting Over: the platform saves the replay on Over.
        if self.daily {
            store::submit_score(BOARD_DAILY, score as i64);
            if score > self.best_daily {
                self.best_daily = score;
                store::set_u64(&self.daily_key(), score);
            }
        } else {
            store::submit_score(BOARD, score as i64);
            if score > self.best {
                self.best = score;
                store::set_u64("best", score);
            }
        }
        self.title.set_best(self.current_best() as i64);
        let label = if self.daily { look::DAILY_LABEL } else { look::SCORE_LABEL };
        let seed = self.rng.next_u32() as u64;
        self.results = Some(ResultsCard::new(score as i64, prev as i64, self.theme, &self.layout, seed).heading(look::OVER_HEADING).label(label).floating());
        self.daily_btn.reset();
        self.scores_btn.reset();
        self.mode = Mode::Over(0.0);
        sys::round(Round::Over);
    }

    /// The title card's bird stacks by itself: mostly neat drops, every
    /// fifth one sloppy, so the card wobbles and now and then tumbles.
    /// Deterministic (it reads only the sim).
    fn autopilot(&mut self) {
        let Some(held) = self.sim.held else { return };
        if held.age < 0.7 {
            return;
        }
        let Some(pred) = self.sim.predict() else { return };
        let slack = if self.sim.drops % 5 == 4 { 24.0 } else { 5.0 };
        if (pred.land.x - self.sim.tower_top().x).abs() < slack {
            self.sim.drop_piece();
        }
    }

    /// Until the player has dropped a few themselves, and whenever they stall.
    fn show_hint(&self) -> bool {
        self.mode == Mode::Play && self.sim.ending.is_none() && self.sim.held.is_some_and(|h| h.age > 0.4) && (self.sim.drops < 3 || self.idle > HINT_IDLE)
    }

    /// A ring pulsing out of the carried snack (in world space).
    fn hint_ring(&self) {
        let Some(p) = self.sim.held_pos() else { return };
        let k = (self.time * 1.4).fract();
        let r = 18.0 + Ease::CubicOut.at(k) * 34.0;
        ui::shape::ring(p.x, p.y, r, 3.0, with_alpha(0xffffffff, 0.8 * (1.0 - k)));
    }

    fn set_banner(&mut self, text: String, color: u32, icon: Option<Icon>) {
        self.banner = Some(Banner { text, color, icon, t: 0.0 });
    }

    fn draw_banner(&self) {
        let Some(b) = &self.banner else { return };
        let pop = Ease::BackOut.at((b.t / 0.25).min(1.0));
        let a = if b.t > 1.3 { 1.0 - (b.t - 1.3) / 0.3 } else { 1.0 };
        let (cx, cy) = (self.layout.screen.cx(), self.layout.hud.y + 128.0);
        gfx2d::push();
        gfx2d::translate(cx, cy);
        gfx2d::scale(pop.max(0.01), pop.max(0.01));
        let r = text(&b.text).size(30.0).color(b.color).outline(0.1, self.theme.outline).soft_shadow(0.0, 3.0, 0.06, self.theme.shadow).alpha(a).middle().draw(0.0, 0.0);
        if let Some(icon) = b.icon {
            let wob = (self.time * 10.0).sin() * 3.0;
            icon.draw(r.right() + 22.0 + wob, 0.0, 26.0, with_alpha(b.color, a));
        }
        gfx2d::pop();
    }

    /// Particles, popups, shake and banners from the sim's cues.
    fn juice(&mut self, cues: &[Cue]) {
        let playing = self.mode == Mode::Play;
        let th = self.theme;
        for &cue in cues {
            match cue {
                Cue::Drop => {
                    let b = self.sim.bird();
                    self.fx.burst(b.x, b.y + 16.0, 4, 0xffffffa0);
                }
                Cue::Thud { x, y, vol, heavy } => {
                    self.fx.burst(x, y, if heavy { 7 } else { 3 }, 0xfff3dcb0);
                    if heavy {
                        self.shake.add(0.12 + 0.1 * vol);
                    }
                }
                Cue::Land { x, y, streak, neat, close } if playing => {
                    self.hud.set_score(self.sim.score as i64);
                    if close {
                        self.popups.spawn(x, y - 34.0, look::CLOSE, 0xffffffff, 22.0);
                        self.fx.burst(x, y, 6, 0xfff3dcb0);
                    } else if neat {
                        self.popups.spawn(x, y - 34.0, look::NEAT, th.gold, 24.0);
                        self.fx.sparkles(x, y, 40.0, 10, 0xfff2b0ff);
                    } else {
                        self.popups.score(x, y - 30.0, 1, 0xffffffff);
                        self.fx.sparkles(x, y, 30.0, 5, 0xfff2b0ff);
                    }
                    if streak >= 5 && streak.is_multiple_of(5) {
                        self.popups.spawn(x, y - 62.0, &format!("{streak} IN A ROW!"), th.gold, 20.0);
                        self.fx.stars(x, y, 8, th.gold);
                    }
                }
                Cue::Lost { x, y, counted: true, landed, .. } if playing => {
                    // Show it on screen even if it fell out of view.
                    let (x, y) = (x.clamp(80.0, 280.0), y.clamp(self.sim.cam_y + 140.0, self.sim.cam_y + 540.0));
                    self.popups.spawn(x, y, if landed { look::TUMBLE } else { look::MISS }, th.bad, 26.0);
                    self.fx.burst(x, y, 14, th.bad);
                    self.shake.add(0.45);
                    self.hitstop.freeze(0.07);
                    self.hurt.fire();
                    if self.sim.lives == 1 {
                        self.set_banner(look::LAST_HEART.into(), th.bad, Some(Icon::HEART));
                    }
                }
                Cue::HeartBack if playing => {
                    self.set_banner(look::HEART_BACK.into(), th.bad, Some(Icon::HEART));
                    let b = self.sim.bird();
                    self.fx.stars(b.x, b.y, 8, th.bad);
                }
                Cue::NewKind(kind) if playing => {
                    self.set_banner(format!("NEW: {}!", look::snack(kind).name), th.gold, None);
                    if let Some(p) = self.sim.held_pos() {
                        self.fx.sparkles(p.x, p.y, 50.0, 14, 0xfff2b0ff);
                    }
                }
                Cue::Warn(kind, dir) if playing => {
                    let (word, icon) = match kind {
                        TwistKind::Gust => (look::GUST, if dir > 0.0 { Icon::ARROW_RIGHT } else { Icon::ARROW_LEFT }),
                        TwistKind::Tilt => (look::TILT, if dir > 0.0 { Icon::ARROW_RIGHT } else { Icon::ARROW_LEFT }),
                    };
                    self.set_banner(word.into(), 0xffffffff, Some(icon));
                }
                Cue::Collapse if playing => {
                    self.shake.add(0.8);
                    let p = vec2(sim::W / 2.0, sim::PLATE_TOP);
                    self.fx.burst(p.x, p.y, 20, 0xfff3dcd0);
                }
                _ => {}
            }
        }
    }
}

export_game!(Picnic);
