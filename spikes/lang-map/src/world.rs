//! Bố cục thung lũng: địa hình, sông, đê, ruộng, ao, núi xa, mặt nước.

use bevy::prelude::*;

use crate::geom::{Geo, Rng, fbm, hex, mix, shade, smoothstep};

pub const RIVER_W: f32 = 7.0;
pub const WATER_Y: f32 = -0.45;
pub const BRIDGE_X: f32 = 22.0;
pub const VILLAGE: Vec2 = Vec2::new(-5.0, -34.0);
pub const VILLAGE_R: Vec2 = Vec2::new(56.0, 32.0);
pub const POND: Vec2 = Vec2::new(8.0, 50.0);
pub const POND_R: f32 = 11.0;
pub const PLOT: Vec2 = Vec2::new(14.0, 10.0);
/// Đê chạy song song bờ bắc, cách tâm sông.
pub const DIKE_OFF: f32 = -13.0;

/// Tâm sông theo `x` (sông chảy dọc trục x).
pub fn river_z(x: f32) -> f32 {
    20.0 + 6.0 * (x * 0.032).sin() + 2.5 * (x * 0.087 + 1.0).sin()
}

/// Khoảng cách có dấu tới tâm sông: âm là phía làng (bắc).
pub fn river_d(x: f32, z: f32) -> f32 {
    z - river_z(x)
}

/// Tâm đê tại `x`.
pub fn dike_z(x: f32) -> f32 {
    river_z(x) + DIKE_OFF
}

/// Độ "trong làng": < 1 là trong.
pub fn village_e(x: f32, z: f32) -> f32 {
    let d = (Vec2::new(x, z) - VILLAGE) / VILLAGE_R;
    d.length_squared()
}

fn hills(x: f32, z: f32) -> f32 {
    smoothstep(100.0, 175.0, (x + 5.0).abs())
        .max(smoothstep(-78.0, -150.0, z))
        .max(smoothstep(96.0, 150.0, z))
}

/// Đường đất: từ cầu vào làng, ngõ chính trong làng, đường bờ nam ra ao.
pub fn path_w(x: f32, z: f32) -> f32 {
    let mut w: f32 = 0.0;
    // Cầu → cổng làng.
    let px = BRIDGE_X + 3.0 * (z * 0.07).sin();
    if z < river_z(BRIDGE_X) - 9.0 && z > VILLAGE.y + 8.0 {
        w = w.max(smoothstep(2.0, 1.0, (x - px).abs()));
    }
    // Ngõ chính.
    let lz = VILLAGE.y + 10.0 + 3.0 * (x * 0.05).sin();
    if village_e(x, z) < 0.95 {
        w = w.max(smoothstep(1.8, 0.9, (z - lz).abs()));
    }
    // Bờ nam: cầu → ao.
    if z > river_z(BRIDGE_X) + 9.0 && z < 92.0 {
        let sx = BRIDGE_X - 0.18 * (z - river_z(BRIDGE_X) - 9.0) + 2.0 * (z * 0.09).sin();
        w = w.max(smoothstep(1.8, 0.9, (x - sx).abs()));
    }
    w
}

/// Ô ruộng chứa (x, z), nếu có: (chỉ số, tọa độ trong ô [0,1]²).
pub fn paddy(x: f32, z: f32) -> Option<(IVec2, Vec2)> {
    let d = river_d(x, z);
    let side_ok = d < DIKE_OFF - 3.5 || d > RIVER_W + 4.0;
    if !side_ok || village_e(x, z) < 1.2 || hills(x, z) > 0.05 || z > 92.0 {
        return None;
    }
    if (Vec2::new(x, z) - POND).length() < POND_R + 6.0 || path_w(x, z) > 0.0 {
        return None;
    }
    let gx = (x + 3.0) / PLOT.x;
    let gz = (z + 1.0) / PLOT.y;
    let id = IVec2::new(gx.floor() as i32, gz.floor() as i32);
    Some((id, Vec2::new(gx - gx.floor(), gz - gz.floor())))
}

/// Ô ruộng ngập nước (soi trời).
pub fn flooded(id: IVec2) -> bool {
    let h = (id.x.wrapping_mul(73_856_093) ^ id.y.wrapping_mul(19_349_663)) as u32;
    h % 5 == 0
}

/// Ô ruộng lúa chín vàng.
pub fn ripe(id: IVec2) -> bool {
    let h = (id.x.wrapping_mul(83_492_791) ^ id.y.wrapping_mul(2_971_215_073u32 as i32)) as u32;
    h % 3 == 0
}

/// Độ cao mặt đất.
pub fn height(x: f32, z: f32) -> f32 {
    let n = fbm(x * 0.03, z * 0.03);
    let mut h = (n - 0.5) * 0.9 + 0.2;
    let hl = hills(x, z);
    h += hl * (5.0 + 24.0 * fbm(x * 0.014 + 3.0, z * 0.014 - 2.0));
    if let Some((id, uv)) = paddy(x, z) {
        let edge = uv.x.min(1.0 - uv.x).min(uv.y.min(1.0 - uv.y));
        let base = if flooded(id) { -0.18 } else { 0.0 };
        let bund = smoothstep(0.06, 0.02, edge) * 0.45;
        h = base + bund;
    }
    // Gò làng.
    let ve = village_e(x, z);
    h = h + (0.7 - h) * smoothstep(1.15, 0.85, ve);
    // Đê.
    let d = river_d(x, z);
    let dike = (1.0 - ((d - DIKE_OFF) / 3.2).powi(2)).max(0.0);
    h = h.max(0.2 + 1.7 * dike.sqrt() * (1.0 - hl));
    // Sông.
    let bank = smoothstep(RIVER_W + 3.5, RIVER_W - 1.0, d.abs());
    h = h + (-2.1 - h) * bank;
    // Ao sen.
    let dp = (Vec2::new(x, z) - POND).length();
    h = h + (-1.4 - h) * smoothstep(POND_R + 2.5, POND_R - 2.0, dp);
    // Đường hơi đắp cao.
    h + path_w(x, z) * 0.08 * (1.0 - bank)
}

fn ground_color(x: f32, z: f32, h: f32, ny: f32) -> Color {
    let n = fbm(x * 0.11 + 7.0, z * 0.11);
    let grass = mix(hex(0x6f9a45), hex(0x8aa850), n);
    let d = river_d(x, z);
    if h < -0.8 {
        return mix(hex(0x5b5340), hex(0x7d6e4e), n);
    }
    if h < WATER_Y + 0.5 && d.abs() < RIVER_W + 4.0 {
        return mix(hex(0x8d7c58), hex(0x6f8a45), smoothstep(WATER_Y, WATER_Y + 0.5, h));
    }
    let pw = path_w(x, z);
    if pw > 0.3 {
        return mix(hex(0xa8865c), hex(0xb89a70), n);
    }
    let dike = (1.0 - ((d - DIKE_OFF) / 3.2).powi(2)).max(0.0);
    if dike > 0.0 && hills(x, z) < 0.5 {
        if dike > 0.75 && ny > 0.95 {
            return mix(hex(0x9e7f58), hex(0xb0916a), n);
        }
        return mix(hex(0x7aa14a), hex(0x93b356), n);
    }
    if let Some((id, uv)) = paddy(x, z) {
        let edge = uv.x.min(1.0 - uv.x).min(uv.y.min(1.0 - uv.y));
        if edge < 0.05 {
            return mix(hex(0x86a852), hex(0x9bb85c), n);
        }
        return if flooded(id) {
            hex(0x5e5a3c)
        } else if ripe(id) {
            hex(0x8a7a3c)
        } else {
            hex(0x5c6e32)
        };
    }
    let ve = village_e(x, z);
    if ve < 1.0 {
        return mix(grass, mix(hex(0x9c8460), hex(0x8a9a50), n), 0.45);
    }
    let hl = hills(x, z);
    if hl > 0.0 {
        let forest = mix(hex(0x55823c), hex(0x3c6531), smoothstep(4.0, 18.0, h));
        let c = mix(grass, forest, smoothstep(0.0, 0.5, hl));
        return if ny < 0.8 { mix(c, hex(0x8c8470), 0.6) } else { c };
    }
    grass
}

/// Lưới địa hình (tam giác phẳng).
pub fn terrain() -> Geo {
    let mut g = Geo::default();
    let (x0, x1, z0, z1, s) = (-200.0f32, 200.0f32, -190.0f32, 160.0f32, 2.0f32);
    let nx = ((x1 - x0) / s) as i32;
    let nz = ((z1 - z0) / s) as i32;
    let p = |i: i32, j: i32| {
        let (x, z) = (x0 + i as f32 * s, z0 + j as f32 * s);
        Vec3::new(x, height(x, z), z)
    };
    for j in 0..nz {
        for i in 0..nx {
            let (a, b, c, d) = (p(i, j + 1), p(i + 1, j + 1), p(i + 1, j), p(i, j));
            // Xen kẽ đường chéo cho đỡ lộ lưới.
            let tris = if (i + j) % 2 == 0 { [[a, b, c], [a, c, d]] } else { [[a, b, d], [b, c, d]] };
            for [u, v, w] in tris {
                let cen = (u + v + w) / 3.0;
                let ny = (v - u).cross(w - u).normalize_or(Vec3::Y).y;
                let col = ground_color(cen.x, cen.z, cen.y, ny);
                g.tri(u, v, w, col);
            }
        }
    }
    g
}

/// Dữ liệu sóng: vị trí gốc và biên độ sóng theo đỉnh.
#[derive(Component)]
pub struct WaterAnim {
    pub base: Vec<Vec3>,
    pub amp: Vec<f32>,
}

/// Mặt sông, ao, ruộng ngập.
pub fn water() -> (Geo, WaterAnim) {
    let mut g = Geo::default();
    let mut amp = Vec::new();
    let white = Color::WHITE;
    // Sông: dải lưới bám tâm sông.
    let (n_across, s) = (8, 2.0f32);
    let half = RIVER_W + 2.5;
    let mut x = -260.0f32;
    while x < 260.0 {
        for k in 0..n_across {
            let w0 = -half + 2.0 * half * k as f32 / n_across as f32;
            let w1 = -half + 2.0 * half * (k + 1) as f32 / n_across as f32;
            let p = |x: f32, w: f32| Vec3::new(x, WATER_Y, river_z(x) + w);
            let (a, b, c, d) = (p(x, w1), p(x + s, w1), p(x + s, w0), p(x, w0));
            g.quad(a, b, c, d, white);
            for _ in 0..6 {
                amp.push(0.09);
            }
        }
        x += s;
    }
    // Ao: đĩa tròn.
    let seg = 40;
    for i in 0..seg {
        for r in 0..4 {
            let r0 = (POND_R + 2.0) * r as f32 / 4.0;
            let r1 = (POND_R + 2.0) * (r + 1) as f32 / 4.0;
            let a0 = i as f32 / seg as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / seg as f32 * std::f32::consts::TAU;
            let p = |r: f32, a: f32| Vec3::new(POND.x + r * a.cos(), WATER_Y + 0.1, POND.y + r * a.sin());
            g.quad(p(r0, a0), p(r0, a1), p(r1, a1), p(r1, a0), white);
            for _ in 0..6 {
                amp.push(0.04);
            }
        }
    }
    // Ruộng ngập: mỗi ô một tấm phẳng.
    let mut seen = std::collections::BTreeSet::new();
    let mut z = -200.0f32;
    while z < 160.0 {
        let mut x = -200.0f32;
        while x < 200.0 {
            if let Some((id, _)) = paddy(x, z)
                && flooded(id)
                && seen.insert((id.x, id.y))
            {
                let ox = id.x as f32 * PLOT.x - 3.0;
                let oz = id.y as f32 * PLOT.y - 1.0;
                // Chỉ phủ phần trong ô thực sự là ruộng.
                let m = 0.7;
                let n = 4;
                for a in 0..n {
                    for b in 0..n {
                        let u0 = m + (PLOT.x - 2.0 * m) * a as f32 / n as f32;
                        let u1 = m + (PLOT.x - 2.0 * m) * (a + 1) as f32 / n as f32;
                        let v0 = m + (PLOT.y - 2.0 * m) * b as f32 / n as f32;
                        let v1 = m + (PLOT.y - 2.0 * m) * (b + 1) as f32 / n as f32;
                        let ok = [(u0, v0), (u1, v0), (u0, v1), (u1, v1)]
                            .iter()
                            .all(|&(u, v)| paddy(ox + u, oz + v).map(|(i, _)| i) == Some(id));
                        if !ok {
                            continue;
                        }
                        let p = |u: f32, v: f32| Vec3::new(ox + u, 0.02, oz + v);
                        g.quad(p(u0, v1), p(u1, v1), p(u1, v0), p(u0, v0), white);
                        for _ in 0..6 {
                            amp.push(0.012);
                        }
                    }
                }
            }
            x += 2.0;
        }
        z += 2.0;
    }
    let base = g.pos.iter().map(|p| Vec3::from(*p)).collect();
    (g, WaterAnim { base, amp })
}

/// Bốn lớp núi vòng quanh, nhạt dần theo khoảng cách (kiểu tranh lụa).
pub fn mountains() -> Geo {
    let mut g = Geo::default();
    let layers = [
        (300.0f32, 45.0f32, hex(0x3f5f44), hex(0x7f9a86), 1u64),
        (390.0, 75.0, hex(0x4f6c62), hex(0x8ea2a4), 2),
        (500.0, 115.0, hex(0x637a82), hex(0x9fb0bd), 3),
        (640.0, 170.0, hex(0x7a8a9c), hex(0xb3bfcc), 4),
    ];
    let seg = 220;
    for (r, hmax, top, foot, seed) in layers {
        let mut rng = Rng::new(seed);
        let off = rng.range(0.0, 100.0);
        let ridge: Vec<(Vec3, Vec3, Vec3)> = (0..=seg)
            .map(|i| {
                let a = i as f32 / seg as f32 * std::f32::consts::TAU;
                let dir = Vec3::new(a.cos(), 0.0, a.sin());
                // Phía nam (sau lưng máy quay, z > 0) thấp hơn để thoáng.
                let south = smoothstep(-0.2, 0.9, dir.z);
                let n = fbm(a * 4.0 + off, seed as f32 * 3.1);
                let sharp = (fbm(a * 11.0 + off, 7.0) - 0.5) * 0.35;
                let h = hmax * (0.35 + 0.75 * n + sharp) * (1.0 - 0.6 * south);
                let rr = r + (n - 0.5) * 40.0;
                let top_p = dir * rr + Vec3::Y * h;
                let mid_p = dir * (rr + 12.0) + Vec3::Y * (h * 0.45 + rng.range(-3.0, 3.0));
                let base_p = dir * (rr + 30.0) + Vec3::Y * -8.0;
                (top_p, mid_p, base_p)
            })
            .collect();
        for w in ridge.windows(2) {
            let ((t0, m0, b0), (t1, m1, b1)) = (w[0], w[1]);
            let k = 0.94 + 0.12 * rng.f();
            let ct = shade(top, k);
            let cm = shade(mix(top, foot, 0.45), k);
            let cb = foot;
            // Mặt hướng vào tâm (nhìn từ làng).
            g.tri_c(t0, t1, m1, ct, ct, cm);
            g.tri_c(t0, m1, m0, ct, cm, cm);
            g.tri_c(m0, m1, b1, cm, cm, cb);
            g.tri_c(m0, b1, b0, cm, cb, cb);
        }
    }
    g
}
