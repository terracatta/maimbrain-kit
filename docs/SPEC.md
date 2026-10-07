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

Allowed asset types: `.png`, `.jpg`/`.jpeg` (mb3d textures), `.ktx2` (Basis/UASTC), `.glb` (≤ 8 MB; meshopt allowed, see §5.4), `.ogg`/`.opus`, `.ttf`, `.wgsl`, `.bin`, `.json`, `.txt`.

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
| `mb_update` | `(dt: f32) -> ()` | yes | Seconds since last update, clamped to ≤ 0.1 (`mb_time_lost` reports the rest, §5.1). |
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

- Every source of nondeterminism reaches the guest through a host call, and the host records it: per-frame `dt` and lost time (`mb_time_lost`), input events with their own times, sensor values, asset readiness, the `mb_suspend`/`mb_resume` calls made between frames, the session seed, and the session constants (`mb_player_id`, `mb_locale`, `mb_screen`, initial `mb_store` contents). A replay makes the same lifecycle calls at the same points and no others (pausing a replay doesn't call the guest).
- `mb_time()` is **game time**: the sum of all `dt` values passed to `mb_update`, not wall-clock time.
- The host samples sensors once per frame and returns the same snapshot for every read during that frame.
- Input events, asset state changes and other host-side changes become visible only at frame boundaries, before `mb_update`.
- Host-side simulation (mb3d's particles, trails and shockwaves, §5.4) advances only by the recorded `dt` and the guest's calls.
- Guests MUST NOT depend on NaN bit patterns (the only nondeterminism in Wasm floating point). There are no threads and no other entropy sources.
- The **draw hash** checks this: a running FNV-1a hash of every mb2d draw command and mb3d call the guest makes, plus a digest of mb3d's simulated state at each `mb3d_render`, starting over with each session. Two runs that drew the same thing frame for frame have the same hash at the same frame count; §9 shows how to compare a run with its replay.
- The replay log format is reserved; it is not part of v0. It is versioned, and a log keeps replaying exactly on later runtimes (fields added since version 0, such as per-event times and lost time, default to what version 0 meant).

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
mb_time_lost() -> f32                 # wall-clock seconds this frame that game time didn't get (below); usually 0
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

`mb_time_lost` is the time this frame dropped: when a frame comes more than 0.1 s after the last (a hitch, a stall, the page briefly not drawing), `dt` is clamped to 0.1 and the rest is reported here, for that frame only. It is recorded and replayed like `dt`. Game time is then behind the wall clock for good, and so is everything scheduled on it: music started with `mb_play_at` keeps playing and is now `mb_time_lost` ahead of the game's grid (§5.6), so a game that keeps time with music re-anchors when it's > 0. A suspend is not lost time: across `mb_suspend`/`mb_resume` game time simply pauses, the first frame after `mb_resume` has a nominal `dt` and `mb_time_lost` 0, and the platform keeps scheduled audio on game time across the gap (§5.6); the game learns of the gap from `mb_resume` itself. In the preview, `mb.speed(k)` scales lost time like `dt`.

### 5.2 `input` (always available; events only for declared inputs)
```
mb_input_poll(buf_ptr, cap) -> i32    # writes packed InputEvent[]; returns count
```
`Screen` (32 bytes, written by `mb_screen`): `width:f32`, `height:f32`, `inset_top:f32`, `inset_right:f32`, `inset_bottom:f32`, `inset_left:f32`, `dpr:f32` (device pixels per logical unit), `reserved:f32`. Constant for the session.

`InputEvent` (32 bytes): `kind:u8` (1 touch_down, 2 touch_move, 3 touch_up, 4 touch_cancel, 5 key_down, 6 key_up, 7 mouse_down, 8 mouse_move, 9 mouse_up, 10 mouse_wheel, 11 pad_button, 12 pad_axis, 13 text), `id:u8`, `pad:u16`, `x:f32`, `y:f32`, `a:f32`, `b:f32`, `code:u32`, `time:f64`. Touch and mouse coordinates are in logical units. `time` is the game time the event itself happened, finer than a frame: the host maps the event's own timestamp onto the span of game time the frame advances, so a touch midway between two frames gets a time midway between their game times. It is never earlier than the previous update's `mb_time()`, never later than the current one's, and never earlier than the event before it (so a frame's events are in order, and a game that applies them before judging misses in each update stays causal). Touch, mouse and key events carry their own times; gamepad events (polled once a frame) and text get the frame's game time. The times are recorded with the events, so replays see exactly the same values. `time` doesn't include output latency: a player who taps exactly on a beat they see and hear registers about a frame of presentation plus the touch screen's own latency late, which is what a rhythm game's self-calibration absorbs (§5.6).

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
Colors are `0xRRGGBBAA`. Transforms (applied to every draw, sprites and text included) and blend mode reset at the start of every `mb_render`. `mb2d_rotate` takes radians; positive turns clockwise on screen (+y is down). A sprite's `rgba_tint` multiplies the texel color and alpha, so `0xffffffff` draws it unchanged. Images are sampled with linear filtering. The canvas is drawn at up to 2 device pixels per logical unit (e.g. 720×1280 for a 360×640 game on an iPhone), so for crisp pixel art make texels per art pixel = 2 × the logical units you draw each art pixel at (e.g. store at 4×4 texels per art pixel and draw art pixels 2 units wide), and keep sprite positions on whole units. `size` is the em height in logical units; `\n` starts a new line. Host fonts: `0` Inter Regular, `1` Inter Bold (signed-distance-field, crisp at any size), `2` pixel font (5×7, crisp at multiples of 8). Inter covers printable ASCII, Latin-1 (U+00A0–U+00FF except the soft hyphen) and `‘ ’ “ ” • … – — ← ↑ → ↓ ★ ♥ € ™`. The pixel font covers printable ASCII (U+0020–U+007E) plus `× ÷ · ° • … ← ↑ → ↓ ♥ ★`: no accented letters, so use Inter for names and other text that may need them. Missing characters draw as `?`. About 32 000 quads per frame; further draws are dropped.

### 5.4 `mb3d` (stdlib `mb3d = 1`)
A retained 3D scene that the host renders: the game creates meshes, materials and nodes once, moves nodes and the camera each frame, and calls `mb3d_render()` from `mb_render`. Declare `perf_tier = "full"` for anything substantial; add `mb2d = 1` to draw a HUD on top. Units are meters; right-handed, +y up; a camera looks down its local −z. Colors are **linear** RGB (convert picked sRGB colors; the SDK's `srgb()` does) and may exceed 1, which is how things glow and bloom.

Handles are `u32`, sequential per session from 1 (`0` is invalid). Calls that create something return the handle or a §7 code. Packed structs are little-endian `f32`/`u32`, 4-byte aligned, passed by pointer.

```
# resources
mb3d_mesh(ptr, len) -> i32                 # packed mesh (below)
mb3d_texture(asset) -> i32                 # a ready PNG or JPEG asset, ≤ 2048²
mb3d_material(ptr) -> i32                  # Material
mb3d_material_set(material, ptr) -> i32    # replace a Material (e.g. pulse its emissive); 0 or -1
mb3d_gltf(asset) -> i32                    # a ready .glb → a model
mb3d_model_spawn(model, parent) -> i32     # instantiates a model under parent (0 = root); returns its root node
# scene graph
mb3d_node() -> i32                         # at the root, identity transform, visible, no mesh
mb3d_node_parent(node, parent)             # 0 = root; a parent can't be the node or its descendant
mb3d_node_transform(node, ptr)             # Transform, relative to the parent
mb3d_node_mesh(node, mesh, material)       # a node draws at most one mesh; mesh 0 clears it
mb3d_node_visible(node, visible)           # hides the node and everything under it
mb3d_node_destroy(node)                    # and its children; their lights go out, attached trails fade
mb3d_camera(ptr)                           # Camera
mb3d_project(x, y, z, out_ptr) -> i32      # writes f32 × 3: logical screen x, y, depth; 1 if on screen (formula below)
# lighting, sky, post
mb3d_sun(ptr)                              # Sun
mb3d_light_point(node, r, g, b, intensity, range) -> i32   # attach or update a node's point light; ≤ 8 (Limits)
mb3d_sky(ptr)                              # Sky
mb3d_post(ptr)                             # Post
# effects
mb3d_emitter(ptr) -> i32                   # Emitter
mb3d_emit(emitter, x, y, z, dx, dy, dz, count)   # a burst at a point, in a cone around (dx,dy,dz); zero = all around
mb3d_emit_moving(emitter, x, y, z, dx, dy, dz, count, vx, vy, vz)   # the same, in a frame moving at (vx,vy,vz) m/s
mb3d_trail(ptr) -> i32                     # Trail
mb3d_trail_attach(trail, node)             # follow the node's world position; a new node (or any re-attach) starts a new ribbon
mb3d_trail_detach(trail)                   # stop following; what's there fades out
mb3d_shockwave(x, y, z, radius, strength, seconds)   # screen-space ring expanding from a world point
mb3d_render()                              # draw the scene this frame
```

**Packed mesh**: header `vertex_count, index_count, flags` (u32), then `vertex_count` vertices `pos f32×3, normal f32×3, uv f32×2` followed by `tangent f32×4` (xyz, w = ±1 handedness) if `flags & 1` and `color rgba8` (bytes r, g, b, a; sRGB; multiplies the base color) if `flags & 2`, then `index_count` u32 indices: counter-clockwise triangles are front faces. `index_count` is a multiple of 3; every index < `vertex_count`; values finite. Without tangents the host generates them from the UVs. A mesh outside these rules returns −2. UV (0, 0) is a texture's top-left.

| Struct | Size | Fields |
|---|---|---|
| `Transform` | 40 | `pos f32×3, rotation f32×4` (quaternion x, y, z, w; normalized by the host), `scale f32×3` |
| `Camera` | 40 | `pos f32×3, rotation f32×4, fov_y` (radians, vertical), `near, far` |
| `Material` | 64 | `base_color f32×4` (linear rgb, alpha), `emissive f32×3, emissive_strength, metallic, roughness, base_tex, normal_tex, metal_rough_tex, emissive_tex` (texture handles, 0 = none), `flags` (1 unlit, 2 double-sided, 4 alpha blend, 8 additive), `reserved` |
| `Sun` | 36 | `direction f32×3` (the way the light travels), `color f32×3, intensity, shadows` (u32 0/1), `shadow_distance` (meters from the camera) |
| `Sky` | 64 | `seed` (u32), `colors f32×9` (background, nebula 1, nebula 2), `nebula_density` (0–4), `star_density` (0–4), `star_brightness, sun_disc` (angular radius, radians; 0 = none), `reserved u32×2` |
| `Post` | 32 | `exposure, bloom_strength, bloom_threshold, vignette` (0–1), `chromatic_aberration` (0–1), `saturation, contrast, reserved` |
| `Emitter` | 128 | `colors f32×16` (rgba at life 0, ⅓, ⅔, 1), `sizes f32×4` (diameter, same stops), `lifetime f32×2` (min, max s), `speed f32×2` (m/s), `spread` (cone half-angle; π = all around), `drag` (per second), `blend` (u32: 0 additive, 1 alpha), `texture` (0 = soft dot), `gravity f32×3, stretch` (length += stretch × speed: sparks) |
| `Trail` | 64 | `color_start f32×4, color_end f32×4, width_start, width_end, lifetime` (s), `blend` (u32: 0 additive, 1 alpha), `min_segment` (meters between points; 0 = 5 cm), `reserved u32×3` |

**Projection** (`mb3d_project`, using the camera from the latest `mb3d_camera` call, after the clamps that call applies: rotation normalized, `fov_y` to 0.01–3, `near` ≥ 10⁻⁴, `far` > `near`): with the camera's axes `r = rot·(1,0,0)`, `u = rot·(0,1,0)`, `b = rot·(0,0,1)` and `d = p − pos`, the depth is `depth = −d·b` (meters in front of the camera). If `depth ≤ 10⁻⁶` it writes `(0, 0, depth)` and returns 0. Otherwise, with `f = 1 / tan(fov_y / 2)` and the logical size `W × H`, `nx = f·(H/W)·(d·r)/depth` and `ny = f·(d·u)/depth`; it writes `x = (nx + 1)/2 · W`, `y = (1 − ny)/2 · H`, `depth`, and returns 1 when `|nx| ≤ 1`, `|ny| ≤ 1` and `near ≤ depth ≤ far`, else 0. (0, 0) is the top-left of the logical screen, the same space as mb2d and input. The host computes this through a combined view-projection matrix, so a game's own implementation agrees to within float rounding; the SDK's `Camera::project` is one, usable in tests.

Defaults before any call: camera at (0, 0, 5) looking down −z, `fov_y` 1, near 0.1, far 500; a white sun shining down and away (intensity 3, no shadows); a dark blue sky; exposure 1, bloom 0.5 above 1, vignette 0.25.

Textures: `base_tex` and `emissive_tex` are sampled as sRGB; `normal_tex` (tangent space, OpenGL/glTF convention) and `metal_rough_tex` (glTF layout: roughness in green, metallic in blue, multiplying the factors) as linear. Textures are mipmapped and sampled with repeat and trilinear, anisotropic filtering. A texture whose bytes fail to decode draws as white.

**glTF**: `.glb` only (one file, embedded buffer and images). Supported: triangle primitives; float or quantized attributes (`KHR_mesh_quantization`); `EXT_meshopt_compression` / `KHR_meshopt_compression` buffer views; PNG and JPEG images; metallic-roughness materials with base color, normal, metal-rough and emissive textures, `KHR_texture_transform` (on base-color UVs), `KHR_materials_emissive_strength`, `KHR_materials_unlit`, alpha mode `BLEND` (`MASK` draws opaque); node hierarchies (TRS or matrix) of the default scene. Skins, morph targets, animations, cameras, lights and other UV sets are ignored. A file that *requires* another extension (Draco, KTX2/Basis, WebP) returns −7; a malformed one −2. A model's meshes, materials and textures count toward the limits below. A node with several primitives spawns one child node per extra primitive.

**Lighting**: metallic/roughness PBR (GGX, Smith-correlated visibility, Schlick fresnel, Lambert diffuse) from the sun, up to 8 point lights (inverse-square falloff reaching 0 at `range`), and image-based lighting from the sky. Unlit materials draw `base_color × texture + emissive`. With `shadows = 1` the sun casts a 2048² soft shadow map over `shadow_distance` in front of the camera; opaque, lit meshes cast. The sky (a seeded procedural nebula and starfield, plus the sun disc) is also the source of image-based lighting, re-baked over the next few frames whenever `mb3d_sky` changes it — set it once, not every frame.

**Drawing order**: opaque meshes front to back, the sky, alpha and additive meshes back to front, then particles (alpha ones sorted back to front, then additive) and trails, all into an HDR target with 4× MSAA; then bloom, shockwave and chromatic distortion, exposure, saturation, ACES-fitted tonemapping, contrast and vignette. In a frame that calls `mb3d_render`, the scene fills the logical canvas and replaces `mb2d_clear`; every mb2d draw in that frame (before or after the call) composites on top, in display space. A game may use mb3d alone. The 3D image renders at up to 2 device pixels per logical unit, scaled down to as little as 0.6× when frames run long; that changes only resolution.

**Effects** are simulated by the host, deterministically: a burst's random numbers come from the emitter handle and how many bursts it has made (`mb3d_emit` and `mb3d_emit_moving` share the count); particles, trails and shockwaves advance by each frame's `dt` (the value `mb_update` gets) before `mb_update`, whether or not the frame renders; trails record their node's world position at each `mb3d_render`. Particles move ballistically in world space (`gravity`, `drag`), don't collide and aren't lit; a particle has no velocity but the one its burst gives it. With `mb3d_emit_moving`, each particle also carries the burst's frame velocity `v` unchanged for its whole life: position advances by `(vel + v)·dt`, while gravity and drag act on `vel` only, so the cloud keeps pace with whatever exploded (an enemy flying alongside the camera) and spreads as it would at rest. Sparks stretch along `vel`. A trail is one list of points: attaching it to a node other than the one it follows, including re-attaching after `mb3d_trail_detach` or after its node was destroyed, keeps the old points (they fade out over their lifetime as usual) and starts a separate ribbon at the node's next sampled position; nothing is drawn between the two. Attaching it to the node it already follows changes nothing. Every mb3d call and a digest of the simulated state are part of the replay hash (§3).

**Limits** (going over returns −4): 4096 live nodes, 512 meshes with 1 M vertices in total, 1024 materials, 64 textures (≤ 2048² each, sharing 96 MB with their mipmaps: a 2048² texture takes 22 MB, a 1024² one 5.6 MB), 64 models, 8 point lights, 256 emitters and 16 384 live particles (a burst adds ≤ 4096; extra particles are dropped), 64 trails (256 points each), 16 shockwaves (the oldest gives way), 2000 drawn meshes per frame (nearest first). With these, mb3d's GPU memory stays under 256 MB.

Only nodes can be freed: `mb3d_node_destroy` releases the node and its descendants (their slots are reused with new handles; old handles go stale and calls with them do nothing), the point lights attached to them, and detaches their trails. Meshes, textures, materials, models, emitters and trails last until the session ends, and count against the limits for the whole session. So build them once (in `mb_init`, or as assets arrive) and pool: one emitter per particle look, fired wherever needed; one material per look, changed with `mb3d_material_set`; a fixed set of trails moved between nodes with attach/detach; nodes hidden with `mb3d_node_visible` and reused rather than created per event. A point light holds one of the 8 slots from `mb3d_light_point` until its node (or an ancestor) is destroyed: at intensity 0 or under a hidden node it still holds the slot, and a hidden node's light still shines. Darken a light with intensity 0; to move a flash around, keep one light node and move it.

**Budgets**: everything a call does happens in the guest's callback except texture decoding, GPU uploads and drawing, which the host does after `mb_render`. Parsing a large `.glb` in `mb3d_gltf` takes time in the caller's callback (§3 watchdogs): load models in `mb_update`, one per frame, while showing something. Aim for ≤ 200 drawn meshes and ≤ 150 k triangles per frame, and a few thousand particles, for 60 fps on an iPhone 13 class device.

No physics in v1: games use their own (a small allowlisted crate) or the SDK's simple helpers.

### 5.5 `gpu` (escape hatch)
A handle-based subset of `webgpu.h`: `mb_gpu_buffer_create`, `mb_gpu_buffer_write`, `mb_gpu_texture_create`, `mb_gpu_texture_write`, `mb_gpu_sampler_create`, `mb_gpu_shader_create(wgsl_ptr,len)`, `mb_gpu_bind_group_layout_create`, `mb_gpu_pipeline_layout_create`, `mb_gpu_render_pipeline_create`, `mb_gpu_compute_pipeline_create`, `mb_gpu_bind_group_create`, `mb_gpu_pass_begin`, `set_pipeline`, `set_bind_group`, `set_vertex_buffer`, `set_index_buffer`, `draw`, `draw_indexed`, `dispatch`, `mb_gpu_pass_end`, `mb_gpu_surface_texture() -> handle`. Descriptors are passed as packed structs (layouts defined in SDK).

Rules:
- Shaders MUST be WGSL; the host validates with naga and rejects anything outside the allowed feature set.
- Limits: 128 MB GPU memory, max texture 4096², ≤ 256 buffers, ≤ 64 textures, ≤ 2000 draws/frame, workgroup ≤ 256 invocations, ≤ 65535 dispatches/frame.
- No buffer mapping for readback in v0 (prevents fingerprinting/timing side channels).

### 5.6 `audio` (always available)
```
mb_sound(asset) -> handle                  # asset must be ready (.ogg); decodes in the background
mb_play(sound, vol, pan, pitch, loop: bool) -> voice   # vol 0–4, pan −1…1, pitch = playback rate; starts now
mb_play_at(sound, vol, pan, pitch, loop: bool, at: f64) -> voice   # heard from game time `at`
mb_voice_set(voice, vol, pan, pitch) / mb_voice_stop(voice)
mb_voice_pos(voice, x,y,z)                 # 3D positional (with mb3d camera as listener)
mb_bus_fx(bus, kind, ptr)                  # lowpass, highpass, reverb, delay
mb_synth(ptr) -> sound                     # sfxr-style params → generated sound
```
Use mono Ogg Vorbis, 22–48 kHz (`oggenc` from vorbis-tools makes it from WAV). `vol` 1 plays the file at its own level; normalize sounds to roughly the same loudness and mix with `vol`. A looping voice repeats the decoded file sample-accurately, but Vorbis encoders can pad the end, so check music loops for a gap. ≤ 32 simultaneous voices, counting voices scheduled but not yet playing (beyond that the oldest one-shot is stopped, then the oldest loop). Volume, pan and pitch changes glide over ~10 ms, so updating them every frame doesn't click. Audio plays on feed cards too (the live title and game-over screens, and replays) when the player has sound on, so keep a title screen's sound sparse. Implemented in v0: `mb_sound`, `mb_play`, `mb_play_at`, `mb_voice_set`, `mb_voice_stop`.

**Audio is output-only.** Handles are sequential and returned at once, and nothing about playback (decode state, position, latency) is visible to the game, so audio never affects determinism: the game decides *when* a sound should be heard, in game time, and the platform makes it so.

**`mb_play` starts a voice now**, for sounds that answer something that just happened (hits, UI). A one-shot whose sound hasn't finished decoding is skipped; a loop starts as soon as decoding finishes, whenever that is.

**`mb_play_at` makes a voice heard when game time reaches `at`.** Each frame the host maps game time to the wall clock (the frame being produced is seen when it reaches the screen, and game time `g` is heard when the frame showing `g` is seen), and the wall clock to the audio output (including the output's latency: speaker, wired or Bluetooth), so music lines up with the picture. `at` may be in the future (schedule ~0.1 s ahead or more and the voice starts from its first sample) or in the past. If the voice can only start after `at` (its sound finished decoding late, the audio output was still starting, `at` had already passed or was closer than the output latency), it starts *offset into the sound* by its lateness, so it is still exactly where it would have been: a loop joins at `lateness mod length`, and a one-shot more than half its length late is skipped. So:
- **Decode timing doesn't matter.** Scheduling a loop before its sound has decoded is fine: it joins on time when it's ready. The asset itself must be ready to get a sound handle, so a game whose asset arrives late calls `mb_play_at` then with the original `at`, and that voice joins on the grid too.
- **Loops started with the same `at` stay sample-aligned** with each other (stems), even when they start in different frames or one of them starts late.
- A voice keeps the timeline it was scheduled on. If game time later loses time (`mb_time_lost` > 0, §5.1), voices already playing keep playing, now ahead of the game's grid; voices not yet started are rescheduled to the new mapping.

**Suspend and resume.** While the game is suspended (`mb_suspend` … `mb_resume`, or a paused preview) every voice pauses; nothing plays off screen. On resume, voices started with `mb_play_at` (playing or still waiting to start) are rescheduled from game time, which also paused: each comes back exactly where game time says it should be, or is dropped if it's a one-shot that would be more than half over. Voices started with `mb_play` continue from where they paused. The same happens when the audio output is interrupted while the game runs (a call, another app): `mb_play_at` voices come back on game time when it returns. If the platform has to rebuild its audio output, `mb_play` loops restart from the beginning and `mb_play` one-shots are dropped. A game MAY still stop its voices in `mb_suspend` and start fresh on `mb_resume`.

**Rhythm games.** A recipe that keeps music, picture and judging on one clock:
1. **Derive the beat grid from game time.** Pick beat 0 a little in the future, e.g. `origin = mb_time() + 0.12`, so the music can start from its first sample; notes are at `origin + step × seconds_per_step`. Draw markers from the same grid.
2. **Schedule the music with `mb_play_at(…, at = origin)`**, every stem with the same `at`. Make each music loop a whole number of phrases long, so the loop's wrap lands on the grid; later phrases need no rescheduling.
3. **Judge input with the event's own `time`** (§5.2), not `mb_time()`: `offset = event.time − note_time − calibration`.
4. **Re-anchor when the clock breaks:** when `mb_time_lost()` > 0 the music is ahead of the grid, and after `mb_resume` the player needs a run-up. At the next phrase boundary (or at once, if you prefer), stop the music and schedule it again at a fresh `origin = mb_time() + 0.12`, and move the grid there. Both are recorded, so this replays.
5. **Keep a small self-calibration** for what the platform can't know: the touch screen's latency, about a frame of presentation, and the player's habit (people tend to tap slightly early on a beat they can hear). Take the median offset of the first few judged presses, ignore wild ones (±0.2 s), clamp the result to tens of milliseconds (e.g. −60…+100 ms), save it for the next run (stored data is recorded) and keep the windows generous until it settles. Output latency is already compensated, so a large learned offset means something else is wrong.

Measured latencies (what the host compensates, logged at startup as `runtime: audio clock …`, §9): desktop Chrome on a Mac, built-in output: `baseLatency` 5 ms, `outputLatency` 32 ms, via `getOutputTimestamp`. iPhone 16 Pro (iOS WebKit), built-in speaker: `baseLatency` 2.7 ms, `outputLatency` 8.5–15.4 ms, `getOutputTimestamp` available; music on the grid and Perfects landing where expected by ear. Its `getOutputTimestamp` can read a few ms ahead of `currentTime` once settled ("heard −2.7 ms after currentTime"): harmless at that size. Wired and Bluetooth figures are still to be measured; if WebKit doesn't report Bluetooth latency, players on AirPods will hear music ~150–250 ms late.

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

- `mb new <dir>` creates a game crate from the template (`--3d` for the mb3d one). Signed in (`mb login`), the id and creator come from your account; signed out, from `--id` and `--creator`, or else placeholders that build, validate and preview but must be replaced before `mb publish`. `mb build games/<name>` compiles the crate for `wasm32-unknown-unknown`, optimizes it, packs `feed/<id>-<version>.mbx` and validates it. `mb validate` checks an existing bundle; `mb <command> --help` documents each command.
- `mb serve <game dir>` builds the game and serves it with the runtime at http://127.0.0.1:8765/runtime/index.html for a desktop browser with WebGPU (Safari or Chrome); reloading the page rebuilds it if any file changed. (In this repo, `just serve <name>` does the same.) There is no feed around it: touch arrives as mouse clicks, tilt is emulated by the mouse pointer (§5.7), haptics and leaderboards are logged to the console, and the insets are zero except a small bottom one. Leave it running and reload to iterate. A release build of `mb` has the web runtime built in. Built from the Maimbrain repo, `mb` embeds `runtime/dist`, which `just runtime` builds: run that once (and again after runtime changes), then run `mb` through cargo (`cargo run -p mb-cli -- serve …`), which picks up the new runtime automatically.
- Preview aids:
  - `?overlays` in the URL shades where the feed draws over a card, plus the pause pill (and mic badge).
  - `?speed=0.25` runs live time at a quarter speed.
  - Console calls:
    - `mb.suspend()` pauses, `mb.step(n)` advances a paused session n frames (a live one by 1/60 s each, a replay by its next n recorded frames) and resolves to `{ frames, time, hash, replay }`, `mb.resume()` continues, and `mb.speed(k)` changes the speed.
    - `mb.tap(x, y)` taps at page (CSS) pixels; the game gets it in the next frame.
    - `mb.sensors(gx, gy, gz, ax, ay, az)` sets tilt and motion until the pointer next moves. Synthetic input works too, e.g. `window.dispatchEvent(new PointerEvent("pointermove", { clientX, clientY }))` in a timed loop to script a shake.
    - `mb.loudness(v)` pins the mic level; `mb.loudness(-1)` simulates a denied mic, and `mb.loudness()` unpins.
    - `mb.snapshot()` returns `{ frames, time, hash, replay, log }`: the frames this session has run (`mb_update` calls; 0 right after `mb_init`), game time, the session's draw hash (§3), whether it's a replay, and the replay log of those frames.
    - `mb.audio()` returns the audio clock (`baseLatency`, `outputLatency`, whether `getOutputTimestamp` is used, the frame period and presentation delay) and every voice with its `at`, the AudioContext time it's heard from (`when`), when its node starts and how far into the sound (`start`, `offset`). Stems scheduled with the same `at` have the same `when`; a late voice has `start` > `when` and an `offset` to match. This is the way to check music alignment in the preview, since audio isn't part of the draw hash. The console also logs `runtime: audio clock (first|settled): …` when the output starts, and `runtime: voice N started X ms late` / `skipped` for late `mb_play_at` voices.
    - `mb.restart(json)` replaces the session: `"{}"` starts a fresh one, `JSON.stringify({ replay: log })` plays a log back the way a replay card does (in real time, looping). Add `hold: true` to either to run `mb_init` and then no frames at all until `mb.step` or `mb.resume`.
    - `mb.verify()` pauses, replays the current session's log in a fresh held session and compares the draw hash after `mb_init` and after every frame. It resolves to `{ frames, live, replay, match, firstMismatch }` (the first frame that differs, or null) and leaves the replay loaded.
    - `mb.debugGpu(kind)` simulates GPU trouble, which a game never sees: drawing is presentation only, so the runtime drops the frame, logs why (`runtime: gpu: …`), brings up a new device and keeps the game running. `"lose"` loses the device, `"throw"` makes the next frame's `finish()` throw (as WebKit's does once its GPU process is gone), and `"gone"` does that and leaves no adapter to be had, so the page asks the app to reload it (`gpu-lost`). WebGPU validation errors are logged the same way.
  - Pausing, then pinning sensors and stepping, makes any moment reproducible for a screenshot. Before taking a screenshot of a stepped frame, give the page a moment to show it (`await new Promise(r => setTimeout(r, 100))`). For hard-to-reach states (level 7, a near-fail), add debug constants to your game (e.g. `const DEBUG_START_LEVEL: u32 = 0;`) and set them back before shipping.
  - A hidden or background tab (and some headless browsers) runs no animation frames, so a live session doesn't advance there on its own: use `hold: true` and `mb.step`.
- **Checking determinism.** A run and its replay must have equal draw hashes at equal frame counts. Play or script a run, then `await mb.verify()`; `match: false` means the game depends on something that isn't recorded (its own entropy or clock, hash-map iteration order, state changed in `mb_render`). To compare by hand, frame by frame:
  ```js
  await mb.restart('{"hold": true}');            // a live session at frame 0
  mb.tap(180, 500); await mb.step(1);             // script input, a frame at a time
  const live = await mb.step(299);                // { frames: 300, hash, … }
  const log = mb.snapshot().log;
  await mb.restart(JSON.stringify({ replay: log, hold: true }));
  const again = await mb.step(300);               // again.hash === live.hash
  ```
  Two held live sessions (same seed) also match after the same steps with the same input, and so do two replays of one log. Compare `snapshot()`s at the same `frames`, never at the same wall-clock moment.
- The SDK compiles on the host too, so `cargo test -p <game>` can test game logic natively. There every host import is a stub: calls without a result do nothing, and calls with one return zero, except where zero would mean something real: sensors report "not ready" (`sensors::tilt()` and `loudness()` are `None`), `store` reads find nothing, and mb3d's constructors (`mb3d_mesh`, `_texture`, `_material`, `_gltf`, `_model_spawn`, `_node`, `_emitter`, `_trail`) return handle 1, so building a scene succeeds (with every handle equal to 1). `gfx3d::project` returns zeros and "off screen" there; logic that needs projection should use `Camera::project`, which runs anywhere. Keep the simulation separate from drawing and input so tests can drive it directly.
- Assets are yours to make: PNG for images, Ogg Vorbis for sound. Generating them with scripts kept in the crate (e.g. `tools/`) keeps the game remixable.
