# Tầm nhìn

> Một trang. Nếu một quyết định mâu thuẫn với trang này, sửa một trong hai — đừng để cả hai cùng tồn tại.

## Dự án là gì

Nền tảng học tập từ mầm non đến đại học, viết hoàn toàn bằng Rust, chạy trên web và điện thoại, coi trọng 3D và mỹ thuật, bằng tiếng Việt.

**Chưa chọn ý tưởng sư phạm nào làm cốt lõi.** Ý tưởng (thao tác mô hình, trắc nghiệm, chẩn đoán hiểu sai, mô phỏng…) là thứ sẽ được thêm, sửa, thay, bỏ thường xuyên. Việc của giai đoạn này là dựng kiến trúc để làm những việc đó rẻ và an toàn.

## Ràng buộc (do chủ dự án đặt, không thương lượng)

- **Rust cho toàn bộ mã ứng dụng** ([ADR 0001](adr/0001-rust-toan-bo.md)).
- **Web và mobile**; phải chạy được trên máy yếu.
- **3D và mỹ thuật là đầu tư nghiêm túc**, nhưng không chức năng học tập nào chỉ có ở 3D ([ADR 0002](adr/0002-mot-mo-hinh-nhieu-bo-hien-thi.md)).
- **Dữ liệu trẻ em: thu tối thiểu, không bên thứ ba** ([ADR 0005](adr/0005-du-lieu-tre-em.md)).
- **Phát triển bằng vibecode** (AI viết phần lớn code) — nên ranh giới phải do máy kiểm, không chỉ ghi trong tài liệu.

## Nguyên tắc kiến trúc

1. **Lõi trung lập.** Crate `kernel` không biết môn học, sư phạm hay cách chấm cụ thể nào. Nó chỉ định nghĩa hợp đồng chung: một *loại hoạt động* có cấu hình, trạng thái, đầu vào, hàm áp dụng và hàm đánh giá ([ADR 0010](adr/0010-loi-trung-lap.md)).
2. **Mỗi ý tưởng là một crate.** Một loại hoạt động sống trọn trong `crates/kinds/<tên>/`: kiểu, quy tắc, cách đánh giá, thuật ngữ riêng, test. Thêm ý tưởng = thêm thư mục; bỏ ý tưởng = xóa thư mục. Các loại không phụ thuộc nhau.
3. **Một chỗ duy nhất biết mọi loại.** Crate `registry` là enum đóng liệt kê các loại; trình biên dịch chỉ ra mọi chỗ cần sửa khi thêm/bớt. Không hệ plugin động.
4. **Logic thuần, tách khỏi IO và hiển thị.** Mọi quyết định học tập là hàm thuần, tất định, chạy giống nhau ở client, server và test ([ADR 0003](adr/0003-dau-vet-la-nguon-su-that.md)).
5. **Hợp đồng được kiểm bằng máy.** Mỗi loại hoạt động chạy qua cùng bộ kiểm `kernel::contract` (tất định, từ chối không đổi trạng thái, phát lại khớp, bằng chứng hợp lệ). Ranh giới crate do `cargo xtask deps` kiểm ([ADR 0006](adr/0006-ranh-gioi-crate.md)).
6. **Nội dung là dữ liệu.** Hoạt động là file TOML trong git, kiểm tra lúc CI ([ADR 0004](adr/0004-noi-dung-la-file.md)).

## Giai đoạn hiện tại: nền móng

Chưa xây sản phẩm. Có hai loại hoạt động mẫu (`kind-bars`: thanh cắt được + so sánh phân số; `kind-choice`: chọn đáp án) để chứng minh hợp đồng dùng được — chúng không phải định hướng sản phẩm. Xem [docs/README.md](README.md).
