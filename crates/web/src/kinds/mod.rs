//! Bộ hiển thị 2D của từng loại hoạt động. Mỗi bộ chỉ vẽ `State` và phát `Input`
//! qua [`Play::act`]; vỏ giữ phiên có thẩm quyền (ADR 0002).

pub mod bars;
pub mod choice;
pub mod section;

use kernel::Kind;
use leptos::prelude::*;

use crate::{clock, outcome, play::Play};

/// Phiên đang làm, dùng chung giữa bộ hiển thị và phần chân.
pub type PlaySignal<K> = RwSignal<Play<K>, LocalStorage>;

/// Dựng phiên từ cấu hình rồi vẽ `body`; cấu hình hỏng thì báo lỗi thay vì vẽ.
pub fn start<K: Kind + 'static>(
    config: K::Config,
    body: impl FnOnce(PlaySignal<K>) -> AnyView,
) -> AnyView {
    match Play::<K>::new(config, clock::now_ms()) {
        Ok(play) => body(RwSignal::new_local(play)),
        Err(e) => view! { <p class="error">{format!("Cấu hình không hợp lệ: {e}")}</p> }.into_any(),
    }
}

/// Gửi một đầu vào của người học, kèm thời điểm hiện tại.
pub fn act<K: Kind + 'static>(play: PlaySignal<K>, input: K::Input) {
    let now = clock::now_ms();
    play.update(|p| p.act(now, input));
}

/// Phần chân chung: lý do bị từ chối, kết quả (khi `finished`), nút làm lại.
pub fn footer<K: Kind + 'static>(
    play: PlaySignal<K>,
    finished: fn(&Play<K>) -> bool,
) -> impl IntoView {
    let error = move || {
        play.with(|p| p.last_error().map(str::to_owned))
            .map(|e| view! { <p class="error">{e}</p> })
    };
    let result = move || {
        play.with(|p| finished(p).then(|| p.outcome()).flatten())
            .map(|o| outcome::view(&o))
    };
    let retry = move |_| {
        let now = clock::now_ms();
        play.update(|p| p.reset(now));
    };
    view! {
        {error}
        {result}
        <p><button class="secondary" on:click=retry>"Làm lại"</button></p>
    }
}

/// Hằng số đo vùng chạm dùng chung cho test của các bộ hiển thị (ADR 0013).
#[cfg(test)]
pub(crate) mod tests {
    /// Bề rộng hình SVG trên màn 360 px (trừ lề 16 px mỗi bên của `main`).
    pub const PHONE_SVG_PX: f64 = 328.0;
    /// Cạnh nhỏ nhất của một vùng bấm trong hình, px CSS (WCAG 2.5.8).
    pub const MIN_TARGET_PX: f64 = 24.0;
}
