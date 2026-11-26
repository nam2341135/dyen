# ADR 0006: Ranh giới crate và hướng phụ thuộc

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Khi phát triển bằng vibecode, AI có xu hướng "tiện tay" thêm phụ thuộc làm mờ ranh giới (vd. gọi thẳng database từ logic, import thư viện UI vào crate thuần). Ranh giới phải được **máy kiểm tra**, không chỉ ghi trong tài liệu.

## Quyết định

Workspace chia theo ranh giới trách nhiệm:

| Crate | Trách nhiệm | Được phụ thuộc (nội bộ) | Loại |
|---|---|---|---|
| `kernel` | Hợp đồng chung: trait `Kind`, dấu vết, phát lại, `Outcome`, mã định danh, bộ kiểm hợp đồng (ADR 0010) | — | Thuần |
| `lib-*` (`crates/libs/<tên>`) | Thư viện thuần dùng chung, không biết `Kind`/dấu vết/sư phạm (ADR 0012); hiện có `lib-exact` | `lib-*` | Thuần |
| `kind-*` (`crates/kinds/<tên>`) | Một loại hoạt động; hiện có `kind-bars`, `kind-choice`, `kind-section` | `kernel`, `lib-*` (không phụ thuộc loại khác) | Thuần |
| `registry` | Danh sách đóng các loại hoạt động; chỗ duy nhất rẽ nhánh theo loại | `kernel`, `kind-*` | Thuần |
| `content` | Đọc và kiểm tra file nội dung | `kernel`, `registry` | Có IO |
| `cli` | Công cụ dòng lệnh `dyen` (phát lại, kiểm tra nội dung) | `kernel`, `registry`, `content` | Có IO |
| `web` | Vỏ ứng dụng trên trình duyệt (Leptos CSR, ADR 0008): chọn hoạt động, bộ hiển thị 2D của từng loại, kết quả | `kernel`, `lib-*`, `kind-*`, `registry`, `content` | Có IO (trình duyệt) |
| `xtask` | Công cụ dự án (CI cục bộ, kiểm tra ranh giới) | — | Công cụ |

Sẽ thêm sau (chưa tạo): `table3d` (Bevy), `server` (Axum).

`web` được phụ thuộc trực tiếp `kind-*` vì bộ hiển thị 2D cần kiểu `State`/`Input` của từng loại (ADR 0010). Không crate nào được phụ thuộc `web`.

**Crate thuần** (`kernel`, `lib-*`, `kind-*`, `registry`) không được phụ thuộc thư viện bên ngoài nào ngoài danh sách cho phép trong `xtask/src/deps.rs` (hiện tại: `serde`, `thiserror` và phụ thuộc bắc cầu của chúng). Không IO, không mạng, không đồng hồ, không thread.

`cargo xtask deps` kiểm tra bảng trên bằng `cargo metadata`; chạy trong `cargo xtask ci`.

## Hệ quả

- Thêm phụ thuộc vào crate thuần là một quyết định có chủ đích: sửa danh sách cho phép trong `xtask` và ghi lý do trong commit.
- Crate thuần biên dịch được sang wasm mà không cần cấu hình đặc biệt.
- Khi thêm `web`/`table3d`/`server`, cập nhật bảng ở đây và trong `xtask` cùng một lúc.
- Crate loại hoạt động mới tên `kind-<tên>` tự khớp quy tắc `kind-*`; chỉ cần thêm vào `registry`.
- Crate `lib-<tên>` tự khớp quy tắc `lib-*` về mặt máy, nhưng mỗi crate mới phải được ghi tên trong một ADR (ADR 0012).
