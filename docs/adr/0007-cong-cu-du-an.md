# ADR 0007: Công cụ dự án bằng `cargo xtask`, toolchain stable

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Cần một lệnh duy nhất để người và AI tự kiểm tra trước khi coi việc là xong. Máy phát triển là Windows; không muốn yêu cầu cài thêm công cụ (just, make, bash script) chỉ để chạy kiểm tra.

## Quyết định

- Dùng mẫu **xtask**: crate `xtask` trong workspace, gọi qua alias `cargo xtask <lệnh>` (khai báo trong `.cargo/config.toml`). Chỉ cần Rust là chạy được.
- Lệnh chính: `cargo xtask ci` = `fmt --check` → `clippy -D warnings` → `test` → `deps` (kiểm tra ranh giới). Sau này thêm: kiểm tra nội dung, build wasm, ngân sách kích thước wasm.
- `rust-toolchain.toml` dùng kênh `stable` kèm thành phần `rustfmt`, `clippy` và target `wasm32-unknown-unknown`. `Cargo.toml` khai báo `rust-version` làm phiên bản tối thiểu.
- Edition 2024.

## Hệ quả

- Không ghim phiên bản stable chính xác → hai máy có thể chạy bản khác nhau trong vài tuần. Chấp nhận ở giai đoạn một người; khi có CI hoặc thêm người, ghim phiên bản chính xác.
- Lint dùng chung khai báo ở `[workspace.lints]`, mọi crate kế thừa.
- **Bổ sung (2026-11-26):** `ci` có thêm bước `docs` (`cargo xtask docs`). Bước này kiểm:
  - liên kết tương đối trong mọi file `.md`;
  - bảng ADR khớp trạng thái từng file;
  - `CLAUDE.md` nhập `AGENTS.md`;
  - `docs/STATUS.md` có đủ mục;
  - mỗi `kind-*` có README và test hợp đồng.

  Lý do: tài liệu bàn giao cũ đi lặng lẽ thì người đến sau làm theo điều sai. Nợ ghim phiên bản ở trên đã đến hạn, theo dõi ở `docs/STATUS.md`.
