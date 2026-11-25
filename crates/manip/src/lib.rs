//! Mô hình (`Manipulative`): trạng thái, hành động, áp dụng và phát lại dấu vết.
//!
//! Crate thuần và tất định (ADR 0002, 0003, 0009): cùng cấu hình + cùng dấu vết
//! cho cùng kết quả trên mọi nền tảng. Bộ hiển thị không bao giờ sửa trạng thái
//! trực tiếp; chúng chỉ phát `Action` vào [`Session`].

pub mod bars;
mod trace;

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use trace::{
    Actor, Answer, Entry, Replay, ReplayError, Session, SessionError, Step, TRACE_FORMAT, Trace,
    replay,
};

/// Một loại mô hình người học thao tác.
///
/// Hợp đồng: `init` và `apply` là hàm thuần, tất định; `apply` từ chối hành động
/// không hợp lệ thay vì sửa lặng lẽ.
pub trait Manipulative {
    /// Tên loại mô hình, ổn định — dùng trong file nội dung và dữ liệu lưu.
    const KIND: &'static str;
    /// Cấu hình ban đầu, đọc từ file nội dung.
    type Setup: Clone + Debug + PartialEq + Serialize + DeserializeOwned;
    /// Toàn bộ trạng thái của mô hình.
    type State: Clone + Debug + PartialEq + Serialize + DeserializeOwned;
    /// Một thay đổi nguyên tử người học (hoặc hệ thống) gây ra.
    type Action: Clone + Debug + PartialEq + Serialize + DeserializeOwned;

    /// Dựng trạng thái đầu; lỗi nếu cấu hình không hợp lệ.
    fn init(setup: &Self::Setup) -> Result<Self::State, SetupError>;

    /// Áp dụng một hành động, trả về trạng thái mới hoặc lý do từ chối.
    fn apply(state: &Self::State, action: &Self::Action) -> Result<Self::State, Rejection>;
}

/// Cấu hình mô hình không hợp lệ.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("cấu hình mô hình không hợp lệ: {0}")]
pub struct SetupError(pub String);

/// Hành động bị mô hình từ chối.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("hành động bị từ chối: {0}")]
pub struct Rejection(pub String);

/// Cấu hình của mọi loại mô hình — enum đóng (ADR 0009).
///
/// Trong TOML: bảng `[model]` với `kind = "bars"` và các trường của cấu hình.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelSetup {
    /// Thanh cắt được + thước so sánh.
    Bars(bars::Setup),
}

impl ModelSetup {
    /// Kiểm tra cấu hình bằng cách dựng thử trạng thái đầu.
    pub fn validate(&self) -> Result<(), SetupError> {
        match self {
            Self::Bars(s) => bars::Bars::init(s).map(|_| ()),
        }
    }

    /// Tên loại mô hình.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Bars(_) => bars::Bars::KIND,
        }
    }
}
