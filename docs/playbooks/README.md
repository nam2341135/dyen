# Playbook: công thức từng bước

Mỗi playbook là danh sách việc cho một loại thay đổi hay gặp, kèm tiêu chí "xong". Làm theo thì không phải đoán chỗ nào cần sửa. Thiếu bước hoặc bước sai thì sửa playbook trong cùng thay đổi.

| Việc định làm | Đọc |
|---|---|
| Bắt đầu hoặc dừng một phiên làm việc | [ban-giao.md](ban-giao.md) |
| Thêm loại hoạt động mới (một ý tưởng, một cách tương tác) | [them-loai-hoat-dong.md](them-loai-hoat-dong.md) |
| Thêm hoặc sửa nội dung học (hoạt động, bộ, nhãn) | [content/README.md](../../content/README.md), rồi `cargo run -p cli -- check` |
| Sửa vỏ web, bộ hiển thị, giao diện | [crates/web/README.md](../../crates/web/README.md) và [ADR 0013](../adr/0013-quy-uoc-bo-hien-thi.md) |
| Ra quyết định kiến trúc | chép [0000-mau.md](../adr/0000-mau.md), thêm hàng vào bảng ở [docs/README.md](../README.md) |
| Thêm crate hoặc phụ thuộc ngoài | [ADR 0006](../adr/0006-ranh-gioi-crate.md); crate thuần thì hỏi trước |
| Phác thảo một ý tưởng mới | [docs/ideas/README.md](../ideas/README.md) |

Việc không có trong bảng: làm theo `AGENTS.md`, mục "Xong nghĩa là". Nếu loại việc đó sẽ lặp lại, viết thêm một playbook.
