//! Đồ dựng bằng hình khối Bevy ghép lại (gói Kenney không có): vở, cúp, ống bút, cốc, bảng
//! ghi chú và kệ huy chương. Màu lấy từ bảng màu của phòng như mô hình `.glb`. Gốc mỗi món ở
//! tâm đáy, mặt trước hướng +Z. Chữ trên note là ảnh điểm tự vẽ (`pixfont`), không cần font.

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use room_model::ObjectKind;

use super::ambient::SteamAnchor;
use super::models::palette_material;
use crate::palette::color_of;
use crate::pixfont::{NOTES, star_rgba, text_rgba};

/// Bộ đồ nghề dựng một món: mesh, vật liệu riêng (ghi lại để làm sáng) và thực thể cha.
pub struct Kit<'a, 'w, 's> {
    /// Lệnh Bevy.
    pub commands: &'a mut Commands<'w, 's>,
    /// Kho mesh.
    pub meshes: &'a mut Assets<Mesh>,
    /// Kho vật liệu.
    pub mats: &'a mut Assets<StandardMaterial>,
    /// Kho ảnh (texture note).
    pub images: &'a mut Assets<Image>,
    /// Thực thể cha của các mảnh.
    pub parent: Entity,
    /// Vật liệu đã tạo cho món này.
    pub made: Vec<Handle<StandardMaterial>>,
}

impl Kit<'_, '_, '_> {
    /// Một mảnh màu `role` của bảng màu.
    fn part(&mut self, mesh: impl Into<Mesh>, role: &str, t: Transform) -> Entity {
        let m = self.mats.add(palette_material(role));
        self.with_mat(mesh, m, t)
    }

    /// Một mảnh với vật liệu cho sẵn.
    fn with_mat(
        &mut self,
        mesh: impl Into<Mesh>,
        m: Handle<StandardMaterial>,
        t: Transform,
    ) -> Entity {
        self.made.push(m.clone());
        let mesh = self.meshes.add(mesh.into());
        self.commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(m),
                t,
                Pickable::IGNORE,
                ChildOf(self.parent),
            ))
            .id()
    }

    /// Tấm phẳng mang ảnh RGBA `w × h` (giấy note, vé).
    fn sheet(&mut self, size: Vec2, rgba: Vec<u8>, w: u32, h: u32, t: Transform) -> Entity {
        let img = Image::new(
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        let m = self.mats.add(StandardMaterial {
            base_color_texture: Some(self.images.add(img)),
            perceptual_roughness: 0.95,
            reflectance: 0.2,
            ..default()
        });
        let e = self.with_mat(Rectangle::from_size(size), m, t);
        self.commands.entity(e).insert(NotShadowCaster);
        e
    }
}

fn at(x: f32, y: f32, z: f32) -> Transform {
    Transform::from_xyz(x, y, z)
}

/// Màu bảng màu dạng RGBA 8 bit (cho ảnh note).
fn rgba(role: &str) -> [u8; 4] {
    let c = color_of(role);
    [
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
        255,
    ]
}

/// Trộn hai màu RGBA (`t` = 0: `a`).
fn mix(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2]), 255]
}

/// Dựng món `kind` (loại không có `.glb`) làm con của `kit.parent`.
pub fn build(kit: &mut Kit, kind: ObjectKind) {
    match kind {
        ObjectKind::Notebook => notebook(kit),
        ObjectKind::Trophy => trophy(kit),
        ObjectKind::PencilCup => pencil_cup(kit),
        ObjectKind::Mug => mug(kit),
        ObjectKind::NoteBoard => note_board(kit),
        ObjectKind::MedalShelf => medal_shelf(kit),
        ObjectKind::Laptop | ObjectKind::DeskLamp | ObjectKind::Plant | ObjectKind::Books => {}
    }
}

/// Vở mở nằm: bìa màu rèm, ruột giấy, gáy gỗ tối, một cây bút chì.
fn notebook(k: &mut Kit) {
    k.part(
        Cuboid::new(0.21, 0.006, 0.27),
        "curtain",
        at(0.0, 0.003, 0.0),
    );
    k.part(Cuboid::new(0.20, 0.014, 0.26), "paper", at(0.0, 0.013, 0.0));
    k.part(
        Cuboid::new(0.21, 0.004, 0.27),
        "curtain",
        at(0.0, 0.022, 0.0),
    );
    k.part(
        Cuboid::new(0.014, 0.026, 0.27),
        "wood_dark",
        at(-0.104, 0.013, 0.0),
    );
    // Nhãn giấy trên bìa.
    k.part(
        Cuboid::new(0.11, 0.002, 0.06),
        "paper",
        at(0.01, 0.025, -0.05),
    );
    k.part(
        Cylinder::new(0.005, 0.16),
        "gold",
        Transform::from_xyz(0.03, 0.03, 0.06).with_rotation(
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * Quat::from_rotation_x(0.5),
        ),
    );
}

/// Cúp vàng trên đế gỗ.
fn trophy(k: &mut Kit) {
    k.part(
        Cuboid::new(0.09, 0.035, 0.09),
        "wood_dark",
        at(0.0, 0.0175, 0.0),
    );
    k.part(Cylinder::new(0.03, 0.012), "gold", at(0.0, 0.041, 0.0));
    k.part(Cylinder::new(0.011, 0.05), "gold", at(0.0, 0.072, 0.0));
    k.part(
        ConicalFrustum {
            radius_top: 0.055,
            radius_bottom: 0.022,
            height: 0.075,
        },
        "gold",
        at(0.0, 0.134, 0.0),
    );
    for s in [-1.0, 1.0] {
        k.part(
            Torus::new(0.016, 0.024),
            "gold",
            Transform::from_xyz(s * 0.052, 0.138, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        );
    }
    // Ngôi sao nhỏ trên đế (một khối dẹt nghiêng 45°).
    k.part(
        Cuboid::new(0.022, 0.022, 0.004),
        "paper",
        Transform::from_xyz(0.0, 0.018, 0.046).with_rotation(Quat::from_rotation_z(0.785)),
    );
}

/// Ống bút xanh màn hình với ba cây bút chì.
fn pencil_cup(k: &mut Kit) {
    k.part(Cylinder::new(0.035, 0.09), "screen", at(0.0, 0.045, 0.0));
    for (i, role) in ["gold", "curtain", "leaf"].into_iter().enumerate() {
        let a = i as f32 * 2.1;
        let tilt = Quat::from_rotation_z(0.18 * a.cos()) * Quat::from_rotation_x(0.18 * a.sin());
        k.part(
            Cylinder::new(0.0045, 0.15),
            role,
            Transform::from_xyz(0.012 * a.cos(), 0.1, 0.012 * a.sin()).with_rotation(tilt),
        );
    }
}

/// Cốc gốm có quai; hơi nước (`ambient.rs`) bốc lên từ miệng cốc.
fn mug(k: &mut Kit) {
    k.part(Cylinder::new(0.038, 0.09), "ceramic", at(0.0, 0.045, 0.0));
    k.part(Cylinder::new(0.033, 0.002), "ink", at(0.0, 0.084, 0.0));
    k.part(
        Torus::new(0.017, 0.026),
        "ceramic",
        Transform::from_xyz(0.04, 0.047, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    );
    let parent = k.parent;
    k.commands.spawn((
        SteamAnchor,
        Transform::from_xyz(0.0, 0.09, 0.0),
        Visibility::default(),
        ChildOf(parent),
    ));
}

/// Bảng ghi chú bằng bần: khung gỗ, ba tờ note in sẵn, ghim.
fn note_board(k: &mut Kit) {
    let c = 0.20; // tâm bảng cao hơn chân 20 cm
    k.part(Cuboid::new(0.62, 0.42, 0.02), "wood_dark", at(0.0, c, 0.01));
    k.part(
        Cuboid::new(0.57, 0.37, 0.01),
        "wood_light",
        at(0.0, c, 0.022),
    );
    let ink = rgba("ink");
    let paper = rgba("paper");
    let notes = [
        (
            text_rgba(NOTES[0], 128, 128, 2, ink, paper),
            -0.18,
            0.03,
            0.06,
        ),
        (
            text_rgba(NOTES[1], 128, 128, 2, ink, mix(paper, rgba("gold"), 0.35)),
            0.02,
            0.0,
            -0.05,
        ),
        (
            star_rgba(64, rgba("gold"), mix(paper, rgba("ceramic"), 0.3)),
            0.20,
            0.04,
            0.08,
        ),
    ];
    let pins = ["curtain", "screen", "leaf"];
    for (i, (img, x, dy, rot)) in notes.into_iter().enumerate() {
        let side = if i == 2 { 64 } else { 128 };
        let size = if i == 2 { 0.12 } else { 0.16 };
        k.sheet(
            Vec2::splat(size),
            img,
            side,
            side,
            Transform::from_xyz(x, c + dy, 0.0285).with_rotation(Quat::from_rotation_z(rot)),
        );
        k.part(
            Sphere::new(0.008),
            pins[i],
            at(x, c + dy + size / 2.0 - 0.015, 0.034),
        );
    }
}

/// Kệ huy chương: tấm gỗ, bốn móc dồn về phía cửa sổ (nửa phải của kệ ra ngoài khung hình ở
/// màn 360 px); móc đầu, sát cửa sổ, treo vé "Ngày đầu tới lớp", ba móc sau treo bóng mờ huy
/// chương chưa mở.
fn medal_shelf(k: &mut Kit) {
    let c = 0.20;
    k.part(
        Cuboid::new(0.60, 0.34, 0.015),
        "wood_light",
        at(0.0, c, 0.0075),
    );
    k.part(
        Cuboid::new(0.60, 0.03, 0.08),
        "wood_dark",
        at(0.0, c - 0.16, 0.04),
    );
    let hook_y = c + 0.12;
    let ghost = k.mats.add(StandardMaterial {
        base_color: {
            let [r, g, b] = color_of("ink");
            Color::srgba(r, g, b, 0.35)
        },
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.9,
        ..default()
    });
    for i in 0..4 {
        // Vé rộng 0,12 m ở móc đầu; huy chương (đường kính 0,07 m) xếp sát nhau sau đó.
        let x = if i == 0 {
            -0.22
        } else {
            -0.12 + (i - 1) as f32 * 0.075
        };
        k.part(
            Cylinder::new(0.005, 0.03),
            "metal",
            Transform::from_xyz(x, hook_y, 0.03)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        );
        if i > 0 {
            // Dây đeo và mặt huy chương, mờ.
            let ribbon = k.with_mat(
                Cuboid::new(0.025, 0.09, 0.003),
                ghost.clone(),
                at(x, hook_y - 0.05, 0.035),
            );
            let disc = k.with_mat(
                Cylinder::new(0.035, 0.006),
                ghost.clone(),
                Transform::from_xyz(x, hook_y - 0.125, 0.036)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            );
            k.commands.entity(ribbon).insert(NotShadowCaster);
            k.commands.entity(disc).insert(NotShadowCaster);
        } else {
            let img = text_rgba(
                NOTES[2],
                128,
                80,
                2,
                rgba("ink"),
                mix(rgba("paper"), rgba("gold"), 0.55),
            );
            k.sheet(
                Vec2::new(0.12, 0.075),
                img,
                128,
                80,
                Transform::from_xyz(x, hook_y - 0.06, 0.035)
                    .with_rotation(Quat::from_rotation_z(-0.06)),
            );
        }
    }
}
