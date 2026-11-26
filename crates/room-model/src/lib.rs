//! Mô hình phòng học 3D (ADR 0016): bảng chỗ đặt đồ, khu và thông điệp cầu nối.
//!
//! Crate thuần: không biết Bevy, Leptos hay nội dung học. `web` và `room3d` cùng đọc bảng
//! [`ROOM`], nên "vật nào dẫn vào khu nào" chỉ có một nguồn.

use serde::{Deserialize, Serialize};

/// Một khu của vỏ ứng dụng; phòng chỉ là lối điều hướng tới đây (ADR 0002).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Zone {
    /// Ôn tập (quyển vở).
    Review,
    /// Khóa học (máy tính).
    Courses,
    /// Thi đấu (cúp).
    Contests,
}

impl Zone {
    /// Mọi khu, theo thứ tự cố định.
    pub const ALL: [Zone; 3] = [Zone::Review, Zone::Courses, Zone::Contests];
}

/// Mã một chỗ đặt đồ trong [`ROOM`].
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotId {
    /// Quyển vở trên bàn.
    Notebook,
    /// Máy tính xách tay.
    Laptop,
    /// Cúp.
    Trophy,
    /// Đèn bàn.
    DeskLamp,
    /// Chậu cây.
    Plant,
    /// Ống bút.
    PencilCup,
    /// Cốc.
    Mug,
    /// Chồng sách.
    Books,
    /// Bảng ghi chú trên tường.
    NoteBoard,
    /// Kệ huy chương trên tường.
    MedalShelf,
}

/// Loại đồ vật đặt ở một chỗ. Mốc 1: mỗi chỗ một loại; mốc 3 cho đổi loại ở một chỗ.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectKind {
    /// Quyển vở.
    Notebook,
    /// Máy tính xách tay.
    Laptop,
    /// Cúp.
    Trophy,
    /// Đèn bàn.
    DeskLamp,
    /// Chậu cây.
    Plant,
    /// Ống bút.
    PencilCup,
    /// Cốc.
    Mug,
    /// Chồng sách.
    Books,
    /// Bảng ghi chú.
    NoteBoard,
    /// Kệ huy chương.
    MedalShelf,
}

/// Một chỗ đặt đồ. Là bảng hằng nên không có serde.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// Mã chỗ.
    pub id: SlotId,
    /// Loại đồ vật đặt ở đây.
    pub kind: ObjectKind,
    /// Vị trí tính bằng cm (số nguyên để tất định).
    pub pos_cm: [i32; 3],
    /// Góc quay quanh trục đứng, độ.
    pub yaw_deg: i16,
    /// Khu mà vật này dẫn vào; đồ trang trí thì `None`.
    pub zone: Option<Zone>,
}

const fn slot(
    id: SlotId,
    kind: ObjectKind,
    pos_cm: [i32; 3],
    yaw_deg: i16,
    zone: Option<Zone>,
) -> Slot {
    Slot {
        id,
        kind,
        pos_cm,
        yaw_deg,
        zone,
    }
}

/// Bảng chỗ đặt đồ mặc định: nguồn duy nhất cho "vật nào dẫn vào khu nào".
///
/// Mặt bàn ở y = 75, tâm bàn x = 200, z = 70 (rộng 140, sâu 60); tường sau z = 0, tường trái x = 0.
pub const ROOM: &[Slot] = &[
    slot(
        SlotId::Notebook,
        ObjectKind::Notebook,
        [165, 75, 80],
        10,
        Some(Zone::Review),
    ),
    slot(
        SlotId::Laptop,
        ObjectKind::Laptop,
        [205, 75, 58],
        0,
        Some(Zone::Courses),
    ),
    slot(
        SlotId::Trophy,
        ObjectKind::Trophy,
        [250, 75, 72],
        -15,
        Some(Zone::Contests),
    ),
    slot(
        SlotId::DeskLamp,
        ObjectKind::DeskLamp,
        [140, 75, 55],
        30,
        None,
    ),
    slot(SlotId::Plant, ObjectKind::Plant, [262, 75, 50], 0, None),
    slot(
        SlotId::PencilCup,
        ObjectKind::PencilCup,
        [228, 75, 85],
        0,
        None,
    ),
    slot(SlotId::Mug, ObjectKind::Mug, [185, 75, 92], 0, None),
    slot(SlotId::Books, ObjectKind::Books, [145, 75, 82], -5, None),
    slot(
        SlotId::NoteBoard,
        ObjectKind::NoteBoard,
        [150, 140, 1],
        0,
        None,
    ),
    slot(
        SlotId::MedalShelf,
        ObjectKind::MedalShelf,
        [290, 140, 1],
        0,
        None,
    ),
];

/// Chỗ dẫn vào `zone`.
pub fn entry_slot(zone: Zone) -> &'static Slot {
    match ROOM.iter().find(|s| s.zone == Some(zone)) {
        Some(s) => s,
        None => unreachable!("mỗi khu có đúng một chỗ trong ROOM (test moi_khu_dung_mot_cho)"),
    }
}

/// Mức chất lượng đồ họa.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quality {
    /// Đầy đủ hiệu ứng.
    High,
    /// Giảm hiệu ứng cho máy yếu.
    Low,
}

/// Kích thước khung nhìn, px CSS.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Viewport {
    /// Rộng.
    pub width: u32,
    /// Cao.
    pub height: u32,
}

/// Thông điệp từ vỏ `web` tới phòng 3D.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "t", content = "v")]
pub enum ToRoom {
    /// Khởi tạo phòng.
    Init {
        /// Giờ trong ngày (0..24).
        hour: u8,
        /// Chất lượng đồ họa.
        quality: Quality,
        /// Người dùng muốn giảm chuyển động.
        reduced_motion: bool,
        /// Khung nhìn ban đầu.
        viewport: Viewport,
    },
    /// Đổi giờ trong ngày.
    SetHour(u8),
    /// Đổi kích thước khung nhìn.
    Resize(Viewport),
    /// Tạm dừng vẽ.
    Suspend,
    /// Vẽ tiếp.
    Resume,
    /// Quay về phòng từ một khu.
    ReturnFrom(Zone),
    /// Làm sáng một chỗ (hoặc bỏ sáng).
    Highlight(Option<SlotId>),
    /// Đi tới một khu (từ danh sách chữ).
    Go(Zone),
    /// Đổi chất lượng đồ họa.
    SetQuality(Quality),
}

/// Thông điệp từ phòng 3D tới vỏ `web`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "t", content = "v")]
pub enum FromRoom {
    /// Đã vẽ khung đầu tiên.
    FirstFrame,
    /// Con trỏ đang ở trên một chỗ (hoặc rời đi).
    Hover(Option<SlotId>),
    /// Người học chọn mở một khu.
    Open(Zone),
    /// Camera đã tới khu.
    Arrived(Zone),
    /// Phòng gặp lỗi.
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ma_cho_khong_trung() {
        let mut ids: Vec<SlotId> = ROOM.iter().map(|s| s.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), ROOM.len());
        assert_eq!(ROOM.len(), 10);
    }

    #[test]
    fn moi_khu_dung_mot_cho() {
        for z in Zone::ALL {
            let n = ROOM.iter().filter(|s| s.zone == Some(z)).count();
            assert_eq!(n, 1, "{z:?}");
            assert_eq!(entry_slot(z).zone, Some(z));
        }
    }

    #[test]
    fn json_khu_hoi() {
        let vp = Viewport {
            width: 360,
            height: 740,
        };
        let to = [
            ToRoom::Init {
                hour: 23,
                quality: Quality::Low,
                reduced_motion: true,
                viewport: vp,
            },
            ToRoom::SetHour(7),
            ToRoom::Resize(vp),
            ToRoom::Suspend,
            ToRoom::Resume,
            ToRoom::ReturnFrom(Zone::Contests),
            ToRoom::Highlight(Some(SlotId::Laptop)),
            ToRoom::Highlight(None),
            ToRoom::Go(Zone::Courses),
            ToRoom::SetQuality(Quality::High),
        ];
        for m in to {
            let s = serde_json::to_string(&m).unwrap();
            assert_eq!(serde_json::from_str::<ToRoom>(&s).unwrap(), m, "{s}");
        }
        let from = [
            FromRoom::FirstFrame,
            FromRoom::Hover(Some(SlotId::Trophy)),
            FromRoom::Hover(None),
            FromRoom::Open(Zone::Review),
            FromRoom::Arrived(Zone::Courses),
            FromRoom::Failed("x".into()),
        ];
        for m in from {
            let s = serde_json::to_string(&m).unwrap();
            assert_eq!(serde_json::from_str::<FromRoom>(&s).unwrap(), m, "{s}");
        }
    }

    #[test]
    fn json_dang_on_dinh() {
        assert_eq!(
            serde_json::to_string(&ToRoom::Go(Zone::Review)).unwrap(),
            r#"{"t":"Go","v":"Review"}"#
        );
    }
}
