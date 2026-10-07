---
name: maimbrain-game
description: Design and build a Maimbrain game (a short Rust→wasm game for the iPhone feed) — from a brief to a validated, playable, genuinely engaging .mbx. Use when creating a new game under games/, reworking an existing one, adding art or sound to one, or tuning how one feels.
---

# Making a Maimbrain game

A Maimbrain game lives in a vertical feed, between other games, in front of someone with a thumb on the glass and about one second of patience. It isn't installed, chosen or explained. Building a *valid* game is the easy part (`docs/SPEC.md` + `mb build`). Building one people stop swiping for is the job.

Read these before writing code:
1. **`docs/SPEC.md`**: the contract. Especially §2 (manifest), §3 (determinism), §5.2 (feed navigation: browse vs play, the card overlay areas), §5.7 (sensors only in play), §9 (build, preview, test).
2. **[engagement.md](engagement.md)**: what makes a game work *in this feed*. Its rules are requirements, not suggestions.
3. **[template/](template/)**: a tiny complete game (tap the bubble) to start from. `mb new <name>` creates a game from it with your id and handle filled in (`mb login` first). Or copy it to `games/<name>/` by hand and replace `NAME` in `Cargo.toml` and `manifest.toml` (and the bubble `icon.png` with your own), and keep its shape: pure rules in `src/sim.rs` with tests, and a thin host layer in `src/lib.rs` that handles tap to start, round reporting, score-before-Over, the retry guard and saving the best score.
4. **`sdk/maimbrain/src/`**: the SDK (it's small; read all of it).

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

### 3. Art and sound with the bundled scripts
- `scripts/pixel.py`: an RGBA canvas (rects, ellipses, polygons, lines, outline, dither, blit, flip), a PNG writer and an atlas packer that writes a Rust table of source rects. Stdlib only.
- `scripts/check_audio.py`: you can't listen, so run this on your assets after every audio change. It reports length, peak, loudness, harshness and loop clicks, and flags silent or clipping files.
- `scripts/sfx.py`: 8-bit effects (square/triangle/saw/noise with sweeps, vibrato, arpeggios, stutters) and a step sequencer for chiptune loops, encoded to mono Ogg Vorbis with `oggenc`.

Put your own generator script in `games/<name>/tools/` and commit it (the art stays remixable). Draw anything that changes color with mood or state (skin, a liquid) in light greys and tint it at draw time: `rgba_tint` multiplies, so one sprite set gives smooth pink → red → purple transitions. Import the helpers with `sys.path.insert(0, "<this skill's directory>/scripts")` (in the Maimbrain repo: `.claude/skills/maimbrain-game/scripts`; from the plugin: the installed skill's directory). Better: copy the helper you need into `games/<name>/tools/` so the game's tools are self-contained. Make things big: the main character ≥ 1/3 of the screen width, anything the player must track ≥ ~40 units, and an art pixel ≥ 2–3 units. Small sprites that look fine in a desktop screenshot are unreadable on a phone. Store pixel art at `scale=4` (crisp under the platform's linear filtering) and draw each art pixel as a whole number of logical units. Make the icon (`icon.png`, 256×256) a close-up of your character or hook, not text.

### 4. Build, preview, look at it
- `mb build <game dir>` compiles, packs and validates. Fix every error and warning. (In the Maimbrain repo, `mb` is `mise exec -- cargo run -q -p mb-cli --`.)
- `mb serve <game dir>` runs it in a desktop browser at http://127.0.0.1:8765/runtime/index.html?overlays. It runs in the foreground, so start it in the background once; after each save, reload the page and it rebuilds. Use `--port` if 8765 is taken. Mouse = touch, mouse position = tilt, console = log, haptics and leaderboard calls.
- Preview aids (SPEC §9): `?overlays` shades the card overlays and the pause pill. `?speed=0.25` slows time. In the console, `mb.suspend()`, `mb.step(n)` and `mb.resume()` freeze a moment for a screenshot. Add `DEBUG_*` constants (start level, force a shape, fake a sensor reading) to reach late states quickly, and reset them before you finish.
- **Look at screenshots** of the title, the first seconds, mid-round, danger, failure and game-over. Check them against the [engagement checklist](engagement.md#checklist). Then fix what you see and look again.

### 5. Publish (when the user asks)
`mb publish <game dir>` builds the game, uploads it to maimbrain.com and waits for the server's validation. It then waits for a moderator's review; `maimbrain.com/create` shows the status and any reviewer notes. Ids must be `com.maimbrain.<username>.<game>` and `creator` must be `@<username>` (`mb new` sets both). Each upload needs a higher `version` than the last.

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
- You'll want small helpers every game needs: text with a drop shadow, easing, a particle pool, screen shake. Keep them in their own module.
- **Text**: the pixel font (id 2) is crisp at multiples of 8. Use ≥ 16 for anything the player must read, 8 only for fine print. Inter (0/1) for sizes that aren't multiples of 8.

## Before you call it done

- [ ] `mb build` validates with no warnings; `cargo test -p <name>` passes; the bot says the difficulty curve is right.
- [ ] Every item in the [engagement checklist](engagement.md#checklist).
- [ ] Screenshots reviewed for the title card, game-over card, mid-round and both edges of danger.
- [ ] DESIGN.md updated to match what you built, with the "not verified on device" list and tuning constants.
