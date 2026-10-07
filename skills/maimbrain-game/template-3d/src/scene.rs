//! The 3D scene, built once in `init`: sky (also the image-based lighting),
//! a shadow-casting sun, post, meshes made in code, one particle emitter,
//! one trail and two point lights. Everything here persists; the game only
//! moves nodes and changes light intensities afterwards (SPEC §5.4).

use maimbrain::gfx3d::{
    self, Emitter, EmitterDesc, FxBlend, Material, MaterialDesc, MeshData, Node, Post, Quat, Sky, Sun, Trail, TrailDesc, Transform, Vec3, srgb,
    srgba, vec3,
};

pub struct Scene {
    /// The drone's parent node: move this one.
    pub drone: Node,
    /// The glowing ring, a child of the drone (spun on its own).
    pub ring: Node,
    pub trail: Trail,
    pub sparks: Emitter,
    /// A point light at each catch that flashes and fades.
    pub flash: Node,
}

fn material(desc: MaterialDesc) -> Material {
    Material::new(&desc).expect("material")
}

pub fn build() -> Scene {
    // The sky is the background *and* the ambient light: set it once.
    gfx3d::sky(&Sky {
        seed: 3,
        colors: [[0.004, 0.006, 0.02], [0.06, 0.16, 0.42], [0.45, 0.08, 0.3]],
        nebula_density: 1.0,
        star_density: 0.8,
        star_brightness: 1.0,
        sun_disc: 0.0,
        ..Default::default()
    });
    gfx3d::sun(&Sun { direction: vec3(-0.5, -0.8, -0.35), color: [1.0, 0.92, 0.82], intensity: 3.0, shadows: 1, shadow_distance: 18.0 });
    gfx3d::post(&Post { exposure: 1.1, bloom_strength: 0.7, bloom_threshold: 1.0, vignette: 0.3, saturation: 1.05, ..Default::default() });

    // The platform: a beveled disc (a lathe), flat shaded.
    let disc = MeshData::lathe(&[[0.0, -0.4], [3.4, -0.4], [3.7, -0.2], [3.7, 0.0], [0.0, 0.0]], 40).flat_shaded();
    let deck = material(MaterialDesc { base_color: srgba(0x5a606cff), metallic: 0.2, roughness: 0.5, ..Default::default() });
    Node::with_mesh(disc.upload().expect("disc"), deck);

    // Six pillars around the rim, merged into one mesh: one draw instead of six.
    let mut pillars = MeshData::new();
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.0;
        let at = Transform::at(vec3(3.1 * a.cos(), 0.6, 3.1 * a.sin()));
        pillars.merge(&MeshData::cylinder(0.18, 1.2, 12).flat_shaded().transformed(&at.matrix()));
    }
    let stone = material(MaterialDesc { base_color: srgba(0x8a8f99ff), roughness: 0.7, ..Default::default() });
    Node::with_mesh(pillars.upload().expect("pillars"), stone);

    // The drone: a faceted metal body with a glowing ring around it.
    let drone = Node::new().expect("node");
    let body = material(MaterialDesc { base_color: srgba(0xd8dde6ff), metallic: 0.8, roughness: 0.3, ..Default::default() });
    let core = Node::with_mesh(MeshData::icosphere(0.32, 1).flat_shaded().upload().expect("body"), body).expect("node");
    core.set_parent(Some(drone));
    // Unlit with emissive above 1: it glows and blooms.
    let glow = material(MaterialDesc { base_color: [0.0, 0.0, 0.0, 1.0], emissive: srgb(0x4cc9f0), emissive_strength: 4.0, flags: Material::UNLIT, ..Default::default() });
    let ring = Node::with_mesh(MeshData::torus(0.55, 0.05, 40, 8).upload().expect("ring"), glow).expect("node");
    ring.set_parent(Some(drone));
    // The drone lights its surroundings (2 of the 8 point lights in use).
    let lamp = Node::new().expect("node");
    lamp.set_parent(Some(drone));
    lamp.point_light(srgb(0x4cc9f0), 4.0, 4.0);

    let trail = Trail::new(&TrailDesc {
        color_start: [0.6, 2.4, 4.0, 1.0],
        color_end: [0.3, 0.2, 1.5, 0.0],
        width_start: 0.16,
        width_end: 0.0,
        lifetime: 0.5,
        blend: FxBlend::Additive,
        ..Default::default()
    })
    .expect("trail");
    trail.attach(drone);

    let sparks = Emitter::new(&EmitterDesc {
        colors: [[2.0, 6.0, 9.0, 1.0], [0.8, 2.5, 6.0, 1.0], [0.4, 0.6, 3.0, 0.7], [0.1, 0.1, 0.6, 0.0]],
        sizes: [0.09, 0.07, 0.05, 0.0],
        lifetime: [0.35, 0.8],
        speed: [3.0, 8.0],
        spread: std::f32::consts::PI,
        drag: 2.0,
        gravity: vec3(0.0, -4.0, 0.0),
        stretch: 0.04,
        ..Default::default()
    })
    .expect("emitter");

    let flash = Node::new().expect("node");
    flash.point_light([0.0; 3], 0.0, 6.0);

    Scene { drone, ring, trail, sparks, flash }
}

impl Scene {
    /// Places the drone; `spin` turns its ring.
    pub fn place_drone(&self, pos: Vec3, spin: f32) {
        self.drone.set_transform(&Transform::at(pos));
        self.ring.set_transform(&Transform::at(Vec3::ZERO).with_rot(Quat::from_rotation_x(1.2) * Quat::from_rotation_z(spin)));
    }

    /// A catch at `at`: sparks that keep the drone's momentum, and a flash.
    /// The drone jumps away, so its trail starts a new ribbon rather than
    /// drawing a streak across the screen (re-attaching does that).
    pub fn caught(&self, at: Vec3, velocity: Vec3) {
        self.sparks.emit_moving(at, Vec3::ZERO, 120, velocity * 0.5);
        gfx3d::shockwave(at, 2.5, 0.01, 0.6);
        self.flash.set_transform(&Transform::at(at));
        self.trail.detach();
        self.trail.attach(self.drone);
    }

    /// The flash light, `age` seconds after the last catch.
    pub fn flash_light(&self, age: Option<f32>) {
        let i = age.map_or(0.0, |a| 40.0 * (-a * 8.0).exp());
        self.flash.point_light(srgb(0x9fe8ff), if i > 0.05 { i } else { 0.0 }, 6.0);
    }
}
