# room3d

Cảnh phòng học 3D (ADR 0016), gói wasm riêng nói chuyện với `web` qua thông điệp của `room-model`.
Phần logic thuần (`sky`, `flight`, `poses`, `ctl`, `fps`) chạy và test được trên máy thường;
phần vẽ Bevy (`app`, chỉ wasm) dựng cảnh, picking và bay camera. Số thực `f32` ở đây chỉ để hiển thị (ADR 0002).

## Thuật ngữ riêng của crate

- **Pose**: tư thế camera, gồm mắt và điểm nhìn (mét).
- **Flight**: chuyến bay camera 0,7 s giữa hai pose, đổi đích giữa chừng vẫn liền mạch.
- **Ctl**: bộ điều khiển; nhận thông điệp/chạm/đồng hồ, trả pose mỗi khung và thông điệp về vỏ.
- **Collider**: hộp va chạm của vật, to hơn mô hình để trẻ bấm trúng; hình chiếu lối vào >= 44 px.
- **Sky**: màu/độ sáng trời theo giờ; bốn khoảng sáng, trưa, chiều tà, đêm.
- **FpsProbe**: đo fps 3 s đầu; trung bình dưới 30 thì đề nghị hạ chất lượng.
- **Inbox**: hàng đợi thông điệp vỏ → phòng; `push` xếp vào, mỗi khung lấy hết theo thứ tự.
- **SlotTag / SlotVisual**: khối va chạm vô hình / vật nhìn thấy của một chỗ trong `ROOM`.

## Feature Bevy (wasm)

Bộ feature của `spikes/leptos-bevy/view3d`, cộng `bevy_gltf`, `bevy_animation`, `bevy_scene`.
Bloom nằm ở feature **`bevy_post_process`** (bevy 0.20.1 không có feature riêng tên "bloom").

Thêm (Task 6):

- không có feature con trỏ riêng: con trỏ hệ thống (`CursorIcon::System`) chạy sẵn,
  `custom_cursor` chỉ cần cho ảnh con trỏ tự vẽ;
- không bật `tonemapping_luts`: `Tonemapping::AcesFitted` tính bằng công thức, không đọc LUT
  (bevy_core_pipeline 0.20.1 chỉ báo lỗi thiếu LUT với AgX, TonyMcMapface, BlenderFilmic).

Phụ thuộc wasm thêm `web-sys` (workspace) để đọc `performance.now()` (đo khung đầu) và
bỏ `tabindex` winit gắn vào canvas.

## Build

```
crates/web/build.sh dev            # vỏ + phòng, bản lặp nhanh (profile dev / room-dev)
crates/web/build.sh                # vỏ + phòng, bản đo (wasm-release / room-release)
crates/web/build.sh dev --no-room  # chỉ vỏ; giữ dist/room/ cũ nếu có
crates/room3d/build.sh [dev]       # chỉ phòng, ra crates/web/dist/room/
```

- `room-release` (Cargo.toml gốc): `opt-level = 3`, LTO fat, 1 codegen unit, `panic = "abort"`;
  sau đó `wasm-opt -O3` và `brotli -q 11`. Không dùng `opt-level = "z"`: spike `lang-map` đo
  khung đầu chậm 16 lần.
- `room-dev`: như `dev`, nhưng mọi thư viện (Bevy) ở `opt-level = 3` để cảnh chạy mượt.
- Đầu ra: `room3d.js`, `room3d_bg.wasm` và mọi `assets/*.glb`. Bevy đọc tài nguyên từ `room/`
  (`AssetPlugin.file_path`), không có file `.meta` (`AssetMetaCheck::Never`).

Test native (logic, không cần trình duyệt): `cargo test -p room3d`. Phần `app` chỉ biên dịch
cho wasm; `cargo xtask ci` chạy clippy cho nó (`--target wasm32-unknown-unknown`).

## Thông điệp

JSON của `room_model::{ToRoom, FromRoom}`, dạng `{ "t": tên, "v": giá trị }`.

- Vỏ → phòng: `push(json)` (xuất bằng `wasm_bindgen`). Thông điệp xếp hàng, không ghi đè;
  khung kế tiếp xử lý theo thứ tự. JSON hỏng: gửi `Failed` rồi chạy tiếp.
- Phòng → vỏ: gọi `window.dyenFromRoom(json)`.
- `start()` khởi động Bevy trên `<canvas id="room3d">` (`fit_canvas_to_parent`). Trên web,
  winit thoát `start` bằng ngoại lệ "exceptions for control flow"; vỏ bỏ qua đúng ngoại lệ đó.
- `FirstFrame`: khi đã có `Init` và 2 khung đã vẽ sau đó. Tường, bàn, cửa sổ và 3 lối vào dựng
  ở `Startup`, trước khung đầu (bản thiết kế, quyết định 10). Console in thời điểm khung đầu.
- Phòng panic: hook gửi `Failed` (vỏ ẩn canvas, hiện danh sách chữ).
- `Quality(q)`: chỉ khi phòng tự đổi mức (`FpsProbe`); vỏ cập nhật nút chất lượng và lớp
  `thap`. `SetQuality` của vỏ không được gửi lại.
- Bàn phím: phòng không nhận phím. `prevent_default_event_handling = false`, cửa sổ không tự
  lấy focus, và `tabindex` winit gắn vào canvas bị bỏ, để Tab/Enter vẫn tới danh sách khu.

## Tạm dừng

Khi `Ctl::suspended()` (vỏ gửi `Suspend` lúc rời phòng):

- `WinitSettings` chuyển sang `UpdateMode::reactive_low_power(3600 s)`: vòng lặp ngủ, chỉ chạy
  khi có sự kiện cửa sổ;
- camera tắt (`Camera::is_active = false`), nên lỡ có khung chạy thì GPU cũng không vẽ cảnh;
- `push` đánh thức vòng lặp (`EventLoopProxy`, `WinitUserEvent::WakeUp`), nên `Resume` có tác
  dụng ngay.

`FpsProbe` chỉ đo sau `FirstFrame`, không đo lúc tạm dừng, và bỏ khung đầu sau khi tiếp tục
(khung đó có `dt` bằng cả thời gian ngủ).

## Chất lượng

| Mức | Bật |
|---|---|
| `High` | bloom nhẹ (`Bloom::NATURAL`, cường độ 0,08; cần HDR), MSAA 4×, bóng cho đèn bàn |
| `Low` | không bloom, không HDR, MSAA tắt, không bóng. Độ phân giải 0,75×: vỏ gắn lớp `thap` cho canvas (rộng, cao 75%, `transform: scale(1.3333)` từ góc trên trái); winit đặt bộ đệm = cỡ CSS × devicePixelRatio nên bộ đệm nhỏ theo. `scale_factor_override` không dùng được: trên web nó không đổi bộ đệm |

Vỏ chọn mức trong `Init`; `FpsProbe` hạ xuống `Low` một lần nếu 3 s đầu dưới 30 fps và gửi `FromRoom::Quality(Low)`.

## Bảng màu của phòng

Màu phẳng sRGB, ấm, tương phản thấp (`app/scene.rs`, mô-đun `palette`). Mô hình CC0 (Task 7) đổi
vật liệu về các màu này.

| Tên | Màu | Dùng cho |
|---|---|---|
| `WALL` | `#E8D8C4` | tường sau |
| `WALL_SIDE` | `#D9C6B0` | tường trái |
| `FLOOR` | `#8A6A52` | sàn gỗ |
| `DESK` | `#B0805A` | bàn |
| `FRAME` | `#F7F3EA` | khung cửa sổ |
| `NOTEBOOK` | `#E07A5F` | vở |
| `LAPTOP` | `#5B7DB1` | máy tính |
| `TROPHY` | `#E3B23C` | cúp |
| `LAMP` | `#F2E3C6` | đèn bàn |
| `PLANT` | `#6A994E` | cây |
| `PENCIL_CUP` | `#81B29A` | ống bút |
| `MUG` | `#F4F1DE` | cốc |
| `BOOKS` | `#9C6644` | chồng sách |
| `CORK` | `#C19A6B` | bảng ghi chú (bần) |
| `SHELF` | `#7F5539` | kệ huy chương |
| `BULB` | `#FFD9A0` | ánh đèn bàn |
| `AMBIENT` | `#FFE8D0` | ánh sáng chung |
| `VOID` | `#2B221C` | nền sau phòng |

Trời (`sky.rs`, RGB tuyến tính) và độ sáng: nắng 1500 / 2500 / 900 / 60 lux và đèn bàn
500 / 300 / 900 / 1600 lumen cho sáng / trưa / chiều tà / đêm. Đổi giờ thì chuyển dần trong 2 s.

## Ngân sách

- `room3d_bg.wasm` ≤ 5 MB brotli (ADR 0008);
- mô hình + kết cấu ≤ 3 MB sau nén;
- khung 3D đầu ≤ 3 s trên máy tính.
