//! Phiên làm một hoạt động: vỏ giữ `Session` có thẩm quyền (ADR 0002).
//!
//! Phần thuần (không Leptos), để test được trên máy host.

use kernel::{Actor, ConfigError, Kind, Outcome, Session};

/// Một lần làm hoạt động: phiên có thẩm quyền, mốc thời gian bắt đầu, lỗi gần nhất.
pub struct Play<K: Kind> {
    session: Session<K>,
    started_ms: u32,
    last_error: Option<String>,
}

impl<K: Kind> Play<K> {
    /// Bắt đầu phiên mới; `now_ms` là đồng hồ của trang lúc bắt đầu.
    pub fn new(config: K::Config, now_ms: u32) -> Result<Self, ConfigError> {
        Ok(Self {
            session: Session::new(config)?,
            started_ms: now_ms,
            last_error: None,
        })
    }

    /// Cấu hình của hoạt động.
    pub fn config(&self) -> &K::Config {
        self.session.config()
    }

    /// Trạng thái hiện tại.
    pub fn state(&self) -> &K::State {
        self.session.state()
    }

    /// Số bước đã được chấp nhận.
    pub fn steps(&self) -> usize {
        self.session.trace().entries.len()
    }

    /// Thời điểm (ms từ lúc bắt đầu phiên) của từng bước.
    pub fn trace_times(&self) -> Vec<u32> {
        self.session
            .trace()
            .entries
            .iter()
            .map(|e| e.at_ms)
            .collect()
    }

    /// Đầu vào gần nhất đã được chấp nhận.
    pub fn last_input(&self) -> Option<&K::Input> {
        self.session.trace().entries.last().map(|e| &e.input)
    }

    /// Lý do đầu vào gần nhất bị từ chối; `None` nếu bước gần nhất hợp lệ.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Đưa một đầu vào của người học qua `Session::act`. Bị từ chối thì trạng thái giữ nguyên.
    pub fn act(&mut self, now_ms: u32, input: K::Input) {
        let at_ms = now_ms.saturating_sub(self.started_ms);
        self.last_error = self
            .session
            .act(at_ms, Actor::Learner, input)
            .err()
            .map(|e| e.to_string());
    }

    /// Làm lại từ đầu với cùng cấu hình.
    pub fn reset(&mut self, now_ms: u32) {
        let config = self.session.config().clone();
        // Cấu hình đã dựng được phiên một lần nên `new` không lỗi; nếu có, giữ phiên cũ.
        if let Ok(fresh) = Self::new(config, now_ms) {
            *self = fresh;
        }
    }

    /// Kết quả đánh giá dấu vết hiện tại (`Kind::evaluate`); `None` khi chưa có bước nào.
    pub fn outcome(&self) -> Option<Outcome> {
        if self.steps() == 0 {
            return None;
        }
        kernel::run::<K>(self.session.config(), self.session.trace())
            .ok()
            .map(|(_, outcome)| outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kind_choice::{Choice, Input};

    fn config() -> kind_choice::Config {
        let lib = crate::library::load().expect("nội dung nhúng hợp lệ");
        let act = lib
            .activities()
            .values()
            .find(|a| a.id.as_str() == "frac-largest-unit")
            .expect("có hoạt động frac-largest-unit");
        match &act.config {
            registry::AnyConfig::Choice(c) => c.clone(),
            _ => panic!("frac-largest-unit là loại choice"),
        }
    }

    #[test]
    fn act_tu_choi_giu_nguyen_trang_thai() {
        let mut play = Play::<Choice>::new(config(), 0).expect("cấu hình hợp lệ");
        let before = play.state().clone();
        play.act(10, Input::Submit);
        assert!(play.last_error().is_some());
        assert_eq!(play.state(), &before);
        assert_eq!(play.steps(), 0);
        assert_eq!(play.outcome(), None);
    }

    #[test]
    fn chon_dung_thi_dat() {
        let mut play = Play::<Choice>::new(config(), 1000).expect("cấu hình hợp lệ");
        play.act(1200, Input::Select { key: "half".into() });
        assert_eq!(play.last_error(), None);
        play.act(1500, Input::Submit);
        let outcome = play.outcome().expect("đã có bước");
        assert!(outcome.completed);
        // Thời điểm tính từ lúc bắt đầu phiên.
        assert_eq!(play.trace_times(), vec![200, 500]);
        assert_eq!(play.last_input(), Some(&Input::Submit));
    }

    #[test]
    fn loi_bien_mat_khi_buoc_sau_hop_le() {
        let mut play = Play::<Choice>::new(config(), 0).expect("cấu hình hợp lệ");
        play.act(1, Input::Submit);
        play.act(
            2,
            Input::Select {
                key: "third".into(),
            },
        );
        assert_eq!(play.last_error(), None);
    }

    #[test]
    fn lam_lai_tao_phien_moi() {
        let mut play = Play::<Choice>::new(config(), 0).expect("cấu hình hợp lệ");
        play.act(
            5,
            Input::Select {
                key: "third".into(),
            },
        );
        play.act(6, Input::Submit);
        play.reset(100);
        assert_eq!(play.steps(), 0);
        assert_eq!(play.outcome(), None);
        assert_eq!(play.last_error(), None);
        play.act(150, Input::Select { key: "half".into() });
        assert_eq!(play.trace_times(), vec![50]);
    }
}
