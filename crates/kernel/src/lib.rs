//! Lõi của dyen: hợp đồng chung cho mọi loại hoạt động (ADR 0010).
//!
//! Lõi không biết gì về môn học hay sư phạm. Nó chỉ định nghĩa:
//! - [`Kind`] — một loại hoạt động: cấu hình, trạng thái, đầu vào, áp dụng, đánh giá;
//! - [`Trace`], [`replay`], [`run`], [`Session`] — nhật ký đầu vào và phát lại tất định (ADR 0003);
//! - [`Outcome`] — kết quả đánh giá: điểm, hoàn thành, phát hiện gắn nhãn;
//! - mã định danh ổn định cho nội dung;
//! - [`contract`] — bộ kiểm hợp đồng dùng lại cho mọi loại hoạt động.
//!
//! Crate thuần (ADR 0006): không IO, không mạng, không đồng hồ, không thread.

pub mod contract;
mod id;
mod outcome;
mod trace;

use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub use id::{ActivityId, ActivitySetId, InvalidId, TagId};
pub use outcome::{Confidence, Finding, Outcome, Score, ScoreOutOfRange};
pub use trace::{
    Actor, Entry, Replay, ReplayError, Session, SessionError, TRACE_FORMAT, Trace, replay, run,
};

/// Một loại hoạt động.
///
/// Hợp đồng (kiểm bằng [`contract::check`]):
/// - `init`, `apply`, `evaluate` là hàm thuần, tất định — cùng đầu vào, cùng kết quả, mọi máy;
/// - `apply` từ chối đầu vào không hợp lệ thay vì sửa lặng lẽ;
/// - `evaluate` chỉ dùng bước của [`Actor::Learner`] làm bằng chứng, bằng chứng tăng dần,
///   và chỉ dùng nhãn trong [`Kind::TAGS`].
pub trait Kind {
    /// Tên loại, ổn định — dùng trong file nội dung và dữ liệu lưu.
    const KIND: &'static str;
    /// Mọi nhãn `evaluate` có thể gắn vào phát hiện.
    const TAGS: &'static [&'static str];
    /// Cấu hình của một hoạt động, đọc từ file nội dung.
    type Config: Clone + Debug + PartialEq + Serialize + DeserializeOwned;
    /// Toàn bộ trạng thái.
    type State: Clone + Debug + PartialEq + Serialize + DeserializeOwned;
    /// Một đầu vào nguyên tử từ người học hoặc hệ thống.
    type Input: Clone + Debug + PartialEq + Serialize + DeserializeOwned;

    /// Dựng trạng thái đầu; lỗi nếu cấu hình không hợp lệ.
    fn init(config: &Self::Config) -> Result<Self::State, ConfigError>;

    /// Áp dụng một đầu vào, trả về trạng thái mới hoặc lý do từ chối.
    fn apply(state: &Self::State, input: &Self::Input) -> Result<Self::State, Rejection>;

    /// Đánh giá một dấu vết đã phát lại được.
    ///
    /// `states` là kết quả [`replay`] của chính `trace`: `states[i + 1]` là trạng thái sau bước `i`.
    fn evaluate(
        config: &Self::Config,
        trace: &Trace<Self::Input>,
        states: &[Self::State],
    ) -> Outcome;
}

/// Cấu hình không hợp lệ.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("cấu hình không hợp lệ: {0}")]
pub struct ConfigError(pub String);

/// Đầu vào bị từ chối.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("đầu vào bị từ chối: {0}")]
pub struct Rejection(pub String);
