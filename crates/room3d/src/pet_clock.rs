//! Nhịp của con mèo: cứ 8–15 s làm một cử chỉ (ngẩng đầu). Khoảng chờ lấy từ một bộ sinh số
//! giả ngẫu nhiên xorshift32 có seed cố định, nên mỗi lần mở phòng mèo cử động y như nhau và
//! không đọc đồng hồ thật để lấy ngẫu nhiên. `now` là giây kể từ lúc Bevy chạy.

/// Seed cố định của bộ sinh số.
pub const SEED: u32 = 0x0CA7_5EED;
/// Khoảng chờ ngắn nhất giữa hai cử chỉ (giây).
pub const MIN_GAP: f32 = 8.0;
/// Khoảng chờ dài nhất giữa hai cử chỉ (giây).
pub const MAX_GAP: f32 = 15.0;

/// Đồng hồ cử chỉ của mèo.
#[derive(Clone, Debug)]
pub struct PetClock {
    state: u32,
    next_at: f32,
}

impl PetClock {
    /// Đồng hồ mới; cử chỉ đầu tiên sau `start` một khoảng chờ.
    pub fn new(start: f32) -> Self {
        let mut c = Self {
            state: SEED,
            next_at: 0.0,
        };
        c.next_at = start + c.gap();
        c
    }

    /// Khoảng chờ kế tiếp, trong [`MIN_GAP`, `MAX_GAP`], bước 1 ms.
    fn gap(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        let span_ms = ((MAX_GAP - MIN_GAP) * 1000.0) as u32 + 1;
        MIN_GAP + (x % span_ms) as f32 / 1000.0
    }

    /// Tới lúc làm cử chỉ chưa. Có thì hẹn lần sau và trả `true` (một lần cho mỗi hẹn).
    pub fn due(&mut self, now: f32) -> bool {
        if now < self.next_at {
            return false;
        }
        self.next_at = now + self.gap();
        true
    }

    /// Thời điểm hẹn kế tiếp.
    pub fn next_at(&self) -> f32 {
        self.next_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn khoang_cho_trong_8_den_15_giay() {
        let mut c = PetClock::new(0.0);
        let mut last = 0.0;
        for _ in 0..200 {
            let gap = c.next_at() - last;
            assert!((MIN_GAP..=MAX_GAP).contains(&gap), "{gap}");
            last = c.next_at();
            assert!(c.due(last));
        }
    }

    #[test]
    fn cung_seed_cung_nhip_va_khong_deu() {
        let (mut a, mut b) = (PetClock::new(0.0), PetClock::new(0.0));
        let mut gaps = Vec::new();
        for _ in 0..10 {
            assert_eq!(a.next_at(), b.next_at());
            let t = a.next_at();
            gaps.push(t);
            a.due(t);
            b.due(t);
        }
        let d: Vec<f32> = gaps.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(d.iter().any(|x| (x - d[0]).abs() > 0.5), "{d:?}");
    }

    #[test]
    fn chua_toi_hen_thi_khong_cu_dong() {
        let mut c = PetClock::new(10.0);
        assert!(!c.due(10.0 + MIN_GAP - 0.01));
        let t = c.next_at();
        assert!(c.due(t));
        assert!(!c.due(t + 0.1));
    }
}
