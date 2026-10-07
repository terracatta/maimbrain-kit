# Maimbrain Game Spec — v0 (draft)

This document is the build target for Maimbrain games. It is written to be followed by humans and AI agents. Words **MUST**, **SHOULD**, **MAY** are normative.

## 1. Bundle format (`.mbx`)

A `.mbx` is a ZIP archive (deflate or store):

```
manifest.toml        # required
game.wasm            # required, wasm32 core module
icon.png             # required, 256×256
src/                 # required: the Rust crate the wasm was built from
assets/              # optional
```

| Limit | Value |
|---|---|
| Total compressed size | ≤ 10 MB |
| `game.wasm` uncompressed | ≤ 4 MB (SHOULD be < 1 MB) |
| Startup set (wasm + assets listed in `startup_assets`) | ≤ 3 MB |
| Files | ≤ 512 |

Allowed asset types: `.png`, `.ktx2` (Basis/UASTC), `.glb` (meshopt allowed), `.ogg`/`.opus`, `.ttf`, `.wgsl`, `.bin`, `.json`, `.txt`.

## 2. Manifest

```toml
abi = 0                          # host ABI major version
id = "dev.example.stack"         # reverse-DNS, unique per creator
name = "Stack"
version = "0.1.0"
creator = "@jm"

orientation = "portrait"         # "portrait" | "landscape"
logical_size = [360, 640]        # game coordinate space; host letterboxes
inputs = ["touch", "mouse", "keyboard"]   # subset of touch|mouse|keyboard|gamepad
perf_tier = "lite"               # "lite" | "full"; sets the memory cap (§3)

stdlib = { mb2d = 1 }            # host engines + versions, e.g. { mb3d = 1 }
sensors = ["tilt"]               # subset of tilt|motion|loudness|light|haptics
needs = ["tilt"]                 # sensors the game can't be played without (subset of sensors, not haptics)
capabilities = ["store", "score"]# subset of store|score|text_input|links

startup_assets = ["assets/sprites.png"]

[[scores]]                       # one per leaderboard; requires capabilities = ["score"]
board = 0                        # 0–15, the `board` argument to mb_score_*
label = "Height"                 # 1–32 characters
order = "higher"                 # "higher" | "lower" is better
format = "integer"               # "integer" | "milliseconds" (shown as m:ss.mmm)

[remix]
allowed = true                   # default true
parent = ""                      # id@version of the game this remixes, if any
```

A game MUST NOT import a host module whose stdlib, sensor, or capability is not declared.

`needs` is for games played *with* a sensor (tilting, blowing into the mic): the feed only offers them on devices that have every sensor listed, and the game's `inputs` cover everything else (menus, starting a round). A game that uses a sensor only as garnish leaves it out of `needs` and MUST stay playable when the sensor is missing.

Example for a tilt-driven game: `inputs = ["touch"]` (taps for menus), `sensors = ["tilt", "haptics"]`, `needs = ["tilt"]`. `haptics` is listed under `sensors` because it is device hardware the platform gates the same way, even though it is an output. Audio and input need no declaration.

`startup_assets` is a size budget, not a preload: listed assets count toward the 3 MB startup set (they're what the first seconds of play need), but every asset — listed or not — loads asynchronously after `mb_asset_load` and MUST be polled with `mb_asset_state`. Draw something sensible until assets are ready.

Unknown manifest keys are rejected, so a typo fails validation rather than being ignored.

## 3. Execution model

- The module MUST target `wasm32-unknown-unknown` and MUST import only functions in the `mb` namespace (Section 5). No WASI.
- Allowed features are exactly WebAssembly 2.0: mutable globals, bulk memory, reference types, sign-extension, non-trapping float-to-int, multi-value, 128-bit SIMD. No threads, relaxed SIMD, exceptions, tail calls, memory64, multi-memory or GC (v0).
- Export names beginning with `__mb` are reserved for the platform.
- Linear memory: the declared maximum MUST be set, and MUST be ≤ 128 MB (2048 pages) for `perf_tier = "lite"` and ≤ 256 MB (4096 pages) for `"full"`. The feed MAY skip `full` games on 3 GB devices.

### Exports

| Export | Signature | Required | Notes |
|---|---|---|---|
| `memory` | memory | yes | |
| `mb_init` | `() -> ()` | yes | Called once. MUST return within 500 ms. First frame MUST follow. |
| `mb_update` | `(dt: f32) -> ()` | yes | Seconds since last update, clamped to ≤ 0.1. |
| `mb_render` | `() -> ()` | yes | Issue draw commands only. |
| `mb_suspend` | `() -> ()` | no | Game is going off screen. Persist anything important. |
| `mb_resume` | `() -> ()` | no | Game is back. Do not assume time continuity. |
| `mb_alloc` | `(len: u32) -> u32` | yes | Host uses it to hand variable-length data to the guest. |

The host owns the loop. It MAY stop calling `mb_update`/`mb_render` at any time (suspension) and MAY destroy the instance without notice after `mb_suspend`.

### Watchdogs

| Rule | Consequence |
|---|---|
| One callback exceeds 250 ms CPU | Instance killed |
| One callback runs more than 300 million loop iterations | Instance killed (loop metering, below) |
| 3 consecutive frames exceed 50 ms | Instance killed |
| GPU frame work exceeds 100 ms | Instance killed |
| `mb_init` → first input-ready frame > 1 s | Validator failure / feed demotion |

**Loop metering.** Before running a game, the client rewrites `game.wasm` so every `loop` decrements a fuel counter (exported as `__mb_fuel`) and traps at zero; the host refills it before each callback. This is what actually stops a game that never yields: the host can't preempt running wasm. Metering doesn't change behaviour and is deterministic, so it replays. (Cost: ~19% on a loop-heavy benchmark.)

### Determinism

A run MUST be exactly reproducible from its seed and recorded host inputs, so that the platform can replay it ("watch my run", ghosts, preview clips, bug reports).

- Every source of nondeterminism reaches the guest through a host call, and the host records it: per-frame `dt`, input events, sensor values, asset readiness, the session seed, and the session constants (`mb_player_id`, `mb_locale`, `mb_screen`, initial `mb_store` contents).
- `mb_time()` is **game time**: the sum of all `dt` values passed to `mb_update`, not wall-clock time.
- The host samples sensors once per frame and returns the same snapshot for every read during that frame.
- Input events, asset state changes and other host-side changes become visible only at frame boundaries, before `mb_update`.
- Guests MUST NOT depend on NaN bit patterns (the only nondeterminism in Wasm floating point). There are no threads and no other entropy sources.
- The replay log format is reserved; it is not part of v0.

## 4. ABI conventions

- All handles are `u32`; `0` is invalid/null.
- Strings and byte buffers are passed guest→host as `(ptr: u32, len: u32)`, UTF-8 for strings.
- Host→guest variable data: host calls `mb_alloc`, writes, and returns `(ptr,len)` packed as `u64` (`ptr << 32 | len`).
- Fallible functions return `i32`: `>= 0` success / value, `< 0` an error code (Section 7).
- Coordinates are in logical units, origin top-left, +y down (2D). 3D is right-handed, +y up, meters.

## 5. Host modules (import namespace `mb`)

Signatures are abbreviated; the Rust SDK (`sdk/maimbrain`) is the reference binding.

### 5.1 `sys` (always available)
```
mb_log(level: u32, ptr, len)           # level: 0 debug, 1 info, 2 warn, 3 error
mb_time() -> f64                      # game time: sum of dt passed to mb_update (§3 Determinism)
mb_rand_seed() -> u64                 # per-session seed; game owns its PRNG
mb_daily_seed() -> u64                # same for every player of this game on a UTC day (daily challenges)
mb_round(state: u32)                  # 0 idle (title/menu), 1 playing, 2 over; see Feed navigation below
mb_player_id(ptr) -> i32              # writes the 16-byte opaque id, unique per (player, game); returns 16
mb_locale(ptr, cap) -> i32            # BCP 47 tag; writes ≤ cap bytes, returns the full length
mb_screen(out_ptr)                    # logical w/h, safe-area insets (incl. the reserved feed strip), dpr
mb_asset_load(path_ptr, len) -> handle  # async; poll with mb_asset_state
mb_asset_state(h) -> i32              # 0 pending, 1 ready, <0 error
mb_asset_read(h, ptr, cap) -> i32     # raw bytes for .json/.bin/.txt; writes ≤ cap bytes, returns the full length
```

### 5.2 `input` (always available; events only for declared inputs)
```
mb_input_poll(buf_ptr, cap) -> i32    # writes packed InputEvent[]; returns count
```
`Screen` (32 bytes, written by `mb_screen`): `width:f32`, `height:f32`, `inset_top:f32`, `inset_right:f32`, `inset_bottom:f32`, `inset_left:f32`, `dpr:f32` (device pixels per logical unit), `reserved:f32`. Constant for the session.

`InputEvent` (32 bytes): `kind:u8` (1 touch_down, 2 touch_move, 3 touch_up, 4 touch_cancel, 5 key_down, 6 key_up, 7 mouse_down, 8 mouse_move, 9 mouse_up, 10 mouse_wheel, 11 pad_button, 12 pad_axis, 13 text), `id:u8`, `pad:u16`, `x:f32`, `y:f32`, `a:f32`, `b:f32`, `code:u32`, `time:f64`. Touch and mouse coordinates are in logical units; `time` is game time.

Key events carry the USB HID usage ID (keyboard page 0x07) in `code`, e.g. 4–29 = A–Z, 44 = Space, 40 = Enter, 79–82 = Right/Left/Down/Up; auto-repeat is not delivered. Gamepad events carry the pad index in `pad`, the W3C "standard" mapping button or axis index in `code`, and the value in `a` (buttons 0–1; axes −1…1 with a 0.1 dead zone). A game that declares only `touch` receives mouse input as touch.

**Feed navigation.** The feed has two modes. In *browse*, the game's card is visible but receives no touches: the player swipes anywhere to move between games, and a tap enters *play*; that tap is also delivered to the game (after the platform starts a fresh session, or as-is on a game-over screen), so one tap both enters play and starts the game. In *play*, the game receives every touch except on the platform's **pause pill** (top-left, inside the top safe area: about 64×44 points); the pill, an interruption, or the game reporting `mb_round(2)` (over) or `mb_round(0)` (idle) returns to browse. Games SHOULD report their round state; a game that never calls `mb_round` stays in play until the pill or an interruption. The system home-indicator zone at the bottom is never delivered to the game, and is included in the bottom inset. On a browse card the platform draws the game's title and creator over its bottom edge (about 70 points; on a replay card the replay banner and a "Beat …" button take the bottom ~230 points) and like/comment/report buttons down its right edge (about 56 points).: keep buttons and important text out of those areas on title and game-over screens, since that's what a card shows, and keep the HUD (score, level) at the top, where nothing covers it.

### 5.3 `mb2d` (stdlib `mb2d = 1`)
```
mb2d_image(asset) -> handle           # asset must be a ready PNG (≤ 4096², ≤ 128 images); < 0 if not ready / invalid
mb2d_font(asset) -> handle            # v0: returns -7 (unsupported); use the host fonts below
mb2d_clear(rgba: u32)
mb2d_push() / mb2d_pop()              # transform stack, ≤ 64 deep
mb2d_translate(x,y) / mb2d_rotate(r) / mb2d_scale(sx,sy)
mb2d_blend(mode: u32)                 # 0 alpha, 1 add, 2 multiply, 3 screen
mb2d_rect(x,y,w,h,rgba) / mb2d_circle(x,y,r,rgba) / mb2d_line(x0,y0,x1,y1,w,rgba)
mb2d_rect_gradient(x,y,w,h, rgba_top, rgba_bottom)
mb2d_poly(pts_ptr, n, rgba)           # n (x, y) f32 pairs, 3–512; simple polygons, convex or concave
mb2d_sprite(img, sx,sy,sw,sh, dx,dy,dw,dh, rgba_tint)   # src in image pixels, dst in logical units
mb2d_text(font, size, x, y, rgba, ptr, len)             # UTF-8; (x, y) = top-left of the first line
mb2d_measure(font, size, ptr, len) -> f32               # width of the widest line
```
Colors are `0xRRGGBBAA`. Transforms (applied to every draw, sprites and text included) and blend mode reset at the start of every `mb_render`. `mb2d_rotate` takes radians; positive turns clockwise on screen (+y is down). A sprite's `rgba_tint` multiplies the texel color and alpha, so `0xffffffff` draws it unchanged. Images are sampled with linear filtering. The canvas is drawn at up to 2 device pixels per logical unit (e.g. 720×1280 for a 360×640 game on an iPhone), so for crisp pixel art make texels per art pixel = 2 × the logical units you draw each art pixel at (e.g. store at 4×4 texels per art pixel and draw art pixels 2 units wide), and keep sprite positions on whole units. `size` is the em height in logical units; `\n` starts a new line. Host fonts: `0` Inter Regular, `1` Inter Bold (signed-distance-field, crisp at any size), `2` pixel font (5×7, crisp at multiples of 8). Glyphs cover ASCII and Latin-1; missing characters draw as `?`. About 32 000 quads per frame; further draws are dropped.

### 5.4 `mb3d` (stdlib `mb3d = 1`)
Retained scene graph, host-rendered. Nodes are handles.
```
mb3d_scene_load(glb_asset) -> node        # instantiate glTF scene
mb3d_node_new() -> node
mb3d_node_parent(node, parent)
mb3d_node_transform(node, ptr)            # pos[3], rot quat[4], scale[3]
mb3d_node_visible(node, bool)
mb3d_mesh_box/sphere/plane(...) -> mesh
mb3d_material_pbr(ptr) -> material        # base color, metallic, roughness, emissive, textures
mb3d_node_mesh(node, mesh, material)
mb3d_camera(node, fov, near, far)         # sets active camera
mb3d_light_dir(node, rgb, intensity, shadows: bool)
mb3d_light_point(node, rgb, intensity, range)
mb3d_environment(ktx2_asset, intensity)   # IBL + skybox
mb3d_post(ptr)                            # tonemap, exposure, bloom, vignette
mb3d_raycast(origin, dir, out_ptr) -> i32
mb3d_particles(...) -> node               # v1 GPU particle emitter
mb3d_render()                             # draw scene this frame; may combine with mb2d overlay
```
No physics in v1: games use their own (e.g. a small allowlisted crate) or the SDK's simple helpers.

### 5.5 `gpu` (escape hatch)
A handle-based subset of `webgpu.h`: `mb_gpu_buffer_create`, `mb_gpu_buffer_write`, `mb_gpu_texture_create`, `mb_gpu_texture_write`, `mb_gpu_sampler_create`, `mb_gpu_shader_create(wgsl_ptr,len)`, `mb_gpu_bind_group_layout_create`, `mb_gpu_pipeline_layout_create`, `mb_gpu_render_pipeline_create`, `mb_gpu_compute_pipeline_create`, `mb_gpu_bind_group_create`, `mb_gpu_pass_begin`, `set_pipeline`, `set_bind_group`, `set_vertex_buffer`, `set_index_buffer`, `draw`, `draw_indexed`, `dispatch`, `mb_gpu_pass_end`, `mb_gpu_surface_texture() -> handle`. Descriptors are passed as packed structs (layouts defined in SDK).

Rules:
- Shaders MUST be WGSL; the host validates with naga and rejects anything outside the allowed feature set.
- Limits: 128 MB GPU memory, max texture 4096², ≤ 256 buffers, ≤ 64 textures, ≤ 2000 draws/frame, workgroup ≤ 256 invocations, ≤ 65535 dispatches/frame.
- No buffer mapping for readback in v0 (prevents fingerprinting/timing side channels).

### 5.6 `audio` (always available)
```
mb_sound(asset) -> handle                  # asset must be ready (.ogg); decodes in the background
mb_play(sound, vol, pan, pitch, loop: bool) -> voice   # vol 0–4, pan −1…1, pitch = playback rate
mb_voice_set(voice, vol, pan, pitch) / mb_voice_stop(voice)
mb_voice_pos(voice, x,y,z)                 # 3D positional (with mb3d camera as listener)
mb_bus_fx(bus, kind, ptr)                  # lowpass, highpass, reverb, delay
mb_synth(ptr) -> sound                     # sfxr-style params → generated sound
```
Use mono Ogg Vorbis, 22–48 kHz (`oggenc` from vorbis-tools makes it from WAV). `vol` 1 plays the file at its own level; normalize sounds to roughly the same loudness and mix with `vol`. A looping voice repeats the decoded file sample-accurately, but Vorbis encoders can pad the end, so check music loops for a gap. ≤ 32 simultaneous voices (beyond that the oldest one-shot is stopped, then the oldest loop). Volume, pan and pitch changes glide over ~10 ms, so updating them every frame doesn't click. Audio plays on feed cards too (the live title and game-over screens, and replays) when the player has sound on, so keep a title screen's sound sparse. Audio is output-only: handles are sequential; a one-shot played before its sound has finished decoding is skipped, and a loop starts as soon as decoding finishes; and nothing about playback is visible to the game, so audio never affects determinism. Implemented in v0: `mb_sound`, `mb_play`, `mb_voice_set`, `mb_voice_stop`.

### 5.7 `sensors` (only declared)
```
mb_tilt(out_ptr) -> i32       # writes Tilt; 0 on success, -6 before the first sample
mb_motion(out_ptr) -> i32     # writes Motion; 0 on success, -6 before the first sample
mb_loudness() -> f32          # 0..1 smoothed mic level, or -1 while unavailable; never raw audio
mb_light() -> f32             # 0..1 ambient light from camera; never frames
mb_haptic(kind: u32)          # 0 tap (light tick), 1 success, 2 fail, 3 heavy (strong thud)
```
`Tilt` (12 bytes): `x:f32`, `y:f32`, `z:f32` — the gravity vector in g, in the game's axes: +x right, +y down the screen, +z out of the screen toward the player. Held upright in portrait it reads about (0, 1, 0); flat on a table face-up, (0, 0, −1). It is low-pass filtered (no shake or sudden-movement component) and sampled once per frame like every sensor (§3). There is no calibration: a game that wants "relative to how the player is holding it" records a reference vector and compares against it. In a desktop browser the mouse pointer emulates it: horizontal position rolls the phone and vertical position pitches it, from upright at the window's centre to ±90° two-thirds of the way to each edge.

Like touches, sensor readings reach the game only while it is being played (feed *play* mode): on a feed card, `mb_tilt` keeps returning the last snapshot (or −6 if there has never been one), so never start a round from a sensor reading alone — wait for a tap. When play begins, a fresh reading arrives no later than the frame that delivers the entering tap, so calibrating on that tap is safe.

`Motion` (12 bytes): `x:f32`, `y:f32`, `z:f32` — acceleration with gravity removed, in g, in the same axes as `Tilt`. About zero when still, ±1–3 g in a vigorous shake. It is one snapshot per frame (~1/60 s), so measure shakes by accumulating readings over time. In a desktop browser, shaking the mouse emulates it (a brisk shake reads ~2–3 g).

`mb_loudness` is the mic's level shaped to 0–1 (RMS on a −60…0 dBFS scale, fast attack ~30 ms, slower release ~150 ms): Measured on an iPhone 16 Pro: a quiet room with the game's own music playing reads ~0.40–0.55, and the hardest blow into the mic ~0.86 (iOS's input gain control lifts quiet sound and squashes loud sound, so the usable range is narrow; treat ~0.85 as full strength). It returns −1 until permission is granted, if the player denied it, or before the first reading; a game that needs it MUST show what to do (e.g. "ALLOW THE MIC TO PLAY") rather than appear broken. The platform asks for permission the first time the game is played, the mic runs only while the game is being played, and the platform shows a mic badge beside the pause pill while it's on. The game never receives audio. In a desktop browser it reads the computer's microphone if allowed; holding 1–9 gives 0.1–0.9 and Space a strong blow (0.85).

Readings of every sensor are quantized (1/4096 g, 1/1024 loudness) before the game sees them and are recorded, so runs replay exactly.

`mb_haptic` is output only, like audio: nothing is returned and it never affects determinism. The platform plays haptics only while the game is being played (not as a feed card or replay) and drops requests beyond about 30 per second.

Using `light` will trigger a platform permission prompt and an on-screen indicator. Implemented in v0: `mb_tilt`, `mb_motion`, `mb_loudness`, `mb_haptic`.

### 5.8 `store` / `score`
```
mb_store_get(key_ptr,len, buf_ptr,cap) -> i32   # writes ≤ cap bytes, returns the value's length, or -1 if absent
mb_store_set(key_ptr,len, val_ptr,len) -> i32   # keys 1–64 bytes; empty value deletes; total quota 256 KB per game
mb_score_submit(board: u32, value: i64) -> i32  # board must be declared in [[scores]] (else -3)
mb_score_show(board: u32)                       # platform overlay; the game is suspended while it shows
```

### 5.9 `text_input` / `links`
```
mb_text_begin(hint_ptr,len, max_len)    # host shows native keyboard/IME
mb_text_end()                           # text arrives as input events
mb_link_open(url_ptr,len) -> i32        # always interstitial; http(s) only
```

Multiplayer (`net`) and player-to-player chat are not part of v0. The `net` capability name is reserved.

## 6. Content & review rules

- Playable within 1 s of load, with no mandatory menu or tutorial gate.
- No monetization of any kind: no ads, no purchases, no payment or donation links.
- Each `inputs` entry the manifest declares MUST make the game fully playable on its own, together with the sensors in `needs`.
- Source in `src/` MUST build to the shipped `game.wasm` using the platform toolchain and the crate allowlist. `build.rs` and proc-macro crates outside the allowlist are rejected.

## 7. Error codes

`-1` invalid handle · `-2` invalid argument · `-3` not declared in manifest · `-4` quota exceeded · `-5` rate limited · `-6` not ready · `-7` unsupported on this device

## 8. Feed wire format (client ↔ server contract)

`GET /feed.json`
```json
{
  "version": 0,
  "items": [
    {
      "id": "dev.example.stack",
      "version": "0.1.0",
      "url": "games/stack-0.1.0.mbx",
      "sha256": "…",
      "size": 812345,
      "title": "Stack",
      "creator": "@jm",
      "icon": "games/stack.png",
      "orientation": "portrait",
      "inputs": ["touch", "mouse", "keyboard"],
      "needs": [],
      "perf_tier": "lite",
      "likes": 0,
      "remix_of": null
    }
  ]
}
```
The client downloads, verifies `sha256` and size, re-validates the manifest locally, then caches the bundle.

## 9. Building, previewing and testing

- `mb build games/<name>` compiles the crate for `wasm32-unknown-unknown`, optimizes it, packs `feed/<id>-<version>.mbx` and validates it. `mb validate` checks an existing bundle; `mb <command> --help` documents each command.
- `mb serve <game dir>` builds the game and serves it with the runtime at http://127.0.0.1:8765/runtime/index.html for a desktop browser with WebGPU (Safari or Chrome); reloading the page rebuilds it if any file changed. (In this repo, `just serve <name>` does the same.) There is no feed around it: touch arrives as mouse clicks, tilt is emulated by the mouse pointer (§5.7), haptics and leaderboards are logged to the console, and the insets are zero except a small bottom one. Leave it running and reload to iterate.
- Preview aids:
  - `?overlays` in the URL shades where the feed draws over a card, plus the pause pill (and mic badge).
  - `?speed=0.25` runs live time at a quarter speed.
  - Console calls:
    - `mb.suspend()` pauses, `mb.step(n)` advances a paused game n frames, `mb.resume()` continues, and `mb.speed(k)` changes the speed.
    - `mb.tap(x, y)` taps at page (CSS) pixels.
    - `mb.sensors(gx, gy, gz, ax, ay, az)` sets tilt and motion until the pointer next moves. Synthetic input works too, e.g. `window.dispatchEvent(new PointerEvent("pointermove", { clientX, clientY }))` in a timed loop to script a shake.
    - `mb.loudness(v)` pins the mic level; `mb.loudness(-1)` simulates a denied mic, and `mb.loudness()` unpins.
    - `mb.snapshot()` returns `{ frames, time, hash, log }` (the replay log so far).
    - `mb.restart(JSON.stringify({ replay: mb.snapshot().log }))` plays that run back the way a replay card does; `mb.restart("{}")` starts a fresh session.
  - Pausing, then pinning sensors and stepping, makes any moment reproducible for a screenshot. For hard-to-reach states (level 7, a near-fail), add debug constants to your game (e.g. `const DEBUG_START_LEVEL: u32 = 0;`) and set them back before shipping.
- The SDK compiles on the host too: there, every host call is a no-op stub returning zero (`sensors::tilt()` is `None`), so `cargo test -p <game>` can test game logic natively. Keep the simulation separate from drawing and input so tests can drive it directly.
- Assets are yours to make: PNG for images, Ogg Vorbis for sound. Generating them with scripts kept in the crate (e.g. `tools/`) keeps the game remixable.
