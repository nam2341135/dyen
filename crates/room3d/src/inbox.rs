//! Hàng đợi thông điệp vỏ → phòng. `push` (wasm) xếp chuỗi JSON vào đây; mỗi khung Bevy lấy
//! hết ra theo đúng thứ tự đã gửi. Không ghi đè: vỏ có thể gửi `Init` rồi `Highlight` liền nhau.

use std::cell::RefCell;

thread_local! {
    // wasm chạy một luồng, nên hàng đợi theo luồng là đủ; test native mỗi test một luồng.
    static QUEUE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Thêm một thông điệp (chuỗi JSON của `ToRoom`) vào cuối hàng đợi.
pub fn push_raw(s: String) {
    QUEUE.with(|q| q.borrow_mut().push(s));
}

/// Lấy hết thông điệp đang chờ, theo thứ tự đã thêm; hàng đợi rỗng sau đó.
pub fn drain() -> Vec<String> {
    QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hang_doi_giu_thu_tu() {
        for s in ["a", "b", "c"] {
            push_raw(s.to_owned());
        }
        assert_eq!(drain(), ["a", "b", "c"]);
        assert!(drain().is_empty());
    }
}
