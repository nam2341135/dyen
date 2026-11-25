//! Số học và hình học 3D chính xác (ADR 0012).
//!
//! Thư viện thuần, trung lập: không biết loại hoạt động, dấu vết hay sư phạm.
//! Mọi phép tính có thể tràn đều trả [`ExactError`] thay vì panic.
//! Số thực chỉ xuất hiện ở [`Rational::to_f64`], để vẽ.

mod plane;
mod rational;
mod vec3;

pub use plane::Plane;
pub use rational::{ParseRationalError, Rational};
pub use vec3::Vec3;

/// Lỗi số học chính xác.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ExactError {
    /// Kết quả vượt phạm vi `i64` (sau khi tối giản), hoặc trung gian vượt `i128`.
    #[error("tràn số")]
    Overflow,
    /// Chia cho 0.
    #[error("chia cho 0")]
    DivisionByZero,
}
