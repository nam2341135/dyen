//! Phần vẽ Bevy (chỉ wasm): dựng cảnh, nhận thông điệp của vỏ, picking, bay camera, báo về vỏ.
//!
//! Mọi quyết định nằm ở [`Ctl`]; ở đây chỉ chuyển đầu vào (thông điệp, chuột/chạm, đồng hồ
//! `Time::elapsed_secs`) vào `Ctl` và vẽ theo những gì nó trả ra.

mod look;
mod pick;
mod scene;

use std::cell::RefCell;
use std::time::Duration;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::winit::{
    EventLoopProxy, EventLoopProxyWrapper, UpdateMode, WinitSettings, WinitUserEvent,
};
use room_model::{FromRoom, Quality, ToRoom};
use wasm_bindgen::prelude::*;

use crate::ctl::Ctl;
use crate::fps::FpsProbe;
use crate::inbox;

thread_local! {
    /// Đánh thức vòng lặp winit khi có thông điệp mới (cần khi đang tạm dừng: vòng lặp ngủ).
    static WAKER: RefCell<Option<EventLoopProxy<WinitUserEvent>>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
extern "C" {
    /// Phòng → vỏ: một `FromRoom` dạng JSON (hàm `window.dyenFromRoom` của `index.html`).
    #[wasm_bindgen(js_name = dyenFromRoom)]
    fn from_room(json: &str);
}

/// Gửi một thông điệp về vỏ.
fn send(msg: &FromRoom) {
    match serde_json::to_string(msg) {
        Ok(json) => from_room(&json),
        Err(e) => error!("không mã hoá được {msg:?}: {e}"),
    }
}

/// Gửi lần lượt các thông điệp `Ctl` trả ra.
fn send_all(msgs: Vec<FromRoom>) {
    for m in &msgs {
        send(m);
    }
}

/// Vỏ → phòng: một `ToRoom` dạng JSON. Xếp vào hàng đợi; khung kế tiếp xử lý theo thứ tự.
#[wasm_bindgen]
pub fn push(json: String) {
    inbox::push_raw(json);
    WAKER.with(|w| {
        if let Some(p) = &*w.borrow() {
            // Vòng lặp đã đóng thì thôi; không có gì để đánh thức.
            let _ = p.send_event(WinitUserEvent::WakeUp);
        }
    });
}

/// Khởi động phòng 3D trên `<canvas id="room3d">` của vỏ.
///
/// Trên web, winit thoát `start` bằng một ngoại lệ "exceptions for control flow"; vỏ bỏ qua
/// đúng ngoại lệ đó. Thông điệp đầu tiên vỏ gửi qua [`push`] là `Init`.
#[wasm_bindgen]
pub fn start() {
    let window = Window {
        canvas: Some("#room3d".into()),
        fit_canvas_to_parent: true,
        // Không chặn phím mặc định (Tab, Enter của danh sách khu) và không tự lấy focus.
        prevent_default_event_handling: false,
        focused: false,
        ..default()
    };
    let _ = App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                })
                .set(AssetPlugin {
                    // Mô hình nằm cạnh room3d.js trong `dist/room/`.
                    file_path: "room".into(),
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(MeshPickingPlugin)
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .insert_resource(RoomState::new())
        .init_resource::<pick::MouseOver>()
        .add_systems(Startup, (scene::spawn, hook_up))
        .add_systems(
            Update,
            (
                unfocusable,
                read_inbox,
                pick::pointer,
                fly,
                look::highlight,
                look::sky,
                look::quality,
                pick::cursor,
                suspend,
                first_frame,
            )
                .chain(),
        )
        .run();
}

/// Trạng thái phòng phía Bevy: bộ điều khiển và việc báo khung đầu / đo fps.
#[derive(Resource)]
struct RoomState {
    ctl: Ctl,
    /// Đã nhận `Init` (viewport, giờ, chất lượng đúng).
    inited: bool,
    /// Số khung đã chạy từ khi có `Init`, đến lúc báo `FirstFrame`.
    frames: u32,
    first_sent: bool,
    fps: FpsProbe,
    /// Bỏ khung kế tiếp khi đo fps (khung ngay sau tạm dừng có `dt` rất dài).
    skip_dt: bool,
}

impl RoomState {
    fn new() -> Self {
        let vp = room_model::Viewport {
            width: 1280,
            height: 720,
        };
        Self {
            ctl: Ctl::new(vp, false),
            inited: false,
            frames: 0,
            first_sent: false,
            fps: FpsProbe::new(),
            skip_dt: false,
        }
    }
}

/// Giữ cách đánh thức vòng lặp, và báo `Failed` cho vỏ nếu phòng panic.
fn hook_up(proxy: Res<EventLoopProxyWrapper>) {
    let p: EventLoopProxy<WinitUserEvent> = (**proxy).clone();
    WAKER.with(|w| *w.borrow_mut() = Some(p));
    // Chạy sau khi Bevy đặt hook của nó (in lỗi ra console), nên bọc lấy hook đó.
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        prev(info);
        send(&FromRoom::Failed(format!("room3d: {info}")));
    }));
}

/// winit đặt `tabindex` cho canvas để nhận phím; phòng không dùng phím, và canvas không được
/// chen vào thứ tự Tab của danh sách khu. Bỏ thuộc tính đó một lần khi canvas đã có nó.
fn unfocusable(mut done: Local<bool>) {
    if *done {
        return;
    }
    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("room3d"));
    if let Some(c) = canvas
        && c.has_attribute("tabindex")
    {
        let _ = c.remove_attribute("tabindex");
        *done = true;
    }
}

/// Lấy hết thông điệp của vỏ, chuyển vào `Ctl`. JSON hỏng thì báo `Failed` và đi tiếp.
fn read_inbox(mut room: ResMut<RoomState>, time: Res<Time>) {
    let now = time.elapsed_secs();
    for raw in inbox::drain() {
        match serde_json::from_str::<ToRoom>(&raw) {
            Ok(msg) => {
                if matches!(msg, ToRoom::Init { .. }) {
                    room.inited = true;
                }
                let out = room.ctl.on_msg(&msg, now);
                send_all(out);
            }
            Err(e) => {
                warn!("thông điệp hỏng {raw:?}: {e}");
                send(&FromRoom::Failed(format!("thông điệp hỏng: {e}")));
            }
        }
    }
}

/// Đặt camera theo pose của `Ctl` và gửi `Arrived` khi bay xong.
fn fly(
    mut room: ResMut<RoomState>,
    time: Res<Time>,
    mut cams: Query<&mut Transform, With<Camera3d>>,
) {
    let (pose, out) = room.ctl.tick(time.elapsed_secs());
    send_all(out);
    for mut t in &mut cams {
        *t = Transform::from_translation(pose.eye).looking_at(pose.look, Vec3::Y);
    }
}

/// Tạm dừng: vòng lặp chỉ chạy khi có sự kiện (hoặc `push` đánh thức) và camera tắt,
/// nên GPU không vẽ. Tiếp tục: chạy liên tục lại.
fn suspend(
    mut room: ResMut<RoomState>,
    mut was: Local<bool>,
    mut winit: ResMut<WinitSettings>,
    mut cams: Query<&mut Camera>,
) {
    let now = room.ctl.suspended();
    if now == *was {
        return;
    }
    *was = now;
    let mode = if now {
        UpdateMode::reactive_low_power(Duration::from_secs(3600))
    } else {
        room.skip_dt = true;
        UpdateMode::Continuous
    };
    winit.focused_mode = mode;
    winit.unfocused_mode = mode;
    for mut c in &mut cams {
        c.is_active = !now;
    }
}

/// Báo `FirstFrame` sau 2 khung đã vẽ kể từ khi có `Init` (tường, bàn, cửa sổ và ba lối vào
/// dựng ở `Startup`, trước đó). Sau đó đo fps 3 s, bỏ qua lúc tạm dừng; chậm thì hạ chất lượng.
fn first_frame(mut room: ResMut<RoomState>, time: Res<Time>) {
    if !room.inited || room.ctl.suspended() {
        return;
    }
    if !room.first_sent {
        room.frames += 1;
        // Khung thứ 3 chạy nghĩa là 2 khung trước đã được vẽ.
        if room.frames >= 3 {
            room.first_sent = true;
            send(&FromRoom::FirstFrame);
            let page_ms = web_sys::window()
                .and_then(|w| w.performance())
                .map_or(-1.0, |p| p.now());
            info!(
                "room3d: khung đầu ở {page_ms:.0} ms từ lúc mở trang ({:.0} ms từ lúc Bevy chạy)",
                time.elapsed_secs() * 1000.0
            );
        }
        return;
    }
    if std::mem::take(&mut room.skip_dt) {
        return;
    }
    if room.fps.frame(time.delta_secs()) == Some(true) && room.ctl.quality() == Quality::High {
        info!("room3d: fps thấp trong 3 s đầu, hạ chất lượng");
        let now = time.elapsed_secs();
        let out = room.ctl.on_msg(&ToRoom::SetQuality(Quality::Low), now);
        send_all(out);
        // Vỏ cập nhật nút chất lượng và lớp `thap`; chỉ gửi khi phòng tự đổi, không dội lại `SetQuality`.
        send(&FromRoom::Quality(Quality::Low));
    }
}
