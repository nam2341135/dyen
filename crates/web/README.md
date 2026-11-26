# web

Vỏ trình duyệt tối thiểu (Leptos CSR, ADR 0008): danh mục hoạt động → chơi một hoạt động → xem kết quả. Chỉ vẽ `State` và phát `Input` (ADR 0002); mọi quyết định do các `kind-*` làm qua `kernel::Session`. Không lưu gì, không gọi mạng, không tài nguyên ngoài (ADR 0005).

## Thuật ngữ riêng

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| (vỏ của phiên) | `Play<K>` | Bọc một Phiên (`Session`) của loại `K`: gắn mốc thời gian từ đồng hồ trang, nhớ lỗi đầu vào gần nhất; giữ trong signal cục bộ (`RwSignal::new_local`). Không phải thuật ngữ mới — chỉ là chỗ vỏ giữ phiên. |
| Bộ hiển thị | `kinds::{bars,choice,section}::view` | Vẽ trạng thái của một loại; danh sách đóng, khớp `AnyConfig` ở `activity.rs`. |
| Chân trang | `kinds::footer` | Lỗi đầu vào cuối, kết quả khi đã xong, nút "Làm lại". |

| Chuyển cảnh | `transition::Transition` | Máy trạng thái thuần của lớp phủ giữa phòng và khu (bản thiết kế phòng học 3D, mục 6.7): nhận `Ev`, trả `Effect` (đặt lớp phủ, đi tới màn, gửi phòng, hẹn giờ). Hẹn giờ mang `generation()`; đổi đích hoặc Back thì hẹn giờ cũ bị bỏ. |
| Cầu nối phòng | `room` | `send` gọi JS `dyenToRoom`; `room3d` gọi `window.dyenFromRoom` → `dyen_from_room`. Lựa chọn "Phòng đơn giản" và chất lượng nằm trong `localStorage` (`dyen.simpleRoom`, `dyen.quality`). |
| Màn phòng | `shell::Shell` | Nối hai thứ trên vào Leptos: nền phòng (ảnh chờ, canvas `#room3d`), danh sách khu, menu "⋯", lớp phủ. Không có `room3d` (hoặc lỗi, không WebGL2) thì danh sách chữ là lối vào, bấm chỉ còn lớp phủ mờ. |

Nội dung `content/` được nhúng lúc biên dịch (`build.rs`) và dựng thành `content::Library` một lần (`library::get`).

Thêm loại hoạt động mới vào vỏ: thêm `src/kinds/<tên>.rs` có `pub fn view(Config) -> AnyView` và một nhánh trong `activity::renderer`.

## Chạy

Cần target `wasm32-unknown-unknown`, `wasm-bindgen-cli` đúng phiên bản `wasm-bindgen` trong `Cargo.lock`, và (bản đo) `wasm-opt`, `brotli`.

```
crates/web/build.sh                 # bản đo: vỏ (wasm-release, -Oz) + phòng 3D (room-release, -O3), brotli
crates/web/build.sh dev             # bản lặp nhanh, cả vỏ và phòng
crates/web/build.sh dev --no-room   # chỉ build vỏ; giữ dist/room/ cũ nếu có (đối số theo thứ tự bất kỳ)
python -m http.server -d crates/web/dist 8091
```

- Phòng 3D (`crates/room3d/build.sh`, được gọi từ đây) xoá và tạo lại `dist/room/`. Vì vậy ảnh nền chờ `room-still.webp` được chép sang `dist/room/still.webp` **sau** bước build phòng, kể cả khi có `--no-room`.
- Bản đo của phòng chạy lâu (nhiều phút, LTO trên cây Bevy).
- Windows: nếu `rustc` chết với `STATUS_STACK_BUFFER_OVERRUN` khi build song song, đặt `CARGO_BUILD_JOBS=4`. Lỗi do môi trường, không phải do code.

### Ảnh nền chờ

`room-still.webp` (10 386 B; ngân sách 60 KB): phòng ở 1280 × 720, 17 giờ (chiều tà), góc nhà. Hiện ngay khi mở trang, trước khi 3D sẵn sàng, và là nền của "Phòng đơn giản". Lấy lại khi cảnh đổi nhiều:

1. mở phòng ở 1280 × 720;
2. gọi `dyenToRoom('{"t":"SetHour","v":17}')`;
3. trong một callback `requestAnimationFrame`, vẽ canvas `#room3d` lên một canvas 2D 1280 × 720, rồi gọi `toDataURL('image/webp', 0.6)` ngay trong callback đó. Canvas WebGL không bật `preserveDrawingBuffer`, nên gọi `toDataURL` thẳng trên `#room3d` ra ảnh trống.

Test đơn vị chạy trên máy chủ như mọi crate: `cargo test -p web`.

## Bản trên mạng

<https://nam2341135.github.io/dyen/app/> — workflow `.github/workflows/pages.yml` build lại (cả phòng 3D) sau mỗi lần đẩy lên `main` và chỉ thay thư mục `app/` của nhánh `gh-pages`; `dist/room/` (trừ `*.br`) vào `app/room/`. GitHub Pages chỉ nén gzip (không brotli), nên lượng tải thật lớn hơn số brotli dưới đây.

## Kích thước đo được (2026-11-26)

| | Byte |
|---|---|
| `web_bg.wasm` sau LTO, `opt-level = "z"` | 3 173 270 |
| sau `wasm-opt -Oz` | 582 502 |
| brotli q11 | 188 148 |
| `web.js` brotli | 5 528 |

## Phòng 3D: kích thước và khung đầu (2026-11-27)

Dòng in của `crates/web/build.sh` (bản đo):

| | Byte |
|---|---|
| vỏ `web_bg.wasm` sau `-Oz` / brotli | 735 213 / 232 040 |
| `room3d_bg.wasm` thô / sau `wasm-opt -O3` / brotli (ngân sách 5 MB) | 60 195 667 / 30 522 796 / 4 566 510 |
| `room3d.js` brotli | 12 793 |
| mô hình `.glb` cộng lại (ngân sách 3 MB) | 179 228 |
| ảnh nền chờ `still.webp` (ngân sách 60 KB) | 10 386 |

Khung 3D đầu (`FirstFrame`, dòng `room3d: khung đầu ở … ms` trong console), đo trên Chrome thường của máy tính phát triển (Windows, DPR 1,25, cửa sổ 1536 × 742), phục vụ bằng `python -m http.server` trên localhost (không nén):

- lần đầu, chưa có cache mã wasm: **10 991 ms** từ lúc mở trang. Tải `room3d_bg.wasm` xong ở 583 ms; gần như toàn bộ phần còn lại là biên dịch 30 MB wasm. Bevy chạy tới khung đầu mất 927 ms;
- nạp lại, có cache mã đã biên dịch: **942 ms**.

Lần đầu **vượt ngân sách 3 s** (mục 6.8 bản thiết kế); lần sau đạt. Chưa tối ưu ở mốc 1. Pane xem thử trong ứng dụng Claude chạy chậm và bị bóp nhịp khi ẩn (đo được 15–40 s), không dùng số đó.
