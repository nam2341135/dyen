//! Vỏ ứng dụng trên trình duyệt (Leptos CSR, ADR 0008).
//!
//! Vỏ giữ trạng thái có thẩm quyền (`kernel::Session`, ADR 0002); bộ hiển thị 2D của
//! từng loại chỉ vẽ `State` và phát `Input`. Đây là chỗ thứ hai (sau `registry`) rẽ
//! nhánh theo loại hoạt động, chỉ để chọn bộ hiển thị (ADR 0010).

pub mod activity;
pub mod catalog;
pub mod clock;
pub mod kinds;
pub mod library;
pub mod outcome;
pub mod play;

use kernel::ActivityId;
use leptos::prelude::*;

/// Gốc ứng dụng: danh mục, hoặc màn hình hoạt động đang chọn.
#[component]
pub fn App() -> impl IntoView {
    let lib = match library::get() {
        Ok(lib) => lib,
        Err(e) => {
            return view! { <pre class="error">{format!("Nội dung lỗi:\n{e}")}</pre> }.into_any();
        }
    };
    let current = RwSignal::new(None::<ActivityId>);
    let screen = move || match current.get().and_then(|id| lib.activities().get(&id)) {
        Some(a) => activity::view(a).into_any(),
        None => catalog::view(lib, current).into_any(),
    };
    // Đang làm một hoạt động thì thanh tiêu đề có nút về danh mục (vùng bấm 44 px, ADR 0013).
    let back = move || {
        current.with(Option::is_some).then(|| {
            view! {
                <button class="back" aria-label="Về danh mục" on:click=move |_| current.set(None)>
                    "←"
                </button>
            }
        })
    };
    view! {
        <header>{back}<h1>"dyen"</h1></header>
        <main>{screen}</main>
    }
    .into_any()
}

/// Điểm vào wasm: gắn ứng dụng vào trang.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    leptos::mount::mount_to_body(App);
}
