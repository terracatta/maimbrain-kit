use super::*;

const DT: f32 = 1.0 / 60.0;

fn floor(w: &mut World) -> BodyId {
    w.add(Body::fixed(vec3(0.0, -0.5, 0.0)), Shape::cuboid(vec3(40.0, 1.0, 40.0)))
}

fn jitter_dt(i: u32) -> f32 {
    let x = (i.wrapping_mul(2654435761) >> 16) as f32 / 65536.0;
    1.0 / 60.0 + (x - 0.5) * 0.004
}

/// Dominoes in an arc, a marble run ramp (trimesh), marbles, a ragdoll-ish
/// chain of capsules on spherical joints and a motorized spinner.
fn scene() -> (World, BodyId, Vec<BodyId>) {
    let mut w = World::new();
    floor(&mut w);
    let mut dominoes = Vec::new();
    for i in 0..24 {
        let a = i as f32 * 0.12;
        let pos = vec3(a.cos() * 4.0, 0.5, a.sin() * 4.0);
        let rot = Quat::from_rotation_y(-a);
        dominoes.push(w.add(Body::dynamic(pos).rotation(rot), Shape::cuboid(vec3(0.1, 1.0, 0.5)).density(600.0)));
    }
    let ramp = MeshData::plane(2.0, 6.0).transformed(&Transform::at(vec3(-3.0, 1.0, 0.0)).with_rot(Quat::from_rotation_x(0.3)).matrix());
    w.add(Body::fixed(Vec3::ZERO), Shape::trimesh(&ramp));
    for i in 0..10 {
        w.add(Body::dynamic(vec3(-3.0 + (i % 3) as f32 * 0.3, 3.0 + i as f32 * 0.25, -2.0)), Shape::sphere(0.12).restitution(0.3));
    }
    let top = w.add(Body::fixed(vec3(2.0, 4.0, -3.0)), Shape::sphere(0.05));
    let mut prev = top;
    for k in 0..5 {
        let c = vec3(2.0, 3.6 - k as f32 * 0.5, -3.0);
        let seg = w.add(Body::dynamic(c), Shape::capsule(0.1, 0.5));
        w.join(Joint::spherical(prev, seg, c + vec3(0.0, 0.25, 0.0)));
        prev = seg;
    }
    let hub = w.add(Body::fixed(vec3(-1.0, 0.3, 2.0)), Shape::sphere(0.05).sensor());
    let spinner = w.add(Body::dynamic(vec3(-1.0, 0.3, 2.0)), Shape::cuboid(vec3(2.0, 0.1, 0.2)));
    w.join(Joint::revolute(hub, spinner, vec3(-1.0, 0.3, 2.0), Vec3::Y).motor(3.0, 500.0));
    (w, dominoes[0], dominoes)
}

#[test]
fn identical_inputs_give_bit_identical_worlds() {
    let run = || {
        let (mut w, first, _) = scene();
        let mut hashes = Vec::new();
        let mut events = Vec::new();
        for i in 0..3000u32 {
            if i == 30 {
                w.apply_impulse_at(first, vec3(0.0, 0.0, 2.0), w.position(first) + vec3(0.0, 0.4, 0.0));
            }
            w.step(jitter_dt(i));
            events.extend(w.events().iter().map(|e| format!("{i} {e:?}")));
            hashes.push(w.state_hash());
        }
        (hashes, events)
    };
    let (h1, e1) = run();
    let (h2, e2) = run();
    for (i, (a, b)) in h1.iter().zip(&h2).enumerate() {
        assert_eq!(a, b, "diverged at frame {i}");
    }
    assert_eq!(e1, e2);
    assert!(e1.len() > 50, "{} events", e1.len());
}

#[test]
fn dominoes_fall_in_order() {
    let (mut w, first, dominoes) = scene();
    for i in 0..600u32 {
        if i == 10 {
            w.apply_impulse_at(first, vec3(0.0, 0.0, 3.0), w.position(first) + vec3(0.0, 0.4, 0.0));
        }
        w.step(DT);
    }
    let fallen = dominoes.iter().filter(|&&d| w.position(d).y < 0.35).count();
    assert!(fallen >= 20, "{fallen} of {} fell", dominoes.len());
}

#[test]
fn free_fall_and_contact_speed() {
    let mut w = World::new();
    let f = floor(&mut w);
    let ball = w.add(Body::dynamic(vec3(0.0, 5.1, 0.0)), Shape::sphere(0.1));
    let mut hit = None;
    for _ in 0..120 {
        w.step(DT);
        for e in w.events() {
            if let Event::ContactBegin(c) = *e {
                hit.get_or_insert(c);
            }
        }
    }
    let c = hit.expect("no contact");
    assert!(c.involves(ball) && c.other(ball) == Some(f));
    // √(2·9.81·5) ≈ 9.9 m/s
    assert!((c.speed - 9.9).abs() < 0.5, "speed {}", c.speed);
    assert!(c.impulse > 0.0);
    assert!(c.normal.dot(w.position(c.b) - w.position(c.a)) > 0.0);
}

#[test]
fn queries_sync_and_mesh() {
    let mut w = World::new();
    let f = floor(&mut w);
    let b = w.add(Body::dynamic(vec3(0.0, 0.25, 0.0)), Shape::cube(0.5));
    w.step(DT);
    let hit = w.raycast(vec3(3.0, 5.0, 3.0), vec3(3.0, -5.0, 3.0)).unwrap();
    assert_eq!(hit.body, f);
    assert!(hit.point.y.abs() < 1e-3 && (hit.normal.y - 1.0).abs() < 1e-3 && (hit.distance - 5.0).abs() < 1e-3);
    assert_eq!(w.raycast(vec3(-2.0, 0.25, 0.0), vec3(2.0, 0.25, 0.0)).map(|h| h.body), Some(b));
    assert_eq!(w.bodies_at(vec3(0.0, 0.25, 0.0)), vec![b]);
    let t = w.draw_transform(b);
    assert!((t.pos.y - 0.25).abs() < 0.05);
    w.sync(b, Node::new().unwrap()); // stub host: no panic
    let m = Shape::capsule(0.2, 1.0).mesh(16).unwrap();
    let ys: Vec<f32> = m.positions.iter().map(|p| p.y).collect();
    let (lo, hi) = (ys.iter().cloned().fold(f32::MAX, f32::min), ys.iter().cloned().fold(f32::MIN, f32::max));
    assert!((lo + 0.5).abs() < 1e-3 && (hi - 0.5).abs() < 1e-3, "{lo} {hi}");
    assert!(Shape::convex_hull_of(&MeshData::icosphere(1.0, 1)).mesh(8).is_none());
}

#[test]
fn joints_and_motor() {
    let mut w = World::with_gravity(Vec3::ZERO);
    let hub = w.add(Body::fixed(Vec3::ZERO), Shape::sphere(0.05));
    let wheel = w.add(Body::dynamic(Vec3::ZERO).rotation(Quat::from_rotation_x(0.7)), Shape::cylinder(0.5, 0.2));
    let j = w.join(Joint::revolute(hub, wheel, Vec3::ZERO, vec3(0.0, 0.0, 1.0)).motor(2.0, f32::INFINITY));
    let slide = w.add(Body::dynamic(vec3(2.0, 0.0, 0.0)).rotation(Quat::from_rotation_y(0.4)), Shape::cube(0.2));
    w.join(Joint::prismatic(hub, slide, vec3(2.0, 0.0, 0.0), Vec3::X).limits(-0.5, 0.5).motor(1.0, f32::INFINITY));
    let rot0 = w.rotation(slide);
    for _ in 0..120 {
        w.step(DT);
    }
    let s = w.spin(wheel);
    assert!((s.z - 2.0).abs() < 0.05 && s.x.abs() < 0.05 && s.y.abs() < 0.05, "spin {s:?}");
    let p = w.position(slide);
    assert!((p.x - 2.5).abs() < 0.02 && p.y.abs() < 1e-3 && p.z.abs() < 1e-3, "slider {p:?}");
    assert!(w.rotation(slide).dot(rot0).abs() > 0.9999, "the slider kept its rotation");
    w.set_motor(j, -1.0, f32::INFINITY);
    for _ in 0..60 {
        w.step(DT);
    }
    assert!((w.spin(wheel).z + 1.0).abs() < 0.05);
}

#[test]
fn kinematic_targets_and_forces() {
    let mut w = World::with_gravity(Vec3::ZERO);
    let k = w.add(Body::kinematic(Vec3::ZERO), Shape::cube(1.0));
    let target_rot = Quat::from_rotation_y(1.0);
    for i in 1..=60 {
        let t = i as f32 / 60.0;
        w.move_kinematic(k, vec3(t * 2.0, 0.0, 0.0), Quat::IDENTITY.slerp(target_rot, t));
        w.step(DT);
    }
    assert!((w.position(k).x - 2.0).abs() < 0.01);
    assert!(w.rotation(k).dot(target_rot).abs() > 0.9999);
    let b = w.add(Body::dynamic(vec3(10.0, 0.0, 0.0)), Shape::cube(0.1)); // 1 kg
    assert!((w.mass(b) - 1.0).abs() < 1e-3);
    for _ in 0..120 {
        w.apply_force(b, vec3(0.0, 3.0, 0.0));
        w.step(1.0 / 120.0);
    }
    assert!((w.velocity(b).y - 3.0).abs() < 0.05, "{:?}", w.velocity(b));
}
