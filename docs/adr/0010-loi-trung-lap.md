# ADR 0010: Lõi trung lập — loại hoạt động là crate, sổ đăng ký là enum đóng

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26
- **Thay thế:** [ADR 0009](0009-hop-dong-loi.md)

## Bối cảnh

ADR 0009 đặt một ý tưởng sư phạm cụ thể (mô hình thao tác + chẩn đoán khái niệm/hiểu sai) vào lõi: `domain` có `Concept`/`Misconception`, `diagnose` có `Task` chung, `content` có đồ thị khái niệm. Chủ dự án chưa muốn chọn ý tưởng nào làm cốt lõi; mục tiêu là thêm, sửa, thay, bỏ ý tưởng dễ dàng. Với ADR 0009, đổi ý tưởng nghĩa là sửa cả ba crate thuần và lược đồ nội dung.

## Quyết định

### Lõi (`kernel`)

Trait `Kind` là hợp đồng duy nhất:

- `KIND` (tên ổn định), `TAGS` (mọi nhãn có thể gắn vào phát hiện);
- kiểu `Config` (từ file nội dung), `State`, `Input`;
- `init(&Config) -> Result<State, ConfigError>`, `apply(&State, &Input) -> Result<State, Rejection>`, `evaluate(&Config, &Trace<Input>, &[State]) -> Outcome`.

`Input` gồm mọi thứ người học hoặc hệ thống làm, kể cả trả lời — lõi không phân biệt "thao tác" và "câu trả lời". Một đầu vào không đổi trạng thái (vd. trả lời) vẫn là đầu vào hợp lệ.

`Outcome { score, completed, findings }`; `Finding { tag, recovered, confidence, evidence }`. `tag` là mã tự do (`TagId`) — lõi không biết nó là khái niệm, hiểu sai, kỹ năng hay gì khác. `recovered = true` nghĩa là người học vướng điều được gắn nhãn rồi tự vượt qua.

Lõi còn có: `Trace`, `Entry { at_ms, actor, input }`, `replay`, `run`, `Session`, mã định danh nội dung (`ActivityId`, `ActivitySetId`, `TagId`), và `contract::check` — bộ kiểm hợp đồng dùng lại cho mọi loại.

### Loại hoạt động (`crates/kinds/<tên>`, crate `kind-<tên>`)

- Chỉ phụ thuộc `kernel` (+ `serde`, `thiserror`). Không phụ thuộc loại khác.
- Chứa toàn bộ ý tưởng: kiểu, quy tắc, đánh giá, thuật ngữ riêng (trong README của crate, không trong glossary chung), test, và một test chạy `kernel::contract::check`.

### Sổ đăng ký (`registry`)

- Enum đóng `AnyConfig` (gắn thẻ `kind`) với một biến thể mỗi loại; hàm `run` đọc dấu vết theo đúng kiểu đầu vào của loại, phát lại và đánh giá.
- Đây là chỗ duy nhất `match` theo loại **ở tầng logic**. Thêm loại: một biến thể + vài dòng `match` mà trình biên dịch chỉ ra.
- Sửa đổi 2026-11-26: vỏ `web` có thêm một `match` đóng trên `AnyConfig`, chỉ để chọn bộ hiển thị 2D của loại (bộ hiển thị nằm trong `web/src/kinds/<tên>.rs`). Thêm loại thì trình biên dịch chỉ ra cả chỗ này. Đã cân nhắc tách mỗi bộ hiển thị thành crate `view-<tên>`; bỏ vì gấp đôi số crate mà chưa có ca dùng thứ hai.
- Không hệ plugin động, không đăng ký lúc chạy.

### Nội dung

- `content/tags.toml` thay `concepts.toml`: danh sách `[[tag]] { id, title }`. Mọi nhãn trong `TAGS` của mọi loại phải có ở đây (và mọi nhãn trong `config_tags` của từng hoạt động — [ADR 0011](0011-nhan-tu-cau-hinh.md)).
- Hoạt động: `id`, `title`, `prompt`, `tags` (tùy chọn), bảng `[config]` có `kind` và các trường do loại định nghĩa. Chữ trình bày (`prompt`) thuộc nội dung, không thuộc `Config`.

### Định dạng dấu vết

`format = 1` được định nghĩa lại (bước mang `input` thay cho `step`) vì chưa phát hành và chưa có dữ liệu thật. Từ lúc phát hành, quy tắc của ADR 0003 áp dụng nghiêm: đổi phá tương thích phải tăng phiên bản.

## Hệ quả

- Thay ý tưởng không chạm lõi. Ý tưởng mới cần thứ lõi chưa có (vd. đánh giá cần dữ liệu ngoài dấu vết) → sửa `kernel` có chủ đích, kèm ADR.
- Lõi không có "khái niệm", "tiến độ theo khái niệm", "đồ thị kiến thức". Nếu sau này cần, đó là một quyết định riêng xây trên `TagId`.
- Mỗi loại tự công bố nhãn; `dyen check` bắt nhãn thiếu tên hiển thị.
- `content` và `cli` không biết loại nào; chúng đi qua `registry`. `web` biết từng loại, nhưng chỉ để vẽ.
