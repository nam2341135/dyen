# ADR 0009: Hợp đồng lõi — mô hình, dấu vết, chấm, nội dung

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26

## Bối cảnh

Tầng 2 của nền móng. Các kiểu dữ liệu ở đây sẽ đi vào dữ liệu lưu trữ lâu dài (dấu vết) và file nội dung, nên khó đổi về sau. Cụ thể hóa ADR 0002, 0003, 0004.

## Quyết định

### Mô hình (`manip`)

- Trait `Manipulative` gồm ba kiểu liên kết `Setup` (cấu hình ban đầu từ nội dung), `State`, `Action` và hai hàm thuần `init(&Setup) -> Result<State>`, `apply(&State, &Action) -> Result<State, Rejection>`. Mỗi mô hình có `KIND` là chuỗi ổn định.
- `apply` **từ chối** hành động không hợp lệ thay vì sửa lặng lẽ. Bộ hiển thị phải xử lý từ chối (vd. nảy vật về chỗ cũ).
- Danh sách mô hình là enum đóng `ModelSetup` (hiện có `Bars`). Thêm mô hình = thêm một module + một biến thể.

### Dấu vết

- `Trace<A> { format, entries }`, `format = TRACE_FORMAT = 1`.
- Mỗi `Entry { at_ms, actor, step }`:
  - `at_ms`: mili-giây tương đối từ lúc bắt đầu hoạt động, **không giảm**.
  - `actor`: `Learner` hoặc `System` — phản hồi của hệ thống (vd. tự chia thanh để "dựng lại") cũng nằm trong dấu vết nhưng được tách khỏi hành động của người học khi chẩn đoán.
  - `step`: `Model(Action)` hoặc `Answer(Answer)`. Câu trả lời không đổi trạng thái mô hình; trả lời nhiều lần là hợp lệ (vòng phản hồi).
- **Dấu vết chỉ chứa hành động đã được chấp nhận.** `Session` chỉ nối vào dấu vết khi `apply` thành công. Khi phát lại, bất kỳ bước nào bị từ chối → dấu vết không hợp lệ (bị sửa hoặc lệch phiên bản).
- Phát lại trả về toàn bộ chuỗi trạng thái (`states[0]` là trạng thái đầu, `states[i+1]` sau bước `i`) để chẩn đoán xem được trạng thái trung gian.

### Số học tất định

- Phân số dùng kiểu `Fraction` (số nguyên, luôn tối giản, so sánh bằng nhân chéo `u64`).
- Mô hình thanh dùng đơn vị **vạch** nguyên: một thanh dài `TICKS = 840` vạch (= BCNN của 1..8), nên mọi cách chia đều 2–8 phần đều chính xác.
- Điểm số `Score` là **phần nghìn** (`0..=1000`), không dùng số thực.

### Chấm và chẩn đoán (`diagnose`)

- `Task` là enum đóng mô tả yêu cầu của hoạt động (hiện có `Compare`, `Partition`). Hàm chấm nhận (task, setup, trace) và trả về `Assessment { score, solved, observations }`.
- `Observation { concept, finding, confidence, evidence }`; `finding` là `Misconception(id)`, `SelfCorrected(id)` hoặc `Demonstrated`; `evidence` là chỉ số bước trong dấu vết.
- Chỉ hành động của `Learner` được dùng làm bằng chứng.
- `diagnose` công bố danh sách mã khái niệm và hiểu sai mà các quy tắc của nó dùng; `content` kiểm tra mọi mã đó tồn tại trong đồ thị khái niệm.

### Nội dung (`content`)

- Định dạng **TOML** (dễ đọc với người không lập trình; enum biểu diễn bằng trường `kind`).
- Bố cục: `content/concepts.toml`, `content/activities/<id>.toml`, `content/sets/<id>.toml`. Tên file phải trùng mã.
- `content` được phụ thuộc `diagnose` (để kiểm tra task hợp lệ và mã chẩn đoán tồn tại) — cập nhật ADR 0006.
- Tách đọc file khỏi phân tích: `Library::from_sources(...)` (thuần, dễ test) và `Library::load(dir)` (đọc đĩa rồi gọi hàm trên). Mọi lỗi được gom lại và báo một lần.
- Một test trong `content` kiểm tra toàn bộ thư mục `content/` của repo, nên nội dung sai làm `cargo xtask ci` đỏ.

## Hệ quả

- Trạng thái của mô hình có trường riêng tư; bộ hiển thị đọc qua phương thức, không tự dựng trạng thái.
- Dấu vết JSON gắn `format`; khi đổi định dạng phải tăng số và giữ đường đọc cũ.
- Tolerance chẩn đoán (vd. ±10% cho "chia đều") là hằng số trong `diagnose`, cần hiệu chỉnh sau thử nghiệm thật.
