//! Màn phòng của vỏ: nối máy trạng thái chuyển cảnh ([`crate::transition`]) và cầu nối
//! phòng 3D ([`crate::room`]) vào Leptos.

use std::time::Duration;

use leptos::prelude::*;
use room_model::{FromRoom, Quality, ROOM, ToRoom, Zone, entry_slot};

use crate::room;
use crate::route::{self, Route};
use crate::transition::{Effect, Ev, OverlayStep, Phase, Transition, overlay_class};
use crate::zones;

/// Tình trạng phòng 3D theo góc nhìn của vỏ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreeD {
    /// Đang nạp; ảnh nền chờ và danh sách chữ đầy đủ.
    Loading,
    /// Đã có khung đầu: canvas hiện, danh sách thu lại.
    Ready,
    /// Không có 3D (lỗi, không WebGL2): danh sách chữ đầy đủ, canvas ẩn.
    Off,
}

/// Phần màn phòng đổi theo thông điệp từ phòng.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellView {
    /// Tình trạng 3D.
    pub three_d: ThreeD,
    /// Nhãn của vật lối vào đang được rê tới, ví dụ "Vở · Ôn tập".
    pub label: Option<String>,
}

/// Bước tiếp của màn phòng khi nhận `msg` từ phòng.
pub fn shell_view(prev: ShellView, msg: &FromRoom) -> ShellView {
    match msg {
        FromRoom::FirstFrame => ShellView {
            three_d: ThreeD::Ready,
            ..prev
        },
        FromRoom::Hover(slot) => ShellView {
            label: slot
                .and_then(|id| ROOM.iter().find(|s| s.id == id))
                .and_then(|s| s.zone)
                .map(|z| format!("{} · {}", zones::object(z), zones::label(z))),
            ..prev
        },
        FromRoom::Failed(_) => ShellView {
            three_d: ThreeD::Off,
            label: None,
        },
        FromRoom::Open(_) | FromRoom::Arrived(_) => prev,
    }
}

/// Trạng thái của màn phòng (toàn signal, nên `Copy`): máy chuyển cảnh, lớp phủ, phòng 3D
/// và các lựa chọn của người dùng.
#[derive(Clone, Copy)]
pub struct Shell {
    tr: RwSignal<Transition>,
    overlay: RwSignal<Option<(Zone, OverlayStep)>>,
    opening: RwSignal<bool>,
    view: RwSignal<ShellView>,
    simple: RwSignal<bool>,
    quality: RwSignal<Quality>,
    menu_open: RwSignal<bool>,
    /// Đã gọi `dyenRoom.load` (chỉ gọi một lần mỗi trang).
    loaded: StoredValue<bool>,
}

impl Shell {
    /// Đọc lựa chọn đã lưu và tạo trạng thái ban đầu. Gọi trong `App`.
    pub fn new() -> Self {
        let simple = room::simple_room();
        Shell {
            tr: RwSignal::new(Transition::new(room::reduced_motion())),
            overlay: RwSignal::new(None),
            opening: RwSignal::new(false),
            view: RwSignal::new(ShellView {
                three_d: if simple { ThreeD::Off } else { ThreeD::Loading },
                label: None,
            }),
            simple: RwSignal::new(simple),
            quality: RwSignal::new(room::current_quality()),
            menu_open: RwSignal::new(false),
            loaded: StoredValue::new(false),
        }
    }

    /// Phòng 3D đang chạy và hiện (có khung đầu, không bật "Phòng đơn giản").
    fn running(self) -> bool {
        self.view.with(|v| v.three_d == ThreeD::Ready) && !self.simple.get()
    }

    /// Đưa một sự kiện vào máy chuyển cảnh và làm các việc nó trả về.
    pub fn fire(self, ev: Ev) {
        let fx = self.tr.try_update(|t| t.on(ev)).unwrap_or_default();
        for e in fx {
            self.run(e);
        }
    }

    fn run(self, e: Effect) {
        match e {
            Effect::Overlay(o) => {
                if !matches!(o, Some((_, OverlayStep::Hold))) {
                    self.opening.set(false);
                }
                self.overlay.set(o);
            }
            Effect::Navigate(r) => route::go(&r),
            // "Phòng đơn giản" đang bật: phòng đã dừng, giữ dừng.
            Effect::Send(ToRoom::Resume) if self.simple.get_untracked() => {}
            Effect::Send(m) => room::send(&m),
            Effect::After(ms, ev) => {
                // Hẹn giờ của thế hệ cũ (đã đổi đích, đã Back) thì bỏ.
                let gen_at = self.tr.with_untracked(Transition::generation);
                set_timeout(
                    move || {
                        if self.tr.with_untracked(Transition::generation) == gen_at {
                            self.fire(ev);
                        }
                    },
                    Duration::from_millis(ms.into()),
                );
            }
            Effect::ShowOpeningText => self.opening.set(true),
        }
    }

    /// Bấm một mục trong danh sách chữ.
    pub fn pick(self, z: Zone) {
        if self.running() {
            room::send(&ToRoom::Go(z));
        } else {
            // Không có 3D thì không có lượt bay: phủ ngay, chỉ còn lớp phủ mờ.
            self.fire(Ev::Open(z));
            self.fire(Ev::Arrived(z));
            self.fire(Ev::CoverStart);
        }
    }

    /// Focus (hoặc rê chuột vào) một mục trong danh sách chữ; `None` khi rời đi.
    pub fn focus(self, z: Option<Zone>) {
        room::send(&ToRoom::Highlight(z.map(|z| entry_slot(z).id)));
    }

    /// Danh sách chữ thu lại khi phòng 3D đang hiện.
    pub fn collapsed(self) -> Signal<bool> {
        Signal::derive(move || self.running())
    }

    /// Nạp phòng 3D nếu chưa nạp và người dùng không chọn "Phòng đơn giản".
    fn ensure_loaded(self) {
        if self.loaded.get_value() || self.simple.get_untracked() {
            return;
        }
        self.loaded.set_value(true);
        room::load(&ToRoom::Init {
            hour: room::hour_now(),
            quality: self.quality.get_untracked(),
            reduced_motion: room::reduced_motion(),
            viewport: room::viewport(),
        });
    }

    /// Màn đã đổi (sau `hashchange`, hoặc lúc mở trang với `old == new`).
    pub fn route_changed(self, old: &Route, new: &Route) {
        if *new == Route::Room {
            if *old != Route::Room {
                // Back của trình duyệt, nút "←", "Về phòng": cùng một đường về.
                self.fire(Ev::Back);
            }
            self.ensure_loaded();
        }
        request_animation_frame(move || self.fire(Ev::PageReady));
    }

    /// Đăng ký nghe phòng, phím Esc, đổi cỡ cửa sổ và đồng hồ. Gọi một lần trong `App`.
    pub fn listen(self, route: RwSignal<Route>) {
        room::on_from_room(move |msg| {
            self.view.update(|v| *v = shell_view(v.clone(), &msg));
            match msg {
                FromRoom::Open(z) => self.fire(Ev::Open(z)),
                FromRoom::Arrived(z) => self.fire(Ev::Arrived(z)),
                FromRoom::Failed(e) => leptos::logging::warn!("phòng 3D: Failed: {e}"),
                FromRoom::FirstFrame | FromRoom::Hover(_) => {}
            }
        });
        // Bộ nghe sống suốt đời trang; bỏ handle không gỡ bộ nghe.
        _ = window_event_listener(leptos::ev::keydown, move |e| {
            if e.key() != "Escape" {
                return;
            }
            if self.tr.with_untracked(|t| t.phase != Phase::Idle) {
                self.fire(Ev::Back);
            } else {
                let r = route.get_untracked();
                if r != Route::Room {
                    route::go(&r.up());
                }
            }
        });
        // Gộp đổi cỡ: tối đa một `Resize` mỗi 200 ms, gửi kích thước sau cùng.
        let pending = StoredValue::new(false);
        _ = window_event_listener(leptos::ev::resize, move |_| {
            if pending.get_value() {
                return;
            }
            pending.set_value(true);
            set_timeout(
                move || {
                    pending.set_value(false);
                    room::send(&ToRoom::Resize(room::viewport()));
                },
                Duration::from_millis(200),
            );
        });
        set_interval(
            || room::send(&ToRoom::SetHour(room::hour_now())),
            Duration::from_secs(60),
        );
    }

    fn set_simple(self, on: bool) {
        self.simple.set(on);
        room::set_simple_room(on);
        if on {
            room::send(&ToRoom::Suspend);
        } else if self.loaded.get_value() {
            room::send(&ToRoom::Resume);
        } else {
            self.ensure_loaded();
        }
    }

    fn toggle_quality(self) {
        let q = match self.quality.get_untracked() {
            Quality::High => Quality::Low,
            Quality::Low => Quality::High,
        };
        self.quality.set(q);
        room::set_quality_pref(q);
        room::send(&ToRoom::SetQuality(q));
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

/// Nền phòng: ảnh chờ, canvas 3D, chỉ báo "Đang dọn phòng…". Luôn gắn trong trang (để
/// `room3d` giữ được canvas); lớp `away` khi không ở phòng.
pub fn backdrop(shell: Shell, route: RwSignal<Route>) -> impl IntoView {
    let still_broken = RwSignal::new(false);
    let ready = move || shell.running();
    let canvas_hidden = move || shell.simple.get() || shell.view.with(|v| v.three_d == ThreeD::Off);
    let loading = move || {
        route.get() == Route::Room
            && !shell.simple.get()
            && shell.view.with(|v| v.three_d == ThreeD::Loading)
    };
    view! {
        <div class="room" class:away=move || route.get() != Route::Room aria-hidden="true">
            <img
                class="room-still"
                src="room/still.webp"
                alt=""
                hidden=move || still_broken.get() || ready()
                on:error=move |_| still_broken.set(true)
            />
            <canvas id="room3d" class:ready=ready hidden=canvas_hidden></canvas>
        </div>
        <Show when=loading>
            <p class="room-loading" role="status">
                "Đang dọn phòng…"
            </p>
        </Show>
    }
}

/// Phần màn phòng nằm trên nền: nhãn vật đang rê, danh sách chữ, menu phòng.
pub fn screen(shell: Shell) -> impl IntoView {
    let label = move || shell.view.with(|v| v.label.clone());
    let quality_text = move || match shell.quality.get() {
        Quality::High => "Chất lượng: Cao",
        Quality::Low => "Chất lượng: Thấp",
    };
    view! {
        <p class="room-label" aria-live="polite" hidden=move || label().is_none()>
            {label}
        </p>
        {zones::list(move |z| shell.pick(z), move |z| shell.focus(z), shell.collapsed())}
        <div class="room-menu">
            <button
                class="room-menu-toggle secondary"
                aria-label="Menu phòng"
                aria-expanded=move || shell.menu_open.get().to_string()
                on:click=move |_| shell.menu_open.update(|o| *o = !*o)
            >
                "⋯"
            </button>
            <Show when=move || shell.menu_open.get()>
                <div class="room-menu-items">
                    <button on:click=move |_| shell.toggle_quality()>{quality_text}</button>
                    <button
                        aria-pressed=move || shell.simple.get().to_string()
                        on:click=move |_| shell.set_simple(!shell.simple.get_untracked())
                    >
                        "Phòng đơn giản"
                    </button>
                </div>
            </Show>
        </div>
    }
}

/// Lớp phủ chuyển cảnh (không chặn đầu vào: `pointer-events: none` trong CSS).
pub fn overlay(shell: Shell) -> impl IntoView {
    move || {
        shell.overlay.get().map(|(z, step)| {
            view! {
                <div class=overlay_class(z, step) aria-hidden="true">
                    <span class="overlay-wait"></span>
                    <Show when=move || shell.opening.get()>
                        <p class="overlay-text">"Đang mở…"</p>
                    </Show>
                </div>
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use room_model::SlotId;

    fn start() -> ShellView {
        ShellView {
            three_d: ThreeD::Loading,
            label: None,
        }
    }

    #[test]
    fn that_bai_sau_khung_dau_hien_lai_danh_sach() {
        let v = shell_view(start(), &FromRoom::FirstFrame);
        assert_eq!(v.three_d, ThreeD::Ready);
        let v = shell_view(v, &FromRoom::Failed("mất WebGL".to_owned()));
        assert_eq!(v.three_d, ThreeD::Off);
        assert_eq!(v.label, None);
    }

    #[test]
    fn that_bai_truoc_khung_dau_tat_3d() {
        let v = shell_view(start(), &FromRoom::Failed("không có WebGL2".to_owned()));
        assert_eq!(v.three_d, ThreeD::Off);
    }

    #[test]
    fn nhan_chi_cho_vat_loi_vao() {
        let v = shell_view(start(), &FromRoom::FirstFrame);
        let v = shell_view(v, &FromRoom::Hover(Some(SlotId::Notebook)));
        assert_eq!(v.label.as_deref(), Some("Vở · Ôn tập"));
        let v = shell_view(v, &FromRoom::Hover(Some(SlotId::Mug)));
        assert_eq!(v.label, None);
        let v = shell_view(v, &FromRoom::Hover(Some(SlotId::Trophy)));
        assert_eq!(v.label.as_deref(), Some("Cúp · Thi đấu"));
        let v = shell_view(v, &FromRoom::Hover(None));
        assert_eq!(v.label, None);
        let v = shell_view(v, &FromRoom::Open(Zone::Review));
        assert_eq!(v.three_d, ThreeD::Ready);
    }
}
