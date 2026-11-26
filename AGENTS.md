# Hướng dẫn cho người và AI làm việc trong dự án này

Dự án: nền tảng học tập, lõi trung lập về sư phạm (xem `docs/vision.md`, ADR 0010). Giai đoạn hiện tại: **xây nền móng**, đã có vỏ web tối thiểu (`crates/web`).

File này là nguồn hướng dẫn duy nhất cho mọi công cụ AI. `CLAUDE.md` chỉ nhập file này. Sửa ở đây, không sửa bản chép.

## Bắt đầu mỗi phiên

1. [`docs/STATUS.md`](docs/STATUS.md): đang làm gì, việc tiếp theo, việc chờ người quyết. Nếu người dùng chỉ nói "làm tiếp": làm mục dở dang ở "Đang làm"; không có thì làm mục đầu tiên ở "Tiếp theo".
2. [`docs/playbooks/`](docs/playbooks/README.md): công thức từng bước cho loại việc đang làm (thêm loại hoạt động, thêm nội dung, sửa vỏ web…).
3. [`docs/glossary.md`](docs/glossary.md): dùng đúng thuật ngữ, không đặt tên mới cho khái niệm đã có.
4. ADR liên quan trong `docs/adr/`. Nếu việc cần làm mâu thuẫn với một ADR đã "Chấp nhận": **dừng lại và hỏi**, đừng lặng lẽ làm khác.

## "Xong" nghĩa là

- `cargo xtask ci` xanh. Nếu đỏ, báo nguyên văn lỗi. Không báo "xong" khi chưa chạy.
- Tài liệu đi cùng code trong cùng thay đổi:
  - thuật ngữ mới vào glossary hoặc README của crate;
  - quyết định kiến trúc vào ADR;
  - tiến độ vào `docs/STATUS.md`.
- Snapshot dấu vết đổi thì phải giải thích lý do trong mô tả thay đổi.
- Quyết định nhỏ đã tự đưa ra (chọn cách A thay vì B) ghi trong mô tả PR, mục "Quyết định đã tự đưa ra".

## Trước khi dừng (hết phiên, sắp hết token, đổi việc)

Cập nhật mục "Đang làm" trong `docs/STATUS.md` theo [`docs/playbooks/ban-giao.md`](docs/playbooks/ban-giao.md). Điều gì chỉ nằm trong bộ nhớ của công cụ (memory, ledger trong `.superpowers/`, lịch sử chat) thì người sau **không thấy**.

## Lệnh

```
cargo xtask ci      # fmt --check, clippy -D warnings, test, ranh giới crate, tài liệu — PHẢI xanh trước khi báo xong
cargo xtask deps    # chỉ kiểm tra ranh giới crate
cargo xtask docs    # chỉ kiểm tra tài liệu: liên kết, bảng ADR, STATUS, README của loại hoạt động
cargo xtask fmt     # tự định dạng code
cargo test -p <crate>
cargo run -p cli -- check                          # kiểm tra thư mục content/
cargo run -p cli -- replay <hoạt-động> <dấu-vết.json>  # phát lại + chấm, in JSON
UPDATE_SNAPSHOTS=1 cargo test -p cli               # cập nhật snapshot CÓ CHỦ ĐÍCH (giải thích trong mô tả thay đổi)
crates/web/build.sh dev                            # build vỏ web; xem crates/web/README.md
```

Snapshot ở `crates/cli/tests/traces/` giữ định dạng dấu vết `format = 1` đọc được mãi. Snapshot đổi mà không giải thích được là lỗi, không phải "cập nhật cho qua".

**Môi trường Windows:** cần Visual Studio C++ Build Tools (workload "Desktop development with C++"). Nếu thiếu, mọi bước cần link (test, chạy binary, build script) sẽ báo `link: extra operand`. Đó là `link` của Git Bash bị gọi nhầm, không phải lỗi code. Trong Git Bash, `cargo` nằm ở `~/.cargo/bin` (có thể chưa có trong PATH).

## Ranh giới (ADR 0006): máy kiểm tra, không thương lượng

- Hướng phụ thuộc: `kernel` ← `kind-*` ← `registry` ← `content` ← `cli` / `web`. Mỗi `kind-*` chỉ phụ thuộc `kernel` và `lib-*`, không phụ thuộc loại khác. Bảng đầy đủ ở ADR 0006.
- `lib-*` (`crates/libs/<tên>`, ADR 0012): thư viện thuần dùng chung cho các `kind-*` (ví dụ `lib-exact`: số hữu tỉ, hình học chính xác).
  - Không biết `Kind`, dấu vết, nhãn hay sư phạm; chỉ phụ thuộc `lib-*` khác.
  - Chỉ thêm khi có ít nhất 2 loại dùng (hoặc 1 loại + 1 loại sắp làm đã có bản nháp), và ghi tên crate trong ADR.
- **Crate thuần** (`kernel`, `lib-*`, `kind-*`, `registry`, `room-model`): không IO, không mạng, không đồng hồ, không thread, không số ngẫu nhiên không seed. Chỉ dùng phụ thuộc ngoài có trong danh sách cho phép ở `xtask/src/deps.rs`.
- Muốn thêm phụ thuộc vào crate thuần: hỏi trước. Nếu được đồng ý, sửa danh sách trong `xtask/src/deps.rs` và cập nhật ADR 0006.
- Thêm crate mới: cập nhật cả ADR 0006 và `xtask/src/deps.rs` trong cùng thay đổi. Ngoại lệ: loại hoạt động mới `crates/kinds/<tên>` (crate `kind-<tên>`) tự khớp quy tắc `kind-*`.

## Loại hoạt động (ADR 0010)

- Lõi (`kernel`) không biết môn học hay sư phạm. Đừng đưa khái niệm của một ý tưởng cụ thể vào `kernel`, `registry`, `content`, `cli` hay phần chung của `web`.
- Thêm ý tưởng = làm theo [`docs/playbooks/them-loai-hoat-dong.md`](docs/playbooks/them-loai-hoat-dong.md).
- Thuật ngữ riêng của một loại nằm trong README của crate đó, không trong `docs/glossary.md`.

## Quy tắc logic và hiển thị (ADR 0002, 0003, 0013)

- Bộ hiển thị (2D/3D) **không** giữ hay sửa trạng thái học tập. Chỉ vẽ `State` và phát `Input`.
- `apply` / `replay` phải tất định. Dùng số nguyên hoặc phân số hữu tỉ cho mọi quyết định logic; `f32/f64` chỉ dùng cho hiển thị.
- Không dùng `HashMap`/`HashSet` ở nơi thứ tự duyệt ảnh hưởng kết quả; dùng `BTreeMap`/`Vec`.
- Định dạng dấu vết có số phiên bản. Thay đổi phá tương thích phải tăng phiên bản và giữ đường đọc bản cũ.
- Bộ hiển thị theo ADR 0013:
  - vùng bấm ≥ 24 px ở màn 360 px, và có test;
  - nút ≥ 44 px;
  - animation không bao giờ chặn đầu vào;
  - màu chỉ là biến CSS; code Rust chỉ sinh `class`.

## Dữ liệu trẻ em (ADR 0005)

- Không thêm trường thông tin cá nhân nào (email, SĐT, ngày sinh, ảnh, giọng nói) khi chưa có ADR cho phép.
- Không thêm thư viện analytics/tracking/CDN bên thứ ba vào bất kỳ phần nào học sinh dùng.

## Phong cách

- Edition 2024. Lint chung ở `[workspace.lints]` trong `Cargo.toml` gốc. Không tắt lint bằng `#[allow]` mà không có chú thích lý do.
- Lỗi trong thư viện: dùng `thiserror`, kiểu lỗi cụ thể. Không `unwrap()`/`expect()` ngoài test và `xtask`/`cli`/`build.rs` (ở đó chấp nhận `expect` có thông điệp rõ).
- Doc comment (`///`) cho mọi mục `pub`, bằng tiếng Việt hoặc tiếng Anh, nhất quán trong một file. Tên trong code bằng tiếng Anh, theo cột "Trong code" của glossary.
- Test đơn vị đặt cạnh code (`#[cfg(test)] mod tests`); test tích hợp ở `tests/`. Tên test bằng tiếng Việt không dấu, mô tả hành vi (`vung_bam_du_lon`).
- Thay đổi nhỏ, mỗi thay đổi một mục đích. Không tạo trừu tượng "để sau này dùng" khi chưa có ca dùng thứ hai.
- Commit: `loại(phạm vi): mô tả` bằng tiếng Việt. Loại là `feat`, `fix`, `docs`, `ci`, `refactor` hoặc `test`. Ví dụ: `fix(web): …`. PR theo mẫu `.github/pull_request_template.md`.

## Không làm khi chưa được yêu cầu

- Không tạo crate `table3d`, `server` (thuộc giai đoạn sau).
- Không thêm framework, ORM, hệ plugin tổng quát.
- Không commit, không push. Khi được yêu cầu: làm trên nhánh riêng, không đẩy thẳng lên `main`.
