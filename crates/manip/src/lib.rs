//! Mô hình (`Manipulative`): trạng thái, hành động, áp dụng và phát lại dấu vết.
//!
//! Crate thuần và tất định (ADR 0002, 0003): cùng trạng thái đầu + cùng dấu vết
//! cho cùng kết quả trên mọi nền tảng. Bộ hiển thị không bao giờ sửa trạng thái
//! trực tiếp; chúng chỉ phát `Action` vào đây. Hợp đồng được định nghĩa ở tầng 2.
