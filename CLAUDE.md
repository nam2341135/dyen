# Hướng dẫn cho AI làm việc trong dự án này

Dự án: nền tảng học tập, lõi trung lập về sư phạm (xem `docs/vision.md`, ADR 0010). Giai đoạn hiện tại: **xây nền móng**, chưa xây sản phẩm. Lộ trình ở `docs/README.md`.

## Đọc trước khi viết code

1. `docs/glossary.md` — dùng đúng thuật ngữ. Không đặt tên mới cho khái niệm đã có.
2. ADR liên quan trong `docs/adr/`. Nếu việc cần làm mâu thuẫn với một ADR đã "Chấp nhận": **dừng lại và hỏi**, đừng lặng lẽ làm khác.

## Lệnh

```
cargo xtask ci      # fmt --check, clippy -D warnings, test, kiểm tra ranh giới — PHẢI xanh trước khi báo xong
cargo xtask deps    # chỉ kiểm tra ranh giới crate
cargo xtask fmt     # tự định dạng code
cargo test -p <crate>
cargo run -p cli -- check                          # kiểm tra thư mục content/
cargo run -p cli -- replay <hoạt-động> <dấu-vết.json>  # phát lại + chấm, in JSON
UPDATE_SNAPSHOTS=1 cargo test -p cli               # cập nhật snapshot CÓ CHỦ ĐÍCH (giải thích trong mô tả thay đổi)
```

Snapshot ở `crates/cli/tests/traces/` giữ định dạng dấu vết `format = 1` đọc được mãi. Snapshot đổi mà không giải thích được = lỗi, không phải "cập nhật cho qua".

Không báo "xong" khi chưa chạy `cargo xtask ci` và thấy nó qua. Nếu nó đỏ, báo nguyên văn lỗi.

**Môi trường Windows:** cần Visual Studio C++ Build Tools (workload "Desktop development with C++"). Nếu thiếu, mọi bước cần link (test, chạy binary, build script) sẽ báo `link: extra operand` — đó là `link` của Git Bash bị gọi nhầm, không phải lỗi code. Trong Git Bash, `cargo` nằm ở `~/.cargo/bin` (có thể chưa có trong PATH).

## Ranh giới (ADR 0006) — máy kiểm tra, không thương lượng

- `kernel` ← `kind-*` ← `registry` ← `content` ← `cli`. Mỗi `kind-*` chỉ phụ thuộc `kernel` và `lib-*`, không phụ thuộc loại khác. Bảng đầy đủ ở ADR 0006.
- `lib-*` (`crates/libs/<tên>`, ADR 0012): thư viện thuần dùng chung cho các `kind-*` (vd. `lib-exact`: số hữu tỉ, hình học chính xác). Không biết `Kind`, dấu vết, nhãn hay sư phạm; chỉ phụ thuộc `lib-*` khác. Chỉ thêm khi có ≥ 2 loại dùng (hoặc 1 loại + 1 loại sắp làm có bản nháp), và ghi tên crate trong ADR.
- **Crate thuần** (`kernel`, `lib-*`, `kind-*`, `registry`): không IO, không mạng, không đồng hồ, không thread, không số ngẫu nhiên không seed. Chỉ phụ thuộc ngoài trong danh sách cho phép ở `xtask/src/deps.rs`.
- Muốn thêm phụ thuộc vào crate thuần: hỏi trước. Nếu được đồng ý, sửa danh sách trong `xtask/src/deps.rs` và cập nhật ADR 0006.
- Thêm crate mới: cập nhật cả ADR 0006 và `xtask/src/deps.rs` trong cùng thay đổi. Ngoại lệ: loại hoạt động mới `crates/kinds/<tên>` (crate `kind-<tên>`) tự khớp quy tắc `kind-*`.

## Loại hoạt động (ADR 0010)

- Lõi (`kernel`) không biết môn học hay sư phạm. Đừng đưa khái niệm của một ý tưởng cụ thể vào `kernel`, `registry`, `content` hay `cli`.
- Thêm ý tưởng = thêm crate `crates/kinds/<tên>` hiện thực `kernel::Kind`, có README thuật ngữ riêng và một test chạy `kernel::contract::check`; rồi thêm biến thể vào `registry`. Nhãn mới phải có trong `content/tags.toml`.
- Thuật ngữ riêng của một loại nằm trong README của crate đó, không trong `docs/glossary.md`.

## Quy tắc logic (ADR 0002, 0003)

- Bộ hiển thị (2D/3D) **không** giữ hay sửa trạng thái học tập. Chỉ vẽ `State` và phát `Input`.
- Bộ hiển thị theo ADR 0013: vùng bấm ≥ 24 px ở màn 360 px (có test), nút ≥ 44 px; animation không bao giờ chặn đầu vào; màu chỉ là biến CSS, code Rust chỉ sinh `class`.
- `apply` / `replay` phải tất định. Dùng số nguyên hoặc phân số hữu tỉ cho mọi quyết định logic; `f32/f64` chỉ dùng cho hiển thị.
- Không dùng `HashMap`/`HashSet` ở nơi thứ tự duyệt ảnh hưởng kết quả — dùng `BTreeMap`/`Vec`.
- Định dạng dấu vết có số phiên bản; thay đổi phá tương thích phải tăng phiên bản và giữ đường đọc bản cũ.

## Dữ liệu trẻ em (ADR 0005)

- Không thêm trường thông tin cá nhân nào (email, SĐT, ngày sinh, ảnh, giọng nói, chữ tự do của trẻ) khi chưa có ADR cho phép.
- Không thêm thư viện analytics/tracking/CDN bên thứ ba vào bất kỳ phần nào học sinh dùng.

## Phong cách

- Edition 2024. Lint chung ở `[workspace.lints]` trong `Cargo.toml` gốc; không tắt lint bằng `#[allow]` mà không có chú thích lý do.
- Lỗi trong thư viện: `thiserror`, kiểu lỗi cụ thể. Không `unwrap()`/`expect()` ngoài test và `xtask`/`cli` (ở đó chấp nhận `expect` có thông điệp rõ).
- Doc comment (`///`) cho mọi mục `pub`, bằng tiếng Việt hoặc tiếng Anh, nhất quán trong một file. Tên trong code bằng tiếng Anh theo cột "Trong code" của glossary.
- Test đặt cạnh code (`#[cfg(test)] mod tests`) cho đơn vị; test tích hợp ở `tests/`.
- Thay đổi nhỏ, mỗi thay đổi một mục đích. Không tạo trừu tượng "để sau này dùng" khi chưa có ca dùng thứ hai.

## Không làm khi chưa được yêu cầu

- Không tạo crate `web`, `table3d`, `server` (thuộc giai đoạn sau / spike).
- Không thêm framework, ORM, hệ plugin tổng quát.
- Không commit, không push.
