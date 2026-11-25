# ADR 0006: Ranh giới crate và hướng phụ thuộc

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Khi phát triển bằng vibecode, AI có xu hướng "tiện tay" thêm phụ thuộc làm mờ ranh giới (vd. gọi thẳng database từ logic, import thư viện UI vào crate thuần). Ranh giới phải được **máy kiểm tra**, không chỉ ghi trong tài liệu.

## Quyết định

Workspace chia theo ranh giới trách nhiệm:

| Crate | Trách nhiệm | Được phụ thuộc (nội bộ) | Loại |
|---|---|---|---|
| `domain` | Kiểu dùng chung: mã định danh, khái niệm, hiểu sai, quan sát | — | Thuần |
| `manip` | Mô hình: trạng thái, hành động, áp dụng, phát lại | `domain` | Thuần |
| `diagnose` | Chấm và chẩn đoán từ dấu vết | `domain`, `manip` | Thuần |
| `content` | Đọc và kiểm tra file nội dung | `domain`, `manip`, `diagnose` (kiểm tra nhiệm vụ và mã chẩn đoán — ADR 0009) | Có IO |
| `cli` | Công cụ dòng lệnh `dyen` (phát lại, kiểm tra nội dung) | mọi crate trên | Có IO |
| `xtask` | Công cụ dự án (CI cục bộ, kiểm tra ranh giới) | — | Công cụ |

Sẽ thêm sau (chưa tạo): `web` (Leptos, bộ hiển thị 2D), `table3d` (Bevy), `server` (Axum).

**Crate thuần** (`domain`, `manip`, `diagnose`) không được phụ thuộc thư viện bên ngoài nào ngoài danh sách cho phép trong `xtask/src/deps.rs` (hiện tại: `serde`, `thiserror` và phụ thuộc bắc cầu của chúng). Không IO, không mạng, không đồng hồ, không thread.

`cargo xtask deps` kiểm tra bảng trên bằng `cargo metadata`; chạy trong `cargo xtask ci`.

## Hệ quả

- Thêm phụ thuộc vào crate thuần là một quyết định có chủ đích: sửa danh sách cho phép trong `xtask` và ghi lý do trong commit.
- Crate thuần biên dịch được sang wasm mà không cần cấu hình đặc biệt.
- Khi thêm `web`/`table3d`/`server`, cập nhật bảng ở đây và trong `xtask` cùng một lúc.
