//! Bộ điều khiển phòng: nhận thông điệp, chạm, đồng hồ; trả tư thế camera và thông điệp
//! gửi về vỏ. Thuần logic, không biết Bevy ngoài kiểu toán.

use crate::flight::{Flight, Pose};
use crate::poses::{close_pose, home_pose};
use room_model::{FromRoom, Quality, ROOM, SlotId, ToRoom, Viewport, Zone, entry_slot};

/// Thời lượng bay (giây).
const FLIGHT_SECS: f32 = 0.7;

/// Đích camera đang hướng tới hoặc đang đứng ở.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Nhìn cả phòng.
    Home,
    /// Gần vật lối vào của một khu.
    Zone(Zone),
}

/// Trạng thái điều khiển phòng.
#[derive(Debug, Clone)]
pub struct Ctl {
    vp: Viewport,
    reduced: bool,
    quality: Quality,
    hour: u8,
    suspended: bool,
    pose: Pose,
    target: Target,
    flight: Option<Flight>,
    highlighted: Option<SlotId>,
    last_hover: Option<SlotId>,
    pinned: Option<SlotId>,
}

impl Ctl {
    /// Điều khiển mới, đứng ở nhà; `reduced` là giảm chuyển động.
    pub fn new(vp: Viewport, reduced: bool) -> Self {
        Self {
            vp,
            reduced,
            quality: Quality::High,
            hour: 12,
            suspended: false,
            pose: home_pose(vp),
            target: Target::Home,
            flight: None,
            highlighted: None,
            last_hover: None,
            pinned: None,
        }
    }

    fn pose_of(&self, target: Target) -> Pose {
        match target {
            Target::Home => home_pose(self.vp),
            Target::Zone(z) => close_pose(entry_slot(z), self.vp),
        }
    }

    /// Bay (hoặc đặt ngay nếu giảm chuyển động) tới `to`; đang bay thì đổi đích liền mạch.
    fn go(&mut self, to: Pose, now: f32) {
        if self.reduced {
            self.flight = None;
            self.pose = to;
            return;
        }
        self.flight = Some(match self.flight {
            Some(f) => f.retarget(now, to),
            None => Flight::new(self.pose, to, now, FLIGHT_SECS),
        });
    }

    fn fly_to_zone(&mut self, z: Zone, now: f32) -> Vec<FromRoom> {
        if self.flight.is_some() && self.target == Target::Zone(z) {
            return vec![];
        }
        self.target = Target::Zone(z);
        self.go(self.pose_of(self.target), now);
        if self.reduced {
            vec![FromRoom::Open(z), FromRoom::Arrived(z)]
        } else {
            vec![FromRoom::Open(z)]
        }
    }

    /// Xử lý một thông điệp từ vỏ.
    pub fn on_msg(&mut self, msg: &ToRoom, now: f32) -> Vec<FromRoom> {
        match msg {
            ToRoom::Init {
                hour,
                quality,
                reduced_motion,
                viewport,
            } => {
                self.hour = *hour;
                self.quality = *quality;
                self.reduced = *reduced_motion;
                self.vp = *viewport;
                self.flight = None;
                self.target = Target::Home;
                self.pose = home_pose(self.vp);
            }
            ToRoom::SetHour(h) => self.hour = *h,
            ToRoom::SetQuality(q) => self.quality = *q,
            ToRoom::Resize(vp) => {
                self.vp = *vp;
                let to = self.pose_of(self.target);
                match self.flight {
                    Some(f) => self.flight = Some(f.retarget(now, to)),
                    None => self.pose = to,
                }
            }
            ToRoom::Suspend => self.suspended = true,
            ToRoom::Resume => self.suspended = false,
            ToRoom::Highlight(s) => self.highlighted = *s,
            ToRoom::Go(z) => return self.fly_to_zone(*z, now),
            ToRoom::ReturnFrom(z) => {
                if self.flight.is_none() {
                    self.pose = close_pose(entry_slot(*z), self.vp);
                }
                self.target = Target::Home;
                self.go(home_pose(self.vp), now);
            }
        }
        vec![]
    }

    /// Con trỏ chuột ở trên `slot` (hoặc rời đi). Phát `Hover` chỉ khi đổi.
    pub fn on_hover(&mut self, slot: Option<SlotId>) -> Vec<FromRoom> {
        self.highlighted = slot;
        if slot == self.last_hover {
            return vec![];
        }
        self.last_hover = slot;
        vec![FromRoom::Hover(slot)]
    }

    /// Chạm/bấm vào `slot` (hoặc ra ngoài). `touch`: cảm ứng, cần chạm hai lần để vào khu.
    pub fn on_tap(&mut self, slot: Option<SlotId>, touch: bool, now: f32) -> Vec<FromRoom> {
        let Some(id) = slot else {
            self.pinned = None;
            self.highlighted = None;
            self.last_hover = None;
            return vec![FromRoom::Hover(None)];
        };
        let zone = ROOM.iter().find(|s| s.id == id).and_then(|s| s.zone);
        let Some(z) = zone else {
            return vec![];
        };
        if touch && self.pinned != Some(id) {
            self.pinned = Some(id);
            self.highlighted = Some(id);
            self.last_hover = Some(id);
            return vec![FromRoom::Hover(Some(id))];
        }
        self.pinned = None;
        self.fly_to_zone(z, now)
    }

    /// Tiến một khung: trả tư thế camera và thông điệp (đúng một `Arrived` khi bay xong).
    pub fn tick(&mut self, now: f32) -> (Pose, Vec<FromRoom>) {
        let Some(f) = self.flight else {
            return (self.pose, vec![]);
        };
        let pose = f.pose(now);
        if !f.done(now) || self.suspended {
            return (pose, vec![]);
        }
        self.flight = None;
        self.pose = pose;
        match self.target {
            Target::Zone(z) => (pose, vec![FromRoom::Arrived(z)]),
            Target::Home => (pose, vec![]),
        }
    }

    /// Chỗ đang sáng.
    pub fn highlighted(&self) -> Option<SlotId> {
        self.highlighted
    }

    /// Đang tạm dừng vẽ.
    pub fn suspended(&self) -> bool {
        self.suspended
    }

    /// Chất lượng hiện tại.
    pub fn quality(&self) -> Quality {
        self.quality
    }

    /// Giờ trong ngày hiện tại.
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Có đang giảm chuyển động.
    pub fn reduced(&self) -> bool {
        self.reduced
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::pose_near;
    use room_model::Zone::{Contests, Courses, Review};

    const WIDE: Viewport = Viewport {
        width: 1280,
        height: 720,
    };
    const NARROW: Viewport = Viewport {
        width: 360,
        height: 740,
    };

    fn slot(id: SlotId) -> &'static room_model::Slot {
        ROOM.iter().find(|s| s.id == id).unwrap()
    }

    #[test]
    fn bam_chuot_bay_roi_toi() {
        let mut c = Ctl::new(WIDE, false);
        assert_eq!(
            c.on_tap(Some(SlotId::Notebook), false, 0.0),
            [FromRoom::Open(Review)]
        );
        assert!(c.tick(0.69).1.is_empty());
        assert_eq!(c.tick(0.71).1, [FromRoom::Arrived(Review)]);
        assert!(c.tick(1.0).1.is_empty());
    }

    #[test]
    fn cham_hai_lan_moi_vao() {
        let mut c = Ctl::new(NARROW, false);
        assert_eq!(
            c.on_tap(Some(SlotId::Notebook), true, 0.0),
            [FromRoom::Hover(Some(SlotId::Notebook))]
        );
        assert_eq!(
            c.on_tap(Some(SlotId::Notebook), true, 0.1),
            [FromRoom::Open(Review)]
        );
    }

    #[test]
    fn cham_vat_khac_doi_gai() {
        let mut c = Ctl::new(NARROW, false);
        c.on_tap(Some(SlotId::Notebook), true, 0.0);
        assert_eq!(
            c.on_tap(Some(SlotId::Laptop), true, 0.1),
            [FromRoom::Hover(Some(SlotId::Laptop))]
        );
        assert_eq!(c.on_tap(None, true, 0.2), [FromRoom::Hover(None)]);
        assert_eq!(
            c.on_tap(Some(SlotId::Laptop), true, 0.3),
            [FromRoom::Hover(Some(SlotId::Laptop))]
        );
    }

    #[test]
    fn doi_dich_khi_dang_bay() {
        let mut c = Ctl::new(WIDE, false);
        c.on_tap(Some(SlotId::Notebook), false, 0.0);
        let before = c.tick(0.3).0;
        assert_eq!(
            c.on_tap(Some(SlotId::Trophy), false, 0.3),
            [FromRoom::Open(Contests)]
        );
        assert!(pose_near(c.tick(0.3).0, before));
        assert!(c.tick(0.9).1.is_empty());
        assert_eq!(c.tick(1.0).1, [FromRoom::Arrived(Contests)]);
        assert!(c.tick(2.0).1.is_empty());
    }

    #[test]
    fn bam_lai_vat_dang_bay_khong_mo_lai() {
        let mut c = Ctl::new(WIDE, false);
        c.on_tap(Some(SlotId::Notebook), false, 0.0);
        assert!(c.on_tap(Some(SlotId::Notebook), false, 0.2).is_empty());
    }

    #[test]
    fn giam_chuyen_dong_toi_ngay() {
        let mut c = Ctl::new(WIDE, true);
        assert_eq!(
            c.on_tap(Some(SlotId::Notebook), false, 0.0),
            [FromRoom::Open(Review), FromRoom::Arrived(Review)]
        );
        assert!(pose_near(
            c.tick(0.0).0,
            close_pose(slot(SlotId::Notebook), WIDE)
        ));
    }

    #[test]
    fn quay_ve_tu_khu() {
        let mut c = Ctl::new(WIDE, false);
        assert!(c.on_msg(&ToRoom::ReturnFrom(Review), 0.0).is_empty());
        let (p, m) = c.tick(0.0);
        assert!(pose_near(p, close_pose(slot(SlotId::Notebook), WIDE)));
        assert!(m.is_empty());
        let (p, m) = c.tick(0.7);
        assert!(pose_near(p, home_pose(WIDE)));
        assert!(m.is_empty());
    }

    #[test]
    fn doi_ti_le_giua_luc_bay_van_toi_dich() {
        let mut c = Ctl::new(WIDE, false);
        c.on_tap(Some(SlotId::Laptop), false, 0.0);
        c.tick(0.3);
        c.on_msg(&ToRoom::Resize(NARROW), 0.3);
        let (p, m) = c.tick(1.0);
        assert_eq!(m, [FromRoom::Arrived(Courses)]);
        assert!(pose_near(p, close_pose(slot(SlotId::Laptop), NARROW)));
    }

    #[test]
    fn doi_ti_le_khi_dung_o_nha() {
        let mut c = Ctl::new(WIDE, false);
        c.on_msg(&ToRoom::Resize(NARROW), 0.0);
        assert!(pose_near(c.tick(0.0).0, home_pose(NARROW)));
    }

    #[test]
    fn tam_dung_khong_phat() {
        let mut c = Ctl::new(WIDE, false);
        c.on_tap(Some(SlotId::Notebook), false, 0.0);
        c.on_msg(&ToRoom::Suspend, 0.1);
        assert!(c.suspended());
        assert!(c.tick(1.0).1.is_empty());
        c.on_msg(&ToRoom::Resume, 1.1);
        assert_eq!(c.tick(1.2).1, [FromRoom::Arrived(Review)]);
    }

    #[test]
    fn do_trang_tri_khong_mo_khu() {
        let mut c = Ctl::new(WIDE, false);
        assert!(c.on_tap(Some(SlotId::Mug), false, 0.0).is_empty());
    }

    #[test]
    fn hover_chi_phat_khi_doi() {
        let mut c = Ctl::new(WIDE, false);
        assert_eq!(c.on_hover(Some(SlotId::Mug)).len(), 1);
        assert!(c.on_hover(Some(SlotId::Mug)).is_empty());
        assert_eq!(c.highlighted(), Some(SlotId::Mug));
        assert_eq!(c.on_hover(None), [FromRoom::Hover(None)]);
    }

    #[test]
    fn go_nhu_cham_chuot() {
        let mut c = Ctl::new(WIDE, false);
        assert_eq!(
            c.on_msg(&ToRoom::Go(Courses), 0.0),
            [FromRoom::Open(Courses)]
        );
    }
}
