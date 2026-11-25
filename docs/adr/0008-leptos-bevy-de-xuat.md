# ADR 0008: Leptos cho giao diện, Bevy cho 3D, Axum cho server

- **Trạng thái:** **Đề xuất — chờ kết quả spike**
- **Ngày:** 2026-11-26

## Bối cảnh

Cần chọn thư viện UI, 3D và server. Khả năng chạy trên trình duyệt điện thoại và kích thước gói phụ thuộc phiên bản; chưa đo trên máy thật.

## Đề xuất

- **Leptos** (CSR trước) cho vỏ ứng dụng và bộ hiển thị 2D bằng SVG.
- **Bevy**, build thành gói wasm riêng, nạp lười; backend WebGL2 trước.
- **Axum** + **sqlx** + **PostgreSQL** cho server.

## Điều kiện để chuyển sang "Chấp nhận"

Spike trong `spikes/` phải đo và ghi lại:

1. Kích thước gói Bevy tối thiểu (một cảnh: mặt bàn + một vật thể + ánh sáng + picking) sau `wasm-opt` và nén brotli. **Ngưỡng: ≤ 5 MB nén.**
2. Thời gian từ lúc tải tới khung hình 3D đầu tiên trên một điện thoại Android 3–4 GB RAM qua 4G. **Ngưỡng: ≤ 10 giây.**
3. Leptos và Bevy cùng một trang, trao đổi `State`/`Action` hai chiều, đổi qua lại 2D/3D không mất trạng thái.
4. Phiên bản Leptos, Bevy hiện hành và tình trạng WebGL2/WebGPU trên Safari iOS và Chrome Android — đối chiếu tài liệu chính thức.

## Phương án dự phòng

Nếu (1) hoặc (2) không đạt: thay Bevy bằng renderer nhẹ hơn (`three-d`, hoặc `wgpu` với renderer tối giản tự viết). Nhờ [ADR 0002](0002-mot-mo-hinh-nhieu-bo-hien-thi.md), việc thay này không chạm logic.
