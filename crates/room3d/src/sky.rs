//! Màu và độ sáng của trời theo giờ trong ngày. Giá trị tuyến tính (không gamma), chọn theo
//! bảng màu phòng; phía Bevy chuyển dần sang giá trị mới trong 2 s.

/// Trạng thái trời tại một giờ.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyState {
    /// Màu đỉnh trời (RGB tuyến tính).
    pub top: [f32; 3],
    /// Màu chân trời (RGB tuyến tính).
    pub bottom: [f32; 3],
    /// Màu ánh nắng.
    pub sun_color: [f32; 3],
    /// Cường độ nắng (lux).
    pub sun_lux: f32,
    /// Quang thông đèn bàn (lumen).
    pub lamp_lumens: f32,
    /// Độ hiện của sao, 0..1.
    pub stars: f32,
}

/// Sáng (6–10 giờ).
pub const MORNING: SkyState = SkyState {
    top: [0.45, 0.62, 0.90],
    bottom: [1.00, 0.76, 0.58],
    sun_color: [1.00, 0.85, 0.68],
    sun_lux: 1500.0,
    lamp_lumens: 500.0,
    stars: 0.0,
};

/// Trưa (10–16 giờ).
pub const NOON: SkyState = SkyState {
    top: [0.38, 0.60, 0.92],
    bottom: [0.75, 0.86, 0.96],
    sun_color: [1.00, 0.96, 0.88],
    sun_lux: 2500.0,
    lamp_lumens: 300.0,
    stars: 0.0,
};

/// Chiều tà (16–19 giờ).
pub const DUSK: SkyState = SkyState {
    top: [0.36, 0.27, 0.58],
    bottom: [0.98, 0.52, 0.36],
    sun_color: [1.00, 0.62, 0.40],
    sun_lux: 900.0,
    lamp_lumens: 900.0,
    stars: 0.0,
};

/// Đêm (19–6 giờ).
pub const NIGHT: SkyState = SkyState {
    top: [0.03, 0.04, 0.12],
    bottom: [0.08, 0.09, 0.22],
    sun_color: [0.45, 0.55, 1.00],
    sun_lux: 60.0,
    lamp_lumens: 1600.0,
    stars: 1.0,
};

fn mix(a: SkyState, b: SkyState) -> SkyState {
    let m3 = |x: [f32; 3], y: [f32; 3]| {
        [
            (x[0] + y[0]) / 2.0,
            (x[1] + y[1]) / 2.0,
            (x[2] + y[2]) / 2.0,
        ]
    };
    SkyState {
        top: m3(a.top, b.top),
        bottom: m3(a.bottom, b.bottom),
        sun_color: m3(a.sun_color, b.sun_color),
        sun_lux: (a.sun_lux + b.sun_lux) / 2.0,
        lamp_lumens: (a.lamp_lumens + b.lamp_lumens) / 2.0,
        stars: (a.stars + b.stars) / 2.0,
    }
}

/// Trời ở giờ `hour` (`hour >= 24` lấy theo `% 24`). Giờ đầu mỗi khoảng (6, 10, 16, 19) trộn
/// 50/50 giữa trạng thái trước và trạng thái của khoảng đó; các giờ khác là trạng thái thuần.
pub fn sky_at(hour: u8) -> SkyState {
    match hour % 24 {
        6 => mix(NIGHT, MORNING),
        10 => mix(MORNING, NOON),
        16 => mix(NOON, DUSK),
        19 => mix(DUSK, NIGHT),
        7..=9 => MORNING,
        11..=15 => NOON,
        17..=18 => DUSK,
        _ => NIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avg(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            (a[0] + b[0]) / 2.0,
            (a[1] + b[1]) / 2.0,
            (a[2] + b[2]) / 2.0,
        ]
    }

    #[test]
    fn troi_theo_gio() {
        assert_eq!(sky_at(0), NIGHT);
        assert_eq!(sky_at(12), NOON);
        assert_eq!(sky_at(18), DUSK);
        assert_eq!(sky_at(23), NIGHT);
        let s = sky_at(6);
        for (got, want) in [
            (s.top, avg(NIGHT.top, MORNING.top)),
            (s.bottom, avg(NIGHT.bottom, MORNING.bottom)),
            (s.sun_color, avg(NIGHT.sun_color, MORNING.sun_color)),
        ] {
            for i in 0..3 {
                assert!((got[i] - want[i]).abs() < 1e-6);
            }
        }
        assert!((s.sun_lux - (NIGHT.sun_lux + MORNING.sun_lux) / 2.0).abs() < 1e-6);
        assert!((s.lamp_lumens - (NIGHT.lamp_lumens + MORNING.lamp_lumens) / 2.0).abs() < 1e-6);
        assert!((s.stars - 0.5).abs() < 1e-6);
    }

    #[test]
    fn troi_moi_gio_hop_le() {
        for h in 0..48u8 {
            let s = sky_at(h);
            let all = s
                .top
                .iter()
                .chain(&s.bottom)
                .chain(&s.sun_color)
                .copied()
                .chain([s.sun_lux, s.lamp_lumens, s.stars]);
            for v in all {
                assert!(v.is_finite() && v >= 0.0, "h={h} v={v}");
            }
        }
    }

    #[test]
    fn dem_den_ban_sang_hon() {
        const { assert!(NIGHT.lamp_lumens > NOON.lamp_lumens) };
        assert_eq!(NIGHT.stars, 1.0);
        for s in [MORNING, NOON, DUSK] {
            assert_eq!(s.stars, 0.0);
        }
    }
}
