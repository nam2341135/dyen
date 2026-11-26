//! Cầu nối phía vỏ tới phòng 3D (`room3d`, gói wasm riêng; bản thiết kế mục 6.4).
//!
//! Vỏ gửi [`ToRoom`] qua hàm JS `dyenToRoom(json)` trong `index.html`; phòng gọi
//! `window.dyenFromRoom(json)`, hàm này chuyển tới [`dyen_from_room`]. Cũng giữ các tùy
//! chọn của phòng trong `localStorage` (chỉ cờ, không phải dữ liệu học tập, ADR 0005).
//! Ngoài trình duyệt (test trên máy chủ), các hàm gọi JS là hàm rỗng hoặc trả `None`.

use room_model::{FromRoom, Quality, ToRoom, Viewport};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    /// `window.dyenToRoom` trong `index.html`.
    #[wasm_bindgen(catch, js_name = dyenToRoom)]
    fn to_room_js(json: &str) -> Result<(), JsValue>;

    /// `window.dyenRoom.load` trong `index.html`.
    #[wasm_bindgen(catch, js_namespace = dyenRoom, js_name = load)]
    fn load_js(init: &str) -> Result<(), JsValue>;
}

/// Khoá `localStorage` của cờ "Phòng đơn giản".
const SIMPLE_KEY: &str = "dyen.simpleRoom";
/// Khoá `localStorage` của mức chất lượng người dùng chọn.
const QUALITY_KEY: &str = "dyen.quality";

/// Gửi một thông điệp tới phòng. Phòng chưa nạp thì JS xếp hàng hoặc bỏ.
pub fn send(msg: &ToRoom) {
    #[cfg(target_arch = "wasm32")]
    match serde_json::to_string(msg) {
        Ok(json) => {
            if let Err(e) = to_room_js(&json) {
                leptos::logging::warn!("dyenToRoom lỗi: {e:?}");
            }
        }
        Err(e) => leptos::logging::warn!("không ghi được thông điệp: {e}"),
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = msg;
}

/// Bắt đầu nạp phòng 3D với thông điệp khởi tạo `init` (`window.dyenRoom.load`).
pub fn load(init: &ToRoom) {
    #[cfg(target_arch = "wasm32")]
    match serde_json::to_string(init) {
        Ok(json) => {
            if let Err(e) = load_js(&json) {
                leptos::logging::warn!("dyenRoom.load lỗi: {e:?}");
            }
        }
        Err(e) => leptos::logging::warn!("không ghi được thông điệp: {e}"),
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = init;
}

type Handler = std::rc::Rc<dyn Fn(FromRoom)>;

thread_local! {
    static HANDLER: std::cell::RefCell<Option<Handler>> = const { std::cell::RefCell::new(None) };
}

/// Đăng ký bộ xử lý thông điệp từ phòng (thay bộ cũ nếu có).
pub fn on_from_room(f: impl Fn(FromRoom) + 'static) {
    HANDLER.with(|h| *h.borrow_mut() = Some(std::rc::Rc::new(f)));
}

/// JS gọi khi phòng gửi một thông điệp (JSON của [`FromRoom`]).
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn dyen_from_room(json: String) {
    let msg = match serde_json::from_str::<FromRoom>(&json) {
        Ok(m) => m,
        Err(e) => {
            leptos::logging::warn!("thông điệp lạ từ phòng ({e}): {json}");
            return;
        }
    };
    // Lấy bản sao rồi mới gọi, để bộ xử lý gửi lại phòng (có thể gọi ngược vào đây) không
    // đụng mượn.
    let handler = HANDLER.with(|h| h.borrow().clone());
    if let Some(f) = handler {
        f(msg);
    }
}

/// Giờ trong ngày theo đồng hồ máy (0..24). Ngoài trình duyệt là 12.
pub fn hour_now() -> u8 {
    #[cfg(target_arch = "wasm32")]
    {
        // `getHours` luôn trong 0..24.
        u8::try_from(js_sys::Date::new_0().get_hours()).unwrap_or(12)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        12
    }
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn read(key: &str) -> Option<String> {
    storage()?.get_item(key).ok()?
}

#[cfg(target_arch = "wasm32")]
fn write(key: &str, value: Option<&str>) {
    if let Some(s) = storage() {
        // Không ghi được (chế độ riêng tư, đầy) thì chỉ mất lựa chọn lần sau.
        let _ = match value {
            Some(v) => s.set_item(key, v),
            None => s.remove_item(key),
        };
    }
}

/// Người dùng đã chọn "Phòng đơn giản" (không nạp 3D).
pub fn simple_room() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        read(SIMPLE_KEY).as_deref() == Some("1")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = SIMPLE_KEY;
        false
    }
}

/// Ghi nhớ lựa chọn "Phòng đơn giản".
pub fn set_simple_room(on: bool) {
    #[cfg(target_arch = "wasm32")]
    write(SIMPLE_KEY, on.then_some("1"));
    #[cfg(not(target_arch = "wasm32"))]
    let _ = on;
}

/// Mức chất lượng người dùng đã chọn, nếu có.
pub fn quality_pref() -> Option<Quality> {
    #[cfg(target_arch = "wasm32")]
    {
        match read(QUALITY_KEY).as_deref() {
            Some("high") => Some(Quality::High),
            Some("low") => Some(Quality::Low),
            _ => None,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = QUALITY_KEY;
        None
    }
}

/// Ghi nhớ mức chất lượng người dùng chọn.
pub fn set_quality_pref(q: Quality) {
    #[cfg(target_arch = "wasm32")]
    write(
        QUALITY_KEY,
        Some(match q {
            Quality::High => "high",
            Quality::Low => "low",
        }),
    );
    #[cfg(not(target_arch = "wasm32"))]
    let _ = q;
}

/// Mức chất lượng mặc định (mục 6.8): cao khi màn rộng ≥ 1024 px và không cảm ứng.
pub fn default_quality(width: u32, touch: bool) -> Quality {
    if width >= 1024 && !touch {
        Quality::High
    } else {
        Quality::Low
    }
}

/// `matchMedia(query).matches`; `false` khi không đọc được.
pub fn media(query: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.match_media(query).ok().flatten())
            .is_some_and(|m| m.matches())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = query;
        false
    }
}

/// Kích thước cửa sổ, px CSS.
pub fn viewport() -> Viewport {
    #[cfg(target_arch = "wasm32")]
    {
        let px = |v: Result<JsValue, JsValue>| {
            // Kẹp vào u32; kích thước cửa sổ luôn không âm và nhỏ.
            v.ok()
                .and_then(|v| v.as_f64())
                .map_or(0, |f| f.clamp(0.0, f64::from(u32::MAX)) as u32)
        };
        match web_sys::window() {
            Some(w) => Viewport {
                width: px(w.inner_width()),
                height: px(w.inner_height()),
            },
            None => Viewport {
                width: 0,
                height: 0,
            },
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Viewport {
            width: 0,
            height: 0,
        }
    }
}

/// Người dùng muốn giảm chuyển động.
pub fn reduced_motion() -> bool {
    media("(prefers-reduced-motion: reduce)")
}

/// Mức chất lượng đang dùng: lựa chọn đã lưu, không có thì mặc định theo máy.
pub fn current_quality() -> Quality {
    quality_pref().unwrap_or_else(|| default_quality(viewport().width, media("(pointer: coarse)")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_luong_mac_dinh() {
        assert_eq!(default_quality(1280, false), Quality::High);
        assert_eq!(default_quality(1280, true), Quality::Low);
        assert_eq!(default_quality(1023, false), Quality::Low);
        assert_eq!(default_quality(1024, false), Quality::High);
    }

    #[test]
    fn thong_diep_tu_phong_toi_bo_xu_ly() {
        let got = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let sink = got.clone();
        on_from_room(move |m| sink.borrow_mut().push(m));
        dyen_from_room(r#"{"t":"Failed","v":"x"}"#.to_owned());
        dyen_from_room("rác".to_owned());
        assert_eq!(*got.borrow(), vec![FromRoom::Failed("x".to_owned())]);
    }
}
