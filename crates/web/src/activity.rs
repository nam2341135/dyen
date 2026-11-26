//! Màn hình làm một hoạt động: tiêu đề, đề bài, bộ hiển thị của loại, nút quay lại.

use content::Activity;
use kernel::ActivityId;
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

/// Vẽ màn hình hoạt động; nút "Quay lại" xoá `current`.
pub fn view(activity: &'static Activity, current: RwSignal<Option<ActivityId>>) -> impl IntoView {
    view! {
        <p><button class="link" on:click=move |_| current.set(None)>"← Quay lại"</button></p>
        <h2>{activity.title.clone()}</h2>
        <p class="prompt">{activity.prompt.clone()}</p>
        {renderer(&activity.config)}
    }
}
