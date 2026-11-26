# ADR 0015: Ngôn ngữ thiết kế "giấy và mực", nút nổi

- **Trạng thái:** Đề xuất
- **Ngày:** 2026-11-26
- **Bổ sung:** [ADR 0013](0013-quy-uoc-bo-hien-thi.md)

## Bối cảnh

ADR 0013 chốt quy tắc vùng chạm, animation và cách đặt màu, nhưng cố ý chưa chọn nhận diện hình ảnh. Vỏ web đang dùng màu xám xanh mặc định, không có cá tính, và chưa có bản tối. Chủ dự án yêu cầu khảo sát phong cách và chọn một hướng.

Ràng buộc khi chọn:

- người học từ mầm non đến đại học: không quá "trẻ con", không quá "văn phòng";
- máy yếu: không mờ kính, không bóng nhiều lớp, không ảnh nền;
- không CDN, không tài nguyên bên thứ ba ([ADR 0005](0005-du-lieu-tre-em.md)): không tải font từ Google Fonts;
- nội dung chính là hình vẽ toán: nền phải để hình nổi lên.

Ba hướng đã xem (bản phác ở 360 px):

| | Giấy và mực | Khối nổi vui | Tối giản tông màu |
|---|---|---|---|
| Gần với | sách in, Brilliant | ứng dụng học cho trẻ | Material 3, iOS |
| Hợp lứa tuổi | mọi lứa tuổi | mầm non đến THCS | THPT, đại học |
| Chi phí | thấp, chủ yếu đổi biến | làm lại hình dạng mọi nút | thấp |
| Rủi ro | làm nhạt thì trông cũ | sinh viên thấy trẻ con | không có cá tính |

Đã loại: mờ kính (nặng), tân thô mộc (gắt với trẻ nhỏ), nền tối mặc định (khó in, khó chiếu trên lớp).

## Quyết định

Chủ dự án chọn **"giấy và mực"**, lấy thêm **nút nổi** từ hướng "khối nổi vui".

1. **Màu.**
   - Nền giấy ấm, chữ mực đậm, một màu nhấn xanh mực, đất nung cho số thứ tự.
   - Đúng/sai dùng xanh lá và đỏ gạch dịu, vẫn đi kèm ký hiệu (ADR 0013).
   - Có bản tối theo `prefers-color-scheme: dark`, cùng tên biến, chỉ đổi giá trị.
2. **Màu hình vẽ.**
   - Bốn màu mảnh của `bars` lấy từ bảng Okabe–Ito, an toàn cho người mù màu đỏ–lục.
   - Bản tối dùng sắc sáng hơn của cùng bảng.
3. **Chữ.**
   - Chữ thân dùng `system-ui`.
   - Tiêu đề dùng chuỗi serif có sẵn trên máy: `ui-serif, "Noto Serif", Cambria, "Times New Roman", serif` (macOS/iOS, Android, Windows, dự phòng). Không dùng Georgia: trên Windows, Georgia thiếu glyph dựng sẵn cho dấu chồng (ố, ế), nên "số" hiện thành "sô´".
   - Chưa tự chứa font riêng. Font riêng tốn 20–30 KB, nên phải đi cùng ngân sách kích thước.
4. **Nút nổi.**
   - Mọi `button` (trừ thẻ danh mục và nút "←") có cạnh dưới đặc 3 px, vẽ bằng `box-shadow` không mờ. Cách này rẻ, không phải bóng mờ.
   - Khi nhấn, nút dịch xuống 3 px và mất cạnh. Không đổi bố cục, nên không xô các phần tử khác.
   - Nút hành động chính của một màn (Nộp, Cắt, Xong) có lớp `primary`: nền màu nhấn, chữ sáng. Mỗi màn có tối đa một nút `primary`.
   - Nút bị tắt không có cạnh nổi.
   - Có viền focus 3 px cho bàn phím.
5. **Viền điều khiển.** Viền của nút và lựa chọn dùng `--line-strong`, tương phản ≥ 3:1 với nền (WCAG 1.4.11). Đường kẻ trang trí dùng `--line`, được phép nhạt hơn.

Mọi giá trị là biến CSS trong `crates/web/index.html`. Code Rust chỉ thêm lớp `primary` (ADR 0013, mục 3).

## Hệ quả

- Giao diện có cá tính riêng, hợp mọi lứa tuổi, và hình vẽ toán vẫn là thứ nổi nhất.
- Có bản tối mà không phải sửa bộ hiển thị nào.
- Màu mảnh của `bars` phân biệt được với người mù màu đỏ–lục.
- Mất:
  - tiêu đề trông hơi khác nhau giữa các hệ điều hành, vì dùng font có sẵn;
  - nút nổi chiếm thêm 3 px bên dưới mỗi nút.
- Xem lại khi:
  - thử với người học thật mà trẻ nhỏ thấy khô: tăng độ bo góc và cỡ chữ bằng biến, không đổi hướng;
  - có ngân sách kích thước: cân nhắc tự chứa một font tiêu đề.
