//! Kiểm tra toàn bộ thư mục `content/` của repo: nội dung sai làm CI đỏ (ADR 0009).

use std::path::Path;

#[test]
fn noi_dung_repo_hop_le() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    match content::Library::load(&dir) {
        Ok(lib) => assert!(!lib.activities().is_empty(), "chưa có hoạt động nào"),
        Err(e) => panic!("{e}"),
    }
}
