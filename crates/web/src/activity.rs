//! Màn hình làm một hoạt động: tiêu đề, đề bài, bộ hiển thị của loại.
//!
//! Nút về danh mục nằm ở thanh tiêu đề (`App`).

use content::Activity;
use leptos::prelude::*;
use registry::AnyConfig;

use crate::kinds;

/// Chọn bộ hiển thị theo loại. Đây là chỗ duy nhất trong `web` rẽ nhánh theo loại (ADR 0010).
fn renderer(config: &AnyConfig) -> AnyView {
    match config {
        AnyConfig::Bars(c) => kinds::bars::view(c.clone()),
        AnyConfig::Choice(c) => kinds::choice::view(c.clone()),
        AnyConfig::Section(c) => kinds::section::view(c.clone()),
    }
}

/// Vẽ màn hình hoạt động.
pub fn view(activity: &'static Activity) -> impl IntoView {
    view! {
        <h2>{activity.title.clone()}</h2>
        <p class="prompt">{activity.prompt.clone()}</p>
        {renderer(&activity.config)}
    }
}
