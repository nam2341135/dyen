//! Sự sống: gió, nước, đèn, khói bếp, cò, thuyền, người, trâu, diều, đom đóm.

use std::f32::consts::{PI, TAU};

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::geom::{Geo, Rng, hex, mix, shade, smoothstep};
use crate::sky::{Clock, Env};
use crate::village::{Mats, NightLamp, Spots, Sway, VertexSway};
use crate::world::{self, BRIDGE_X, POND, POND_R, RIVER_W, WATER_Y, WaterAnim, height, river_z};

/// Chất lượng: cao = bóng, bloom, MSAA, lúa lay theo đỉnh.
#[derive(Resource)]
pub struct Quality {
    pub high: bool,
}

impl Default for Quality {
    fn default() -> Self {
        Self { high: true }
    }
}

/// Hướng gió (mặt phẳng ngang).
const WIND: Vec3 = Vec3::new(0.94, 0.0, 0.34);

// ───────────────────────── Gió ─────────────────────────

pub fn sway(clock: Res<Clock>, env: Res<Env>, mut q: Query<(&Sway, &mut Transform)>) {
    let t = clock.t;
    let axis = Vec3::Y.cross(WIND).normalize();
    for (s, mut tr) in &mut q {
        // Sóng gió lan theo vị trí: các bụi không lay đồng loạt.
        let along = tr.translation.dot(WIND) * 0.06;
        let wave = (t * s.freq + s.phase - along).sin() * 0.6 + (t * s.freq * 2.3 + s.phase * 1.7).sin() * 0.25;
        let lean = s.amp * env.gust * (0.6 + wave);
        let side = s.amp * 0.35 * (t * s.freq * 1.3 + s.phase).sin();
        tr.rotation = Quat::from_axis_angle(axis, lean) * Quat::from_axis_angle(WIND, side) * s.base;
    }
}

/// Lúa lay theo đỉnh; mỗi khung chỉ cập nhật một phần ba số ô (đỡ tải lên GPU).
pub fn rice(
    clock: Res<Clock>,
    env: Res<Env>,
    quality: Res<Quality>,
    mut frame: Local<u32>,
    q: Query<(&VertexSway, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !quality.high {
        return;
    }
    *frame = frame.wrapping_add(1);
    let t = clock.t;
    for (i, (vs, m)) in q.iter().enumerate() {
        if (i as u32 + *frame) % 3 != 0 {
            continue;
        }
        let Some(mut mesh) = meshes.get_mut(&m.0) else { continue };
        let pos: Vec<[f32; 3]> = vs
            .base
            .iter()
            .zip(&vs.w)
            .map(|(p, &w)| {
                if w == 0.0 {
                    return p.to_array();
                }
                // Sóng lúa: dải gió chạy ngang đồng.
                let ph = t * 1.6 - p.dot(WIND) * 0.12;
                let k = vs.amp * env.gust * w * (0.55 + 0.45 * ph.sin() + 0.25 * (ph * 2.7 + p.z * 0.3).sin());
                (*p + WIND * k - Vec3::Y * k * k * 0.4).to_array()
            })
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
}

// ───────────────────────── Nước ─────────────────────────

pub fn water(
    clock: Res<Clock>,
    env: Res<Env>,
    quality: Res<Quality>,
    mut frame: Local<u32>,
    cam: Query<&Transform, With<Camera3d>>,
    q: Query<(&WaterAnim, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Sóng chậm: cập nhật 1/2 khung (cao) hoặc 1/4 khung (thấp) là đủ mượt, tiết kiệm CPU + tải lên GPU.
    *frame = frame.wrapping_add(1);
    if *frame % if quality.high { 2 } else { 4 } != 0 {
        return;
    }
    let Ok(cam) = cam.single() else { return };
    let eye = cam.translation;
    let t = clock.t;
    let (wat, hz, zen, fog) = (env.water.to_linear(), env.horizon.to_linear(), env.zenith.to_linear(), env.fog.to_linear());
    let v3 = |c: LinearRgba| Vec3::new(c.red, c.green, c.blue);
    let (wat, hz, zen, fog) = (v3(wat), v3(hz), v3(zen), v3(fog));
    let sun_on = 1.0 - env.night;
    let light_dir = if env.sun_dir.y > -0.04 { env.sun_dir } else { env.moon_dir };
    let glint_col = v3(env.sun_col.to_linear()) * (2.6 * sun_on + 0.9 * env.night);
    for (wa, m) in &q {
        let Some(mut mesh) = meshes.get_mut(&m.0) else { continue };
        let n = wa.base.len();
        let mut pos = Vec::with_capacity(n);
        for (p, &a) in wa.base.iter().zip(&wa.amp) {
            let h = (p.x * 0.45 + t * 1.3).sin() * 0.5
                + (p.z * 0.7 - t * 1.1 + p.x * 0.2).sin() * 0.35
                + (p.x * 1.3 - p.z * 0.9 + t * 2.1).sin() * 0.15;
            pos.push(Vec3::new(p.x, p.y + a * h, p.z));
        }
        let mut nor = Vec::with_capacity(n);
        let mut col = Vec::with_capacity(n);
        // Pháp tuyến giải tích theo từng đỉnh (mượt, không lộ tam giác) + gợn nhỏ chỉ cho ánh lóa.
        for (p, (b, &a)) in pos.iter().zip(wa.base.iter().zip(&wa.amp)) {
            let (ca, cb, cc) = (
                (b.x * 0.45 + t * 1.3).cos(),
                (b.z * 0.7 - t * 1.1 + b.x * 0.2).cos(),
                (b.x * 1.3 - b.z * 0.9 + t * 2.1).cos(),
            );
            let cd = (b.x * 2.9 + b.z * 2.3 + t * 3.4).cos();
            let ce = (b.x * -2.1 + b.z * 3.1 + t * 2.7).cos();
            let dx = 0.225 * ca + 0.07 * cb + 0.195 * cc + 0.09 * cd - 0.07 * ce;
            let dz = 0.245 * cb - 0.135 * cc + 0.07 * cd + 0.1 * ce;
            let k = 1.3 * a.max(0.25);
            let nn = Vec3::new(-dx * k, 1.0, -dz * k).normalize();
            let v = (*p - eye).normalize();
            let r = v - 2.0 * v.dot(nn) * nn;
            let fres = (1.0 - (-v).dot(nn).max(0.0)).powi(4) * 0.8 + 0.12;
            let sky = hz.lerp(zen, r.y.max(0.0).powf(0.5));
            let sky = if r.y < 0.0 { sky.lerp(fog, 0.5) } else { sky };
            let glint = r.dot(light_dir).max(0.0).powf(90.0) * (light_dir.y > 0.0) as u8 as f32;
            let c = wat.lerp(sky, fres) + glint_col * glint;
            col.push([c.x, c.y, c.z, 0.86 + 0.12 * fres]);
            nor.push(nn.to_array());
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos.iter().map(|p| p.to_array()).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nor);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
    }
}

// ───────────────────────── Đèn ─────────────────────────

pub fn lamps(
    clock: Res<Clock>,
    env: Res<Env>,
    mats: Res<Mats>,
    life: Res<LifeMats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(&NightLamp, &mut PointLight, &Transform)>,
) {
    // Đèn thắp dần khi chiều tắt nắng.
    let k = smoothstep(0.0, 0.7, env.night);
    let t = clock.t;
    if let Some(mut m) = materials.get_mut(&mats.glow) {
        let b = 0.45 + 7.0 * k;
        m.base_color = Color::linear_rgb(b, b, b);
    }
    if let Some(mut m) = materials.get_mut(&life.firefly) {
        let a = smoothstep(0.35, 0.8, env.night);
        m.base_color = Color::linear_rgba(5.0, 7.0, 1.6, a);
    }
    for (l, mut pl, tr) in &mut q {
        let flick = 0.9 + 0.1 * (t * 9.0 + tr.translation.x).sin() * (t * 5.3 + tr.translation.z).sin();
        pl.intensity = l.max * 140.0 * k * flick;
    }
}

// ───────────────────────── Vật liệu dùng chung ─────────────────────────

#[derive(Resource)]
pub struct LifeMats {
    pub smoke: Vec<Handle<StandardMaterial>>,
    pub firefly: Handle<StandardMaterial>,
}

// ───────────────────────── Khói bếp ─────────────────────────

#[derive(Component)]
pub struct Puff {
    origin: Vec3,
    phase: f32,
}

const SMOKE_LIFE: f32 = 9.0;

pub fn smoke(
    clock: Res<Clock>,
    env: Res<Env>,
    life: Res<LifeMats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(&Puff, &mut Transform, &mut MeshMaterial3d<StandardMaterial>, &mut Visibility)>,
) {
    // Khói bếp nấu cơm: chiều 15h–20h, sáng 5h–7h.
    let h = clock.hour;
    let on = (smoothstep(15.0, 16.2, h) * smoothstep(20.5, 19.5, h)).max(smoothstep(4.8, 5.4, h) * smoothstep(7.5, 6.8, h));
    let tint = mix(hex(0xd8d4cc), env.fog, 0.35);
    let tl = tint.to_linear();
    for (i, mh) in life.smoke.iter().enumerate() {
        if let Some(mut m) = materials.get_mut(mh) {
            let a = 0.42 * (1.0 - i as f32 / life.smoke.len() as f32) * on;
            m.base_color = Color::linear_rgba(tl.red, tl.green, tl.blue, a);
        }
    }
    let t = clock.t;
    for (p, mut tr, mut mat, mut vis) in &mut q {
        if on < 0.01 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        let age = (t / SMOKE_LIFE + p.phase).fract();
        let a = age * SMOKE_LIFE;
        let drift = WIND * (a * 0.5 + a * a * 0.06) * env.gust;
        let curl = Vec3::new((a * 0.9 + p.phase * 9.0).sin(), 0.0, (a * 0.7 + p.phase * 5.0).cos()) * a * 0.12;
        tr.translation = p.origin + Vec3::Y * (a * 1.1 - a * a * 0.02) + drift + curl;
        tr.scale = Vec3::splat(0.35 + a * 0.32);
        let tier = ((age.powf(0.8)) * life.smoke.len() as f32) as usize;
        let want = &life.smoke[tier.min(life.smoke.len() - 1)];
        if mat.0 != *want {
            mat.0 = want.clone();
        }
    }
}

// ───────────────────────── Cò ─────────────────────────

#[derive(Component)]
pub struct Bird {
    slot: Vec3,
    phase: f32,
}
#[derive(Component)]
pub struct Wing {
    side: f32,
    phase: f32,
}
#[derive(Component)]
pub struct Flock;

fn flock_pos(t: f32) -> (Vec3, Vec3) {
    // Vòng bay rộng trên đồng, chậm rãi.
    let a = t * 0.018;
    let p = Vec3::new(30.0 + 150.0 * a.cos(), 34.0 + 8.0 * (a * 3.0).sin(), -10.0 + 85.0 * a.sin());
    let d = Vec3::new(-150.0 * a.sin(), 24.0 * (a * 3.0).cos(), 85.0 * a.cos()).normalize();
    (p, d)
}

pub fn birds(
    clock: Res<Clock>,
    env: Res<Env>,
    mut flock: Query<(&mut Transform, &mut Visibility), (With<Flock>, Without<Bird>, Without<Wing>)>,
    mut q: Query<(&Bird, &mut Transform), (Without<Flock>, Without<Wing>)>,
    mut wings: Query<(&Wing, &mut Transform), (Without<Flock>, Without<Bird>)>,
) {
    let t = clock.t;
    let (p, d) = flock_pos(t);
    if let Ok((mut tr, mut vis)) = flock.single_mut() {
        *vis = if env.night > 0.6 { Visibility::Hidden } else { Visibility::Inherited };
        *tr = Transform::from_translation(p).looking_to(d, Vec3::Y);
    }
    for (b, mut tr) in &mut q {
        let bob = (t * 0.8 + b.phase * 7.0).sin() * 0.4;
        tr.translation = b.slot + Vec3::new((t * 0.5 + b.phase).sin() * 0.5, bob, 0.0);
    }
    for (w, mut tr) in &mut wings {
        // Vỗ chậm rồi lượn.
        let cyc = (t * 0.25 + w.phase).fract();
        let flap = if cyc < 0.6 { (t * 5.5 + w.phase * 6.0).sin() * 0.7 } else { 0.08 };
        tr.rotation = Quat::from_rotation_z(w.side * flap);
    }
}

fn spawn_birds(commands: &mut Commands, meshes: &mut Assets<Mesh>, mat: &Handle<StandardMaterial>, rng: &mut Rng) {
    let white = hex(0xf4f2ea);
    let mut body = Geo::default();
    body.blob(Vec3::ZERO, Vec3::new(0.22, 0.2, 0.55), white, 0.05, rng);
    body.tube(Vec3::new(0.0, 0.05, -0.45), Vec3::new(0.0, 0.12, -0.85), 0.08, 0.06, 5, white, white, false);
    body.blob(Vec3::new(0.0, 0.14, -0.92), Vec3::new(0.1, 0.1, 0.13), white, 0.0, rng);
    body.tube(Vec3::new(0.0, 0.14, -1.0), Vec3::new(0.0, 0.1, -1.3), 0.03, 0.0, 4, hex(0xe0b030), hex(0xe0b030), false);
    body.tube(Vec3::new(0.0, -0.05, 0.4), Vec3::new(0.0, -0.08, 0.9), 0.04, 0.02, 4, hex(0x30302a), hex(0x30302a), false);
    let mut wing = Geo::default();
    wing.tri2(Vec3::new(0.0, 0.0, -0.3), Vec3::new(1.5, 0.0, 0.05), Vec3::new(0.0, 0.0, 0.3), white);
    wing.tri2(Vec3::new(1.5, 0.0, 0.05), Vec3::new(1.1, 0.0, 0.35), Vec3::new(0.0, 0.0, 0.3), shade(white, 0.9));
    let body = meshes.add(body.mesh());
    let wing = meshes.add(wing.mesh());
    let flock = commands.spawn((Flock, Transform::default(), Visibility::default())).id();
    let slots = [(0.0, 0.0), (-3.0, 3.0), (3.0, 3.2), (-6.2, 6.0), (6.0, 6.5), (-9.0, 9.4), (9.5, 9.0)];
    for (k, &(x, z)) in slots.iter().enumerate() {
        let phase = rng.f();
        let slot = Vec3::new(x, rng.range(-0.6, 0.6), z);
        let bird = commands
            .spawn((
                Bird { slot, phase },
                Mesh3d(body.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_translation(slot).with_scale(Vec3::splat(1.0 + 0.05 * k as f32)),
                NotShadowCaster,
            ))
            .with_children(|b| {
                for side in [1.0f32, -1.0] {
                    b.spawn((
                        Wing { side, phase },
                        Mesh3d(wing.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::default(),
                        NotShadowCaster,
                    ))
                    .insert(Transform::from_scale(Vec3::new(side, 1.0, 1.0)));
                }
            })
            .id();
        commands.entity(flock).add_child(bird);
    }
}

// ───────────────────────── Thuyền ─────────────────────────

#[derive(Component)]
pub struct Boat;
#[derive(Component)]
pub struct Oar;

const BOAT_SPAN: f32 = 360.0;
const BOAT_SPEED: f32 = 1.6;

fn boat_x(t: f32) -> f32 {
    -BOAT_SPAN * 0.5 + (t * BOAT_SPEED + 230.0).rem_euclid(BOAT_SPAN)
}

pub fn boat(
    clock: Res<Clock>,
    mut q: Query<&mut Transform, (With<Boat>, Without<Oar>)>,
    mut oar: Query<&mut Transform, (With<Oar>, Without<Boat>)>,
) {
    let t = clock.t;
    let x = boat_x(t);
    let z = river_z(x) + 1.5;
    let dz = river_z(x + 1.0) + 1.5 - z;
    if let Ok(mut tr) = q.single_mut() {
        let yaw = (-dz).atan2(1.0);
        tr.translation = Vec3::new(x, WATER_Y + 0.05 + 0.05 * (t * 1.7).sin(), z);
        tr.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(0.025 * (t * 1.3).sin()) * Quat::from_rotation_z(0.03 * (t * 0.9).sin());
    }
    for mut tr in &mut oar {
        let a = (t * 1.4).sin();
        tr.rotation = Quat::from_rotation_y(0.5 * a) * Quat::from_rotation_z(-0.9 + 0.08 * a);
    }
}

fn person(rng: &mut Rng, shirt: Color, hat: bool) -> Geo {
    let mut g = Geo::default();
    let skin = hex(0xc4946a);
    g.tube(Vec3::new(0.0, 0.85, 0.0), Vec3::new(0.0, 1.45, 0.0), 0.21, 0.18, 7, shirt, shade(shirt, 1.08), true);
    g.blob(Vec3::new(0.0, 1.6, 0.0), Vec3::new(0.12, 0.13, 0.12), skin, 0.0, rng);
    if hat {
        let leaf = hex(0xeadbb0);
        g.tube(Vec3::new(0.0, 1.69, 0.0), Vec3::new(0.0, 1.98, 0.0), 0.43, 0.0, 12, shade(leaf, 0.92), leaf, false);
        g.tube(Vec3::new(0.0, 1.69, 0.0), Vec3::new(0.0, 1.69, 0.0), 0.0, 0.43, 12, shade(leaf, 0.6), shade(leaf, 0.6), false);
    } else {
        g.blob(Vec3::new(0.0, 1.66, 0.0), Vec3::new(0.13, 0.08, 0.13), hex(0x2a2420), 0.0, rng);
    }
    // Tay buông.
    for s in [-1.0f32, 1.0] {
        g.beam(Vec3::new(s * 0.22, 1.4, 0.0), Vec3::new(s * 0.26, 0.95, 0.05), 0.05, shirt);
    }
    g
}

fn leg_geo() -> Geo {
    let mut g = Geo::default();
    let trousers = hex(0x2e2a28);
    g.beam(Vec3::ZERO, Vec3::new(0.0, -0.8, 0.0), 0.065, trousers);
    g.bxa(Vec3::new(0.0, -0.83, -0.05), Vec3::new(0.06, 0.03, 0.1), hex(0xa27a58));
    g
}

fn spawn_boat(commands: &mut Commands, meshes: &mut Assets<Mesh>, mats: &Mats, glow: &Handle<StandardMaterial>, rng: &mut Rng) {
    let mut g = Geo::default();
    // Thân thuyền nan: vỏ cong, đan tre.
    let (len, w, d) = (3.4f32, 0.85f32, 0.45f32);
    let n = 14;
    let sec = |u: f32| {
        let s = (u * PI).sin();
        (w * s.powf(0.7), d * (0.5 + 0.5 * s), 0.25 * (2.0 * u - 1.0).powi(4))
    };
    let k = 8;
    for i in 0..n {
        let (u0, u1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
        let (w0, d0, r0) = sec(u0);
        let (w1, d1, r1) = sec(u1);
        let (x0, x1) = (-len + 2.0 * len * u0, -len + 2.0 * len * u1);
        for j in 0..k {
            let (a0, a1) = (j as f32 / k as f32 * PI, (j + 1) as f32 / k as f32 * PI);
            let p = |x: f32, ww: f32, dd: f32, rr: f32, a: f32| Vec3::new(x, rr - dd * a.sin() + 0.25, ww * a.cos());
            let col = if (i + j) % 2 == 0 { hex(0x5a4632) } else { hex(0x4e3c2a) };
            g.quad2(p(x0, w0, d0, r0, a0), p(x1, w1, d1, r1, a0), p(x1, w1, d1, r1, a1), p(x0, w0, d0, r0, a1), col);
        }
    }
    // Mui thuyền (mái che cong).
    for i in 0..8 {
        let (a0, a1) = (i as f32 / 8.0 * PI, (i + 1) as f32 / 8.0 * PI);
        let p = |x: f32, a: f32| Vec3::new(x, 0.3 + 0.7 * a.sin(), 0.7 * a.cos());
        let c = if i % 2 == 0 { hex(0x9a8258) } else { hex(0x8a744e) };
        g.quad2(p(-1.1, a0), p(0.6, a0), p(0.6, a1), p(-1.1, a1), c);
    }
    // Người chèo đứng ở đuôi, nón lá.
    let rower = person(rng, hex(0x4a5a6a), true);
    g.add(&rower, Transform::from_xyz(2.3, 0.1, 0.0).with_scale(Vec3::splat(0.95)));
    for s in [-1.0f32, 1.0] {
        g.beam(Vec3::new(2.3, 0.85, s * 0.1), Vec3::new(2.3, 0.15, s * 0.12), 0.065, hex(0x2e2a28));
    }
    let boat = commands
        .spawn((Boat, Mesh3d(meshes.add(g.mesh())), MeshMaterial3d(mats.vc.clone()), Transform::default()))
        .id();
    let mut oar = Geo::default();
    oar.beam(Vec3::ZERO, Vec3::new(0.0, -3.2, 0.0), 0.04, hex(0x8a6a40));
    oar.bxa(Vec3::new(0.0, -3.0, 0.0), Vec3::new(0.03, 0.35, 0.13), hex(0x7a5a38));
    let oar = commands
        .spawn((Oar, Mesh3d(meshes.add(oar.mesh())), MeshMaterial3d(mats.vc.clone()), Transform::from_xyz(2.6, 1.3, 0.3)))
        .id();
    // Đèn lồng treo mũi thuyền.
    let mut lg = Geo::default();
    lg.blob(Vec3::ZERO, Vec3::new(0.16, 0.22, 0.16), hex(0xff9a40), 0.0, rng);
    let lamp = commands
        .spawn((Mesh3d(meshes.add(lg.mesh())), MeshMaterial3d(glow.clone()), Transform::from_xyz(-1.0, 1.25, 0.0), NotShadowCaster))
        .id();
    let light = commands
        .spawn((
            NightLamp { max: 4000.0 },
            PointLight { color: hex(0xffa850), intensity: 0.0, range: 10.0, shadow_maps_enabled: false, ..default() },
            Transform::from_xyz(-1.0, 1.0, 0.0),
        ))
        .id();
    commands.entity(boat).add_children(&[oar, lamp, light]);
}

// ───────────────────────── Người đi trên đê ─────────────────────────

#[derive(Component)]
pub struct Walker {
    x0: f32,
    x1: f32,
    speed: f32,
    phase: f32,
    /// Lệch khỏi tâm đường đi.
    off: f32,
    /// Đường đi: 0 = đê, 1 = bờ nam ra ao.
    route: u8,
}
#[derive(Component)]
pub struct Leg {
    side: f32,
}
#[derive(Component)]
pub struct Basket {
    side: f32,
}

fn walker_pos(w: &Walker, t: f32) -> (Vec3, f32, f32) {
    let span = w.x1 - w.x0;
    let s = (t * w.speed / span + w.phase).rem_euclid(2.0);
    let (u, dir) = if s < 1.0 { (s, 1.0) } else { (2.0 - s, -1.0) };
    let x = w.x0 + span * u;
    match w.route {
        0 => {
            let z = world::dike_z(x) + w.off;
            (Vec3::new(x, height(x, z), z), dir, 0.0)
        }
        _ => {
            // Bờ nam: tham số theo z, đường đi chéo về ao.
            let z = x;
            let rz = river_z(BRIDGE_X);
            let sx = BRIDGE_X - 0.18 * (z - rz - 9.0) + 2.0 * (z * 0.09).sin() + w.off;
            (Vec3::new(sx, height(sx, z), z), dir, 1.0)
        }
    }
}

pub fn walkers(
    clock: Res<Clock>,
    env: Res<Env>,
    mut q: Query<(&Walker, &mut Transform, &mut Visibility), (Without<Leg>, Without<Basket>)>,
    mut legs: Query<(&Leg, &ChildOf, &mut Transform), (Without<Walker>, Without<Basket>)>,
    mut baskets: Query<(&Basket, &mut Transform), (Without<Walker>, Without<Leg>)>,
) {
    let t = clock.t;
    let home = env.night > 0.85;
    for (w, mut tr, mut vis) in &mut q {
        *vis = if home { Visibility::Hidden } else { Visibility::Inherited };
        let (p, dir, route) = walker_pos(w, t);
        let (p2, _, _) = walker_pos(w, t + 0.2);
        let mut fwd = (p2 - p).with_y(0.0);
        if fwd.length_squared() < 1e-6 {
            fwd = if route == 0.0 { Vec3::X * dir } else { Vec3::Z * dir };
        }
        let step = (t * w.speed * 3.2 + w.phase * 10.0).sin();
        tr.translation = p + Vec3::Y * (step.abs() * 0.05);
        tr.look_to(fwd.normalize(), Vec3::Y);
    }
    for (l, parent, mut tr) in &mut legs {
        let Ok((w, _, _)) = q.get(parent.parent()) else { continue };
        let a = (t * w.speed * 3.2 + w.phase * 10.0).sin() * 0.5;
        tr.rotation = Quat::from_rotation_x(a * l.side);
    }
    for (b, mut tr) in &mut baskets {
        tr.rotation = Quat::from_rotation_x(0.12 * (t * 4.0 + b.side).sin());
    }
}

fn spawn_walkers(commands: &mut Commands, meshes: &mut Assets<Mesh>, vc: &Handle<StandardMaterial>, rng: &mut Rng) {
    let leg = meshes.add(leg_geo().mesh());
    let list = [
        (hex(0x5a3e30), true, true, -110.0, 90.0, 1.1, 0.0, 0.6, 0u8),
        (hex(0x3e4a5a), true, false, -80.0, 120.0, 1.3, 0.55, -0.6, 0),
        (hex(0x7a5a8a), false, false, -40.0, 60.0, 0.9, 1.3, 0.3, 0),
        (hex(0x8a3a2a), true, false, 32.0, 88.0, 1.0, 0.2, 0.0, 1),
    ];
    for (shirt, hat, yoke, x0, x1, speed, phase, off, route) in list {
        let body = meshes.add(person(rng, shirt, hat).mesh());
        let id = commands
            .spawn((
                Walker { x0, x1, speed, phase, off, route },
                Mesh3d(body),
                MeshMaterial3d(vc.clone()),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        let mut kids = Vec::new();
        for side in [1.0f32, -1.0] {
            kids.push(
                commands
                    .spawn((Leg { side }, Mesh3d(leg.clone()), MeshMaterial3d(vc.clone()), Transform::from_xyz(side * 0.1, 0.88, 0.0)))
                    .id(),
            );
        }
        if yoke {
            // Đòn gánh trên vai, hai thúng.
            let mut pole = Geo::default();
            pole.beam(Vec3::new(0.0, 0.0, -1.1), Vec3::new(0.0, 0.0, 1.1), 0.035, hex(0xb89a60));
            kids.push(
                commands
                    .spawn((Mesh3d(meshes.add(pole.mesh())), MeshMaterial3d(vc.clone()), Transform::from_xyz(0.18, 1.5, 0.0)))
                    .id(),
            );
            let mut bk = Geo::default();
            for s in [-1.0f32, 1.0] {
                bk.beam(Vec3::ZERO, Vec3::new(s * 0.25, -0.9, 0.0), 0.012, hex(0x6a5a40));
            }
            bk.tube(Vec3::new(0.0, -1.2, 0.0), Vec3::new(0.0, -0.85, 0.0), 0.26, 0.36, 9, hex(0x9a7a48), hex(0xb08a50), true);
            bk.blob(Vec3::new(0.0, -0.82, 0.0), Vec3::new(0.3, 0.12, 0.3), hex(0x7aa04a), 0.15, rng);
            let bk = meshes.add(bk.mesh());
            for side in [-1.0f32, 1.0] {
                kids.push(
                    commands
                        .spawn((Basket { side }, Mesh3d(bk.clone()), MeshMaterial3d(vc.clone()), Transform::from_xyz(0.18, 1.5, side * 1.0)))
                        .id(),
                );
            }
        }
        commands.entity(id).add_children(&kids);
    }
}

// ───────────────────────── Trâu và trẻ chăn trâu ─────────────────────────

#[derive(Component)]
pub struct Buffalo {
    home: Vec2,
    phase: f32,
}

fn buffalo_geo(rng: &mut Rng, rider: bool) -> Geo {
    let mut g = Geo::default();
    let hide = hex(0x3e3a3a);
    g.blob(Vec3::new(0.0, 1.15, 0.0), Vec3::new(0.62, 0.55, 1.15), hide, 0.05, rng);
    g.blob(Vec3::new(0.0, 1.05, -1.35), Vec3::new(0.3, 0.3, 0.42), shade(hide, 1.1), 0.05, rng);
    // Sừng cong.
    for s in [-1.0f32, 1.0] {
        let pts = [Vec3::new(s * 0.2, 1.3, -1.35), Vec3::new(s * 0.55, 1.4, -1.25), Vec3::new(s * 0.75, 1.55, -1.0), Vec3::new(s * 0.6, 1.7, -0.85)];
        for w in pts.windows(2) {
            g.beam(w[0], w[1], 0.05, hex(0xc8bca0));
        }
    }
    for (x, z) in [(-0.35, -0.75), (0.35, -0.75), (-0.35, 0.8), (0.35, 0.8)] {
        g.tube(Vec3::new(x, 0.9, z), Vec3::new(x, 0.0, z), 0.13, 0.09, 5, hide, shade(hide, 0.8), true);
    }
    g.beam(Vec3::new(0.0, 1.3, 1.1), Vec3::new(0.0, 0.6, 1.3), 0.03, hide);
    if rider {
        let kid = person(rng, hex(0x6a7a4a), true);
        g.add(&kid, Transform::from_xyz(0.0, 0.85, 0.2).with_scale(Vec3::splat(0.7)));
        for s in [-1.0f32, 1.0] {
            g.beam(Vec3::new(s * 0.08, 1.45, 0.2), Vec3::new(s * 0.5, 1.0, 0.0), 0.05, hex(0x2e2a28));
        }
    }
    g
}

pub fn buffalo(clock: Res<Clock>, mut q: Query<(&Buffalo, &mut Transform)>) {
    let t = clock.t;
    for (b, mut tr) in &mut q {
        // Đi chậm, dừng gặm cỏ, quay đầu.
        let a = t * 0.035 + b.phase;
        let p = b.home + Vec2::new(a.cos() * 9.0, (a * 2.0).sin() * 4.0);
        let n = b.home + Vec2::new((a + 0.05).cos() * 9.0, ((a + 0.05) * 2.0).sin() * 4.0);
        let graze = (t * 0.4 + b.phase).sin() > 0.3;
        let d = (n - p).normalize_or(Vec2::X);
        tr.translation = Vec3::new(p.x, height(p.x, p.y) - 0.05, p.y);
        tr.look_to(Vec3::new(d.x, 0.0, d.y), Vec3::Y);
        if graze {
            tr.rotate_local_x(-0.08);
        }
    }
}

// ───────────────────────── Diều ─────────────────────────

#[derive(Component)]
pub struct Kite {
    anchor: Vec3,
}
#[derive(Component)]
pub struct KiteLine;

pub fn kite(
    clock: Res<Clock>,
    env: Res<Env>,
    mut q: Query<(&Kite, &mut Transform, &mut Visibility), Without<KiteLine>>,
    mut line: Query<(&mut Transform, &mut Visibility), (With<KiteLine>, Without<Kite>)>,
) {
    let t = clock.t;
    let show = env.night < 0.5;
    for (k, mut tr, mut vis) in &mut q {
        *vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        let p = k.anchor
            + Vec3::new(-14.0 + 2.5 * (t * 0.31).sin(), 24.0 + 2.0 * (t * 0.47).sin() * env.gust, -9.0 + 2.0 * (t * 0.23).cos());
        tr.translation = p;
        tr.rotation = Quat::from_rotation_y(0.9) * Quat::from_rotation_z(0.25 * (t * 1.1).sin()) * Quat::from_rotation_x(-0.5 + 0.1 * (t * 0.8).sin());
        for (mut lt, mut lv) in &mut line {
            *lv = *vis;
            let d = p - k.anchor;
            lt.translation = k.anchor;
            lt.rotation = Quat::from_rotation_arc(Vec3::Y, d.normalize());
            lt.scale = Vec3::new(1.0, d.length(), 1.0);
        }
    }
}

fn spawn_kite(commands: &mut Commands, meshes: &mut Assets<Mesh>, mats: &Mats, rng: &mut Rng) {
    let (ax, az) = (46.0, 62.0);
    let anchor = Vec3::new(ax, height(ax, az) + 1.2, az);
    let mut kid = person(rng, hex(0xd8a030), false);
    kid.beam(Vec3::new(0.22, 1.4, 0.0), Vec3::new(0.3, 1.25, -0.3), 0.05, hex(0xd8a030));
    for s in [-1.0f32, 1.0] {
        kid.beam(Vec3::new(s * 0.08, 0.85, 0.0), Vec3::new(s * 0.1, 0.0, 0.0), 0.05, hex(0x2e2a28));
    }
    commands.spawn((
        Mesh3d(meshes.add(kid.mesh())),
        MeshMaterial3d(mats.vc.clone()),
        Transform::from_xyz(ax, height(ax, az), az).with_scale(Vec3::splat(0.72)).looking_to(Vec3::new(0.8, 0.0, 0.6), Vec3::Y),
    ));
    let mut g = Geo::default();
    let (top, l, r, bot) = (Vec3::new(0.0, 1.1, 0.0), Vec3::new(-0.8, 0.2, 0.0), Vec3::new(0.8, 0.2, 0.0), Vec3::new(0.0, -0.9, 0.0));
    g.tri2(top, l, Vec3::new(0.0, 0.2, 0.0), hex(0xe83a2a));
    g.tri2(top, Vec3::new(0.0, 0.2, 0.0), r, hex(0xf0c030));
    g.tri2(l, bot, Vec3::new(0.0, 0.2, 0.0), hex(0xf0c030));
    g.tri2(Vec3::new(0.0, 0.2, 0.0), bot, r, hex(0xe83a2a));
    // Đuôi diều.
    let mut prev = bot;
    for k in 1..10 {
        let q = Vec3::new(0.25 * (k as f32 * 1.3).sin(), -0.9 - 0.55 * k as f32, 0.1 * k as f32);
        g.tri2(prev - Vec3::X * 0.06, prev + Vec3::X * 0.06, q, if k % 2 == 0 { hex(0xe83a2a) } else { hex(0x3a6ac8) });
        prev = q;
    }
    commands.spawn((Kite { anchor }, Mesh3d(meshes.add(g.mesh())), MeshMaterial3d(mats.plant.clone()), Transform::default(), Visibility::default()));
    let mut ln = Geo::default();
    ln.tube(Vec3::ZERO, Vec3::Y, 0.015, 0.015, 3, hex(0xeeeeee), hex(0xeeeeee), false);
    commands.spawn((KiteLine, Mesh3d(meshes.add(ln.mesh())), MeshMaterial3d(mats.vc.clone()), Transform::default(), Visibility::default(), NotShadowCaster));
}

// ───────────────────────── Đom đóm ─────────────────────────

#[derive(Component)]
pub struct Firefly {
    home: Vec3,
    phase: f32,
}

pub fn fireflies(clock: Res<Clock>, env: Res<Env>, mut q: Query<(&Firefly, &mut Transform, &mut Visibility)>) {
    let t = clock.t;
    let show = env.night > 0.3;
    for (f, mut tr, mut vis) in &mut q {
        *vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        if !show {
            continue;
        }
        let p = f.phase * 37.0;
        tr.translation = f.home
            + Vec3::new(
                2.2 * (t * 0.23 + p).sin() + 0.8 * (t * 0.71 + p * 1.3).sin(),
                0.6 * (t * 0.37 + p * 0.7).sin(),
                2.2 * (t * 0.19 + p * 1.9).cos() + 0.8 * (t * 0.53 + p).cos(),
            );
        // Nhấp nháy: mỗi con một nhịp.
        let blink = ((t * (0.6 + f.phase * 0.5) + p).sin() * 0.5 + 0.5).powf(3.0);
        tr.scale = Vec3::splat(0.02 + 0.13 * blink);
    }
}

// ───────────────────────── Dựng ─────────────────────────

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mats: Res<Mats>,
    spots: Res<Spots>,
) {
    let mut rng = Rng::new(4242);
    // Khói: các bậc độ trong suốt dùng chung.
    let smoke: Vec<Handle<StandardMaterial>> = (0..6)
        .map(|_| {
            materials.add(StandardMaterial {
                base_color: Color::srgba(0.85, 0.85, 0.85, 0.3),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 1.0,
                ..default()
            })
        })
        .collect();
    let mut pg = Geo::default();
    pg.blob(Vec3::ZERO, Vec3::splat(1.0), Color::WHITE, 0.2, &mut rng);
    let puff = meshes.add(pg.mesh());
    for &c in &spots.chimneys {
        for k in 0..9 {
            commands.spawn((
                Puff { origin: c, phase: k as f32 / 9.0 + rng.range(-0.03, 0.03) },
                Mesh3d(puff.clone()),
                MeshMaterial3d(smoke[0].clone()),
                Transform::from_translation(c),
                Visibility::default(),
                NotShadowCaster,
            ));
        }
    }

    // Cò, thuyền, người, trâu, diều.
    let bird_mat = materials.add(StandardMaterial { perceptual_roughness: 0.9, cull_mode: None, double_sided: true, ..default() });
    spawn_birds(&mut commands, &mut meshes, &bird_mat, &mut rng);
    spawn_boat(&mut commands, &mut meshes, &mats, &mats.glow, &mut rng);
    spawn_walkers(&mut commands, &mut meshes, &mats.vc, &mut rng);
    for (home, rider) in [(Vec2::new(-62.0, 42.0), true), (Vec2::new(-48.0, 58.0), false), (Vec2::new(70.0, -2.0), false)] {
        commands.spawn((
            Buffalo { home, phase: rng.range(0.0, TAU) },
            Mesh3d(meshes.add(buffalo_geo(&mut rng, rider).mesh())),
            MeshMaterial3d(mats.vc.clone()),
            Transform::default(),
        ));
    }
    spawn_kite(&mut commands, &mut meshes, &mats, &mut rng);

    // Đom đóm quanh ao, ruộng và lũy tre.
    let firefly = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(5.0, 7.0, 1.6, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let mut fg = Geo::default();
    fg.blob(Vec3::ZERO, Vec3::splat(1.0), Color::WHITE, 0.0, &mut rng);
    let fm = meshes.add(fg.mesh());
    for k in 0..160 {
        let home2 = match k % 4 {
            0 | 1 => POND + Vec2::from_angle(rng.range(0.0, TAU)) * rng.range(0.0, POND_R + 6.0),
            2 => Vec2::new(rng.range(-80.0, 80.0), rng.range(30.0, 85.0)),
            _ => {
                let a = rng.range(0.0, TAU);
                world::VILLAGE + Vec2::new(a.cos(), a.sin()) * world::VILLAGE_R * rng.range(0.95, 1.2)
            }
        };
        let y = height(home2.x, home2.y).max(WATER_Y) + rng.range(0.5, 2.2);
        commands.spawn((
            Firefly { home: Vec3::new(home2.x, y, home2.y), phase: rng.f() },
            Mesh3d(fm.clone()),
            MeshMaterial3d(firefly.clone()),
            Transform::from_xyz(home2.x, y, home2.y).with_scale(Vec3::splat(0.1)),
            Visibility::Hidden,
            NotShadowCaster,
        ));
    }
    let _ = RIVER_W;
    commands.insert_resource(LifeMats { smoke, firefly });
}
