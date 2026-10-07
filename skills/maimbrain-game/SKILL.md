---
name: maimbrain-game
description: Design and build a Maimbrain game (a short Rust→wasm game for the iPhone feed) — from a brief to a validated, playable, genuinely engaging .mbx. Use when creating a new game under games/, reworking an existing one, adding art or sound to one, or tuning how one feels.
---

# Making a Maimbrain game

A Maimbrain game lives in a vertical feed, between other games, in front of someone with a thumb on the glass and about one second of patience. It isn't installed, chosen or explained. Building a *valid* game is the easy part (`docs/SPEC.md` + `mb build`). Building one people stop swiping for is the job.

Read these before writing code:
1. **`docs/SPEC.md`**: the contract. Especially §2 (manifest), §3 (determinism), §5.2 (feed navigation: browse vs play, the card overlay areas), §5.7 (sensors only in play), §9 (build, preview, test).
2. **[engagement.md](engagement.md)**: what makes a game work *in this feed*. Its rules are requirements, not suggestions.
3. **[template/](template/)**: a tiny complete game (tap the bubble) to start from; **[template-3d/](template-3d/)** is the 3D one (catch the drone; see the 3D section). `mb new games/<name>` creates a game from the 2D template, `mb new --3d games/<name>` from the 3D one. Signed in (`mb login`), the id and creator come from your account; signed out it still works, with a placeholder id and creator (or pass `--id dev.you.name --creator @you`), which build and preview fine but must become your namespace before `mb publish`. You can also copy a template to `games/<name>/` by hand and replace `NAME` in `Cargo.toml` and `manifest.toml` (and `icon.png` with your own). Keep its shape: pure rules in `src/sim.rs` with tests, and a thin host layer in `src/lib.rs` that handles tap to start, round reporting, score-before-Over, the retry guard and saving the best score.
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
- `scripts/sfx.py`: 8-bit effects (square/triangle/saw/noise with sweeps, vibrato, arpeggios, stutters) and a step sequencer for chiptune loops, encoded to mono Ogg Vorbis with `oggenc`. `save_ogg` normalizes loudness and limits peaks, then decodes what it wrote and re-encodes it quieter if Vorbis overshot, so its files pass check_audio's clip check; very peaky sounds (an explosion with a long tail) come out quieter than the loudness target as a result. `python3 sfx.py --self-test` checks the encoder setup.
- `scripts/make_glb.py`: an example `.glb` generator for 3D games (see the 3D section).

Put your own generator script in `games/<name>/tools/` and commit it (the art stays remixable). Draw anything that changes color with mood or state (skin, a liquid) in light greys and tint it at draw time: `rgba_tint` multiplies, so one sprite set gives smooth pink → red → purple transitions. Import the helpers with `sys.path.insert(0, "<this skill's directory>/scripts")` (in the Maimbrain repo: `.claude/skills/maimbrain-game/scripts`; from the plugin: the installed skill's directory). Better: copy the helper you need into `games/<name>/tools/` so the game's tools are self-contained. Make things big: the main character ≥ 1/3 of the screen width, anything the player must track ≥ ~40 units, and an art pixel ≥ 2–3 units. Small sprites that look fine in a desktop screenshot are unreadable on a phone. Store pixel art at `scale=4` (crisp under the platform's linear filtering) and draw each art pixel as a whole number of logical units. Make the icon (`icon.png`, 256×256) a close-up of your character or hook, not text.

### 4. Build, preview, look at it
- `mb build <game dir>` compiles, packs and validates. Fix every error and warning. (In the Maimbrain repo, `mb` is `mise exec -- cargo run -q -p mb-cli --`.)
- `mb serve <game dir>` runs it in a desktop browser at http://127.0.0.1:8765/runtime/index.html?overlays. It runs in the foreground, so start it in the background once; after each save, reload the page and it rebuilds. Use `--port` if 8765 is taken. Mouse = touch, mouse position = tilt, console = log, haptics and leaderboard calls. An installed `mb` has the web runtime built in; in the Maimbrain repo, run `just runtime` once first (it builds `runtime/dist`, which `cargo run -p mb-cli -- serve` then embeds automatically). If `mb serve` says it was built without the web runtime, that's what's missing.
- Preview aids (SPEC §9): `?overlays` shades the card overlays and the pause pill. `?speed=0.25` slows time. In the console, `mb.suspend()`, `await mb.step(n)` and `mb.resume()` freeze a moment for a screenshot (wait ~100 ms after stepping before the screenshot). A hidden tab or headless browser runs no animation frames, so there start a held session (`await mb.restart('{"hold": true}')`) and drive it with `mb.tap(x, y)` and `await mb.step(n)`. Add `DEBUG_*` constants (start level, force a shape, fake a sensor reading) to reach late states quickly, and reset them before you finish.
- **Check determinism** before you call it done: play a round (or script one in a held session), then `await mb.verify()` in the console. It replays the run frame by frame and compares draw hashes: `{ match: true }` is the goal; otherwise `firstMismatch` is the first frame that differs, and the cause is something unrecorded (randomness not from the session seed, `dt`-independent timing, hash-map iteration order, game state changed in `render`). SPEC §9 shows the manual version with `mb.restart({ hold: true })`, `mb.step` and `mb.snapshot()`.
- **Look at screenshots** of the title, the first seconds, mid-round, danger, failure and game-over. Check them against the [engagement checklist](engagement.md#checklist). Then fix what you see and look again.

### 5. Publish (when the user asks)
`mb publish <game dir>` builds the game, uploads it to maimbrain.com and waits for the server's validation. It then waits for a moderator's review; `maimbrain.com/create` shows the status and any reviewer notes. Ids must be in your namespace (`mb whoami` shows it: `com.maimbrain.<username>`, with `_` as `-`) and `creator` must be `@<username>` (`mb new` sets both). Each upload needs a higher `version` than the last.

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
- **Text**: the pixel font (id 2) is crisp at multiples of 8. Use ≥ 16 for anything the player must read, 8 only for fine print. Inter (0/1) for sizes that aren't multiples of 8. The pixel font is ASCII plus `× ÷ · ° • … ← ↑ → ↓ ♥ ★`; anything else (é, ü, €) draws as `?`, so use Inter for player names and other text that might need it (SPEC §5.3).

## 3D (mb3d)

Start from **[template-3d/](template-3d/)** (`mb new --3d games/<name>`): a small complete 3D game (catch the drone) that builds, validates and passes its tests as is. Read all three files before writing your own:
- `src/scene.rs` builds the scene once: the procedural sky (also the image-based lighting), a shadow-casting sun, post (bloom, vignette), meshes made in code (a lathed platform, pillars merged into one mesh, a faceted drone with a glowing ring as a child node), one particle emitter, one trail and two point lights (one on the drone, one flash that decays).
- `src/sim.rs` holds the rules *and* the camera, with tests: the tap test projects the drone with `Camera::project`, the same formula the host uses, so hit-testing is plain tested Rust.
- `src/lib.rs` is the host layer: rounds, input, moving nodes, and the HUD drawn with mb2d at `gfx3d::project` positions (a reticle on the drone, a hint under it).

The API is SPEC §5.4; the SDK binding is `maimbrain::gfx3d`.

- **Manifest**: `stdlib = { mb3d = 1, mb2d = 1 }` (mb2d for the HUD) and `perf_tier = "full"`.
- **Shape of a 3D game**: build meshes, materials, emitters, trails and nodes in `init` (or when assets arrive), keep their handles in your state, update node transforms in `update`, and in `render` call `gfx3d::camera(..)`, `gfx3d::render()`, then any mb2d HUD. The scene persists; don't recreate it every frame.
- **Nothing but nodes can be freed.** Meshes, textures, materials, models, emitters and trails last the whole session and count against the limits (512 meshes, 1024 materials, 64 textures, 64 models, 256 emitters, 64 trails; SPEC §5.4). Build them once and pool: one emitter per particle look fired wherever it's needed, one material per look (`Material::set` to pulse it), a fixed pool of enemy nodes hidden with `set_visible(false)` and reused, trails moved between nodes with `detach()` + `attach(node)` (that starts a fresh ribbon; the old one fades out where it was). `Node::destroy` does free a node and its children.
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

### HUD over 3D
`gfx3d::project(world)` gives the logical screen position (mb2d's coordinates), depth and whether it's on screen: put labels, reticles and off-screen arrows there. Everything mb2d draws in the frame lands on top of the 3D image. Call it after `gfx3d::camera` in `render` (it uses the latest camera). For game logic (lock-on, tap hit tests), use `camera.project(world, W, H)` on your own `Camera` value instead: the same formula (SPEC §5.4) in plain Rust, so it runs in `cargo test` (where `gfx3d::project` returns zeros) and agrees with the host to within float rounding.

### Previewing
`mb serve` works the same. The first few frames show the sky without its nebula while the lighting bakes; screenshots after `await mb.step(n)` with n ≥ 10 are representative. Check that effects replay with `await mb.verify()` (see Build, preview, look at it): explosions, particles and trails are part of the draw hash, so `match: true` means they land exactly where they did live.

## Before you call it done

- [ ] `mb build` validates with no warnings; `cargo test -p <name>` passes; the bot says the difficulty curve is right.
- [ ] `await mb.verify()` after a played round returns `match: true` (the run replays exactly).
- [ ] Every item in the [engagement checklist](engagement.md#checklist).
- [ ] Screenshots reviewed for the title card, game-over card, mid-round and both edges of danger.
- [ ] DESIGN.md updated to match what you built, with the "not verified on device" list and tuning constants.
