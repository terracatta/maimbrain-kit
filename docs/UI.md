# UI and juice kit

The SDK ships a UI and motion kit so a game's menus, HUD and "juice" look finished without hand-rolling them: `maimbrain::motion` (easing, tweens, springs, shake, counters), `maimbrain::juice` (confetti, sparkles, popups, a combo meter, flashes) and `maimbrain::ui` (themes, shapes, text, icons, buttons, meters, layout, and ready-made title, results and HUD screens). It's plain Rust on top of mb2d (SPEC §5.3) plus three small host calls for anti-aliased shapes and styled text.

`games/gallery` shows every piece on nine pages under the 17 identities and the five original themes. Build it with `mb serve games/gallery` and page with the arrows (or ← →, 1–9; T or the button top-right cycles the theme). It draws in Inter: the identities' fonts aren't baked into it (see [Identities](#identities) for them with their fonts).

## The rules it keeps

- **Determinism.** Everything advances only by the `dt` you pass to `update` and draws from its own seeded `Rng`, so runs replay exactly (SPEC §3). Update kit state in `Game::update`, draw it in `Game::render`, seed particle pools from your game's `Rng` (or `sys::rand_seed()`), never from anything else.
- **Feed cards.** The title and game-over screens are feed cards (SPEC §5.2): live, untouchable, with the platform's title strip over the bottom ~70 pt and buttons down the right ~56 pt. `Layout::card` is the area clear of both (symmetric, so centred things stay centred); the screens keep their text and buttons inside it. The title card sells (a name, a hook, the best), never "tap to play".
- **The platform's own UI.** The pause pill (top-left in play) and leaderboards are the platform's. The kit has no pause button or score list; `Layout::pill_zone()` is the corner to keep clear, and a SCORES button should call `store::show_scores`.
- **Size.** Only what a game calls is linked. Icons are function pointers, so naming `Icon::TROPHY` links the trophy alone. Measured with `mb build` (wasm-opt): the 2D template went from 24.6 KB to 84.0 KB of wasm (+25 KB gzipped) with the title card, HUD, results card and pop juice; Stack from 42.3 KB to 102.3 KB (+25 KB gzipped). The styles and identities add more, since the screens can draw every layout: the 2D template using `Theme::from_identity` is 160.9 KB, Stack 192.5 KB, the gallery 222.9 KB. Each game font adds its atlas (25–60 KB, shown by `mb font add`).

## Host calls (mb2d)

| Call | What |
|---|---|
| `gfx2d::rrect(x, y, w, h, radius, stroke, feather, top, bottom)` | Signed-distance rounded rect, anti-aliased under any transform. `radius` clamps to half the short side (pills, circles); `stroke` > 0 draws only a border; `feather` blurs the edge (shadows, glows, soft dots); vertical gradient. |
| `gfx2d::text_style(TextStyle { weight, outline, outline_rgba, soft })` | How distance-field fonts (Inter and game fonts) draw until changed or the frame ends: bolder or lighter, an outline, a soft outer edge. In ems, up to ~0.12 em in total. |
| `gfx2d::antialias(true)` | `poly` gets a one-pixel soft fringe and `line` smooth edges until turned off or the frame ends. |

Game fonts (mb2d 2) are `Font::asset("assets/fonts/<name>.mbf", fallback)`, baked by `mb font add`: a `Font` is a small `Copy` value usable in any theme or text call, drawing in its fallback until its asset is ready. The kit wraps these; most games never call them directly. The Inter SDF atlas is baked with exact distances at 8× oversampling (`tools/fontbake`), so outlines and glows stay smooth. Text is crisp at any size; measuring is unchanged.

## motion

```rust
use maimbrain::motion::*;

let mut fade = Tween::new(0.0, 1.0, 0.4, Ease::CubicOut).delay(0.2);   // any Lerp value: f32, [f32; N]
let mut pos = Spring::bouncy(0.0, 3.0);  pos.target = 120.0;           // critical(), bouncy(), wobbly(), new(v, hz, damping)
let mut seq = Seq::new(0.0).to(1.0, 0.3, Ease::BackOut).wait(0.5).to(0.0, 0.2, Ease::QuadIn);
let k = Ease::BackOut.at(stagger(t, i, 0.05, 0.4));                    // item i of a staggered group
let mut shake = Shake::new();  shake.add(0.5);                          // trauma: 0.25 small hit, 0.8 death
let mut punch = Punch::new();  punch.kick(0.3);                         // scale() jumps and springs back
let mut score = Counter::new(0);  score.set(1500);                      // rolls up; update() says when it ticked
let mut stop = HitStop::default();  stop.freeze(0.06);                  // sim_dt = stop.step(dt)
let mut flash = Pulse::new(0.3);  flash.fire();                         // value(): 1 → 0
// each frame, in update:   fade.update(dt); pos.update(dt); seq.update(dt); shake.update(dt); ...
// in render:               gfx2d::push(); shake.apply(180.0, 320.0); /* scene */ gfx2d::pop();
```

`Ease` has linear and quad, cubic, quart, quint, sine, expo, circ, back, elastic and bounce in in/out/in-out (`Ease::ALL`). Springs are integrated exactly, so they're stable at any `dt`. Helpers: `lerp`, `inv_lerp`, `remap`, `damp` (frame-rate independent smoothing), `approach`, `noise1`, `Timer`.

## juice

```rust
use maimbrain::juice::*;

let mut fx = Particles::new(rng.next_u32() as u64);   // a pool (600 max; the oldest give way)
fx.confetti(x, y, 80, &theme.confetti);               // paper that tumbles and flutters
fx.confetti_cannons(layout.screen, 80, &theme.confetti);
fx.sparkles(x, y, 60.0, 12, 0xfff2b0ff);              // twinkles
fx.stars(x, y, 10, theme.gold);                       // spinning stars bursting out
fx.burst(x, y, 20, theme.accent);                     // soft additive puff
fx.sparks(x, y, -FRAC_PI_2, 1.0, 30, theme.gold);     // streaks in a cone
fx.ring(x, y, 120.0, 0xffffffc0);                     // shockwave
fx.celebrate(x, y, &theme);                           // ring + stars + sparkles + confetti
let mut popups = Popups::new();  popups.score(x, y, 100, theme.gold);   // "+100" pops, rises, fades
let mut combo = Combo::new(2.0);  combo.hit();        // breaks after 2 s without a hit; multiplier(); draw()
let mut flash = Flash::new(0.3);  flash.fire(0xffffff60);
let mut hurt = VignettePulse::new(0xff2040ff);  hurt.fire();             // or hurt.level = 0.4 for a steady warning
// update: fx.update(dt); popups.update(dt); combo.update(dt); ...   render: fx.draw(); popups.draw(); ...
```

A `Particle` is a plain struct (look, position, velocity, gravity, drag, life, size, color, spin) if you want your own presets: build one and `fx.add(p)`.

## ui

**Theme.** One struct of colors, fonts and a `Style` that everything reads. Start from the game's identity: `Theme::from_identity("haunted carnival, crimson, jittery")` or `Theme::preset("saloon")` (below), then `.load_fonts()` for the library fonts it was designed with, and change any field. `with_accent`, `with_panel`, `with_radius`, `with_font`, `with_style`; `theme.backdrop(rect, t)` paints its background; `theme.case(s)` applies its letter case. The original `Theme::candy()`, `night()`, `arcade()`, `paper()` and `jungle()` remain for existing games, but they are the house look docs/IDENTITY.md describes: don't start new games from them.

**Style** (`theme.style`, ui/style.rs): what makes themes look different beyond color.
- `shape`: `Rounded`, `Sharp`, `Pill`, `Cut` (chamfered), `Wobbly` (hand-drawn), `Pixel` (stepped corners), `Bevel`, `Ticket` (punched notches). Panels, buttons (the round retry button becomes a square, octagon or stepped block), pills and badges follow it.
- `border`: `None`, `Line(w)`, `Double(w)`, `Dashed(w)`. `shadow`: `Soft`, `Hard(dx, dy)` (a solid offset copy: print, neo-brutalism; buttons sink into it), `Glow` (neon), `None`.
- `texture` on panels and `backdrop_texture` on the background: `Grain`, `Scanlines`, `Halftone`, `Stripes`, `Grid`, `Lines` (ruled paper). `backdrop`: `Gradient`, `Flat`, `Rays`, `Vignette`, `Horizon` (a synthwave grid floor).
- `title`: `Drop` (letters drop in and bob), `Stack` (one word per line, left-aligned and huge, with a rule), `Banner` (on a tilted band), `Stamp` (a rubber stamp thumped down), `Typewriter` (typed out with a cursor), `Marquee` (a sign ringed with chasing bulbs), `Arc` (letters on an arch), `Neon` (a tube that flickers on), `Plate` (an engraved plaque with ornaments).
- `results`: `Panel`, `Floating`, `Receipt` (a printed slip with dotted rows), `Scoreboard` (digits in tiles, a lamp for a new best), `Stamp` (the heading stamped over a huge number), `Editorial` (left-aligned label, rule, huge number). `hud`: `Center`, `Corner` (top-right), `Badge` (in a shaped badge).
- `motion`: `Bouncy`, `Snappy`, `Floaty`, `Mechanical` (stepped), `Jittery` (small jolts), `Elastic`. Entrances, idles and staggers in every screen use it (`Motion::enter`, `progress`, `idle`, `spring` for your own animation).
- `case` (`AsWritten`, `Upper`, `Lower`), `tracking` (ems, titles and headings) and `outlined` (titles get `theme.outline` around them).

**Shapes** (`ui::shape`): `rrect`, `rrect_gradient`, `rrect_stroke`, `shadow`, `glow`, `disc`, `soft_disc`, `ring`, `arc` (round-ended bands: progress rings, gauges), `capsule`, `polyline`, `poly` (anti-aliased), `rounded_poly` / `rounded_points` (fillet any polygon's corners), `star`, `sparkle`, `gradient`, `vignette`, `nine_slice` (an image region stretched without stretching its corners).

**Text.**
```rust
ui::text("STACK").size(64.0).color(theme.text)
    .outline(0.08, theme.outline).soft_shadow(0.0, 4.0, 0.06, theme.shadow)
    .middle().draw(180.0, 120.0);                         // returns the bounds
ui::text(long).font(Font::Sans).size(16.0).wrap(280.0).center().draw(180.0, 400.0);
let (w, h) = ui::text("BEST 42").size(18.0).measure();
```
`align(Left | Center | Right)`, `valign(Top | Middle | Baseline | Bottom)` (Middle centres the capitals: right for buttons), `glow`, `weight`, `tracking`, `leading`, `alpha`. The pixel font gets outlines and shadows from offset copies a whole art pixel apart; keep it at multiples of 8 (`theme.fit(font, size)` rounds for you). `fmt_int(12345)` → `"12,345"`.

**Icons.** `Icon::PLAY.draw(cx, cy, size, color)`: play, pause, restart, star, heart, trophy, crown, lock, unlock, check, close, plus, minus, four arrows, home, gear, sound, mute, music, coin, bolt, flag, clock, medal, fire, gem, shield, target, person, bomb (`Icon::ALL`). Original drawings, one color with darker or lighter details, any size.

**Widgets.**
```rust
let mut play = Button::new(Rect::centered(180.0, 420.0, 200.0, 56.0), "PLAY").with_icon(Icon::PLAY)
    .with_haptic(|| sensors::haptic(Haptic::Tap));        // needs sensors = ["haptics"]
let mut retry = Button::icon(180.0, 500.0, 64.0, Icon::RESTART);
// update: for e in &events { if play.handle(e) == Some(Tap::Clicked) { start() } }  play.update(dt);
// render: play.draw(&theme);
ui::panel(rect, &theme);
ui::pill("BEST 42", cx, cy, 32.0, bg, fg, theme.body_font, Some(Icon::TROPHY));
ui::progress_bar(rect, 0.6, theme.accent, track);
ui::segments(rect, 5, 3.5, theme.gold, track);
ui::ring_meter(cx, cy, 26.0, 7.0, 0.25, theme.accent, track);
ui::ribbon("NEW BEST!", cx, cy, 40.0, theme.gold, theme.outline, theme.title_font, -0.05);
```
Buttons are `Primary` (raised, sinks when pressed), `Secondary` (translucent with a border) or `Ghost`; the touch area is at least 44 × 44 even when the button looks smaller. `handle` returns `Pressed` (going down: a tick, a haptic), `Clicked` (released on it) or `Cancelled`; any `Some` means the event was the button's, so don't also treat it as a tap on the game.

**Layout.** `Layout::new()` (once, in init) gives `screen`, `safe` (inside the insets), `card` (clear of the card overlays) and `hud`, plus `pill_zone()`. `Rect` has `centered`, `inset`, `place(Anchor, w, h, margin)`, `split_top/bottom/left`, `columns`, `rows`, `at_least`, `contains`, `lerp`. `Column` hands out rows top to bottom; `stack_v` / `stack_h` lay out a group centred in an area.

## Identities

Seventeen complete themes that look nothing alike, each designed around fonts from the library. `Theme::preset(name)` returns one; `Theme::from_identity(words)` picks the closest to a description and applies any colors, shapes, motions, textures and layouts it names (its docs list the words). `mb font add <game> --identity <name>` bakes an identity's fonts, and `.load_fonts()` points the theme at them. The contact sheet [identity/identities.jpg](identity/identities.jpg) shows each one's title, results and HUD with its fonts.

| Identity | Look | Shape · layouts · motion | Fonts (title / body / numbers) |
|---|---|---|---|
| `riso` | Risograph zine: cream paper, fluoro pink, teal misregistration, grain | sharp, hard teal shadow · stack / receipt / corner · snappy | Bricolage Grotesque 800 / Space Mono / Bricolage 800 |
| `crt` | Phosphor-green arcade monitor, scanlines, glow | pixel-stepped · marquee / scoreboard / corner · mechanical | Press Start 2P / VT323 / Press Start 2P |
| `neon` | Synthwave night: magenta and cyan tubes over a grid floor | pill, glow · neon / floating / center · floaty | Monoton / Exo 2 / Orbitron 700 |
| `storybook` | Picture book: parchment, forest green, hand-drawn frames | wobbly · arc / panel / badge · floaty | Fraunces 800 / Patrick Hand / Fraunces 800 |
| `saloon` | Wild west: dark wood, brass, red, ticket stubs | ticket, double border, hard shadow · plate / stamp / badge · mechanical | Rye / Zilla Slab 700 / Rye |
| `swiss` | International style: white, black, one red, lowercase | sharp, no shadow · stack / editorial / corner · snappy | Archivo 800 / Archivo / Archivo 800 |
| `brutalist` | Neo-brutalism: yellow, thick black borders, 6 px offset shadows | sharp · banner / panel / badge · snappy | Rubik Mono One / Space Grotesk 700 / Rubik Mono One |
| `comic` | Comic book: halftone sunburst, red and yellow, outlined caps | sharp, hard shadow · stamp / stamp / badge · elastic | Bangers / Comic Neue 700 / Bangers |
| `gothic` | Blackletter and blood red on black, candle flicker | chamfered, double border · drop / editorial / center · jittery | UnifrakturMaguntia / EB Garamond / EB Garamond 700 |
| `bubblegum` | Pastel pink to mint, soft pills, rounded type | pill · drop / panel / center · bouncy | Fredoka 700 / Nunito 800 / Fredoka 700 |
| `terminal` | Amber monochrome console, dashed boxes, lowercase | sharp, dashed · typewriter / receipt / corner · mechanical | JetBrains Mono 800 / JetBrains Mono / JetBrains Mono 800 |
| `field` | Military kit: olive drab, khaki, hazard yellow, stencils | chamfered · stamp / stamp / corner · mechanical | Black Ops One / Chakra Petch 700 / Big Shoulders Stencil 800 |
| `orbital` | Spacecraft HUD: black-blue, cyan lines, grid | chamfered, glow · typewriter / scoreboard / corner · snappy | Orbitron 900 / Chakra Petch / Tektur 700 |
| `gala` | Art deco black tie: black and gold, wide-tracked didone | sharp, double border · plate / editorial / center · floaty | Playfair Display 900 / EB Garamond / Playfair Display 700 |
| `notebook` | Ballpoint doodles on ruled paper, red pen, highlighter | wobbly · stack / receipt / badge · jittery | Caveat 700 / Patrick Hand / Caveat 700 |
| `stadium` | Scoreboard sports: navy, orange, stripes, condensed caps | sharp, hard shadow · banner / scoreboard / badge · snappy | Big Shoulders Display 900 / Barlow Condensed 700 / Jersey 10 |
| `cozy-pixel` | Cozy pixel RPG: moss green, pumpkin, cream | pixel-stepped, hard shadow · drop / panel / badge · mechanical | Pixelify Sans 700 / Pixelify Sans / Pixelify Sans 700 |

An identity is a starting point, not a uniform: change the palette and fonts to the game's own (the skill's identity.md). Two games on the same preset will still look alike.

## Screens

**`TitleCard`**: the name in the theme's title layout (dropping in letter by letter, stacked, on a banner, stamped, typed, on a marquee, arched, in neon, on a plaque), a tagline, the best in a pill. It leaves the middle of the screen to your live scene.

**`ResultsCard`**: in the theme's results layout (receipt, scoreboard, stamp, editorial; or the original:) a heading straddling a panel, the score rolling up (punched when it lands), then the best, or, if the score beat a previous best, a "NEW BEST!" ribbon stamping in with confetti, cannons and a gold rim; "3 MORE TO BEAT IT" when it was close; a round retry button; an optional extra button. `.floating()` drops the panel and floats outlined text near the top, for games whose aftermath is the card (Stack's tower). `update` returns `Tick`, `NewBest` or `Landed` for sounds and haptics. A first-ever score shows as the best without the celebration.

**`Hud`**: the score big at the top (rolling and punching), the best under it until it's beaten, then a gold NEW BEST marker; optional hearts (`lives`) top-right and a progress bar. It stays clear of the pause pill.

**`Countdown`**: "3, 2, 1, GO!" with a draining ring. Start it in `Game::resume` (a recorded lifecycle call) when your game needs a run-up after a pause, and skip the simulation while `active()`. Since everything else runs on `update(dt)`, the rest of the kit simply freezes while the platform has the game paused.

## Recipes

**A title screen in 10 lines**
```rust
// fields:  layout: Layout, title: TitleCard
let layout = Layout::new();
let theme = Theme::from_identity("seed packet, 1950s, mustard and teal").load_fonts();
let title = TitleCard::new("BUBBLE", theme).tagline("don't let it pop").best(best as i64);
// update:
self.title.update(dt);
// render, after drawing your live scene:
self.title.draw(&self.layout);
// any tap starts the round (the card never asks for one)
```

**A results card**
```rust
// on game over, after store::submit_score and before sys::round(Round::Over):
let prev = self.best;                                         // the best before this round
self.results = Some(ResultsCard::new(score, prev, theme, &self.layout, self.rng.next_u32() as u64)
    .heading("SPLAT!").label("POINTS")
    .extra(Button::new(Rect::default(), "SCORES").kind(ButtonKind::Secondary).with_icon(Icon::TROPHY)));
// update: feed events, then advance
if let Some(card) = &mut self.results {
    for e in &events {
        match card.handle(e) {
            Some(ResultsAction::Retry) => restart = true,
            Some(ResultsAction::Extra) => store::show_scores(BOARD),
            None if e.is_press() && !card.claims(e.x, e.y) && guard_passed => restart = true, // the whole screen retries
            None => {}
        }
    }
    if card.update(dt) == Some(ResultsEvent::NewBest) { sfx.play(FANFARE) }
}
// render:
if let Some(card) = &self.results { card.draw() }
```

**A combo meter**
```rust
// fields: combo: Combo, popups: Popups, fx: Particles
self.combo = Combo::new(1.5);                    // 1.5 s to keep it going
// on each success:
let n = self.combo.hit();
self.popups.score(x, y, 10 * self.combo.multiplier() as i64, theme.gold);
if n % 10 == 0 { self.fx.stars(x, y, 10, theme.gold) }
// on a miss:
self.combo.break_combo();
// update (returns Some(count) when a combo times out):
if let Some(lost) = self.combo.update(dt) { if lost >= 5 { sfx.play(AWW) } }
// render:
self.combo.draw(300.0, layout.hud.y + 90.0, &theme);
```

**Juice on a hit**: `shake.add(0.3); punch.kick(0.25); stop.freeze(0.05); fx.burst(x, y, 16, color); popups.score(x, y, 50, theme.gold);` and play the hit sound pitched up with the combo.

The 2D and 3D templates (`mb new`, `mb new --3d`) start with the title card, HUD and results card wired up, and `games/stack` uses all three plus popups, sparkles and confetti.
