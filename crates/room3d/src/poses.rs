//! Tư thế camera cố định (nhà, gần vật), hộp va chạm và phép chiếu để kiểm vùng bấm.

use crate::flight::Pose;
use bevy::math::Vec3;
use room_model::{ObjectKind, Slot, Viewport};

/// Góc nhìn dọc (radian), cố định; bề ngang suy ra từ tỉ lệ khung.
pub const FOV_Y: f32 = 0.8;

/// Đổi cm sang mét.
pub fn cm(p: [i32; 3]) -> Vec3 {
    Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32) / 100.0
}

fn aspect(vp: Viewport) -> f32 {
    vp.width.max(1) as f32 / vp.height.max(1) as f32
}

/// Chỗ camera nhìn khi ở nhà: giữa mặt bàn (mét).
const DESK_LOOK: Vec3 = Vec3::new(2.05, 0.85, 0.70);
/// Nửa bề ngang cảnh cần thấy trên bàn (mét).
const HOME_HALF_W: f32 = 0.85;
/// Nửa chiều cao cảnh cần thấy (mét).
const HOME_HALF_H: f32 = 0.60;
/// Độ chúc xuống của camera ở nhà (radian).
const HOME_PITCH: f32 = 0.35;

/// Tư thế ở nhà: nhìn cả mặt bàn. Màn dọc đứng xa hơn màn ngang để vẫn thấy hết bề ngang.
pub fn home_pose(vp: Viewport) -> Pose {
    let ty = (FOV_Y / 2.0).tan();
    let tx = ty * aspect(vp);
    let d = (HOME_HALF_W / tx).max(HOME_HALF_H / ty);
    Pose {
        eye: DESK_LOOK + d * Vec3::new(0.0, HOME_PITCH.sin(), HOME_PITCH.cos()),
        look: DESK_LOOK,
    }
}

/// Nửa kích thước hộp va chạm của vật (mét), theo trục thế giới. Cố ý rộng hơn mô hình để
/// ngón tay trẻ bấm trúng (ADR 0013).
pub fn collider_half(slot: &Slot) -> Vec3 {
    let cmv = match slot.kind {
        ObjectKind::Notebook => [16, 8, 14],
        ObjectKind::Laptop => [20, 14, 16],
        ObjectKind::Trophy => [12, 16, 12],
        ObjectKind::DeskLamp => [10, 20, 10],
        ObjectKind::Plant => [12, 20, 12],
        ObjectKind::PencilCup => [6, 8, 6],
        ObjectKind::Mug => [6, 6, 6],
        ObjectKind::Books => [14, 8, 10],
        ObjectKind::NoteBoard => [30, 20, 3],
        ObjectKind::MedalShelf => [30, 20, 3],
    };
    cm(cmv)
}

/// Tâm hộp va chạm của vật (mét): chân vật cộng nửa chiều cao hộp.
pub fn collider_center(slot: &Slot) -> Vec3 {
    cm(slot.pos_cm) + Vec3::new(0.0, collider_half(slot).y, 0.0)
}

/// Tư thế gần vật: mắt ở trên và trước vật (theo `yaw_deg`), nhìn vào tâm; đứng đủ xa để vật
/// chiếm khoảng 60 % chiều cao khung (và không tràn bề ngang).
pub fn close_pose(slot: &Slot, vp: Viewport) -> Pose {
    const FILL: f32 = 0.6;
    const PITCH: f32 = 0.5;
    let half = collider_half(slot);
    let center = collider_center(slot);
    let ty = (FOV_Y / 2.0).tan();
    let tx = ty * aspect(vp);
    // Bán kính cầu bao hộp: vật không tràn khung dù nhìn chếch từ trên xuống.
    let r = half.length();
    let d = r / (FILL * ty.min(tx));
    let yaw = (slot.yaw_deg as f32).to_radians();
    let dir = Vec3::new(
        yaw.sin() * PITCH.cos(),
        PITCH.sin(),
        yaw.cos() * PITCH.cos(),
    );
    Pose {
        eye: center + d * dir,
        look: center,
    }
}

/// Kích thước (rộng, cao) tính bằng px của khung bao hình chiếu hộp `center ± half` qua
/// camera `pose` với khung `vp`.
pub fn projected_size_px(center: Vec3, half: Vec3, pose: Pose, vp: Viewport) -> (f32, f32) {
    // Hệ trục camera: phải, lên, trước (tự dựng để khỏi phụ thuộc API glam đang đổi).
    let fwd = (pose.look - pose.eye).normalize();
    let right = fwd.cross(Vec3::Y).normalize();
    let up = right.cross(fwd);
    let ty = (FOV_Y / 2.0).tan();
    let tx = ty * aspect(vp);
    let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
    for i in 0..8 {
        let sign = Vec3::new(
            if i & 1 == 0 { -1.0 } else { 1.0 },
            if i & 2 == 0 { -1.0 } else { 1.0 },
            if i & 4 == 0 { -1.0 } else { 1.0 },
        );
        let d = center + half * sign - pose.eye;
        let depth = d.dot(fwd).max(0.01);
        let x = d.dot(right) / (depth * tx) * 0.5 * vp.width as f32;
        let y = d.dot(up) / (depth * ty) * 0.5 * vp.height as f32;
        min = [min[0].min(x), min[1].min(y)];
        max = [max[0].max(x), max[1].max(y)];
    }
    (max[0] - min[0], max[1] - min[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use room_model::{ROOM, SlotId};

    const PORTRAIT: Viewport = Viewport {
        width: 360,
        height: 740,
    };
    const LANDSCAPE: Viewport = Viewport {
        width: 1280,
        height: 720,
    };

    #[test]
    fn vung_bam_lon_hon_44px() {
        for s in ROOM.iter().filter(|s| s.zone.is_some()) {
            for vp in [PORTRAIT, LANDSCAPE] {
                let (w, h) =
                    projected_size_px(collider_center(s), collider_half(s), home_pose(vp), vp);
                assert!(w >= 44.0 && h >= 44.0, "{:?} {vp:?}: {w} x {h}", s.id);
            }
        }
    }

    #[test]
    fn goc_doc_khac_goc_ngang() {
        assert_ne!(home_pose(PORTRAIT), home_pose(LANDSCAPE));
        let d = |p: Pose| (p.eye - p.look).length();
        assert!(d(home_pose(PORTRAIT)) > d(home_pose(LANDSCAPE)));
    }

    #[test]
    fn gan_vat_chiem_khoang_60_phan_tram() {
        let nb = ROOM.iter().find(|s| s.id == SlotId::Notebook).unwrap();
        let (_, h) = projected_size_px(
            collider_center(nb),
            collider_half(nb),
            close_pose(nb, LANDSCAPE),
            LANDSCAPE,
        );
        assert!(h / 720.0 > 0.3 && h / 720.0 < 0.8, "{h}");
    }
}
