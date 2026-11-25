# Nội dung học

Dạng file TOML trong git (ADR 0004). `cargo xtask ci` và `dyen check` kiểm tra toàn bộ thư mục này.

- `tags.toml` — `[[tag]]` (`id`, `title`). Phải chứa mọi nhãn mà các loại hoạt động công bố, và mọi nhãn hoạt động đặt trong `[config]` (vd. `tag` của lựa chọn trong loại `choice`).
- `activities/<id>.toml` — `id` (trùng tên file), `title`, `prompt`, `tags` (tùy chọn), và bảng `[config]` với `kind = "<loại hoạt động>"`; các trường còn lại do loại hoạt động định nghĩa (xem README của crate trong `crates/kinds/<loại>/`).
- `sets/<id>.toml` — `id`, `title`, `activities` theo thứ tự.

Không đổi mã đã phát hành: dữ liệu đã lưu trỏ tới mã.
