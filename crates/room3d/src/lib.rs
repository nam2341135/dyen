//! Cảnh phòng học 3D (ADR 0016). Phần logic thuần (trời, bay camera, chạm, đo fps) nằm ở các
//! module dưới đây và chạy được trên máy thường; phần vẽ Bevy chỉ biên dịch cho wasm.

pub mod ctl;
pub mod flight;
pub mod fps;
pub mod inbox;
pub mod poses;
pub mod sky;

#[cfg(target_arch = "wasm32")]
mod app;

/// Công cụ chung cho test: so sánh số thực có sai số.
#[cfg(test)]
pub(crate) mod testutil {
    use crate::flight::Pose;
    use bevy::math::Vec3;

    /// Hai vectơ gần nhau (sai số 1e-4).
    pub fn near(a: Vec3, b: Vec3) -> bool {
        (a - b).abs().max_element() < 1e-4
    }

    /// Hai tư thế gần nhau.
    pub fn pose_near(a: Pose, b: Pose) -> bool {
        near(a.eye, b.eye) && near(a.look, b.look)
    }
}
