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
pub mod room;
pub mod route;
pub mod shell;
pub mod transition;
pub mod zones;

use leptos::prelude::*;
use route::Route;

/// Gốc ứng dụng: phòng (nền 3D và danh sách khu), các khu, lớp phủ chuyển cảnh.
#[component]
pub fn App() -> impl IntoView {
    let lib = match library::get() {
        Ok(lib) => lib,
        Err(e) => {
            return view! { <pre class="error">{format!("Nội dung lỗi:\n{e}")}</pre> }.into_any();
        }
    };
    let route = RwSignal::new(Route::parse(&route::current_hash(), lib));
    let shell = shell::Shell::new();
    // Điều hướng chỉ đặt `location.hash`; `hashchange` cập nhật `route` (Back hoạt động).
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, closure::Closure};
        shell.listen(route);
        let first = route.get_untracked();
        shell.route_changed(&first, &first);
        let on_change = Closure::<dyn Fn()>::new(move || {
            let new = Route::parse(&route::current_hash(), lib);
            let old = route.get_untracked();
            route.set(new.clone());
            shell.route_changed(&old, &new);
        });
        if let Some(w) = web_sys::window() {
            // Lỗi đăng ký nghe là không có gì để làm thêm.
            let _ = w
                .add_event_listener_with_callback("hashchange", on_change.as_ref().unchecked_ref());
        }
        on_change.forget(); // App sống suốt đời trang.
    }
    let go = |r: Route| route::go(&r);
    let screen = move || match route.get() {
        Route::Room => shell::screen(shell).into_any(),
        Route::Review => catalog::view(lib, move |id| go(Route::Activity(id))).into_any(),
        Route::Activity(id) => match lib.activities().get(&id) {
            Some(a) => activity::view(a).into_any(),
            None => catalog::view(lib, move |id| go(Route::Activity(id))).into_any(),
        },
        Route::Courses => zones::coming_soon(room_model::Zone::Courses).into_any(),
        Route::Contests => zones::coming_soon(room_model::Zone::Contests).into_any(),
    };
    // Không ở phòng thì thanh tiêu đề có nút lên một cấp (vùng bấm 44 px, ADR 0013).
    let back = move || {
        (route.get() != Route::Room).then(|| {
            view! {
                <button class="back" aria-label="Lên một cấp" on:click=move |_| go(route.get_untracked().up())>
                    "←"
                </button>
            }
        })
    };
    view! {
        {shell::backdrop(shell, route)}
        <header>{back}<h1>"dyen"</h1></header>
        <main class:on-room=move || route.get() == Route::Room>{screen}</main>
        {shell::overlay(shell)}
    }
    .into_any()
}

/// Điểm vào wasm: gắn ứng dụng vào trang.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    leptos::mount::mount_to_body(App);
}
