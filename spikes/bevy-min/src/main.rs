//! Spike ADR 0008 (1): cảnh Bevy tối thiểu — mặt bàn, một vật, ánh sáng, picking.
//! Bấm vào khối để đổi màu (chứng minh picking chạy).

use bevy::prelude::*;
use bevy::render::view::Tonemapping;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct Block;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(6.0, 0.2, 4.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.55, 0.4, 0.25))),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));
    commands
        .spawn((
            Block,
            Mesh3d(meshes.add(Cuboid::new(1.0, 0.5, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
            Transform::from_xyz(0.0, 0.25, 0.0),
        ))
        .observe(on_click);
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 6.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        // Mặc định TonyMcMapFace cần feature `tonemapping_luts` (thêm LUT + zstd); ACES không cần.
        Tonemapping::AcesFitted,
        Transform::from_xyz(0.0, 4.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn on_click(
    click: On<PointerClick>,
    blocks: Query<&MeshMaterial3d<StandardMaterial>, With<Block>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(handle) = blocks.get(click.entity)
        && let Some(mut m) = materials.get_mut(&handle.0)
    {
        let hue = (m.base_color.hue() + 60.0) % 360.0;
        m.base_color = m.base_color.with_hue(hue);
    }
}
