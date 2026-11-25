# Tài liệu dự án

Đọc theo thứ tự này:

1. [vision.md](vision.md) — sản phẩm là gì, nguyên tắc không thương lượng.
2. [glossary.md](glossary.md) — bộ từ cố định.
3. [adr/](adr/) — các quyết định kiến trúc.

## Quyết định kiến trúc

| ADR | Tiêu đề | Trạng thái |
|---|---|---|
| [0001](adr/0001-rust-toan-bo.md) | Toàn bộ mã ứng dụng viết bằng Rust | Chấp nhận |
| [0002](adr/0002-mot-mo-hinh-nhieu-bo-hien-thi.md) | Một mô hình, nhiều bộ hiển thị | Chấp nhận |
| [0003](adr/0003-dau-vet-la-nguon-su-that.md) | Dấu vết là nguồn sự thật | Chấp nhận |
| [0004](adr/0004-noi-dung-la-file.md) | Nội dung là file trong git | Chấp nhận |
| [0005](adr/0005-du-lieu-tre-em.md) | Nguyên tắc dữ liệu trẻ em | Chấp nhận (pháp lý chưa xác minh) |
| [0006](adr/0006-ranh-gioi-crate.md) | Ranh giới crate và hướng phụ thuộc | Chấp nhận |
| [0007](adr/0007-cong-cu-du-an.md) | Công cụ dự án bằng `cargo xtask` | Chấp nhận |
| [0008](adr/0008-leptos-bevy-de-xuat.md) | Leptos, Bevy, Axum | **Đề xuất — chờ spike** |
| [0009](adr/0009-hop-dong-loi.md) | Hợp đồng lõi: mô hình, dấu vết, chấm, nội dung | Chấp nhận |

Thêm ADR mới: chép [0000-mau.md](adr/0000-mau.md), đánh số tiếp theo, cập nhật bảng này.

## Lộ trình nền móng

| Tầng | Nội dung | Trạng thái |
|---|---|---|
| 0 | Tài liệu quyết định (thư mục này, `CLAUDE.md`) | Xong |
| 1 | Bộ khung workspace, `cargo xtask ci`, kiểm tra ranh giới | Xong |
| 2 | Hợp đồng lõi: trait mô hình, định dạng dấu vết có phiên bản, chấm/chẩn đoán, lược đồ nội dung; một mô hình thật (thanh cắt + thước) | Xong |
| 3 | `dyen replay`, dấu vết mẫu + snapshot test, property test, kiểm tra nội dung trong CI | Xong |
| — | Spike Leptos + Bevy (chốt ADR 0008) | Chưa |
