# Tài liệu dự án

Muốn biết đang làm gì và làm gì tiếp: [STATUS.md](STATUS.md). Muốn biết làm một việc thế nào: [playbooks/](playbooks/README.md).

Muốn hiểu dự án, đọc theo thứ tự này:

1. [vision.md](vision.md): ràng buộc và nguyên tắc kiến trúc.
2. [glossary.md](glossary.md): bộ từ cố định.
3. [adr/](adr/): các quyết định kiến trúc (bảng dưới).
4. [ideas/](ideas/README.md): phác thảo ý tưởng. Đây là nháp để thử lõi, không phải quyết định.

## Quyết định kiến trúc

| ADR | Tiêu đề | Trạng thái |
|---|---|---|
| [0001](adr/0001-rust-toan-bo.md) | Toàn bộ mã ứng dụng viết bằng Rust | Chấp nhận |
| [0002](adr/0002-mot-mo-hinh-nhieu-bo-hien-thi.md) | Logic tách khỏi hiển thị | Chấp nhận |
| [0003](adr/0003-dau-vet-la-nguon-su-that.md) | Dấu vết là nguồn sự thật | Chấp nhận |
| [0004](adr/0004-noi-dung-la-file.md) | Nội dung là file trong git | Chấp nhận |
| [0005](adr/0005-du-lieu-tre-em.md) | Nguyên tắc dữ liệu trẻ em | Chấp nhận (pháp lý chưa xác minh) |
| [0006](adr/0006-ranh-gioi-crate.md) | Ranh giới crate và hướng phụ thuộc | Chấp nhận |
| [0007](adr/0007-cong-cu-du-an.md) | Công cụ dự án bằng `cargo xtask` | Chấp nhận |
| [0008](adr/0008-leptos-bevy-de-xuat.md) | Leptos, Bevy, Axum | Chấp nhận (Leptos, Bevy); Axum còn đề xuất |
| [0009](adr/0009-hop-dong-loi.md) | Hợp đồng lõi: mô hình, dấu vết, chấm, nội dung | Bị thay thế bởi 0010 |
| [0010](adr/0010-loi-trung-lap.md) | Lõi trung lập: loại hoạt động là crate, sổ đăng ký là enum đóng | Chấp nhận |
| [0011](adr/0011-nhan-tu-cau-hinh.md) | Loại hoạt động công bố thêm nhãn từ cấu hình | Chấp nhận |
| [0012](adr/0012-toan-chinh-xac.md) | Tầng thư viện thuần `lib-*` và toán chính xác `lib-exact` | Chấp nhận |
| [0013](adr/0013-quy-uoc-bo-hien-thi.md) | Quy ước chung cho bộ hiển thị: vùng chạm, animation, kiểu dáng | Chấp nhận |
| [0014](adr/0014-luu-bai-lam-cuc-bo.md) | Lưu bài làm cục bộ, ẩn danh, trên máy người học | Đề xuất |
| [0015](adr/0015-ngon-ngu-thiet-ke-giay-muc.md) | Ngôn ngữ thiết kế "giấy và mực", nút nổi | Đề xuất |

Thêm ADR mới: chép [0000-mau.md](adr/0000-mau.md), đánh số tiếp theo, cập nhật bảng này.

## Lộ trình

Tiến độ, việc tiếp theo và các mốc đã xong nằm ở [STATUS.md](STATUS.md). Không ghi ở đây, để tránh hai nơi lệch nhau.
