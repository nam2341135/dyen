# ADR 0008: Leptos cho giao diện, Bevy cho 3D, Axum cho server

- **Trạng thái:** **Đề xuất — 3/4 điều kiện đạt, chờ đo trên điện thoại (điều kiện 2)**
- **Ngày:** 2026-11-26

## Bối cảnh

Cần chọn thư viện UI, 3D và server. Khả năng chạy trên trình duyệt điện thoại và kích thước gói phụ thuộc phiên bản; chưa đo trên máy thật.

## Đề xuất

- **Leptos** (CSR trước) cho vỏ ứng dụng và bộ hiển thị 2D bằng SVG.
- **Bevy**, build thành gói wasm riêng, nạp lười; backend WebGL2 trước.
- **Axum** + **sqlx** + **PostgreSQL** cho server.

## Điều kiện để chuyển sang "Chấp nhận"

Spike trong `spikes/` phải đo và ghi lại:

1. Kích thước gói Bevy tối thiểu (một cảnh: mặt bàn + một vật thể + ánh sáng + picking) sau `wasm-opt` và nén brotli. **Ngưỡng: ≤ 5 MB nén.**
2. Thời gian từ lúc tải tới khung hình 3D đầu tiên trên một điện thoại Android 3–4 GB RAM qua 4G. **Ngưỡng: ≤ 10 giây.**
3. Leptos và Bevy cùng một trang, trao đổi `State`/`Input` hai chiều, đổi qua lại 2D/3D không mất trạng thái.
4. Phiên bản Leptos, Bevy hiện hành và tình trạng WebGL2/WebGPU trên Safari iOS và Chrome Android — đối chiếu tài liệu chính thức.

## Kết quả spike (2026-11-26)

Máy đo: Windows 11, Intel Iris Xe, Chrome (WebGL2 qua ANGLE/D3D11). Build: `opt-level="z"`, `lto="fat"`, `codegen-units=1`, `panic="abort"`, rồi `wasm-opt -Oz`, nén `brotli -q 11`. Phiên bản: **Bevy 0.20.1**, **Leptos 0.8.23**, wasm-bindgen 0.2.130, Rust 1.97.1.

| # | Điều kiện | Kết quả | Đạt? |
|---|---|---|---|
| 1 | Gói Bevy tối thiểu ≤ 5 MB nén | [`spikes/bevy-min`](../../spikes/bevy-min): wasm 16,7 MB → 14,2 MB sau `-Oz` → **2,89 MB brotli** (+ 12 KB JS) | ✓ |
| 2 | Khung 3D đầu ≤ 10 s, Android 3–4 GB, 4G | Chưa đo trên điện thoại. Trên máy đo (localhost): tải + khởi tạo wasm 67 ms, khung đầu 343 ms sau khi bật 3D. Ước lượng tải 2,9 MB qua 4G ~10 Mbit/s ≈ 2,5 s | **chờ** |
| 3 | Leptos + Bevy cùng trang, hai chiều, đổi 2D/3D không mất trạng thái | [`spikes/leptos-bevy`](../../spikes/leptos-bevy): vỏ Leptos giữ `Session<Bars>`; Bevy nạp lười bằng `import()` khi bật 3D, nhận `State` (JSON), phát `Input` (JSON) khi bấm mảnh. Đã thử: thao tác 2D → 3D vẽ đúng; bấm trong 3D → `Session::act` chấp nhận; tắt 3D, thao tác 2D, bật lại → 3D đúng trạng thái, không khởi tạo lại; chấm 1000/1000 | ✓ |
| 4 | Phiên bản + WebGL2/WebGPU | WebGL2: Safari iOS 15+ và Chrome Android đều có — dùng làm nền. WebGPU: Safari 26 (iOS 26, 9/2025) bật mặc định, iOS cũ hơn không có; Chrome Android 121+ trên Android 12+ với GPU Qualcomm/ARM. → giữ WebGL2 làm mặc định, WebGPU là nâng cấp sau (Bevy cần build riêng cho mỗi backend) | ✓ |

Kích thước vỏ Leptos (2D + `kernel` + `kind-bars`): **86 KB brotli** (+ 5 KB JS) — trang 2D không phải chờ 3D.

### Ghi chú kỹ thuật

- Bộ feature Bevy tối thiểu (không `default`): xem `spikes/bevy-min/Cargo.toml`.
- Tonemapping mặc định (`TonyMcMapFace`) cần feature `tonemapping_luts`; thiếu thì cảnh ra màu hồng tím. Spike dùng `Tonemapping::AcesFitted` để không phải kéo LUT.
- `strip = true` làm wasm-bindgen mất section `target_features`; để `strip = false`, `wasm-opt` đã bỏ tên.
- Build release fat-LTO của Bevy cần ~3,5 GB RAM cho một tiến trình `rustc`; hai build song song trên máy 16 GB bị LLVM hết bộ nhớ. Build lần đầu ~5–6 phút.
- Cầu nối JS chỉ ~30 dòng (`spikes/leptos-bevy/web/index.html`). Bộ hiển thị 3D không giữ trạng thái học tập: mỗi `State` mới dựng lại cảnh (đủ cho spike; sản phẩm cần cập nhật theo phần khác biệt nếu cảnh lớn).
- Chưa spike: Axum/sqlx/PostgreSQL (không thuộc rủi ro trình duyệt; quyết khi có crate `server`).

### Cách đo điều kiện 2

```
cd spikes/leptos-bevy && ./build.sh
```

Phục vụ `dist/` qua HTTPS hoặc mạng LAN (vd. `python -m http.server -d dist 8080`), mở trên điện thoại Android 3–4 GB RAM với 4G (hoặc Chrome DevTools giới hạn "Fast 4G" + 4× CPU slowdown làm ước lượng), bật "Hiển thị 3D"; dòng chữ đầu trang ghi thời gian khởi tạo wasm và khung đầu tiên. Lưu ý `http.server` không gửi `.br`; số đo qua mạng thật cần máy chủ có nén brotli/gzip.

## Phương án dự phòng

Nếu (1) hoặc (2) không đạt: thay Bevy bằng renderer nhẹ hơn (`three-d`, hoặc `wgpu` với renderer tối giản tự viết). Nhờ [ADR 0002](0002-mot-mo-hinh-nhieu-bo-hien-thi.md), việc thay này không chạm logic.
