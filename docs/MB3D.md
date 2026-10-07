# mb3d v1 — design (Phase 6)

The host 3D engine: a retained scene graph that games drive through `mb3d_*` calls, rendered by `mb-host` (Rust → wasm32, wgpu on WebGPU) in the same frame as `mb2d`. The flagship that drives it is **Starfall**: a portrait, touch-only, on-rails space shooter through a nebula canyon (sweep to lock on to up to 8 enemies, release for homing lasers; bloom, particles, trails, a crystal boss).

## Decisions

| Area | Decision |
|---|---|
| Look | **Full PBR from the start**: metallic/roughness, image-based lighting, a shadowed sun, emissive glow, HDR, bloom, filmic tonemapping |
| Models | **Built in code and glTF**: raw mesh upload (SDK builders for primitives, lathes, extrusions, flat-shaded low-poly) plus `.glb` loading in v1 (meshopt allowed; PNG/JPEG textures; KTX2 later) |
| Perf floor | **60 fps on iPhone 13 class (A15)**, `perf_tier = "full"`. Effects scale down (particle cap, bloom resolution, render scale) automatically on slower devices |
| Effects | Procedural nebula sky, host particles, ribbon trails, screen effects (shockwaves, chromatic aberration, vignette); camera shake is game-side |
| Determinism | Everything visible comes from guest calls. Particle simulation is host-side, but seeded and stepped only by guest calls and frame dt, so replays reproduce the draw hash exactly |
| Composition | `mb3d_render()` draws the scene; `mb2d` calls in the same `mb_render` draw on top (HUD). A game may use either stdlib alone |

## ABI (import namespace `mb`, stdlib `mb3d = 1`)

Handles are `u32` (0 invalid); fallible calls return `i32` (SPEC §7 codes). Packed structs are little-endian f32/u32, defined in the SDK and SPEC §5.4. Units: meters; right-handed; +y up.

### Resources
```
mb3d_mesh(ptr, len) -> i32                 # packed: header {vertex_count, index_count, flags} + vertices + u32 indices
                                           # vertex: pos f32x3, normal f32x3, uv f32x2 [, tangent f32x4 if flags&1] [, color rgba8 if flags&2]
mb3d_texture(asset) -> i32                 # ready PNG/JPEG asset; sRGB unless used as normal/metal-rough map
mb3d_material(ptr) -> i32                  # Material (below); mb3d_material_set(handle, ptr) updates it (e.g. pulse emissive)
mb3d_gltf(asset) -> i32                    # ready .glb → a model (meshes, materials, textures, node hierarchy)
mb3d_model_spawn(model, parent) -> i32     # instantiates a model's hierarchy under `parent` (0 = root); returns its root node
```
`Material` (64 bytes): `base_color f32x4`, `emissive f32x3`, `emissive_strength f32`, `metallic f32`, `roughness f32`, `base_tex u32`, `normal_tex u32`, `metal_rough_tex u32`, `emissive_tex u32`, `flags u32` (1 unlit, 2 double-sided, 4 alpha-blend, 8 additive), `reserved`.

### Scene graph
```
mb3d_node() -> i32
mb3d_node_parent(node, parent)             # 0 = root
mb3d_node_transform(node, ptr)             # pos f32x3, rotation quat f32x4 (x,y,z,w), scale f32x3
mb3d_node_mesh(node, mesh, material)       # a node draws at most one mesh
mb3d_node_visible(node, visible)
mb3d_node_destroy(node)                    # and its children
mb3d_camera(ptr)                           # pos f32x3, rotation quat f32x4, fov_y (rad), near, far
mb3d_project(x, y, z, out_ptr) -> i32      # world → logical screen xy (+ depth); for 2D HUD over 3D (lock-on reticles)
```

### Lighting and sky
```
mb3d_sun(ptr)                              # direction f32x3, color f32x3, intensity, shadows (0/1), shadow_distance
mb3d_light_point(node, r, g, b, intensity, range) -> i32   # attached to a node; ≤ 8 active
mb3d_sky(ptr)                              # procedural: seed, colors (3 × rgb), nebula density, star density, star brightness,
                                           # sun disc; also the source of image-based lighting (re-baked when it changes)
mb3d_post(ptr)                             # exposure, bloom strength + threshold, vignette, chromatic aberration, saturation, contrast
```

### Effects
```
mb3d_emitter(ptr) -> i32                   # particle look and physics: color over life (4 stops), size over life, lifetime range,
                                           # speed range, spread cone, gravity, drag, blend (additive/alpha), texture (0 = soft dot)
mb3d_emit(emitter, x, y, z, dx, dy, dz, count)   # a burst at a point along a direction
mb3d_emit_moving(emitter, x, y, z, dx, dy, dz, count, vx, vy, vz)  # the same, in a frame moving at v (added after v1 shipped)
mb3d_trail(ptr) -> i32                     # width over age, color over age, lifetime, blend
mb3d_trail_attach(trail, node)             # follows the node's world position; mb3d_trail_detach(trail) leaves it to fade
                                           # attaching to another node (or again after a detach) starts a new ribbon
mb3d_shockwave(x, y, z, radius, strength, seconds)  # screen-space distortion ring at a world point
mb3d_render()                              # draw the scene this frame (mb2d after it draws on top)
```

### Limits (v1)
- **Scene:** 4096 nodes, 512 meshes (1M vertices total), 1024 materials.
- **Textures:** 64 textures, up to 2048² each.
- **Lights:** 8 point lights.
- **Effects:** 16k live particles and 64 trails.
- **Per frame:** 2000 draws.

Going over a limit returns `-4`. GPU memory is capped at 256 MB for `full` games.

Only nodes are ever freed (`mb3d_node_destroy`, with their lights and descendants). Meshes, textures, materials, models, emitters and trails live for the whole session: games build them once and pool them (see "Patterns" below).

## Renderer (mb-host)

- **Targets:** HDR `Rgba16Float` color plus `Depth32Float`, at render scale × (≤ 2× logical). The render scale adapts from frame timings, between 0.6 and 1.0. MSAA 4× where it fits the budget.
- **Forward PBR:**
  - GGX specular, Smith-correlated visibility, Schlick fresnel, Lambert diffuse.
  - Up to 8 point lights plus the sun.
  - The sun casts a 2048² shadow map fitted to `shadow_distance` around the camera, with PCF filtering.
- **Image-based lighting:** the procedural sky is rendered into a 128² cubemap. From that:
  - prefiltered specular mip chain (GGX importance sampling)
  - irradiance (spherical harmonics)
  - a BRDF lookup table baked at init

  Everything is re-baked only when `mb3d_sky` changes.
- **Transparency:** opaque first (front-to-back), then the sky, then alpha and additive (back-to-front), then particles and trails.
- **Post:**
  - bloom (dual-filter down/up chain from a soft threshold)
  - shockwave and chromatic distortion
  - ACES-fitted (or AgX) tonemapping, exposure, contrast and saturation, vignette
  - then `mb2d` composites on top in display space
- **glTF:**
  - `gltf` crate (glb only) and `meshopt` decoding
  - PNG/JPEG textures via the `png` crate and a JPEG decoder
  - node transforms, PBR material mapping and emissive strength
  - skins and animations are ignored in v1

## SDK (`maimbrain::gfx3d`)

- **Math:** `Vec3`, `Quat` and `Mat4`, using `glam` if it stays small.
- **Handles:** `Mesh`, `Material`, `Node`, `Model`, `Emitter`, `Trail`, and setters for the camera, sun, sky and post.
- **Mesh builders:**
  - box, UV sphere, icosphere (smooth or faceted), cylinder, cone, torus, plane
  - lathe (a profile revolved), extrude (a 2D polygon along z)
  - `flat_shaded()` for low-poly facets, `with_vertex_colors`
  - merge, transform
- **Helpers:** an orbit/follow camera and `look_at`.

## Spec, validator, runtime

- **SPEC §5.4:** rewritten to this API.
- **mb-format:** ABI table entries, gated by `stdlib = { mb3d = 1 }`; `.glb` size checks.
- **runtime:** `abi.ts` marshals the memory-passing calls into mb-host's scratch buffer, as mb2d does.
- **Skill:** a "3D" section with performance budgets, how to make meshes in code, how to build a glTF with a script, and lighting recipes.

## Milestones (each ends with a demo scene, screenshots and tests)

1. **Core:**
   - meshes (raw plus SDK builders), materials (direct-light PBR), nodes, camera, sun without shadows
   - HDR target and tonemap, and composition with mb2d
   - a `games/mb3d-demo` scene: a grid of spheres sweeping metallic and roughness, plus a rotating faceted ship
2. **Sky and lighting:** the procedural nebula sky, image-based lighting from it, sun shadows, point lights.
3. **Post:** bloom, exposure, vignette, chromatic aberration, shockwaves.
4. **Effects:** particles and trails, deterministic (a replay of the demo matches its draw hash).
5. **glTF:** models, textures, normal and metal-rough maps; a script-generated `.glb` in the demo.
6. **Docs and device:**
   - SPEC §5.4, the skill's 3D section
   - frame timing on the iPhone 16 Pro, plus the A15 budget estimate
   - adaptive render scale

After these, a clean-room agent builds Starfall from the docs (like the 2D games), and we tune it on device.

## Implementation notes (v1, as built)

The API above is what shipped; SPEC §5.4 is the normative version with every struct layout. Where the build differs from this design, deliberately:

- **glTF without the `gltf` and `meshopt` crates.** mb-host reads `.glb` with its own small JSON reader and glTF walker, and decodes meshopt with a safe Rust port of meshoptimizer's reference decoder (tested against vectors from its encoder). The `gltf` crate pulls serde/serde_json, and the `meshopt` crate is a C build that doesn't fit the wasm32 toolchain. Besides `EXT_meshopt_compression`, the loader takes `KHR_meshopt_compression` (vertex codec v1, color filter), which current gltfpack can emit.
- **Small ABI decisions** the design left open: `mb3d_material_set` returns `i32` (0 or −1); `mb3d_light_point` returns 0 or a §7 code (lights are addressed by their node, so there's no light handle); `mb3d_project` returns 1 when the point is on screen and writes the view depth; the sun's `direction` is the way light travels; `Emitter` has a `gravity` vector and a velocity `stretch`, `Trail` a `min_segment`; `Sky` is 64 bytes with two reserved words.
- **Limits added**: 256 emitters, 64 models, and a 96 MB texture budget (64 textures at 2048² with mips would be 1.4 GB, far over the 256 MB cap).
- **Scaling on slow devices is resolution only.** The render scale adapts (0.6–1.0) from frame timings; the particle cap does not, because particle counts are simulated state and must replay identically on every device. MSAA is always 4×.
- **Determinism plumbing**: the runtime calls a new `host_step(dt)` before each `mb_update` so effects advance by the recorded `dt` even on frames that don't render; `host_frame_time(ms)` feeds the adaptive render scale. Every mb3d call, plus a digest of particles, trails and shockwaves at each `mb3d_render`, is folded into the draw hash.
- **Bakes without compute**: the nebula cube (every mip rendered directly), the GGX-prefiltered specular cube, the SH coefficients (a 9×1 render target the PBR shader reads) and the BRDF table are all render passes, so nothing needs compute or storage textures, and nothing is read back. A sky change is compared by value and re-baked over six frames (the sky draws stars only until its nebula is ready).
- **Tonemapping is ACES-fitted** (AgX not added). No fog in v1. glTF alpha `MASK` draws opaque.
- **Assets**: `.jpg`/`.jpeg` are now allowed in bundles (SPEC §1) for `mb3d_texture`; `.glb` assets are structure-checked by the validator and capped at 8 MB.

### Changes after Starfall (the clean-room build)

- **`mb3d_emit_moving`**: particles had no base velocity, so a burst on an enemy flying alongside the camera streamed away behind it. A burst can now carry its frame's velocity: each particle keeps it unchanged and moves by `vel + base`, with gravity and drag acting on `vel` only (so the cloud keeps pace and spreads as it would at rest). The emitter's burst counter is shared with `mb3d_emit`; `base` isn't in the particle digest because it only shows through positions, which are. ABI table, validator, SDK (`Emitter::emit_moving`), SPEC §5.4 and tests (`a_moving_burst_drifts_with_its_frame`).
- **Trail re-attach**: re-attaching used to clear the trail's points at once (a fading ribbon popped out of existence when a pooled trail moved to a new node). Now the old points stay and fade on their own clock, and the new ribbon starts at the node's next sample, with nothing drawn between them (the strip joins ribbons with degenerate triangles; the ribbon buffer is 3 × points per trail to fit the joins). Attaching to the node already followed is a no-op.
- **Projection** is specified exactly in SPEC §5.4, and the SDK's `Camera::project` computes it in plain Rust (tested against the host formula), so lock-on and hit tests can live in a game's tested simulation.
- **Draw hash per session**: `host_new_session` restarts the hash, and the runtime counts frames per session, so a run and its replay (or two runs) can be compared at equal frame counts; see SPEC §9 for the procedure (`mb.restart({hold})`, `mb.step`, `mb.verify`).

## Patterns

- **Explosions on things that move with the camera.** Particles are world-space. For an on-rails game where the camera (and the enemies around it) fly forward at the rail speed, fire bursts with `Emitter::emit_moving(pos, dir, count, velocity_of_the_thing)` so the cloud rides along. The alternative is a floating frame: keep the player and camera near the origin and move the world past them (scenery scrolls backward); then bursts on things flying with the camera need no velocity, and bursts on scenery need the scenery's (backward) velocity. Pick one frame for the whole game; mixing them is where streaks come from. Long-lived effects on curved paths will drift (the frame velocity is constant per burst), so keep those short.
- **Build once, pool.** Create meshes, materials, emitters and trails in `init` (or when an asset arrives) and keep the handles. One emitter per particle *look*, fired wherever needed; one material per look, changed with `Material::set`; enemies as a fixed pool of nodes, hidden with `set_visible(false)` when dead and moved when reused. Pooled trails move with `detach()`/`attach(node)`, which starts a fresh ribbon.
- **Lights.** A point light holds one of the 8 slots from its first `mb3d_light_point` until its node (or an ancestor) is destroyed. Intensity 0 darkens it but keeps the slot; hiding its node doesn't turn it off. For explosion flashes, keep one or two light nodes, move them to the latest blast and decay their intensity.

Still to do from the milestones: frame timing on an iPhone 16 Pro and the A15 estimate (needs the device), and checking MSAA on rgba16float and the frame budget there.
