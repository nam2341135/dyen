# kind-section

Loại hoạt động `section`: tìm **thiết diện** của hình lập phương `ABCD.A'B'C'D'` cắt bởi mặt phẳng qua 3 điểm đề cho. Là **một ý tưởng** cắm vào lõi (ADR 0010) — gỡ bằng cách xóa crate này và biến thể `Section` trong `crates/registry`.

Ứng viên đầu tiên từ bản nháp [Làng Thợ](../../../docs/ideas/lang-tho.md); bản thân crate không biết gì về làng. Mọi quyết định dùng số hữu tỉ chính xác của `lib-exact` (ADR 0012).

## Thuật ngữ riêng

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Vị trí | `site` | Một đỉnh hoặc một cạnh của lập phương — nơi đỉnh thiết diện có thể nằm. 20 vị trí, thứ tự chuẩn: 8 đỉnh rồi 12 cạnh. |
| Điểm đề cho | `EdgePoint` | Điểm trên cạnh, `at ∈ [0, 1]` tính từ đỉnh đầu của cạnh; `0`/`1` là chính đỉnh. Ba điểm xác định mặt cắt. |
| Đỉnh thiết diện | `Corner` | Đỉnh của đa giác thiết diện: đỉnh lập phương nằm trên mặt cắt, hoặc giao điểm với cạnh có hai đầu nằm hai phía. Mỗi vị trí chứa nhiều nhất một đỉnh. |
| Lần cắt | `Input::Cut` | Nộp tập vị trí đang đánh dấu. Đúng khi trùng khớp tập vị trí của các đỉnh thiết diện. Tối đa `attempts` lần. |

## Khóa

- Đỉnh: `a b c d` (đáy), `a1 b1 c1 d1` (nắp; `a1` là `A'`). Tọa độ: `A(0,0,0)`, `B(1,0,0)`, `C(1,1,0)`, `D(0,1,0)`, nắp có `z = 1`.
- Cạnh: `ab bc cd da`, `a1b1 b1c1 c1d1 d1a1`, `aa1 bb1 cc1 dd1`. Hướng cạnh theo thứ tự chữ trong khóa.

## Cấu hình

Đúng 3 điểm, không thẳng hàng; mẫu số của `at` không quá `MAX_DENOM` (1000) để phép tính không tràn. Mặt phẳng phải đi vào trong lập phương (có đỉnh nằm hẳn ở hai phía) — không nhận mặt phẳng chỉ chạm hoặc trùng một mặt.

## Đầu vào (`op`)

`mark`, `unmark` đổi tập vị trí đang đánh dấu; `cut` nộp (cần ít nhất 3). Sau khi cắt đúng hoặc hết lượt, mọi đầu vào bị từ chối.

## Đánh giá

- **Hoàn thành**: có một lần cắt của người học đúng.
- **Điểm**: `1000 / k` phần nghìn, `k` là số thứ tự lần cắt đúng; chưa đúng → 0.
- **Phát hiện** (bằng chứng là các lần cắt sai mang dấu hiệu; `strong` nếu từ hai lần; `recovered` nếu về sau cắt đúng, khi ấy lần cắt đúng được thêm vào bằng chứng):

| Nhãn | Dấu hiệu |
|---|---|
| `geo.section-only-given` | Cắt sai khi chỉ đánh dấu vị trí của các điểm đề cho — chỉ nối điểm có sẵn, quên tìm giao tuyến với các mặt khác. |
| `geo.section-off-plane` | Cắt với một vị trí mà mặt phẳng không đi qua. |

## Bộ hiển thị

`State::section()` có tọa độ chính xác của các đỉnh thiết diện theo thứ tự vị trí, không theo vòng đa giác; bộ hiển thị tự sắp (bằng `f32`, chỉ để vẽ) và chỉ hiện sau khi cắt.
