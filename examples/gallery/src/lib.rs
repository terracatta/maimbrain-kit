//! UI Gallery: every widget, icon, text style, easing curve and juice
//! effect in `maimbrain::ui`, `maimbrain::motion` and `maimbrain::juice`,
//! one page at a time. Not a feed game: a living reference (docs/UI.md).
//!
//! Arrows at the bottom (or ← →, 1–9 on a keyboard) change pages; the pill
//! top-right (or T) cycles the theme; taps elsewhere poke the page.

use maimbrain::gfx2d::{self, Font};
use maimbrain::input::{self, Kind};
use maimbrain::juice::{Combo, Flash, Particles, Popups, VignettePulse};
use maimbrain::motion::{Counter, Ease, Punch, Seq, Shake, Spring, Timer, stagger};
use maimbrain::sys::{self, Round};
use maimbrain::ui::shape::{self, capsule, disc, rrect, soft_disc};
use maimbrain::ui::{
    self, Align, Button, ButtonKind, Countdown, Hud, Icon, Layout, Rect, ResultsCard, Tap, Theme, TitleCard, VAlign, fade, text,
    with_alpha,
};
use maimbrain::{Game, Rng, export_game};

const PAGES: [&str; 9] = ["TITLE", "WIDGETS", "ICONS", "TEXT", "EASING", "MOTION", "JUICE", "RESULTS", "HUD"];

// HID usage codes (SPEC §5.2).
const KEY_RIGHT: u32 = 79;
const KEY_LEFT: u32 = 80;
const KEY_1: u32 = 30;
const KEY_T: u32 = 23;

struct Gallery {
    layout: Layout,
    theme_i: usize,
    theme: Theme,
    page: usize,
    t: f32,
    page_t: f32,
    rng: Rng,
    prev: Button,
    next: Button,
    theme_btn: Button,
    // widgets page
    buttons: Vec<Button>,
    // title page
    title: TitleCard,
    // motion page
    springs: [Spring; 3],
    spring_timer: Timer,
    counter: Counter,
    count_timer: Timer,
    punch: Punch,
    shake: Shake,
    seq: Seq,
    // juice page
    fx: Particles,
    popups: Popups,
    combo: Combo,
    flash: Flash,
    vignette: VignettePulse,
    juice_timer: Timer,
    juice_step: u32,
    // results / hud pages
    results: ResultsCard,
    results_round: u32,
    hud: Hud,
    hud_score: i64,
    hud_timer: Timer,
    countdown: Countdown,
}

impl Gallery {
    fn set_theme(&mut self, i: usize) {
        self.theme_i = i % Theme::presets().len();
        self.theme = Theme::presets()[self.theme_i].1;
        self.title.theme = self.theme;
        self.results.theme = self.theme;
        self.hud.theme = self.theme;
        self.theme_btn.label = Theme::presets()[self.theme_i].0.to_uppercase();
    }

    fn go(&mut self, page: usize) {
        self.page = page % PAGES.len();
        self.page_t = 0.0;
        self.fx.clear();
        self.popups.list.clear();
        self.combo = Combo::new(3.0);
        match self.page {
            0 => self.title.restart(),
            7 => self.new_results(),
            8 => {
                self.hud_score = 0;
                self.hud.reset(120);
                self.countdown.start(3);
            }
            _ => {}
        }
    }

    fn new_results(&mut self) {
        // Alternate a new best with a near miss.
        let (score, best) = if self.results_round.is_multiple_of(2) { (1280, 940) } else { (88, 92) };
        self.results_round += 1;
        let extra = Button::new(Rect::default(), "SCORES").kind(ButtonKind::Secondary).with_icon(Icon::TROPHY);
        self.results = ResultsCard::new(score, best, self.theme, &self.layout, self.rng.next_u32() as u64).heading("SPLAT!").label("POINTS").extra(extra);
    }

    fn poke(&mut self, x: f32, y: f32) {
        match self.page {
            0 => self.title.restart(),
            5 => {
                self.punch.kick(0.4);
                self.shake.add(0.6);
            }
            6 => {
                self.fx.confetti(x, y, 60, &self.theme.confetti);
                self.popups.score(x, y - 20.0, 100 * (1 + self.combo.count as i64), self.theme.gold);
                self.combo.hit();
            }
            8 => {
                self.hud_score += 25;
                self.hud.set_score(self.hud_score);
            }
            _ => {}
        }
    }

    fn update_page(&mut self, dt: f32) {
        match self.page {
            0 => self.title.update(dt),
            1 => {
                for b in &mut self.buttons {
                    b.update(dt);
                }
            }
            5 => {
                if self.spring_timer.update(dt) {
                    let to = if self.springs[0].target > 0.5 { 0.0 } else { 1.0 };
                    for s in &mut self.springs {
                        s.target = to;
                    }
                }
                for s in &mut self.springs {
                    s.update(dt);
                }
                if self.count_timer.update(dt) {
                    let next = self.counter.target() + 250 + (self.rng.next_u32() % 2000) as i64;
                    self.counter.set(next);
                    self.punch.kick(0.3);
                    self.shake.add(0.35);
                }
                self.counter.update(dt);
                self.punch.update(dt);
                self.shake.update(dt);
                self.seq.update(dt);
            }
            6 => {
                if self.juice_timer.update(dt) {
                    self.juice_step += 1;
                    let (cx, cy) = (180.0, 300.0);
                    let th = self.theme;
                    match self.juice_step % 6 {
                        0 => {
                            self.fx.confetti(cx, cy, 90, &th.confetti);
                            self.flash.fire(with_alpha(th.gold, 0.35));
                        }
                        1 => self.fx.sparkles(cx, cy, 70.0, 18, ui::lighten(th.gold, 0.4)),
                        2 => self.fx.stars(cx, cy, 12, th.gold),
                        3 => {
                            self.fx.burst(cx, cy, 30, th.accent);
                            self.fx.ring(cx, cy, 110.0, th.text);
                            self.vignette.fire();
                        }
                        4 => self.fx.sparks(cx, cy, -std::f32::consts::FRAC_PI_2, 1.2, 40, th.gold),
                        _ => self.fx.confetti_cannons(self.layout.screen, 80, &th.confetti),
                    }
                    let x = 70.0 + self.rng.f32() * 220.0;
                    self.popups.score(x, 470.0, 50 * (1 + self.juice_step as i64 % 5), th.text);
                    if self.combo.count < 12 {
                        self.combo.hit();
                    } else {
                        self.combo.break_combo();
                    }
                }
                self.fx.update(dt);
                self.popups.update(dt);
                self.combo.window = 3.0;
                self.combo.update(dt);
                self.flash.update(dt);
                self.vignette.update(dt);
            }
            7 => {
                self.results.update(dt);
                if self.page_t > 7.0 {
                    self.page_t = 0.0;
                    self.new_results();
                }
            }
            8 => {
                if self.countdown.update(dt).is_some() {}
                if !self.countdown.active() && self.hud_timer.update(dt) {
                    self.hud_score += 5 + (self.rng.next_u32() % 30) as i64;
                    self.hud.set_score(self.hud_score);
                    let lives = 3 - ((self.hud_score / 120) % 4) as u32;
                    self.hud.lives = Some((lives.min(3), 3));
                    self.hud.progress = Some((self.hud_score % 120) as f32 / 120.0);
                    self.combo.hit();
                }
                self.hud.update(dt);
                self.combo.window = 1.4;
                self.combo.update(dt);
                if self.page_t > 9.0 {
                    self.go(8);
                }
            }
            _ => {}
        }
    }
}

impl Game for Gallery {
    fn init() -> Self {
        sys::round(Round::Idle);
        let layout = Layout::new();
        let mut rng = Rng::from_host();
        let theme = Theme::candy();
        let foot = layout.screen.bottom() - 46.0;
        let mut buttons = Vec::new();
        buttons.push(Button::new(Rect::centered(180.0, 132.0, 220.0, 58.0), "PLAY").with_icon(Icon::PLAY));
        buttons.push(Button::new(Rect::new(40.0, 186.0, 136.0, 48.0), "SCORES").kind(ButtonKind::Secondary).with_icon(Icon::TROPHY));
        buttons.push(Button::new(Rect::new(184.0, 186.0, 136.0, 48.0), "SKIP").kind(ButtonKind::Ghost).with_icon(Icon::ARROW_RIGHT));
        for (i, icon) in [Icon::RESTART, Icon::HOME, Icon::GEAR, Icon::SOUND, Icon::PAUSE].into_iter().enumerate() {
            buttons.push(Button::icon(64.0 + i as f32 * 58.0, 270.0, 50.0, icon));
        }
        let seed = rng.next_u32() as u64;
        let results = ResultsCard::new(1280, 940, theme, &layout, seed);
        let mut g = Gallery {
            layout,
            theme_i: 0,
            theme,
            page: 0,
            t: 0.0,
            page_t: 0.0,
            prev: Button::icon(54.0, foot, 48.0, Icon::ARROW_LEFT).kind(ButtonKind::Secondary),
            next: Button::icon(306.0, foot, 48.0, Icon::ARROW_RIGHT).kind(ButtonKind::Secondary),
            theme_btn: Button::new(Rect::new(250.0, layout.safe.y + 10.0, 96.0, 32.0), "CANDY").kind(ButtonKind::Secondary),
            buttons,
            title: TitleCard::new("BUBBLE POP", theme).tagline("don't let it pop").best(1240),
            springs: [Spring::critical(0.0, 1.6), Spring::bouncy(0.0, 1.6), Spring::wobbly(0.0, 1.6)],
            spring_timer: Timer::new(1.6),
            counter: Counter::new(0),
            count_timer: Timer::new(1.5),
            punch: Punch::new(),
            shake: Shake::new(),
            seq: Seq::new(0.0).to(1.0, 0.5, Ease::BackOut).wait(0.4).to(0.3, 0.3, Ease::QuadIn).wait(0.2).to(0.0, 0.6, Ease::BounceOut).wait(0.3).looped(),
            fx: Particles::new(rng.next_u32() as u64),
            popups: Popups::new(),
            combo: Combo::new(3.0),
            flash: Flash::new(0.4),
            vignette: VignettePulse::new(0xff2040ff),
            juice_timer: Timer::new(1.1),
            juice_step: 0,
            results,
            results_round: 0,
            hud: Hud::new(theme, 120),
            hud_score: 0,
            hud_timer: Timer::new(0.45),
            countdown: Countdown::default(),
            rng,
        };
        g.set_theme(0);
        g
    }

    fn update(&mut self, dt: f32) {
        self.t += dt;
        self.page_t += dt;
        for e in input::poll() {
            match e.kind() {
                Kind::KeyDown if e.code == KEY_RIGHT => self.go(self.page + 1),
                Kind::KeyDown if e.code == KEY_LEFT => self.go(self.page + PAGES.len() - 1),
                Kind::KeyDown if (KEY_1..KEY_1 + 9).contains(&e.code) => self.go((e.code - KEY_1) as usize),
                Kind::KeyDown if e.code == KEY_T => self.set_theme(self.theme_i + 1),
                _ => {}
            }
            if self.prev.handle(&e) == Some(Tap::Clicked) {
                self.go(self.page + PAGES.len() - 1);
                continue;
            }
            if self.next.handle(&e) == Some(Tap::Clicked) {
                self.go(self.page + 1);
                continue;
            }
            if self.theme_btn.handle(&e) == Some(Tap::Clicked) {
                self.set_theme(self.theme_i + 1);
                continue;
            }
            if self.prev.is_held() || self.next.is_held() || self.theme_btn.is_held() {
                continue;
            }
            let mut claimed = false;
            if self.page == 1 {
                for b in &mut self.buttons {
                    claimed |= b.handle(&e).is_some();
                }
            }
            if self.page == 7 {
                if let Some(a) = self.results.handle(&e) {
                    if a == ui::ResultsAction::Retry {
                        self.new_results();
                    }
                    claimed = true;
                }
                claimed |= self.results.claims(e.x, e.y);
            }
            if !claimed && e.is_press() {
                self.poke(e.x, e.y);
            }
        }
        self.prev.update(dt);
        self.next.update(dt);
        self.theme_btn.update(dt);
        self.update_page(dt);
    }

    fn render(&self) {
        let th = &self.theme;
        let l = &self.layout;
        shape::gradient(l.screen, th.bg_top, th.bg_bottom);
        match self.page {
            0 => self.draw_title(),
            1 => self.draw_widgets(),
            2 => self.draw_icons(),
            3 => self.draw_text(),
            4 => self.draw_easing(),
            5 => self.draw_motion(),
            6 => self.draw_juice(),
            7 => self.results.draw(),
            _ => self.draw_hud(),
        }
        // Chrome: page name, theme switch, arrows and dots.
        if self.page != 0 && self.page != 8 {
            text(PAGES[self.page]).font(th.title_font).size(th.fit(th.title_font, 22.0)).color(th.text).outline(0.08, th.outline).valign(VAlign::Middle).draw(16.0, l.safe.y + 26.0);
        }
        if self.page != 8 {
            self.theme_btn.draw(th);
        }
        self.prev.draw(th);
        self.next.draw(th);
        let n = PAGES.len();
        for i in 0..n {
            let x = 180.0 + (i as f32 - (n - 1) as f32 / 2.0) * 16.0;
            let on = i == self.page;
            disc(x, self.prev.rect.cy(), if on { 5.0 } else { 3.5 }, if on { th.accent } else { with_alpha(th.text, 0.35) });
        }
    }
}

impl Gallery {
    fn draw_title(&self) {
        // A stand-in scene: bubbles drifting up behind the title.
        let th = &self.theme;
        for i in 0..14 {
            let k = i as f32;
            let x = 30.0 + (k * 97.0) % 300.0 + (self.t * 0.7 + k).sin() * 12.0;
            let y = 640.0 - ((self.t * (18.0 + k * 3.0) + k * 61.0) % 720.0);
            let r = 10.0 + (k * 7.0) % 26.0;
            shape::ring(x, y, r, 2.5, with_alpha(th.text, 0.25));
            soft_disc(x - r * 0.35, y - r * 0.35, r * 0.18, r * 0.2, with_alpha(0xffffffff, 0.35));
        }
        let pulse = 1.0 + 0.05 * (self.t * 4.0).sin();
        soft_disc(180.0, 380.0, 92.0 * pulse, 30.0, with_alpha(th.accent, 0.35));
        disc(180.0, 380.0, 78.0 * pulse, with_alpha(th.accent, 0.85));
        soft_disc(152.0, 352.0, 16.0, 10.0, 0xffffffb0);
        self.title.draw(&self.layout);
    }

    fn draw_widgets(&self) {
        let th = &self.theme;
        for b in &self.buttons {
            b.draw(th);
        }
        let y = 322.0;
        let f = th.body_font;
        let pills: [(&str, Option<Icon>, u32, u32); 3] = [
            ("1,240", Some(Icon::TROPHY), with_alpha(th.outline, 0.5), th.gold),
            ("×3", None, th.accent, th.on_accent),
            ("LOCKED", Some(Icon::LOCK), with_alpha(th.text, 0.15), th.text_dim),
        ];
        let widths: Vec<f32> = pills.iter().map(|p| ui::pill_width(p.0, 32.0, f, p.1.is_some())).collect();
        for (r, p) in ui::stack_h(Rect::new(0.0, y - 16.0, 360.0, 32.0), &widths, 8.0).iter().zip(pills) {
            ui::pill(p.0, r.cx(), y, 32.0, p.2, p.3, f, p.1);
        }
        let p = Rect::new(24.0, 352.0, 312.0, 196.0);
        ui::panel(p, th);
        let k = (self.t * 0.35).fract();
        ui::progress_bar(Rect::new(44.0, 372.0, 272.0, 18.0), k, th.accent, with_alpha(th.outline, 0.45));
        ui::progress_bar(Rect::new(44.0, 400.0, 180.0, 10.0), 1.0 - k, th.good, with_alpha(th.outline, 0.45));
        ui::segments(Rect::new(236.0, 398.0, 80.0, 14.0), 5, 5.0 * (1.0 - k), th.gold, with_alpha(th.outline, 0.45));
        for (i, frac) in [0.25, 0.6, k].into_iter().enumerate() {
            let cx = 70.0 + i as f32 * 62.0;
            ui::ring_meter(cx, 448.0, 24.0, 7.0, frac, [th.accent, th.good, th.gold][i], with_alpha(th.outline, 0.45));
            text(&format!("{}%", (frac * 100.0) as u32)).font(Font::Sans).size(12.0).color(th.text).middle().draw(cx, 448.0);
        }
        // Shadows, glows, strokes.
        let r1 = Rect::new(250.0, 428.0, 66.0, 40.0);
        shape::glow(r1, 12.0, 16.0, with_alpha(th.accent, 0.7));
        rrect(r1, 12.0, th.accent);
        ui::ribbon("NEW BEST!", 180.0, 505.0, 32.0, th.gold, th.outline, th.title_font, -0.04);
        text("panel · bars · segments · rings · glow · ribbon").font(Font::Sans).size(12.0).color(th.text_dim).center().draw(180.0, 556.0);
    }

    fn draw_icons(&self) {
        let th = &self.theme;
        let cols = 6;
        let (cw, ch) = (54.0, 64.0);
        let x0 = 180.0 - cw * cols as f32 / 2.0 + cw / 2.0;
        let y0 = self.layout.safe.y + 92.0;
        for (i, (name, icon)) in Icon::ALL.iter().enumerate() {
            let (cx, cy) = (x0 + (i % cols) as f32 * cw, y0 + (i / cols) as f32 * ch);
            let k = Ease::BackOut.at(stagger(self.page_t, i, 0.02, 0.35));
            if k <= 0.0 {
                continue;
            }
            rrect(Rect::centered(cx, cy, 46.0 * k, 46.0 * k), 12.0, with_alpha(th.outline, 0.35));
            let col = match i % 4 {
                0 => th.text,
                1 => th.accent,
                2 => th.gold,
                _ => th.good,
            };
            icon.draw(cx, cy, 30.0 * k, col);
            text(name).font(Font::Sans).size(10.0).color(th.text_dim).center().draw(cx, cy + 25.0);
        }
    }

    fn draw_text(&self) {
        let th = &self.theme;
        let mut y = self.layout.safe.y + 70.0;
        text("Outline").size(46.0).color(th.text).outline(0.08, th.outline).center().draw(180.0, y);
        y += 62.0;
        text("Soft shadow").size(34.0).color(th.text).soft_shadow(0.0, 5.0, 0.08, th.shadow).center().draw(180.0, y);
        y += 50.0;
        gfx2d::blend(gfx2d::Blend::Add);
        text("Glow").size(36.0).color(th.text).glow(0.1, with_alpha(th.accent, 0.9)).center().draw(110.0, y);
        gfx2d::blend(gfx2d::Blend::Alpha);
        text("Heavy").size(36.0).color(th.gold).weight(0.035).outline(0.04, th.outline).center().draw(250.0, y);
        y += 52.0;
        text("Light weight").font(Font::Sans).size(28.0).weight(-0.02).color(th.text).center().draw(180.0, y);
        y += 44.0;
        text("PIXEL OUTLINE").font(Font::Pixel).size(24.0).color(th.gold).outline(0.125, th.outline).shadow(0.0, 3.0, with_alpha(th.outline, 0.6)).center().draw(180.0, y);
        y += 38.0;
        text("S P A C E D").size(20.0).tracking(0.1).color(th.text_dim).center().draw(180.0, y);
        y += 36.0;
        // Alignment against a guide.
        capsule(180.0, y - 4.0, 180.0, y + 70.0, 2.0, with_alpha(th.accent, 0.6));
        for (i, a) in [Align::Left, Align::Center, Align::Right].into_iter().enumerate() {
            text(["left", "center", "right"][i]).font(Font::Sans).size(18.0).color(th.text).align(a).draw(180.0, y + i as f32 * 24.0);
        }
        y += 84.0;
        let p = Rect::new(36.0, y, 288.0, 92.0);
        ui::panel(p, th);
        text("Wrapped text flows onto new lines to fit the width you give it, centred or not.").font(Font::Sans).size(16.0).color(th.text).wrap(p.w - 32.0).center().valign(VAlign::Middle).draw(p.cx(), p.cy() - 18.0);
    }

    fn draw_easing(&self) {
        let th = &self.theme;
        let cols = 4;
        let (cw, ch) = (82.0, 60.0);
        let x0 = 180.0 - cw * cols as f32 / 2.0;
        let y0 = self.layout.safe.y + 54.0;
        let k = ((self.t * 0.6).fract() * 1.3).min(1.0);
        for (i, e) in Ease::ALL.iter().enumerate() {
            let (x, y) = (x0 + (i % cols) as f32 * cw, y0 + (i / cols) as f32 * ch);
            let cell = Rect::new(x + 4.0, y + 4.0, cw - 8.0, ch - 8.0);
            rrect(cell, 8.0, with_alpha(th.outline, 0.35));
            let g = cell.inset_sides(8.0, 14.0, 8.0, 10.0);
            let mut pts = Vec::with_capacity(48);
            for j in 0..=40 {
                let u = j as f32 / 40.0;
                pts.extend([g.x + u * g.w, g.bottom() - e.at(u) * g.h]);
            }
            shape::polyline(&pts, 1.6, ui::mix(th.text, th.panel_bottom, 0.2));
            disc(g.x + k * g.w, g.bottom() - e.at(k) * g.h, 3.5, th.accent);
            text(&format!("{e:?}")).font(Font::Sans).size(9.0).color(th.text_dim).draw(cell.x + 5.0, cell.y + 2.0);
        }
    }

    fn draw_motion(&self) {
        let th = &self.theme;
        let top = self.layout.safe.y + 64.0;
        let names = ["critical", "bouncy", "wobbly"];
        for (i, s) in self.springs.iter().enumerate() {
            let y = top + i as f32 * 46.0;
            text(names[i]).font(Font::Sans).size(13.0).color(th.text_dim).valign(VAlign::Middle).draw(20.0, y);
            capsule(100.0, y, 320.0, y, 4.0, with_alpha(th.outline, 0.4));
            let x = 110.0 + s.value * 200.0;
            disc(x, y, 14.0, [th.accent, th.gold, th.good][i]);
        }
        // A sequence: up, hang, dip, bounce down.
        let sy = top + 150.0;
        text("sequence").font(Font::Sans).size(13.0).color(th.text_dim).draw(20.0, sy - 6.0);
        let v = self.seq.value();
        Icon::STAR.draw(150.0, sy + 50.0 - v * 50.0, 30.0 + v * 10.0, th.gold);
        // Stagger.
        text("stagger").font(Font::Sans).size(13.0).color(th.text_dim).draw(200.0, sy - 6.0);
        let lt = (self.t * 0.5).fract() * 2.0;
        for i in 0..6 {
            let k = Ease::BackOut.at(stagger(lt, i, 0.08, 0.4));
            rrect(Rect::centered(215.0 + i as f32 * 20.0, sy + 40.0 - k * 26.0, 14.0, 14.0), 4.0, fade(th.accent, k.clamp(0.0, 1.0)));
        }
        // Counter with punch inside a shaking panel.
        let p = Rect::new(40.0, top + 240.0, 280.0, 150.0);
        gfx2d::push();
        self.shake.apply(p.cx(), p.cy());
        ui::panel(p, th);
        text("counter + punch + shake").font(Font::Sans).size(13.0).color(th.text_dim).center().draw(p.cx(), p.y + 14.0);
        gfx2d::push();
        gfx2d::translate(p.cx(), p.cy() + 10.0);
        let s = self.counter.scale() * self.punch.scale();
        gfx2d::scale(s, s);
        text(&ui::fmt_int(self.counter.value())).font(th.number_font).size(th.fit(th.number_font, 56.0)).color(th.gold).outline(th.outline_em, th.outline).middle().draw(0.0, 0.0);
        gfx2d::pop();
        gfx2d::pop();
        text("tap to punch and shake").font(Font::Sans).size(13.0).color(th.text_dim).center().draw(180.0, p.bottom() + 16.0);
    }

    fn draw_juice(&self) {
        let th = &self.theme;
        self.fx.draw();
        self.popups.draw();
        self.combo.draw(300.0, self.layout.safe.y + 100.0, th);
        text("confetti · sparkles · stars · burst · sparks · cannons").font(Font::Sans).size(12.0).color(th.text_dim).center().draw(180.0, 530.0);
        text("tap for more").font(Font::Sans).size(12.0).color(th.text_dim).center().draw(180.0, 548.0);
        self.vignette.draw(self.layout.screen);
        self.flash.draw(self.layout.screen);
    }

    fn draw_hud(&self) {
        let th = &self.theme;
        let l = &self.layout;
        // Shade the pause-pill zone the HUD keeps clear.
        let z = l.pill_zone();
        rrect(z.inset(4.0), 10.0, with_alpha(th.text, 0.08));
        text("pause pill").font(Font::Sans).size(10.0).color(th.text_dim).middle().draw(z.cx(), z.cy());
        self.hud.draw(l);
        self.combo.draw(64.0, l.safe.y + 130.0, th);
        self.countdown.draw(l, th);
    }
}

export_game!(Gallery);
