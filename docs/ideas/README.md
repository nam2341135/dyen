# Phác thảo ý tưởng

Thư mục này là **nháp**, không phải quyết định. Mục đích của mỗi ý tưởng là đem ra thử lõi: nó cần gì mà lõi chưa có? Không ý tưởng nào là cốt lõi của dự án (ADR 0010).

Quy tắc:

- Mỗi ý tưởng là một file, viết theo mẫu bên dưới.
- Chỗ nào lõi còn thiếu thì ghi rõ ở mục "Lõi còn thiếu gì". Khi quyết định cách lấp chỗ thiếu đó, viết **ADR**, không quyết định ở đây.
- Thuật ngữ trong file ý tưởng chỉ là nháp. Thuật ngữ chỉ chính thức khi nằm trong README của crate `kind-*` hoặc trong `glossary.md`.
- Bỏ một ý tưởng thì ghi "Bỏ" kèm lý do, đừng xóa file.

## Danh sách

| Ý tưởng | Tóm tắt | Trạng thái |
|---|---|---|
| [Làng Thợ](lang-tho.md) | Thế giới 3D đậm chất Việt; học sinh THCS–THPT học hình học không gian bằng cách dựng công trình | Nháp |

## Mẫu

```markdown
# <Tên tạm>

- **Trạng thái:** Nháp | Đang thử | Bỏ (lý do)
- **Người học:** độ tuổi, bối cảnh (một mình / có người lớn / trên lớp)
- **Thiết bị:** …

## Vấn đề muốn sửa
## Trải nghiệm (một buổi học trông thế nào)
## Các loại hoạt động cần có (ứng viên `kind-*`)
   Với mỗi loại: trạng thái, đầu vào, chấm và phát hiện.
## Dữ liệu trẻ em (ADR 0005)
   Ý tưởng thu thập gì? Có chữ tự do, giọng nói, ảnh không? Có gửi ra ngoài không?
## Lõi còn thiếu gì
## Câu hỏi mở
```
