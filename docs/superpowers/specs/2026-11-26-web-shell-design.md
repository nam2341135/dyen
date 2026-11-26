# Thiết kế: vỏ ứng dụng web đầu tiên (crate `web`)

Ngày: 2026-11-26. Trạng thái: chờ duyệt.

## Mục đích

ADR 0008 đã chấp nhận Leptos cho vỏ và 2D. Bước này dựng **vỏ ứng dụng thật, tối thiểu**: người học mở trang, chọn một hoạt động trong `content/`, làm ở 2D, thấy kết quả chấm. Mục đích là chứng minh chuỗi `content` → `registry` → `kind-*` → bộ hiển thị chạy được trên trình duyệt với nội dung thật, không phải cấu hình viết cứng như spike.

## Mục tiêu

- Crate `crates/web` (Leptos CSR, wasm), có trong workspace và qua `cargo xtask ci`.
- Màn hình chọn: liệt kê gói hoạt động (`sets`) và hoạt động trong mỗi gói.
- Màn hình hoạt động: đề bài (`prompt`), bộ hiển thị 2D của loại, kết quả chấm (`Outcome`) với tên nhãn lấy từ `content/tags.toml`; nút làm lại, nút quay về.
- Bộ hiển thị 2D cho **cả ba loại**: `bars` (SVG thanh + thước, dựa trên spike), `choice` (nút chọn + nộp), `section` (SVG hình lập phương chiếu xiên, bấm đỉnh/cạnh để đánh dấu, nút cắt).
- Script build (`crates/web/build.sh`) ra thư mục `dist/` tĩnh: `index.html` + wasm + JS, đo kích thước brotli.

## Không làm (bước này)

- 3D/Bevy, server, đăng nhập, lưu tiến độ (kể cả `localStorage`), router, i18n, theme.
- Đưa lên GitHub Pages (việc sau, khi vỏ ổn).
- Không sửa `kind-*`. Nếu bộ hiển thị cần dữ liệu mà `State` chưa lộ ra, thêm hàm đọc `pub` nhỏ trong crate loại đó, không đổi logic.

## Phản mục tiêu (làm thế này là hỏng dù chạy được)

- Bộ hiển thị tự sửa trạng thái học tập hoặc tự chấm (vi phạm ADR 0002). Mọi đầu vào phải qua `Session::act`; kết quả chấm phải từ `Kind::evaluate`.
- Tải font, script, ảnh từ bên ngoài (ADR 0005).
- Dùng `f32/f64` cho quyết định logic (chỉ được dùng để vẽ).

## Ràng buộc

- ADR 0006: thêm hàng `web` vào bảng và vào `RULES` trong `xtask/src/deps.rs`, cùng một thay đổi.
- `web` phải biên dịch được trên máy host để `clippy --workspace` và `test --workspace` chạy trong CI (Leptos CSR + web-sys biên dịch được trên host; phần gọi trình duyệt chỉ chạy khi là wasm).
- Phiên bản Leptos `=0.8.23` như spike đã đo.

## Cách đã chọn

Đã cân nhắc ba cách đặt bộ hiển thị từng loại:

1. **Trong `web`, mỗi loại một module `web/src/kinds/<tên>.rs`; `web` có một `match` đóng chọn bộ hiển thị.** — **Đã chọn** (chủ dự án duyệt 2026-11-26).
2. Mỗi loại một crate hiển thị `view-<tên>`. Bỏ: gấp đôi số crate, cần quy tắc ranh giới mới, chưa có ca dùng thứ hai.
3. Hoãn, viết ADR riêng trước. Bỏ: cách 1 chỉ cần sửa một câu trong ADR 0010.

Hệ quả cho tài liệu: ADR 0010 và glossary sửa câu "`registry` là chỗ duy nhất rẽ nhánh theo loại" thành "chỗ duy nhất rẽ nhánh theo loại **ở tầng logic**; vỏ `web` có một `match` đóng chọn bộ hiển thị". Thêm loại mới thì trình biên dịch chỉ ra cả hai chỗ.

## Thiết kế

**Phụ thuộc** (ADR 0006): `web` → `kernel`, `registry`, `content`, `kind-*`, `lib-*`. Loại: có IO (trình duyệt). Ngoài: `leptos` (csr), `wasm-bindgen`, `web-sys`, `serde_json`.

**Nạp nội dung:** `build.rs` của `web` đọc `content/` (tags, activities, sets) lúc biên dịch và sinh một hàm trả về `content::Sources`; lúc chạy gọi `Library::from_sources` (không IO). Lỗi nội dung đã bị `dyen check` chặn trong CI; trong `web` chỉ hiện thông báo lỗi nếu có.

**Phiên chơi:** mỗi bộ hiển thị giữ một `kernel::Session<K>` (vỏ giữ trạng thái có thẩm quyền, ADR 0002). Bộ hiển thị nhận `&K::State`, phát `K::Input`; vỏ gọi `Session::act` với `Actor::Learner` và thời điểm `performance.now()`. Đầu vào bị từ chối: hiện lý do, không đổi trạng thái. Kết quả: `kernel::run::<K>(config, session.trace())` sau mỗi bước; hiện `Outcome` khi hoạt động kết thúc hoặc khi người học nộp.

**Cấu trúc file:**

```
crates/web/
  Cargo.toml
  build.rs            nhúng content/
  build.sh            wasm → dist/ (wasm-bindgen, wasm-opt -Oz, brotli)
  index.html          không tài nguyên ngoài
  README.md           thuật ngữ, cách chạy
  src/lib.rs          điểm vào wasm, App, chọn màn hình (signal, không router)
  src/catalog.rs      màn hình chọn gói/hoạt động
  src/play.rs         màn hình hoạt động, match đóng theo AnyConfig
  src/outcome.rs      hiển thị Outcome + tên nhãn
  src/kinds/bars.rs   bộ hiển thị thanh
  src/kinds/choice.rs bộ hiển thị chọn đáp án
  src/kinds/section.rs bộ hiển thị lập phương (chiếu xiên, f64 chỉ để vẽ)
```

**Test:** test đơn vị cho phần thuần trong `web` (phép chiếu lập phương, ánh xạ nội dung nhúng → `Library` không lỗi). Kiểm tra trên trình duyệt thủ công bằng bản build `dist/` (xem ở khung xem trong app).

## Để người làm tự quyết

Bố cục và màu sắc cụ thể, chữ trên nút, cách hiện lý do từ chối. Ghi lại trong README của `web` những gì đáng nhớ.
