# ADR 0014: Lưu bài làm cục bộ, ẩn danh, trên máy người học

- **Trạng thái:** Đề xuất
- **Ngày:** 2026-11-26
- **Liên quan:** [ADR 0003](0003-dau-vet-la-nguon-su-that.md) (dấu vết), [ADR 0004](0004-noi-dung-la-file.md) (phiên bản nội dung), [ADR 0005](0005-du-lieu-tre-em.md) (dữ liệu trẻ em)

## Bối cảnh

Vỏ web hiện không lưu gì. Học xong một hoạt động, tải lại trang là bài làm mất. Giá trị khác biệt của dự án là đọc được *cách* người học làm (ADR 0003). Muốn thế thì cần dấu vết thật của người học thật, và cần trước khi có server. Các bộ phát hiện (`Finding`) hiện chỉ được thử trên dấu vết mẫu do người viết.

Ràng buộc:

- **Dữ liệu trẻ em (ADR 0005).** Thu tối thiểu, không dữ liệu cá nhân trong dấu vết, có hạn lưu, có xuất và xoá. Phần pháp lý chưa xác minh.
- **Chưa có server.** Axum mới "đề xuất" (ADR 0008), crate `server` thuộc giai đoạn sau. Vì vậy chưa có tài khoản, chưa có danh tính.
- **Phiên bản nội dung chưa có trong code.** ADR 0004 yêu cầu "bài làm ghi lại phiên bản đã dùng", nhưng `content::Activity` không có trường phiên bản. Hoạt động trong `content/` được sửa tại chỗ trong git. Nếu bài làm chỉ ghi mã hoạt động, thì sau khi cấu hình đổi, dấu vết cũ sẽ phát lại sai hoặc không phát lại được.
- **Crate thuần không IO (ADR 0006).** Việc lưu chỉ được nằm ở vỏ (`web`).

Vì sao quyết bây giờ: định dạng bài làm khó đổi khi đã có dữ liệu thật trên máy người dùng. Mọi việc có lưu trữ (lịch sử, tiến độ, đồng bộ sau này) đều dựa trên quyết định này.

## Quyết định

### 1. Bài làm tự chứa

Một bài làm (`Attempt`) đã lưu gồm:

| Trường | Nội dung |
|---|---|
| `format` | Phiên bản định dạng bản ghi bài làm, bắt đầu từ `1`. Độc lập với `format` của dấu vết. |
| `activity` | Mã hoạt động (`ActivityId`), để nhóm và hiển thị. |
| `config` | **Bản chép nguyên cấu hình** (`AnyConfig`) lúc làm, gồm cả `kind`. |
| `trace` | Dấu vết (`Trace`), đúng định dạng hiện có. |
| `day` | Ngày bắt đầu, dạng `YYYY-MM-DD`, theo giờ máy. Chỉ dùng cho hạn lưu. |

- **Phát lại chỉ cần chính bản ghi:** `registry::run(&config, trace)`. Không phụ thuộc nội dung hiện tại trong `content/`.
- **Bản chép cấu hình là "phiên bản đã dùng" mà ADR 0004 yêu cầu.** Không thêm trường số phiên bản vào nội dung lúc này. Nếu sau này cần biết hai bài làm có cùng phiên bản hay không, so sánh hai cấu hình là đủ.
- **Không lưu kết quả (`Outcome`).** Kết quả luôn được tính lại bằng phát lại (ADR 0003). Như vậy sửa bộ chấm thì bài làm cũ được chấm theo bộ mới.
- **Không lưu `title`/`prompt`.** Đó là chữ trình bày, không ảnh hưởng chấm. Hiển thị lấy từ nội dung hiện tại theo `activity`. Nếu mã không còn trong nội dung thì hiện mã.

### 2. Ẩn danh, không danh tính

- Không có trường nào về người học: không tên, không mã người dùng, không mã thiết bị.
- Một trình duyệt là một nơi lưu. Hai em dùng chung máy thì bài làm lẫn vào nhau. Chấp nhận ở giai đoạn này.
- Ngày chỉ lấy tới ngày. Không lưu giờ đồng hồ, không lưu múi giờ. Thời gian trong dấu vết vẫn là mili-giây tương đối (ADR 0003).

### 3. Lưu ở đâu, khi nào

- **Nơi lưu:** `localStorage` của trình duyệt, một khoá duy nhất là `dyen.attempts`. Giá trị là JSON `{"format": 1, "attempts": [...]}`. Không cookie, không IndexedDB, không gửi đi đâu.
- **Lúc lưu:** lưu một lần khi phiên kết thúc (`Outcome` báo đã hoàn thành).
  - Phiên bỏ dở thì không lưu. Đơn giản hơn, và chưa có ai dùng dữ liệu bỏ dở.
  - Xem lại nếu cần nghiên cứu chỗ người học bỏ cuộc.
- **Giới hạn:** giữ tối đa **500** bài làm mới nhất. Mỗi bài làm cỡ vài KB, nên vẫn nằm xa dưới hạn mức ~5 MB của `localStorage`. Vượt quá thì bỏ bài cũ nhất.
- **Hạn lưu (ADR 0005 mục 6):** mỗi lần mở trang, bỏ các bài làm có `day` cũ hơn **12 tháng**.
- **Đọc lỗi thì không mất dữ liệu.** JSON hỏng hoặc `format` lạ thì không ghi đè. Vỏ báo lỗi và ngừng lưu cho tới khi người dùng xoá. Bài làm có `format` dấu vết lạ thì bỏ qua khi phát lại, không xoá.

### 4. Xuất và xoá (ADR 0005 mục 6)

- Có hai nút ở vỏ web:
  - **"Tải bài làm"**: tải về một file `.json` đúng định dạng đang lưu.
  - **"Xoá bài làm"**: xoá khoá `dyen.attempts`, có hỏi xác nhận.
- File xuất là cách duy nhất để dữ liệu rời khỏi máy. Việc đó do người dùng tự làm. Đây là đường để người thử nghiệm gửi dấu vết thật cho nhóm phát triển.
- `cargo run -p cli -- replay` sẽ đọc được file xuất. Nếu cần lệnh mới thì thêm khi làm.

### 5. Code đặt ở đâu

- **Kiểu bản ghi và đọc/ghi JSON:** module thuần trong `crates/web` (`attempts.rs`). Phần chạm vào `localStorage` tách riêng, mỏng, để phần thuần test được trên máy host như `play.rs`.
  - Chưa đưa vào `registry` hay `content`: mới có một nơi dùng (vỏ web).
  - Khi `cli` cần đọc file xuất, hoặc khi có server, chuyển kiểu bản ghi xuống crate dùng chung. Đó là ca dùng thứ hai (AGENTS.md: không trừu tượng trước).
- Không thêm phụ thuộc mới. `web-sys` đã có; nếu cần thì chỉ bật thêm feature `Storage`.

## Hệ quả

- **Được:**
  - Bài làm thật phát lại được mãi, kể cả khi nội dung đổi.
  - Không cần quy trình đánh số phiên bản nội dung.
  - Có đường lấy dấu vết thật để thử bộ phát hiện.
  - Không danh tính, không gửi mạng, nên rủi ro về dữ liệu trẻ em thấp nhất có thể.
- **Mất / chấp nhận:**
  - Bản ghi to hơn vì chứa cấu hình (vài KB).
  - Dùng chung máy thì lẫn bài làm.
  - Xoá dữ liệu trình duyệt là mất bài làm.
  - Không đồng bộ giữa các máy.
  - Phiên bỏ dở không được lưu.
- **Lệch với ADR 0004:** ADR 0004 nói "mỗi hoạt động có mã ổn định và số phiên bản". ADR này thay "số phiên bản" bằng "bản chép cấu hình" cho bài làm. Nếu chấp nhận, cập nhật ADR 0004 để trỏ về đây.
- **Pháp lý:** lưu trên máy người dùng, không gửi đi, vẫn có thể được coi là xử lý dữ liệu theo Nghị định 13/2023. Chưa xác minh, giống ADR 0005. Không thử nghiệm với trẻ thật trước khi xác minh.
- **Xem lại khi:**
  - có server: đồng bộ, danh tính ẩn danh theo ADR 0005 mục 3, và bản ghi có thể chuyển sang tham chiếu phiên bản thay vì chép cấu hình;
  - cấu hình lớn đáng kể (ví dụ 3D);
  - cần lưu phiên bỏ dở;
  - có người dùng chung máy thật sự cần tách bài làm.
