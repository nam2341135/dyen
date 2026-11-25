# ADR 0002: Logic tách khỏi hiển thị — 3D không giữ trạng thái

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Sản phẩm đầu tư vào 3D, nhưng phải chạy được trên máy yếu, khi 3D tải chậm, và khi người dùng bật giảm chuyển động — với cùng nhiệm vụ, cùng cách chấm, cùng dữ liệu.

## Quyết định

- Logic của mọi loại hoạt động (`State`, `Input`, `apply`, `evaluate`) nằm trong crate thuần (`kernel`, `kind-*`), không phụ thuộc bất kỳ thư viện hiển thị, mạng hay IO nào.
- Bộ hiển thị (2D SVG, 3D) chỉ làm hai việc: **vẽ trạng thái nhận được** và **chuyển cử chỉ thành `Input`**. Bộ hiển thị không bao giờ tự sửa trạng thái học tập.
- Trạng thái có thẩm quyền ở phía client được giữ bởi vỏ ứng dụng (app shell), không bởi bộ hiển thị. Đổi bộ hiển thị giữa chừng không mất trạng thái.
- Trạng thái riêng của bộ hiển thị (vị trí camera, tiến độ animation, ngón tay đang kéo) được phép tồn tại nhưng không được ghi vào dấu vết.

## Hệ quả

- 2D là bản hoàn chỉnh, không phải bản phụ; 3D có thể tháo ra, thay renderer khác mà không chạm logic.
- Cử chỉ "đang kéo chưa thả" thuộc về bộ hiển thị; chỉ khi người dùng xác nhận (thả tay) mới thành `Input`. Ngoại lệ có chủ đích — ví dụ ghi lại việc đổi ý — phải được định nghĩa thành `Input` rõ ràng trong crate của loại hoạt động.
- Mọi hiệu ứng đẹp phải biểu diễn được như chuyển tiếp giữa hai trạng thái; không có hiệu ứng nào mang thông tin mà 2D không có.
