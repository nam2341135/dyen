# ADR 0011: Loại hoạt động công bố thêm nhãn từ cấu hình

- **Trạng thái:** Chấp nhận
- **Ngày:** 2026-11-26
- **Mở rộng:** [ADR 0010](0010-loi-trung-lap.md) (không thay thế)

## Bối cảnh

ADR 0010 cho mỗi loại hoạt động công bố nhãn bằng hằng `Kind::TAGS`. Điều đó ngầm giả định mọi nhãn loại đó có thể gắn đều biết lúc biên dịch.

Loại thứ hai, `choice` (chọn đáp án), cho thấy giả định này sai. Loại này không gắn với môn học. Ý nghĩa của từng phương án sai, ví dụ "chọn 1/8 vì nghĩ mẫu lớn hơn thì lớn hơn", do người soạn nội dung biết, không phải do code biết. Nếu nhãn bị ép vào `TAGS`, crate `kind-choice` phải liệt kê nhãn của mọi môn học, tức đưa sư phạm cụ thể vào một loại vốn trung lập. Nếu bỏ nhãn đi, loại này mất phát hiện, và phát hiện là thứ lõi sinh ra để chở.

## Quyết định

Thêm vào `kernel::Kind` một hàm mặc định:

```rust
fn config_tags(config: &Self::Config) -> Vec<TagId> { Vec::new() }
```

- Đây là các nhãn do cấu hình đặt mà `evaluate` có thể gắn, **ngoài** `TAGS`. Mặc định là rỗng, nên các loại hiện có không phải sửa gì.
- `kernel::contract::check` chấp nhận nhãn trong `TAGS ∪ config_tags(config)`.
- `registry::AnyConfig::config_tags()` chuyển tiếp lời gọi. `registry::all_tags()` vẫn chỉ gồm `TAGS`, vì đó là nhãn có sẵn bất kể nội dung.
- `content` (`dyen check`) kiểm: mọi nhãn trong `config_tags` của từng hoạt động phải có trong `content/tags.toml`. Lỗi được báo theo file hoạt động.

## Hệ quả

- Loại trung lập với môn học vẫn sinh được phát hiện có nhãn. Ý nghĩa của nhãn vẫn nằm trong nội dung (`tags.toml`), không nằm trong code.
- Một nhãn giờ có hai nguồn. Muốn biết "mọi nhãn đang dùng" thì phải duyệt nội dung, không chỉ đọc hằng. `dyen check` đã làm việc này.
- Không đổi định dạng dấu vết, không đổi `Outcome`, và không có snapshot cũ nào đổi.
- Xem lại quyết định này nếu cần nhãn phụ thuộc vào thứ khác ngoài cấu hình, ví dụ dữ liệu ngoài dấu vết. Đó sẽ là một mở rộng lõi khác và cần ADR riêng.
