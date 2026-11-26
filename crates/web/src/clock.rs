//! Đồng hồ của trang, chỉ dùng để ghi thời điểm vào dấu vết (không dùng cho quyết định).

/// Mili giây từ lúc trang mở (`performance.now()`); `0` khi không chạy trong trình duyệt.
pub fn now_ms() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            // Làm tròn xuống, kẹp vào u32 (đủ cho ~49 ngày).
            .map_or(0, |p| p.now().clamp(0.0, f64::from(u32::MAX)) as u32)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}
