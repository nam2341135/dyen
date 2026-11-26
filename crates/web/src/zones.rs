//! Ba khu của vỏ: tên, đồ vật tương ứng, danh sách chữ và trang "Sắp có".

use leptos::prelude::*;
use room_model::Zone;

use crate::route::Route;

/// Tên khu hiển thị cho học sinh.
pub fn label(z: Zone) -> &'static str {
    match z {
        Zone::Review => "Ôn tập",
        Zone::Courses => "Khoá học",
        Zone::Contests => "Thi đấu",
    }
}

/// Tên đồ vật đại diện khu trong phòng.
pub fn object(z: Zone) -> &'static str {
    match z {
        Zone::Review => "Vở",
        Zone::Courses => "Máy tính",
        Zone::Contests => "Cúp",
    }
}

/// Màn vào của khu.
pub fn route(z: Zone) -> Route {
    match z {
        Zone::Review => Route::Review,
        Zone::Courses => Route::Courses,
        Zone::Contests => Route::Contests,
    }
}

/// Danh sách chữ ba khu (lối vào thay thế cho phòng 3D). Khi `collapsed`, chỉ còn nút
/// "Danh sách khu" ở góc; bấm thì mở lại. `on_focus` báo khu đang trỏ tới hoặc rời đi.
pub fn list(
    on_pick: impl Fn(Zone) + Copy + 'static,
    on_focus: impl Fn(Option<Zone>) + Copy + 'static,
    collapsed: Signal<bool>,
) -> impl IntoView {
    let forced_open = RwSignal::new(false);
    let hidden = move || collapsed.get() && !forced_open.get();
    let items = Zone::ALL
        .into_iter()
        .map(|z| {
            view! {
                <li>
                    <button
                        class="zone"
                        on:click=move |_| {
                            forced_open.set(false);
                            on_pick(z)
                        }
                        on:pointerenter=move |_| on_focus(Some(z))
                        on:pointerleave=move |_| on_focus(None)
                        on:focus=move |_| on_focus(Some(z))
                        on:blur=move |_| on_focus(None)
                    >
                        {format!("{} · {}", object(z), label(z))}
                    </button>
                </li>
            }
        })
        .collect_view();
    view! {
        <nav class="zones" class:collapsed=move || collapsed.get() aria-label="Các khu">
            <button
                class="zones-toggle secondary"
                aria-expanded=move || (!hidden()).to_string()
                on:click=move |_| forced_open.update(|o| *o = !*o)
            >
                "Danh sách khu"
            </button>
            <ul hidden=hidden>{items}</ul>
        </nav>
    }
}

/// Trang "Sắp có" của một khu chưa làm.
pub fn coming_soon(z: Zone) -> impl IntoView {
    view! {
        <h2>{label(z)}</h2>
        <p class="soon">"Sắp có"</p>
        <button on:click=|_| crate::route::go(&Route::Room)>"Về phòng"</button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn danh_sach_du_ba_khu() {
        let pairs: std::collections::BTreeSet<_> = Zone::ALL
            .into_iter()
            .map(|z| (label(z), object(z)))
            .collect();
        assert_eq!(pairs.len(), 3);
        assert!(pairs.contains(&("Ôn tập", "Vở")));
        assert!(pairs.contains(&("Khoá học", "Máy tính")));
        assert!(pairs.contains(&("Thi đấu", "Cúp")));
    }
}
