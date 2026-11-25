# dyen

Nền tảng học tập từ mầm non đến đại học, bằng Rust, cho web và mobile.

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
  kernel/     hợp đồng chung: Kind, dấu vết, phát lại, Outcome (thuần)
  kinds/      mỗi loại hoạt động một crate, vd. kinds/bars (thuần)
  registry/   danh sách đóng các loại hoạt động (thuần)
  content/    đọc + kiểm tra nội dung học
  cli/        công cụ `dyen`
xtask/        công cụ dự án (cargo xtask)
content/      file nội dung học
docs/         tầm nhìn, thuật ngữ, ADR
spikes/       thử nghiệm có thể vứt bỏ (không thuộc workspace)
```
