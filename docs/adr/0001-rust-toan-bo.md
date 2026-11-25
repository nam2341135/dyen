# ADR 0001: Toàn bộ mã ứng dụng viết bằng Rust

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Đội nhỏ, phát triển chủ yếu bằng vibecode. Logic mô hình và chẩn đoán phải chạy giống hệt ở trình duyệt (phản hồi tức thì) và ở server (chấm có thẩm quyền).

## Quyết định

Mọi mã do đội duy trì — giao diện, backend, logic tương tác, điều khiển đồ họa, công cụ dự án — viết bằng Rust. Được phép: CSS, shader (WGSL), mô hình 3D, hình ảnh, file nội dung (RON/TOML), và phần kết nối JS do công cụ build tự sinh (wasm-bindgen, Trunk…).

Công cụ dự án (kiểm tra, CI cục bộ) cũng viết bằng Rust qua crate `xtask` — xem [ADR 0007](0007-cong-cu-du-an.md).

## Hệ quả

- Một crate logic dùng chung cho client và server; compiler kiểm tra ranh giới thay cho kỷ luật thủ công.
- Hệ sinh thái UI/3D trên web của Rust non hơn JS: API đổi giữa các phiên bản, ít ví dụ hơn. Phải ghim phiên bản và cho AI đọc đúng tài liệu phiên bản đó.
- Kích thước wasm là rủi ro thật, phải đo bằng spike trước khi chốt Bevy ([ADR 0008](0008-leptos-bevy-de-xuat.md)).
- Xem lại nếu: không đạt ngân sách kích thước/hiệu năng trên điện thoại thật sau khi đã tối ưu.
