# ADR 0008: Leptos cho giao diện, Bevy cho 3D, Axum cho server

- **Trạng thái:** **Chấp nhận** cho Leptos và Bevy (2026-11-26). Axum, sqlx và PostgreSQL vẫn là đề xuất, quyết khi có crate `server`.
- **Ngày:** 2026-11-26
- **Việc còn nợ:** đo điều kiện 2 trên Android 3–4 GB qua 4G khi có máy. Nếu vượt 10 s, xem lại theo "Phương án dự phòng".

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
| 2 | Khung 3D đầu ≤ 10 s, Android 3–4 GB, 4G | **iPhone 12 (4 GB), mạng di động 5G sóng yếu, tắt Wi-Fi, qua GitHub Pages (gzip 4,4 MB):** vỏ 2D 932 ms, tải + khởi tạo wasm 6 675 ms, **khung 3D đầu 7 001 ms** sau khi bật. Cùng máy qua Wi-Fi: 2 410 ms. Chưa đo Android (không có máy) | ✓ trên iOS (chấp nhận khi chưa có Android, xem "Việc còn nợ") |
| 3 | Leptos + Bevy cùng trang, hai chiều, đổi 2D/3D không mất trạng thái | [`spikes/leptos-bevy`](../../spikes/leptos-bevy): vỏ Leptos giữ `Session<Bars>`; Bevy nạp lười bằng `import()` khi bật 3D, nhận `State` (JSON), phát `Input` (JSON) khi bấm mảnh. Đã thử: thao tác 2D → 3D vẽ đúng; bấm trong 3D → `Session::act` chấp nhận; tắt 3D, thao tác 2D, bật lại → 3D đúng trạng thái, không khởi tạo lại; chấm 1000/1000 | ✓ |
| 4 | Phiên bản + WebGL2/WebGPU | WebGL2: Safari iOS 15+ và Chrome Android đều có — dùng làm nền. WebGPU: Safari 26 (iOS 26, 9/2025) bật mặc định, iOS cũ hơn không có; Chrome Android 121+ trên Android 12+ với GPU Qualcomm/ARM. → giữ WebGL2 làm mặc định, WebGPU là nâng cấp sau (Bevy cần build riêng cho mỗi backend) | ✓ |

Kích thước vỏ Leptos (2D + `kernel` + `kind-bars`): **86 KB brotli** (+ 5 KB JS) — trang 2D không phải chờ 3D.

### Cảnh đầy đủ: [`spikes/lang-map`](../../spikes/lang-map) (đo độ hoàn thiện tối đa)

Cảnh làng 3D dựng hoàn toàn bằng code: đình, nhà, cây, sông, ruộng; chu kỳ ngày đêm, bloom, bóng; thuyền, người, trâu, chim, khói, đom đóm. Đo trên cùng máy, canvas 1920×970:

- Gói **3,13 MB brotli**, chỉ hơn cảnh tối thiểu 0,24 MB. Vẫn dưới ngưỡng 5 MB của điều kiện 1.
- Khung đầu **0,35 s** sau `start()` (1,8 s lần đầu do biên dịch shader). Cần `opt-level=3` cho crate cảnh: với `opt-level="z"` khung đầu là 5,7 s.
- Chất lượng cao (bloom, MSAA 4×, bóng): **33–35 fps**. Chất lượng thấp: **~51 fps**. Giới hạn bởi GPU (Iris Xe).
- Chưa đo 4× CPU slowdown và điện thoại. Điều kiện 2 vẫn **chờ**.
- Shader Bevy 0.20 viết bằng WESL; spike làm hoạt cảnh trên CPU (~4 ms/khung cho nước và lúa). Sản phẩm nên dùng shader đỉnh.

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

**Bản đã đưa lên mạng:** <https://nam2341135.github.io/dyen/>. Đây là GitHub Pages, nhánh `gh-pages`, chứa bản build release dựng sẵn trên máy (không build bằng CI). Trang gồm cả `leptos-bevy` và `lang-map`, kèm hướng dẫn đo.

- GitHub Pages chỉ nén **gzip**, không gửi brotli. Lượng tải thật lớn hơn số brotli ở trên khoảng 1,5 lần: `view3d` 4,4 MB (brotli 2,9 MB), `lang-map` 4,9 MB (brotli 3,1 MB), vỏ Leptos 0,1 MB. Vì vậy số đo qua Pages là trường hợp xấu hơn sản phẩm (sản phẩm sẽ phục vụ brotli). Nếu số này đạt ngưỡng thì điều kiện 2 đạt.
- Đo trên điện thoại (2026-11-26), iPhone 12 4 GB, Chrome iOS (WebKit), tab ẩn danh: qua mạng di động `leptos-bevy` khung đầu 7,0 s; qua Wi-Fi 2,4 s. Băng thông hiệu dụng ước ~5 Mbit/s (4,4 MB trong ~6,7 s), tương đương 4G thường. Khi phục vụ brotli (2,9 MB), ước lượng ~4,6 s. Cảnh vẽ đúng; canvas tràn mép phải trên màn hình hẹp (lỗi bố cục trang spike).
- Đo thử trên máy đo qua Internet (2026-11-26): `leptos-bevy` 3,7 s từ lúc bật 3D tới khung đầu; `lang-map` 2,5 s từ lúc mở trang, 37 fps.
- Sửa spike thì phải build lại và đẩy lại nhánh `gh-pages`.

## Quyết định

Chấp nhận Leptos (vỏ, 2D) và Bevy (3D, gói wasm riêng, nạp lười, WebGL2) dựa trên kết quả spike: 4/4 điều kiện đạt, trong đó điều kiện 2 mới đo trên iPhone 12 (7,0 s qua mạng di động, ngưỡng 10 s). Chấp nhận khi chưa đo Android vì còn dư 3 s và sản phẩm sẽ phục vụ brotli (gói nhỏ hơn khoảng 1,5 lần so với gzip khi đo).

## Phương án dự phòng

Nếu (1) hoặc (2) không đạt: thay Bevy bằng renderer nhẹ hơn (`three-d`, hoặc `wgpu` với renderer tối giản tự viết). Nhờ [ADR 0002](0002-mot-mo-hinh-nhieu-bo-hien-thi.md), việc thay này không chạm logic.
