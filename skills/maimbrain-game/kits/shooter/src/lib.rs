//! Nova Pip: a vertical arcade shooter for one thumb. Drag anywhere to fly
//! (the ship moves with the thumb, from where it is, never jumping to it);
//! it fires on its own. Waves of jellies, swoopers, darters and bulbs, a
//! boss with three attacks near the minute mark, then round again, faster.
//! Three hearts. Daily mode: the same waves for everyone today.
//!
//! Layout: `sim` (rules, tuning knobs at the top), `waves` (formations and
//! the schedule), `bot` (a thumb that plays like a person: the title card's
//! attract mode and the difficulty tests), `draw`, `sound`, `look` (palette,
//! theme, strings), `sprites` (the atlas's source rects, written by
//! `mb art atlas`), and this file: rounds, input, saving, juice.

mod bot;
mod draw;
mod look;
mod sim;
mod sound;
#[rustfmt::skip]
mod sprites;
mod waves;
#[cfg(test)]
mod tests;

use std::f32::consts::{FRAC_PI_2, PI};

use maimbrain::input::{self, Event, Kind as Input};
use maimbrain::juice::{Flash, Particles, Popups, VignettePulse};
use maimbrain::motion::{HitStop, Punch, Shake};
use maimbrain::sys::{self, Level, Round};
use maimbrain::ui::{Button, ButtonKind, Hud, Icon, Layout, Rect, ResultsAction, ResultsCard, ResultsEvent, Tap, Theme, TitleCard, fmt_int, lighten, with_alpha};
use maimbrain::{Game, Rng, export_game, store};

use bot::Bot;
use sim::{Cue, Kind, LIVES, Sim};

const BOARD: u32 = 0;
const BOARD_DAILY: u32 = 1;
/// Taps right after a round ends don't restart it (a frantic tap during the death).
const RETRY_GUARD: f32 = 0.35;
/// The drag hint comes back after this long without the thumb moving.
const HINT_IDLE: f32 = 3.0;
/// The hint shows until the thumb has dragged this far (units) in a round.
const HINT_UNTIL: f32 = 60.0;
/// Debug: start each round this many seconds in (e.g. `sim::BOSS_AT - 3.0`
/// to look at the boss). Keep at 0 to ship.
const DEBUG_SKIP_TO: f32 = 0.0;
/// The title card's attract run starts this far into a round (bullets already
/// flying), is pre-run for `ATTRACT_WARMUP` seconds so the very first frame is
/// mid-action, and restarts after `ATTRACT_SECONDS` (it reaches the boss).
const ATTRACT_FROM: f32 = 12.0;
const ATTRACT_WARMUP: f32 = 3.0;
const ATTRACT_SECONDS: f32 = 75.0;

/// A fresh attract run for the title card.
fn attract(rng: &mut Rng) -> (Sim, Bot) {
    let mut sim = Sim::demo(rng.next_u32() as u64);
    let mut bot = Bot::new(bot::GOOD, rng.next_u32() as u64);
    sim.skip_to(ATTRACT_FROM);
    for _ in 0..(ATTRACT_WARMUP * 60.0) as u32 {
        bot.drive(&mut sim, 1.0 / 60.0);
        sim.step(1.0 / 60.0);
    }
    sim.cues.clear();
    (sim, bot)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// The feed card: Pip flies itself (the bot) through the waves.
    Title,
    Play,
    /// Seconds since the round ended.
    Over(f32),
}

pub struct Shooter {
    pub mode: Mode,
    pub daily: bool,
    pub sim: Sim,
    demo_bot: Bot,
    rng: Rng,
    best: u64,
    best_daily: u64,
    /// Host time, for animation (stars, blinks); advances with `dt`.
    pub time: f32,
    /// Seconds since the thumb last moved, and how far it has dragged this round.
    pub idle: f32,
    pub moved: f32,
    /// The steering touch: (id, last x, last y).
    finger: Option<(u8, f32, f32)>,
    pub layout: Layout,
    pub theme: Theme,
    pub title: TitleCard,
    pub hud: Hud,
    pub results: Option<ResultsCard>,
    pub daily_btn: Button,
    pub scores_btn: Button,
    pub fx: Particles,
    pub popups: Popups,
    pub shake: Shake,
    hitstop: HitStop,
    pub flash: Flash,
    pub hurt: VignettePulse,
    pub mult_punch: Punch,
    /// Pip perks up when the thumb touches down (a response the same frame).
    pub ship_punch: Punch,
    graze_popup: f32,
    sound: sound::Sound,
    /// The generated sprite atlas and backdrop, once decoded.
    pub art: draw::Art,
}

impl Game for Shooter {
    fn init() -> Self {
        sys::round(Round::Idle);
        let layout = Layout::new();
        let theme = look::theme();
        let mut rng = Rng::from_host();
        let best = store::get_u64("best").unwrap_or(0);
        let best_daily = store::get_u64(&daily_key()).unwrap_or(0);
        // DAILY on the title card, SCORES and DAILY on the results card: low in the card area.
        let row = Rect::new(layout.card.x, layout.card.bottom() - 46.0, layout.card.w, 44.0).columns(2, 12.0);
        let (sim, demo_bot) = attract(&mut rng);
        let mut g = Shooter {
            mode: Mode::Title,
            daily: false,
            sim,
            demo_bot,
            best,
            best_daily,
            time: 0.0,
            idle: 0.0,
            moved: 0.0,
            finger: None,
            layout,
            theme,
            title: TitleCard::new(look::TITLE, theme).tagline(look::TAGLINE).best(best as i64),
            hud: Hud::new(theme, best as i64),
            results: None,
            daily_btn: Button::new(row[1], "DAILY").kind(ButtonKind::Secondary).with_icon(Icon::STAR),
            scores_btn: Button::new(row[0], "SCORES").kind(ButtonKind::Secondary).with_icon(Icon::TROPHY),
            fx: Particles::new(rng.next_u32() as u64),
            popups: Popups::new(),
            shake: Shake::new(),
            hitstop: HitStop::default(),
            flash: Flash::new(0.25),
            hurt: VignettePulse::new(look::DANGER),
            mult_punch: Punch::new(),
            ship_punch: Punch::new(),
            graze_popup: 0.0,
            sound: sound::Sound::new(),
            art: draw::Art::new(),
            rng,
        };
        g.place_daily();
        g
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        self.idle += dt;
        self.art.poll();
        self.graze_popup -= dt;
        for e in input::poll() {
            if self.ui_event(&e) {
                continue;
            }
            match e.kind() {
                Input::TouchDown | Input::MouseDown => self.press(&e),
                Input::TouchMove | Input::MouseMove => self.drag(&e),
                Input::TouchUp | Input::TouchCancel | Input::MouseUp => {
                    if self.finger.is_some_and(|f| f.0 == e.id) {
                        self.finger = None;
                        self.sim.touch_up();
                    }
                }
                _ => {}
            }
        }

        // Hit-stop freezes the world for a few frames on big hits.
        let sdt = self.hitstop.step(dt);
        match self.mode {
            Mode::Title => {
                self.demo_bot.drive(&mut self.sim, sdt);
                self.sim.step(sdt);
                if self.sim.t > ATTRACT_SECONDS {
                    (self.sim, self.demo_bot) = attract(&mut self.rng);
                }
            }
            Mode::Play => {
                self.sim.step(sdt);
                if self.sim.over {
                    self.finish();
                }
            }
            Mode::Over(ref mut t) => {
                *t += dt;
                self.sim.step(sdt);
            }
        }
        let cues = std::mem::take(&mut self.sim.cues);
        self.juice(&cues);
        let playing = self.mode == Mode::Play;
        let boss_live = self.sim.boss.as_ref().is_some_and(|b| b.dying.is_none());
        let shield = self.sim.ship.shield && self.sim.alive();
        self.sound.update(dt, &cues, playing, self.sim.intensity(), boss_live, shield);

        self.hud.lives = Some((self.sim.ship.hp, LIVES));
        if self.mode == Mode::Play && self.hud.score.target() != self.sim.score as i64 {
            self.hud.set_score(self.sim.score as i64);
        }
        self.hurt.level = if playing && self.sim.ship.hp == 1 && self.sim.alive() { 0.22 + 0.08 * (self.time * 4.0).sin() } else { 0.0 };
        self.title.update(dt);
        self.hud.update(dt);
        if let Some(card) = &mut self.results
            && card.update(dt) == Some(ResultsEvent::NewBest)
        {
            self.sound.new_best();
        }
        self.daily_btn.update(dt);
        self.scores_btn.update(dt);
        self.fx.update(dt);
        self.popups.update(dt);
        self.shake.update(dt);
        self.flash.update(dt);
        self.hurt.update(dt);
        self.mult_punch.update(dt);
        self.ship_punch.update(dt);
    }

    fn render(&self) {
        draw::frame(self);
    }

    fn suspend(&mut self) {
        self.sound.stop_music();
        self.finger = None;
        self.sim.touch_up();
    }

    fn resume(&mut self) {
        if self.mode == Mode::Play && self.sim.alive() {
            self.sound.resume_music();
        }
    }
}

impl Shooter {
    fn current_best(&self) -> u64 {
        if self.daily { self.best_daily } else { self.best }
    }

    /// Whether to show the drag hint: until the thumb has dragged a little,
    /// and again whenever it sits still for a while.
    pub fn show_hint(&self) -> bool {
        self.mode == Mode::Play && self.sim.alive() && (self.moved < HINT_UNTIL || self.idle > HINT_IDLE)
    }

    /// A fresh round. The tap that started it also starts steering.
    fn start(&mut self, e: &Event) {
        let seed = if self.daily { sys::daily_seed() } else { self.rng.next_u32() as u64 };
        self.sim = Sim::new(seed);
        if DEBUG_SKIP_TO > 0.0 {
            self.sim.skip_to(DEBUG_SKIP_TO);
        }
        self.mode = Mode::Play;
        self.results = None;
        self.moved = 0.0;
        self.idle = 0.0;
        self.fx.clear();
        self.popups.list.clear();
        self.hud.reset(self.current_best() as i64);
        sys::round(Round::Playing);
        self.sound.start();
        self.finger = Some((e.id, e.x, e.y));
        self.sim.touch_down(e.x, e.y);
        self.ship_punch.kick(0.25);
    }

    fn press(&mut self, e: &Event) {
        match self.mode {
            // The tap that enters play starts the round and can already steer (SPEC §5.2).
            Mode::Title => self.start(e),
            Mode::Play => {
                if self.finger.is_none() {
                    self.finger = Some((e.id, e.x, e.y));
                    self.sim.touch_down(e.x, e.y);
                    self.ship_punch.kick(0.2);
                    self.idle = 0.0;
                }
            }
            Mode::Over(t) if t > RETRY_GUARD && !self.results.as_ref().is_some_and(|c| c.claims(e.x, e.y)) => self.start(e),
            Mode::Over(_) => {}
        }
    }

    fn drag(&mut self, e: &Event) {
        let Some((id, x, y)) = self.finger else { return };
        if id != e.id || self.mode != Mode::Play {
            return;
        }
        let d = (e.x - x).hypot(e.y - y);
        if d > 0.5 {
            self.idle = 0.0;
            self.moved += d;
        }
        self.finger = Some((id, e.x, e.y));
        self.sim.touch_move(e.x, e.y);
    }

    /// The death animation has played: submit, save, show the card, report Over.
    fn finish(&mut self) {
        let score = self.sim.score;
        let s = &self.sim;
        sys::log(Level::Info, &format!("nova pip: over at {:.1}s, score {score}, {} kills, {} bosses (daily: {})", s.t, s.kills, s.loop_n, self.daily));
        let prev = self.current_best();
        // Submit before reporting Over: the platform saves the replay on Over.
        if self.daily {
            store::submit_score(BOARD_DAILY, score as i64);
            if score > self.best_daily {
                self.best_daily = score;
                store::set_u64(&daily_key(), score);
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
        self.results = Some(ResultsCard::new(score as i64, prev as i64, self.theme, &self.layout, seed).heading(look::HEADING).label(label));
        self.sound.verdict(prev > 0 && score > prev);
        self.daily_btn.reset();
        self.scores_btn.reset();
        self.finger = None;
        self.mode = Mode::Over(0.0);
        self.place_daily();
        sys::round(Round::Over);
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
                    return false; // press() ignores it too
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
                match self.results.as_mut().and_then(|r| r.handle(e)) {
                    Some(ResultsAction::Retry) => {
                        self.start(e);
                        true
                    }
                    _ => self.results.as_ref().is_some_and(|r| r.claims(e.x, e.y)),
                }
            }
        }
    }

    fn toggle_daily(&mut self) {
        self.daily = !self.daily;
        self.sound.ui(self.daily);
        self.sound.stop_music();
        self.results = None;
        self.mode = Mode::Title;
        (self.sim, self.demo_bot) = attract(&mut self.rng);
        self.title.set_best(self.current_best() as i64);
        self.title.restart();
        self.place_daily();
        sys::round(Round::Idle);
    }

    /// The DAILY button: centred on the title card, beside SCORES on the results card.
    fn place_daily(&mut self) {
        let l = &self.layout;
        let row = Rect::new(l.card.x, l.card.bottom() - 46.0, l.card.w, 44.0).columns(2, 12.0);
        self.daily_btn.rect = if self.mode == Mode::Title { Rect::centered(l.card.cx(), row[1].cy(), 150.0, 44.0) } else { row[1] };
        self.daily_btn.kind = if self.daily { ButtonKind::Primary } else { ButtonKind::Secondary };
    }

    /// Particles, popups, shake, hit-stop and flashes for this frame's cues.
    fn juice(&mut self, cues: &[Cue]) {
        let th = self.theme;
        let card = self.mode == Mode::Title;
        for &c in cues {
            match c {
                Cue::Kill { x, y, kind, points, mult, combo } => {
                    let col = look::ENEMY[kind.index()];
                    let big = matches!(kind, Kind::Bulb | Kind::Spinner);
                    self.fx.burst(x, y, if big { 26 } else { 12 }, col);
                    self.fx.sparks(x, y, -FRAC_PI_2, PI, if big { 18 } else { 8 }, lighten(col, 0.5));
                    self.fx.ring(x, y, if big { 90.0 } else { 44.0 }, with_alpha(col, 0.8));
                    if !card {
                        // Kept below the HUD's score, sized by the points.
                        let pc = if mult > 1 { th.gold } else { 0xffffffff };
                        let size = (14.0 + 3.0 * (points as f32).log10()).min(25.0);
                        let py = (y - 14.0).max(self.layout.hud.y + 150.0);
                        self.popups.spawn(x, py, &format!("+{}", fmt_int(points as i64)), pc, size);
                        if combo % 5 == 0 && combo > 0 {
                            self.mult_punch.kick(0.5);
                        }
                    }
                    self.shake.add(if big { 0.4 } else { 0.12 });
                    if big {
                        self.hitstop.freeze(0.05);
                    }
                }
                Cue::Hit { x, y } | Cue::BossHit { x, y } => self.fx.sparks(x, y, -FRAC_PI_2, 1.2, 2, 0xfff2c0ff),
                Cue::Gem { x, y, .. } => self.fx.sparkles(x, y, 6.0, 2, look::GEM),
                Cue::Power { x, y, power } => {
                    let col = look::power_color(power);
                    self.fx.ring(x, y, 70.0, col);
                    self.fx.sparkles(x, y, 30.0, 10, col);
                    self.popups.spawn(x, y - 26.0, look::power_name(power), col, 22.0);
                    self.flash.fire(with_alpha(col, 0.25));
                }
                Cue::Heart { x, y } => {
                    self.fx.ring(x, y, 70.0, look::HEART);
                    self.fx.stars(x, y, 8, look::HEART);
                    self.popups.spawn(x, y - 26.0, "+♥", look::HEART, 26.0);
                }
                Cue::Graze { x, y } => {
                    self.fx.sparks(x, y, -FRAC_PI_2, PI, 3, 0xffffffff);
                    if self.graze_popup <= 0.0 && !card {
                        self.graze_popup = 1.6;
                        self.popups.spawn(self.sim.ship.x, self.sim.ship.y - 36.0, "CLOSE!", 0xffffffff, 18.0);
                    }
                }
                Cue::ShieldBreak { x, y } => {
                    self.fx.ring(x, y, 110.0, look::SHIELD);
                    self.fx.burst(x, y, 20, look::SHIELD);
                    self.shake.add(0.35);
                    self.hitstop.freeze(0.06);
                }
                Cue::Hurt { x, y, .. } => {
                    self.fx.burst(x, y, 18, look::DANGER);
                    self.fx.sparks(x, y, FRAC_PI_2, PI, 14, 0xffd0d0ff);
                    self.shake.add(0.55);
                    self.hitstop.freeze(0.09);
                    if !card {
                        self.flash.fire(0xff304060);
                        self.hurt.fire();
                    }
                }
                Cue::Dying { .. } => {
                    if !card {
                        sys::log(Level::Info, &format!("nova pip: last heart lost at {:.1}s", self.sim.t));
                    }
                    // A longer freeze first, so the player sees what got them.
                    self.hitstop.freeze(0.3);
                    self.shake.add(0.4);
                }
                Cue::Boom { x, y, size } => {
                    self.fx.burst(x, y, 8 + (size * 30.0) as usize, 0xff5ad0ff);
                    self.fx.sparks(x, y, -FRAC_PI_2, PI, 6 + (size * 24.0) as usize, 0xffd890ff);
                    self.fx.ring(x, y, 30.0 + 110.0 * size, 0xffffffc0);
                    self.shake.add(0.12 + 0.6 * size);
                    if size > 0.9 {
                        self.flash.fire(0xffffff90);
                    }
                }
                Cue::BossBeaten { x, y } => {
                    self.hitstop.freeze(0.22);
                    self.flash.fire(0xffffffb0);
                    self.shake.add(0.6);
                    self.fx.ring(x, y, 200.0, 0xffffffe0);
                }
                Cue::BossDown { x, y, points } => {
                    self.fx.celebrate(x, y, &th);
                    self.popups.spawn(x, y + 40.0, &format!("+{}", fmt_int(points as i64)), th.gold, 34.0);
                    self.shake.add(0.8);
                }
                Cue::Warning => self.flash.fire(0xff203050),
                Cue::BossArrive => self.shake.add(0.5),
                _ => {}
            }
        }
    }
}

fn daily_key() -> String {
    format!("daily:{:016x}", sys::daily_seed())
}

export_game!(Shooter);
