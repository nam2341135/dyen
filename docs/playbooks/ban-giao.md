# Bàn giao: bắt đầu và dừng một phiên

Người (hoặc AI) đến sau chỉ thấy những gì nằm trong repo và trên GitHub. Không thấy lịch sử chat, memory của công cụ, hay ledger trong `.superpowers/` (bị `.gitignore`).

## Bắt đầu

1. Đọc [docs/STATUS.md](../STATUS.md).
2. Xem trạng thái thật, vì STATUS có thể đã cũ:

   ```
   git status
   git log --oneline -5
   gh pr list
   ```

3. Chạy `cargo xtask ci` trước khi sửa gì. Nếu đã đỏ từ đầu, ghi lại: đó không phải lỗi do mình.
4. Chọn việc:
   - người dùng giao việc cụ thể thì làm việc đó;
   - người dùng nói "làm tiếp" thì làm mục dở dang ở "Đang làm", nếu không có thì làm mục đầu tiên ở "Tiếp theo".
5. Đọc playbook của loại việc đó ([README.md](README.md)).

## Trong khi làm

- Mỗi việc một nhánh: `loại/tên-ngắn`, ví dụ `fix/bars-khoa`. Không làm trên `main`.
- Quyết định nhỏ (chọn A thay vì B, tên gọi, ngưỡng số):
  - ghi vào mô tả PR, mục "Quyết định đã tự đưa ra";
  - nếu chưa có PR, ghi tạm vào "Đang làm".
- Quyết định lớn (đổi ranh giới, thêm phụ thuộc, đổi định dạng dấu vết): viết ADR "Đề xuất" và **hỏi**.
- Mâu thuẫn với ADR đã "Chấp nhận": dừng lại và hỏi.

## Trước khi dừng

Dừng vì hết phiên, sắp hết token, hay đổi việc đều như nhau.

1. Chạy `cargo xtask ci`. Ghi kết quả, kể cả khi đỏ.
2. Sửa [STATUS.md](../STATUS.md):
   - **Đang làm**: một mục theo mẫu dưới đây;
   - **Tiếp theo**: bỏ mục đã xong, thêm việc mới phát hiện;
   - **Chờ người quyết**: thêm câu hỏi đang chờ;
   - **Đã xong gần đây**: thêm hàng nếu việc đã merge.
3. Việc chưa commit được thì để lại trên nhánh. Ghi rõ trong "Đang làm" là chưa commit.

Mẫu một mục "Đang làm":

```markdown
- **<Tên việc>.** Nhánh `<nhánh>`, PR #<số> (hoặc "chưa có PR", "chưa commit").
  Đã xong: <…>. Còn: <…>. CI: xanh / đỏ (<lỗi đầu tiên>).
  Lệnh tiếp theo: `<lệnh cụ thể người sau chạy đầu tiên>`.
```

Khi việc đã merge thì xoá mục khỏi "Đang làm".
