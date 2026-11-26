//! Định tuyến theo `location.hash`: mỗi màn của vỏ có đúng một hash, để nút Back của
//! trình duyệt hoạt động (ADR 0002: phòng chỉ là lối điều hướng).

use content::Library;
use kernel::ActivityId;
use room_model::Zone;

/// Một màn của vỏ ứng dụng.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// Màn chính (phòng).
    Room,
    /// Danh mục ôn tập.
    Review,
    /// Làm một hoạt động.
    Activity(ActivityId),
    /// Khu khóa học.
    Courses,
    /// Khu thi đấu.
    Contests,
}

impl Route {
    /// Đọc một `location.hash`. Hash lạ về `Room`; mã hoạt động không có trong `lib` về `Review`.
    pub fn parse(hash: &str, lib: &Library) -> Route {
        let path = hash.strip_prefix('#').unwrap_or(hash);
        match path {
            "" | "/" => Route::Room,
            "/on-tap" => Route::Review,
            "/khoa-hoc" => Route::Courses,
            "/thi-dau" => Route::Contests,
            _ => match path.strip_prefix("/on-tap/") {
                Some(code) => ActivityId::new(code)
                    .ok()
                    .filter(|id| lib.activities().contains_key(id))
                    .map_or(Route::Review, Route::Activity),
                None => Route::Room,
            },
        }
    }

    /// Hash của màn này (đọc lại bằng [`Route::parse`] ra đúng màn này).
    pub fn to_hash(&self) -> String {
        match self {
            Route::Room => "#/".to_owned(),
            Route::Review => "#/on-tap".to_owned(),
            Route::Activity(id) => format!("#/on-tap/{id}"),
            Route::Courses => "#/khoa-hoc".to_owned(),
            Route::Contests => "#/thi-dau".to_owned(),
        }
    }

    /// Màn cha, một cấp lên.
    pub fn up(&self) -> Route {
        match self {
            Route::Activity(_) => Route::Review,
            _ => Route::Room,
        }
    }

    /// Khu chứa màn này, nếu có.
    pub fn zone(&self) -> Option<Zone> {
        match self {
            Route::Room => None,
            Route::Review | Route::Activity(_) => Some(Zone::Review),
            Route::Courses => Some(Zone::Courses),
            Route::Contests => Some(Zone::Contests),
        }
    }
}

/// Đi tới một màn: chỉ đặt `location.hash`; `hashchange` mới cập nhật vỏ (để Back hoạt động).
pub fn go(route: &Route) {
    if let Some(w) = web_sys::window() {
        // Lỗi đặt hash (không có trên trang thật) không có gì để làm, bỏ qua.
        let _ = w.location().set_hash(&route.to_hash());
    }
}

/// Đẩy một mục lịch sử chặn `#/` (lúc bắt đầu lượt bay), để Back của trình duyệt huỷ lượt
/// bay thay vì rời trang. `pushState` không phát `hashchange` hay `popstate`.
pub fn push_guard() {
    if let Some(h) = web_sys::window().and_then(|w| w.history().ok()) {
        // Lỗi (không có trên trang thật) thì Back như cũ, bỏ qua.
        let _ = h.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("#/"));
    }
}

/// Thay mục lịch sử hiện tại bằng màn `route` (vẫn phát `hashchange`).
pub fn replace(route: &Route) {
    if let Some(w) = web_sys::window() {
        // Như `go`: lỗi không có gì để làm.
        let _ = w.location().replace(&route.to_hash());
    }
}

/// Lùi một mục lịch sử (`history.back()`), để bỏ mục chặn.
pub fn back() {
    if let Some(h) = web_sys::window().and_then(|w| w.history().ok()) {
        // Như `go`: lỗi không có gì để làm.
        let _ = h.back();
    }
}

/// `location.hash` hiện tại ("" nếu không đọc được).
pub fn current_hash() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib() -> Library {
        crate::library::load().expect("nội dung nhúng hợp lệ")
    }

    fn some_id(lib: &Library) -> ActivityId {
        lib.activities()
            .keys()
            .next()
            .expect("có hoạt động")
            .clone()
    }

    #[test]
    fn doc_moi_hash_hop_le() {
        let lib = lib();
        let id = some_id(&lib);
        assert_eq!(Route::parse("", &lib), Route::Room);
        assert_eq!(Route::parse("#", &lib), Route::Room);
        assert_eq!(Route::parse("#/", &lib), Route::Room);
        assert_eq!(Route::parse("#/on-tap", &lib), Route::Review);
        assert_eq!(
            Route::parse(&format!("#/on-tap/{id}"), &lib),
            Route::Activity(id)
        );
        assert_eq!(Route::parse("#/khoa-hoc", &lib), Route::Courses);
        assert_eq!(Route::parse("#/thi-dau", &lib), Route::Contests);
    }

    #[test]
    fn hash_rac_ve_phong() {
        let lib = lib();
        for h in ["#/abc", "#on-tap", "%%%", "#/thi-dau/x"] {
            assert_eq!(Route::parse(h, &lib), Route::Room, "{h}");
        }
        assert_eq!(Route::parse("#/on-tap/", &lib), Route::Review);
    }

    #[test]
    fn ma_hoat_dong_la_ve_danh_muc() {
        assert_eq!(Route::parse("#/on-tap/khong-co", &lib()), Route::Review);
    }

    #[test]
    fn ghi_roi_doc_lai_khong_doi() {
        let lib = lib();
        let mut routes = vec![Route::Room, Route::Review, Route::Courses, Route::Contests];
        routes.extend(lib.activities().keys().cloned().map(Route::Activity));
        for r in routes {
            assert_eq!(Route::parse(&r.to_hash(), &lib), r);
        }
    }

    #[test]
    fn len_mot_cap() {
        let lib = lib();
        assert_eq!(Route::Activity(some_id(&lib)).up(), Route::Review);
        for r in [Route::Review, Route::Courses, Route::Contests, Route::Room] {
            assert_eq!(r.up(), Route::Room);
        }
    }
}
