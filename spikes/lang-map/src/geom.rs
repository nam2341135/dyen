//! Dựng hình low-poly bằng code: tam giác phẳng có màu đỉnh, số ngẫu nhiên có seed, nhiễu.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Mesh, PrimitiveTopology};
use bevy::prelude::*;

/// Màu sRGB từ mã hex `0xRRGGBB`.
pub fn hex(c: u32) -> Color {
    Color::srgb_u8((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// Nhân độ sáng một màu (trong không gian tuyến tính).
pub fn shade(c: Color, k: f32) -> Color {
    let l = c.to_linear();
    Color::linear_rgba(l.red * k, l.green * k, l.blue * k, l.alpha)
}

/// Trộn hai màu.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (a.to_linear(), b.to_linear());
    let t = t.clamp(0.0, 1.0);
    Color::linear_rgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

/// Bộ sinh số ngẫu nhiên có seed (xorshift64*): cảnh giống nhau mỗi lần mở.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn u(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// [0, 1)
    pub fn f(&mut self) -> f32 {
        (self.u() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[(self.u() % xs.len() as u64) as usize]
    }
}

fn hash2(x: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (z as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

/// Nhiễu giá trị 2D trong [0, 1].
pub fn noise(x: f32, z: f32) -> f32 {
    let (xi, zi) = (x.floor() as i32, z.floor() as i32);
    let (fx, fz) = (x - x.floor(), z - z.floor());
    let (sx, sz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let a = hash2(xi, zi);
    let b = hash2(xi + 1, zi);
    let c = hash2(xi, zi + 1);
    let d = hash2(xi + 1, zi + 1);
    let ab = a + (b - a) * sx;
    let cd = c + (d - c) * sx;
    ab + (cd - ab) * sz
}

/// Nhiễu nhiều tầng trong ~[0, 1].
pub fn fbm(x: f32, z: f32) -> f32 {
    let mut s = 0.0;
    let mut amp = 0.5;
    let (mut x, mut z) = (x, z);
    for _ in 0..4 {
        s += amp * noise(x, z);
        x = x * 2.03 + 17.1;
        z = z * 2.03 - 9.7;
        amp *= 0.5;
    }
    s / 0.9375
}

pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Lưới tam giác phẳng (mỗi tam giác có đỉnh riêng, pháp tuyến mặt), màu đỉnh tuyến tính.
#[derive(Default, Clone)]
pub struct Geo {
    pub pos: Vec<[f32; 3]>,
    pub nor: Vec<[f32; 3]>,
    pub col: Vec<[f32; 4]>,
}

fn lin(c: Color) -> [f32; 4] {
    c.to_linear().to_f32_array()
}

impl Geo {
    pub fn len(&self) -> usize {
        self.pos.len()
    }

    /// Tam giác ngược chiều kim đồng hồ khi nhìn từ mặt trước.
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, col: Color) {
        let cl = lin(col);
        self.tri3(a, b, c, cl, cl, cl);
    }

    pub fn tri_c(&mut self, a: Vec3, b: Vec3, c: Vec3, ca: Color, cb: Color, cc: Color) {
        self.tri3(a, b, c, lin(ca), lin(cb), lin(cc));
    }

    fn tri3(&mut self, a: Vec3, b: Vec3, c: Vec3, ca: [f32; 4], cb: [f32; 4], cc: [f32; 4]) {
        let n = (b - a).cross(c - a).normalize_or(Vec3::Y).to_array();
        self.pos.extend([a.to_array(), b.to_array(), c.to_array()]);
        self.nor.extend([n, n, n]);
        self.col.extend([ca, cb, cc]);
    }

    /// Tứ giác a-b-c-d ngược chiều kim đồng hồ.
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: Color) {
        self.tri(a, b, c, col);
        self.tri(a, c, d, col);
    }

    /// Tứ giác hai mặt.
    pub fn quad2(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: Color) {
        self.quad(a, b, c, d, col);
        self.quad(a, d, c, b, col);
    }

    pub fn tri2(&mut self, a: Vec3, b: Vec3, c: Vec3, col: Color) {
        self.tri(a, b, c, col);
        self.tri(a, c, b, col);
    }

    /// Hộp theo `t` (tâm, xoay), nửa kích thước `h`. Mặt trên sáng hơn một chút.
    pub fn bx(&mut self, t: Transform, h: Vec3, col: Color) {
        let m = t.to_matrix();
        let p = |x: f32, y: f32, z: f32| m.transform_point3(Vec3::new(x * h.x, y * h.y, z * h.z));
        let (top, side) = (shade(col, 1.08), col);
        // +y
        self.quad(p(-1., 1., 1.), p(1., 1., 1.), p(1., 1., -1.), p(-1., 1., -1.), top);
        // -y
        self.quad(p(-1., -1., -1.), p(1., -1., -1.), p(1., -1., 1.), p(-1., -1., 1.), shade(col, 0.7));
        // +z
        self.quad(p(-1., -1., 1.), p(1., -1., 1.), p(1., 1., 1.), p(-1., 1., 1.), side);
        // -z
        self.quad(p(1., -1., -1.), p(-1., -1., -1.), p(-1., 1., -1.), p(1., 1., -1.), side);
        // +x
        self.quad(p(1., -1., 1.), p(1., -1., -1.), p(1., 1., -1.), p(1., 1., 1.), side);
        // -x
        self.quad(p(-1., -1., -1.), p(-1., -1., 1.), p(-1., 1., 1.), p(-1., 1., -1.), side);
    }

    /// Hộp thẳng trục, tâm `c`.
    pub fn bxa(&mut self, c: Vec3, h: Vec3, col: Color) {
        self.bx(Transform::from_translation(c), h, col);
    }

    /// Thanh hộp nối hai điểm, tiết diện vuông cạnh `2r`.
    pub fn beam(&mut self, a: Vec3, b: Vec3, r: f32, col: Color) {
        let d = b - a;
        let len = d.length();
        if len < 1e-4 {
            return;
        }
        let rot = Quat::from_rotation_arc(Vec3::Y, d / len);
        self.bx(
            Transform::from_translation((a + b) * 0.5).with_rotation(rot),
            Vec3::new(r, len * 0.5, r),
            col,
        );
    }

    /// Ống/nón cụt nối `a` → `b`, `sides` cạnh, bán kính `ra` → `rb`. Có nắp nếu `cap`.
    #[allow(clippy::too_many_arguments)] // spike: tham số hình học tường minh dễ đọc hơn struct
    pub fn tube(&mut self, a: Vec3, b: Vec3, ra: f32, rb: f32, sides: usize, ca: Color, cb: Color, cap: bool) {
        let d = (b - a).normalize_or(Vec3::Y);
        let rot = Quat::from_rotation_arc(Vec3::Y, d);
        let ring = |c: Vec3, r: f32, i: usize| {
            let ang = i as f32 / sides as f32 * std::f32::consts::TAU;
            c + rot * Vec3::new(ang.cos() * r, 0.0, ang.sin() * r)
        };
        for i in 0..sides {
            let (a0, a1) = (ring(a, ra, i), ring(a, ra, i + 1));
            let (b0, b1) = (ring(b, rb, i), ring(b, rb, i + 1));
            self.tri_c(a0, b1, a1, ca, cb, ca);
            if rb > 1e-4 {
                self.tri_c(a0, b0, b1, ca, cb, cb);
            }
            if cap {
                if rb > 1e-4 {
                    self.tri(b, b1, b0, cb);
                }
                self.tri(a, a0, a1, shade(ca, 0.8));
            }
        }
    }

    /// Khối cầu low-poly (lưỡng thập nhị diện chia một lần), méo ngẫu nhiên. Tâm `c`, bán kính theo trục `r`.
    pub fn blob(&mut self, c: Vec3, r: Vec3, col: Color, jitter: f32, rng: &mut Rng) {
        let t = (1.0 + 5f32.sqrt()) / 2.0;
        let mut v: Vec<Vec3> = [
            (-1., t, 0.), (1., t, 0.), (-1., -t, 0.), (1., -t, 0.),
            (0., -1., t), (0., 1., t), (0., -1., -t), (0., 1., -t),
            (t, 0., -1.), (t, 0., 1.), (-t, 0., -1.), (-t, 0., 1.),
        ]
        .iter()
        .map(|&(x, y, z)| Vec3::new(x, y, z).normalize())
        .collect();
        let f: [[usize; 3]; 20] = [
            [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
            [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
            [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
            [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
        ];
        // Chia mỗi mặt thành 4.
        let mut faces = Vec::with_capacity(80);
        let mut mid = std::collections::BTreeMap::new();
        let mut midpoint = |i: usize, j: usize, v: &mut Vec<Vec3>| -> usize {
            let k = (i.min(j), i.max(j));
            *mid.entry(k).or_insert_with(|| {
                v.push(((v[i] + v[j]) * 0.5).normalize());
                v.len() - 1
            })
        };
        for [a, b, cc] in f {
            let ab = midpoint(a, b, &mut v);
            let bc = midpoint(b, cc, &mut v);
            let ca = midpoint(cc, a, &mut v);
            faces.extend([[a, ab, ca], [b, bc, ab], [cc, ca, bc], [ab, bc, ca]]);
        }
        let pts: Vec<Vec3> = v
            .iter()
            .map(|p| c + *p * r * (1.0 + rng.range(-jitter, jitter)))
            .collect();
        for [a, b, cc] in faces {
            let k = 0.92 + 0.16 * rng.f();
            // Mặt dưới tối hơn: giả lập tự che bóng.
            let up = ((v[a].y + v[b].y + v[cc].y) / 3.0 * 0.5 + 0.5).clamp(0.0, 1.0);
            self.tri(pts[a], pts[b], pts[cc], shade(col, k * (0.65 + 0.45 * up)));
        }
    }

    /// Nối thêm `o` sau khi biến đổi bởi `t`.
    pub fn add(&mut self, o: &Geo, t: Transform) {
        let m = t.to_matrix();
        let nm = Mat3::from_quat(t.rotation);
        self.pos.extend(o.pos.iter().map(|p| m.transform_point3(Vec3::from(*p)).to_array()));
        self.nor.extend(o.nor.iter().map(|n| (nm * Vec3::from(*n)).to_array()));
        self.col.extend_from_slice(&o.col);
    }

    pub fn mesh(&self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.pos.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.nor.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.col.clone())
    }
}
