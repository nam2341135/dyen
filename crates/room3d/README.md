# room3d

Cảnh phòng học 3D (ADR 0016), gói wasm riêng nói chuyện với `web` qua thông điệp của `room-model`.
Phần logic thuần (`sky`, `flight`, `poses`, `ctl`, `fps`) chạy và test được trên máy thường;
phần vẽ Bevy (`app`, chỉ wasm) do Task 6 thêm. Số thực `f32` ở đây chỉ để hiển thị (ADR 0002).

## Thuật ngữ riêng của crate

- **Pose**: tư thế camera, gồm mắt và điểm nhìn (mét).
- **Flight**: chuyến bay camera 0,7 s giữa hai pose, đổi đích giữa chừng vẫn liền mạch.
- **Ctl**: bộ điều khiển; nhận thông điệp/chạm/đồng hồ, trả pose mỗi khung và thông điệp về vỏ.
- **Collider**: hộp va chạm của vật, to hơn mô hình để trẻ bấm trúng; hình chiếu lối vào >= 44 px.
- **Sky**: màu/độ sáng trời theo giờ; bốn khoảng sáng, trưa, chiều tà, đêm.
- **FpsProbe**: đo fps 3 s đầu; trung bình dưới 30 thì đề nghị hạ chất lượng.

## Feature Bevy (wasm)

Bộ feature của `spikes/leptos-bevy/view3d`, cộng `bevy_gltf`, `bevy_animation`, `bevy_scene`.
Bloom nằm ở feature **`bevy_post_process`** (bevy 0.20.1 không có feature riêng tên "bloom").
