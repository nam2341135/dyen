# ADR 0013: Quy ước chung cho bộ hiển thị — vùng chạm, animation, kiểu dáng

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26
- **Bổ sung:** [ADR 0002](0002-mot-mo-hinh-nhieu-bo-hien-thi.md)

## Bối cảnh

Vỏ `web` đã có ba bộ hiển thị (`bars`, `choice`, `section`), và sẽ có thêm một bộ cho mỗi loại hoạt động mới. Chưa nên thiết kế bố cục hay nhận diện hình ảnh, vì còn chưa biết ai dùng và bài làm lưu ở đâu. Nhưng có ba điều mà bộ hiển thị nào cũng gặp. Nếu không chốt bây giờ, mỗi bộ sẽ tự làm một kiểu, và về sau phải sửa tất cả.

1. **Vùng chạm.** Người dùng chính là trẻ em, dùng điện thoại. Đo ở bề rộng 360 px (vùng vẽ rộng 328 px):

   | Bộ hiển thị | Mục tiêu | Kích thước thật |
   |---|---|---|
   | `section` | điểm bấm trên hình lập phương | 15 px (các điểm cách nhau ≥ 40 px) |
   | `bars` | dải cắt / thanh / vạch cắt | 14 / 20 / 9 px |
   | mọi loại | nút HTML | cao 40 px |

   Dải cắt của `bars` từng chỉ cao 7 px, và phát hiện ra chỉ nhờ tình cờ kiểm tay.

2. **Animation.** ADR 0002 đã cho phép tiến độ animation là trạng thái riêng của bộ hiển thị, và yêu cầu mọi hiệu ứng phải là chuyển tiếp giữa hai trạng thái. ADR 0002 chưa nói animation có được làm chậm đầu vào hay không. Đây là chỗ nguy hiểm: dấu vết ghi `at_ms` (ADR 0003), và có phát hiện dựa vào thời gian. Nếu bộ hiển thị chờ animation chạy xong rồi mới nhận cú chạm tiếp theo, thời gian trong dấu vết đo cả animation chứ không chỉ người học.

3. **Kiểu dáng.** Màu đang nằm rải trong `crates/web/index.html`: một phần dùng biến (`--accent`, `--ok`…), một phần là mã hex (màu mảnh của `bars`, nền vùng cắt của `section`). Muốn đổi giao diện sau này thì phải biết chỗ nào đặt màu.

## Quyết định

### 1. Vùng chạm

- Nút và các điều khiển HTML khác: tối thiểu **44 × 44 px CSS**.
- Mục tiêu bấm trong hình (SVG, sau này cả 3D): tối thiểu **24 × 24 px CSS**, đo ở bề rộng màn hình 360 px. Đây là mức WCAG 2.2, tiêu chí 2.5.8 (AA).
- Vùng bấm được phép lớn hơn hình vẽ. Ví dụ: vẽ chấm nhỏ, phía trên phủ một hình trong suốt đủ lớn để bấm. Hai vùng bấm không được chồng lên nhau. Nếu dữ liệu khiến hai mục tiêu nằm quá gần thì đó là lỗi bố cục, cần sửa cách vẽ; không được thu nhỏ vùng bấm.
- Mỗi bộ hiển thị có một test đơn vị, tính kích thước vùng bấm ra px ở bề rộng 360 px từ hằng số bố cục của nó và `viewBox`.

### 2. Animation

- Animation chỉ là trạng thái hiển thị, suy ra từ trạng thái trước và sau một `Input` (đúng như ADR 0002). Không ghi vào dấu vết.
- **Không bao giờ chặn đầu vào.** Cử chỉ thành `Input` ngay lúc người học chạm, và `at_ms` là thời điểm chạm. Không có hàng đợi, không khoá nút trong lúc animation chạy. Có đầu vào mới giữa chừng thì animation đang chạy nhảy thẳng tới trạng thái mới.
- Ngắn: tối đa **300 ms** cho phản hồi một thao tác. Hiệu ứng dài hơn (ví dụ mở đầu một màn) phải bỏ qua được bằng một lần chạm, và vẫn không chặn đầu vào.
- Khung hình cuối cùng (tĩnh) phải chứa đủ mọi thông tin. Không có thông tin nào chỉ hiện trong lúc chuyển động.
- Tôn trọng `prefers-reduced-motion: reduce`: khi bật thì không chuyển động, trạng thái đổi ngay. Stylesheet chung tắt `transition`/`animation` toàn cục trong media query này.
- Dùng CSS transition hoặc animation của SVG trước. Không thêm thư viện animation (CLAUDE.md: không thêm framework). 3D (Bevy) theo cùng các quy tắc trên.

### 3. Kiểu dáng

- Bộ hiển thị (code Rust) chỉ sinh cấu trúc, tên lớp (`class`) và toạ độ hình học. Không có thuộc tính `style=`, không đặt màu, phông chữ hay cỡ chữ trong code.
- Mọi màu nằm trong stylesheet chung, dưới dạng biến CSS ở `:root`. Màu riêng của một loại vẫn là biến, đặt tên theo loại (ví dụ `--bars-c0`). Đổi giao diện = sửa một khối biến.
- Không dùng màu làm cách duy nhất để báo nghĩa (WCAG 1.4.1). Đã chọn, đã đặt, đúng, sai… phải khác nhau thêm ở ít nhất một điểm ngoài màu: nét viền, ký hiệu hoặc chữ.

## Hệ quả

- Mỗi bộ hiển thị mới có danh sách kiểm sẵn, và test kích thước bắt lỗi kiểu dải cắt 7 px trước khi lên điện thoại.
- Dấu vết giữ đúng nghĩa: thời gian là thời gian của người học, không lẫn thời gian chạy animation.
- Thiết kế giao diện sau này chỉ cần sửa stylesheet, không phải sửa từng bộ hiển thị.
- Mất: hình phải chừa chỗ cho vùng bấm. `bars` phải cao lên khoảng gấp đôi trên điện thoại, nên hình chiếm nhiều chỗ hơn. Không làm được hiệu ứng "chờ xem xong rồi mới cho bấm".
- **Đã làm khi chấp nhận** (2026-11-26):
  - `button` `min-height` 40 → 44 px.
  - `section`: vùng bấm trong suốt bán kính 15 (≈ 24,6 px), chấm vẽ giữ nguyên cỡ.
  - `bars`: thanh và dải phía trên cao 44 (≈ 24 px). Vạch chia / vết cắt bấm bằng nút tròn trên dải đó, không bấm trên thân thanh, để vùng bấm không chồng lên mảnh. Hai vạch gần nhau chia đôi khoảng giữa. Vết cắt tự do sát nhau vẫn có thể cho vùng bấm < 24 px; đó là do người học tạo, không phải do bố cục.
  - Lựa chọn đã chọn của `choice` có thêm dấu ✓ và viền dày, không chỉ khác màu.
  - Chuyển các mã hex còn lại sang biến CSS.
  - Thêm media query `prefers-reduced-motion`.
  - Thêm test kích thước cho từng bộ hiển thị.
  - Thêm một dòng tóm tắt vào `CLAUDE.md`, mục "Quy tắc logic".
- Xem lại khi: có người dùng thật với nhu cầu khác (ví dụ trẻ nhỏ hơn cần vùng bấm 44 px cả trong hình), hoặc có bộ hiển thị 3D (vùng chạm trong không gian 3D phải đo theo khoảng cách camera).
