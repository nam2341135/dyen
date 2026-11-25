# ADR 0012: Tầng thư viện thuần dùng chung (`lib-*`) và toán chính xác `lib-exact`

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26
- **Sửa:** [ADR 0006](0006-ranh-gioi-crate.md) (thêm hàng `lib-*` vào bảng ranh giới)

## Bối cảnh

ADR 0002 và 0003 yêu cầu mọi quyết định logic phải tất định, dùng số nguyên hoặc phân số hữu tỉ; `f32/f64` chỉ dùng để vẽ. ADR 0006 và 0010 cho mỗi crate `kind-*` chỉ phụ thuộc `kernel`, để các ý tưởng không dính vào nhau.

Bản nháp [Làng Thợ](../ideas/lang-tho.md) cần ít nhất bốn loại hoạt động hình học không gian (`section`, `net`, `measure`, `build`) và một loại `short-answer`. Tất cả đều cần cùng một bộ công cụ: số hữu tỉ, vectơ 3D, mặt phẳng, giao tuyến, so sánh độ dài và góc, đọc đáp số như `3√2`. `kind-bars` đã có một kiểu `Fraction` viết riêng.

Với luật hiện tại, có ba lối đi và lối nào cũng tệ:

| Phương án | Vấn đề |
|---|---|
| Mỗi loại tự viết hình học riêng | Năm bản sao của phần khó và dễ sai nhất. Lỗi tràn số hay sai dấu được sửa ở chỗ này mà chỗ khác vẫn còn. |
| Cho `kind-*` phụ thuộc lẫn nhau | Các ý tưởng dính vào nhau. Gỡ một loại sẽ kéo theo loại khác, trái mục tiêu của ADR 0010. |
| Đưa toán vào `kernel` | Lõi phình ra, và mọi loại đều phải mang toán dù không dùng. `kernel` nên chỉ là hợp đồng. |

## Quyết định

### 1. Tầng mới: thư viện thuần `lib-*`

- **Vị trí và tên:** crate đặt ở `crates/libs/<tên>`, tên package là `lib-<tên>`.
- **Hướng phụ thuộc:** `lib-*` → `lib-*` khác (Cargo tự cấm vòng lặp). `kind-*` → `kernel` và `lib-*`. `kernel`, `registry`, `content`, `cli` **không** phụ thuộc `lib-*` khi chưa có ADR. Bộ hiển thị sau này (`web`, `table3d`) được phép, để đổi sang `f32` khi vẽ.
- **Là crate thuần:** cùng luật với `kernel`. Không IO, đồng hồ, thread hay số ngẫu nhiên; phụ thuộc ngoài chỉ trong danh sách cho phép.
- **Điều kiện để một thứ được vào `lib-*`:**
  - Trung lập: không biết `Kind`, dấu vết, nhãn hay sư phạm. Nó phải có nghĩa như một thư viện độc lập; đặt lên crates.io cũng không ai thấy lạ.
  - Có ít nhất hai loại hoạt động dùng, hoặc một loại đang dùng và một loại sắp làm đã có bản nháp cụ thể (CLAUDE.md: không trừu tượng trước ca dùng thứ hai).
  - Mỗi crate `lib-*` mới được ghi tên trong một ADR, kèm một câu nói nó chứa gì. Đây là mặt tiếp xúc dùng chung, nên thêm vào phải có chủ đích.
- **Kiểm tra bằng máy:** `xtask/src/deps.rs` thêm luật `("lib-*", &["lib-*"], true)` và cho `kind-*` được phụ thuộc thêm `lib-*`.

### 2. Crate đầu tiên: `lib-exact`

Chứa số học và hình học chính xác:

| Thành phần | Nội dung |
|---|---|
| `Rational` | Số hữu tỉ có dấu, tử và mẫu `i64`, luôn tối giản, mẫu > 0. Phép tính trung gian dùng `i128`. **Mọi phép tính đều `checked`** và trả `Result<_, ExactError>` (`Overflow` hoặc `DivisionByZero`), không panic, không tràn lặng lẽ. Serde dạng chuỗi `"-3/4"` (giống `Fraction` của `kind-bars`). |
| `Vec3`, `Point3` | Tọa độ `Rational`. Có cộng, trừ, tích vô hướng, tích có hướng. |
| `Line3`, `Plane`, `Segment3` | Mặt phẳng qua 3 điểm, giao đường với mặt, giao mặt với mặt, điểm nằm trên mặt, cùng phía hay khác phía. |
| Khoảng cách và góc | Không lấy căn: trả về **bình phương độ dài** (`Rational`) và **cos² của góc kèm dấu**. Mọi so sánh "dài hơn", "góc lớn hơn", "vuông góc" đều chính xác trên các số này. |
| `Surd` | Dạng `a√b` (`a: Rational`, `b` nguyên dương không có ước chính phương) để biểu diễn và **đọc đáp số**, ví dụ `"3√2"`, `"√6/2"`. Hai `Surd` so được bằng nhau. Chưa hỗ trợ tổng các căn khác nhau. |
| `to_f32()` / `to_f64()` | Chỉ dùng để vẽ. Doc comment nói rõ điều này. |

**Giới hạn đầu vào thay cho số lớn tùy ý:** không dùng bigint. Bù lại, loại hoạt động kiểm trong `init` rằng tọa độ của cấu hình nằm trong một giới hạn (ví dụ `|tử|, mẫu ≤ 10⁴`). Nhờ vậy các phép hình học trong phạm vi đề bài không tràn. Nếu vẫn tràn thì đó là `ConfigError` hoặc `Rejection`, không bao giờ là panic. Test hợp đồng có thêm trường hợp sát biên.

**Không thêm phụ thuộc ngoài.** Ta tự viết, không dùng `num-rational`, vì phạm vi nhỏ, cần kiểm soát tràn số theo cách riêng, và giữ nguyên danh sách cho phép (`serde`, `thiserror`). Nếu sau này cần CAS (rút gọn biểu thức, đa thức) thì đó là ADR khác.

### 3. Thời điểm làm

- **Không tạo `lib-exact` ngay khi chấp nhận ADR này.** Tạo nó cùng lúc với loại hình học đầu tiên (dự kiến `kind-section`), để API được định hình bởi một ca dùng thật.
- **Đã làm (2026-11-26):** `lib-exact` được tạo cùng `kind-section`, chỉ gồm `Rational`, `Vec3` (dùng cho cả điểm), `Plane` (qua 3 điểm, xét phía, giao với đoạn). `Line3`, `Segment3`, giao mặt–mặt, khoảng cách, góc và `Surd` để dành cho loại hoạt động cần đến chúng.
- `kind-bars` chuyển từ `Fraction` sang `lib_exact::Rational` sau đó, khi tiện, **với điều kiện snapshot dấu vết không đổi** (serde vẫn là `"1/3"`). Không bắt buộc.

## Hệ quả

- **Được:**
  - Một bản cài đặt duy nhất, test kỹ, cho phần dễ sai nhất.
  - Các loại hoạt động vẫn tách rời nhau: gỡ một `kind-*` không ảnh hưởng `lib-*`.
  - Lõi vẫn chỉ là hợp đồng.
- **Phải chấp nhận:**
  - Có thêm một tầng trong sơ đồ phụ thuộc.
  - Đổi API của một `lib-*` có thể làm vỡ nhiều loại cùng lúc. Bù lại, trình biên dịch và test hợp đồng của từng loại sẽ bắt được.
- **Giới hạn đã biết của tọa độ hữu tỉ:**
  - Nhiều hình đều (tam giác đều nằm ngang, lục giác đều nằm ngang, đường tròn) không có tọa độ hữu tỉ khi đặt thẳng trục. Nhưng vẫn dựng được nhiều hình đều có tọa độ hữu tỉ: tứ diện đều nội tiếp lập phương, lục giác đều là thiết diện của lập phương.
  - Nếu nội dung thật sự cần các hình kia, ta mở rộng sang trường `ℚ(√d)` với `d` cố định cho từng hoạt động. Đó là một ADR sửa đổi ADR này.
  - Khối tròn xoay (trụ, nón, cầu) có π. Thể tích được biểu diễn dạng `k·π` với `k` hữu tỉ.
- **Xem lại khi:**
  - tầng `lib-*` có hơn khoảng 5 crate, vì có thể nó đang thành nơi chứa đồ thừa;
  - một `lib-*` bắt đầu cần biết khái niệm của lõi;
  - giới hạn tọa độ hữu tỉ chặn nội dung thật.

## Việc đã làm khi chấp nhận

1. Sửa ADR 0006: thêm hàng `lib-*` vào bảng, sửa hàng `kind-*` thành "`kernel`, `lib-*`".
2. Sửa `xtask/src/deps.rs`: thêm luật `lib-*`, cho `kind-*` phụ thuộc thêm `lib-*`, và thêm test cho cả hai.
3. Sửa `CLAUDE.md`, mục Ranh giới: thêm `lib-*` và điều kiện để một thứ được vào `lib-*`.
4. Chưa tạo `crates/libs/exact` (xem mục 3 của Quyết định). Khi tạo, thêm `crates/libs/*` vào `members` của `Cargo.toml` gốc.
