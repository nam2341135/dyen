# ADR 0016: Trang chủ là phòng học 3D

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-27

## Bối cảnh

Trang chủ của vỏ web hiện là danh mục dạng thẻ. Chủ dự án muốn trang chủ là một căn phòng học 3D kiểu lofi: chạm vào quyển vở, máy tính, cúp thì vào khu ôn tập, khóa học, thi đấu. Bản thiết kế: `docs/superpowers/specs/2026-11-27-phong-hoc-3d-design.md`.

## Quyết định

- Trang chủ là phòng 3D. Phòng **chỉ là điều hướng** (ADR 0002): không chức năng học nào chỉ có ở 3D; luôn có lối thay thế bằng danh sách chữ.
- "Phòng", "khu" (`Zone`) và "chỗ đặt đồ" (`Slot`) là khái niệm **của vỏ** (ADR 0010), không đi vào `kernel`, `registry`, `content`.
- Bảng `ROOM` trong `room-model` là **nguồn duy nhất** cho "vật nào dẫn vào khu nào"; `web` (danh sách chữ, định tuyến) và `room3d` (đặt mô hình, chọn vật) cùng đọc nó.
- Hai crate mới (cập nhật ADR 0006 và `xtask/src/deps.rs`):
  - `room-model`: thuần, chỉ phụ thuộc `serde`; chứa bảng, kiểu và thông điệp cầu nối `ToRoom`/`FromRoom`.
  - `room3d`: Bevy, gói wasm riêng nạp lười; chỉ phụ thuộc `room-model`. `web` không phụ thuộc `room3d`, hai bên chỉ nói chuyện qua cầu nối.
- Các cách đã cân nhắc:
  - **Cách 1 (chọn): Bevy riêng + Leptos điều hướng.** Phòng là gói wasm Bevy nạp lười, báo "mở khu X" cho vỏ Leptos. Trang trí, kéo thả, trời theo giờ đều làm được; đã có spike chứng minh. Đổi lại: thêm 3–4 MB tải và một crate mới.
  - **Cách 2 (loại): ảnh dựng sẵn + điểm bấm.** Dựng cảnh trong Blender, Leptos vẽ vùng bấm đè lên. Rất nhẹ, nhưng mỗi màu tường, mỗi giờ trời, mỗi vị trí đồ là một lần dựng lại; không hợp trang trí, kéo thả, giờ thật.
  - **Cách 3 (loại): toàn bộ bằng Bevy.** Bỏ Leptos, giao diện bằng `bevy_ui`. Chuyển cảnh liền mạch nhất, nhưng ngược ADR 0008, chữ khó đọc, khó dùng bàn phím, phải viết lại mọi trang bài.

## Hệ quả

- Được: trang chủ sống động mà lõi vẫn trung lập; một bảng duy nhất cho phòng và danh sách chữ.
- Mất: thêm gói Bevy nạp lười (ngưỡng theo ADR 0008: ≤ 5 MB brotli), thêm hai crate và hai gói wasm phải giữ đồng bộ qua cầu nối.
- Xem lại nếu gói Bevy vượt ngưỡng, hoặc nếu khung 3D đầu quá 10 s trên điện thoại tầm trung.
