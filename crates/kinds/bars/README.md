# kind-bars

Loại hoạt động `bars`: thanh cắt được và thước so sánh. Là **một ý tưởng** cắm vào lõi (ADR 0010) — gỡ bằng cách xóa crate này và biến thể `Bars` trong `crates/registry`.

## Thuật ngữ riêng

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Vạch | `TICKS` | Đơn vị độ dài nguyên; một thanh = 840 vạch (BCNN 1..8). |
| Vạch chia | `Bar::marks` | Vạch đặt bởi `partition`, chỉ để gợi ý chỗ cắt. |
| Mảnh | `Piece` | Đoạn thanh giữa hai vết cắt (hoặc đầu thanh); mã `(thanh, vạch bắt đầu)`. |
| Thước | `State::ruler` | Nơi đặt mảnh để so sánh. Mảnh trên thước không cắt/nối được. |
| Nhiệm vụ | `Task` | `compare` (so sánh hai phân số) hoặc `partition` (chia thanh thành n phần bằng nhau). |

## Đầu vào (`op`)

`partition`, `cut`, `join`, `place`, `unplace` đổi trạng thái; `compare` (trả lời `<`/`=`/`>`) và `done` không đổi trạng thái, chỉ để đánh giá.

## Nhãn

Xem `tags::ALL`. `content/tags.toml` phải đặt tên hiển thị cho mọi nhãn.
