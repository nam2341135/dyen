//! Máy trạng thái của lớp phủ chuyển cảnh giữa phòng và các khu (bản thiết kế phòng học
//! 3D, mục 6.7). Thuần: không DOM, không đồng hồ. Vỏ thực thi các [`Effect`] trả về; hẹn
//! giờ là [`Effect::After`], vỏ gọi lại [`Transition::on`] khi tới giờ.

use room_model::{ToRoom, Zone};

use crate::route::Route;
use crate::zones;

/// Thời lượng lượt bay của camera (ms).
const FLY_MS: u32 = 700;
/// Lớp phủ hiện lên (ms).
const COVER_MS: u32 = 250;
/// Lớp phủ mờ đi, lộ trang (ms).
const OUT_MS: u32 = 200;
/// Lớp phủ khi giảm chuyển động (ms), cả vào lẫn ra.
const REDUCED_MS: u32 = 150;
/// Chờ trang bao lâu thì bật hiệu ứng chờ của khu (ms).
const HOLD_MS: u32 = 600;
/// Chờ trang bao lâu thì hiện chữ "Đang mở…" (ms). 250 + 2400 < 3000.
const SLOW_MS: u32 = 2400;

/// Pha của chuyển cảnh. Khu đi kèm là khu đang tới (hoặc đang rời, với `Leaving`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Không có chuyển cảnh.
    Idle,
    /// Camera đang bay tới vật; chưa có lớp phủ.
    Flying(Zone),
    /// Lớp phủ đang hiện lên.
    Covering(Zone),
    /// Lớp phủ đục, chờ trang khu sẵn sàng.
    Waiting(Zone),
    /// Lớp phủ đang mờ đi, lộ trang khu.
    Revealing(Zone),
    /// Đang về phòng: lớp phủ mờ đi, lộ phòng.
    Leaving(Zone),
}

impl Phase {
    /// Khu của pha (không có khi `Idle`).
    fn zone(self) -> Option<Zone> {
        match self {
            Phase::Idle => None,
            Phase::Flying(z)
            | Phase::Covering(z)
            | Phase::Waiting(z)
            | Phase::Revealing(z)
            | Phase::Leaving(z) => Some(z),
        }
    }
}

/// Sự kiện vào máy trạng thái: từ phòng, từ vỏ, hoặc hẹn giờ tới.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ev {
    /// Bắt đầu đi tới khu (phòng bắt đầu bay, hoặc bấm danh sách chữ khi không có 3D).
    Open(Zone),
    /// Hẹn giờ: bắt đầu phủ (250 ms cuối của lượt bay).
    CoverStart,
    /// Phòng báo camera đã tới. Chỉ để ghi nhận: lớp phủ theo đồng hồ của vỏ.
    Arrived(Zone),
    /// Hẹn giờ: lớp phủ đã đục.
    Covered,
    /// Hẹn giờ: trang chưa xong, bật hiệu ứng chờ.
    HoldStart,
    /// Hẹn giờ: trang lâu quá, hiện chữ "Đang mở…".
    SlowTick,
    /// Trang khu đã vẽ.
    PageReady,
    /// Hẹn giờ: lớp phủ đã mờ hết.
    Revealed,
    /// Về phòng (Esc, nút Back, hash về phòng).
    Back,
}

/// Bước của lớp phủ, ứng với lớp CSS `in`, `hold`, `out`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayStep {
    /// Mờ dần lên tới đục.
    In,
    /// Đục, có hiệu ứng chờ riêng của khu.
    Hold,
    /// Bắt đầu đục rồi mờ dần đi.
    Out,
}

/// Lớp CSS của lớp phủ, ví dụ `overlay in zone-review`.
pub fn overlay_class(zone: Zone, step: OverlayStep) -> String {
    let step = match step {
        OverlayStep::In => "in",
        OverlayStep::Hold => "hold",
        OverlayStep::Out => "out",
    };
    let zone = match zone {
        Zone::Review => "review",
        Zone::Courses => "courses",
        Zone::Contests => "contests",
    };
    format!("overlay {step} zone-{zone}")
}

/// Việc vỏ phải làm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Đặt lớp phủ (`None` là bỏ).
    Overlay(Option<(Zone, OverlayStep)>),
    /// Đi tới màn (đặt hash).
    Navigate(Route),
    /// Gửi thông điệp tới phòng.
    Send(ToRoom),
    /// Sau số mili giây này, đưa sự kiện vào lại (bỏ nếu [`Transition::generation`] đã đổi).
    After(u32, Ev),
    /// Hiện chữ "Đang mở…" trên lớp phủ.
    ShowOpeningText,
}

/// Máy trạng thái chuyển cảnh.
#[derive(Clone, Debug)]
pub struct Transition {
    /// Pha hiện tại.
    pub phase: Phase,
    reduced: bool,
    page_ready: bool,
    last: Option<Zone>,
    generation: u32,
}

impl Transition {
    /// Máy mới ở `Idle`. `reduced`: người dùng muốn giảm chuyển động.
    pub fn new(reduced: bool) -> Self {
        Transition {
            phase: Phase::Idle,
            reduced,
            page_ready: false,
            last: None,
            generation: 0,
        }
    }

    /// Thế hệ hẹn giờ: đổi khi lượt bay bắt đầu, đổi đích, hoặc khi Back. Hẹn giờ đặt ở
    /// thế hệ cũ thì vỏ bỏ.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    fn d_cover(&self) -> u32 {
        if self.reduced { REDUCED_MS } else { COVER_MS }
    }

    fn d_out(&self) -> u32 {
        if self.reduced { REDUCED_MS } else { OUT_MS }
    }

    fn fly(&mut self, z: Zone) -> Vec<Effect> {
        self.phase = Phase::Flying(z);
        self.page_ready = false;
        self.generation = self.generation.wrapping_add(1);
        let wait = if self.reduced {
            0
        } else {
            FLY_MS - self.d_cover()
        };
        vec![Effect::After(wait, Ev::CoverStart)]
    }

    fn reveal(&mut self, z: Zone) -> Vec<Effect> {
        self.phase = Phase::Revealing(z);
        vec![
            Effect::Overlay(Some((z, OverlayStep::Out))),
            Effect::After(self.d_out(), Ev::Revealed),
        ]
    }

    /// Đưa một sự kiện vào; trả về các việc vỏ phải làm, theo thứ tự.
    pub fn on(&mut self, ev: Ev) -> Vec<Effect> {
        use Phase::*;
        match (self.phase, ev) {
            (Idle, Ev::Open(z)) => self.fly(z),
            (Flying(z), Ev::Open(z2)) if z2 != z => self.fly(z2),
            (Flying(z), Ev::CoverStart) => {
                self.phase = Covering(z);
                vec![
                    Effect::Overlay(Some((z, OverlayStep::In))),
                    Effect::After(self.d_cover(), Ev::Covered),
                ]
            }
            (Covering(z), Ev::Covered) => {
                self.last = Some(z);
                let mut fx = vec![Effect::Navigate(zones::route(z))];
                if self.page_ready {
                    // Quyết định 13: trang xong sớm vẫn chạy đủ lớp phủ vào và ra.
                    fx.extend(self.reveal(z));
                } else {
                    self.phase = Waiting(z);
                    fx.push(Effect::After(HOLD_MS, Ev::HoldStart));
                    fx.push(Effect::After(SLOW_MS, Ev::SlowTick));
                }
                fx
            }
            (Waiting(z), Ev::PageReady) => self.reveal(z),
            (Waiting(z), Ev::HoldStart) if !self.reduced => {
                vec![Effect::Overlay(Some((z, OverlayStep::Hold)))]
            }
            (Waiting(_), Ev::SlowTick) => vec![Effect::ShowOpeningText],
            (Revealing(_), Ev::Revealed) => {
                self.phase = Idle;
                vec![Effect::Overlay(None), Effect::Send(ToRoom::Suspend)]
            }
            (Leaving(_), Ev::Revealed) => {
                self.phase = Idle;
                self.last = None;
                vec![Effect::Overlay(None)]
            }
            // R2: Back khi đang về phòng bị bỏ (Navigate(Room) → hashchange → Back).
            (Leaving(_), Ev::Back) => Vec::new(),
            (phase, Ev::Back) => match phase.zone().or(self.last) {
                Some(z) => {
                    self.phase = Leaving(z);
                    self.generation = self.generation.wrapping_add(1);
                    let mut fx = Vec::with_capacity(5);
                    // Đang bay thì lớp phủ chưa hiện: không chớp màu khu lên rồi mờ đi.
                    if !matches!(phase, Flying(_)) {
                        fx.push(Effect::Overlay(Some((z, OverlayStep::Out))));
                    }
                    fx.extend([
                        Effect::Navigate(Route::Room),
                        Effect::Send(ToRoom::Resume),
                        Effect::Send(ToRoom::ReturnFrom(z)),
                        Effect::After(self.d_out(), Ev::Revealed),
                    ]);
                    fx
                }
                None => Vec::new(),
            },
            (_, Ev::PageReady) => {
                self.page_ready = true;
                Vec::new()
            }
            // `Arrived` chỉ ghi nhận; sự kiện tới sai pha thì bỏ.
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Đưa lần lượt các sự kiện, gộp mọi hiệu ứng.
    fn run(t: &mut Transition, evs: &[Ev]) -> Vec<Effect> {
        evs.iter().flat_map(|&e| t.on(e)).collect()
    }

    fn overlay(z: Zone, s: OverlayStep) -> Effect {
        Effect::Overlay(Some((z, s)))
    }

    #[test]
    fn luong_day_du() {
        let mut t = Transition::new(false);
        let fx = run(
            &mut t,
            &[
                Ev::Open(Zone::Review),
                Ev::CoverStart,
                Ev::Covered,
                Ev::PageReady,
                Ev::Revealed,
            ],
        );
        let navs = fx
            .iter()
            .filter(|e| **e == Effect::Navigate(Route::Review))
            .count();
        assert_eq!(navs, 1, "{fx:?}");
        assert_eq!(fx.last(), Some(&Effect::Send(ToRoom::Suspend)), "{fx:?}");
        assert_eq!(t.phase, Phase::Idle);
        assert_eq!(fx[0], Effect::After(450, Ev::CoverStart));
    }

    #[test]
    fn trang_san_sang_som_van_du_lop_phu() {
        let mut t = Transition::new(false);
        let fx = run(
            &mut t,
            &[
                Ev::Open(Zone::Contests),
                Ev::CoverStart,
                Ev::PageReady,
                Ev::Covered,
            ],
        );
        let i = fx
            .iter()
            .position(|e| *e == overlay(Zone::Contests, OverlayStep::In))
            .expect("có Overlay(In)");
        let o = fx
            .iter()
            .position(|e| *e == overlay(Zone::Contests, OverlayStep::Out))
            .expect("có Overlay(Out)");
        assert!(i < o, "{fx:?}");
        assert!(fx.contains(&Effect::After(200, Ev::Revealed)), "{fx:?}");
        assert_eq!(t.phase, Phase::Revealing(Zone::Contests));
    }

    #[test]
    fn giam_chuyen_dong_150ms() {
        let mut t = Transition::new(true);
        assert_eq!(
            t.on(Ev::Open(Zone::Review)),
            vec![Effect::After(0, Ev::CoverStart)]
        );
        assert!(
            t.on(Ev::CoverStart)
                .contains(&Effect::After(150, Ev::Covered))
        );
        // Khi giảm chuyển động, không có hiệu ứng chờ động.
        t.on(Ev::Covered);
        assert!(t.on(Ev::HoldStart).is_empty());
        assert!(
            t.on(Ev::PageReady)
                .contains(&Effect::After(150, Ev::Revealed))
        );
    }

    #[test]
    fn doi_dich_khi_dang_bay() {
        let mut t = Transition::new(false);
        let fx = run(
            &mut t,
            &[
                Ev::Open(Zone::Review),
                Ev::Open(Zone::Contests),
                Ev::CoverStart,
                Ev::Covered,
            ],
        );
        assert!(fx.contains(&Effect::Navigate(Route::Contests)), "{fx:?}");
        assert!(!fx.contains(&Effect::Navigate(Route::Review)), "{fx:?}");
    }

    #[test]
    fn back_o_moi_pha_deu_ve_phong() {
        let z = Zone::Courses;
        let setups: [&[Ev]; 5] = [
            &[Ev::Open(z)],
            &[Ev::Open(z), Ev::CoverStart],
            &[Ev::Open(z), Ev::CoverStart, Ev::Covered],
            &[Ev::Open(z), Ev::CoverStart, Ev::Covered, Ev::PageReady],
            &[
                Ev::Open(z),
                Ev::CoverStart,
                Ev::Covered,
                Ev::PageReady,
                Ev::Revealed,
            ],
        ];
        for setup in setups {
            let mut t = Transition::new(false);
            run(&mut t, setup);
            let fx = t.on(Ev::Back);
            for want in [
                Effect::Navigate(Route::Room),
                Effect::Send(ToRoom::Resume),
                Effect::Send(ToRoom::ReturnFrom(z)),
                Effect::After(200, Ev::Revealed),
            ] {
                assert!(fx.contains(&want), "{setup:?}: {fx:?}");
            }
            // Đang bay thì chưa có lớp phủ: không chớp màu khu lên rồi mờ đi.
            let out = Effect::Overlay(Some((z, OverlayStep::Out)));
            let flying = setup.len() == 1;
            assert_eq!(fx.contains(&out), !flying, "{setup:?}: {fx:?}");
            assert_eq!(t.phase, Phase::Leaving(z), "{setup:?}");
            assert_eq!(t.on(Ev::Revealed), vec![Effect::Overlay(None)]);
            assert_eq!(t.phase, Phase::Idle, "{setup:?}");
            // Đã về phòng: Back nữa không làm gì.
            assert!(t.on(Ev::Back).is_empty(), "{setup:?}");
        }
    }

    #[test]
    fn back_khi_dang_o_phong_khong_lam_gi() {
        assert!(Transition::new(false).on(Ev::Back).is_empty());
    }

    #[test]
    fn back_khi_dang_ve_bi_bo_qua() {
        let mut t = Transition::new(false);
        run(&mut t, &[Ev::Open(Zone::Review), Ev::Back]);
        assert!(t.on(Ev::Back).is_empty());
        assert_eq!(t.phase, Phase::Leaving(Zone::Review));
    }

    #[test]
    fn cho_lau_hien_chu_dang_mo() {
        let mut t = Transition::new(false);
        let fx = run(
            &mut t,
            &[Ev::Open(Zone::Review), Ev::CoverStart, Ev::Covered],
        );
        assert!(fx.contains(&Effect::After(2400, Ev::SlowTick)), "{fx:?}");
        assert!(fx.contains(&Effect::After(600, Ev::HoldStart)), "{fx:?}");
        assert_eq!(
            t.on(Ev::HoldStart),
            vec![overlay(Zone::Review, OverlayStep::Hold)]
        );
        assert_eq!(t.on(Ev::SlowTick), vec![Effect::ShowOpeningText]);
        assert_eq!(t.phase, Phase::Waiting(Zone::Review));
    }

    #[test]
    fn su_kien_cu_bi_bo_qua() {
        let mut t = Transition::new(false);
        assert!(t.on(Ev::Revealed).is_empty());
        assert!(t.on(Ev::Covered).is_empty());
        assert!(t.on(Ev::Arrived(Zone::Review)).is_empty());
        assert_eq!(t.phase, Phase::Idle);
    }

    #[test]
    fn hen_gio_cu_bi_bo_qua() {
        let mut t = Transition::new(false);
        t.on(Ev::Open(Zone::Review));
        let g0 = t.generation();
        assert!(
            t.on(Ev::Open(Zone::Review)).is_empty(),
            "cùng khu: không gì"
        );
        assert_eq!(t.generation(), g0);
        t.on(Ev::Open(Zone::Contests));
        let g1 = t.generation();
        assert_ne!(g1, g0, "đổi đích đổi thế hệ");
        t.on(Ev::Back);
        assert_ne!(t.generation(), g1, "Back đổi thế hệ");
    }

    #[test]
    fn lop_css_cua_lop_phu() {
        assert_eq!(
            overlay_class(Zone::Review, OverlayStep::In),
            "overlay in zone-review"
        );
        assert_eq!(
            overlay_class(Zone::Courses, OverlayStep::Hold),
            "overlay hold zone-courses"
        );
        assert_eq!(
            overlay_class(Zone::Contests, OverlayStep::Out),
            "overlay out zone-contests"
        );
    }
}
