//! Dựng phòng: vỏ phòng bằng hình khối (sàn, tường, cửa sổ, rèm, trời sau cửa sổ), bàn và đồ
//! trang trí từ mô hình CC0 (`models.rs`), đồ không có mô hình dựng bằng hình khối
//! (`props.rs`), và đồ theo bảng `ROOM`. Mỗi chỗ có một khối va chạm vô hình to hơn vật để
//! bấm trúng. Màu theo bảng màu chung (`palette.rs`).

use bevy::camera::Hdr;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::Tonemapping;
use room_model::{ObjectKind, ROOM};

use super::ambient::Curtain;
use super::look::{DeskLampLight, SkyPane, SlotVisual, Star, SunLight};
use super::models::{
    self, BOOKCASE, DESK, Glow, ModelSpec, Pending, RUG, Restyle, SHELF_BOOKS, palette_material,
};
use super::pick::SlotTag;
use super::props::{self, Kit};
use crate::palette::color_of;
use crate::poses::{FOV_Y, cm, collider_center, collider_half, home_pose};

/// Màu ánh sáng (không phải vật liệu, nên không nằm trong bảng màu vật liệu).
mod light {
    use bevy::color::Color;

    /// Ánh đèn bàn.
    pub const BULB: Color = Color::srgb_u8(0xFF, 0xD9, 0xA0);
    /// Màu nền sau phòng (không nên thấy).
    pub const VOID: Color = Color::srgb_u8(0x2B, 0x22, 0x1C);
    /// Ánh sáng chung, ấm.
    pub const AMBIENT: Color = Color::srgb_u8(0xFF, 0xE8, 0xD0);
}

/// Đèn bàn: nguồn sáng nằm trong chao, trên chỗ đặt đèn 25 cm.
pub const LAMP_ABOVE: Vec3 = Vec3::new(0.0, 0.25, 0.0);

/// Cửa sổ trên tường sau: tâm và kích thước ô kính (mét), giữa bảng ghi chú và kệ huy chương.
const WINDOW_CENTER: Vec3 = Vec3::new(2.20, 1.60, 0.0);
const WINDOW_SIZE: Vec2 = Vec2::new(0.70, 1.10);
/// Bề dày khung cửa sổ.
const FRAME: f32 = 0.05;

fn srgb(role: &str) -> Color {
    let [r, g, b] = color_of(role);
    Color::srgb(r, g, b)
}

/// Màu tự sáng khi vật lối vào được chọn.
fn glow_of(kind: ObjectKind) -> LinearRgba {
    let role = match kind {
        ObjectKind::Notebook => "curtain",
        ObjectKind::Laptop => "screen",
        ObjectKind::Trophy => "gold",
        _ => "paper",
    };
    LinearRgba::from(srgb(role)) * 0.6
}

/// Dựng toàn bộ cảnh, camera và đèn.
pub fn spawn(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(ClearColor(light::VOID));
    commands.insert_resource(GlobalAmbientLight {
        color: light::AMBIENT,
        brightness: 600.0,
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

    shell(&mut commands, &mut meshes, &mut mats);
    window(&mut commands, &mut meshes, &mut mats);
    furniture(&mut commands, &assets);
    lights(&mut commands);
    slots(&mut commands, &assets, &mut meshes, &mut mats, &mut images);
}

/// Khối đặc một màu `color`, không bấm được.
fn block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    size: Vec3,
    at: Vec3,
    m: StandardMaterial,
) -> Entity {
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(mats.add(m)),
            Transform::from_translation(at),
        ))
        .id()
}

/// Sàn, tường sau (z = 0), tường trái (x = 0), chân tường. Gốc ở góc sàn nơi hai tường gặp nhau.
fn shell(commands: &mut Commands, meshes: &mut Assets<Mesh>, mats: &mut Assets<StandardMaterial>) {
    let floor = palette_material("wood_light");
    let wall = palette_material("wall");
    let mut side = palette_material("wall");
    side.base_color = side.base_color.darker(0.06);
    let base = palette_material("wood_dark");
    let mut b = |size, at, m| block(commands, meshes, mats, size, at, m);
    b(Vec3::new(4.0, 0.02, 3.0), Vec3::new(2.0, -0.01, 1.5), floor);
    b(Vec3::new(4.0, 2.7, 0.04), Vec3::new(2.0, 1.35, -0.02), wall);
    b(Vec3::new(0.04, 2.7, 3.0), Vec3::new(-0.02, 1.35, 1.5), side);
    b(
        Vec3::new(4.0, 0.08, 0.015),
        Vec3::new(2.0, 0.04, 0.0075),
        base.clone(),
    );
    b(
        Vec3::new(0.015, 0.08, 3.0),
        Vec3::new(0.0075, 0.04, 1.5),
        base,
    );
}

/// Cửa sổ: khung, song, bậu (mèo nằm ở đây), rèm hai bên, trời sau kính, sao và đèn thành phố.
fn window(commands: &mut Commands, meshes: &mut Assets<Mesh>, mats: &mut Assets<StandardMaterial>) {
    let c = WINDOW_CENTER;
    let (hw, hh) = (WINDOW_SIZE.x / 2.0, WINDOW_SIZE.y / 2.0);
    let z = 0.03;
    let frame = palette_material("paper");
    let pieces = [
        // trái, phải, trên, dưới, song dọc, song ngang
        (
            Vec3::new(FRAME, WINDOW_SIZE.y + 2.0 * FRAME, 0.06),
            Vec3::new(-hw - FRAME / 2.0, 0.0, z),
        ),
        (
            Vec3::new(FRAME, WINDOW_SIZE.y + 2.0 * FRAME, 0.06),
            Vec3::new(hw + FRAME / 2.0, 0.0, z),
        ),
        (
            Vec3::new(WINDOW_SIZE.x, FRAME, 0.06),
            Vec3::new(0.0, hh + FRAME / 2.0, z),
        ),
        (
            Vec3::new(WINDOW_SIZE.x, FRAME, 0.06),
            Vec3::new(0.0, -hh - FRAME / 2.0, z),
        ),
        (
            Vec3::new(0.025, WINDOW_SIZE.y, 0.04),
            Vec3::new(0.0, 0.0, z),
        ),
        (Vec3::new(WINDOW_SIZE.x, 0.04, 0.04), Vec3::new(0.0, 0.0, z)),
    ];
    for (size, off) in pieces {
        block(commands, meshes, mats, size, c + off, frame.clone());
    }
    // Bậu cửa: mặt trên ở y = 1,00 m.
    block(
        commands,
        meshes,
        mats,
        Vec3::new(WINDOW_SIZE.x + 0.18, 0.03, 0.18),
        Vec3::new(c.x, c.y - hh - FRAME - 0.015, 0.09),
        palette_material("wood_light"),
    );
    // Thanh treo rèm và hai tấm rèm xoay quanh mép trên.
    let rod_y = c.y + hh + FRAME + 0.06;
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.012, WINDOW_SIZE.x + 0.40))),
        MeshMaterial3d(mats.add(palette_material("wood_dark"))),
        Transform::from_xyz(c.x, rod_y, 0.08)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    ));
    let drape = meshes.add(Cuboid::new(0.17, 1.30, 0.02));
    for (i, dx) in [-(hw + 0.07), hw + 0.07].into_iter().enumerate() {
        let pivot = commands
            .spawn((
                Curtain {
                    phase: i as f32 * 1.7,
                },
                Transform::from_xyz(c.x + dx, rod_y - 0.01, 0.08),
                Visibility::default(),
            ))
            .id();
        commands.spawn((
            Mesh3d(drape.clone()),
            MeshMaterial3d(mats.add(palette_material("curtain"))),
            Transform::from_xyz(0.0, -0.65, 0.0),
            Pickable::IGNORE,
            ChildOf(pivot),
        ));
    }

    // Trời sau cửa sổ: hai tấm phẳng tự sáng (đỉnh trời, chân trời), song ngang ở giữa.
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
            Transform::from_translation(c + Vec3::new(0.0, dy, 0.0025)),
            NotShadowCaster,
        ));
    }
    // Sao (trên) và đèn thành phố xa (dưới): hiện theo `SkyState::stars` (`look::sky`).
    let dots = [
        (
            Color::srgba(1.0, 0.97, 0.85, 0.0),
            0.012,
            &[
                (-0.25, 0.48),
                (-0.10, 0.36),
                (0.05, 0.50),
                (0.18, 0.30),
                (0.27, 0.44),
                (-0.20, 0.20),
                (0.10, 0.12),
            ][..],
        ),
        (
            Color::srgba(1.0, 0.78, 0.45, 0.0),
            0.016,
            &[
                (-0.30, -0.50),
                (-0.22, -0.47),
                (-0.12, -0.52),
                (-0.02, -0.45),
                (0.08, -0.50),
                (0.20, -0.48),
                (0.29, -0.52),
                (-0.17, -0.40),
                (0.14, -0.42),
            ][..],
        ),
    ];
    for (color, size, at) in dots {
        let m = mats.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let mesh = meshes.add(Cuboid::new(size, size, 0.004));
        for &(x, y) in at {
            commands.spawn((
                Star,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(m.clone()),
                Transform::from_translation(c + Vec3::new(x, y, 0.008)),
                NotShadowCaster,
            ));
        }
    }
}

/// Đặt một mô hình không thuộc `ROOM` ở `at`, quay `yaw_deg`.
fn place(
    commands: &mut Commands,
    assets: &AssetServer,
    spec: ModelSpec,
    at: Vec3,
    yaw_deg: f32,
) -> Entity {
    let root = commands
        .spawn((
            Transform::from_translation(at)
                .with_rotation(Quat::from_rotation_y(yaw_deg.to_radians())),
            Visibility::default(),
        ))
        .id();
    models::spawn_model(commands, assets, root, spec, Restyle::default())
}

/// Bàn (khung đầu chờ nó), kệ sách, thảm, sách trên kệ.
fn furniture(commands: &mut Commands, assets: &AssetServer) {
    let desk = place(commands, assets, DESK, Vec3::new(2.05, 0.0, 0.70), 0.0);
    commands.entity(desk).insert(Pending);
    place(commands, assets, BOOKCASE, Vec3::new(0.80, 0.0, 0.16), 0.0);
    place(commands, assets, RUG, Vec3::new(2.05, 0.0, 1.10), 0.0);
    place(
        commands,
        assets,
        SHELF_BOOKS,
        Vec3::new(0.74, 1.056, 0.15),
        0.0,
    );
}

/// Ánh trời qua cửa sổ (nguồn phụ) và đèn bàn (nguồn chính); `look::sky` đặt màu, độ sáng.
fn lights(commands: &mut Commands) {
    commands.spawn((
        SunLight,
        DirectionalLight::default(),
        Transform::from_xyz(2.2, 2.6, -1.2).looking_at(Vec3::new(2.0, 0.7, 1.0), Vec3::Y),
    ));
    if let Some(lamp) = ROOM.iter().find(|s| s.kind == ObjectKind::DeskLamp) {
        commands.spawn((
            DeskLampLight,
            PointLight {
                color: light::BULB,
                range: 6.0,
                ..default()
            },
            Transform::from_translation(cm(lamp.pos_cm) + LAMP_ABOVE),
        ));
    }
}

/// Đồ theo bảng `ROOM`: vật nhìn thấy (vật liệu riêng, để làm sáng) và khối va chạm vô hình.
fn slots(
    commands: &mut Commands,
    assets: &AssetServer,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let hit_mat = mats.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    for slot in ROOM {
        let rot = Quat::from_rotation_y((slot.yaw_deg as f32).to_radians());
        let base = Transform::from_translation(cm(slot.pos_cm)).with_rotation(rot);
        let root = commands
            .spawn((
                SlotVisual {
                    id: slot.id,
                    base,
                    glow: glow_of(slot.kind),
                },
                base,
                Visibility::default(),
            ))
            .id();
        if let Some(spec) = models::model_for(slot.kind) {
            let lamp = slot.kind == ObjectKind::DeskLamp;
            let style = Restyle {
                glow: lamp.then_some("lamp"),
                no_shadow: lamp,
            };
            let model = models::spawn_model(commands, assets, root, spec, style);
            commands.entity(root).insert(Glow::default());
            // Khung đầu chờ vật lối vào (quyết định 10); đồ trang trí hiện dần sau.
            if slot.zone.is_some() {
                commands.entity(model).insert(Pending);
            }
        } else {
            let mut kit = Kit {
                commands,
                meshes,
                mats,
                images,
                parent: root,
                made: Vec::new(),
            };
            props::build(&mut kit, slot.kind);
            let made = kit.made;
            commands.entity(root).insert(Glow(made));
        }
        let half = collider_half(slot);
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
