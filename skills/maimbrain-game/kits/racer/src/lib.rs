//! Swerve: a 3D endless lane racer for one thumb. Tap the left or right half
//! of the screen (or swipe) to change lanes; dodge traffic, trucks that
//! change lanes, roadworks and oil; grab coins and boost pads. It speeds up
//! every 12 s and something new joins the road every ~15 s.
//!
//! Shape (as in the 3D template): the rules and the camera in `sim` (tested,
//! with a bot in `tests`), the scene in `scene`, sound in `sound`, names and
//! colors in `look`, and this thin host layer: rounds, input, saving, juice
//! and the HUD (the SDK's UI kit, drawn with mb2d over the 3D image).
//!
//! Daily mode plays the same road for everyone today (`sys::daily_seed`).

mod feel;
mod look;
mod scene;
mod sim;
mod sound;
#[cfg(test)]
mod tests;

use maimbrain::gfx2d::{self, Font};
use maimbrain::gfx3d::{self, vec3};
use maimbrain::input::{self, Event, Kind as Input};
use maimbrain::juice::{Flash, Particles};
use maimbrain::motion::{Ease, HitStop};
use maimbrain::sys::{self, Round};
use maimbrain::ui::{self, Button, ButtonKind, Countdown, Hud, Icon, Layout, Rect, ResultsAction, ResultsCard, ResultsEvent, Tap, Theme, TitleCard, shape, text, with_alpha};
use maimbrain::{Game, Rng, export_game, store};

use scene::Scene;
use sim::{Cue, Kind, Sim, W};
use sound::Audio;

const BOARD: u32 = 0;
const BOARD_DAILY: u32 = 1;
/// Taps right after a round ends don't restart it (a frantic tap during the crash).
const RETRY_GUARD: f32 = 0.35;
/// Pixels of sideways drag that make a swipe.
const SWIPE: f32 = 34.0;
/// Seconds without a press before the hint comes back.
const HINT_IDLE: f32 = 3.0;
/// Debug: the car drives itself for this many seconds of play, then lets go
/// (to screenshot late rounds and the crash). Keep it 0.
const DEBUG_AUTOPILOT: f32 = 0.0;
/// The speedo reads this many times the real speed: arcade km/h (a 4 m car at
/// 16 m/s "is" 115 km/h, not 58). Display only.
const SPEEDO: f32 = 2.0;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Title,
    Play,
    /// Seconds since the results card appeared.
    Over(f32),
}

/// A big announcement across the screen ("TRUCKS!", "SPEED UP!").
struct Banner {
    text: String,
    age: f32,
    color: u32,
    ribbon: bool,
}

struct Racer {
    mode: Mode,
    sim: Sim,
    scene: Scene,
    rng: Rng,
    daily: bool,
    best: u32,
    best_daily: u32,
    /// Seconds since init (idle animation; a function of dt only).
    time: f32,
    /// 1 on the title (attract camera), easing to 0 in play.
    title_k: f32,
    /// The current touch: where it went down, and whether it already swiped.
    touch: Option<(f32, bool)>,
    trucks_seen: u32,
    layout: Layout,
    theme: Theme,
    title: TitleCard,
    hud: Hud,
    results: Option<ResultsCard>,
    daily_btn: Button,
    scores_btn: Button,
    fx: Particles,
    flash: Flash,
    stop: HitStop,
    banner: Option<Banner>,
    /// A run-up after the platform pauses the game mid-round.
    countdown: Countdown,
    audio: Audio,
}

impl Racer {
    fn new_sim(&mut self) -> Sim {
        let seed = if self.daily { sys::daily_seed() } else { self.rng.next_u32() as u64 };
        Sim::new(seed)
    }

    fn current_best(&self) -> u32 {
        if self.daily { self.best_daily } else { self.best }
    }

    fn daily_key() -> String {
        format!("daily:{:016x}", sys::daily_seed())
    }

    fn start(&mut self) {
        self.sim = self.new_sim();
        self.hud.reset(self.current_best() as i64);
        self.results = None;
        self.banner = None;
        self.fx.clear();
        self.trucks_seen = 0;
        self.mode = Mode::Play;
        self.audio.start();
        sys::round(Round::Playing);
    }

    fn back_to_title(&mut self) {
        self.mode = Mode::Title;
        self.daily_btn.rect = daily_alone(&self.layout);
        self.results = None;
        self.sim = self.new_sim();
        self.title.set_best(self.current_best() as i64);
        self.title.restart();
        sys::round(Round::Idle);
    }

    fn toggle_daily(&mut self) {
        self.daily = !self.daily;
        self.audio.ui();
        self.daily_btn.kind = if self.daily { ButtonKind::Primary } else { ButtonKind::Secondary };
        self.daily_btn.label = if self.daily { "DAILY ON".into() } else { "DAILY".into() };
        self.back_to_title();
    }

    /// The round is over (the crash has played out): score first, then Over.
    fn finish(&mut self) {
        let score = self.sim.score();
        let prev = self.current_best();
        if self.daily {
            store::submit_score(BOARD_DAILY, score as i64);
            if score > self.best_daily {
                self.best_daily = score;
                store::set_u64(&Racer::daily_key(), score as u64);
            }
        } else {
            store::submit_score(BOARD, score as i64);
            if score > self.best {
                self.best = score;
                store::set_u64("best", score as u64);
            }
        }
        self.title.set_best(self.current_best() as i64);
        let label = if self.daily { look::DAILY_LABEL } else { look::SCORE_LABEL };
        let seed = self.rng.next_u32() as u64;
        self.results = Some(ResultsCard::new(score as i64, prev as i64, self.theme, &self.layout, seed).heading(look::HEADING).label(label).floating());
        self.daily_btn.reset();
        self.scores_btn.reset();
        self.daily_btn.rect = card_row(&self.layout)[1];
        self.mode = Mode::Over(0.0);
        sys::round(Round::Over);
    }

    fn banner(&mut self, s: &str, color: u32, ribbon: bool) {
        self.banner = Some(Banner { text: s.into(), age: 0.0, color, ribbon });
    }

    /// Turns this frame's cues into sound, haptics and screen juice.
    fn cues(&mut self, playing: bool) {
        let cues = std::mem::take(&mut self.sim.cues);
        for c in &cues {
            self.scene.cue(c, &self.sim, self.time);
            self.audio.cue(c, playing);
            if !playing {
                continue;
            }
            match *c {
                Cue::Coin { .. } => self.hud.score.punch.kick(0.12),
                Cue::NearMiss { chain, .. } => {
                    self.hud.score.punch.kick(0.3);
                    if chain >= 3 {
                        self.flash.fire(0xffd24a30);
                    }
                }
                Cue::Boost { .. } => {
                    self.flash.fire(0x7af4ff50);
                    self.banner(look::BOOST, 0x7af4ffff, false);
                }
                Cue::Smash { .. } => {
                    self.stop.freeze(0.05);
                    self.hud.score.punch.kick(0.35);
                }
                Cue::SpeedUp { .. } => {
                    let kmh = (sim::Sim::cruise_at(self.sim.t) * 3.6 * SPEEDO).round() as u32;
                    self.banner(&format!("{}  {kmh} KM/H", look::SPEED_UP), self.theme.text, false);
                }
                Cue::New(kind) => {
                    if kind == Kind::Truck {
                        self.trucks_seen += 1;
                    }
                    let s = look::new_hazard(kind, self.trucks_seen <= 1);
                    self.banner(s, self.theme.gold, true);
                }
                Cue::Crash { at } => {
                    self.stop.freeze(0.08);
                    self.flash.fire(0xfff4e880);
                    let p = self.sim.camera(self.title_k, self.time).project(at, sim::W, sim::H);
                    if p.on_screen {
                        self.fx.ring(p.x, p.y, 160.0, 0xfff4e8c0);
                        self.fx.sparks(p.x, p.y, -std::f32::consts::FRAC_PI_2, 1.4, 26, 0xffb35aff);
                    }
                    self.banner = None;
                }
                _ => {}
            }
        }
    }

    /// Feeds an event to the card buttons; true if one took it.
    fn card_event(&mut self, e: &Event) -> bool {
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
                if t < RETRY_GUARD {
                    return false;
                }
                if let Some(tap) = self.daily_btn.handle(e) {
                    if tap == Tap::Clicked {
                        self.toggle_daily();
                    }
                    return true;
                }
                if let Some(tap) = self.scores_btn.handle(e) {
                    if tap == Tap::Clicked {
                        self.audio.ui();
                        store::show_scores(if self.daily { BOARD_DAILY } else { BOARD });
                    }
                    return true;
                }
                if self.results.as_mut().and_then(|r| r.handle(e)) == Some(ResultsAction::Retry) {
                    self.start();
                    return true;
                }
                self.results.as_ref().is_some_and(|r| r.claims(e.x, e.y))
            }
        }
    }
}

impl Game for Racer {
    fn init() -> Self {
        sys::round(Round::Idle);
        let mut rng = Rng::from_host();
        let sim = Sim::new(rng.next_u32() as u64);
        let scene = Scene::build(rng.next_u32() as u64);
        let layout = Layout::new();
        let theme = look::theme();
        let best = store::get_u64("best").unwrap_or(0) as u32;
        let best_daily = store::get_u64(&Racer::daily_key()).unwrap_or(0) as u32;
        let row = card_row(&layout);
        let fx_seed = rng.next_u32() as u64;
        Racer {
            mode: Mode::Title,
            sim,
            scene,
            rng,
            daily: false,
            best,
            best_daily,
            time: 0.0,
            title_k: 1.0,
            touch: None,
            trucks_seen: 0,
            layout,
            theme,
            // The title is a feed card, an ad: a name, a hook and a brag, never instructions.
            title: TitleCard::new(look::TITLE, theme).tagline(look::TAGLINE).best(best as i64),
            hud: Hud::new(theme, best as i64),
            results: None,
            daily_btn: Button::new(daily_alone(&layout), "DAILY").kind(ButtonKind::Secondary).with_icon(Icon::STAR),
            scores_btn: Button::new(row[0], "SCORES").kind(ButtonKind::Secondary).with_icon(Icon::TROPHY),
            fx: Particles::new(fx_seed),
            flash: Flash::new(0.35),
            stop: HitStop::default(),
            banner: None,
            countdown: Countdown::default(),
            audio: Audio::new(),
        }
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        for e in input::poll() {
            if self.card_event(&e) {
                continue;
            }
            match e.kind() {
                Input::TouchDown | Input::MouseDown => {
                    let dir = if e.x < W / 2.0 { -1 } else { 1 };
                    match self.mode {
                        // The tap that enters play is also the first lane change (SPEC §5.2).
                        Mode::Title => {
                            self.start();
                            self.sim.press(dir);
                            self.touch = Some((e.x, false));
                        }
                        Mode::Play => {
                            self.sim.press(dir);
                            self.touch = Some((e.x, false));
                        }
                        Mode::Over(t) if t > RETRY_GUARD => {
                            self.start();
                            self.sim.press(dir);
                            self.touch = Some((e.x, false));
                        }
                        Mode::Over(_) => {}
                    }
                }
                Input::TouchMove | Input::MouseMove => {
                    if let (Mode::Play, Some((x0, false))) = (self.mode, self.touch)
                        && (e.x - x0).abs() > SWIPE
                    {
                        self.sim.swipe(if e.x > x0 { 1 } else { -1 });
                        self.touch = Some((x0, true));
                    }
                }
                Input::TouchUp | Input::TouchCancel | Input::MouseUp => self.touch = None,
                _ => {}
            }
        }

        match &mut self.mode {
            Mode::Play => {
                if self.sim.t < DEBUG_AUTOPILOT {
                    self.sim.autopilot();
                }
                if self.countdown.update(dt).is_some() {
                    self.audio.ui();
                }
                let sdt = if self.countdown.active() { 0.0 } else { self.stop.step(dt) };
                self.sim.step(sdt);
                self.cues(true);
                self.hud.set_score(self.sim.score() as i64);
                self.hud.progress = (self.sim.boost > 0.0).then(|| self.sim.boost / sim::BOOST_TIME);
                if self.sim.over {
                    self.finish();
                }
            }
            Mode::Over(t) => {
                *t += dt;
                self.sim.step(dt);
                self.cues(false);
            }
            // The title card is an attract mode: the car drives and dodges by itself.
            Mode::Title => {
                self.sim.autopilot();
                self.sim.step(dt);
                self.cues(false);
                if self.sim.over {
                    self.sim = self.new_sim();
                }
            }
        }
        let want = if self.mode == Mode::Title { 1.0 } else { 0.0 };
        self.title_k += (want - self.title_k) * (1.0 - (-dt * 2.2).exp());
        self.scene.update(&self.sim, self.title_k, self.time, dt);
        self.audio.update(&self.sim, self.mode == Mode::Play, dt);
        self.title.update(dt);
        self.hud.update(dt);
        self.fx.update(dt);
        self.flash.update(dt);
        self.daily_btn.update(dt);
        self.scores_btn.update(dt);
        if let Some(b) = &mut self.banner {
            b.age += dt;
            if b.age > 1.8 {
                self.banner = None;
            }
        }
        if let Some(card) = &mut self.results
            && card.update(dt) == Some(ResultsEvent::NewBest)
        {
            self.audio.new_best();
        }
    }

    /// Back from a pause mid-round: 3, 2, 1 before the road moves again.
    fn resume(&mut self) {
        if self.mode == Mode::Play && !self.sim.crashed() {
            self.countdown.start(3);
        }
    }

    fn render(&self) {
        gfx3d::camera(&self.sim.camera(self.title_k, self.time));
        gfx3d::render();

        match self.mode {
            Mode::Title => {
                self.title.draw(&self.layout);
                self.daily_btn.draw(&self.theme);
            }
            Mode::Play => {
                self.draw_speed_lines();
                self.draw_tells();
                self.hud.draw(&self.layout);
                self.draw_speed();
                self.draw_hint();
                self.draw_banner();
                self.countdown.draw(&self.layout, &self.theme);
            }
            Mode::Over(_) => {
                gfx2d::rect(0.0, 0.0, W, sim::H, with_alpha(self.theme.outline, 0.25));
                if let Some(card) = &self.results {
                    card.draw();
                }
                self.scores_btn.draw(&self.theme);
                self.daily_btn.draw(&self.theme);
            }
        }
        self.fx.draw();
        self.flash.draw(self.layout.screen);
    }
}

impl Racer {
    /// Readable danger: an arrow over a truck that's about to move over.
    fn draw_tells(&self) {
        for o in self.sim.obstacles.iter().filter(|o| o.signalling() && o.blink.is_some()) {
            let z = -(o.d - self.sim.dist);
            let p = gfx3d::project(vec3(o.x, 3.9, z));
            if !p.on_screen || p.depth > 70.0 {
                continue;
            }
            let dir = o.change.map_or(0, |to| to - o.from_lane);
            let pulse = 1.0 + 0.15 * (self.time * 12.0).sin();
            let s = (38.0 * 14.0 / p.depth.max(8.0)).clamp(18.0, 40.0) * pulse;
            let icon = if dir < 0 { Icon::ARROW_LEFT } else { Icon::ARROW_RIGHT };
            shape::disc(p.x, p.y, s * 0.62, with_alpha(self.theme.outline, 0.7));
            icon.draw(p.x, p.y, s, 0xffa628ff);
        }
    }

    /// Speed lines: thin streaks shooting out from the vanishing point, only
    /// in an outer ring (the lanes ahead stay clear), stronger with the speed
    /// feel and on boost. Stateless: each line is a hash of its time slot.
    fn draw_speed_lines(&self) {
        let s = &self.sim;
        let k = s.feel.lines();
        if k <= 0.0 || s.crashed() {
            return;
        }
        let vp = gfx3d::project(vec3(0.0, 1.0, -120.0));
        let (cx, cy) = if vp.on_screen { (vp.x, vp.y) } else { (W / 2.0, sim::H * 0.42) };
        let hash = |a: u32| {
            let mut x = a.wrapping_mul(0x9e37_79b9) ^ 0x85eb_ca6b;
            x ^= x >> 15;
            x = x.wrapping_mul(0x2c1b_3c6d);
            x ^= x >> 12;
            (x & 0xffff) as f32 / 65535.0
        };
        let count = (10.0 + 30.0 * k) as u32;
        let life = 0.22;
        let col = if s.boost_k > 0.3 { 0xbff8ffff } else { 0xfff1dcff };
        for i in 0..count {
            // Each line has its own phase so they don't all fire together.
            let t = self.time / life + hash(i * 7 + 3);
            let slot = t.floor() as u32;
            let age = t.fract();
            let seed = slot.wrapping_mul(131) ^ i.wrapping_mul(977);
            let ang = hash(seed) * std::f32::consts::TAU;
            let (dx, dy) = (ang.cos(), ang.sin());
            // Never over the car and the road just ahead of it (straight down from the vanishing point).
            if dy > 0.55 && dx.abs() < 0.75 {
                continue;
            }
            // Start at the edge of a clear ellipse round the road ahead, shoot outward.
            let r0 = 85.0 + 90.0 * hash(seed ^ 0x51);
            let r = r0 + 380.0 * age * age;
            let len = (30.0 + 80.0 * hash(seed ^ 0x77)) * (0.5 + age);
            let (x0, y0) = (cx + dx * r, cy + dy * r * 1.5);
            let (x1, y1) = (cx + dx * (r + len), cy + dy * (r + len) * 1.5);
            let a = k * (1.0 - age * age) * (0.3 + 0.4 * hash(seed ^ 0x9));
            shape::capsule(x0, y0, x1, y1, 1.0 + 1.2 * hash(seed ^ 0x3), with_alpha(col, a));
        }
    }

    fn draw_speed(&self) {
        let kmh = (self.sim.speed_now() * 3.6 * SPEEDO).round() as u32;
        let x = self.layout.hud.right() - 52.0;
        let y = self.layout.hud.y + 24.0;
        let fg = if self.sim.boost_k > 0.2 { 0x7af4ffff } else { self.theme.text };
        ui::pill(&format!("{kmh}"), x, y, 30.0, with_alpha(self.theme.outline, 0.55), fg, Font::SansBold, Some(Icon::BOLT));
        text("KM/H").size(10.0).color(self.theme.text_dim).middle().draw(x, y + 23.0);
        if self.daily {
            ui::pill("DAILY", x, y + 46.0, 22.0, with_alpha(self.theme.outline, 0.5), self.theme.gold, Font::SansBold, Some(Icon::STAR));
        }
    }

    /// Play teaches: ghost taps on both halves and arrows by the car, until
    /// the player has steered, and again whenever they stall.
    fn draw_hint(&self) {
        let s = &self.sim;
        if s.crashed() {
            return;
        }
        // Back after a stall: early on, or when something's coming and they haven't moved.
        let stalled = s.idle > HINT_IDLE && (s.steers < 6 || s.threat(s.lane, 1.0, false) < 1.0);
        let show = (s.steers < 3 && s.t < 10.0) || stalled;
        if !show {
            return;
        }
        let a = if stalled { ((s.idle - HINT_IDLE) * 3.0).min(1.0) } else { 1.0 - ((s.t - 8.0) / 2.0).clamp(0.0, 1.0) };
        let car = gfx3d::project(vec3(s.x, 0.7, 0.0));
        let cy = if car.on_screen { car.y } else { 470.0 };
        let phase = (self.time * 1.25).fract();
        let side = if ((self.time * 1.25) as u32).is_multiple_of(2) { -1.0 } else { 1.0 };
        for dir in [-1.0f32, 1.0] {
            // Chevrons beside the car pointing out, the tapped side brighter.
            let on = dir == side;
            let col = with_alpha(0xfff4e8ff, a * if on { 0.95 } else { 0.45 });
            for k in 0..2 {
                let x = W / 2.0 + dir * (78.0 + k as f32 * 18.0 + if on { 6.0 * Ease::QuadOut.at(phase) } else { 0.0 });
                shape::capsule(x, cy - 12.0, x + dir * 10.0, cy, 5.0, with_alpha(self.theme.outline, a * 0.6));
                shape::capsule(x + dir * 10.0, cy, x, cy + 12.0, 5.0, with_alpha(self.theme.outline, a * 0.6));
                shape::capsule(x, cy - 12.0, x + dir * 10.0, cy, 3.0, col);
                shape::capsule(x + dir * 10.0, cy, x, cy + 12.0, 3.0, col);
            }
        }
        // A ghost thumb tapping the side it points to.
        let fx = W / 2.0 + side * 112.0;
        let fy = cy + 92.0;
        let press = if phase < 0.25 { Ease::QuadOut.at(phase / 0.25) } else { 1.0 - ((phase - 0.25) / 0.75).min(1.0) };
        shape::ring(fx, fy, 22.0 + 26.0 * phase, 3.0, with_alpha(0xfff4e8ff, a * (1.0 - phase) * 0.8));
        shape::disc(fx, fy + 2.0, 17.0 - 3.0 * press, with_alpha(self.theme.outline, a * 0.5));
        shape::disc(fx, fy, 15.0 - 3.0 * press, with_alpha(0xfff4e8ff, a * 0.85));
        text(look::HINT)
            .size(20.0)
            .color(with_alpha(self.theme.text, a))
            .outline(0.1, with_alpha(self.theme.outline, a))
            .middle()
            .draw(W / 2.0, cy + 150.0);
    }

    fn draw_banner(&self) {
        let Some(b) = &self.banner else { return };
        let k = Ease::BackOut.at((b.age / 0.3).min(1.0));
        let a = 1.0 - ((b.age - 1.4) / 0.4).clamp(0.0, 1.0);
        let y = self.layout.hud.y + 150.0;
        gfx2d::push();
        gfx2d::translate(W / 2.0, y);
        gfx2d::scale(k, k);
        if b.ribbon {
            ui::ribbon(&b.text, 0.0, 0.0, 40.0, with_alpha(b.color, a), with_alpha(self.theme.outline, a), Font::SansBold, -0.04);
        } else {
            text(&b.text).size(26.0).color(with_alpha(b.color, a)).outline(0.1, with_alpha(self.theme.outline, a)).middle().draw(0.0, 0.0);
        }
        gfx2d::pop();
    }
}

/// SCORES and DAILY on the results card: low in the card area, clear of the feed's overlays.
fn card_row(layout: &Layout) -> Vec<Rect> {
    Rect::new(layout.card.x + 8.0, layout.card.bottom() - 48.0, layout.card.w - 16.0, 44.0).columns(2, 12.0)
}

/// DAILY alone on the title card, centred.
fn daily_alone(layout: &Layout) -> Rect {
    Rect::centered(layout.card.cx(), layout.card.bottom() - 26.0, 150.0, 44.0)
}

export_game!(Racer);
