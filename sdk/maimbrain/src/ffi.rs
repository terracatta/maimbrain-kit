//! Raw host imports (SPEC §5). Signatures must match
//! `crates/mb-format/src/abi.rs`; only the functions a game actually calls end
//! up as imports in its wasm.
//!
//! Off wasm32 (e.g. `cargo test` on the host), every import is a no-op stub
//! that returns zero (or "absent" / "not ready" where zero would mean
//! success, and handle 1 for mb3d's constructors), so game logic can be
//! tested natively (SPEC §9).

macro_rules! imports {
    ($( pub fn $name:ident($($arg:ident: $ty:ty),* $(,)?) $(-> $ret:ty $(= $stub:expr)?)?; )*) => {
        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "mb")]
        unsafe extern "C" {
            $( pub fn $name($($arg: $ty),*) $(-> $ret)?; )*
        }

        $(
            #[cfg(not(target_arch = "wasm32"))]
            #[allow(unused_variables, unreachable_code, clippy::missing_safety_doc, clippy::too_many_arguments)]
            pub unsafe fn $name($($arg: $ty),*) $(-> $ret)? {
                $($( return $stub; )?)?
                Default::default()
            }
        )*
    };
}

imports! {
    // 5.1 sys
    pub fn mb_log(level: u32, ptr: *const u8, len: u32);
    pub fn mb_time() -> f64;
    pub fn mb_time_lost() -> f32;
    pub fn mb_rand_seed() -> u64;
    pub fn mb_daily_seed() -> u64;
    pub fn mb_round(state: u32);
    pub fn mb_player_id(ptr: *mut u8) -> i32;
    pub fn mb_locale(ptr: *mut u8, cap: u32) -> i32;
    pub fn mb_screen(out: *mut u8);
    pub fn mb_asset_load(path: *const u8, len: u32) -> u32;
    pub fn mb_asset_state(h: u32) -> i32;
    pub fn mb_asset_read(h: u32, ptr: *mut u8, cap: u32) -> i32;

    // 5.2 input
    pub fn mb_input_poll(buf: *mut u8, cap: u32) -> i32;

    // 5.3 mb2d
    pub fn mb2d_image(asset: u32) -> i32;
    pub fn mb2d_font(asset: u32) -> i32;
    pub fn mb2d_clear(rgba: u32);
    pub fn mb2d_push();
    pub fn mb2d_pop();
    pub fn mb2d_translate(x: f32, y: f32);
    pub fn mb2d_rotate(r: f32);
    pub fn mb2d_scale(sx: f32, sy: f32);
    pub fn mb2d_blend(mode: u32);
    pub fn mb2d_rect(x: f32, y: f32, w: f32, h: f32, rgba: u32);
    pub fn mb2d_rect_gradient(x: f32, y: f32, w: f32, h: f32, top: u32, bottom: u32);
    pub fn mb2d_circle(x: f32, y: f32, r: f32, rgba: u32);
    pub fn mb2d_line(x0: f32, y0: f32, x1: f32, y1: f32, w: f32, rgba: u32);
    pub fn mb2d_poly(pts: *const f32, n: u32, rgba: u32);
    pub fn mb2d_sprite(img: u32, sx: f32, sy: f32, sw: f32, sh: f32, dx: f32, dy: f32, dw: f32, dh: f32, tint: u32);
    pub fn mb2d_text(font: u32, size: f32, x: f32, y: f32, rgba: u32, ptr: *const u8, len: u32);
    pub fn mb2d_measure(font: u32, size: f32, ptr: *const u8, len: u32) -> f32;
    pub fn mb2d_rrect(x: f32, y: f32, w: f32, h: f32, radius: f32, stroke: f32, feather: f32, top: u32, bottom: u32);
    pub fn mb2d_text_style(weight: f32, outline: f32, outline_rgba: u32, soft: f32);
    pub fn mb2d_antialias(on: u32);

    // 5.4 mb3d
    pub fn mb3d_mesh(ptr: *const u8, len: u32) -> i32 = 1;
    pub fn mb3d_texture(asset: u32) -> i32 = 1;
    pub fn mb3d_material(ptr: *const u8) -> i32 = 1;
    pub fn mb3d_material_set(handle: u32, ptr: *const u8) -> i32;
    pub fn mb3d_gltf(asset: u32) -> i32 = 1;
    pub fn mb3d_model_spawn(model: u32, parent: u32) -> i32 = 1;
    pub fn mb3d_node() -> i32 = 1;
    pub fn mb3d_node_parent(node: u32, parent: u32);
    pub fn mb3d_node_transform(node: u32, ptr: *const u8);
    pub fn mb3d_node_mesh(node: u32, mesh: u32, material: u32);
    pub fn mb3d_node_visible(node: u32, visible: u32);
    pub fn mb3d_node_destroy(node: u32);
    pub fn mb3d_camera(ptr: *const u8);
    pub fn mb3d_project(x: f32, y: f32, z: f32, out: *mut u8) -> i32;
    pub fn mb3d_sun(ptr: *const u8);
    pub fn mb3d_light_point(node: u32, r: f32, g: f32, b: f32, intensity: f32, range: f32) -> i32;
    pub fn mb3d_sky(ptr: *const u8);
    pub fn mb3d_post(ptr: *const u8);
    pub fn mb3d_emitter(ptr: *const u8) -> i32 = 1;
    pub fn mb3d_emit(emitter: u32, x: f32, y: f32, z: f32, dx: f32, dy: f32, dz: f32, count: u32);
    pub fn mb3d_emit_moving(emitter: u32, x: f32, y: f32, z: f32, dx: f32, dy: f32, dz: f32, count: u32, vx: f32, vy: f32, vz: f32);
    pub fn mb3d_trail(ptr: *const u8) -> i32 = 1;
    pub fn mb3d_trail_attach(trail: u32, node: u32);
    pub fn mb3d_trail_detach(trail: u32);
    pub fn mb3d_shockwave(x: f32, y: f32, z: f32, radius: f32, strength: f32, seconds: f32);
    pub fn mb3d_render();
    // 5.4 mb3d 2
    pub fn mb3d_free(kind: u32, handle: u32) -> i32;
    pub fn mb3d_instances(node: u32, ptr: *const u8, count: u32) -> i32;
    pub fn mb3d_clip_count(model: u32) -> i32;
    pub fn mb3d_clip_find(model: u32, name: *const u8, len: u32) -> i32;
    pub fn mb3d_clip_duration(model: u32, clip: u32) -> f32 = 1.0;
    pub fn mb3d_anim(node: u32, clip: u32, time: f32, weight: f32) -> i32;
    pub fn mb3d_node_find(node: u32, name: *const u8, len: u32) -> i32 = 1;
    pub fn mb3d_node_morph(node: u32, w0: f32, w1: f32, w2: f32, w3: f32);
    pub fn mb3d_node_world(node: u32, out: *mut u8) -> i32;
    pub fn mb3d_node_material(node: u32) -> i32 = 1;
    pub fn mb3d_fog(ptr: *const u8);
    pub fn mb3d_material_style(material: u32, ptr: *const u8) -> i32;
    pub fn mb3d_text(node: u32, desc: *const u8, text: *const u8, len: u32) -> i32;
    pub fn mb3d_dof(ptr: *const u8);

    // 5.6 audio
    pub fn mb_sound(asset: u32) -> i32;
    pub fn mb_play(sound: u32, vol: f32, pan: f32, pitch: f32, looped: u32) -> u32;
    pub fn mb_play_at(sound: u32, vol: f32, pan: f32, pitch: f32, looped: u32, at: f64) -> u32;
    pub fn mb_voice_set(voice: u32, vol: f32, pan: f32, pitch: f32);
    pub fn mb_voice_stop(voice: u32);

    // 5.7 sensors
    pub fn mb_tilt(out: *mut u8) -> i32 = -6;
    pub fn mb_motion(out: *mut u8) -> i32 = -6;
    pub fn mb_loudness() -> f32 = -1.0;
    pub fn mb_haptic(kind: u32);

    // 5.8 store / score
    pub fn mb_store_get(key: *const u8, key_len: u32, buf: *mut u8, cap: u32) -> i32 = -1;
    pub fn mb_store_set(key: *const u8, key_len: u32, val: *const u8, val_len: u32) -> i32;
    pub fn mb_score_submit(board: u32, value: i64) -> i32;
    pub fn mb_score_show(board: u32);

    // 5.9 links
    pub fn mb_link_open(ptr: *const u8, len: u32) -> i32;
}
