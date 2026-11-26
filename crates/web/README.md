# web

Vỏ trình duyệt tối thiểu (Leptos CSR, ADR 0008): danh mục hoạt động → chơi một hoạt động → xem kết quả. Chỉ vẽ `State` và phát `Input` (ADR 0002); mọi quyết định do các `kind-*` làm qua `kernel::Session`. Không lưu gì, không gọi mạng, không tài nguyên ngoài (ADR 0005).

## Thuật ngữ riêng

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| (vỏ của phiên) | `Play<K>` | Bọc một Phiên (`Session`) của loại `K`: gắn mốc thời gian từ đồng hồ trang, nhớ lỗi đầu vào gần nhất; giữ trong signal cục bộ (`RwSignal::new_local`). Không phải thuật ngữ mới — chỉ là chỗ vỏ giữ phiên. |
| Bộ hiển thị | `kinds::{bars,choice,section}::view` | Vẽ trạng thái của một loại; danh sách đóng, khớp `AnyConfig` ở `activity.rs`. |
| Chân trang | `kinds::footer` | Lỗi đầu vào cuối, kết quả khi đã xong, nút "Làm lại". |

Nội dung `content/` được nhúng lúc biên dịch (`build.rs`) và dựng thành `content::Library` một lần (`library::get`).

Thêm loại hoạt động mới vào vỏ: thêm `src/kinds/<tên>.rs` có `pub fn view(Config) -> AnyView` và một nhánh trong `activity::renderer`.

## Chạy

Cần target `wasm32-unknown-unknown`, `wasm-bindgen-cli` đúng phiên bản `wasm-bindgen` trong `Cargo.lock`, và (bản đo) `wasm-opt`, `brotli`.

```
crates/web/build.sh          # bản đo: profile wasm-release, wasm-opt -Oz, brotli
crates/web/build.sh dev      # bản lặp nhanh
python -m http.server -d crates/web/dist 8091
```

Test đơn vị chạy trên máy chủ như mọi crate: `cargo test -p web`.

## Kích thước đo được (2026-11-26)

| | Byte |
|---|---|
| `web_bg.wasm` sau LTO, `opt-level = "z"` | 3 173 270 |
| sau `wasm-opt -Oz` | 582 502 |
| brotli q11 | 188 148 |
| `web.js` brotli | 5 528 |
