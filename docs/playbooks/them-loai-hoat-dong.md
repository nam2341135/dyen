# Thêm một loại hoạt động

Loại hoạt động là một crate `kind-<tên>` cắm vào lõi trung lập (ADR 0010). Lõi không biết môn học. Mọi khái niệm riêng của loại nằm trong crate của nó.

Mẫu để chép: `crates/kinds/choice` (đơn giản nhất). Nếu cần hình học hay toán chính xác thì chép `crates/kinds/section`, loại dùng `lib-exact`.

## Trước khi viết code

1. Ý tưởng đã có file trong [docs/ideas/](../ideas/README.md), mô tả trạng thái, đầu vào, chấm và phát hiện của loại này. Chưa có thì viết trước.
2. Kiểm tra loại mới có cần gì lõi chưa có không: đồng hồ, ngẫu nhiên, nhiều người, 3D. Nếu có, viết ADR "Đề xuất" và hỏi. Đừng sửa `kernel` cho riêng một loại.

## Crate thuần

3. Tạo `crates/kinds/<tên>`:
   - `Cargo.toml`: tên `kind-<tên>`, phụ thuộc `kernel`, `serde`, `thiserror`, cùng `lib-*` nếu cần. Chép nguyên khối `version.workspace = true …` và `[lints] workspace = true` từ mẫu.
   - Thêm `kind-<tên> = { path = "crates/kinds/<tên>" }` vào `[workspace.dependencies]` ở `Cargo.toml` gốc. `members` đã có glob `crates/kinds/*`.
   - `xtask/src/deps.rs` tự nhận crate tên `kind-*`, không cần sửa.
4. `src/lib.rs`: hiện thực trait `kernel::Kind`:
   - `Config`, `State`, `Input`, lỗi đầu vào bằng `thiserror`;
   - `apply` tất định;
   - chấm và nhãn.
   Tuân `AGENTS.md`, mục "Quy tắc logic và hiển thị": không float trong quyết định, không `HashMap` ở chỗ thứ tự ảnh hưởng kết quả.
5. `README.md` của crate, theo mẫu `kind-choice`. Các mục:
   - "Thuật ngữ riêng" (bảng Tiếng Việt / Trong code / Định nghĩa);
   - "Đầu vào (`op`)";
   - "Đánh giá";
   - "Nhãn".
6. `tests/contract.rs`: chạy bộ kiểm hợp đồng chung `kernel::contract` (chép từ mẫu, đổi kiểu). `cargo xtask docs` đỏ nếu thiếu README hay file này.

## Nối vào hệ thống

7. `crates/registry`:
   - phụ thuộc `kind-<tên>.workspace = true`;
   - thêm biến thể vào `AnyConfig`, `AnyState` và hằng `KINDS`;
   - sửa từng `match` mà trình biên dịch chỉ ra;
   - sửa test liệt kê `KINDS`.
8. Nội dung (`content/`, xem [content/README.md](../../content/README.md)):
   - ít nhất một `activities/<id>.toml` có `[config] kind = "<tên>"`;
   - nhãn mới vào `tags.toml`;
   - thêm hoạt động vào một bộ ở `sets/`.
   Kiểm bằng `cargo run -p cli -- check`.
9. Dấu vết mẫu: tạo `crates/cli/tests/traces/<id>--<tình-huống>.json` (đúng ngay, tự sửa, hết lượt…, và ít nhất một ca `--hong-` bị từ chối). Sinh kết quả bằng `UPDATE_SNAPSHOTS=1 cargo test -p cli`, rồi **đọc** từng file `.expected.json` để chắc kết quả đúng ý.

## Vỏ web

10. Tạo `crates/web/src/kinds/<tên>.rs`:
    - `pub fn view(config) -> AnyView`;
    - khai báo trong `kinds/mod.rs`;
    - một nhánh trong `activity.rs`;
    - phụ thuộc trong `crates/web/Cargo.toml`.
11. Theo ADR 0013:
    - vùng bấm ≥ 24 px ở 360 px, kèm test dùng `kinds::tests::{PHONE_SVG_PX, MIN_TARGET_PX}` (xem `bars.rs`, `section.rs`);
    - không `style=`, màu là biến `--<tên>-…` trong `:root` của `crates/web/index.html`;
    - trạng thái không chỉ báo bằng màu.

## Xong khi

- `cargo xtask ci` xanh.
- `cargo run -p cli -- check` không lỗi.
- `crates/web/build.sh dev` build được. Mở cấu hình `web` (cổng 8091) ở bề rộng 360 px, chơi hết hoạt động mới, và thấy kết quả đúng.
- `docs/ideas/<ý tưởng>.md` ghi loại đã làm. `docs/STATUS.md` đã cập nhật.
- Mô tả PR có mục "Quyết định đã tự đưa ra" và giải thích các snapshot mới.
