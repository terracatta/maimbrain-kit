//! The host 3D engine, `mb3d` (SPEC §5.4). Declare `stdlib = { mb3d = 1 }`
//! in manifest.toml (add `mb2d = 1` too to draw a HUD on top).
//!
//! A retained scene: create meshes, materials and nodes once, move nodes
//! and the camera every frame, and call [`render`] from `Game::render`. mb2d
//! calls in the same frame draw over the 3D image. Units are meters;
//! right-handed, +y up; colors are linear RGB (use [`srgb`] to convert a
//! picked color) and may exceed 1 for things that should glow and bloom.
//!
//! ```ignore
//! let mesh = MeshData::icosphere(1.0, 1).flat_shaded().upload().unwrap();
//! let mat = Material::new(&MaterialDesc { base_color: srgba(0x4cc9f0ff), roughness: 0.4, ..Default::default() }).unwrap();
//! let ship = Node::new().unwrap();
//! ship.set_mesh(mesh, mat);
//! // every frame:
//! ship.set_transform(&Transform::at(vec3(0.0, 1.0, 0.0)).with_rot(Quat::from_rotation_y(t)));
//! gfx3d::camera(&Camera::looking_at(vec3(0.0, 2.0, 6.0), Vec3::ZERO));
//! gfx3d::render();
//! ```

mod math;
mod mesh;

use std::num::NonZeroU32;

pub use math::{Mat4, Quat, Transform, Vec3, look_at, srgb, srgba, vec3};
pub use mesh::MeshData;

use crate::ffi;
use crate::sys::Asset;

fn handle(h: i32) -> Option<NonZeroU32> {
    if h > 0 { NonZeroU32::new(h as u32) } else { None }
}

fn ptr<T>(v: &T) -> *const u8 {
    (v as *const T).cast()
}

/// A mesh uploaded to the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Mesh(NonZeroU32);

impl Mesh {
    /// Uploads mesh data. None if it's malformed or over the limits (512 meshes, 1M vertices).
    pub fn new(data: &MeshData) -> Option<Mesh> {
        let b = data.pack();
        handle(unsafe { ffi::mb3d_mesh(b.as_ptr(), b.len() as u32) }).map(Mesh)
    }
    pub fn raw(self) -> u32 {
        self.0.get()
    }
}

impl MeshData {
    /// Shorthand for [`Mesh::new`].
    pub fn upload(&self) -> Option<Mesh> {
        Mesh::new(self)
    }
}

/// A texture decoded from a ready PNG or JPEG asset (≤ 2048², ≤ 64 per game).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Texture(NonZeroU32);

impl Texture {
    /// None until the asset is ready, or if it isn't a valid PNG/JPEG.
    pub fn new(asset: Asset) -> Option<Texture> {
        handle(unsafe { ffi::mb3d_texture(asset.0) }).map(Texture)
    }
}

/// `Material` (64 bytes, SPEC §5.4): metallic/roughness PBR. Colors are linear.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct MaterialDesc {
    /// Linear RGB and opacity (opacity only matters with [`Material::ALPHA`]).
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
    /// Multiplies `emissive`; values above ~1 bloom.
    pub emissive_strength: f32,
    pub metallic: f32,
    pub roughness: f32,
    /// sRGB color texture, multiplied with `base_color`.
    pub base_tex: Option<Texture>,
    /// Tangent-space normal map (linear).
    pub normal_tex: Option<Texture>,
    /// glTF layout: roughness in green, metallic in blue (linear); multiplies the factors.
    pub metal_rough_tex: Option<Texture>,
    /// sRGB emissive texture, multiplied with `emissive`.
    pub emissive_tex: Option<Texture>,
    /// [`Material::UNLIT`], [`Material::DOUBLE_SIDED`], [`Material::ALPHA`], [`Material::ADDITIVE`].
    pub flags: u32,
    pub _reserved: u32,
}

impl Default for MaterialDesc {
    fn default() -> MaterialDesc {
        MaterialDesc {
            base_color: [1.0, 1.0, 1.0, 1.0],
            emissive: [0.0; 3],
            emissive_strength: 1.0,
            metallic: 0.0,
            roughness: 0.5,
            base_tex: None,
            normal_tex: None,
            metal_rough_tex: None,
            emissive_tex: None,
            flags: 0,
            _reserved: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Material(NonZeroU32);

impl Material {
    /// Ignores lights: base color + emissive, as is.
    pub const UNLIT: u32 = 1;
    /// Draws back faces too (lit as seen from that side).
    pub const DOUBLE_SIDED: u32 = 2;
    /// Blends by `base_color` alpha; drawn after opaque things, back to front.
    pub const ALPHA: u32 = 4;
    /// Adds its light onto what's behind (glows, beams).
    pub const ADDITIVE: u32 = 8;

    pub fn new(desc: &MaterialDesc) -> Option<Material> {
        handle(unsafe { ffi::mb3d_material(ptr(desc)) }).map(Material)
    }
    /// Changes the material (e.g. pulse its emissive) for every node using it.
    pub fn set(self, desc: &MaterialDesc) {
        unsafe { ffi::mb3d_material_set(self.0.get(), ptr(desc)) };
    }
}

/// A scene node: a transform in a hierarchy, optionally drawing one mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Node(NonZeroU32);

impl Node {
    /// A new node at the root with an identity transform. None past 4096 nodes.
    pub fn new() -> Option<Node> {
        handle(unsafe { ffi::mb3d_node() }).map(Node)
    }
    /// A node already drawing `mesh` with `material`.
    pub fn with_mesh(mesh: Mesh, material: Material) -> Option<Node> {
        let n = Node::new()?;
        n.set_mesh(mesh, material);
        Some(n)
    }
    pub fn raw(self) -> u32 {
        self.0.get()
    }
    /// Re-parents the node (None = the root). Its transform becomes relative to the parent.
    pub fn set_parent(self, parent: Option<Node>) {
        unsafe { ffi::mb3d_node_parent(self.0.get(), parent.map_or(0, |p| p.0.get())) }
    }
    pub fn set_transform(self, t: &Transform) {
        unsafe { ffi::mb3d_node_transform(self.0.get(), ptr(t)) }
    }
    pub fn set_mesh(self, mesh: Mesh, material: Material) {
        unsafe { ffi::mb3d_node_mesh(self.0.get(), mesh.0.get(), material.0.get()) }
    }
    pub fn clear_mesh(self) {
        unsafe { ffi::mb3d_node_mesh(self.0.get(), 0, 0) }
    }
    /// Hides or shows the node and everything under it.
    pub fn set_visible(self, visible: bool) {
        unsafe { ffi::mb3d_node_visible(self.0.get(), visible as u32) }
    }
    /// Destroys the node and its children. Their lights go out; attached trails fade.
    pub fn destroy(self) {
        unsafe { ffi::mb3d_node_destroy(self.0.get()) }
    }
    /// Attaches (or updates) a point light at this node. False past 8 lights.
    /// `color` is linear; light falls off with distance and reaches 0 at `range`.
    pub fn point_light(self, color: [f32; 3], intensity: f32, range: f32) -> bool {
        unsafe { ffi::mb3d_light_point(self.0.get(), color[0], color[1], color[2], intensity, range) >= 0 }
    }
}

/// A glTF model (.glb) loaded from an asset: meshes, materials, textures, hierarchy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Model(NonZeroU32);

impl Model {
    /// None until the asset is ready, or if it's not a supported .glb.
    pub fn load(asset: Asset) -> Option<Model> {
        handle(unsafe { ffi::mb3d_gltf(asset.0) }).map(Model)
    }
    /// Instantiates the model under `parent`; returns its root node (move that).
    pub fn spawn(self, parent: Option<Node>) -> Option<Node> {
        handle(unsafe { ffi::mb3d_model_spawn(self.0.get(), parent.map_or(0, |p| p.0.get())) }).map(Node)
    }
}

/// The camera (40 bytes). It looks down its local −z.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Camera {
    pub pos: Vec3,
    pub rot: Quat,
    /// Vertical field of view in radians.
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera { pos: vec3(0.0, 0.0, 5.0), rot: Quat::IDENTITY, fov_y: 1.0, near: 0.1, far: 500.0 }
    }
}

impl Camera {
    pub fn looking_at(pos: Vec3, target: Vec3) -> Camera {
        Camera { pos, rot: look_at(pos, target, Vec3::Y), ..Default::default() }
    }

    /// Where world point `p` lands on a `screen_w` × `screen_h` logical
    /// screen (your `logical_size`) seen through this camera: the formula
    /// [`project`] (`mb3d_project`) uses on the host (SPEC §5.4), computed
    /// here in plain Rust. Use it for hit tests and lock-on logic in your
    /// simulation, where `cargo test` can run it; results agree with the
    /// host's to within float rounding, so compare with a tolerance.
    pub fn project(&self, p: Vec3, screen_w: f32, screen_h: f32) -> Projected {
        // The host's clamps on mb3d_camera.
        let rot = self.rot.normalize();
        let fov_y = self.fov_y.clamp(0.01, 3.0);
        let near = self.near.max(1e-4);
        let far = self.far.max(near * 1.001);
        let d = p - self.pos;
        let depth = -d.dot(rot * Vec3::Z);
        if depth <= 1e-6 {
            return Projected { x: 0.0, y: 0.0, depth, on_screen: false };
        }
        let f = 1.0 / (fov_y * 0.5).tan();
        let nx = f / (screen_w / screen_h) * d.dot(rot * Vec3::X) / depth;
        let ny = f * d.dot(rot * Vec3::Y) / depth;
        Projected {
            x: (nx * 0.5 + 0.5) * screen_w,
            y: (0.5 - ny * 0.5) * screen_h,
            depth,
            on_screen: nx.abs() <= 1.0 && ny.abs() <= 1.0 && depth >= near && depth <= far,
        }
    }
}

pub fn camera(c: &Camera) {
    unsafe { ffi::mb3d_camera(ptr(c)) }
}

/// Where a world point lands on screen, for 2D HUD over 3D.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projected {
    /// Logical screen coordinates (mb2d's space).
    pub x: f32,
    pub y: f32,
    /// Distance in front of the camera (≤ 0: behind it).
    pub depth: f32,
    pub on_screen: bool,
}

/// Where a world point lands on screen through the camera last passed to
/// [`camera`] (`mb3d_project`, SPEC §5.4): call it after `camera(..)` in the
/// same frame. Off wasm (in `cargo test`) it returns all zeros and
/// `on_screen: false`; game logic that needs projection should use
/// [`Camera::project`], which computes the same thing in plain Rust.
pub fn project(p: Vec3) -> Projected {
    let mut out = [0f32; 3];
    let r = unsafe { ffi::mb3d_project(p.x, p.y, p.z, out.as_mut_ptr().cast()) };
    Projected { x: out[0], y: out[1], depth: out[2], on_screen: r == 1 }
}

/// The sun (36 bytes): a directional light, optionally casting shadows.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Sun {
    /// The direction the light travels, e.g. `(0, -1, 0)` shines straight down.
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    /// 1 to cast shadows (a 2048² map around the camera).
    pub shadows: u32,
    /// How far from the camera shadows reach, in meters.
    pub shadow_distance: f32,
}

impl Default for Sun {
    fn default() -> Sun {
        Sun { direction: vec3(-0.4, -0.8, -0.45), color: [1.0, 0.96, 0.9], intensity: 3.0, shadows: 0, shadow_distance: 30.0 }
    }
}

pub fn sun(s: &Sun) {
    unsafe { ffi::mb3d_sun(ptr(s)) }
}

/// The procedural sky (64 bytes): a starfield and nebula, also the source of
/// image-based lighting. Changing it re-bakes the lighting, so set it once
/// (or rarely), not every frame.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Sky {
    pub seed: u32,
    /// Linear RGB: the background, then two nebula colors. Brighter colors light the scene more.
    pub colors: [[f32; 3]; 3],
    /// 0 (none) – 1 (lots); up to 4.
    pub nebula_density: f32,
    pub star_density: f32,
    pub star_brightness: f32,
    /// Angular radius of a visible sun disc in the sun's direction, radians (0 = none).
    pub sun_disc: f32,
    pub _reserved: [u32; 2],
}

impl Default for Sky {
    fn default() -> Sky {
        Sky {
            seed: 1,
            colors: [[0.004, 0.005, 0.012], [0.05, 0.06, 0.16], [0.18, 0.05, 0.12]],
            nebula_density: 0.5,
            star_density: 0.5,
            star_brightness: 1.0,
            sun_disc: 0.0,
            _reserved: [0; 2],
        }
    }
}

pub fn sky(s: &Sky) {
    unsafe { ffi::mb3d_sky(ptr(s)) }
}

/// Post-processing (32 bytes).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Post {
    pub exposure: f32,
    pub bloom_strength: f32,
    /// Brightness above which things bloom (1 = full white before exposure).
    pub bloom_threshold: f32,
    /// Darkening toward the corners, 0–1.
    pub vignette: f32,
    /// Color fringing toward the edges, 0–1.
    pub chromatic_aberration: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub _reserved: f32,
}

impl Default for Post {
    fn default() -> Post {
        Post { exposure: 1.0, bloom_strength: 0.5, bloom_threshold: 1.0, vignette: 0.25, chromatic_aberration: 0.0, saturation: 1.0, contrast: 1.0, _reserved: 0.0 }
    }
}

pub fn post(p: &Post) {
    unsafe { ffi::mb3d_post(ptr(p)) }
}

/// How effects blend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum FxBlend {
    /// Light adds up (sparks, fire, glows).
    Additive = 0,
    /// Covers what's behind (smoke, debris).
    Alpha = 1,
}

/// A particle look and physics (128 bytes).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct EmitterDesc {
    /// Linear RGBA at life 0, 1/3, 2/3 and 1 (rgb above 1 blooms).
    pub colors: [[f32; 4]; 4],
    /// Diameter in meters at the same stops.
    pub sizes: [f32; 4],
    /// Seconds, picked per particle in [min, max].
    pub lifetime: [f32; 2],
    /// Meters per second, picked per particle.
    pub speed: [f32; 2],
    /// Half-angle of the emission cone around the burst direction, radians (π = all around).
    pub spread: f32,
    /// Velocity damping per second.
    pub drag: f32,
    pub blend: FxBlend,
    /// None = a soft round dot.
    pub texture: Option<Texture>,
    pub gravity: Vec3,
    /// Stretches particles along their velocity: length += stretch × speed (sparks).
    pub stretch: f32,
}

impl Default for EmitterDesc {
    fn default() -> EmitterDesc {
        EmitterDesc {
            colors: [[4.0, 2.0, 0.8, 1.0], [2.0, 0.6, 0.2, 1.0], [0.6, 0.1, 0.05, 0.6], [0.0, 0.0, 0.0, 0.0]],
            sizes: [0.15, 0.12, 0.08, 0.0],
            lifetime: [0.4, 0.9],
            speed: [2.0, 6.0],
            spread: std::f32::consts::PI,
            drag: 1.5,
            blend: FxBlend::Additive,
            texture: None,
            gravity: vec3(0.0, -2.0, 0.0),
            stretch: 0.0,
        }
    }
}

/// A particle emitter. Particles are simulated by the host, deterministically
/// (seeded by the emitter and how many bursts it has made), so replays match.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Emitter(NonZeroU32);

impl Emitter {
    pub fn new(desc: &EmitterDesc) -> Option<Emitter> {
        handle(unsafe { ffi::mb3d_emitter(ptr(desc)) }).map(Emitter)
    }
    /// A burst of `count` particles at `pos`, in the cone around `dir` (zero = all around). 16k live at most.
    pub fn emit(self, pos: Vec3, dir: Vec3, count: u32) {
        unsafe { ffi::mb3d_emit(self.0.get(), pos.x, pos.y, pos.z, dir.x, dir.y, dir.z, count) }
    }
    /// Like [`emit`](Self::emit), in a frame moving at `velocity` (m/s): the
    /// whole burst keeps drifting with it, while the emitter's speed, gravity
    /// and drag act relative to it. Pass the velocity of the thing that
    /// exploded, so a burst on something flying alongside the camera stays
    /// with it instead of streaming away.
    pub fn emit_moving(self, pos: Vec3, dir: Vec3, count: u32, velocity: Vec3) {
        unsafe { ffi::mb3d_emit_moving(self.0.get(), pos.x, pos.y, pos.z, dir.x, dir.y, dir.z, count, velocity.x, velocity.y, velocity.z) }
    }
}

/// A ribbon trail look (64 bytes).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct TrailDesc {
    /// Linear RGBA at the head and at the end of its life.
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub width_start: f32,
    pub width_end: f32,
    /// Seconds a point lives.
    pub lifetime: f32,
    pub blend: FxBlend,
    /// Minimum distance between recorded points, meters (0 = 5 cm).
    pub min_segment: f32,
    pub _reserved: [u32; 3],
}

impl Default for TrailDesc {
    fn default() -> TrailDesc {
        TrailDesc {
            color_start: [2.0, 3.0, 6.0, 1.0],
            color_end: [0.2, 0.3, 1.0, 0.0],
            width_start: 0.3,
            width_end: 0.0,
            lifetime: 0.6,
            blend: FxBlend::Additive,
            min_segment: 0.0,
            _reserved: [0; 3],
        }
    }
}

/// A camera-facing ribbon that follows a node (64 trails at most).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Trail(NonZeroU32);

impl Trail {
    pub fn new(desc: &TrailDesc) -> Option<Trail> {
        handle(unsafe { ffi::mb3d_trail(ptr(desc)) }).map(Trail)
    }
    pub fn attach(self, node: Node) {
        unsafe { ffi::mb3d_trail_attach(self.0.get(), node.0.get()) }
    }
    /// Stops following; what's there fades out over the trail's lifetime.
    pub fn detach(self) {
        unsafe { ffi::mb3d_trail_detach(self.0.get()) }
    }
}

/// A screen-space distortion ring expanding from a world point to `radius`
/// meters over `seconds`. `strength` ≈ 0.01–0.05 (fraction of the screen height).
pub fn shockwave(pos: Vec3, radius: f32, strength: f32, seconds: f32) {
    unsafe { ffi::mb3d_shockwave(pos.x, pos.y, pos.z, radius, strength, seconds) }
}

/// Draws the scene this frame. Call it in `Game::render`; mb2d calls in the
/// same frame draw on top.
pub fn render() {
    unsafe { ffi::mb3d_render() }
}

/// A camera orbiting a target: yaw around +y, pitch up from the horizon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_y: f32,
}

impl OrbitCamera {
    pub fn camera(&self) -> Camera {
        let dir = vec3(self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), self.yaw.cos() * self.pitch.cos());
        let pos = self.target + dir * self.distance;
        Camera { pos, rot: look_at(pos, self.target, Vec3::Y), fov_y: self.fov_y, ..Default::default() }
    }
}

/// A camera that eases toward a moving point while looking at another.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowCamera {
    pub pos: Vec3,
    pub look: Vec3,
    /// How quickly it catches up (per second); ~3–8 feels good.
    pub stiffness: f32,
    pub fov_y: f32,
}

impl FollowCamera {
    pub fn update(&mut self, want_pos: Vec3, want_look: Vec3, dt: f32) -> Camera {
        let k = 1.0 - (-self.stiffness * dt).exp();
        self.pos = self.pos.lerp(want_pos, k);
        self.look = self.look.lerp(want_look, k);
        Camera { pos: self.pos, rot: look_at(self.pos, self.look, Vec3::Y), fov_y: self.fov_y, ..Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_struct_sizes_match_the_spec() {
        assert_eq!(size_of::<Transform>(), 40);
        assert_eq!(size_of::<Camera>(), 40);
        assert_eq!(size_of::<MaterialDesc>(), 64);
        assert_eq!(size_of::<Sun>(), 36);
        assert_eq!(size_of::<Sky>(), 64);
        assert_eq!(size_of::<Post>(), 32);
        assert_eq!(size_of::<EmitterDesc>(), 128);
        assert_eq!(size_of::<TrailDesc>(), 64);
        assert_eq!(size_of::<Option<Texture>>(), 4);
    }

    /// The explosion example in the maimbrain-game skill's 3D section compiles and runs off-wasm.
    #[test]
    fn skill_explosion_example() {
        let at = vec3(1.0, 2.0, 3.0);
        let fire = Emitter::new(&EmitterDesc { colors: [[4.5, 2.6, 1.0, 1.0], [2.6, 0.8, 0.2, 0.75], [0.7, 0.14, 0.05, 0.35], [0.0; 4]],
            sizes: [0.5, 1.3, 1.7, 1.9], lifetime: [0.35, 0.7], speed: [0.4, 2.6], drag: 2.5, gravity: vec3(0.0, 0.6, 0.0), ..Default::default() }).unwrap();
        let sparks = Emitter::new(&EmitterDesc { colors: [[9.0, 6.0, 2.5, 1.0], [6.0, 2.2, 0.5, 1.0], [2.5, 0.5, 0.1, 0.8], [0.4, 0.05, 0.0, 0.0]],
            sizes: [0.07, 0.06, 0.045, 0.02], lifetime: [0.5, 1.2], speed: [4.0, 11.0], drag: 1.6, gravity: vec3(0.0, -3.5, 0.0), stretch: 0.045, ..Default::default() }).unwrap();
        let smoke = Emitter::new(&EmitterDesc { colors: [[0.2, 0.12, 0.08, 0.0], [0.13, 0.09, 0.09, 0.5], [0.07, 0.06, 0.08, 0.3], [0.04, 0.04, 0.05, 0.0]],
            sizes: [0.6, 1.3, 1.9, 2.4], lifetime: [1.4, 2.4], speed: [0.3, 1.4], drag: 1.2, blend: FxBlend::Alpha, gravity: vec3(0.0, 0.35, 0.0), ..Default::default() }).unwrap();
        fire.emit(at, Vec3::ZERO, 36);
        sparks.emit(at, Vec3::ZERO, 170);
        smoke.emit(at, Vec3::ZERO, 26);
        sparks.emit_moving(at, Vec3::ZERO, 50, vec3(0.0, 0.0, -30.0));
        shockwave(at, 4.5, 0.012, 0.9);
    }

    #[test]
    fn camera_project_follows_the_spec_formula() {
        // The default camera at z = 5 looking down -z, on a 360x640 screen.
        let c = Camera::default();
        let p = c.project(Vec3::ZERO, 360.0, 640.0);
        assert!(p.on_screen && (p.x - 180.0).abs() < 1e-3 && (p.y - 320.0).abs() < 1e-3 && (p.depth - 5.0).abs() < 1e-5);
        // +y is up on screen; the top edge is at tan(fov/2) * depth.
        let top = c.project(vec3(0.0, (0.5f32).tan() * 5.0, 0.0), 360.0, 640.0);
        assert!(top.y.abs() < 1e-3, "{top:?}");
        // x is scaled by the aspect ratio: the right edge is aspect times as far.
        let right = c.project(vec3((0.5f32).tan() * 5.0 * 360.0 / 640.0, 0.0, 0.0), 360.0, 640.0);
        assert!((right.x - 360.0).abs() < 1e-3, "{right:?}");
        assert!(!c.project(vec3(0.0, 0.0, 6.0), 360.0, 640.0).on_screen, "behind the camera");
        assert!(!c.project(vec3(0.0, 0.0, -600.0), 360.0, 640.0).on_screen, "past far");
        let looking = Camera::looking_at(vec3(3.0, 4.0, 5.0), vec3(1.0, 0.0, -2.0));
        let q = looking.project(vec3(1.0, 0.0, -2.0), 360.0, 640.0);
        assert!((q.x - 180.0).abs() < 1e-2 && (q.y - 320.0).abs() < 1e-2, "{q:?}");
    }

    #[test]
    fn orbit_camera_looks_at_its_target() {
        let c = OrbitCamera { target: vec3(1.0, 0.0, 0.0), distance: 5.0, yaw: 0.3, pitch: 0.4, fov_y: 1.0 }.camera();
        assert!((c.pos.distance(vec3(1.0, 0.0, 0.0)) - 5.0).abs() < 1e-4);
        assert!((c.rot.forward() - (vec3(1.0, 0.0, 0.0) - c.pos).normalize()).length() < 1e-4);
    }
}
