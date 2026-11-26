//! Con mèo trên bậu cửa sổ (Quaternius "Cat", CC0). Thở/nghỉ lặp (`Idle`), cứ 8–15 s ngẩng
//! đầu một lần (`Yes`; gói không có xương đuôi nên không vẫy đuôi). Nhịp lấy từ
//! [`PetClock`](crate::pet_clock::PetClock), seed cố định. Không bấm được. Giảm chuyển động:
//! đứng yên ở tư thế đầu của `Idle`. Nạp sau khung đầu được (quyết định 10).

use std::time::Duration;

use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use super::RoomState;
use super::models::Restyle;
use crate::pet_clock::PetClock;

/// File mô hình.
const FILE: &str = "cat.glb";
/// Chỉ số clip trong `cat.glb` (thứ tự trong file): 4 `CharacterArmature|Idle`,
/// 8 `CharacterArmature|Yes`.
const CLIP_IDLE: usize = 4;
const CLIP_YES: usize = 8;
/// Mô hình cao chừng 1,9 m; phóng về chừng 17 cm.
const SCALE: f32 = 0.09;
/// Chỗ ngồi: trên bậu cửa sổ (mặt bậu cao 1,00 m), lệch phải.
const AT: Vec3 = Vec3::new(2.40, 1.00, 0.09);
/// Quay về phía bàn một chút (độ).
const YAW_DEG: f32 = -20.0;
/// Thời gian chuyển giữa hai clip.
const BLEND: Duration = Duration::from_millis(350);

/// Gốc con mèo: đồ thị hoạt ảnh và hai clip dùng.
#[derive(Component)]
pub struct Cat {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    yes: AnimationNodeIndex,
}

/// Bộ phát hoạt ảnh của mèo (thực thể con mang `AnimationPlayer`), và nhịp cử chỉ.
#[derive(Component)]
pub struct CatPlayer {
    idle: AnimationNodeIndex,
    yes: AnimationNodeIndex,
    clock: Option<PetClock>,
    /// Đang đứng yên vì giảm chuyển động.
    still: bool,
}

/// Bắt đầu nạp mèo.
pub fn spawn(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clip = |i| assets.load(GltfAssetLabel::Animation(i).from_asset(FILE));
    let (graph, idx) = AnimationGraph::from_clips([clip(CLIP_IDLE), clip(CLIP_YES)]);
    let scene = assets.load(GltfAssetLabel::Scene(0).from_asset(FILE));
    commands
        .spawn((
            Cat {
                graph: graphs.add(graph),
                idle: idx[0],
                yes: idx[1],
            },
            WorldAssetRoot(scene),
            Restyle::default(),
            Transform::from_translation(AT)
                .with_rotation(Quat::from_rotation_y(YAW_DEG.to_radians()))
                .with_scale(Vec3::splat(SCALE)),
        ))
        .observe(ready);
}

/// Mèo nạp xong: gắn đồ thị hoạt ảnh vào bộ phát và bắt đầu `Idle` lặp.
fn ready(
    ev: On<WorldInstanceReady>,
    mut commands: Commands,
    cats: Query<&Cat>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let Ok(cat) = cats.get(ev.entity) else {
        return;
    };
    for e in children.iter_descendants(ev.entity) {
        let Ok(mut player) = players.get_mut(e) else {
            continue;
        };
        let mut tr = AnimationTransitions::new();
        tr.play(&mut player, cat.idle, Duration::ZERO).repeat();
        commands.entity(e).insert((
            AnimationGraphHandle(cat.graph.clone()),
            tr,
            CatPlayer {
                idle: cat.idle,
                yes: cat.yes,
                clock: None,
                still: false,
            },
        ));
    }
}

/// Mỗi khung: tới hẹn thì ngẩng đầu; ngẩng xong thì về `Idle`. Giảm chuyển động thì dừng ở
/// đầu `Idle`.
pub fn tick(
    room: Res<RoomState>,
    time: Res<Time>,
    mut q: Query<(
        &mut CatPlayer,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    let now = time.elapsed_secs();
    let reduced = room.ctl.reduced();
    for (mut cat, mut player, mut tr) in &mut q {
        if reduced {
            if !cat.still {
                cat.still = true;
                tr.play(&mut player, cat.idle, Duration::ZERO)
                    .seek_to(0.0)
                    .pause();
            }
            continue;
        }
        if cat.still {
            cat.still = false;
            tr.play(&mut player, cat.idle, Duration::ZERO).repeat();
        }
        let (idle, yes) = (cat.idle, cat.yes);
        let clock = cat.clock.get_or_insert_with(|| PetClock::new(now));
        if clock.due(now) {
            tr.play(&mut player, yes, BLEND);
        } else if tr.get_main_animation() == Some(yes)
            && player.animation(yes).is_none_or(|a| a.is_finished())
        {
            tr.play(&mut player, idle, BLEND).repeat();
        }
    }
}
