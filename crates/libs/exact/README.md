# lib-exact

Số học và hình học 3D **chính xác** cho mọi quyết định logic (ADR 0002, 0003, 0012). Không biết `Kind`, dấu vết, nhãn hay sư phạm.

| Kiểu | Nội dung |
|---|---|
| `Rational` | Số hữu tỉ tối giản, tử/mẫu `i64`, trung gian `i128`. Mọi phép tính trả `Result<_, ExactError>`: không panic, không tràn lặng lẽ. Serde dạng chuỗi `"-3/4"`, `"2"`. |
| `Vec3` | Vectơ/điểm 3D tọa độ `Rational`: cộng, trừ (`checked_add`, `checked_sub`), nhân vô hướng, tích vô hướng, tích có hướng, nội suy trên đoạn. |
| `Plane` | Mặt phẳng `n·p = d`: dựng qua 3 điểm, xét điểm nằm phía nào, giao với đoạn thẳng. |

`to_f64()` chỉ dùng để vẽ.

Chưa có (thêm khi có loại hoạt động cần, ADR 0012): `Line3`, `Segment3`, giao mặt–mặt, `Surd` (`a√b`), khoảng cách bình phương, cos² góc.

**Không bigint.** Loại hoạt động phải giới hạn đầu vào trong `init` để phép tính trong phạm vi đề bài không tràn; nếu vẫn tràn thì báo lỗi, không panic.
