# dyen

Nền tảng học tập từ mầm non đến đại học, bằng Rust, cho web và mobile.

**Giai đoạn hiện tại: nền móng.** Đã có vỏ web tối thiểu với ba loại hoạt động, chạy thử ở <https://nam2341135.github.io/dyen/app/>.

Mới đến (người hay AI), đọc theo thứ tự:

1. [AGENTS.md](AGENTS.md): quy tắc làm việc, "xong" nghĩa là gì, ranh giới không được vượt. `CLAUDE.md` chỉ nhập file này.
2. [docs/STATUS.md](docs/STATUS.md): đang làm gì, việc tiếp theo, việc chờ người quyết.
3. [docs/playbooks/](docs/playbooks/README.md): công thức từng bước cho các việc hay gặp.
4. [docs/README.md](docs/README.md): tầm nhìn, thuật ngữ, ADR.

## Yêu cầu

- Rust (rustup) — `rust-toolchain.toml` tự cài đúng thành phần và target wasm.
- Windows: Visual Studio C++ Build Tools.

## Lệnh

```
cargo xtask ci     # kiểm tra toàn bộ: fmt, clippy, test, ranh giới crate, tài liệu
cargo xtask deps   # chỉ kiểm tra ranh giới crate
cargo xtask docs   # chỉ kiểm tra tài liệu
cargo xtask fmt    # định dạng code
crates/web/build.sh dev   # build vỏ web (crates/web/README.md)
```

## Cấu trúc

```
crates/
  kernel/     hợp đồng chung: Kind, dấu vết, phát lại, Outcome (thuần)
  kinds/      mỗi loại hoạt động một crate: bars, choice, section (thuần)
  libs/       thư viện thuần dùng chung cho các loại, vd. libs/exact (ADR 0012)
  registry/   danh sách đóng các loại hoạt động (thuần)
  content/    đọc + kiểm tra nội dung học
  cli/        công cụ `dyen`: kiểm nội dung, phát lại dấu vết
  web/        vỏ trình duyệt (Leptos CSR)
xtask/        công cụ dự án (cargo xtask)
content/      file nội dung học
docs/         tầm nhìn, thuật ngữ, ADR, STATUS, playbook
spikes/       thử nghiệm có thể vứt bỏ (không thuộc workspace)
```
