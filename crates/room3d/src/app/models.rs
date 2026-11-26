//! Mô hình CC0 (`.glb`, bảng ở README) và bước "đồng bộ phong cách" lúc chạy: khi một mô hình
//! nạp xong (`WorldInstanceReady`), mọi vật liệu của nó được thay bằng màu phẳng của bảng màu
//! theo `role_for(tên vật liệu)`, bỏ texture gốc. Mỗi mesh nhận vật liệu riêng, nên làm sáng
//! một vật không lây sang vật khác dùng cùng file.

use bevy::gltf::GltfMaterialName;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use room_model::ObjectKind;

use crate::palette::{color_of, role_for};

/// Một mô hình `.glb`: file, cách đặt chân mô hình về gốc và cỡ.
#[derive(Clone, Copy, Debug)]
pub struct ModelSpec {
    /// Tên file trong `assets/` (Bevy đọc từ `room/`).
    pub file: &'static str,
    /// Tâm đáy của mô hình trong toạ độ của file (mét, sau biến đổi nút gốc của file).
    pub center: Vec3,
    /// Phóng (có thể khác nhau theo trục: bàn).
    pub scale: Vec3,
    /// Xoay thêm quanh Y (độ) để mặt trước hướng +Z.
    pub yaw_deg: f32,
}

impl ModelSpec {
    /// Transform của nút con mang mô hình: đặt tâm đáy vào gốc, phóng, rồi xoay.
    pub fn child_transform(&self) -> Transform {
        let m = Mat4::from_rotation_y(self.yaw_deg.to_radians())
            * Mat4::from_scale(self.scale)
            * Mat4::from_translation(-self.center);
        Transform::from_matrix(m)
    }
}

const fn spec(file: &'static str, center: [f32; 3], scale: [f32; 3], yaw_deg: f32) -> ModelSpec {
    ModelSpec {
        file,
        center: Vec3::from_array(center),
        scale: Vec3::from_array(scale),
        yaw_deg,
    }
}

/// Mô hình `.glb` của từng loại vật; `None`: dựng bằng hình khối trong code (`props.rs`),
/// vì gói Kenney không có vở, cúp, cốc, ống bút, bảng ghi chú, kệ huy chương.
pub fn model_for(kind: ObjectKind) -> Option<ModelSpec> {
    let s = match kind {
        ObjectKind::Laptop => spec("laptop.glb", [0.132, 0.0, -0.120], [1.0; 3], 0.0),
        ObjectKind::DeskLamp => spec("lampSquareTable.glb", [0.06, 0.0, -0.06], [1.2; 3], 0.0),
        ObjectKind::Plant => spec("plantSmall2.glb", [0.0; 3], [1.5; 3], 0.0),
        ObjectKind::Books => spec("books.glb", [0.075, 0.0, -0.047], [1.3; 3], 0.0),
        ObjectKind::Notebook
        | ObjectKind::Trophy
        | ObjectKind::PencilCup
        | ObjectKind::Mug
        | ObjectKind::NoteBoard
        | ObjectKind::MedalShelf => return None,
    };
    Some(s)
}

/// Bàn học: mặt bàn 1,8 × 0,7 m, cao 75 cm (phóng không đều từ `desk.glb`).
pub const DESK: ModelSpec = spec("desk.glb", [0.357, 0.0, -0.102], [2.452, 1.953, 1.259], 0.0);
/// Kệ sách cạnh bàn, sát tường sau.
pub const BOOKCASE: ModelSpec = spec("bookcaseOpen.glb", [0.2, 0.0, -0.125], [1.2; 3], 0.0);
/// Thảm dưới bàn.
pub const RUG: ModelSpec = spec("rugRectangle.glb", [0.785, 0.0, -0.46], [1.2; 3], 0.0);
/// Chồng sách trên kệ (dùng lại `books.glb`).
pub const SHELF_BOOKS: ModelSpec = spec("books.glb", [0.075, 0.0, -0.047], [1.6; 3], 90.0);

/// Mô hình chưa nạp xong mà `FirstFrame` phải chờ (bàn, máy tính: quyết định 10).
#[derive(Component)]
pub struct Pending;

/// Gắn vào gốc mô hình: cách đồng bộ phong cách khi nạp xong.
#[derive(Component, Clone, Copy, Default)]
pub struct Restyle {
    /// Vật liệu có tên chứa chuỗi này tự sáng ấm (chao đèn bàn).
    pub glow: Option<&'static str>,
    /// Không đổ bóng (chao đèn: đèn nằm bên trong).
    pub no_shadow: bool,
}

/// Vật liệu riêng của một vật (để làm sáng khi chọn); mô hình điền khi nạp xong.
#[derive(Component, Default)]
pub struct Glow(pub Vec<Handle<StandardMaterial>>);

/// Vật liệu bảng màu của vai trò `role`: màu phẳng, mờ (lofi), không texture.
pub fn palette_material(role: &str) -> StandardMaterial {
    let [r, g, b] = color_of(role);
    let (roughness, metallic) = match role {
        "gold" => (0.45, 0.3),
        "screen" | "ceramic" => (0.5, 0.0),
        _ => (0.85, 0.0),
    };
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        perceptual_roughness: roughness,
        metallic,
        reflectance: 0.3,
        ..default()
    }
}

/// Sinh thực thể mô hình `spec` làm con của `parent`. Trả thực thể gốc của mô hình.
pub fn spawn_model(
    commands: &mut Commands,
    assets: &AssetServer,
    parent: Entity,
    spec: ModelSpec,
    restyle: Restyle,
) -> Entity {
    let scene = assets.load(GltfAssetLabel::Scene(0).from_asset(spec.file));
    commands
        .spawn((
            WorldAssetRoot(scene),
            spec.child_transform(),
            restyle,
            ChildOf(parent),
        ))
        .id()
}

/// Khi một mô hình nạp xong: thay vật liệu theo bảng màu, không cho bấm (khối va chạm lo việc
/// đó), ghi vật liệu mới vào `Glow` của vật cha nếu có, và bỏ dấu `Pending`.
#[allow(clippy::too_many_arguments)] // Observer Bevy đọc nhiều loại thực thể; tách ra không gọn hơn.
pub fn restyle(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    styles: Query<(&Restyle, Option<&ChildOf>)>,
    children: Query<&Children>,
    named: Query<&GltfMaterialName>,
    meshes: Query<(), With<Mesh3d>>,
    mut glows: Query<&mut Glow>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let root = ready.entity;
    let Ok((style, parent)) = styles.get(root) else {
        return;
    };
    let mut made = Vec::new();
    for e in children.iter_descendants(root) {
        if meshes.get(e).is_err() {
            continue;
        }
        let mut ec = commands.entity(e);
        ec.insert(Pickable::IGNORE);
        if style.no_shadow {
            ec.insert(NotShadowCaster);
        }
        let name = named.get(e).map_or("", |n| n.0.as_str());
        let mut m = palette_material(role_for(name));
        if style
            .glow
            .is_some_and(|g| name.to_ascii_lowercase().contains(g))
        {
            m.emissive = LinearRgba::rgb(2.4, 1.6, 0.8);
        }
        let h = mats.add(m);
        made.push(h.clone());
        ec.insert(MeshMaterial3d(h));
    }
    if let Some(p) = parent
        && let Ok(mut g) = glows.get_mut(p.parent())
    {
        g.0.extend(made);
    }
    commands.entity(root).remove::<Pending>();
}
