# ADR 0003: Dấu vết là nguồn sự thật; phát lại phải tất định

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Giá trị khác biệt của sản phẩm là đọc được *cách* người học làm, không chỉ kết quả. Dữ liệu này phải tin được (không phụ thuộc client) và dùng được lâu dài (dấu vết năm nay phát lại được năm sau).

## Quyết định

- Bài làm được lưu dưới dạng **dấu vết** (trạng thái đầu xác định bởi hoạt động + phiên bản nội dung, cộng danh sách đầu vào). Không lưu trạng thái cuối do client gửi như nguồn sự thật.
- Server **chạy lại** và **đánh giá** dấu vết bằng chính các crate thuần (`kernel`, `kind-*`, `registry`) mà client dùng.
- `apply` và `replay` phải **tất định**: không đọc đồng hồ, không số ngẫu nhiên không có seed, không phụ thuộc thứ tự duyệt `HashMap`, không dùng phép tính dấu phẩy động có thể khác nhau giữa nền tảng cho các quyết định logic (ưu tiên số nguyên / phân số hữu tỉ).
- Định dạng dấu vết mang **số phiên bản định dạng** ngay từ bản đầu. Thay đổi phá tương thích phải tăng phiên bản và giữ đường đọc cho phiên bản cũ.
- Thời điểm trong dấu vết là **mili-giây tương đối** tính từ lúc bắt đầu hoạt động, không phải giờ đồng hồ thật.

## Hệ quả

- Kiểm thử dễ: một file dấu vết mẫu là một ca kiểm thử đầy đủ, chạy được không cần giao diện.
- Chống gian lận cơ bản miễn phí.
- Phải kỷ luật về tất định; một `f32` sai chỗ có thể làm client và server bất đồng. Có property test để bắt.
- Đầu vào phải đủ nhỏ và có ý nghĩa — không ghi từng khung hình kéo ngón tay.
