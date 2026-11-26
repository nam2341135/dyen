# ADR 0005: Nguyên tắc dữ liệu trẻ em

- **Trạng thái:** Chấp nhận (nguyên tắc kỹ thuật) — **phần pháp lý chưa xác minh**
- **Ngày:** 2026-11-26
- **Sửa đổi 2026-11-27:** chủ dự án bỏ quy tắc về ô nhập chữ tự do để có bảng ghi chú (ADR 0016, mốc 2).

## Bối cảnh

Người dùng chính là trẻ em. Quyết định về dữ liệu khó sửa về sau nếu schema và luồng dữ liệu đã hình thành.

## Quyết định

1. **Thu tối thiểu.** Học sinh không có email, số điện thoại, ngày sinh, ảnh. Giáo viên nhập *tên gọi* (vd. "Minh N."). Đăng nhập học sinh bằng mã lớp + thẻ hình.
2. **Dấu vết không chứa dữ liệu cá nhân.** Chỉ hành động trên mô hình. Bản đầu không có ghi âm, quay video.
3. **Tách danh tính.** Tên gọi nằm ở bảng riêng; bài làm chỉ tham chiếu ID ẩn danh. Dữ liệu xuất cho phân tích không bao giờ kèm tên.
4. **Phạm vi truy cập hẹp.** Giáo viên → chỉ lớp mình. Người chăm sóc → chỉ con mình, liên kết bằng mã một lần do giáo viên phát. Học sinh không xem dữ liệu của học sinh khác.
5. **Không bên thứ ba trên trang học sinh.** Không analytics ngoài, không pixel, không font/CDN ngoài. Đo hiệu năng tự gửi về server của mình.
6. **Có hạn lưu.** Dấu vết thô có hạn lưu (mặc định đề xuất 12 tháng), sau đó chỉ giữ số liệu gộp. Có chức năng xuất và xóa.

Kiểm tra tự động (khi có server): bài test phân quyền cho mọi endpoint; kiểm tra crate phía học sinh không phụ thuộc thư viện analytics.

## Hệ quả

- Một số tính năng hấp dẫn (chat, ghi âm giải thích, bảng xếp hạng công khai) bị chặn cho tới khi có ADR riêng.
- **Cần tư vấn pháp lý trước khi thử nghiệm với trẻ thật**: Nghị định 13/2023/NĐ-CP, Luật Bảo vệ dữ liệu cá nhân (hiệu lực dự kiến từ 2026), yêu cầu đồng ý của cha mẹ (và của trẻ từ 7 tuổi), yêu cầu về nơi lưu trữ. Các mốc này chưa đối chiếu văn bản gốc.
