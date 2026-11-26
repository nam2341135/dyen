//! Thời gian trong ngày: bầu trời, mặt trời, trăng, sao, mây, ánh sáng, sương.

use bevy::light::{CascadeShadowConfigBuilder, GlobalAmbientLight};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use crate::geom::{Geo, Rng, hex, mix, smoothstep};

/// Đồng hồ cảnh: giờ trong ngày [0, 24).
#[derive(Resource)]
pub struct Clock {
    pub hour: f32,
    /// Giờ cảnh trôi trên một giây thật.
    pub rate: f32,
    pub paused: bool,
    pub fast: bool,
    /// Thời gian thật (giây) để chạy hoạt cảnh.
    pub t: f32,
}

impl Default for Clock {
    fn default() -> Self {
        Self { hour: 15.6, rate: 24.0 / 180.0, paused: false, fast: false, t: 0.0 }
    }
}

/// Môi trường tính ra mỗi khung, dùng chung cho các hệ khác.
#[derive(Resource, Default)]
pub struct Env {
    pub sun_dir: Vec3,
    pub moon_dir: Vec3,
    /// 0 ban ngày → 1 ban đêm.
    pub night: f32,
    pub zenith: Color,
    pub horizon: Color,
    pub fog: Color,
    pub sun_col: Color,
    pub water: Color,
    /// Sức gió [0.4, 1.3], đổi chậm theo đợt.
    pub gust: f32,
}

struct Key {
    h: f32,
    zenith: u32,
    horizon: u32,
    fog: u32,
    light: u32,
    lux: f32,
    ambient: u32,
    amb: f32,
    water: u32,
    cloud: u32,
}

#[rustfmt::skip]
const KEYS: [Key; 11] = [
    Key { h: 0.0,  zenith: 0x070b1e, horizon: 0x1a2546, fog: 0x18223c, light: 0x9fb4ea, lux: 1600.0,  ambient: 0x4a5c98, amb: 330.0, water: 0x1a2844, cloud: 0x2a3352 },
    Key { h: 4.6,  zenith: 0x0d1430, horizon: 0x2a3358, fog: 0x232c4a, light: 0x9fb4ea, lux: 1400.0,  ambient: 0x4a5c98, amb: 310.0, water: 0x141e36, cloud: 0x2f3858 },
    Key { h: 5.6,  zenith: 0x2a3a6e, horizon: 0xc98a8a, fog: 0x8a7a8c, light: 0xff9a6a, lux: 900.0,   ambient: 0x6a6a90, amb: 220.0, water: 0x2c3550, cloud: 0xd99a9a },
    Key { h: 6.6,  zenith: 0x5378b8, horizon: 0xf5b98a, fog: 0xd9b49a, light: 0xffc28a, lux: 4500.0,  ambient: 0x9a9ab0, amb: 380.0, water: 0x3d5a6e, cloud: 0xffd2b0 },
    Key { h: 9.0,  zenith: 0x4f8fdd, horizon: 0xcfe3f2, fog: 0xbcd3e4, light: 0xfff1da, lux: 10000.0, ambient: 0xb8cce0, amb: 520.0, water: 0x3f6f7a, cloud: 0xffffff },
    Key { h: 13.0, zenith: 0x4a88d8, horizon: 0xd8e9f5, fog: 0xc6dcec, light: 0xfffaf0, lux: 11000.0, ambient: 0xbcd0e4, amb: 560.0, water: 0x3e7280, cloud: 0xffffff },
    Key { h: 16.2, zenith: 0x5a8acc, horizon: 0xf0dcc0, fog: 0xe2d2b8, light: 0xffe2b0, lux: 8500.0,  ambient: 0xc8c0b0, amb: 480.0, water: 0x46707a, cloud: 0xfff1de },
    Key { h: 17.4, zenith: 0x6578b4, horizon: 0xffbe78, fog: 0xf0b48a, light: 0xffb868, lux: 5200.0,  ambient: 0xc0a898, amb: 400.0, water: 0x56687a, cloud: 0xffcf9a },
    Key { h: 18.2, zenith: 0x40488a, horizon: 0xff8656, fog: 0xd88a78, light: 0xff7840, lux: 1800.0,  ambient: 0x9a7890, amb: 380.0, water: 0x4a4a6a, cloud: 0xff9a78 },
    Key { h: 19.1, zenith: 0x1e2554, horizon: 0x7a5478, fog: 0x4e4466, light: 0x9aa8e0, lux: 1200.0,  ambient: 0x565c92, amb: 300.0, water: 0x1e2442, cloud: 0x6a5478 },
    Key { h: 20.4, zenith: 0x080d22, horizon: 0x1c2748, fog: 0x18223c, light: 0x9fb4ea, lux: 1600.0,  ambient: 0x4a5c98, amb: 330.0, water: 0x1a2844, cloud: 0x2a3352 },
];

fn lerp_c(a: u32, b: u32, t: f32) -> Color {
    mix(hex(a), hex(b), t)
}

/// Bảng màu tại giờ `h`.
struct Pal {
    zenith: Color,
    horizon: Color,
    fog: Color,
    light: Color,
    lux: f32,
    ambient: Color,
    amb: f32,
    water: Color,
    cloud: Color,
}

fn palette(h: f32) -> Pal {
    let h = h.rem_euclid(24.0);
    let n = KEYS.len();
    let mut i = n - 1;
    for (k, key) in KEYS.iter().enumerate() {
        if key.h <= h {
            i = k;
        }
    }
    let a = &KEYS[i];
    let (b, span) = if i + 1 < n {
        (&KEYS[i + 1], KEYS[i + 1].h - a.h)
    } else {
        (&KEYS[0], 24.0 - a.h)
    };
    let t = ((h - a.h) / span).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    Pal {
        zenith: lerp_c(a.zenith, b.zenith, t),
        horizon: lerp_c(a.horizon, b.horizon, t),
        fog: lerp_c(a.fog, b.fog, t),
        light: lerp_c(a.light, b.light, t),
        lux: a.lux + (b.lux - a.lux) * t,
        ambient: lerp_c(a.ambient, b.ambient, t),
        amb: a.amb + (b.amb - a.amb) * t,
        water: lerp_c(a.water, b.water, t),
        cloud: lerp_c(a.cloud, b.cloud, t),
    }
}

/// Hướng tới mặt trời: mọc sau lưng máy quay (+z), lặn phía trước (−z), đi qua phía +x.
pub fn sun_dir(h: f32) -> Vec3 {
    let a = (h - 6.0) / 12.0 * std::f32::consts::PI;
    Vec3::new(0.42 * a.sin(), 0.82 * a.sin() - 0.02, a.cos()).normalize()
}

#[derive(Component)]
pub struct SkyDome {
    dirs: Vec<Vec3>,
}

#[derive(Component)]
pub struct SunDisc;
#[derive(Component)]
pub struct MoonDisc;
#[derive(Component)]
pub struct Cloud {
    speed: f32,
}
#[derive(Component)]
pub struct MainLight;

#[derive(Resource)]
pub struct SkyMats {
    pub stars: Handle<StandardMaterial>,
    pub cloud: Handle<StandardMaterial>,
    pub sun: Handle<StandardMaterial>,
    pub sun_halo: Handle<StandardMaterial>,
    pub moon: Handle<StandardMaterial>,
    pub moon_halo: Handle<StandardMaterial>,
}

const DOME_R: f32 = 1800.0;

fn unlit(c: Color, blend: bool) -> StandardMaterial {
    StandardMaterial {
        base_color: c,
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        alpha_mode: if blend { AlphaMode::Blend } else { AlphaMode::Opaque },
        ..default()
    }
}

fn disc(r: f32, seg: usize, col: Color) -> Geo {
    let mut g = Geo::default();
    for i in 0..seg {
        let a0 = i as f32 / seg as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / seg as f32 * std::f32::consts::TAU;
        g.tri(
            Vec3::ZERO,
            Vec3::new(a0.cos() * r, a0.sin() * r, 0.0),
            Vec3::new(a1.cos() * r, a1.sin() * r, 0.0),
            col,
        );
    }
    g
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Vòm trời: lưới kinh–vĩ, màu đỉnh tính lại mỗi khung.
    let (nu, nv) = (48, 22);
    let mut g = Geo::default();
    let mut dirs = Vec::new();
    let d = |i: usize, j: usize| {
        let az = i as f32 / nu as f32 * std::f32::consts::TAU;
        let el = -0.35 + (j as f32 / nv as f32) * (std::f32::consts::FRAC_PI_2 + 0.35);
        Vec3::new(az.cos() * el.cos(), el.sin(), az.sin() * el.cos())
    };
    for j in 0..nv {
        for i in 0..nu {
            let (a, b, c, e) = (d(i, j), d(i + 1, j), d(i + 1, j + 1), d(i, j + 1));
            for p in [a, b, c, a, c, e] {
                dirs.push(p);
            }
            g.tri(a * DOME_R, b * DOME_R, c * DOME_R, Color::WHITE);
            g.tri(a * DOME_R, c * DOME_R, e * DOME_R, Color::WHITE);
        }
    }
    commands.spawn((
        SkyDome { dirs },
        Mesh3d(meshes.add(g.mesh())),
        MeshMaterial3d(materials.add(unlit(Color::WHITE, false))),
        Transform::default(),
        bevy::light::NotShadowCaster,
    ));

    // Sao.
    let mut rng = Rng::new(77);
    let mut s = Geo::default();
    for _ in 0..700 {
        let az = rng.range(0.0, std::f32::consts::TAU);
        let el = rng.f().powf(0.7) * 1.45 + 0.06;
        let dir = Vec3::new(az.cos() * el.cos(), el.sin(), az.sin() * el.cos());
        let size = rng.range(1.5, 4.5) * if rng.f() < 0.05 { 2.2 } else { 1.0 };
        let b = rng.range(0.6, 2.4);
        let tint = mix(hex(0xbfd0ff), hex(0xfff0d0), rng.f());
        let col = Color::linear_rgb(
            tint.to_linear().red * b,
            tint.to_linear().green * b,
            tint.to_linear().blue * b,
        );
        let c = dir * (DOME_R - 60.0);
        let u = dir.any_orthonormal_vector() * size;
        let v = dir.cross(u);
        s.tri2(c - u - v, c + u - v, c + u + v, col);
        s.tri2(c - u - v, c + u + v, c - u + v, col);
    }
    let stars = materials.add(unlit(Color::srgba(1.0, 1.0, 1.0, 0.0), true));
    commands.spawn((
        Mesh3d(meshes.add(s.mesh())),
        MeshMaterial3d(stars.clone()),
        Transform::default(),
        bevy::light::NotShadowCaster,
    ));

    // Mặt trời, trăng (đĩa luôn quay về máy quay) và quầng.
    let sun = materials.add(unlit(Color::linear_rgb(30.0, 18.0, 8.0), false));
    let sun_halo = materials.add(unlit(Color::linear_rgba(4.0, 2.0, 1.0, 0.18), true));
    let moon = materials.add(unlit(Color::linear_rgb(3.0, 3.0, 2.6), false));
    let moon_halo = materials.add(unlit(Color::linear_rgba(0.6, 0.7, 1.0, 0.12), true));
    let disc_m = meshes.add(disc(1.0, 24, Color::WHITE).mesh());
    commands
        .spawn((SunDisc, Transform::default(), Visibility::default()))
        .with_children(|p| {
            p.spawn((Mesh3d(disc_m.clone()), MeshMaterial3d(sun.clone()), Transform::from_scale(Vec3::splat(38.0))));
            p.spawn((
                Mesh3d(disc_m.clone()),
                MeshMaterial3d(sun_halo.clone()),
                Transform::from_xyz(0.0, 0.0, -2.0).with_scale(Vec3::splat(110.0)),
            ));
        });
    commands
        .spawn((MoonDisc, Transform::default(), Visibility::default()))
        .with_children(|p| {
            p.spawn((Mesh3d(disc_m.clone()), MeshMaterial3d(moon.clone()), Transform::from_scale(Vec3::splat(26.0))));
            p.spawn((
                Mesh3d(disc_m.clone()),
                MeshMaterial3d(moon_halo.clone()),
                Transform::from_xyz(0.0, 0.0, -2.0).with_scale(Vec3::splat(90.0)),
            ));
        });

    // Mây: cụm khối dẹt, trôi chậm.
    let cloud = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        fog_enabled: false,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let mut rng = Rng::new(9);
    for k in 0..18 {
        let mut c = Geo::default();
        let n = 4 + (rng.u() % 5) as usize;
        let len = rng.range(40.0, 110.0);
        for i in 0..n {
            let x = (i as f32 / n as f32 - 0.5) * len + rng.range(-8.0, 8.0);
            let r = rng.range(14.0, 30.0) * (1.0 - (x / len).abs());
            let top = hex(0xffffff);
            let bottom = hex(0xc8cfe0);
            let mut b = Geo::default();
            b.blob(Vec3::new(x, r * 0.25, rng.range(-10.0, 10.0)), Vec3::new(r * 1.3, r * 0.55, r), top, 0.12, &mut rng);
            // Đáy mây tối hơn.
            for (p, col) in b.pos.iter().zip(b.col.iter_mut()) {
                let t = smoothstep(-r * 0.2, r * 0.6, p[1]);
                let m = mix(bottom, top, t).to_linear();
                col[0] *= m.red;
                col[1] *= m.green;
                col[2] *= m.blue;
            }
            c.add(&b, Transform::IDENTITY);
        }
        let a = k as f32 / 18.0 * std::f32::consts::TAU + rng.range(-0.2, 0.2);
        let r = rng.range(380.0, 900.0);
        let y = rng.range(140.0, 260.0);
        commands.spawn((
            Cloud { speed: rng.range(1.2, 3.0) },
            Mesh3d(meshes.add(c.mesh())),
            MeshMaterial3d(cloud.clone()),
            Transform::from_xyz(a.cos() * r, y, a.sin() * r).with_rotation(Quat::from_rotation_y(-a)),
            bevy::light::NotShadowCaster,
        ));
    }

    commands.insert_resource(SkyMats { stars, cloud, sun, sun_halo, moon, moon_halo });

    // Đèn chính (mặt trời hoặc trăng), có bóng.
    commands.spawn((
        MainLight,
        DirectionalLight { illuminance: 8000.0, shadow_maps_enabled: true, ..default() },
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            first_cascade_far_bound: 45.0,
            maximum_distance: 260.0,
            ..default()
        }
        .build(),
        Transform::default(),
    ));
    commands.insert_resource(GlobalAmbientLight { color: Color::WHITE, brightness: 300.0, ..default() });
}

/// Tiến đồng hồ.
pub fn tick(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut clock: ResMut<Clock>) {
    let dt = time.delta_secs();
    clock.t += dt;
    let fast = clock.fast || keys.pressed(KeyCode::KeyT);
    if !clock.paused {
        let k = if fast { 14.0 } else { 1.0 };
        clock.hour = (clock.hour + dt * clock.rate * k).rem_euclid(24.0);
    }
}

#[allow(clippy::too_many_arguments)] // hệ Bevy: mỗi tham số là một truy vấn
pub fn update(
    clock: Res<Clock>,
    mut env: ResMut<Env>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mats: Res<SkyMats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    dome: Query<(&SkyDome, &Mesh3d)>,
    mut light: Query<(&mut DirectionalLight, &mut Transform), With<MainLight>>,
    mut discs: Query<(&mut Transform, Option<&SunDisc>), (Or<(With<SunDisc>, With<MoonDisc>)>, Without<MainLight>, Without<Camera3d>)>,
    mut clouds: Query<(&Cloud, &mut Transform), (Without<SunDisc>, Without<MoonDisc>, Without<MainLight>, Without<Camera3d>)>,
    mut cams: Query<(&Transform, &mut DistanceFog), (With<Camera3d>, Without<MainLight>)>,
    time: Res<Time>,
) {
    let h = clock.hour;
    let p = palette(h);
    let sun = sun_dir(h);
    let moon = (-sun + Vec3::new(0.25, 0.0, 0.0)).normalize();
    let night = smoothstep(0.05, -0.18, sun.y);
    env.sun_dir = sun;
    env.moon_dir = moon;
    env.night = night;
    env.zenith = p.zenith;
    env.horizon = p.horizon;
    env.fog = p.fog;
    env.water = p.water;
    let t = clock.t;
    env.gust = 0.85 + 0.3 * (t * 0.23).sin() + 0.15 * (t * 0.61 + 1.3).sin();

    // Vòm trời.
    let sun_glow = mix(p.light, p.horizon, 0.3).to_linear();
    let low_sun = smoothstep(0.55, 0.0, sun.y) * (1.0 - night);
    env.sun_col = p.light;
    if let Ok((sky, m)) = dome.single()
        && let Some(mut mesh) = meshes.get_mut(&m.0)
    {
        let (z, hz, fg) = (p.zenith.to_linear(), p.horizon.to_linear(), p.fog.to_linear());
        let cols: Vec<[f32; 4]> = sky
            .dirs
            .iter()
            .map(|d| {
                let up = d.y.max(0.0);
                let t = up.powf(0.45);
                let mut c = Vec3::new(hz.red, hz.green, hz.blue).lerp(Vec3::new(z.red, z.green, z.blue), t);
                if d.y < 0.0 {
                    c = c.lerp(Vec3::new(fg.red, fg.green, fg.blue), smoothstep(0.0, -0.15, d.y));
                }
                let s = d.dot(sun).max(0.0);
                let glow = s.powf(6.0) * 0.55 * low_sun + s.powf(60.0) * 0.8 * (1.0 - night);
                c += Vec3::new(sun_glow.red, sun_glow.green, sun_glow.blue) * glow;
                [c.x, c.y, c.z, 1.0]
            })
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, cols);
    }

    // Đèn chính: mặt trời ban ngày, trăng ban đêm.
    if let Ok((mut l, mut tr)) = light.single_mut() {
        let dir = if sun.y > -0.04 { sun } else { moon };
        // Tránh bóng quá dài khi mặt trời sát chân trời.
        let dir = Vec3::new(dir.x, dir.y.max(0.07), dir.z).normalize();
        *tr = Transform::from_translation(dir * 100.0).looking_at(Vec3::ZERO, Vec3::Y);
        l.color = p.light;
        l.illuminance = p.lux;
    }
    ambient.color = p.ambient;
    ambient.brightness = p.amb;

    // Đĩa mặt trời/trăng, quay về phía máy quay.
    let cam_pos = cams.iter().next().map(|(t, _)| t.translation).unwrap_or(Vec3::ZERO);
    for (mut tr, is_sun) in &mut discs {
        let d = if is_sun.is_some() { sun } else { moon };
        tr.translation = cam_pos + d * (DOME_R - 150.0);
        tr.look_at(cam_pos, Vec3::Y);
    }
    if let Some(mut m) = materials.get_mut(&mats.sun) {
        let k = 1.0 - smoothstep(-0.02, -0.08, sun.y);
        let warm = mix(hex(0xfff3d8), hex(0xff8a40), low_sun).to_linear();
        m.base_color = Color::linear_rgb(warm.red * 28.0 * k, warm.green * 28.0 * k, warm.blue * 28.0 * k);
    }
    if let Some(mut m) = materials.get_mut(&mats.sun_halo) {
        let warm = mix(hex(0xfff3d8), hex(0xff7a40), low_sun).to_linear();
        let a = 0.22 * (1.0 - night);
        m.base_color = Color::linear_rgba(warm.red * 3.0, warm.green * 3.0, warm.blue * 3.0, a);
    }
    for (h, a) in [(&mats.moon, 1.0), (&mats.moon_halo, 0.14)] {
        if let Some(mut m) = materials.get_mut(h) {
            let c = m.base_color.to_linear();
            m.base_color = Color::linear_rgba(c.red, c.green, c.blue, a * night);
            m.alpha_mode = AlphaMode::Blend;
        }
    }
    if let Some(mut m) = materials.get_mut(&mats.stars) {
        m.base_color = Color::srgba(1.0, 1.0, 1.0, night * 0.95);
    }
    if let Some(mut m) = materials.get_mut(&mats.cloud) {
        let c = p.cloud.to_linear();
        let k = 1.0 + 0.4 * low_sun;
        m.base_color = Color::linear_rgba(c.red * k, c.green * k, c.blue * k, 0.92);
    }

    // Mây trôi quanh trục y.
    let dt = time.delta_secs();
    for (c, mut tr) in &mut clouds {
        let ang = c.speed * dt / tr.translation.xz().length().max(1.0);
        tr.rotate_around(Vec3::ZERO, Quat::from_rotation_y(ang));
    }

    // Sương theo khoảng cách, tán sáng về phía mặt trời.
    for (_, mut fog) in &mut cams {
        fog.color = p.fog;
        fog.directional_light_color = mix(Color::NONE, mix(p.light, p.horizon, 0.4).with_alpha(0.55), low_sun.max(0.15) * (1.0 - night));
        fog.directional_light_exponent = 18.0;
        // Mật độ mũ: ~45% ở lớp núi đầu (300), ~75% ở lớp cuối (640): các lớp nhạt dần.
        let density = 0.0011 + 0.0007 * low_sun + 0.0004 * night;
        fog.falloff = FogFalloff::Exponential { density };
    }
}
