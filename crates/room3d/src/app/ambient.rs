//! Chuyển động nhỏ cho không khí: hơi nước trên cốc, rèm khẽ lay. Chỉ để nhìn, không đổi trạng
//! thái học tập; tắt hết khi giảm chuyển động (`Ctl::reduced`). Sao và đèn thành phố sau cửa
//! sổ theo `SkyState::stars`, và đổi trời mượt 2 s, nằm ở `look::sky`.

use std::f32::consts::{PI, TAU};

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::RoomState;
use crate::palette::color_of;

/// Số tấm hơi nước.
const PUFFS: usize = 3;
/// Một tấm hơi nước bay hết quãng trong chừng ấy giây.
const PUFF_LIFE: f32 = 3.0;
/// Hơi nước bay cao chừng ấy mét.
const PUFF_RISE: f32 = 0.11;

/// Chỗ hơi nước bốc lên (miệng cốc); `props::mug` đặt nó.
#[derive(Component)]
pub struct SteamAnchor;

/// Một tấm hơi nước; `i` lệch pha.
#[derive(Component)]
pub struct Puff {
    i: usize,
}

/// Một tấm rèm, xoay quanh mép trên; `phase` lệch pha giữa hai tấm.
#[derive(Component)]
pub struct Curtain {
    /// Lệch pha (radian).
    pub phase: f32,
}

/// Gắn các tấm hơi nước vào miệng cốc khi chỗ đó vừa xuất hiện.
pub fn add_steam(
    mut commands: Commands,
    anchors: Query<Entity, Added<SteamAnchor>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    for a in &anchors {
        let mesh = meshes.add(Circle::new(0.014));
        let [r, g, b] = color_of("paper");
        for i in 0..PUFFS {
            let m = mats.add(StandardMaterial {
                base_color: Color::srgba(r, g, b, 0.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
            commands.spawn((
                Puff { i },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(m),
                Transform::default(),
                NotShadowCaster,
                Pickable::IGNORE,
                ChildOf(a),
            ));
        }
    }
}

/// Hơi nước bay lên, lắc nhẹ, mờ dần; rèm lay. Giảm chuyển động: ẩn hơi nước, rèm đứng yên.
pub fn animate(
    room: Res<RoomState>,
    time: Res<Time>,
    mut puffs: Query<(
        &Puff,
        &mut Transform,
        &mut Visibility,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut curtains: Query<(&Curtain, &mut Transform), Without<Puff>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let still = room.ctl.reduced();
    let t = time.elapsed_secs();
    for (p, mut tr, mut vis, m) in &mut puffs {
        if still {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        let k = (t / PUFF_LIFE + p.i as f32 / PUFFS as f32).fract();
        let sway = (t * 1.3 + p.i as f32 * 2.0).sin() * 0.008 * k;
        tr.translation = Vec3::new(sway + (p.i as f32 - 1.0) * 0.008, k * PUFF_RISE, 0.0);
        tr.scale = Vec3::splat(0.6 + 0.8 * k);
        tr.rotation = Quat::from_rotation_z(sway * 8.0);
        if let Some(mut mat) = mats.get_mut(&m.0) {
            mat.base_color.set_alpha(0.22 * (PI * k).sin());
        }
    }
    for (c, mut tr) in &mut curtains {
        let a = if still {
            0.0
        } else {
            0.035 * (t * TAU / 5.0 + c.phase).sin()
        };
        tr.rotation = Quat::from_rotation_x(a);
    }
}
