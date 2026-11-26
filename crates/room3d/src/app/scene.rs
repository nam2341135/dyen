//! Cảnh tạm bằng hình khối (Task 7 thay bằng mô hình CC0): tường, sàn, bàn, cửa sổ, đồ theo
//! bảng `ROOM`. Mỗi chỗ có một khối va chạm vô hình to hơn vật để bấm trúng.

use bevy::camera::Hdr;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::Tonemapping;
use room_model::{ObjectKind, ROOM};

use super::look::{DeskLampLight, SkyPane, SlotVisual, Star, SunLight};
use super::pick::SlotTag;
use crate::poses::{FOV_Y, cm, collider_center, collider_half, home_pose};

/// Bảng màu của phòng (sRGB). Giữ đồng bộ với bảng trong `crates/room3d/README.md`.
mod palette {
    use bevy::color::Color;

    pub const WALL: Color = Color::srgb_u8(0xE8, 0xD8, 0xC4);
    pub const WALL_SIDE: Color = Color::srgb_u8(0xD9, 0xC6, 0xB0);
    pub const FLOOR: Color = Color::srgb_u8(0x8A, 0x6A, 0x52);
    pub const DESK: Color = Color::srgb_u8(0xB0, 0x80, 0x5A);
    pub const FRAME: Color = Color::srgb_u8(0xF7, 0xF3, 0xEA);
    pub const NOTEBOOK: Color = Color::srgb_u8(0xE0, 0x7A, 0x5F);
    pub const LAPTOP: Color = Color::srgb_u8(0x5B, 0x7D, 0xB1);
    pub const TROPHY: Color = Color::srgb_u8(0xE3, 0xB2, 0x3C);
    pub const LAMP: Color = Color::srgb_u8(0xF2, 0xE3, 0xC6);
    pub const PLANT: Color = Color::srgb_u8(0x6A, 0x99, 0x4E);
    pub const PENCIL_CUP: Color = Color::srgb_u8(0x81, 0xB2, 0x9A);
    pub const MUG: Color = Color::srgb_u8(0xF4, 0xF1, 0xDE);
    pub const BOOKS: Color = Color::srgb_u8(0x9C, 0x66, 0x44);
    pub const CORK: Color = Color::srgb_u8(0xC1, 0x9A, 0x6B);
    pub const SHELF: Color = Color::srgb_u8(0x7F, 0x55, 0x39);
    pub const BULB: Color = Color::srgb_u8(0xFF, 0xD9, 0xA0);
    /// Màu nền sau phòng (không nên thấy).
    pub const VOID: Color = Color::srgb_u8(0x2B, 0x22, 0x1C);
    /// Ánh sáng chung, ấm.
    pub const AMBIENT: Color = Color::srgb_u8(0xFF, 0xE8, 0xD0);
}

/// Màu tạm của từng loại vật.
fn color_of(kind: ObjectKind) -> Color {
    use palette::*;
    match kind {
        ObjectKind::Notebook => NOTEBOOK,
        ObjectKind::Laptop => LAPTOP,
        ObjectKind::Trophy => TROPHY,
        ObjectKind::DeskLamp => LAMP,
        ObjectKind::Plant => PLANT,
        ObjectKind::PencilCup => PENCIL_CUP,
        ObjectKind::Mug => MUG,
        ObjectKind::Books => BOOKS,
        ObjectKind::NoteBoard => CORK,
        ObjectKind::MedalShelf => SHELF,
    }
}

/// Vị trí đèn bàn: trên chỗ đặt đèn 35 cm.
pub const LAMP_ABOVE: Vec3 = Vec3::new(0.0, 0.35, 0.0);

/// Cửa sổ trên tường sau: tâm và kích thước (mét), giữa bảng ghi chú và kệ huy chương.
const WINDOW_CENTER: Vec3 = Vec3::new(2.20, 1.60, 0.0);
const WINDOW_SIZE: Vec2 = Vec2::new(0.70, 1.10);

/// Dựng toàn bộ cảnh tạm, camera và đèn.
pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ClearColor(palette::VOID));
    commands.insert_resource(GlobalAmbientLight {
        color: palette::AMBIENT,
        brightness: 250.0,
        ..default()
    });

    let home = home_pose(room_model::Viewport {
        width: 1280,
        height: 720,
    });
    commands.spawn((
        Camera3d::default(),
        // Mặc định TonyMcMapface cần feature `tonemapping_luts` (thêm LUT + zstd); ACES không cần.
        Tonemapping::AcesFitted,
        Hdr,
        Msaa::Off,
        Projection::Perspective(PerspectiveProjection {
            fov: FOV_Y,
            ..default()
        }),
        Transform::from_translation(home.eye).looking_at(home.look, Vec3::Y),
    ));

    let mut solid = |commands: &mut Commands, size: Vec3, at: Vec3, color: Color| {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_translation(at),
        ));
    };
    // Sàn, tường sau (z = 0), tường trái (x = 0). Gốc ở góc sàn nơi hai tường gặp nhau.
    solid(
        &mut commands,
        Vec3::new(4.0, 0.02, 3.0),
        Vec3::new(2.0, -0.01, 1.5),
        palette::FLOOR,
    );
    solid(
        &mut commands,
        Vec3::new(4.0, 2.7, 0.04),
        Vec3::new(2.0, 1.35, -0.02),
        palette::WALL,
    );
    solid(
        &mut commands,
        Vec3::new(0.04, 2.7, 3.0),
        Vec3::new(-0.02, 1.35, 1.5),
        palette::WALL_SIDE,
    );
    // Bàn: mặt bàn cao 75 cm, x 1,15..2,95, z 0,35..1,05; bốn chân.
    solid(
        &mut commands,
        Vec3::new(1.8, 0.04, 0.7),
        Vec3::new(2.05, 0.73, 0.70),
        palette::DESK,
    );
    for (x, z) in [(1.18, 0.38), (2.92, 0.38), (1.18, 1.02), (2.92, 1.02)] {
        solid(
            &mut commands,
            Vec3::new(0.05, 0.71, 0.05),
            Vec3::new(x, 0.355, z),
            palette::DESK,
        );
    }
    // Khung cửa sổ, sát tường sau.
    solid(
        &mut commands,
        Vec3::new(WINDOW_SIZE.x + 0.08, WINDOW_SIZE.y + 0.08, 0.02),
        WINDOW_CENTER + Vec3::new(0.0, 0.0, 0.01),
        palette::FRAME,
    );

    // Trời sau cửa sổ: hai tấm phẳng tự sáng (đỉnh trời, chân trời), có thanh khung ở giữa.
    let pane = Vec2::new(WINDOW_SIZE.x, WINDOW_SIZE.y / 2.0 - 0.02);
    for top in [true, false] {
        let dy = if top { 1.0 } else { -1.0 } * (pane.y / 2.0 + 0.02);
        commands.spawn((
            SkyPane { top },
            Mesh3d(meshes.add(Cuboid::new(pane.x, pane.y, 0.005))),
            MeshMaterial3d(mats.add(StandardMaterial {
                unlit: true,
                ..default()
            })),
            Transform::from_translation(WINDOW_CENTER + Vec3::new(0.0, dy, 0.025)),
            NotShadowCaster,
        ));
    }
    let star_mat = mats.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.97, 0.85, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let star_mesh = meshes.add(Cuboid::new(0.012, 0.012, 0.004));
    for (x, y) in [
        (-0.25, 0.48),
        (-0.10, 0.36),
        (0.05, 0.50),
        (0.18, 0.30),
        (0.27, 0.44),
        (-0.20, 0.20),
    ] {
        commands.spawn((
            Star,
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_mat.clone()),
            Transform::from_translation(WINDOW_CENTER + Vec3::new(x, y, 0.03)),
            NotShadowCaster,
        ));
    }

    // Ánh trời qua cửa sổ (nguồn phụ) và đèn bàn (nguồn chính); `look::sky` đặt màu, độ sáng.
    commands.spawn((
        SunLight,
        DirectionalLight::default(),
        Transform::from_xyz(2.2, 2.6, -1.2).looking_at(Vec3::new(2.0, 0.7, 1.0), Vec3::Y),
    ));
    let lamp_slot = ROOM.iter().find(|s| s.kind == ObjectKind::DeskLamp);
    if let Some(lamp) = lamp_slot {
        let at = cm(lamp.pos_cm) + LAMP_ABOVE;
        commands.spawn((
            DeskLampLight,
            PointLight {
                color: palette::BULB,
                range: 6.0,
                ..default()
            },
            Transform::from_translation(at),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.03))),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: palette::BULB,
                emissive: LinearRgba::rgb(6.0, 4.2, 2.2),
                ..default()
            })),
            Transform::from_translation(at + Vec3::new(0.0, 0.03, 0.0)),
            NotShadowCaster,
            Pickable::IGNORE,
        ));
    }

    // Đồ theo bảng ROOM: vật nhìn thấy (vật liệu riêng, để làm sáng) và khối va chạm vô hình.
    let hit_mat = mats.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    for slot in ROOM {
        let rot = Quat::from_rotation_y((slot.yaw_deg as f32).to_radians());
        let half = collider_half(slot);
        let vis_half = half * 0.7;
        let base = Transform::from_translation(cm(slot.pos_cm) + Vec3::new(0.0, vis_half.y, 0.0))
            .with_rotation(rot);
        let color = color_of(slot.kind);
        commands.spawn((
            SlotVisual {
                id: slot.id,
                base,
                glow: LinearRgba::from(color) * 0.6,
            },
            Mesh3d(meshes.add(Cuboid::from_size(vis_half * 2.0))),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.8,
                ..default()
            })),
            base,
            Pickable::IGNORE,
        ));
        commands.spawn((
            SlotTag(slot.id),
            Mesh3d(meshes.add(Cuboid::from_size(half * 2.0))),
            MeshMaterial3d(hit_mat.clone()),
            Transform::from_translation(collider_center(slot)).with_rotation(rot),
            Pickable::default(),
            NotShadowCaster,
        ));
    }
}
