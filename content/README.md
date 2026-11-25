# Nội dung học

Dạng file TOML trong git (ADR 0004), lược đồ ở ADR 0009. `cargo xtask ci` kiểm tra toàn bộ thư mục này.

- `concepts.toml` — `[[concept]]` (`id`, `title`, `requires`) và `[[misconception]]` (`id`, `concept`, `title`).
- `activities/<id>.toml` — `id` (trùng tên file), `title`, `prompt`, `concepts`, bảng `[model]` (`kind = "bars"`, `bars`, `partitions`, `freehand`) và `[task]` (`kind = "compare"` với `left`/`right`, hoặc `kind = "partition"` với `bar`/`parts`).
- `sets/<id>.toml` — `id`, `title`, `activities` theo thứ tự.

Không đổi mã đã phát hành: dấu vết và quan sát trỏ tới mã.
