# dyen

Nền tảng học bằng thao tác mô hình — từ mầm non đến đại học, bắt đầu với phân số lớp 4.

**Giai đoạn hiện tại: nền móng.** Chưa có sản phẩm chạy được.

- Đọc trước: [docs/vision.md](docs/vision.md), [docs/glossary.md](docs/glossary.md), [docs/README.md](docs/README.md) (lộ trình + ADR).
- Quy tắc cho AI: [CLAUDE.md](CLAUDE.md).

## Yêu cầu

- Rust (rustup) — `rust-toolchain.toml` tự cài đúng thành phần và target wasm.
- Windows: Visual Studio C++ Build Tools.

## Lệnh

```
cargo xtask ci     # kiểm tra toàn bộ: fmt, clippy, test, ranh giới crate
cargo xtask deps   # chỉ kiểm tra ranh giới crate
cargo xtask fmt    # định dạng code
```

## Cấu trúc

```
crates/
  domain/     kiểu dùng chung (thuần)
  manip/      mô hình: trạng thái, hành động, phát lại (thuần)
  diagnose/   chấm và chẩn đoán từ dấu vết (thuần)
  content/    đọc + kiểm tra nội dung học
  cli/        công cụ `dyen`
xtask/        công cụ dự án (cargo xtask)
content/      file nội dung học
docs/         tầm nhìn, thuật ngữ, ADR
spikes/       thử nghiệm có thể vứt bỏ (không thuộc workspace)
```
