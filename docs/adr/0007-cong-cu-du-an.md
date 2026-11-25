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
