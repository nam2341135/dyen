# Spike: map làng ven sông (thiết kế)

- **Trạng thái:** Chờ duyệt
- **Ngày:** 2026-11-26
- **Loại:** spike, vứt đi được (xem `spikes/README.md`). Không nằm trong workspace chính, không crate nào phụ thuộc.

## Mục đích

Chủ dự án muốn biết **trần chất lượng** và **tính khả thi**: với stack đã đề xuất (Rust + Bevy chạy trên trình duyệt, ADR 0001, 0008), có dựng nổi một cảnh làng Việt sống động, có chiều sâu, chạm được cảm xúc hay không, và cái giá phải trả là bao nhiêu (dung lượng tải, thời gian khung đầu, khung hình/giây). Kết quả dùng để nghiên cứu cho các sản phẩm sau, như Làng Thợ (`docs/ideas/lang-tho.md`).

Phong cách và nội dung cảnh do AI tự chọn (chủ dự án giao).

## Cảnh

**"Chiều về trên làng ven sông."** Một thung lũng nhỏ, máy quay nhìn từ trên đê xuống.

- **Tiền cảnh:** ao sen (lá, búp, hoa), lau sậy ven bờ, vài cành tre lấp ló sát ống kính.
- **Trung cảnh, là làng:**
  - Đình làng mái cong, đầu đao vểnh, ngói đỏ son, có sân gạch.
  - Năm đến bảy nhà ba gian, mái ngói hoặc mái rạ, có sân, ngõ và bờ rào.
  - Cây đa cổ thụ cạnh giếng, lũy tre bao quanh làng.
  - Dòng sông uốn khúc với cầu tre và một chiếc thuyền nan trôi chậm.
  - Ruộng lúa chia ô có bờ, một con đê đất chạy dọc sông.
- **Hậu cảnh:** ba đến bốn lớp đồi núi nhạt dần trong sương, như tranh lụa; mây trôi.
- **Bầu trời:** dải màu theo giờ, có mặt trời, mặt trăng và sao.

**Cảm xúc muốn có:** bình yên, hoài niệm, ấm áp. Cảm giác "chiều về, khói bếp, đèn thắp".

## Chuyển động và thời gian

- **Vòng ngày đêm khoảng 3 phút**, có thể tua: trưa → hoàng hôn → đêm → bình minh.
  - Màu trời, màu sương và hướng nắng đổi liên tục theo giờ.
  - Đêm xuống thì đèn lồng ở đình và cửa sổ các nhà sáng dần (vật liệu phát sáng + bloom), đom đóm bay trên ruộng và ao.
- **Gió:** shader đỉnh (vertex shader) làm tre, lúa, lau và tán cây lay theo từng đợt. Biên độ khác nhau theo độ cao, pha lệch theo vị trí để không lay đồng loạt.
- **Nước:** sóng lăn tăn trên mặt sông và ao. Màu nước phản chiếu màu trời (fresnel) và đổi theo giờ. Lá sen dập dềnh nhẹ.
- **Sự sống:**
  - Khói bếp bay lên từ vài nóc nhà vào buổi chiều.
  - Đàn cò bay ngang trời.
  - Thuyền nan trôi theo sông.
  - Một cánh diều chao trên đồng.
  - Hai ba người nhỏ (dáng đơn giản, không mặt) đi trên đê; một người gánh hàng.
- **Máy quay:**
  - Mở đầu tự bay lướt giới thiệu khoảng 10 giây, có thể bỏ qua.
  - Sau đó xoay, kéo, phóng to bằng chuột và cảm ứng, có giới hạn góc để không chui xuống đất.

## Phong cách

- **Low-poly cách điệu.** Mọi hình khối sinh bằng code; không có model, texture hay âm thanh từ ngoài.
- **Bảng màu ấm:**
  - đỏ son mái đình, nâu gỗ, vàng rơm;
  - xanh tre, xanh lúa non;
  - trời cam hồng lúc hoàng hôn, xanh chàm lúc đêm.
- **Chiều sâu:**
  - Sương theo khoảng cách, nhuộm màu trời, để các lớp núi nhạt dần.
  - Tiền cảnh đậm và tương phản cao, hậu cảnh nhạt.
  - Ánh sáng định hướng có bóng đổ, cộng ánh sáng môi trường theo giờ.
- **Không bắt chước một trò chơi cụ thể nào.** Tinh thần gợi từ làng quê Bắc Bộ và tranh lụa.

## Cách dựng (đã chọn)

| Cách | Được | Mất | Chọn? |
|---|---|---|---|
| **A. Bevy 3D trong spike** | Đo đúng stack dự định dùng (ADR 0008); kết quả trả lời thẳng câu hỏi khả thi; tận dụng `build.sh` và công cụ wasm sẵn có | Biên dịch chậm; WebGL2 giới hạn (không compute shader, ít đèn điểm) | **Có** |
| B. Bevy 2.5D, các lớp tranh phẳng chồng thị sai | Rất nhẹ, dễ đẹp kiểu tranh | Không trả lời được câu hỏi 3D. Sản phẩm cần xoay khối hình học thật | Không |
| C. Three.js / Babylon.js | Dựng đẹp nhanh hơn | Trái ADR 0001 (toàn Rust), kết quả không dùng để đánh giá stack thật được | Không |

**Chi tiết kỹ thuật:**
- Bevy `=0.20.1`, chạy **bản máy tính (native)** để lặp nhanh và **bản web (WebGL2)** để đo.
- Shader WGSL riêng cho gió và nước.
- Hậu kỳ: bloom, tonemapping, sương khoảng cách, khử răng cưa.
- Rải cây, lúa và nhà bằng bộ sinh số ngẫu nhiên có seed tự viết, nên cảnh giống nhau mỗi lần mở.
- Góc màn hình có dòng đo: FPS, thời gian khung đầu, giờ trong ngày.

**Ranh giới:**
- Thư mục `spikes/lang-map`, workspace Cargo riêng.
- Không phụ thuộc crate nào của dự án; spike này chỉ đo hiển thị.
- Không analytics, không CDN, không tải gì từ mạng lúc chạy.

## Đo và báo cáo

- **Dung lượng:** wasm sau `wasm-opt -Oz` và brotli.
- **Khởi động:** thời gian tải + khởi tạo wasm, thời gian tới khung đầu.
- **Tốc độ khung hình:**
  - FPS trên máy này: bình thường, và khi giới hạn CPU 4× trong DevTools.
  - Có một nút giảm chất lượng (tắt bóng, bloom, giảm mật độ cây) để biết chi phí của từng thứ.
- **Hình ảnh:** ảnh chụp ở 4 thời điểm (trưa, hoàng hôn, đêm, bình minh).
- **Ghi lại:** kết quả và nhận xét "cái gì đẹp được, cái gì đắt, cái gì không nên làm trên điện thoại" ghi vào `spikes/lang-map/README.md`. Con số liên quan ghi thêm vào ADR 0008.

## Không làm

- Không có lối chơi hay logic học tập. Không dùng `kernel` hay `kind-*`.
- Không đưa vào dự án chính, không tạo crate `web` hay `table3d`.
- Không có âm thanh: không có nguồn âm thanh tự làm, và để ngoài phạm vi spike.
- Không tải tài nguyên từ nguồn bên ngoài.

## Thất bại dù vẫn chạy được

- Trông như bản demo kỹ thuật lạnh lẽo: hình khối xám, không có giờ khắc, không có sự sống.
- Đẹp trên máy mạnh nhưng không biết mỗi hiệu ứng tốn bao nhiêu.

## Các vòng dựng

1. **Khung:** dự án spike, địa hình (đê, sông, ruộng), bầu trời, sương, máy quay; chạy được cả native lẫn web.
2. **Làng:** đình, nhà, cầu, cây đa, giếng, lũy tre, lúa, sen.
3. **Sống động:** gió, nước, ngày đêm, đèn, khói, cò, thuyền, diều, người, đom đóm.
4. **Hậu kỳ và tinh chỉnh:** bloom, màu theo giờ, các lớp chiều sâu, máy quay mở đầu; chụp ảnh từng giờ và chỉnh tới khi đẹp.
5. **Đo và ghi kết quả.**

## Để người dựng tự quyết

- Số lượng và vị trí cụ thể của từng vật thể.
- Thông số shader.
- Đường bay của máy quay.
- Cách tối ưu: gộp mesh, instancing, v.v.

## Rủi ro đã biết

- Bevy biên dịch lâu trên Windows; mỗi vòng chỉnh màu sẽ chậm.
- WebGL2 giới hạn số đèn điểm, nên đèn lồng chủ yếu dựa vào vật liệu phát sáng + bloom, không phải đèn thật.
- Đánh giá "đẹp" dựa trên ảnh chụp màn hình; chủ dự án là người chấm cuối.
