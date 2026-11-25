# kind-choice

Loại hoạt động `choice`: chọn một hoặc nhiều lựa chọn rồi nộp. Là **một ý tưởng** cắm vào lõi (ADR 0010) — gỡ bằng cách xóa crate này và biến thể `Choice` trong `crates/registry`.

Không gắn với môn học nào: câu hỏi, lựa chọn và nhãn đều do người soạn nội dung viết.

## Thuật ngữ riêng

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Lựa chọn | `Item` | Một phương án trả lời; có thể thuộc đáp án đúng, có thể gắn nhãn. |
| Khóa | `Item::key` | Mã ổn định của lựa chọn, dùng trong dấu vết. Không đổi khóa đã phát hành. |
| Đáp án đúng | `Item::correct` | Tập các lựa chọn đúng. Chọn một: đúng một lựa chọn; chọn nhiều: ít nhất một. |
| Lần nộp | `Input::Submit` | Nộp tập đang chọn. Đúng khi trùng khớp đáp án đúng. Tối đa `attempts` lần. |

## Đầu vào (`op`)

`select`, `deselect` đổi tập đang chọn (chọn một: `select` thay lựa chọn cũ); `submit` nộp. Sau khi nộp đúng hoặc hết lượt, mọi đầu vào bị từ chối.

## Đánh giá

- **Hoàn thành**: có một lần nộp của người học trùng khớp đáp án đúng.
- **Điểm**: `1000 / k` phần nghìn, `k` là số thứ tự lần nộp đúng; chưa đúng → 0.
- **Phát hiện**: với mỗi lựa chọn có `tag` đã nằm trong ít nhất một lần nộp của người học — bằng chứng là các lần nộp đó; `strong` nếu từ hai lần; `recovered` nếu cuối cùng nộp đúng mà không còn lựa chọn đó (khi ấy lần nộp đúng được thêm vào bằng chứng).

## Nhãn

`TAGS` rỗng. Nhãn đến từ cấu hình, công bố qua `Kind::config_tags` (ADR 0011). `dyen check` kiểm mọi nhãn này có trong `content/tags.toml`.
