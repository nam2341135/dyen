# Thuật ngữ

Bộ từ cố định cho người và AI. Trong code dùng tên tiếng Anh ở cột thứ hai; trong tài liệu dùng tên tiếng Việt. **Không tạo từ đồng nghĩa mới** — nếu cần khái niệm mới, thêm vào đây trước.

| Tiếng Việt | Trong code | Định nghĩa |
|---|---|---|
| Mô hình | `Manipulative` | Một loại vật thể có quy tắc mà người học thao tác (vd. thanh cắt được). Định nghĩa kiểu trạng thái, kiểu hành động và hàm áp dụng. |
| Trạng thái | `State` | Toàn bộ thông tin của một mô hình tại một thời điểm. Có thể suy ra hoàn toàn từ trạng thái đầu + dấu vết. |
| Hành động | `Action` | Một thay đổi nguyên tử người học gây ra (cắt, liền, đặt lên thước, trả lời…). Có thể bị từ chối nếu không hợp lệ. |
| Dấu vết | `Trace` | Danh sách hành động có thứ tự kèm thời điểm tương đối và số phiên bản định dạng. Nguồn sự thật về bài làm. |
| Bước | `Entry` / `Step` | Một phần tử của dấu vết: thời điểm, tác nhân, và hành động mô hình hoặc câu trả lời. |
| Tác nhân | `Actor` | Ai gây ra một bước: người học (`Learner`) hoặc hệ thống (`System`). Chỉ bước của người học là bằng chứng chẩn đoán. |
| Câu trả lời | `Answer` | Bước người học trả lời (chọn `<`, `=`, `>`; báo xong). Không đổi trạng thái mô hình. |
| Từ chối | `Rejection` | Mô hình không chấp nhận một hành động. Hành động bị từ chối không vào dấu vết. |
| Phiên | `Session` | Phiên thao tác đang diễn ra: giữ trạng thái có thẩm quyền và ghi dấu vết. |
| Phát lại | `replay` | Áp dụng lần lượt dấu vết lên trạng thái đầu. Phải tất định: cùng đầu vào → cùng kết quả, ở mọi máy. |
| Vạch | `TICKS` | Đơn vị độ dài nguyên của mô hình thanh; một thanh = 840 vạch. |
| Mảnh | `Piece` | Một đoạn thanh nằm giữa hai vết cắt (hoặc đầu thanh). |
| Hoạt động | `Activity` | Một nhiệm vụ cụ thể: mô hình nào, trạng thái đầu, câu hỏi, cách chấm. Là một file nội dung. |
| Gói hoạt động | `ActivitySet` | Chuỗi hoạt động có thứ tự giáo viên giao một lần. |
| Khái niệm | `Concept` | Một nút trong đồ thị kiến thức (vd. "phân số đơn vị"). Có mã ổn định. |
| Hiểu sai | `Misconception` | Một lối nghĩ sai đã biết, gắn với một khái niệm (vd. "mẫu số lớn hơn thì phân số lớn hơn"). Có mã ổn định. |
| Quan sát | `Observation` | Kết luận chẩn đoán rút ra từ một dấu vết: gắn khái niệm/hiểu sai, mức tin cậy, và chỉ số hành động làm bằng chứng. |
| Chấm | `Assessment` | Kết quả chấm một bài làm: điểm (`Score`, phần nghìn 0–1000, không dùng số thực), đã đạt hay chưa, và danh sách quan sát. |
| Nhiệm vụ | `Task` | Yêu cầu của một hoạt động (so sánh hai phân số, chia thanh…). Quyết định cách chấm. |
| Phản hồi | `Feedback` | Bước tiếp theo hệ thống đưa cho người học sau khi chấm (dựng lại, hỏi lại, bài tương tự). |
| Bài làm | `Attempt` | Một lần người học làm một hoạt động: dấu vết + phiên bản nội dung + kết quả chấm. |
| Tiến độ | `Progress` | Mức thành thạo theo từng khái niệm, suy ra từ các bài làm. |
| Bộ hiển thị | `Renderer` | Thành phần vẽ trạng thái và chuyển cử chỉ thành hành động. Hiện có hai: 2D (SVG) và 3D. Không giữ trạng thái học tập. |
| Khoảnh khắc | `Moment` | Đoạn phát lại ngắn gửi cho người chăm sóc. Dựng từ dấu vết, không phải video. |
| Lớp | `Classroom` | Nhóm học sinh do một giáo viên quản lý. |
| Bài giao | `Assignment` | Một gói hoạt động giao cho một lớp (hoặc nhóm trong lớp) với hạn. |
