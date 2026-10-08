---
name: maimbrain-game
description: Design and build a Maimbrain game (a short Rust→wasm game for the iPhone feed) — from a brief to a validated, playable, genuinely engaging .mbx. Use when creating a new game under games/, reworking an existing one, adding art or sound to one, or tuning how one feels.
---

# Making a Maimbrain game

A Maimbrain game lives in a vertical feed, between other games, in front of someone with a thumb on the glass and about one second of patience. It isn't installed, chosen or explained. Building a *valid* game is the easy part (`docs/SPEC.md` + `mb build`). Building one people stop swiping for is the job.

Read these before writing code:
1. **`docs/SPEC.md`**: the contract. Especially §2 (manifest), §3 (determinism), §5.2 (feed navigation: browse vs play, the card overlay areas), §5.7 (sensors only in play), §9 (build, preview, test).
2. **[engagement.md](engagement.md)**: what makes a game work *in this feed*. Its rules are requirements, not suggestions.
3. **[kits/](kits/)**: if the brief is a familiar genre (runner, stacker, shooter, match-3, racer, tower defense, trivia), start from that genre's kit instead of the template: see [Start from a kit](#start-from-a-kit). Otherwise, **[template/](template/)**: a tiny complete game (tap the bubble) to start from; **[template-3d/](template-3d/)** is the 3D one (catch the drone; see the 3D section). `mb new games/<name>` creates a game from the 2D template, `mb new --3d games/<name>` from the 3D one. Signed in (`mb login`), the id and creator come from your account; signed out it still works, with a placeholder id and creator (or pass `--id dev.you.name --creator @you`), which build and preview fine but must become your namespace before `mb publish`. You can also copy a template to `games/<name>/` by hand and replace `NAME` in `Cargo.toml` and `manifest.toml` (and `icon.png` with your own). Keep its shape: pure rules in `src/sim.rs` with tests, and a thin host layer in `src/lib.rs` that handles tap to start, round reporting, score-before-Over, the retry guard and saving the best score.
4. **`sdk/maimbrain/src/`**: the SDK. Read the core (lib, sys, input, gfx2d, audio, store, sensors) in full; for the UI and juice kit (`ui/`, `motion.rs`, `juice.rs`), **`docs/UI.md`** is the tour and the section [UI and juice](#ui-and-juice) below has the recipes.
5. **[identity.md](identity.md)**: the art-direction step. Pick and write down the game's own look (reference, palette, a font pairing from the library, shape language, motion, one signature idea) before building, so it doesn't come out looking like every other Maimbrain game (docs/IDENTITY.md has the measurements).
6. **[assets.md](assets.md)** before making art or sound: when to generate it (`mb art`, `mb music`, `mb sfx`) and when to draw it in code, the style guide, prompts, iterating, checking results, budgets.
7. **`docs/PHYSICS.md`** if the game has things that stack, topple, bounce, roll, swing or break (see the Physics section).

## Start from a kit

The kits are small, complete, polished games, one per common genre, that already do everything this skill asks: a live title card, the UI kit's HUD and results card, juice, sound and music, a difficulty curve tuned with a bot, rounds, score submission, the retry guard, the best score, a daily mode where it fits, pure rules in `src/sim.rs` with tests, and the tuning knobs at the top of that file. Starting from one puts you at "good" on the first build; your job becomes making it *yours*.

```sh
mb kits                                  # the list, with what each plays like
mb new --kit runner games/<name>         # copies the kit; same --id/--creator rules as the templates
```

| Kit | Ships as | Tech | Pick it when the brief is… |
|---|---|---|---|
| `runner` | Bun Run | 2D | an endless runner, one-tap jumper, "dodge things as you go", auto-scrolling platformer |
| `stacker` | Picnic Pile | 2D + physics2d | stacking, balancing, dropping things on a pile, "how high can you build", anything wobbly |
| `shooter` | Nova Pip | 2D | a vertical shooter, bullet dodger, space/sky/sea shoot-'em-up, waves and a boss |
| `match3` | Poppets | 2D | a swap or match puzzle, gems/candies/tiles, cascades and combos |
| `racer` | Swerve | 3D (mb3d 2) | driving, lane racing, traffic dodging, anything "behind the vehicle" in 3D |
| `tower-defense` | Cake Keep | 2D | defending a base, placing towers or units, lanes and waves |
| `trivia` | Bright Spark | 2D, text | a quiz, questions and answers, true/false, guess-the-thing |

Don't force a kit onto a brief it doesn't fit: games built on a sensor (tilt, mic, shake), rhythm games (`games/summoned`), toys and anything with an unusual verb start from the template. A brief that mixes genres ("a runner where you shoot") starts from the kit with the core verb and borrows from another (copy the module; kits share the same shape).

**After `mb new --kit`:**
1. Read `games/<name>/DESIGN.md` (how it plays, the tuning knobs, "Make it yours", the bot's numbers) and skim `src/`. `cargo test -p <name>` and `mb build` pass as copied; keep them passing.
2. Rewrite DESIGN.md for *your* game before changing code (the workflow below still applies: the verb, the 1-second read, the first 10 seconds…).
3. Make the three changes below, then re-tune with the bot (`cargo test -p <name> --release -- --ignored --nocapture difficulty`), preview, `mb.verify()`, screenshots, the engagement checklist.

**Make it yours: three changes, every time.** A kit played as is feels like every other game made from it. Do all three:
1. **A new identity.** Theme, character, world, title, tagline, palette and sounds, changed together (each kit groups its look in one place; its art and audio come from the generators: edit `art/style.toml` and the prompts in `tools/regen.sh`, then rerun it, or `mb regen` one asset; see [assets.md](assets.md)). A stranger who has seen the kit must not recognize your title card. Never ship the kit's title, character or tagline.
2. **A twist on the verb or a rule.** One change to *how it plays*, not just how it looks: a runner that flips gravity instead of jumping, a stacker whose platform tilts, a match-3 where matches push the board up, a racer that drifts, trivia where you pick the wrong answer. Each kit's DESIGN.md lists ideas and the code to touch.
3. **One new thing.** A mechanic, enemy, power-up or event the kit doesn't have, introduced 15–30 s in, so the second half of a round surprises.

Also change the fail moment and the game-over card's aftermath (they're the most-shared screens), and keep what the kit gets right: the sim/test/bot split, the round and score hooks, the retry guard, determinism (`mb.verify()` must still match).

## Workflow

### 1. Design on one page first
Write `games/<name>/DESIGN.md` before any code, answering:
- **The verb**: the one thing the player does (tap, hold, swipe, tilt). If you need two, the second must come later.
- **The 1-second read**: what a stranger understands, and why they'd want to play, from a single still frame of the title card. It's an ad, not instructions. Sketch it.
- **The first 10 seconds**: exactly what happens after the first tap, beat by beat. The player must succeed at something within 3 s.
- **Tension and failure**: what makes it hard, how the player sees danger coming, and why failing is *funny or satisfying*, not just over.
- **The loop**: round length (aim for 30–90 s for a typical player), what escalates, and what's new every ~15 s.
- **The cards**: what the title card, the game-over card and a replay card each show (engagement.md §Cards).
- **Score**: what's counted, shown big, and saved as best.

### 1b. Art direction: choose an identity
Follow [identity.md](identity.md): add an `## Identity` section to DESIGN.md with a reference era or medium, a palette, a font pairing from the library (`mb fonts`, `mb fonts --preview`), a shape language, a motion personality, the title/results/HUD layouts, and one signature visual idea. Set `IDENTITY` in `src/lib.rs` to match (`mb new` puts a random kit identity there so new games don't all start alike; don't ship it unexamined). Bake the fonts with `mb font add`. Avoid the house defaults that docs/IDENTITY.md lists: pixel-font titles with hard shadows, purple gradients, centred "DON'T…" taglines, "SPLAT!" + big number + "NEW BEST!" cards.

### 2. Simulation first, in plain Rust, with tests
Split the crate: pure simulation modules (state, rules, physics, difficulty) that take inputs as arguments, plus one thin host layer (`lib.rs` in the template, or a `game.rs` it calls) that reads input/sensors, calls the sim, plays sound/haptics, and draws. The SDK compiles on the host with no-op stubs (SPEC §9), so `cargo test -p <name>` runs your sim natively. Put tests in `src/` (`#[cfg(test)] mod tests;`): the crate is a `cdylib`, so `tests/` can't link it unless you add `"rlib"` to `crate-type`.

**Write a bot** that plays the sim like a person, so you can measure difficulty: how long the first round lasts, where players die, whether the first seconds are unfailable. A bot is only as good as its model of a human. A plain controller makes every skill level die at the same place. Model at least:
- **reaction delay**: it acts on what it saw 150–350 ms ago
- **motor noise**: jitter in *control* units, scaled by your control's gain (a 2.6× tilt gain multiplies hand tremor by 2.6)
- **overshoot** on the first move of each new situation, then **caution** (smaller corrections) after an overshoot
- **anticipation** for skilled players: reacting to tells before events

Give it three presets and report each one's median round length over ~200 seeded runs:
- **first-timer**: slow, noisy, no anticipation; should die in 20–45 s
- **decent**
- **good**: should last 1–3 min

Model the specific skills your game demands too (holding still to aim, easing off near a limit, leading a moving target the way hits are actually judged). Report *how and where* each preset dies, not just when, and print one traced run, because medians swing a lot on small changes.

Tune the game's numbers against the bot, not your intuition. If a brief's per-level timings conflict with these targets, follow the targets and say so in DESIGN.md.

Determinism (SPEC §3): all randomness from `Rng` seeded by `sys::rand_seed()`, time only from `dt`, no iteration over hash maps. Replays and friend challenges depend on it.

### 3. Art and sound
Two sources, usually mixed; [assets.md](assets.md) says which to use for what:
- **Generated**: `mb art init games/<name>` writes the game's style guide (`art/style.toml`: style words, palette, references), then `mb art sprite|frames|background|layers|tile|ui|icon`, `mb music` (a seamless whole-bar loop, optional base/high layers) and `mb sfx`. mb post-processes to the platform's rules (transparent sprites, exact sizes, pixel grids, palettes, seamless tiles, PNG budgets; mono Ogg at the right loudness), writes a provenance sidecar per asset and a preview to look at (`art/preview/`), and shows the cost of each call. `mb regen <asset>` redoes one (same seed, `--new-seed`, `--prompt`, or `--reprocess` for free). Needs the user's key (`mb keys list`, `mb models`); `--provider mock` makes placeholders offline. Reference: `docs/GENERATE.md`.
- **Code-drawn**, with the bundled scripts:
  - `scripts/pixel.py`: an RGBA canvas (rects, ellipses, polygons, lines, outline, dither, blit, flip), a PNG writer and an atlas packer that writes a Rust table of source rects. Stdlib only.
  - `scripts/check_audio.py`: you can't listen, so run this on your assets after every audio change. It reports length, peak, loudness, harshness and loop clicks, and flags silent or clipping files.
  - `scripts/mix_check.py`: renders a bot round's sound events with the real files and checks the mix: music audible over the effects, each sound's level for how often it plays, voices, variation, key. The rules and how to get the event log are in [assets.md §5](assets.md#5-mix-the-sound).
  - `scripts/sfx.py`: 8-bit effects (square/triangle/saw/noise with sweeps, vibrato, arpeggios, stutters) and a step sequencer for chiptune loops, encoded to mono Ogg Vorbis with `oggenc`. `save_ogg` normalizes loudness and limits peaks, then decodes what it wrote and re-encodes it quieter if Vorbis overshot, so its files pass check_audio's clip check; very peaky sounds (an explosion with a long tail) come out quieter than the loudness target as a result. `python3 sfx.py --self-test` checks the encoder setup.
  - `scripts/make_glb.py`: an example `.glb` generator for 3D games (see the 3D section).

Put your own generator script in `games/<name>/tools/` and commit it (the art stays remixable). Draw anything that changes color with mood or state (skin, a liquid) in light greys and tint it at draw time: `rgba_tint` multiplies, so one sprite set gives smooth pink → red → purple transitions. Import the helpers with `sys.path.insert(0, "<this skill's directory>/scripts")` (in the Maimbrain repo: `.claude/skills/maimbrain-game/scripts`; from the plugin: the installed skill's directory). Better: copy the helper you need into `games/<name>/tools/` so the game's tools are self-contained. Make things big: the main character ≥ 1/3 of the screen width, anything the player must track ≥ ~40 units, and an art pixel ≥ 2–3 units. Small sprites that look fine in a desktop screenshot are unreadable on a phone. Store pixel art at `scale=4` (crisp under the platform's linear filtering) and draw each art pixel as a whole number of logical units. Make the icon (`icon.png`, 256×256) a close-up of your character or hook, not text.

### 4. Build, preview, look at it
- `mb build <game dir>` compiles, packs and validates. Fix every error and warning. (In the Maimbrain repo, `mb` is `mise exec -- cargo run -q -p mb-cli --`.)
- `mb serve --watch <game dir>` runs it in a desktop browser at http://127.0.0.1:8765/runtime/index.html?overlays and rebuilds every time you save: new code is swapped into the open page within about half a second (a fast build without LTO or wasm-opt; `mb build` is still the real one), changed assets are hot-swapped, and compiler errors appear over the game. It runs in the foreground, so start it in the background once and open the page once; don't reload it by hand. Use `--port` if 8765 is taken. Mouse = touch, mouse position = tilt, console = log, haptics and leaderboard calls. An installed `mb` has the web runtime built in; in the Maimbrain repo, run `just runtime` once first (it builds `runtime/dist`, which `cargo run -p mb-cli -- serve` then embeds automatically). If `mb serve` says it was built without the web runtime, that's what's missing. (Plain `mb serve`, without `--watch`, rebuilds when you reload the page instead.)
- **The iteration loop** with `--watch`: edit, then `curl -s http://127.0.0.1:8765/__mb/wait` (it returns once your change is built and running in the page, as JSON). `"state": "running"` means look at the page; `"failed"` carries the compiler's `errors` (file, line, message), `"error"` a page's runtime error (a trap, a panic message in `pages[].message`). The server's own output has the same as `[mb] build N ok|failed …` and `[mb] page N running|error|log …` lines. Then, in the page's console, `await mb.restore()` replays the session you had before the change into the new build up to the same frame (held if it was held) and resolves to `{ frames, firstMismatch, … }`: `firstMismatch: null` means the run went exactly as before; a number is the first frame that came out differently, expected when you changed what happens or how it's drawn. So set up a moment once (a held session, scripted taps, `mb.step`), and after each edit `wait` → `mb.restore()` → screenshot shows the same moment with your change. Add `&restore` to the URL to restore automatically after every reload. SPEC §9 has the details.
- Preview aids (SPEC §9): `?overlays` shades the card overlays and the pause pill. `?speed=0.25` slows time. In the console, `mb.suspend()`, `await mb.step(n)` and `mb.resume()` freeze a moment for a screenshot (wait ~100 ms after stepping before the screenshot). A hidden tab or headless browser runs no animation frames, so there start a held session (`await mb.restart('{"hold": true}')`) and drive it with `mb.tap(x, y)` and `await mb.step(n)`. Add `DEBUG_*` constants (start level, force a shape, fake a sensor reading) to reach late states quickly, and reset them before you finish.
- **Check determinism** before you call it done: play a round (or script one in a held session), then `await mb.verify()` in the console. It replays the run frame by frame and compares draw hashes: `{ match: true }` is the goal; otherwise `firstMismatch` is the first frame that differs, and the cause is something unrecorded (randomness not from the session seed, `dt`-independent timing, hash-map iteration order, game state changed in `render`). SPEC §9 shows the manual version with `mb.restart({ hold: true })`, `mb.step` and `mb.snapshot()`.
- **Look at screenshots** of the title, the first seconds, mid-round, danger, failure and game-over. Check them against the [engagement checklist](engagement.md#checklist). Then fix what you see and look again.

### 5. Publish (when the user asks)
Publishing has two steps, so the user plays the game on their own phone before anyone else sees it:

1. **`mb publish <game dir>`** builds the game, uploads it to maimbrain.com and waits for the server's validation (the same validator as `mb build`). It lands as a **private draft**: only the user can see it. Tell them to open Maimbrain on their iPhone, where it's first in their feed tagged *DRAFT · ONLY YOU* and under Account → My games. If they find something to fix, fix it and `mb publish` again; the new upload replaces the draft, and it can keep the same `version`.
2. **`mb submit <game dir>`** sends the draft to review, once the user says it's ready. They can also tap *Submit for review* in the app or at `maimbrain.com/create`. Don't submit on your own initiative unless the user has asked for that. A moderator plays it; once approved it's in everyone's feed. If an older version is live, it stays live until then.

`mb publish --submit` does both at once, for when the user wants to skip testing on the phone. `maimbrain.com/create` shows each upload's status and any reviewer notes. Ids must be in your namespace (`mb whoami` shows it: `com.maimbrain.<username>`, with `_` as `-`) and `creator` must be `@<username>` (`mb new` sets both). Each submitted version needs a higher `version` than the last one submitted; drafts don't count.

### 6. Hand off honestly
Write down what you couldn't verify without a phone (feel of tilt, haptics, audio balance, safe areas), with the exact numbers to tune (gain, dead zone, timings) collected as named constants at the top of one file.

## Platform facts that bite

- **Feed cards are untouchable.** In browse, the title or game-over screen *is* the card, running live but receiving no touches and no sensor readings. A tap enters play and is also delivered to your game, so a tap anywhere on the title must start the round. Never start a round from a sensor reading, a timer or a swipe. Because a card can't respond, it must never show playing instructions: no "BLOW", "TILT", "SHAKE" or "TAP", and no phone icons or arrows. Cards sell, play teaches (engagement.md §1).
- **Card overlays.** On a card, the platform draws the title and creator over the bottom ~70 pt and like/comment/report buttons down the right ~56 pt, plus a pause pill top-left in play (inside the top inset). Keep buttons and key text out of these areas: use `sys::screen()` insets, then stay inside the middle.
- **Report rounds** with `sys::round`: `Idle` on the title, `Playing` the moment a round starts, `Over` when it ends. `Over` sends the player back to browse after ~1.2 s, with your game-over screen as the card, so submit the score *before* reporting `Over` and make the game-over screen great.
- **Sound is off for many players** (ringer switch). Everything must read without audio. Sound and haptics confirm what the eye already saw.
- **Haptics** only play in play mode and are capped at ~30/s; never use them as the only signal.
- **Assets load asynchronously**, including `startup_assets`. The first frame must draw something good even with nothing loaded (shapes, gradients, text), then swap in sprites when `Asset::state()` is ready.
- **The screen** is your `logical_size` letterboxed to the phone; 360×640 portrait is the norm. Thumb-reachable area is the lower-middle; the top is for score.
- **Motion controls**: read `sensors::tilt()` and use `Tilt::roll()` for tipping left/right (it works held upright, slumped or flat).
  - **Calibrate**: take a reference when play starts, at each new level, and after a pause, but only while the phone is still (motion near zero). A reference taken mid-shake is garbage. Let the reference follow the player when they tip back past it, so a player still tipped from the last level isn't stuck.
  - **Amplify**: physical actions often need more rotation than a comfortable ~45° of phone. Multiply (gains of 2–3 are typical) and make the on-screen object show the amplified angle, so what the player sees is what they're doing.
  - **Smooth** over ~50–100 ms, with a dead zone of ~1–2°.
- **Mic loudness** (`sensors::loudness()`):
  - It's `None` until the player allows the mic, and if they refuse. Show a clear "ALLOW THE MIC TO PLAY" state, never a dead game.
  - Rooms differ, so learn the noise floor continuously: track the minimum recent reading, let it creep up slowly (~0.03/s) and drop instantly to any lower reading. Treat input relative to that floor. Don't wait for a quiet first second; players blow on the first tap.
  - The game's own audio comes out of a speaker a few centimetres from the mic: measured, music alone read ~0.45–0.55. So:
    - lower the music while the mic is listening
    - require a blow to stay above the threshold for ~0.1 s before it counts (effects are short bursts; blows are sustained)
    - never drive a sound's volume from the mic level (a wind sound that grows as you blow feeds back into the mic as more wind)
  - Measured on an iPhone: quiet room plus game music ~0.40–0.55, hardest blow ~0.86. The range is narrow, so treat ~0.85 as full strength (a "too hard" line above that can't be reached).
  - Always show a live level (a meter, a flame bending) so players learn the scale.
  - The platform's mic badge sits beside the pause pill in play: keep the top-left ~110×50 pt clear.
- **Motion** (`sensors::motion()`):
  - One reading per frame. Measure a shake by accumulating energy, e.g. `energy += max(0, magnitude - 0.3) * dt`, with decay. Don't react to single frames.
  - A vigorous shake is ~1.5–3 g.
  - Show the build-up continuously (a gauge, a wobble, a bulge) and scale haptics with it.
  - Shakes blur the screen, so make the key state readable at a glance.
- **Failure order**: when the player fails, play the freeze and fail animation while still `Playing`, and report `Over` (after submitting the score) when the game-over screen appears. Reporting `Over` at the moment of failure would send the player back to browse ~1.2 s later, mid-animation.
- **After a round ends**: ignore taps for ~0.35 s after reporting `Over`, so a frantic tap during the failure doesn't instantly restart, then let any tap retry.
- The helpers every game needs (outlined text, easing, springs, a particle pool, screen shake, buttons, a title and a game-over card) are in the SDK's kit: see [UI and juice](#ui-and-juice). Don't hand-roll them.
- **Text and fonts**: use the game's own fonts (identity.md). `mb font add <game> <library id>` bakes one of 96 library fonts (OFL/Apache) into `assets/fonts/<id>.mbf`; draw with `Font::asset("assets/fonts/<id>.mbf", fallback)` anywhere a font goes (`gfx2d::text`, `ui::text`, a `Theme`, 3D text). Needs `stdlib = { mb2d = 2 }`. Fonts are subset to `--chars` (ASCII by default), so text outside it draws as `?`; bake `--chars latin1` for player names. Distance-field fonts are crisp at any size and take the host's outlines and glows. Bitmap (pixel) fonts are crisp at multiples of `font.pixel_em()` (`theme.fit(font, size)` rounds for you). Use ≥ 16 units for anything the player must read. The built-in Inter (0/1) and 5×7 pixel font (2) are fallbacks; the pixel font is ASCII plus `× ÷ · ° • … ← ↑ → ↓ ♥ ★` (SPEC §5.3).

## UI and juice

Hand-rolled title screens, score labels and game-over text are where games look cheapest. Use the SDK's kit instead (`docs/UI.md` has the full tour; `games/gallery` shows every piece, `mb serve games/gallery`):
- `maimbrain::ui`: a `Theme` composed from the game's identity (`Theme::from_identity("…")`, or one of 17 `Theme::preset`s that look nothing alike: shape language, borders, textures, title/results/HUD layouts, motion personality, letter case, library fonts; the old `candy`/`night`/`arcade`/`paper`/`jungle` are the house look), anti-aliased shapes (`ui::shape`), styled text (`ui::text(..)` with outline, soft shadow, glow, wrap, alignment, measuring), 33 vector `Icon`s, `Button`s (pressed state, 44 pt touch area, haptic hook), pills, progress bars, ring meters, ribbons, `Layout` (safe area, the card area clear of the feed overlays, the pause-pill zone), and ready-made `TitleCard`, `ResultsCard`, `Hud` and `Countdown`.
- `maimbrain::motion`: `Ease` (the full easings.net set), `Tween`, `Spring` (critical / bouncy / wobbly), `Seq`, `stagger`, `Shake`, `Punch`, `Counter` (rolling numbers), `HitStop`, `Pulse`.
- `maimbrain::juice`: `Particles` (confetti, cannons, sparkles, stars, puffs, sparks, rings, `celebrate`), `Popups` ("+100"), `Combo`, `Flash`, `VignettePulse`.

The templates already use it: copy their shape, not their look. Rules: update every kit object in `update(dt)` and draw it in `render` (it never reads a clock, so it replays); seed `Particles` from your game's `Rng`; restyle by changing the `Theme` and its `style`, not by forking widgets; keep cards' buttons inside `layout.card`. When no layout fits the game's world, draw your own card (a receipt, a rosette, a chalkboard) with the kit's text and shapes: that is often the most memorable thing in a game.

**Title screen in 10 lines**
```rust
// init
let layout = Layout::new();
let theme = Theme::from_identity(IDENTITY).load_fonts();   // identity.md
let title = TitleCard::new("BUBBLE", theme).tagline("don't let it pop").best(best as i64);
// update
self.title.update(dt);
// render, over your live scene (the card sells; any tap starts the round)
self.title.draw(&self.layout);
```

**Results card** (on game over, after `store::submit_score`, before `sys::round(Round::Over)`):
```rust
self.results = Some(ResultsCard::new(score, prev_best, self.theme, &self.layout, self.rng.next_u32() as u64)
    .heading("SPLAT!").label("POINTS"));          // .floating() for no panel; .extra(button) for SCORES
// update: feed it events first; a tap it doesn't claim retries (after your guard)
if let Some(card) = &mut self.results {
    let retry = card.handle(&e) == Some(ResultsAction::Retry);   // per event
    if card.update(dt) == Some(ResultsEvent::NewBest) { /* fanfare, haptic */ }
}
// render
if let Some(card) = &self.results { card.draw() }
```
It rolls the score up, then shows BEST (and "3 MORE TO BEAT IT" when close) or celebrates a new best, in the theme's results layout (a panel with a ribbon, a receipt, a scoreboard, a stamp, an editorial page). Write the heading in the game's own voice; "SPLAT!" is the example everyone copies.

**Combo meter**
```rust
self.combo = Combo::new(1.5);                                     // init: 1.5 s window
let n = self.combo.hit();                                         // each success
self.popups.score(x, y, 10 * self.combo.multiplier() as i64, theme.gold);
self.combo.break_combo();                                         // a miss
if let Some(lost) = self.combo.update(dt) { /* timed out after `lost` hits */ }
self.combo.draw(300.0, self.layout.hud.y + 90.0, &theme);         // render
```

**Juice on every hit**: `shake.add(0.3)` (apply in render inside `gfx2d::push`/`pop`), `punch.kick(0.25)` on the score, `hitstop.freeze(0.05)` (run the sim with `hitstop.step(dt)`), `fx.burst(..)` or `fx.sparkles(..)` at the point, `popups.score(..)`, and the sound pitched up with the combo. Celebrate milestones with `fx.celebrate(x, y, &theme)`. `Hud::set_score` rolls and punches the score for you and marks a beaten best.

Pause-safe HUD: the platform draws the pause pill and pauses the game by not calling `update`, so a HUD built from the kit freezes with it. Don't draw a pause button; if the game needs a run-up after a pause, start a `Countdown` in `Game::resume` and skip the simulation while it's `active()`.

## Rhythm games

Anything judged against music (rhythm games, beat-synced dodgers, a jingle the player taps along to) follows the recipe in SPEC §5.6 "Rhythm games". In short: the beat grid is game time; music is scheduled on it with `Sound::play_looped_at(…, origin)`; presses are judged with `Event::time`; the grid is re-anchored when `sys::time_lost() > 0` and after a resume; a small self-calibration absorbs the rest. `games/summoned` (`src/rhythm.rs`, `src/sound.rs`, the `time_lost`/`resumed` handling in `src/sim.rs`) is a complete worked example. What it learned:
- **Never anchor the grid to "the frame that started the music".** `play` starts whenever the sound finishes decoding and the output is ready. Put beat 0 ~0.12 s in the future (`origin = sys::time() + 0.12`) and schedule every stem with that same `at`; if a stem's asset arrives late, call `play_looped_at` with the *original* `at` when it does, and it joins on the grid.
- **Make each music loop exactly a whole number of phrases** (sfx.py's `Song.stems` + `save_stems` do), so the loop's wrap lands on the grid and later phrases need no scheduling at all. Run `check_audio.py` for end padding and clicks at the seam.
- **Judge with the event's own time**, not `sys::time()`: the host gives touches their own game time within the frame (SPEC §5.2), so judging with the frame's time throws away ±½ frame of accuracy and lets frame pacing leak into judgements. Apply a frame's presses before you auto-miss notes in that update, and clamp each press to `[time before this update, sys::time()]`. If your sim keeps its own clock (from 0 each round), convert: `Event::time` and `play_at`'s `at` are host game time (`sys::time()`), so shift by `sys::time() − sim.t` both ways. Summoned 0.1.0 didn't, and every press silently clamped to its frame's end.
- **Re-anchor, don't patch.** When `sys::time_lost() > 0` (a hitch the `dt` clamp cut short) the music is ahead of the grid: at the next phrase boundary, schedule the music again at a fresh `origin` and move the grid there. After `resume`, restart the current phrase the same way so a returning player gets a run-up. Never guess from `dt` (e.g. "dt ≥ 0.095 means a clamp"): `time_lost` is exact and recorded.
- **Calibrate a little, not a lot.** The host already compensates audio output latency (Bluetooth included) and when frames are shown. Learn the rest from the median of the first ~3–8 judged presses (ignore offsets beyond ±0.2 s), clamp it to about −60…+100 ms, save it as next run's prior, and keep the first encounter's windows wide (×1.4) until it settles.
- **Model the bot's presses with their own times** (planned time, clamped into the frame), with a lateness bias and noise per preset, and test that a steady late player gets calibrated into Perfects.
- **Everything must read without sound.** Many players have the ringer off: markers, rings closing on the beat and on-beat pulses carry the timing; the music confirms it.
- **Audio isn't in the draw hash**, so `mb.verify()` can't see a desync. In the preview, `mb.audio()` lists each voice's `at` and the AudioContext time it's heard from: stems must share one `when`, and a late voice must show an `offset` matching its lateness.

## 3D (mb3d)

Start from **[template-3d/](template-3d/)** (`mb new --3d games/<name>`): a small complete 3D game (catch the drone) that builds, validates and passes its tests as is. Read all three files before writing your own:
- `src/scene.rs` builds the scene once: the procedural sky (also the image-based lighting), a shadow-casting sun, post (bloom, vignette), meshes made in code (a lathed platform, pillars merged into one mesh, a faceted drone with a glowing ring as a child node), one particle emitter, one trail and two point lights (one on the drone, one flash that decays).
- `src/sim.rs` holds the rules *and* the camera, with tests: the tap test projects the drone with `Camera::project`, the same formula the host uses, so hit-testing is plain tested Rust.
- `src/lib.rs` is the host layer: rounds, input, moving nodes, and the HUD drawn with mb2d at `gfx3d::project` positions (a reticle on the drone, a hint under it).

The API is SPEC §5.4; the SDK binding is `maimbrain::gfx3d`.

- **Manifest**: `stdlib = { mb3d = 2, mb2d = 2 }` (mb2d for the HUD; 2 for game fonts, which 3D text can use too) and `perf_tier = "full"`. mb3d 2 (the template's default) adds animated characters, instancing, freeing, fog, toon shading and outlines, 3D text and depth of field (below); a game declaring `mb3d = 1` gets v1 unchanged.
- **Shape of a 3D game**: build meshes, materials, emitters, trails and nodes in `init` (or when assets arrive), keep their handles in your state, update node transforms in `update`, and in `render` call `gfx3d::camera(..)`, `gfx3d::render()`, then any mb2d HUD. The scene persists; don't recreate it every frame.
- **Build once, pool; free what you're done with.** Meshes, textures, materials, models, emitters and trails count against the limits (512 meshes, 1024 materials, 64 textures, 64 models, 256 emitters, 64 trails; SPEC §5.4). Build them once and pool: one emitter per particle look fired wherever it's needed, one material per look (`Material::set` to pulse it), a fixed pool of enemy nodes hidden with `set_visible(false)` and reused, trails moved between nodes with `detach()` + `attach(node)` (that starts a fresh ribbon; the old one fades out where it was). `Node::destroy` frees a node and its children; with mb3d 2, `free()` on a mesh, material, texture, emitter, trail or model releases it (a level's props when the next level loads; don't create and free things every few frames).
- **Point lights**: 8 slots. A light keeps its slot from its first `point_light` call until its node is destroyed; intensity 0 darkens it but keeps the slot, and hiding the node doesn't switch it off. Keep one or two flash lights and move them.
- **Units and colors**: meters, +y up, the camera looks down −z (`Camera::looking_at`, `look_at`, `OrbitCamera`, `FollowCamera`). Colors are linear: write `srgb(0xe63946)` / `srgba(0xe63946ff)` for picked colors. Values above 1 glow: an emissive of `[0.4, 1.6, 4.0]` × `emissive_strength` 3 blooms; use `Material::UNLIT` for pure light sources (engine glows, neon).

### Performance budgets (60 fps on an iPhone 13 class phone)
| Thing | Budget | Notes |
|---|---|---|
| Drawn meshes | ≤ 200 per frame | Each node with a mesh is a draw. Merge static scenery into one mesh (`MeshData::merge`, `transformed`). |
| Triangles | ≤ 150 k on screen | A 48×32 UV sphere is 3 k; a faceted ship ~1 k. |
| Live particles | a few thousand | 16 k is the hard cap. Big soft particles cost fill rate; prefer more small sparks over many huge smoke puffs. |
| Point lights | ≤ 8, keep 3–4 | Every lit pixel loops over them. |
| Sun shadows | one 2048² map | Keep `shadow_distance` tight around the action (15–30 m) for sharp shadows. |
| Sky | set once | `gfx3d::sky` re-bakes the lighting over the next ~6 frames. Never change it per frame. |
| Textures | ≤ 1024² mostly | 64 textures and 96 MB in all; a 2048² texture takes 22 MB. |
| glTF loading | one model per frame, in `update` | Parsing happens inside your callback (watchdogs, SPEC §3). |
| Instances (mb3d 2) | thousands; count instances × triangles in the triangle budget | One instanced node is one draw. Re-send a list only when it changes (coins that spin: fine; a static forest: once). Split big fields into chunk nodes so culling works. |
| Skinned characters | a handful | ≤ 64 joints per skin, 128 skinned draws per frame max. |
| Outlines | characters, props, big shapes | An outlined draw draws twice. |
| Depth of field | on when it matters | ~0.3–0.6 ms GPU (estimate); `blur: 0` turns it off. |
| 3D text | a few dozen labels | Each text node is a draw; recoloring is cheap, changing the words relays it out. |

Check these with `mb.stats()` in the preview: draws, triangles, instances, skinned draws, outlines, text, shadow casters, particles, and CPU ms per callback.

Hide nodes you'll reuse (`set_visible(false)`) instead of destroying and recreating them every few frames. Changing a material (`Material::set`) is cheap: pulse emissives freely.

### Meshes in code
The SDK's `MeshData` builders cover most game shapes: `cube`, `cuboid`, `plane`, `uv_sphere`, `icosphere`, `cylinder`, `cone`, `frustum`, `torus`, `lathe` (revolve a profile: bottles, rockets, fuselages), `extrude` (a 2D polygon along z: wings, fins, logos, level pieces). Then:
- `.flat_shaded()` for the faceted low-poly look (every triangle gets its face normal);
- `.with_vertex_colors(0xRRGGBBAA)` or `.with_vertex_colors_by(|pos, normal| ..)` to color without textures (one material, many colors);
- `.transformed(&Transform::at(..).with_rot(..).matrix())` and `.merge(&other)` to assemble parts (mirroring flips winding back for you);
- `.upload()` → a `Mesh`.

```rust
// A faceted rock: a jittered icosphere, flat shaded, tinted by height.
let mut m = MeshData::icosphere(1.0, 1);
for (i, p) in m.positions.iter_mut().enumerate() {
    *p = *p * (0.8 + 0.4 * (i as f32 * 12.9898).sin().abs());
}
let rock = m.flat_shaded().with_vertex_colors_by(|p, _| if p.y > 0.3 { 0xb8b0a8ff } else { 0x6b625cff }).upload().unwrap();
```
Keep parts that need different materials as separate meshes on child nodes of one parent (the template's drone: a body node and a ring node under one parent you move). Front faces wind counter-clockwise; `cargo test` the builder code like any other logic (the SDK compiles on the host, and mb3d constructors return handle 1 there).

### Making a .glb with a script
For textured or hand-shaped models, generate a glTF binary from a script in `tools/` so the game stays remixable. Copy **[scripts/make_glb.py](scripts/make_glb.py)** (stdlib Python only) into your game's `tools/` and change it; `python3 tools/make_glb.py assets/model.glb` writes a textured cube "relic" as it stands. It works like this:
1. Build the geometry as lists (positions, normals, UVs, u16 indices).
2. Paint textures procedurally from one height field: base color, a normal map from its slopes (OpenGL/glTF convention: green = up), a metal-rough map (roughness in G, metallic in B), an emissive map. Write PNGs with `zlib`; convert the base color to JPEG with `sips` on macOS (smaller).
3. Write the `.glb`: a JSON chunk (asset, scene, nodes, meshes, materials, textures, images, buffers, bufferViews, accessors) and a BIN chunk, each 4-byte aligned.
4. Optionally compress: `npx gltfpack -i raw.glb -o assets/model.glb -c` quantizes and meshopt-compresses the mesh (supported), keeping PNG/JPEG textures. Don't use `-tc` (KTX2 isn't supported yet).

Load it with `Model::load(asset)` once `asset.state()` is `Ready`, then `model.spawn(parent)` returns a root node to move. Use `KHR_materials_emissive_strength` for glowing parts; unsupported required extensions (Draco, KTX2, WebP) fail to load.

### Animated characters (mb3d 2)
A rigged glTF plays its clips through `gfx3d::Animator`. Bones and sockets are nodes you find by name, so holding a sword (or hanging a lantern) is parenting. **You own animation time**: the animator moves only when you `update(dt)` it, and the host samples clips at exactly the times you pass, so replays pose identically. Make the model with **[scripts/make_rigged_glb.py](scripts/make_rigged_glb.py)** (stdlib Python: a skeleton, a skinned mesh with per-vertex weights, a morph target and four clips; `games/mistwood` uses it as is), or any glTF with skins (≤ 64 joints) and animations.

```rust
// When the asset is ready (in update):
let model = Model::load(asset).unwrap();
let fox = model.spawn(None).unwrap();                          // the root: move and turn this
let (idle, run, jump) = (model.clip("Idle").unwrap(), model.clip("Run").unwrap(), model.clip("Jump").unwrap());
lantern.set_parent(fox.find("Socket.Tail"));                     // attach to a bone or socket
fox.find("Fox").and_then(|n| n.material()).map(|m| m.set_style(&MaterialStyle { toon_bands: 3, outline_width: 1.8, rim: 0.5, ..Default::default() }));
let mut anim = Animator::new();
anim.play(idle, Play::Loop, 0.0);
// Every frame:
if jumped { anim.restart(jump, Play::Once, 0.06) } else if anim.finished() || running { anim.play(run, Play::Loop, 0.15) }
anim.set_speed(ground_speed / 8.0);                             // feet match the ground
anim.update(dt);
anim.apply(fox);
let hand = fox.find("Hand.R").and_then(|n| n.world());          // where a bone is, for hit tests
```
`play_synced(clip, fade)` crossfades walk ↔ run at the same point of the stride. If you also simulate the motion (a jump arc you collide with), drive both from one curve, as Mistwood's `sim::jump_lift` mirrors the Jump clip. A model's root faces wherever the file's +z points: turn the root node, not the bones.

### Lots of copies: instancing (mb3d 2)
Bullets, coins, grass, trees, crowds: one node, one mesh, one material, many `Instance`s (position, rotation, scale, a tint that also scales emissive), one draw call.
```rust
let coins = Node::with_mesh(coin_mesh, gold).unwrap();
let list: Vec<Instance> = coins_alive.iter().map(|c| Instance::at(c.pos).with_rot(Quat::from_rotation_y(t * 3.0)).with_color(0xffd27aff)).collect();
coins.set_instances(&list);   // replaces the list; send it again only when it changes
```
The node is culled as one sphere around all its instances: for a forest, use a node per chunk of ground (Mistwood moves 7 chunk nodes forward as the run goes on and re-lays out a chunk's instances only when it leapfrogs). Instances of a transparent material aren't sorted among themselves.

### Looks: fog, toon, outlines, 3D text, depth of field (mb3d 2)
```rust
gfx3d::fog(&Fog { mode: FogMode::Exponential, color: [0.05, 0.075, 0.12], density: 0.045, height_falloff: 0.22, sky: 0.9, ..Default::default() });
material.set_style(&MaterialStyle { toon_bands: 3, outline_width: 1.5, outline_color: [0.01, 0.015, 0.03], rim: 0.3, ..Default::default() });
sign_label.set_text(&TextDesc::new(Font::SansBold, 0.3).with_outline([0.0, 0.0, 0.0, 1.0], 0.07), "100 m");     // in the node's xy plane, facing +z
pop.set_text(&TextDesc { flags: TextDesc::BILLBOARD | TextDesc::SCREEN, ..TextDesc::new(Font::SansBold, 26.0) }.with_color([2.4, 1.8, 0.6, fade]), "+10");
gfx3d::depth_of_field(&DepthOfField { focus: camera_to_player, range: 1.3, blur: 5.0, near: 0.4 });
```
- **Fog** sells depth and hides where the world ends: match its color to the sky near the horizon and set `sky` so the horizon melts. `height_falloff` > 0 makes ground mist. Glows fade out in it; give the player `MaterialStyle::NO_FOG` if it must always read.
- **Toon + outline** is the fastest way to a coherent stylized look for code-built meshes: 2–4 bands, outlines 1–2.5 px in a dark color from the palette (not pure black). Backlight the scene (sun toward the camera) and the rim and outlines carry the shapes.
- **3D text**: world-space for signs and labels in the scene, billboards for floating scores (`SCREEN` keeps them a readable size at any distance). The host fonts only; colors above 1 glow.
- **Depth of field**: focus on the player every frame (distance from the camera); keep `blur` ≤ 6 px on a phone. It's for mood and for separating the player from busy backgrounds, not for hiding things.

### Lighting recipes
- **Space / night**: a near-black sky background with saturated nebula colors (`[0.05, 0.19, 0.42]`, `[0.55, 0.07, 0.32]`, density ~1) gives colorful reflections; a warm sun (intensity ~3, shadows on) as the key; two colored point lights on opposite sides as rims; bloom 0.6–0.8 at threshold 1; vignette 0.3.
- **Bright and friendly**: raise all three sky colors (the sky is the ambient light: `[0.25, 0.35, 0.55]` etc., nebula density low), sun intensity 3–5 from high up, exposure ~1, bloom low (0.2) so whites don't glow, saturation 1.1.
- **Neon**: dark scene, unlit or emissive materials with strength 2–6 for the signs, point lights in the same colors near them so they light their surroundings, bloom 0.8+, chromatic aberration 0.3.
- **Readable metals**: metals show their environment; in a dark sky they look black. Give metals roughness 0.3–0.5 so the sun's highlight spreads, or keep the brightest nebula behind the camera.
- **Flashes**: an explosion is particles plus `shockwave` plus a point light whose intensity decays (`90 × exp(−7t)`) — the flash lighting nearby surfaces sells it more than the particles do. Camera shake is yours: offset the camera with your seeded `Rng`.

### Effects
`Emitter` (particle look + physics: color and size over life, lifetime, speed, cone, gravity, drag, additive or alpha, optional texture, velocity stretch for sparks) and `Trail` (ribbon that follows a node). The host simulates them deterministically, so replays match: never feed them anything but game state. A good explosion is four emitters fired together, each a different look:

```rust
// Built once in init, fired anywhere. Linear colors above 1 bloom.
let fire = Emitter::new(&EmitterDesc { colors: [[4.5, 2.6, 1.0, 1.0], [2.6, 0.8, 0.2, 0.75], [0.7, 0.14, 0.05, 0.35], [0.0; 4]],
    sizes: [0.5, 1.3, 1.7, 1.9], lifetime: [0.35, 0.7], speed: [0.4, 2.6], drag: 2.5, gravity: vec3(0.0, 0.6, 0.0), ..Default::default() }).unwrap();
let sparks = Emitter::new(&EmitterDesc { colors: [[9.0, 6.0, 2.5, 1.0], [6.0, 2.2, 0.5, 1.0], [2.5, 0.5, 0.1, 0.8], [0.4, 0.05, 0.0, 0.0]],
    sizes: [0.07, 0.06, 0.045, 0.02], lifetime: [0.5, 1.2], speed: [4.0, 11.0], drag: 1.6, gravity: vec3(0.0, -3.5, 0.0), stretch: 0.045, ..Default::default() }).unwrap();
let smoke = Emitter::new(&EmitterDesc { colors: [[0.2, 0.12, 0.08, 0.0], [0.13, 0.09, 0.09, 0.5], [0.07, 0.06, 0.08, 0.3], [0.04, 0.04, 0.05, 0.0]],
    sizes: [0.6, 1.3, 1.9, 2.4], lifetime: [1.4, 2.4], speed: [0.3, 1.4], drag: 1.2, blend: FxBlend::Alpha, gravity: vec3(0.0, 0.35, 0.0), ..Default::default() }).unwrap();
// The boom: a burst of each, a shockwave, and a flash light that decays (90 × exp(−7t)).
fire.emit(at, Vec3::ZERO, 36);
sparks.emit(at, Vec3::ZERO, 170);
smoke.emit(at, Vec3::ZERO, 26);
gfx3d::shockwave(at, 4.5, 0.012, 0.9);
```

**Things that move.** Particles live in world space and keep only the velocity their burst gave them. If what exploded was moving (an enemy flying alongside an on-rails camera), plain `emit` leaves the cloud behind and it streams away; fire it with `emit_moving(at, dir, count, velocity)` instead, passing the object's velocity, and the whole cloud rides along while it spreads (the template's catch does this with the drone's velocity). The other way is a floating frame: keep the player and camera near the origin and move the world past them, so things flying with the camera are nearly still. Pick one frame for the whole game. Trails follow their node's world position each frame, so a trail on something the camera flies with streaks only if the node really moves in the world.

**Selling speed** (racers, chases, dashes): `gfx3d::speed_blur(&SpeedBlur { radial, center, motion, stretch, .. })` (mb3d 2) adds a radial blur toward `center` (put it on the projected player), a directional motion blur along `motion` (how far a far point moved on screen since the last frame) and stretching at the screen edges, all cheap. Scale them with speed, and pair them with a field of view that widens with speed, a low camera, a constant small road shake and streak particles flying past the lens.

### HUD over 3D
`gfx3d::project(world)` gives the logical screen position (mb2d's coordinates), depth and whether it's on screen: put labels, reticles and off-screen arrows there. Everything mb2d draws in the frame lands on top of the 3D image. Call it after `gfx3d::camera` in `render` (it uses the latest camera). For game logic (lock-on, tap hit tests), use `camera.project(world, W, H)` on your own `Camera` value instead: the same formula (SPEC §5.4) in plain Rust, so it runs in `cargo test` (where `gfx3d::project` returns zeros) and agrees with the host to within float rounding.

### Previewing
`mb serve` works the same. The first few frames show the sky without its nebula while the lighting bakes; screenshots after `await mb.step(n)` with n ≥ 10 are representative. Check that effects replay with `await mb.verify()` (see Build, preview, look at it): explosions, particles and trails are part of the draw hash, so `match: true` means they land exactly where they did live. `await mb.screenshot("name")` saves the frame to `screenshots/name.png` in your game directory; `mb.stats()` shows what the frame drew against the budgets above.

## Physics

The SDK has deterministic rigid-body physics (Rapier inside your wasm): `maimbrain::physics2d` in logical pixels for mb2d games, `maimbrain::physics3d` in meters for mb3d games. Turn it on in `Cargo.toml`: `maimbrain = { …, features = ["physics2d"] }` (or `"physics3d"`). `docs/PHYSICS.md` is the full API; `games/topple` (2D crane stacker) and `games/knockdown` (3D can toss) are worked examples.

**When to use it**: anything whose fun is how solid things react — stacking and toppling, knocking things over, ragdolls, marbles and pinball, cars and contraptions, launchers, chains and ropes, destruction. **When not to**: games whose motion you want to author exactly (a rhythm game, a platformer with tight jumps, a slingshot that must feel the same every time): hand-written kinematics are smaller, cheaper and easier to tune. Physics adds ~0.45–0.6 MB (2D) or ~0.6–0.8 MB (3D) to `game.wasm`; never enable both.

**The shape of a physics game** (the template's split still holds):
- `sim.rs` owns a `World`, built in `Sim::new` from your seeded `Rng`. `Sim::step(dt)` calls `world.step(dt)` once, then reads `world.events()` for scoring, sounds (push cues) and effects. Input becomes impulses, forces or `move_kinematic` targets in `update`, never in `render`.
- `render` reads only: `world.with_transform(id, || draw centered at 0,0)` or `draw_pose(id)` in 2D, `world.sync(id, node)` in 3D. Prototype with `world.draw_debug()` (2D), then replace it with art that matches the shapes.
- Keep your own `Vec` of game objects holding their `BodyId` and game state (kind, color, "landed"); use `Body::tag` only for a quick lookup. Never iterate a `HashMap` of ids.

**Patterns that work**:
- Sound and haptics from `Event::ContactBegin`: `speed` is the closing speed (px/s or m/s), mass-independent. Ignore contacts under ~60–90 px/s (2D) or ~0.8 m/s (3D), set volume from speed, play at most 2–3 thuds a frame, haptic `Tap` above a threshold and `Heavy` for big hits.
- Goals, pickups, kill zones and "fell in the water": a `.sensor()` shape and `Event::SensorEnter`.
- "Has it settled?": `world.awake_count() == 0`, or a body's speed under a threshold for ~0.3 s.
- Things the player drags or steers: a kinematic body moved with `move_kinematic(id, pos, angle)` every frame; it pushes dynamic bodies properly.
- Held objects: `Joint::rope` from a kinematic body plus `set_fixed_rotation(id, true)` keeps it level while swinging (Topple's crane); release with `unjoin` and `set_fixed_rotation(id, false)`.
- Breakable structures: `Joint::fixed` between pieces, `unjoin` when `joint_torque`/`joint_force` passes a limit you tune with a bot.
- Show where a throw or drop will go (a dotted arc, a landing marker): players can't judge momentum on a phone. Topple's `predicted_arc` simulates the free fall in a few lines.
- Aiming in 3D: a ray through the tapped pixel from your `Camera` (Knockdown's `screen_ray`), `world.raycast` for the target, then solve the launch velocity so the arc passes through it.

**Pitfalls** (all in docs/PHYSICS.md):
- Units: 2D is pixels (100 px = 1 m, gravity 981 px/s², +y down, clockwise angles like `gfx2d::rotate`); keep bodies 10–1000 px. 3D is meters; keep things above ~5 cm and give small fast things `.bullet()`.
- Build stacks awake and a hair apart; `.asleep()` stacks float when the bottom is knocked out.
- `apply_force` lasts one frame: call it every frame you push. Kicks are `apply_impulse` or `add_velocity`.
- Create joints after placing both bodies (anchors are world points now); joined bodies don't collide unless `.collide(true)`.
- Draw `draw_pose`/`sync`, not `position`, or motion judders at 120 Hz.
- Masses that touch should be within ~10:1; tall stacks want `set_solver_iterations(6..8)`.

**Budgets**: ≤ ~150 awake 2D bodies or ≤ ~80 awake 3D bodies keeps physics near 1 ms on an iPhone 13 class phone (an M1 Pro steps 200 bodies in ~0.2 ms 2D, ~0.4 ms 3D). Sleeping bodies are nearly free. Remove what leaves the play area.

**Determinism**: it's built in (fixed 60 Hz steps from the recorded `dt`, `enhanced-determinism`), so the usual check applies unchanged: play a round, `await mb.verify()`, expect `match: true`. If it fails, look for a world changed in `render`, bodies spawned from unordered iteration, or physics fed from anything but game state.

## Before you call it done

- [ ] `mb build` validates with no warnings; `cargo test -p <name>` passes; the bot says the difficulty curve is right.
- [ ] `await mb.verify()` after a played round returns `match: true` (the run replays exactly).
- [ ] Every item in the [engagement checklist](engagement.md#checklist).
- [ ] Screenshots reviewed for the title card, game-over card, mid-round and both edges of danger.
- [ ] The identity check in [identity.md](identity.md#3-check-it-against-the-house-look) answered against docs/IDENTITY.md: the game's own fonts (`mb build` gives no font or theme warning), palette and cards, and no "yes" left that you couldn't justify.
- [ ] Started from a kit? All three "Make it yours" changes are in, and nothing of the kit's title, character or tagline is left.
- [ ] DESIGN.md updated to match what you built, with the "not verified on device" list and tuning constants.
- [ ] Generated assets: every preview looked at, `mb art check` clean (no mock placeholders, within budgets), and the user told what the art and sound cost.
- [ ] Sound: `check_audio.py` and `mix_check.py` (on a bot round) clean, or each flag explained in DESIGN.md.
