//! Chuột và chạm: hover, bấm, con trỏ tay. Chỉ chuyển vào `Ctl`; `Ctl` quyết định.

use bevy::picking::pointer::{PointerButton, PointerId};
use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};
use room_model::{ROOM, SlotId};

use super::{RoomState, send_all};

/// Khối va chạm của một chỗ đặt đồ.
#[derive(Component)]
pub struct SlotTag(pub SlotId);

/// Vật lối vào chuột đang ở trên (chỉ chuột; chạm dùng "chạm lần một = hover" của `Ctl`).
#[derive(Resource, Default)]
pub struct MouseOver(Option<SlotId>);

/// Chỗ này là vật lối vào một khu.
fn is_entry(id: SlotId) -> bool {
    ROOM.iter().any(|s| s.id == id && s.zone.is_some())
}

/// `PointerOver`/`PointerOut` của chuột → `Ctl::on_hover`; `PointerClick` → `Ctl::on_tap`.
/// Bấm trúng chỗ không phải lối vào, hoặc trúng nền (tường, sàn, bàn), là `on_tap(None)`.
pub fn pointer(
    mut room: ResMut<RoomState>,
    time: Res<Time>,
    mut mouse: ResMut<MouseOver>,
    mut over: MessageReader<PointerOver>,
    mut out: MessageReader<PointerOut>,
    mut click: MessageReader<PointerClick>,
    tags: Query<&SlotTag>,
) {
    let mut cur = mouse.0;
    // Rời chỗ cũ trước, rồi mới vào chỗ mới (đi thẳng từ vật này sang vật kia).
    for e in out.read() {
        if e.pointer.id == PointerId::Mouse
            && let Ok(t) = tags.get(e.entity)
            && cur == Some(t.0)
        {
            cur = None;
        }
    }
    for e in over.read() {
        if e.pointer.id == PointerId::Mouse
            && let Ok(t) = tags.get(e.entity)
        {
            // Đồ trang trí (cốc, cây…) không sáng: như rời mọi vật.
            cur = Some(t.0).filter(|&id| is_entry(id));
        }
    }
    if cur != mouse.0 {
        mouse.0 = cur;
        let msgs = room.ctl.on_hover(cur);
        send_all(msgs);
    }
    let now = time.elapsed_secs();
    for e in click.read() {
        if e.button != PointerButton::Primary {
            continue;
        }
        let slot = tags
            .get(e.entity)
            .ok()
            .map(|t| t.0)
            .filter(|&id| is_entry(id));
        let touch = matches!(e.pointer.id, PointerId::Touch(_));
        let msgs = room.ctl.on_tap(slot, touch, now);
        send_all(msgs);
    }
}

/// Con trỏ tay khi chuột ở trên vật lối vào.
pub fn cursor(
    mouse: Res<MouseOver>,
    mut hand: Local<bool>,
    mut commands: Commands,
    windows: Query<Entity, With<Window>>,
) {
    let want = mouse.0.is_some_and(is_entry);
    if want == *hand {
        return;
    }
    *hand = want;
    let icon = if want {
        SystemCursorIcon::Pointer
    } else {
        SystemCursorIcon::Default
    };
    for w in &windows {
        commands.entity(w).insert(CursorIcon::from(icon));
    }
}
