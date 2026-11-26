# Thiết kế: trang chủ là phòng học 3D

- **Ngày:** 2026-11-27
- **Trạng thái:** chờ chủ dự án duyệt
- **Nhánh:** `docs/phong-hoc-3d` (tách từ `feat/phong-cach-giay-muc`, PR #11)

## 1. Ý định

Chủ dự án muốn trang chủ là **một không gian**, không phải một danh sách: một phòng học 3D kiểu hình nền lofi, chi tiết, ấm cúng. Mỗi đồ vật là một lối vào một khu. Học sinh "bước vào phòng của mình" chứ không "mở một trang web".

Phòng cũng là chỗ học sinh thấy mình tiến bộ (huy chương, đồ vật đổi kiểu) và thấy là phòng *của mình* (tự trang trí, tự dán ghi chú).

Đây là bản phát triển, chưa phát hành cho học sinh thật. Ưu tiên máy tính để bàn, nhưng điện thoại cũng có phòng 3D ngay.

## 2. Những gì chủ dự án đã quyết

| Chủ đề | Quyết định |
|---|---|
| Kỹ thuật dựng phòng | Gói Bevy riêng, vỏ Leptos lo điều hướng (mục 4, cách 1) |
| Bàn học | Vở → **ôn tập**; máy tính → **khoá học, bài giảng lý thuyết**; cúp → **thi đấu**; thêm đồ trang trí cho bàn đỡ trống, sau này thay bằng lối vào cho ý tưởng mới |
| Tường | **Bảng ghi chú** (học sinh tự ghi, tương tác được với tờ note); **kệ huy chương, thành tựu** (lúc chưa có gì thì không được trông trống) |
| Chữ tự do của trẻ | **Bỏ hẳn** quy tắc "không có ô nhập chữ tự do" ở ADR 0005 |
| Điện thoại | Cũng có phòng 3D ngay (camera khác cho màn dọc) |
| Camera | Góc cố định; rê chuột thì vật sáng lên; bấm thì camera bay tới vật rồi vào khu |
| Phong cách | Như hình nền lofi đầu tư chi tiết |
| Mô hình 3D | Gói CC0 miễn phí (Kenney, Quaternius, Poly Pizza…) rồi chỉnh lại cho đồng bộ |
| Vào khu | Chuyển sang trang mới, có animation mượt và hiệu ứng chờ |
| Không khí | Trời ngoài cửa sổ theo giờ thật; một con vật nuôi. Không nhạc, không mưa |
| Phòng thay đổi | Học sinh tự trang trí, **và** huy chương, đồ vật đổi kiểu theo tiến độ |
| Gói bài hiện có | Nằm trong **Vở (ôn tập)** |
| Thi đấu | Chỉ là lối vào giữ chỗ ("sắp có") |
| Trang trí bản đầu | Màu tường / giấy dán tường; chọn con vật nuôi; đổi đồ trang trí trên bàn; **kéo thả đồ vật tự do** |
| Mốc đầu | Phòng + điều hướng. Ghi chú, huy chương, trang trí làm ở mốc sau, khi ADR 0014 được duyệt |

## 3. Mục tiêu, không làm, phản mục tiêu

**Mục tiêu của mốc 1** (phạm vi của bản thiết kế này):

1. Mở trang là vào phòng học 3D lofi: bàn, tường, cửa sổ có trời theo giờ thật, một con vật nuôi.
2. Bấm vở, máy tính, cúp → camera bay tới → chuyển mượt sang trang khu.
3. Vở dẫn tới các gói bài hiện có. Máy tính và cúp dẫn tới trang "sắp có".
4. Tường có bảng ghi chú và kệ huy chương ở trạng thái "chưa có gì" nhưng không trống trải. Chưa tương tác được.
5. Dùng được bằng bàn phím và trên điện thoại 360 px. Có danh sách chữ các khu cho lúc 3D đang tải hoặc không chạy được.
6. Nút "quay về phòng" và nút Back của trình duyệt đều đưa về phòng.

**Không làm ở mốc 1** (đã có chỗ trong mốc sau, mục 9): ghi chú tương tác, huy chương thật, trang trí, kéo thả, lưu tiến độ, nội dung khu Máy tính và khu Cúp.

**Không làm trong cả dự án này:** thi đấu với người khác, máy chủ, tài khoản, nhạc nền, âm thanh. Kiểu dáng trang bên trong các khu (giấy, màu, chữ) là một thiết kế riêng, làm sau (PR #11 / ADR 0015 để nguyên, chưa đóng).

**Phản mục tiêu** — làm đúng kỹ thuật mà vẫn là thất bại nếu:

- học sinh phải chờ màn trắng hoặc chờ animation xong mới bấm được;
- không đoán được vật nào bấm được;
- máy yếu hoặc điện thoại giật tới mức không vào được bài;
- phòng trông như ghép từ nhiều bộ đồ họa khác nhau (mô hình CC0 lệch phong cách);
- vào được bài nhưng mất đường về phòng.

## 4. Các cách đã cân nhắc

| | Cách 1: Bevy riêng + Leptos điều hướng | Cách 2: ảnh dựng sẵn + điểm bấm | Cách 3: toàn bộ bằng Bevy |
|---|---|---|---|
| Ý | Phòng là gói wasm Bevy nạp lười; báo "mở khu X" cho vỏ Leptos | Dựng cảnh trong Blender, xuất ảnh/video lặp; Leptos vẽ vùng bấm đè lên | Bỏ Leptos, giao diện bằng `bevy_ui` |
| Được | Trang trí, kéo thả, trời theo giờ đều làm được; đã có spike chứng minh | Rất nhẹ; đúng chất lofi nhất | Chuyển cảnh liền mạch nhất |
| Mất | Thêm 3–4 MB tải; cần crate mới | Mỗi màu tường, mỗi giờ trời, mỗi vị trí đồ là một lần dựng lại | Ngược ADR 0008; chữ khó đọc, khó dùng bàn phím; viết lại mọi trang bài |
| | **Chủ dự án chọn** | Loại: không hợp trang trí/kéo thả/giờ thật | Loại |

## 5. Ràng buộc

- **ADR 0001:** toàn bộ bằng Rust. Không three.js. Cầu nối JS chỉ là vài dòng trong `index.html`, như spike.
- **ADR 0002:** phòng là **điều hướng**, không phải chức năng học. Vẫn có lối thay thế không cần 3D (danh sách chữ), để không chức năng nào chỉ có ở 3D.
- **ADR 0005 (sau khi sửa):** không CDN, không tài nguyên bên thứ ba lúc chạy. Mọi mô hình, kết cấu, font nằm trong repo và được phục vụ cùng trang.
- **ADR 0006:** crate mới phải vào bảng ADR 0006 và `xtask/src/deps.rs` trong cùng thay đổi.
- **ADR 0008:** Bevy 0.20.1, WebGL2, gói wasm riêng, nạp lười. Ngưỡng: gói Bevy ≤ 5 MB brotli; khung 3D đầu ≤ 10 s trên điện thoại tầm trung qua 4G.
- **ADR 0013:**
  - animation không bao giờ chặn đầu vào;
  - tôn trọng `prefers-reduced-motion`;
  - vùng bấm ≥ 24 px ở 360 px, nút ≥ 44 px;
  - màu trong phần Leptos chỉ là biến CSS.
- **ADR 0010:** "phòng", "khu", "ôn tập", "thi đấu" là khái niệm **của vỏ**, không được đi vào `kernel`, `registry`, `content`.
- **Giấy phép:** chỉ dùng mô hình CC0 (hoặc tương đương không ràng buộc). Ghi nguồn từng file.

## 6. Thiết kế mốc 1

### 6.1 Các phần và ranh giới

```
crates/room-model   (thuần, mới)  bảng chỗ đặt đồ, khu, thông điệp cầu nối
crates/room3d       (Bevy, mới)   cảnh 3D, camera, chọn vật; build thành wasm riêng
crates/web          (Leptos)      định tuyến, trang khu, danh sách chữ, chuyển cảnh, cầu nối
```

- **`room-model`** — crate thuần, chỉ phụ thuộc `serde`. Chứa:
  - `Zone`: enum đóng `Review` (vở, ôn tập), `Courses` (máy tính), `Contests` (cúp). Mốc sau thêm `Notes`, `Trophies`.
  - `Slot`: một chỗ đặt đồ trong phòng: mã, vị trí/hướng (số nguyên, đơn vị cm, để tất định), loại đồ vật, và `Option<Zone>` (đồ trang trí thì `None`).
  - `ROOM`: bảng các `Slot` mặc định, viết bằng Rust (const), là **nguồn duy nhất** cho "vật nào dẫn vào khu nào". Cả `web` (danh sách chữ, định tuyến) và `room3d` (đặt mô hình, chọn vật) đọc cùng bảng này.
  - Thông điệp cầu nối (`ToRoom`, `FromRoom`, mục 6.4), serde để đổi qua JSON.
  - Không biết Bevy, Leptos hay nội dung học.
- **`room3d`** — `cdylib` wasm, phụ thuộc `room-model` và `bevy` (bộ feature của spike, thêm `bevy_gltf`, `bevy_animation`, `bevy_scene`, `png`/`ktx2` tùy định dạng kết cấu). Không phụ thuộc `kernel`, `kind-*`, `content`, `web`.
- **`web`** — thêm phụ thuộc `room-model`. Không phụ thuộc `room3d` (hai gói wasm riêng, chỉ nói chuyện qua cầu nối).

Cập nhật ADR 0006: thêm hai dòng `room-model` (thuần, phụ thuộc: —) và `room3d` (trình duyệt, phụ thuộc: `room-model`). Dòng "sẽ thêm sau: `table3d`" giữ nguyên: `table3d` là bộ hiển thị 3D cho *hoạt động*, khác phòng.

### 6.2 Định tuyến

Dùng hash trong URL để nút Back và tải lại trang hoạt động đúng, không cần máy chủ hỗ trợ (GitHub Pages):

| Hash | Màn |
|---|---|
| `#/` hoặc trống | Phòng |
| `#/on-tap` | Khu Vở: danh mục các gói (màn `catalog` hiện có) |
| `#/on-tap/<mã hoạt động>` | Một hoạt động (màn `activity` hiện có) |
| `#/khoa-hoc` | Khu Máy tính: trang "sắp có" |
| `#/thi-dau` | Khu Cúp: trang "sắp có" |

Hash không hợp lệ → về phòng. Nút "←" ở thanh tiêu đề đi lên một cấp (hoạt động → khu → phòng).

### 6.3 Trang chủ: phòng và lối thay thế

Thứ tự hiện khi mở trang:

1. **Ngay lập tức (vỏ Leptos, < 1 s):** khung trang với **danh sách chữ các khu** (Vở – Ôn tập, Máy tính – Khoá học, Cúp – Thi đấu), mỗi mục là nút ≥ 44 px. Phía sau là một ảnh tĩnh nhỏ của phòng (SVG hoặc WebP ≤ 60 KB, tự chứa) làm nền chờ, kèm chỉ báo "Đang dọn phòng…".
2. **Song song:** nạp `room3d` bằng `import()`.
3. **Khi có khung 3D đầu:** canvas mờ dần lên thay ảnh tĩnh. Danh sách chữ thu lại thành một nút "Danh sách khu" ở góc. Nó vẫn nằm trong cây truy cập, và vẫn đi tới được bằng phím Tab.

Nếu WebGL2 không có, hoặc nạp lỗi, hoặc người dùng chọn "Phòng đơn giản": giữ nguyên bước 1. Lựa chọn này nhớ trong `localStorage` (chỉ một cờ, không phải dữ liệu học tập).

### 6.4 Cầu nối vỏ ↔ phòng

Như spike `leptos-bevy`: vài dòng JS trong `index.html`, Rust ở hai phía gọi qua `wasm_bindgen`. Thông điệp là JSON của kiểu trong `room-model`:

- `ToRoom`:
  - `Init { hour, quality, reduced_motion, viewport }`;
  - `SetHour(u8)` (vỏ gửi mỗi phút);
  - `Resize { width, height }`;
  - `Suspend` / `Resume` (rời phòng thì dừng vẽ để đỡ tốn pin);
  - `ReturnFrom(Zone)` (camera đặt sẵn gần vật rồi lùi về góc tổng);
  - `Highlight(Option<SlotId>)` (focus bàn phím trong danh sách chữ);
  - `Go(Zone)` (bấm mục trong danh sách chữ khi 3D đang chạy: bay như bấm vật).
- `FromRoom`:
  - `FirstFrame`;
  - `Hover(Option<SlotId>)` (vỏ hiện nhãn tên khu);
  - `Open(Zone)` (lúc *bắt đầu* bay, để vỏ nạp trang khu song song);
  - `Arrived(Zone)` (lúc bay xong, để vỏ chuyển cảnh);
  - `Failed(String)`.

Giờ trong ngày do **vỏ** đọc từ đồng hồ máy rồi gửi vào, nên `room3d` không tự đọc đồng hồ. Điều này giúp thử các giờ khác nhau dễ.

### 6.5 Cảnh

- **Bố cục:** góc phòng nhìn chéo kiểu isometric. Bàn học ở phía trước. Tường sau có cửa sổ, bảng ghi chú (bần), kệ huy chương. Có kệ sách, đèn bàn, cây, đèn dây.
- **Trên bàn:**
  - ba vật lối vào: vở, máy tính, cúp;
  - 3–5 chỗ đồ trang trí: đèn bàn, chậu cây, ống bút, cốc có hơi bốc, chồng sách.

  Vị trí theo bảng `ROOM`.
- **Ánh sáng lofi:**
  - một đèn bàn vàng ấm là nguồn chính, ánh trời qua cửa sổ là nguồn phụ;
  - tông màu ấm, tương phản thấp, có bloom nhẹ ở mức chất lượng cao;
  - tonemapping `AcesFitted` như spike, để không phải kéo LUT.
- **Trời theo giờ:** 4 trạng thái nội suy theo giờ thật: sáng (6–10), trưa (10–16), chiều tà (16–19), đêm (19–6). Mỗi trạng thái gồm:
  - màu trời trên tấm phẳng sau cửa sổ;
  - màu và cường độ ánh trời;
  - đèn bàn sáng hơn về đêm;
  - đêm có vài ngôi sao hoặc ánh đèn thành phố xa.
- **Con vật nuôi:** một con mèo CC0 có sẵn hoạt ảnh. Mốc 1 chỉ có các trạng thái nằm ngủ, thở, thỉnh thoảng vẫy đuôi hoặc ngẩng đầu. Không di chuyển, không bấm được.
- **Chuyển động nhỏ khác:** hơi nước từ cốc, rèm khẽ lay. Tất cả tắt khi `reduced_motion`.
- **Đồng bộ phong cách:** mọi mô hình CC0 phải qua một bước chỉnh:
  - thay vật liệu bằng một bảng màu chung của phòng (một texture bảng màu dùng chung, hoặc màu phẳng theo vật liệu);
  - bỏ kết cấu gốc lệch tông;
  - cùng mức chi tiết.

  Bảng màu của phòng ghi trong `crates/room3d/README.md`.
- **Tường khi chưa có gì (mốc 1):**
  - bảng ghi chú có sẵn 2–3 tờ note *in sẵn* (không phải chữ của học sinh), ví dụ "Chào mừng tới phòng học!" và "Bấm vào vở để ôn tập", cùng vài cái ghim;
  - kệ huy chương có các móc trống, mỗi móc treo một **bóng mờ** của huy chương chưa mở, và một tấm vé "Ngày đầu tới lớp" luôn được treo sẵn.

  Trông có chủ đích, không trống.

### 6.6 Tương tác và camera

- **Máy tính:**
  - rê chuột vào vật lối vào → vật **sáng viền** (outline hoặc emissive nhẹ) và nổi lên 1–2 cm, kèm con trỏ tay;
  - vỏ hiện nhãn "Vở · Ôn tập" gần con trỏ.
- **Điện thoại:** chạm lần đầu = như rê chuột (sáng lên + nhãn); chạm lần hai = vào. Như vậy tránh vào nhầm.
- **Vùng bấm:** mỗi vật lối vào có một khối va chạm vô hình lớn hơn mô hình. Ở 360 px, hình chiếu của khối này phải ≥ 44 × 44 px (test, mục 8).
- **Bay tới vật:**
  - camera nội suy vị trí và hướng tới điểm nhìn đặt riêng cho từng vật, trong 700 ms, đường cong ease-in-out;
  - lúc bắt đầu bay, phòng gửi `Open(zone)`; lúc tới nơi gửi `Arrived(zone)`;
  - trong lúc bay vẫn nhận đầu vào: bấm vật khác → đổi đích, không phải chờ; bấm Esc hoặc nút Back → huỷ, lùi về.
- **Bàn phím:** các vật lối vào nằm trong danh sách chữ của vỏ. Tab vào danh sách; focus một mục thì phòng làm vật đó sáng lên (`ToRoom::Highlight`); Enter = bay tới (`ToRoom::Go`).
- **Góc camera:**
  - một góc cố định cho màn ngang;
  - một góc khác cho màn dọc, gần hơn và nghiêng hơn để bàn chiếm phần lớn màn hình;
  - chọn theo tỉ lệ khung hình, đổi khi xoay máy.

### 6.7 Chuyển cảnh sang khu

1. **Bấm vật:** camera bắt đầu bay. Vỏ nhận `Open` và bắt đầu dựng trang khu ở nền, chưa hiện.
2. **Camera tới gần vật:**
   - vỏ phủ một lớp màu chủ đạo của khu, mờ dần từ 0 lên 100 % trong khoảng 250 ms cuối của lượt bay (vở màu giấy, máy tính xanh màn hình, cúp vàng);
   - nếu trang khu chưa sẵn sàng, lớp phủ giữ lại với **hiệu ứng chờ** riêng của khu: trang vở lật, con trỏ nhấp nháy trên màn máy tính, cúp lấp lánh.
3. **Trang khu sẵn sàng:** lớp phủ mờ dần đi (200 ms), lộ trang khu. Phòng nhận `Suspend`.
4. **Quay về:**
   - trang khu mờ đi thành lớp phủ;
   - phòng nhận `Resume` và `ReturnFrom(zone)`;
   - camera đặt sẵn gần vật rồi lùi về góc tổng.

- **Giảm chuyển động:** bỏ bay camera và hiệu ứng chờ động; chỉ còn lớp phủ mờ 150 ms.
- **Không chặn đầu vào:** Esc/Back lúc nào cũng có tác dụng. Lớp phủ không bao giờ quá 3 s mà không có chữ "Đang mở…".
- **Không có 3D:** bấm mục trong danh sách chữ → chỉ có lớp phủ mờ.

### 6.8 Chất lượng hình và hiệu năng

| Mức | Khi nào | Bật |
|---|---|---|
| Cao | Màn ≥ 1024 px rộng và không phải thiết bị cảm ứng | Bóng mềm từ đèn bàn, bloom nhẹ, MSAA 4× |
| Thấp | Còn lại, hoặc khung hình < 30 fps trong 3 s đầu | Không bloom, bóng cứng hoặc không bóng, MSAA tắt, giảm độ phân giải canvas về 0,75× |

Người dùng đổi được mức bằng một nút trong menu phòng. Lựa chọn nhớ trong `localStorage`.

**Ngân sách** (ghi vào `crates/room3d/README.md`, đo bằng tay ở mốc 1; máy kiểm thuộc việc "Ngân sách kích thước wasm" trong STATUS):

- `room3d_bg.wasm` ≤ 5 MB brotli (ADR 0008);
- mô hình + kết cấu ≤ 3 MB tổng sau nén;
- trang Leptos vẫn ≈ 100 KB như nay, vì danh sách chữ hiện mà không chờ 3D;
- khung 3D đầu ≤ 3 s trên máy tính qua Wi-Fi; ≤ 10 s trên điện thoại tầm trung qua 4G.

### 6.9 Tài nguyên 3D

- Đặt ở `crates/room3d/assets/`, định dạng `.glb`. Build sao chép sang `dist/room/`.
- `crates/room3d/assets/NGUON.md`: mỗi file ghi tên gốc, tác giả, link, giấy phép (CC0), và đã chỉnh gì.
- Kết cấu nén (KTX2 hoặc PNG nhỏ ≤ 512 px). Chọn định dạng nào là việc của người làm, miễn đạt ngân sách.
- Không tải gì từ máy chủ khác lúc chạy.

### 6.10 Build và triển khai

- `crates/room3d/build.sh` (giống `crates/web/build.sh`) build ra `crates/web/dist/room/`. Profile release: `opt-level=3` cho crate cảnh (spike `lang-map`: `opt-level="z"` làm khung đầu chậm 16 lần).
- `crates/web/build.sh` gọi build phòng, trừ khi có `--no-room` để lặp nhanh phần 2D.
- `.github/workflows/pages.yml`: build và đẩy cả `app/room/`. Build Bevy lần đầu ~6 phút, cần ~3,5 GB RAM (đủ cho máy GitHub).
- `cargo xtask ci`:
  - kiểm `room-model` như crate thuần (test, clippy);
  - kiểm `room3d` bằng `cargo clippy --target wasm32-unknown-unknown` và `cargo test` (phần logic không cần GPU);
  - không chạy build release đầy đủ trong `ci` (quá lâu).

## 7. Thay đổi tài liệu đi cùng

- **ADR 0016 (mới):** "Trang chủ là phòng học 3D, phòng là điều hướng". Ghi:
  - cách 1 và hai cách đã loại;
  - phòng không chứa chức năng học (ADR 0002);
  - "khu" là khái niệm của vỏ (ADR 0010);
  - bảng `ROOM` là nguồn duy nhất;
  - hai crate mới.
- **ADR 0005:** bỏ cụm "hay ô nhập chữ tự do cho trẻ" ở mục 2. Thêm dòng "Sửa đổi 2026-11-27: chủ dự án bỏ quy tắc này để có bảng ghi chú". Bỏ "chat" khỏi danh sách bị chặn thì **không**: chat là chuyện khác, có người nhận.
- **ADR 0006 và `xtask/src/deps.rs`:** thêm `room-model`, `room3d`.
- **`AGENTS.md`:** dòng "Không thêm trường thông tin cá nhân… chữ tự do của trẻ" sửa cho khớp ADR 0005 mới.
- **`docs/glossary.md`:** thêm "phòng" (Room), "khu" (Zone), "chỗ đặt đồ" (Slot).
- **`docs/STATUS.md`:** mục "Đang làm" cho việc này; mốc 2, 3 vào "Tiếp theo".
- **`crates/room3d/README.md`:** bảng màu, ngân sách, cách build, cách thay mô hình.

## 8. Kiểm thử

- **`room-model` (test đơn vị):**
  - mã `Slot` không trùng;
  - mỗi `Zone` của mốc 1 có đúng một `Slot` dẫn tới;
  - JSON của `ToRoom`/`FromRoom` đi–về không đổi.
- **`web`:**
  - phân tích hash → màn, kể cả hash rác về phòng;
  - "←" đi lên đúng một cấp;
  - danh sách chữ có đủ khu;
  - nút ≥ 44 px ở 360 px (cách đo như ADR 0013).
- **`room3d` (không cần GPU):**
  - nội suy trời theo giờ (0, 6, 12, 18, 23 giờ ra trạng thái đúng);
  - đổi đích giữa lúc bay;
  - hình chiếu khối va chạm của mỗi vật lối vào ≥ 44 × 44 px ở viewport 360 × 740 và 1280 × 720 với góc camera tương ứng (tính bằng toán chiếu, không cần vẽ).
- **Thử tay (ghi vào mô tả PR):**
  - Chrome máy tính 1280 px và 360 px;
  - một điện thoại thật nếu có;
  - bật giảm chuyển động;
  - tắt WebGL (`--disable-webgl`) → còn danh sách chữ;
  - Back của trình duyệt từ trang hoạt động → khu → phòng;
  - đo kích thước và thời gian khung đầu.

## 9. Mốc sau (chỉ phác, có thiết kế riêng khi tới lượt)

- **Mốc 2 — phòng của tôi**, cần ADR 0014 được duyệt:
  - lưu tiến độ trên máy;
  - bảng ghi chú: học sinh tạo, sửa, xoá, kéo tờ note; chữ chỉ lưu trên máy;
  - kệ huy chương: huy chương thật thay bóng mờ;
  - đồ vật đổi kiểu theo tiến độ (cúp đồng → bạc → vàng, cây lớn dần).
- **Mốc 3 — trang trí:**
  - màu tường / giấy dán tường;
  - chọn con vật nuôi;
  - đổi đồ trang trí trên bàn;
  - kéo thả đồ vật tự do. Bảng `ROOM` thành bố cục mặc định; bố cục của học sinh lưu trên máy như một danh sách `Slot` đã dời.
- **Khu Máy tính** (khoá học, bài giảng) và **khu Cúp** (thi đấu): mỗi khu một thiết kế riêng.

## 10. Để lại cho người làm

- Chọn mô hình CC0 cụ thể cho từng vật, miễn đạt ngân sách và đồng bộ phong cách. Chủ dự án duyệt ảnh chụp phòng trước khi merge.
- Cách vẽ sáng viền khi rê chuột (outline shader hay emissive).
- Định dạng kết cấu; cách tạo ảnh tĩnh của phòng cho lúc chờ (chụp từ Bevy là đủ).
- Chi tiết hiệu ứng chờ từng khu, trong khuôn khổ mục 6.7.
- Tên crate và tên kiểu có thể đổi nếu glossary có từ tốt hơn, nhưng phải đổi cùng lúc ở ADR 0006, `deps.rs`, glossary.

## 11. Quyết định Claude tự đưa ra (chủ dự án kiểm khi đọc)

1. Bảng `ROOM` viết bằng Rust trong crate thuần `room-model`, không phải file TOML trong `content/`. Lý do:
   - `content` không được biết khái niệm của vỏ (ADR 0010);
   - crate thuần không được thêm thư viện đọc TOML;
   - bảng Rust thì trình biên dịch kiểm.
2. Định tuyến bằng hash URL, để Back và tải lại trang chạy được trên GitHub Pages.
3. Danh sách chữ các khu hiện ngay khi mở trang, và luôn có mặt; 3D phủ lên khi sẵn sàng.
4. Điện thoại: chạm lần một làm vật sáng lên, chạm lần hai mới vào.
5. Chia hai mức chất lượng, tự hạ khi < 30 fps.
6. Thời lượng: bay 700 ms, lớp phủ 250 ms, lộ trang 200 ms, giảm chuyển động 150 ms.
7. Mốc 1 treo sẵn note in sẵn và vé "Ngày đầu tới lớp" để tường không trống.
8. Màu khu Cúp là vàng, khu Máy tính xanh màn hình, khu Vở màu giấy (chỉ cho lớp phủ chuyển cảnh).
9. Tự hạ chất lượng: chỉ đo một lần, là trung bình thời gian khung trong 3 s sau khung đầu. Dưới 30 fps thì hạ một lần. Không tự nâng lại, chỉ người dùng đổi bằng tay.
10. "Khung 3D đầu" (`FirstFrame`, ngân sách 6.8) tính từ khi tường, bàn, cửa sổ và ba vật lối vào đã nạp xong và bấm được. Đồ trang trí và con mèo được phép nạp sau, hiện dần.
11. Toạ độ trong bảng `ROOM` theo Bevy: tay phải, trục Y hướng lên, gốc ở góc sàn chỗ hai bức tường gặp nhau. Hướng chỉ là góc xoay quanh Y, tính bằng độ nguyên.
12. `viewport` trong `Init` là `{ width, height }` theo pixel CSS. Vỏ gửi lại mỗi khi đổi cỡ (`ToRoom::Resize`).
13. Trang khu sẵn sàng trước khi camera tới: vẫn chạy đủ lớp phủ (250 ms vào, 200 ms ra) để nhịp chuyển cảnh luôn như nhau.
