//! Đo fps trong 3 giây đầu để quyết định có hạ chất lượng không.

/// Thời gian đo (giây).
const WINDOW: f32 = 3.0;
/// Dưới ngưỡng này (khung/giây trung bình) thì hạ chất lượng.
const LOW_FPS: f32 = 30.0;

/// Bộ đo fps; chỉ cho kết quả đúng một lần.
#[derive(Debug, Clone, Default)]
pub struct FpsProbe {
    elapsed: f32,
    frames: u32,
    done: bool,
}

impl FpsProbe {
    /// Bộ đo mới.
    pub fn new() -> Self {
        Self::default()
    }

    /// Báo một khung dài `dt` giây. Khi đủ 3 s trả `Some(true)` nếu fps trung bình dưới 30
    /// (nên hạ chất lượng), ngược lại `Some(false)`; mọi lần khác trả `None`.
    pub fn frame(&mut self, dt: f32) -> Option<bool> {
        if self.done {
            return None;
        }
        self.elapsed += dt;
        self.frames += 1;
        if self.elapsed < WINDOW {
            return None;
        }
        self.done = true;
        Some((self.frames as f32 / self.elapsed) < LOW_FPS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dt: f32, secs: f32) -> Vec<Option<bool>> {
        let mut p = FpsProbe::new();
        let n = (secs / dt) as usize;
        (0..n).map(|_| p.frame(dt)).collect()
    }

    #[test]
    fn may_cham_ha_mot_lan() {
        let r = run(1.0 / 20.0, 3.1);
        let some: Vec<_> = r.iter().flatten().collect();
        assert_eq!(some, [&true]);
    }

    #[test]
    fn may_nhanh_giu_nguyen() {
        let r = run(1.0 / 60.0, 3.1);
        let some: Vec<_> = r.iter().flatten().collect();
        assert_eq!(some, [&false]);
    }
}
