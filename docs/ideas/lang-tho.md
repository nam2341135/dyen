# Làng Thợ (tên tạm)

- **Trạng thái:** Nháp
- **Người học:** THCS–THPT (11–18 tuổi), tự học ở nhà. Phiên bản đầu chưa có giáo viên.
- **Thiết bị:** mọi loại máy, thiết kế từ điện thoại Android tầm trung trở lên.
- **Mảng đầu tiên:** hình học không gian. Sau đó mở rộng sang hàm số, đại số, xác suất.

Nguồn: phần hỏi đáp với chủ dự án ngày 2026-11-26.

## Vấn đề muốn sửa

Học sinh học thuộc mà không hiểu. Cả lớp học cùng một nhịp. Học thì chán. Giáo viên không biết học sinh hiểu sai ở đâu. Trong đó, hình học không gian là chỗ "thuộc công thức, không thấy hình" nặng nhất. Sách chỉ có hình vẽ phẳng của vật thể ba chiều.

## Ý tưởng chính

Học sinh là **thợ học việc** trong một làng Việt đang được dựng lại: đình, cầu, tháp, nhà rường, chợ, bến. Mỗi công trình được dựng bằng **toán thật**:

| Trong làng | Toán |
|---|---|
| Lợp mái đình: cần bao nhiêu ngói, mái dốc bao nhiêu | Diện tích xung quanh hình chóp/lăng trụ, góc giữa hai mặt phẳng |
| Đẽo cột, xẻ gỗ, cắt đá | Thiết diện của khối cắt bởi mặt phẳng |
| Dựng tháp (dáng tháp Chăm) | Hình chóp cụt, thể tích, chiều cao |
| Gấp đèn lồng, làm hộp quà bằng giấy dó | Hình khai triển (lưới) của khối |
| Bắc cầu, chống kèo | Khoảng cách, góc giữa đường và mặt, đường vuông góc chung |
| Đào ao, xây bể nước | Thể tích, khối tròn xoay |

Toán vừa là **cách chơi**, vừa xen các bài **kiểu đề thi** để luyện ("pha trộn"). Công trình dựng xong sẽ ở lại trong làng. Bạn bè có thể **ghé thăm làng của nhau**, nhưng không có xếp hạng.

## Trải nghiệm

- **Nhịp hằng ngày, 10–15 phút:** "việc hôm nay" do trưởng làng hoặc thợ cả giao. Ôn lặp được đặt vào câu chuyện: một mái cũ bị dột thì phải tính lại, đúng lúc kiến thức đó sắp quên.
- **Xưởng (phòng thí nghiệm):** trước khi dựng thật, học sinh vào xưởng nghịch mô hình: xoay, cắt, trải phẳng, kéo tham số, rồi tự rút ra quy luật. Ví dụ: "mặt cắt của hình lập phương có thể là lục giác không?"
- **Công trường:** dựng thật, với vật liệu có hạn. Tính sai thì thiếu ngói hoặc thừa gỗ, và thấy được hậu quả ngay trong cảnh.
- **Bảng đố ở đình:** các bài kiểu đề thi (trắc nghiệm, đúng/sai, trả lời ngắn), giải được thì mở vật liệu hoặc kỹ thuật mới.
- **Thợ cả (gia sư AI):**
  - chỉ gợi ý, hỏi ngược lại, không giải hộ;
  - giải thích khi học sinh bí lâu;
  - đọc các bước học sinh đã làm và chỉ ra sai ở bước nào;
  - về sau có thể chấm ảnh bài viết tay.

## Các loại hoạt động cần có (ứng viên `kind-*`)

Mỗi loại dưới đây là một crate riêng, không biết gì về làng. Làng chỉ là lớp bọc bên ngoài.

1. **`section`: thiết diện.** Học sinh chọn 3 điểm trên cạnh hoặc mặt của một khối (lập phương, chóp, lăng trụ), dự đoán hình thiết diện, rồi cắt để kiểm tra.
   - **Đã có bản đầu:** `crates/kinds/section`, chỉ hình lập phương, 3 điểm trên cạnh; người học đánh dấu vị trí các đỉnh thiết diện rồi cắt.
   - Đầu vào: `pick_point`, `predict{shape}`, `cut`.
   - Phát hiện có thể có: chỉ nối các điểm cùng mặt nhưng quên giao tuyến; nghĩ thiết diện luôn là tam giác hoặc tứ giác.
2. **`net`: khai triển.** Gấp hoặc trải một lưới thành khối, chọn lưới nào gấp được thành khối nào.
   - Đầu vào: `fold{edge}`, `unfold{edge}`, `submit`.
   - Phát hiện có thể có: hai mặt chồng lên nhau khi gấp; nhầm cạnh nào ghép với cạnh nào.
3. **`measure`: đo và dựng.** Dựng đường vuông góc, hình chiếu, rồi đọc khoảng cách hoặc góc.
   - Đầu vào: `construct{…}`, `answer{value}`.
   - Phát hiện có thể có: lấy nhầm góc giữa hai đường thay cho góc giữa đường và mặt; hình chiếu sai chân.
4. **`build`: dựng có ràng buộc vật liệu.** Chọn kích thước sao cho đạt yêu cầu (thể tích, diện tích, độ dốc).
   - Đầu vào: `set{param, value}`, `submit`.
   - Chấm: đúng yêu cầu, và mức thừa hoặc thiếu vật liệu.
5. **Dạng đề thi:**
   - trắc nghiệm: dùng lại `choice`;
   - đúng/sai nhiều ý: có thể là `choice` chọn nhiều, hoặc một loại riêng;
   - **trả lời ngắn**: một loại mới `short-answer`, nhận đáp số chính xác như `3√2`, `4/3`.

**Ràng buộc tất định:** độ dài trong hình học không gian thường là căn. Logic phải dùng số chính xác, ví dụ tọa độ hữu tỉ, so sánh bình phương độ dài, hoặc biểu diễn `a√b`. Không dùng `f32/f64` cho quyết định (ADR 0002, 0003). Bộ hiển thị 3D vẫn dùng `f32` để vẽ.

## Dữ liệu trẻ em (ADR 0005)

- **Dấu vết thao tác:** không có thông tin cá nhân. Ổn.
- **Chat với thợ cả (AI):** đây là **chữ tự do của trẻ**, được gửi tới một dịch vụ mô hình ngôn ngữ, mà CLAUDE.md cấm khi chưa có ADR. Cần một ADR riêng trả lời: nhà cung cấp nào, dữ liệu có bị lưu hay dùng để huấn luyện không, cần phụ huynh đồng ý thế nào, có lọc thông tin cá nhân trước khi gửi không, và có phương án chỉ gửi trạng thái hoặc dấu vết (không gửi chữ của trẻ) không.
- **Ảnh bài viết tay:** ảnh là dữ liệu cá nhân và có thể lộ chữ viết, tên, trường. Để sau, cũng cần ADR riêng.
- **Ghé làng bạn:** chỉ chia sẻ công trình (dữ liệu có cấu trúc), không có chữ tự do, không có tên thật. Bạn bè kết nối bằng cách nào cũng cần quyết định riêng.

## Lõi còn thiếu gì

Đây là phần giá trị nhất của bản nháp. Mỗi mục dưới đây sẽ cần một ADR trước khi xây.

1. **Toán chính xác dùng chung** — đã chấp nhận ở [ADR 0012](../adr/0012-toan-chinh-xac.md); `crates/libs/exact` có `Rational`, `Vec3`, `Plane`. `section`, `measure`, `build`, `short-answer` đều cần số hữu tỉ, vectơ 3D hữu tỉ, căn dạng `a√b`, mặt phẳng và giao tuyến. Hiện ADR 0006 cấm `kind-*` phụ thuộc lẫn nhau, còn `kind-bars` thì tự viết `Fraction` riêng. Cần cho phép một tầng crate **tiện ích thuần** (ví dụ `exact`) nằm giữa `kernel` và `kind-*`. Nếu không, mỗi loại sẽ tự viết lại phần hình học.
2. **Tiến độ qua nhiều hoạt động.** Hiện lõi chỉ biết một hoạt động với một dấu vết. Làng cần "trạng thái hành trình": đã dựng gì, mở khóa gì, kỹ năng nào sắp quên. Trạng thái này nên được **suy ra từ các `Outcome`** và tất định (tinh thần ADR 0003), đặt ở một tầng mới trên `registry`. Lõi không được biết "làng".
3. **Thời gian trong ôn lặp.** Muốn lên lịch "sắp quên" thì phải biết thời gian, mà crate thuần không được đọc đồng hồ. Thời gian phải được truyền vào như dữ liệu, ví dụ ngày của từng phiên.
4. **Hợp đồng bộ hiển thị.** Mỗi `kind-*` cần một phần vẽ 3D riêng, và cảnh làng phải chứa được cảnh của hoạt động bên trong. Điều này cần một "sổ đăng ký bộ hiển thị" song song với `registry`, và gắn với ADR 0008 (đang chờ đo trên điện thoại).
5. **Gợi ý và gia sư trong dấu vết.**
   - Gợi ý ảnh hưởng tới điểm và phát hiện, nên phải nằm trong dấu vết. Có thể ghi thành bước `Actor::System`, hoặc thêm một khái niệm chung.
   - Lời của AI không tất định, nên **không được** là thứ cần phát lại. Chỉ ghi lại *việc đã gợi ý ở mức nào*.
   - Gia sư cần đọc được trạng thái của mọi loại. Có thể thêm `Kind::describe(state)` để ra một mô tả có cấu trúc.
6. **Tài nguyên nghệ thuật.** Mô hình kiến trúc Việt Nam nặng. Bản spike Bevy đã đo được 2,9 MB nén chỉ riêng engine. Cần quyết định cách tải dần, mức chi tiết theo thiết bị, và chỗ đặt asset (nội dung trong git theo ADR 0004 hay kho riêng).
7. **Tài khoản và chia sẻ.** Ghé làng bạn cần máy chủ, danh tính và kiểm duyệt. Phần này thuộc giai đoạn máy chủ và cần ADR về dữ liệu.

## Câu hỏi mở

- Làng là **một** thế giới chung có cốt truyện cố định, hay mỗi học sinh tự quy hoạch làng của mình?
- Mức bám chương trình GDPT 2018: công trình được sắp theo lớp 8, 9, 11, 12, hay theo chủ đề?
- Học sinh lớp 8 và lớp 12 có chung một làng không?
- Ôn thi: thể thức đề tốt nghiệp THPT hiện hành (trắc nghiệm, đúng/sai, trả lời ngắn) cần được đối chiếu với nguồn chính thức trước khi làm loại hoạt động.
- Phong cách đồ họa cụ thể: low-poly đậm màu, tranh Đông Hồ, hay màu nước? Lựa chọn này ảnh hưởng trực tiếp tới dung lượng tải.
