//! Làng: đình, nhà ba gian, cổng làng, cầu tre, giếng, cây đa, tre, cau, chuối, lúa, sen, lau.

use std::f32::consts::{PI, TAU};

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::geom::{Geo, Rng, hex, mix, shade};
use crate::world::{self, BRIDGE_X, PLOT, POND, POND_R, RIVER_W, VILLAGE, WATER_Y, height, river_z};

/// Lay theo gió: xoay quanh gốc của thực thể.
#[derive(Component)]
pub struct Sway {
    pub amp: f32,
    pub freq: f32,
    pub phase: f32,
    pub base: Quat,
}

/// Lay theo gió từng đỉnh (lúa): `w` là trọng số theo chiều cao.
#[derive(Component)]
pub struct VertexSway {
    pub base: Vec<Vec3>,
    pub w: Vec<f32>,
    pub amp: f32,
}

/// Đèn điểm chỉ sáng về đêm.
#[derive(Component)]
pub struct NightLamp {
    pub max: f32,
}

#[derive(Resource)]
pub struct Mats {
    /// Màu đỉnh, đục.
    pub vc: Handle<StandardMaterial>,
    /// Màu đỉnh, hai mặt (lá, lúa, cờ).
    pub plant: Handle<StandardMaterial>,
    /// Phát sáng về đêm (đèn lồng, cửa).
    pub glow: Handle<StandardMaterial>,
}

/// Vị trí ống khói bếp (để nhả khói), vị trí đèn lồng thuyền...
#[derive(Resource, Default)]
pub struct Spots {
    pub chimneys: Vec<Vec3>,
}

// ───────────────────────── Mái ─────────────────────────

/// Mái cong: `hip` = mái chái bốn mặt (đình), ngược lại mái hai dốc. Gốc ở cao độ diềm mái.
#[allow(clippy::too_many_arguments)] // spike: tham số hình học tường minh
fn roof(g: &mut Geo, t: Transform, a: f32, b: f32, h: f32, lift: f32, hip: bool, tile: (Color, Color), under: Color) {
    let (nu, nv) = (30, 18);
    let y = |u: f32, v: f32| -> f32 {
        let (au, av) = (u.abs(), v.abs());
        let f = if hip { ((b - av) / b).min((a - au) / b) } else { (b - av) / b }.clamp(0.0, 1.0);
        let mut y = h * f.powf(1.5);
        if hip {
            y += lift * ((au / a) * (av / b)).powi(3) * (1.0 - f).powf(1.5);
        } else {
            y += lift * (au / a).powi(8) * (0.3 + 0.7 * f);
        }
        y
    };
    let fval = |u: f32, v: f32| -> f32 {
        if hip { ((b - v.abs()) / b).min((a - u.abs()) / b) } else { (b - v.abs()) / b }.clamp(0.0, 1.0)
    };
    let m = t.to_matrix();
    let p = |u: f32, v: f32, dy: f32| m.transform_point3(Vec3::new(u, y(u, v) + dy, v));
    for j in 0..nv {
        for i in 0..nu {
            let (u0, u1) = (-a + 2.0 * a * i as f32 / nu as f32, -a + 2.0 * a * (i + 1) as f32 / nu as f32);
            let (v0, v1) = (-b + 2.0 * b * j as f32 / nv as f32, -b + 2.0 * b * (j + 1) as f32 / nv as f32);
            let (cu, cv) = ((u0 + u1) * 0.5, (v0 + v1) * 0.5);
            let f = fval(cu, cv);
            // Hàng ngói song song diềm mái.
            let row = ((1.0 - f) * 16.0) as i32;
            let mut c = if row % 2 == 0 { tile.0 } else { tile.1 };
            if f < 0.06 {
                c = shade(c, 0.75);
            }
            let (a0, a1, b1, b0) = (p(u0, v1, 0.0), p(u1, v1, 0.0), p(u1, v0, 0.0), p(u0, v0, 0.0));
            g.quad(a0, a1, b1, b0, c);
            let (a0, a1, b1, b0) = (p(u0, v1, -0.22), p(u1, v1, -0.22), p(u1, v0, -0.22), p(u0, v0, -0.22));
            g.quad(a0, b0, b1, a1, under);
        }
    }
    // Diềm mái: dải đứng quanh mép.
    let edge = shade(tile.1, 0.6);
    for k in 0..nu {
        for &v in &[-b, b] {
            let (u0, u1) = (-a + 2.0 * a * k as f32 / nu as f32, -a + 2.0 * a * (k + 1) as f32 / nu as f32);
            g.quad2(p(u0, v, 0.0), p(u1, v, 0.0), p(u1, v, -0.22), p(u0, v, -0.22), edge);
        }
    }
    for k in 0..nv {
        for &u in &[-a, a] {
            let (v0, v1) = (-b + 2.0 * b * k as f32 / nv as f32, -b + 2.0 * b * (k + 1) as f32 / nv as f32);
            g.quad2(p(u, v0, 0.0), p(u, v1, 0.0), p(u, v1, -0.22), p(u, v0, -0.22), edge);
        }
    }
    // Bờ nóc.
    let ridge = shade(tile.1, 0.8);
    let rl = if hip { a - b } else { a };
    let steps = 12;
    for k in 0..steps {
        let u0 = -rl + 2.0 * rl * k as f32 / steps as f32;
        let u1 = -rl + 2.0 * rl * (k + 1) as f32 / steps as f32;
        g.beam(p(u0, 0.0, 0.12), p(u1, 0.0, 0.12), 0.2, ridge);
    }
    // Đầu kìm / đầu đao cong vút.
    let curl = |g: &mut Geo, base: Vec3, out: Vec3, s: f32| {
        let pts = [
            base,
            base + out * 0.5 * s + Vec3::Y * 0.25 * s,
            base + out * 0.9 * s + Vec3::Y * 0.8 * s,
            base + out * 0.95 * s + Vec3::Y * 1.45 * s,
            base + out * 0.7 * s + Vec3::Y * 1.85 * s,
        ];
        for w in pts.windows(2) {
            g.beam(w[0], w[1], 0.13 * s.sqrt(), ridge);
        }
    };
    if hip {
        // Bờ chảy theo bốn đường chéo, kết thúc bằng đầu đao.
        for (sx, sz) in [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
            let n = 10;
            let mut prev = p(sx * rl, 0.0, 0.1);
            for k in 1..=n {
                let s = k as f32 / n as f32;
                let (u, v) = (sx * (rl + b * s), sz * b * s);
                let q = p(u, v, 0.1);
                g.beam(prev, q, 0.15, ridge);
                prev = q;
            }
            let out = t.rotation * Vec3::new(sx, 0.0, sz).normalize();
            curl(g, prev, out, 1.3);
        }
        curl(g, p(rl, 0.0, 0.1), t.rotation * Vec3::X, 0.9);
        curl(g, p(-rl, 0.0, 0.1), t.rotation * Vec3::NEG_X, 0.9);
    } else {
        curl(g, p(rl, 0.0, 0.1), t.rotation * Vec3::X, 0.55);
        curl(g, p(-rl, 0.0, 0.1), t.rotation * Vec3::NEG_X, 0.55);
    }
}

fn place(x: f32, z: f32, rot: f32) -> Transform {
    Transform::from_xyz(x, height(x, z), z).with_rotation(Quat::from_rotation_y(rot))
}

// ───────────────────────── Công trình ─────────────────────────

const WOOD: u32 = 0x6a4630;
const LACQUER: u32 = 0x7c2e1c;
const STONE: u32 = 0x8f8778;

fn lantern(g: &mut Geo, c: Vec3, s: f32, rng: &mut Rng) {
    g.blob(c, Vec3::new(0.42, 0.55, 0.42) * s, hex(0xd8352a), 0.02, rng);
    g.tube(c + Vec3::Y * 0.5 * s, c + Vec3::Y * 0.62 * s, 0.25 * s, 0.25 * s, 6, hex(0xd8a640), hex(0xd8a640), true);
    g.tube(c - Vec3::Y * 0.62 * s, c - Vec3::Y * 0.5 * s, 0.25 * s, 0.25 * s, 6, hex(0xd8a640), hex(0xd8a640), true);
    g.beam(c - Vec3::Y * 0.62 * s, c - Vec3::Y * 1.1 * s, 0.04 * s, hex(0xe0b040));
}

/// Đình làng tại gốc, mặt hướng +z. Trả về (khối tĩnh, khối phát sáng, vị trí đèn).
fn dinh(rng: &mut Rng) -> (Geo, Geo, Vec<Vec3>) {
    let mut g = Geo::default();
    let mut glow = Geo::default();
    let mut lamps = Vec::new();
    // Nền đá, bậc.
    g.bxa(Vec3::new(0.0, 0.45, 0.0), Vec3::new(12.5, 0.45, 8.0), hex(STONE));
    g.bxa(Vec3::new(0.0, 0.22, 8.7), Vec3::new(5.0, 0.22, 0.8), shade(hex(STONE), 0.95));
    // Sân gạch.
    for i in -10..10 {
        for j in 0..10 {
            let c = if (i + j) % 2 == 0 { hex(0xa85b3e) } else { hex(0x9a5236) };
            g.bxa(
                Vec3::new(i as f32 * 1.1 + 0.55, 0.05, 9.6 + j as f32 * 1.1 + 0.55),
                Vec3::new(0.55, 0.05, 0.55),
                shade(c, 0.95 + 0.1 * rng.f()),
            );
        }
    }
    // Cột.
    for &x in &[-10.0f32, -6.0, -2.0, 2.0, 6.0, 10.0] {
        for &z in &[-5.5f32, -2.0, 2.0, 5.5] {
            let inner = x.abs() < 7.0 && z.abs() < 3.0;
            let top = if inner { 6.6 } else { 4.5 };
            g.tube(Vec3::new(x, 0.9, z), Vec3::new(x, top, z), 0.34, 0.3, 8, hex(LACQUER), shade(hex(LACQUER), 1.1), true);
            g.tube(Vec3::new(x, 0.9, z), Vec3::new(x, 1.15, z), 0.48, 0.48, 8, hex(STONE), hex(STONE), true);
        }
    }
    // Vách sau và hồi.
    g.bxa(Vec3::new(0.0, 2.75, -5.6), Vec3::new(10.2, 1.85, 0.12), hex(WOOD));
    g.bxa(Vec3::new(-10.0, 2.75, 0.0), Vec3::new(0.12, 1.85, 5.6), hex(WOOD));
    g.bxa(Vec3::new(10.0, 2.75, 0.0), Vec3::new(0.12, 1.85, 5.6), hex(WOOD));
    // Cửa bức bàn mặt trước: ô song, giữa để mở (ánh đèn bên trong).
    for k in 0..5 {
        let x = -8.0 + 4.0 * k as f32;
        if k == 2 {
            glow.quad(
                Vec3::new(x - 1.6, 1.0, 5.2),
                Vec3::new(x + 1.6, 1.0, 5.2),
                Vec3::new(x + 1.6, 4.1, 5.2),
                Vec3::new(x - 1.6, 4.1, 5.2),
                hex(0xffc070),
            );
            continue;
        }
        g.bxa(Vec3::new(x, 2.55, 5.45), Vec3::new(1.75, 1.6, 0.06), hex(0x4a2e20));
        for r in 0..5 {
            g.bxa(Vec3::new(x, 1.3 + r as f32 * 0.65, 5.53), Vec3::new(1.7, 0.04, 0.03), hex(0x8a5a38));
        }
        for c in 0..4 {
            g.bxa(Vec3::new(x - 1.2 + c as f32 * 0.8, 2.55, 5.53), Vec3::new(0.035, 1.55, 0.03), hex(0x8a5a38));
        }
    }
    // Hai tầng mái (chồng diêm).
    let tile = (hex(0x9c3c26), hex(0x86331f));
    roof(&mut g, Transform::from_xyz(0.0, 4.5, 0.0), 14.2, 9.4, 2.0, 1.5, true, tile, hex(0x5a3a26));
    g.bxa(Vec3::new(0.0, 5.6, 0.0), Vec3::new(8.2, 0.9, 3.9), hex(0x4a2e20));
    for k in 0..15 {
        let x = -7.0 + k as f32;
        for &z in &[3.95f32, -3.95] {
            g.bxa(Vec3::new(x, 5.6, z), Vec3::new(0.06, 0.75, 0.04), hex(0x9a6a40));
        }
    }
    roof(&mut g, Transform::from_xyz(0.0, 6.4, 0.0), 11.5, 6.6, 3.6, 1.9, true, tile, hex(0x5a3a26));
    // Lưỡng long chầu nguyệt trên nóc.
    let top = 6.4 + 3.6 + 0.3;
    let gold = hex(0xd9b048);
    g.tube(Vec3::new(0.0, top, -0.05), Vec3::new(0.0, top, 0.05), 0.0, 0.0, 3, gold, gold, false);
    let mut moon = Geo::default();
    for i in 0..14 {
        let a0 = i as f32 / 14.0 * TAU;
        let a1 = (i + 1) as f32 / 14.0 * TAU;
        moon.tri2(
            Vec3::new(0.0, 0.8, 0.0),
            Vec3::new(a0.cos() * 0.55, 0.8 + a0.sin() * 0.55, 0.0),
            Vec3::new(a1.cos() * 0.55, 0.8 + a1.sin() * 0.55, 0.0),
            gold,
        );
    }
    moon.blob(Vec3::new(0.0, 0.15, 0.0), Vec3::new(0.3, 0.3, 0.3), gold, 0.1, rng);
    g.add(&moon, Transform::from_xyz(0.0, top, 0.0));
    for s in [-1.0f32, 1.0] {
        let mut prev = Vec3::new(s * 4.6, top, 0.0);
        for k in 1..=14 {
            let u = k as f32 / 14.0;
            let x = s * (4.6 - 3.9 * u);
            let y = top + 0.15 + 0.45 * (u * PI * 2.5).sin().abs() + u * 0.5;
            let q = Vec3::new(x, y, 0.0);
            g.beam(prev, q, 0.16 * (1.0 - 0.5 * u) + 0.05, hex(0x7a8a5a));
            prev = q;
        }
        g.blob(prev + Vec3::Y * 0.1, Vec3::new(0.32, 0.28, 0.26), hex(0x7a8a5a), 0.15, rng);
    }
    // Đèn lồng dưới hiên.
    for &x in &[-9.0f32, -4.5, 4.5, 9.0] {
        let c = Vec3::new(x, 3.35, 7.6);
        g.beam(c + Vec3::Y * 0.5, c + Vec3::Y * 1.3, 0.03, hex(0x3a2a20));
        lantern(&mut glow, c, 1.0, rng);
        lamps.push(c);
    }
    // Cột nanh trước sân, đỉnh có đèn.
    for s in [-1.0f32, 1.0] {
        let x = s * 6.0;
        g.bxa(Vec3::new(x, 2.2, 20.0), Vec3::new(0.45, 2.2, 0.45), hex(0xb8ab92));
        g.bxa(Vec3::new(x, 4.55, 20.0), Vec3::new(0.62, 0.15, 0.62), hex(0x9a8f7a));
        g.tube(Vec3::new(x, 4.7, 20.0), Vec3::new(x, 5.4, 20.0), 0.35, 0.05, 6, hex(0x9a8f7a), hex(0x9a8f7a), false);
        glow.blob(Vec3::new(x, 4.05, 20.46), Vec3::new(0.22, 0.3, 0.05), hex(0xffcf80), 0.0, rng);
    }
    // Lư hương giữa sân.
    g.tube(Vec3::new(0.0, 0.1, 14.0), Vec3::new(0.0, 0.9, 14.0), 0.7, 0.9, 8, hex(0x4a4038), hex(0x6a5a48), true);
    g.tube(Vec3::new(0.0, 0.9, 14.0), Vec3::new(0.0, 1.5, 14.0), 0.5, 0.15, 8, hex(0x6a5a48), hex(0x8a7a5a), false);
    (g, glow, lamps)
}

/// Nhà ba gian tại gốc, mặt +z. `thatch`: mái rạ. Trả về (tĩnh, sáng, vị trí ống khói theo gốc).
fn house(rng: &mut Rng, thatch: bool) -> (Geo, Geo, Vec3) {
    let mut g = Geo::default();
    let mut glow = Geo::default();
    let wall = mix(hex(0xd8c9a8), hex(0xc8b48c), rng.f());
    g.bxa(Vec3::new(0.0, 0.25, 0.0), Vec3::new(5.4, 0.25, 3.3), hex(0x8a7f6a));
    g.bxa(Vec3::new(0.0, 1.55, -2.9), Vec3::new(5.0, 1.05, 0.15), wall);
    g.bxa(Vec3::new(-4.9, 1.55, 0.0), Vec3::new(0.15, 1.05, 3.0), wall);
    g.bxa(Vec3::new(4.9, 1.55, 0.0), Vec3::new(0.15, 1.05, 3.0), wall);
    // Hồi nhà (tam giác dưới mái).
    for s in [-1.0f32, 1.0] {
        g.tri2(Vec3::new(s * 4.9, 2.6, -3.0), Vec3::new(s * 4.9, 2.6, 3.0), Vec3::new(s * 4.9, 4.3, 0.0), wall);
    }
    // Mặt trước: cột + ba gian cửa.
    for k in 0..4 {
        let x = -4.6 + k as f32 * 3.07;
        g.tube(Vec3::new(x, 0.5, 2.6), Vec3::new(x, 2.65, 2.6), 0.16, 0.14, 6, hex(WOOD), hex(WOOD), true);
    }
    for k in 0..3 {
        let x = -3.07 + k as f32 * 3.07;
        if k == 1 {
            glow.quad(
                Vec3::new(x - 1.2, 0.55, 2.3),
                Vec3::new(x + 1.2, 0.55, 2.3),
                Vec3::new(x + 1.2, 2.35, 2.3),
                Vec3::new(x - 1.2, 2.35, 2.3),
                hex(0xffb860),
            );
            // Hai cánh cửa mở hé.
            for s in [-1.0f32, 1.0] {
                let hinge = Vec3::new(x + s * 1.25, 1.45, 2.35);
                let rot = Quat::from_rotation_y(s * 1.1);
                g.bx(
                    Transform::from_translation(hinge + rot * Vec3::new(-s * 0.6, 0.0, 0.0)).with_rotation(rot),
                    Vec3::new(0.6, 0.9, 0.04),
                    hex(0x5a3a24),
                );
            }
        } else {
            g.bxa(Vec3::new(x, 1.45, 2.4), Vec3::new(1.35, 0.92, 0.07), hex(0x5a3a24));
            for r in 0..4 {
                g.bxa(Vec3::new(x, 0.8 + r as f32 * 0.45, 2.48), Vec3::new(1.3, 0.03, 0.02), hex(0x8a6040));
            }
        }
    }
    let (tile, under) = if thatch {
        ((hex(0xb8975a), hex(0xa8864c)), hex(0x7a6440))
    } else {
        ((hex(0xa6492e), hex(0x934029)), hex(0x5a3a26))
    };
    roof(&mut g, Transform::from_xyz(0.0, 2.6, 0.0), 6.0, 3.9, 1.8, if thatch { 0.15 } else { 0.5 }, false, tile, under);
    // Sân gạch.
    for i in -5..5 {
        for j in 0..5 {
            let c = if (i + j) % 2 == 0 { hex(0xae6446) } else { hex(0xa05a3c) };
            g.bxa(
                Vec3::new(i as f32 + 0.5, 0.04, 3.9 + j as f32 + 0.5),
                Vec3::new(0.5, 0.04, 0.5),
                shade(c, 0.92 + 0.12 * rng.f()),
            );
        }
    }
    // Bếp bên hông, có ống khói.
    g.bxa(Vec3::new(7.4, 0.9, -0.8), Vec3::new(1.5, 0.9, 1.5), shade(wall, 0.9));
    roof(&mut g, Transform::from_xyz(7.4, 1.8, -0.8).with_rotation(Quat::from_rotation_y(PI / 2.0)), 1.9, 2.0, 1.0, 0.0, false, (hex(0xb8975a), hex(0xa8864c)), hex(0x7a6440));
    g.bxa(Vec3::new(8.2, 2.8, -1.3), Vec3::new(0.25, 0.6, 0.25), hex(0x8a7a6a));
    let chimney = Vec3::new(8.2, 3.5, -1.3);
    // Đống rơm, chum nước.
    let hay = hex(0xd2ad5c);
    g.tube(Vec3::new(-6.8, 0.0, 5.5), Vec3::new(-6.8, 1.6, 5.5), 1.1, 1.0, 8, shade(hay, 0.85), hay, false);
    g.tube(Vec3::new(-6.8, 1.6, 5.5), Vec3::new(-6.8, 2.9, 5.5), 1.0, 0.05, 8, hay, shade(hay, 1.1), false);
    g.beam(Vec3::new(-6.8, 2.6, 5.5), Vec3::new(-6.8, 3.4, 5.5), 0.05, hex(0x6a5030));
    for k in 0..3 {
        g.blob(Vec3::new(4.0 + k as f32 * 0.9, 0.45, 3.6), Vec3::new(0.42, 0.5, 0.42), hex(0x5e4030), 0.03, rng);
    }
    (g, glow, chimney)
}

/// Cổng làng: hai trụ, mái ngói nhỏ.
fn gate(rng: &mut Rng) -> (Geo, Geo) {
    let mut g = Geo::default();
    let mut glow = Geo::default();
    let c = hex(0xcabda0);
    for s in [-1.0f32, 1.0] {
        g.bxa(Vec3::new(s * 3.2, 2.0, 0.0), Vec3::new(0.7, 2.0, 0.7), c);
        g.bxa(Vec3::new(s * 4.8, 1.0, 0.0), Vec3::new(1.0, 1.0, 0.35), shade(c, 0.95));
        glow.blob(Vec3::new(s * 3.2, 3.0, 0.75), Vec3::new(0.25, 0.35, 0.05), hex(0xffcf80), 0.0, rng);
    }
    g.bxa(Vec3::new(0.0, 4.3, 0.0), Vec3::new(4.0, 0.35, 0.8), c);
    g.bxa(Vec3::new(0.0, 4.3, 0.82), Vec3::new(1.5, 0.28, 0.03), hex(0x7c2e1c));
    roof(&mut g, Transform::from_xyz(0.0, 4.7, 0.0), 4.6, 1.6, 1.0, 0.6, false, (hex(0x9c3c26), hex(0x86331f)), hex(0x5a3a26));
    (g, glow)
}

fn bridge(g: &mut Geo) {
    let x = BRIDGE_X;
    let zc = river_z(x);
    let half = RIVER_W + 4.5;
    let bam = hex(0xc4a868);
    let deck = |z: f32| {
        let s = (z - zc) / half;
        0.7 + 1.5 * (1.0 - s * s)
    };
    let n = 36;
    for k in 0..n {
        let z0 = zc - half + 2.0 * half * k as f32 / n as f32;
        let z1 = zc - half + 2.0 * half * (k + 1) as f32 / n as f32;
        let zm = (z0 + z1) * 0.5;
        let y = deck(zm);
        let ang = ((deck(z1) - deck(z0)) / (z1 - z0)).atan();
        g.bx(
            Transform::from_xyz(x, y, zm).with_rotation(Quat::from_rotation_x(-ang)),
            Vec3::new(1.3, 0.07, (z1 - z0) * 0.5 - 0.03),
            shade(bam, 0.92 + 0.08 * ((k % 3) as f32)),
        );
    }
    let mut z = zc - half + 1.0;
    while z < zc + half - 0.5 {
        let y = deck(z);
        for s in [-1.0f32, 1.0] {
            g.tube(Vec3::new(x + s * 1.2, -2.2, z), Vec3::new(x + s * 1.2, y + 1.05, z), 0.09, 0.08, 5, shade(bam, 0.8), bam, true);
        }
        g.beam(Vec3::new(x - 1.3, y - 0.2, z), Vec3::new(x + 1.3, y - 0.2, z), 0.06, shade(bam, 0.85));
        z += 2.6;
    }
    for s in [-1.0f32, 1.0] {
        let m = 20;
        for k in 0..m {
            let z0 = zc - half + 1.0 + (2.0 * half - 2.0) * k as f32 / m as f32;
            let z1 = zc - half + 1.0 + (2.0 * half - 2.0) * (k + 1) as f32 / m as f32;
            g.beam(Vec3::new(x + s * 1.2, deck(z0) + 1.0, z0), Vec3::new(x + s * 1.2, deck(z1) + 1.0, z1), 0.05, bam);
        }
    }
}

fn well(g: &mut Geo, at: Vec3) {
    let s = hex(0x9a9080);
    g.tube(at, at + Vec3::Y * 0.8, 1.3, 1.3, 10, shade(s, 0.9), s, false);
    g.tube(at + Vec3::Y * 0.8, at, 1.0, 1.0, 10, shade(s, 0.7), shade(s, 0.7), false);
    g.tube(at + Vec3::Y * 0.5, at + Vec3::Y * 0.5, 1.0, 0.0, 10, hex(0x1a2a30), hex(0x1a2a30), false);
    // Vành miệng giếng.
    for i in 0..10 {
        let a0 = i as f32 / 10.0 * TAU;
        let a1 = (i + 1) as f32 / 10.0 * TAU;
        let p = |a: f32, r: f32| at + Vec3::new(a.cos() * r, 0.8, a.sin() * r);
        g.quad(p(a0, 1.0), p(a0, 1.3), p(a1, 1.3), p(a1, 1.0), shade(s, 1.1));
    }
}

// ───────────────────────── Cây ─────────────────────────

fn leaf(g: &mut Geo, base: Vec3, dir: Vec3, len: f32, w: f32, col: Color) {
    let side = dir.cross(Vec3::Y).normalize_or(Vec3::X) * w;
    let tip = base + dir * len;
    let mid = base + dir * len * 0.45 - Vec3::Y * len * 0.08;
    g.tri2(base, mid + side, tip, col);
    g.tri2(base, tip, mid - side, shade(col, 0.88));
}

/// Bụi tre: nhiều thân cong ra ngoài, lá ở nửa trên.
fn bamboo(rng: &mut Rng) -> Geo {
    let mut g = Geo::default();
    let n = 9 + (rng.u() % 6) as usize;
    let greens = [hex(0x5a8a32), hex(0x6c9c3a), hex(0x7aaa42), hex(0x4c7a2e)];
    for _ in 0..n {
        let a = rng.range(0.0, TAU);
        let r0 = rng.range(0.1, 0.9);
        let mut p = Vec3::new(a.cos() * r0, -0.2, a.sin() * r0);
        let out = Vec3::new(a.cos(), 0.0, a.sin());
        let len = rng.range(9.0, 14.0);
        let segs = 9;
        let lean = rng.range(0.15, 0.45);
        let culm = mix(hex(0x9aab52), hex(0x7a9440), rng.f());
        for s in 0..segs {
            let u = s as f32 / segs as f32;
            let dir = (Vec3::Y + out * (lean * (0.3 + 1.6 * u * u))).normalize();
            let q = p + dir * (len / segs as f32);
            g.tube(p, q, 0.11 * (1.0 - 0.5 * u), 0.11 * (1.0 - 0.5 * (u + 0.1)), 5, culm, shade(culm, 1.05), false);
            if u > 0.3 {
                for _ in 0..7 {
                    let la = rng.range(0.0, TAU);
                    let ld = (Vec3::new(la.cos(), rng.range(-0.6, 0.1), la.sin()) + out * 0.5 + dir * 0.4).normalize();
                    leaf(&mut g, q, ld, rng.range(1.3, 2.3), 0.32, *rng.pick(&greens));
                }
                // Khối lá dày để bụi tre có thể tích nhìn từ xa.
                if s % 2 == 1 {
                    let r = rng.range(1.0, 1.6) * (0.6 + u);
                    g.blob(q + out * r * 0.5, Vec3::new(r, r * 0.75, r), *rng.pick(&greens), 0.22, rng);
                }
            }
            p = q;
        }
        // Ngọn rủ.
        for _ in 0..5 {
            let la = rng.range(0.0, TAU);
            let ld = (out + Vec3::new(la.cos() * 0.5, -0.5, la.sin() * 0.5)).normalize();
            leaf(&mut g, p, ld, rng.range(1.0, 1.8), 0.22, *rng.pick(&greens));
        }
    }
    g
}

/// Cây cau: thân mảnh cao, tán lá xòe rủ.
fn areca(rng: &mut Rng) -> (Geo, f32) {
    let mut g = Geo::default();
    let h = rng.range(8.5, 11.5);
    let bend = Vec3::new(rng.range(-0.4, 0.4), 0.0, rng.range(-0.4, 0.4));
    let mut p = Vec3::ZERO;
    let n = 8;
    for k in 0..n {
        let u = (k + 1) as f32 / n as f32;
        let q = Vec3::new(0.0, h * u, 0.0) + bend * u * u;
        let c = if k % 2 == 0 { hex(0x9a9585) } else { hex(0x8a8576) };
        g.tube(p, q, 0.17, 0.15, 6, c, c, false);
        p = q;
    }
    g.tube(p, p + Vec3::Y * 0.9, 0.2, 0.17, 6, hex(0x7a9a40), hex(0x8aaa48), false);
    let top = p + Vec3::Y * 0.9;
    for f in 0..9 {
        let a = f as f32 / 9.0 * TAU + rng.range(-0.2, 0.2);
        let out = Vec3::new(a.cos(), 0.0, a.sin());
        let mut prev = top;
        let segs = 6;
        let len = rng.range(2.8, 3.6);
        for s in 1..=segs {
            let u = s as f32 / segs as f32;
            let q = top + out * len * u + Vec3::Y * (0.9 * u - 2.0 * u * u);
            g.beam(prev, q, 0.035, hex(0x6a8a35));
            let side = out.cross(Vec3::Y).normalize() * 0.9 * (1.0 - u * 0.6);
            let col = mix(hex(0x4f7d2c), hex(0x6f9a3c), rng.f());
            g.tri2(prev, q, q + side - Vec3::Y * 0.35, col);
            g.tri2(prev, q - side - Vec3::Y * 0.35, q, shade(col, 0.9));
            prev = q;
        }
    }
    (g, h)
}

fn banana(rng: &mut Rng) -> Geo {
    let mut g = Geo::default();
    let h = rng.range(1.6, 2.4);
    g.tube(Vec3::ZERO, Vec3::Y * h, 0.22, 0.17, 6, hex(0x6a8a3a), hex(0x7a9a42), false);
    for k in 0..7 {
        let a = k as f32 / 7.0 * TAU + rng.range(-0.3, 0.3);
        let out = Vec3::new(a.cos(), 0.0, a.sin());
        let side = out.cross(Vec3::Y).normalize();
        let len = rng.range(1.8, 2.6);
        let col = mix(hex(0x6fa840), hex(0x88b84c), rng.f());
        let segs = 5;
        let base = Vec3::Y * (h - 0.1);
        let pt = |u: f32| base + out * len * u + Vec3::Y * (1.2 * u - 1.6 * u * u);
        for s in 0..segs {
            let (u0, u1) = (s as f32 / segs as f32, (s + 1) as f32 / segs as f32);
            let w = |u: f32| 0.45 * (u * PI).sin() + 0.05;
            let (a0, a1) = (pt(u0), pt(u1));
            g.quad2(a0 - side * w(u0), a1 - side * w(u1), a1 + side * w(u1), a0 + side * w(u0), shade(col, 0.95 + 0.1 * (s % 2) as f32));
        }
    }
    g
}

/// Cây đa: (thân + rễ tĩnh, tán lay).
fn banyan(rng: &mut Rng) -> (Geo, Geo) {
    let mut trunk = Geo::default();
    let mut crown = Geo::default();
    let bark = hex(0x6e6252);
    for k in 0..6 {
        let a = k as f32 / 6.0 * TAU;
        let o = Vec3::new(a.cos(), 0.0, a.sin());
        trunk.tube(o * 1.6 - Vec3::Y * 0.3, o * 0.4 + Vec3::Y * 5.5, 0.55, 0.45, 6, shade(bark, 0.85), bark, false);
    }
    let mut tips = Vec::new();
    for k in 0..7 {
        let a = k as f32 / 7.0 * TAU + rng.range(-0.3, 0.3);
        let o = Vec3::new(a.cos(), 0.0, a.sin());
        let tip = o * rng.range(6.0, 9.0) + Vec3::Y * rng.range(8.0, 10.5);
        trunk.tube(Vec3::Y * 5.0, tip, 0.45, 0.18, 6, bark, shade(bark, 1.1), false);
        tips.push(tip);
        // Rễ phụ buông.
        for _ in 0..4 {
            let u = rng.range(0.35, 0.95);
            let p = Vec3::Y * 5.0 + (tip - Vec3::Y * 5.0) * u;
            trunk.tube(p, Vec3::new(p.x, -0.2, p.z), 0.05, 0.07, 4, shade(bark, 1.1), bark, false);
        }
    }
    let greens = [hex(0x4f8a36), hex(0x5e9a3e), hex(0x467d30), hex(0x6aa244)];
    for _ in 0..22 {
        let a = rng.range(0.0, TAU);
        let r = rng.range(0.0, 9.5);
        let c = Vec3::new(a.cos() * r, rng.range(9.0, 13.5) - r * 0.25, a.sin() * r);
        let s = rng.range(3.0, 4.8);
        crown.blob(c, Vec3::new(s, s * 0.7, s), *rng.pick(&greens), 0.18, rng);
    }
    (trunk, crown)
}

/// Bụi lau sậy ven nước.
fn reeds(rng: &mut Rng) -> Geo {
    let mut g = Geo::default();
    for _ in 0..16 {
        let a = rng.range(0.0, TAU);
        let r = rng.range(0.0, 0.8);
        let b = Vec3::new(a.cos() * r, -0.1, a.sin() * r);
        let h = rng.range(1.3, 2.6);
        let lean = Vec3::new(rng.range(-0.4, 0.4), 0.0, rng.range(-0.4, 0.4));
        let tip = b + Vec3::Y * h + lean;
        let side = Vec3::new(lean.z, 0.0, -lean.x).normalize_or(Vec3::X) * 0.05;
        g.tri2(b - side, b + side, tip, mix(hex(0x7a9a48), hex(0x9aae5a), rng.f()));
        if rng.f() < 0.45 {
            g.blob(tip, Vec3::new(0.08, 0.28, 0.08), mix(hex(0xe8e0cc), hex(0xd8c8a8), rng.f()), 0.2, rng);
        }
    }
    g
}

/// Lá sen: đĩa có khía.
fn lotus_pad(rng: &mut Rng) -> Geo {
    let mut g = Geo::default();
    let r = rng.range(0.55, 1.1);
    let col = mix(hex(0x4c8a38), hex(0x6aa046), rng.f());
    let n = 11;
    for i in 1..n {
        let a0 = i as f32 / n as f32 * TAU;
        let a1 = (i + 1) as f32 / n as f32 * TAU;
        let rim = |a: f32| Vec3::new(a.cos() * r, 0.04 * (a * 3.0).sin(), a.sin() * r);
        g.tri(Vec3::Y * 0.03, rim(a1), rim(a0), shade(col, 0.95 + 0.1 * (i % 2) as f32));
    }
    g
}

fn lotus_flower(rng: &mut Rng) -> Geo {
    let mut g = Geo::default();
    let h = rng.range(0.6, 1.2);
    g.tube(Vec3::ZERO, Vec3::Y * h, 0.03, 0.03, 4, hex(0x5a8a3a), hex(0x5a8a3a), false);
    let c = Vec3::Y * h;
    let pink = hex(0xf09ab4);
    let pale = hex(0xfde4ec);
    for ring in 0..2 {
        let n = 7;
        let tilt: f32 = if ring == 0 { 0.9 } else { 0.45 };
        let len = if ring == 0 { 0.55 } else { 0.48 };
        for i in 0..n {
            let a = (i as f32 + ring as f32 * 0.5) / n as f32 * TAU;
            let out = Vec3::new(a.cos(), 0.0, a.sin());
            let dir = (out * tilt.sin() + Vec3::Y * tilt.cos()).normalize();
            let side = out.cross(Vec3::Y).normalize() * 0.17;
            let tip = c + dir * len;
            let mid = c + dir * len * 0.5;
            g.tri_c(c, mid + side, tip, pink, pale, pink);
            g.tri_c(c, tip, mid - side, pink, pink, pale);
            g.tri_c(c, tip, mid + side, pink, pink, pale);
            g.tri_c(c, mid - side, tip, pink, pale, pink);
        }
    }
    g.tube(c, c + Vec3::Y * 0.12, 0.12, 0.1, 6, hex(0xe8c040), hex(0xf0d060), true);
    g
}

/// Một ô lúa: các khóm 3 lá, đỉnh có trọng số lay.
fn rice_plot(id: IVec2, rng: &mut Rng) -> Option<(Geo, Vec<f32>)> {
    let ripe = world::ripe(id);
    let (ox, oz) = (id.x as f32 * PLOT.x - 3.0, id.y as f32 * PLOT.y - 1.0);
    let mut g = Geo::default();
    let mut w = Vec::new();
    let step = 1.1;
    let mut u = 0.8;
    while u < PLOT.x - 0.7 {
        let mut v = 0.8;
        while v < PLOT.y - 0.7 {
            let (x, z) = (ox + u + rng.range(-0.15, 0.15), oz + v + rng.range(-0.15, 0.15));
            if world::paddy(x, z).map(|(i, _)| i) == Some(id) {
                let y = height(x, z);
                let base = Vec3::new(x, y - 0.05, z);
                let (lo, hi) = if ripe {
                    (hex(0x9a8a3a), mix(hex(0xe0b84a), hex(0xd8a83c), rng.f()))
                } else {
                    (hex(0x6a9a34), mix(hex(0x9cc850), hex(0xaed45a), rng.f()))
                };
                for k in 0..3 {
                    let a = k as f32 / 3.0 * TAU + rng.range(-0.5, 0.5);
                    let h = rng.range(0.65, 1.0);
                    let lean = Vec3::new(a.cos(), 0.0, a.sin()) * rng.range(0.15, 0.35);
                    let side = Vec3::new(-a.sin(), 0.0, a.cos()) * 0.12;
                    let tip = base + Vec3::Y * h + lean + if ripe { lean * 0.8 - Vec3::Y * 0.15 } else { Vec3::ZERO };
                    g.tri_c(base - side, base + side, tip, lo, lo, hi);
                    w.extend([0.0, 0.0, h]);
                }
            }
            v += step;
        }
        u += step;
    }
    if g.len() == 0 { None } else { Some((g, w)) }
}

// ───────────────────────── Dựng cảnh ─────────────────────────

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let vc = materials.add(StandardMaterial { perceptual_roughness: 0.88, reflectance: 0.2, ..default() });
    let plant = materials.add(StandardMaterial {
        perceptual_roughness: 0.8,
        reflectance: 0.25,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    // Đèn và cửa: không chiếu sáng, độ sáng nhân theo đêm (vượt 1 → bloom).
    let glow = materials.add(StandardMaterial { unlit: true, ..default() });
    // Nước: màu đỉnh tính trên CPU (fresnel trời + lóa nắng), không chiếu sáng.
    let water = materials.add(StandardMaterial { unlit: true, alpha_mode: AlphaMode::Blend, ..default() });
    let mut rng = Rng::new(2026);
    let mut stat = Geo::default();
    let mut glow_g = Geo::default();
    let mut spots = Spots::default();
    let mut lamps: Vec<(Vec3, f32)> = Vec::new();

    // Địa hình, núi, nước.
    commands.spawn((Mesh3d(meshes.add(world::terrain().mesh())), MeshMaterial3d(vc.clone()), Transform::default()));
    commands.spawn((
        Mesh3d(meshes.add(world::mountains().mesh())),
        MeshMaterial3d(materials.add(StandardMaterial { perceptual_roughness: 1.0, cull_mode: None, double_sided: true, ..default() })),
        Transform::default(),
        NotShadowCaster,
    ));
    let (wg, anim) = world::water();
    commands.spawn((Mesh3d(meshes.add(wg.mesh())), MeshMaterial3d(water), Transform::default(), anim, NotShadowCaster));

    // Đình.
    let dt = place(-12.0, -54.0, 0.0);
    let dt = dt.with_translation(dt.translation.with_y(0.7));
    let (g, gl, lp) = dinh(&mut rng);
    stat.add(&g, dt);
    glow_g.add(&gl, dt);
    for p in lp {
        lamps.push((dt.transform_point(p) - Vec3::Y * 0.6 + Vec3::Z * 0.4, 9000.0));
    }
    lamps.push((dt.transform_point(Vec3::new(0.0, 3.0, 3.0)), 30000.0));
    // Cờ hội.
    for s in [-1.0f32, 1.0] {
        let base = dt.transform_point(Vec3::new(s * 10.0, 0.0, 19.0));
        stat.tube(base, base + Vec3::Y * 10.0, 0.08, 0.06, 5, hex(0x8a6a40), hex(0x8a6a40), false);
        let mut flag = Geo::default();
        let bands = [hex(0xd83a2a), hex(0xe8b030), hex(0x3a8a3a), hex(0x3a6ac8), hex(0xf0f0e8)];
        for (k, c) in bands.iter().enumerate() {
            let y0 = -0.4 * k as f32;
            flag.quad2(Vec3::new(0.0, y0, 0.0), Vec3::new(2.4, y0 - 0.15, 0.0), Vec3::new(2.4, y0 - 0.55, 0.0), Vec3::new(0.0, y0 - 0.4, 0.0), *c);
        }
        commands.spawn((
            Sway { amp: 0.25, freq: 2.2, phase: s, base: Quat::from_rotation_y(-0.6 * s - PI / 2.0) },
            Mesh3d(meshes.add(flag.mesh())),
            MeshMaterial3d(plant.clone()),
            Transform::from_translation(base + Vec3::Y * 9.9).with_rotation(Quat::from_rotation_y(-0.6 * s - PI / 2.0)),
        ));
    }

    // Nhà.
    let houses = [
        (-44.0, -38.0, 0.12, false),
        (-36.0, -16.0, -0.08, true),
        (8.0, -46.0, -0.1, false),
        (32.0, -44.0, 0.05, true),
        (10.0, -14.0, 0.1, false),
        (-20.0, -14.0, -0.04, false),
        (44.0, -30.0, 0.22, true),
        (-54.0, -22.0, 0.3, true),
        (-30.0, -62.0, 0.05, false),
    ];
    for &(x, z, r, thatch) in &houses {
        let t = place(x, z, r);
        let t = t.with_translation(t.translation.with_y(0.7));
        let (g, gl, ch) = house(&mut rng, thatch);
        stat.add(&g, t);
        glow_g.add(&gl, t);
        spots.chimneys.push(t.transform_point(ch));
        lamps.push((t.transform_point(Vec3::new(0.0, 1.6, 3.6)), 6000.0));
    }

    // Cổng làng trên đường từ cầu vào.
    let gz = -1.0;
    let gx = BRIDGE_X + 3.0 * (gz * 0.07f32).sin();
    let gt = place(gx, gz, 0.0);
    let (g, gl) = gate(&mut rng);
    stat.add(&g, gt);
    glow_g.add(&gl, gt);
    lamps.push((gt.transform_point(Vec3::new(0.0, 3.0, 1.5)), 5000.0));

    bridge(&mut stat);
    well(&mut stat, Vec3::new(30.0, height(30.0, -16.0), -16.0));

    // Cây đa đầu làng.
    let (trunk, crown) = banyan(&mut rng);
    let bt = place(36.0, -8.0, 0.4);
    stat.add(&trunk, bt);
    commands.spawn((
        Sway { amp: 0.012, freq: 0.7, phase: 0.0, base: bt.rotation },
        Mesh3d(meshes.add(crown.mesh())),
        MeshMaterial3d(vc.clone()),
        bt,
    ));

    commands.spawn((Mesh3d(meshes.add(stat.mesh())), MeshMaterial3d(vc.clone()), Transform::default()));
    commands.spawn((Mesh3d(meshes.add(glow_g.mesh())), MeshMaterial3d(glow.clone()), Transform::default(), NotShadowCaster));
    for (p, max) in lamps {
        commands.spawn((
            NightLamp { max },
            PointLight { color: hex(0xffa850), intensity: 0.0, range: 14.0, shadow_maps_enabled: false, ..default() },
            Transform::from_translation(p),
        ));
    }

    // Lũy tre quanh làng (chừa cổng).
    let bamboo_meshes: Vec<Handle<Mesh>> = (0..6).map(|_| meshes.add(bamboo(&mut rng).mesh())).collect();
    let mut a = 0.0f32;
    while a < TAU {
        let p = VILLAGE + Vec2::new(a.cos(), a.sin()) * world::VILLAGE_R * rng.range(1.04, 1.12);
        let near_gate = (p - Vec2::new(gx, gz)).length() < 9.0;
        let near_dike = p.y > world::dike_z(p.x) - 4.0;
        if !near_gate && !near_dike {
            spawn_sway(&mut commands, rng.pick(&bamboo_meshes).clone(), &plant, p, &mut rng, 0.035, 0.9);
        }
        a += rng.range(0.07, 0.13);
    }
    // Vài bụi tre trong làng.
    for &(x, z) in &[(-58.0, -40.0), (-8.0, -30.0), (24.0, -54.0), (50.0, -44.0), (-40.0, -52.0)] {
        spawn_sway(&mut commands, rng.pick(&bamboo_meshes).clone(), &plant, Vec2::new(x, z), &mut rng, 0.035, 0.9);
    }

    // Cau, chuối quanh nhà.
    let areca_meshes: Vec<Handle<Mesh>> = (0..4).map(|_| meshes.add(areca(&mut rng).0.mesh())).collect();
    let banana_meshes: Vec<Handle<Mesh>> = (0..3).map(|_| meshes.add(banana(&mut rng).mesh())).collect();
    for &(x, z, r, _) in &houses {
        let rot = Quat::from_rotation_y(r);
        for _ in 0..3 {
            let off = rot * Vec3::new(rng.range(-9.0, 9.0), 0.0, rng.range(-6.0, -4.0));
            spawn_sway(&mut commands, rng.pick(&areca_meshes).clone(), &plant, Vec2::new(x + off.x, z + off.z), &mut rng, 0.02, 0.6);
        }
        for _ in 0..2 {
            let off = rot * Vec3::new(rng.range(-8.0, -6.0), 0.0, rng.range(-3.0, 2.0));
            spawn_sway(&mut commands, rng.pick(&banana_meshes).clone(), &plant, Vec2::new(x + off.x, z + off.z), &mut rng, 0.04, 1.1);
        }
    }
    for &(x, z) in &[(-24.0, -38.0), (0.0, -38.0), (-28.0, -46.0), (4.0, -60.0)] {
        spawn_sway(&mut commands, rng.pick(&areca_meshes).clone(), &plant, Vec2::new(x, z), &mut rng, 0.02, 0.6);
    }

    // Lau sậy ven sông và ao.
    let reed_meshes: Vec<Handle<Mesh>> = (0..5).map(|_| meshes.add(reeds(&mut rng).mesh())).collect();
    let mut x = -190.0f32;
    while x < 190.0 {
        for side in [-1.0f32, 1.0] {
            if (x - BRIDGE_X).abs() < 4.0 || rng.f() < 0.35 {
                continue;
            }
            let z = river_z(x) + side * (RIVER_W + rng.range(0.5, 3.0));
            spawn_sway(&mut commands, rng.pick(&reed_meshes).clone(), &plant, Vec2::new(x, z), &mut rng, 0.07, 1.4);
        }
        x += rng.range(2.5, 6.0);
    }
    for k in 0..26 {
        let a = k as f32 / 26.0 * TAU + rng.range(-0.1, 0.1);
        let r = POND_R + rng.range(0.5, 2.5);
        spawn_sway(&mut commands, rng.pick(&reed_meshes).clone(), &plant, POND + Vec2::new(a.cos(), a.sin()) * r, &mut rng, 0.07, 1.4);
    }

    // Sen.
    let pad_meshes: Vec<Handle<Mesh>> = (0..5).map(|_| meshes.add(lotus_pad(&mut rng).mesh())).collect();
    let flower_meshes: Vec<Handle<Mesh>> = (0..3).map(|_| meshes.add(lotus_flower(&mut rng).mesh())).collect();
    for _ in 0..70 {
        let a = rng.range(0.0, TAU);
        let r = POND_R * rng.f().sqrt() * 0.95;
        let p = POND + Vec2::new(a.cos(), a.sin()) * r;
        let rot = Quat::from_rotation_y(rng.range(0.0, TAU));
        commands.spawn((
            Sway { amp: 0.03, freq: 0.9, phase: rng.range(0.0, TAU), base: rot },
            Mesh3d(rng.pick(&pad_meshes).clone()),
            MeshMaterial3d(plant.clone()),
            Transform::from_xyz(p.x, WATER_Y + 0.12 + rng.range(0.0, 0.25), p.y).with_rotation(rot),
        ));
    }
    for _ in 0..18 {
        let a = rng.range(0.0, TAU);
        let r = POND_R * rng.f().sqrt() * 0.85;
        let p = POND + Vec2::new(a.cos(), a.sin()) * r;
        let rot = Quat::from_rotation_y(rng.range(0.0, TAU));
        commands.spawn((
            Sway { amp: 0.06, freq: 1.1, phase: rng.range(0.0, TAU), base: rot },
            Mesh3d(rng.pick(&flower_meshes).clone()),
            MeshMaterial3d(plant.clone()),
            Transform::from_xyz(p.x, WATER_Y + 0.1, p.y).with_rotation(rot),
        ));
    }

    // Lúa.
    let mut seen = std::collections::BTreeSet::new();
    let mut z = -190.0f32;
    while z < 160.0 {
        let mut x = -200.0f32;
        while x < 200.0 {
            if let Some((id, _)) = world::paddy(x, z)
                && !world::flooded(id)
                && seen.insert((id.x, id.y))
                && let Some((g, w)) = rice_plot(id, &mut rng)
            {
                let base = g.pos.iter().map(|p| Vec3::from(*p)).collect();
                commands.spawn((
                    VertexSway { base, w, amp: 0.22 },
                    Mesh3d(meshes.add(g.mesh())),
                    MeshMaterial3d(plant.clone()),
                    Transform::default(),
                    NotShadowCaster,
                ));
            }
            x += 2.0;
        }
        z += 2.0;
    }

    commands.insert_resource(Mats { vc, plant, glow });
    commands.insert_resource(spots);
}

fn spawn_sway(commands: &mut Commands, mesh: Handle<Mesh>, mat: &Handle<StandardMaterial>, p: Vec2, rng: &mut Rng, amp: f32, freq: f32) {
    let rot = Quat::from_rotation_y(rng.range(0.0, TAU));
    let s = rng.range(0.85, 1.15);
    commands.spawn((
        Sway { amp, freq, phase: rng.range(0.0, TAU), base: rot },
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_xyz(p.x, height(p.x, p.y), p.y).with_rotation(rot).with_scale(Vec3::splat(s)),
    ));
}
