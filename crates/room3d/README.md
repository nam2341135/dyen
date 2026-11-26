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
- **Vai trò màu** (`palette::role_for`): tên một màu trong bảng màu (`wood_dark`, `paper`…), suy
  từ tên vật liệu của mô hình; tên lạ thành `wall`.
- **Restyle**: bước đồng bộ phong cách khi một mô hình `.glb` nạp xong: thay mọi vật liệu bằng
  màu phẳng theo vai trò, tắt picking của mesh, bỏ dấu `Pending`.
- **Pending**: dấu trên mô hình mà `FirstFrame` phải chờ (bàn, máy tính).
- **Glow**: danh sách vật liệu riêng của một vật; làm sáng khi chọn đổi `emissive` của cả danh sách.
- **Kit** (`app/props.rs`): bộ dựng vật bằng hình khối Bevy với màu bảng màu, cho vật không có
  mô hình CC0.
- **PetClock**: nhịp cử chỉ của mèo, khoảng chờ 8–15 s từ bộ sinh xorshift32 seed cố định.
- **pixfont**: phông điểm ảnh 5×12 tự vẽ (có dấu tiếng Việt) để in chữ lên giấy ghi chú.

## Feature Bevy (wasm)

Bộ feature của `spikes/leptos-bevy/view3d`, cộng `bevy_gltf`, `bevy_animation`, `bevy_scene`.

Thêm (Task 7):

- `gltf_animation`: nối `bevy_gltf` với `bevy_animation`; thiếu nó, `cat.glb` không có nhãn
  `Animation<i>`;
- `reflect_auto_register`: `bevy_world_serialization` chép cảnh glTF vào thế giới bằng reflect;
  thiếu nó, kiểu thành phần của cảnh glTF chưa đăng ký và phòng panic khi sinh mô hình.
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
- Đầu ra: `room3d.js`, `room3d_bg.wasm` và mọi `assets/*.glb`, `assets/*.png`. Bevy đọc tài nguyên từ `room/`
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
- `FirstFrame`: khi đã có `Init`, mọi mô hình mang `Pending` (bàn, máy tính) đã nạp xong, và 2
  khung đã vẽ sau đó. Tường, cửa sổ, vở và cúp dựng bằng hình khối ở `Startup`; đồ trang trí
  (kệ sách, thảm, đèn, cây, sách, mèo) hiện dần sau (bản thiết kế, quyết định 10). Một mô hình
  nạp hỏng: gửi `Failed` một lần. Console in thời điểm khung đầu.
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

Màu phẳng sRGB, ấm, tương phản thấp (`palette.rs`, bảng `PALETTE`). Mọi vật, kể cả mô hình CC0,
chỉ dùng các màu này: `Restyle` suy vai trò từ tên vật liệu (`RULES`, khớp chuỗi con theo thứ
tự; tên lạ thành `wall`).

| Vai trò | Màu | Dùng cho |
|---|---|---|
| `wood_dark` | `#8A5A3C` | bàn, kệ sách, khung bảng ghi chú, kệ huy chương |
| `wood_light` | `#C99A6E` | sàn, bậu cửa sổ |
| `wall` | `#E8D8C4` | tường; vật liệu không rõ tên |
| `paper` | `#F4EEDC` | giấy, khung cửa sổ, chao đèn, hơi nước |
| `gold` | `#E3B23C` | cúp, bút chì |
| `screen` | `#6F8FC4` | kính, ống bút |
| `leaf` | `#6A994E` | cây |
| `curtain` | `#D9825F` | rèm, thảm, bìa vở |
| `fur` | `#E3A15C` | lông mèo |
| `fur_dark` | `#B06D3B` | vằn, tai mèo |
| `ceramic` | `#81B29A` | cốc, chậu |
| `metal` | `#A39A90` | móc kệ huy chương |
| `ink` | `#3B3330` | chữ, mắt mèo, huy chương "chưa có" (mờ 0,35) |

Ánh sáng (`app/scene.rs`, mô-đun `light`): đèn bàn `#FFD9A0`, ánh sáng chung `#FFE8D0` (600),
nền sau phòng `#2B221C`.

## Mô hình

`ObjectKind` → mô hình (`app/models.rs`). Nguồn và giấy phép: [`assets/NGUON.md`](assets/NGUON.md).

| Vật | Mô hình |
|---|---|
| `Laptop` | `laptop.glb` |
| `DeskLamp` | `lampSquareTable.glb` (chao tự sáng, đèn điểm bên trong) |
| `Plant` | `plantSmall2.glb` |
| `Books` | `books.glb` |
| `Notebook` | dựng bằng code (`props.rs`) |
| `Trophy` | dựng bằng code |
| `PencilCup` | dựng bằng code |
| `Mug` | dựng bằng code (có hơi nước) |
| `NoteBoard` | dựng bằng code: 3 tờ ghi chú in sẵn chữ (`pixfont`), đinh ghim |
| `MedalShelf` | dựng bằng code: 4 móc, 3 huy chương "chưa có" mờ, vé "Ngày đầu tới lớp" |
| bàn học | `desk.glb` (phóng không đều thành 1,8 × 0,7 × 0,75 m) |
| kệ sách | `bookcaseOpen.glb` + `books.glb` |
| thảm | `rugRectangle.glb` |
| mèo | `cat.glb` (clip `Idle`, `Yes`) |

Gói Kenney không có vở, cúp, cốc, ống bút, bảng ghi chú, kệ huy chương; chủ dự án đồng ý dựng
chúng bằng hình khối.

Đổi một mô hình:

1. chép file `.glb` (CC0) vào `assets/`, thêm một dòng vào `assets/NGUON.md`;
2. sửa `model_for` (hoặc hằng `DESK`, `BOOKCASE`…): `center` là tâm đáy trong toạ độ file (mô
   hình Kenney có gốc ở một góc), `scale`, `yaw_deg` để mặt trước hướng +Z;
3. nếu tên vật liệu mới không khớp `RULES`, thêm luật (và test `moi_vai_tro_co_trong_bang_mau`);
4. khối va chạm (`collider`) không đổi theo mô hình; kiểm lại ở 360 px.

## Không khí

- Mèo trên bậu cửa sổ: `Idle` lặp, cứ 8–15 s (`PetClock`, seed cố định) gật đầu (`Yes`). Gói
  không có clip vẫy đuôi.
- Hơi nước trên cốc (3 đốm mờ, 3 s), rèm lay (chu kỳ 5 s), sao và đèn thành phố sau cửa sổ ban
  đêm. Đổi giờ: trời, nắng, đèn bàn chuyển dần trong 2 s.
- Giảm chuyển động: mèo đứng yên ở tư thế đầu của `Idle`, hơi nước ẩn, rèm đứng yên.

Trời (`sky.rs`, RGB tuyến tính) và độ sáng: nắng 1500 / 2500 / 900 / 60 lux và đèn bàn
500 / 300 / 900 / 1600 lumen cho sáng / trưa / chiều tà / đêm. Đổi giờ thì chuyển dần trong 2 s.

## Ngân sách

- `room3d_bg.wasm` ≤ 5 MB brotli (ADR 0008);
- mô hình + kết cấu ≤ 3 MB sau nén;
- khung 3D đầu ≤ 3 s trên máy tính.

Đo Task 7 (bản release, máy Iris Xe, ANGLE D3D11, pane trình duyệt của công cụ):

| Mục | Đo | Ngân sách |
|---|---|---|
| `room3d_bg.wasm.br` | 4 568 903 B (Task 6: 4 506 122 B) | ≤ 5 MB ✓ |
| mô hình `.glb` (8 file, chưa nén) | 179 228 B | ≤ 3 MB ✓ |
| khung đầu | 15,7 s từ lúc mở trang, 0,5 s từ lúc Bevy chạy | ≤ 3 s ✗ |

Khung đầu đo trong pane bị ẩn (trang chạy như trang nền); một lần khác chờ tới 39 s, nên số này
không tin được. Tải wasm xong ở 0,1 s; phần còn lại là trước khi Bevy chạy. Cần đo lại ở trình
duyệt thường.
