# ADR 0004: Nội dung học là file trong git, kiểm tra lúc build

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Giai đoạn đầu nội dung do đội viết (có AI hỗ trợ), số lượng nhỏ, cần xem lại kỹ. Chưa có nhu cầu giáo viên soạn tự do.

## Quyết định

- Hoạt động, gói hoạt động, đồ thị khái niệm và danh sách hiểu sai là **file văn bản** trong thư mục `content/` (định dạng RON hoặc TOML — chốt khi làm tầng 2).
- Crate `content` đọc và **kiểm tra** toàn bộ nội dung: tham chiếu khái niệm có tồn tại, mã không trùng, trạng thái đầu hợp lệ với mô hình, mọi hiểu sai được chẩn đoán có ít nhất một dấu vết mẫu.
- Kiểm tra nội dung là một phần của `cargo xtask ci`; nội dung sai làm CI đỏ.
- Mỗi hoạt động có mã ổn định và số phiên bản; bài làm ghi lại phiên bản đã dùng.

## Hệ quả

- Xem lại nội dung qua diff; AI thêm nội dung được mà không cần giao diện quản trị.
- Giáo viên chỉ chỉnh tham số (chọn, bỏ, sắp xếp, đổi số) — phần chỉnh này lưu trong database như "bản tùy chỉnh" trỏ về nội dung gốc, không sửa file.
- Xem lại khi cần giáo viên tự soạn hoạt động mới hoàn toàn.
