# Thuật ngữ

Bộ từ cố định cho người và AI. Trong code dùng tên tiếng Anh ở cột thứ hai; trong tài liệu dùng tên tiếng Việt. **Không tạo từ đồng nghĩa mới** — nếu cần khái niệm mới, thêm vào đây trước.

Trang này chỉ chứa thuật ngữ **chung**, không thuộc ý tưởng sư phạm nào. Thuật ngữ riêng của một loại hoạt động nằm trong README của crate đó (vd. [`crates/kinds/bars/README.md`](../crates/kinds/bars/README.md)).

## Lõi

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Loại hoạt động | `Kind` | Một ý tưởng có thể chạy được: định nghĩa cấu hình, trạng thái, đầu vào, cách áp dụng và cách đánh giá. Mỗi loại là một crate `kind-<tên>`. |
| Cấu hình | `Config` | Tham số của một hoạt động thuộc một loại, đọc từ bảng `[config]` của file nội dung. |
| Trạng thái | `State` | Toàn bộ thông tin của một hoạt động đang làm tại một thời điểm. Suy ra hoàn toàn từ cấu hình + dấu vết. |
| Đầu vào | `Input` | Một việc nguyên tử người học hoặc hệ thống làm (thao tác, trả lời, báo xong…). Có thể bị từ chối. |
| Từ chối | `Rejection` | Loại hoạt động không chấp nhận một đầu vào. Đầu vào bị từ chối không vào dấu vết. |
| Dấu vết | `Trace` | Danh sách đầu vào đã được chấp nhận, có thứ tự, kèm thời điểm tương đối và số phiên bản định dạng. Nguồn sự thật về bài làm. |
| Bước | `Entry` | Một phần tử của dấu vết: thời điểm (`at_ms`), tác nhân, đầu vào. |
| Tác nhân | `Actor` | Ai gây ra một bước: người học (`Learner`) hoặc hệ thống (`System`). Chỉ bước của người học là bằng chứng. |
| Phát lại | `replay` | Áp dụng lần lượt dấu vết lên trạng thái đầu. Tất định: cùng đầu vào → cùng kết quả, ở mọi máy. |
| Phiên | `Session` | Phiên làm hoạt động đang diễn ra: giữ trạng thái có thẩm quyền và ghi dấu vết. |
| Kết quả | `Outcome` | Đánh giá một dấu vết: điểm, đã hoàn thành chưa, danh sách phát hiện. |
| Điểm | `Score` | Phần nghìn, `0..=1000`. Không dùng số thực. |
| Phát hiện | `Finding` | Một điều rút ra từ dấu vết: nhãn, đã tự vượt qua chưa (`recovered`), mức tin cậy, chỉ số bước làm bằng chứng. |
| Nhãn | `Tag` / `TagId` | Mã tự do gắn vào phát hiện và hoạt động (vd. `frac.equal-parts`). Lõi không gán nghĩa; tên hiển thị ở `content/tags.toml`. |
| Hợp đồng | `contract` | Các tính chất mọi loại hoạt động phải thỏa; kiểm bằng `kernel::contract::check`. |
| Sổ đăng ký | `registry` | Danh sách đóng các loại hoạt động; chỗ duy nhất rẽ nhánh theo loại ở tầng logic (vỏ `web` có một `match` đóng chọn bộ hiển thị, ADR 0010). |

## Nội dung

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Hoạt động | `Activity` | Một file nội dung: tên, lời dẫn, nhãn, và cấu hình của một loại hoạt động. |
| Gói hoạt động | `ActivitySet` | Chuỗi hoạt động có thứ tự giao một lần. |

## Phòng học 3D (vỏ web, ADR 0016)

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Phòng | `Room` (bảng `ROOM`) | Trang chủ 3D: bảng các chỗ đặt đồ; chỉ để điều hướng, không chứa chức năng học. |
| Khu | `Zone` | Một khu của vỏ (ôn tập, khóa học, thi đấu) mà một vật trong phòng dẫn tới. Khái niệm của vỏ, không có trong lõi. |
| Chỗ đặt đồ | `Slot` | Một vị trí trong phòng: mã, loại đồ vật, vị trí (cm), hướng, và khu dẫn tới (nếu có). |

## Sản phẩm (chưa có code)

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Bộ hiển thị | `Renderer` | Thành phần vẽ trạng thái và chuyển cử chỉ thành đầu vào (2D, 3D). Không giữ trạng thái học tập. |
| Bài làm | `Attempt` | Một lần người học làm một hoạt động: dấu vết + phiên bản nội dung + kết quả. |
| Lớp | `Classroom` | Nhóm học sinh do một giáo viên quản lý. |
| Bài giao | `Assignment` | Một gói hoạt động giao cho một lớp (hoặc nhóm trong lớp) với hạn. |
