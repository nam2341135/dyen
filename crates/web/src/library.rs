//! Nội dung học nhúng sẵn lúc biên dịch (xem `build.rs`).

use std::sync::OnceLock;

use content::{ContentErrors, Library, Sources};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/content.rs"));
}

/// Văn bản nguồn của thư mục `content/` tại lúc biên dịch.
pub fn embedded() -> Sources {
    let own = |files: &[(&str, &str)]| -> Vec<(String, String)> {
        files
            .iter()
            .map(|(n, t)| ((*n).to_owned(), (*t).to_owned()))
            .collect()
    };
    Sources {
        tags: embedded::TAGS.to_owned(),
        activities: own(embedded::ACTIVITIES),
        sets: own(embedded::SETS),
    }
}

/// Phân tích và kiểm tra nội dung nhúng (thuần, không IO).
pub fn load() -> Result<Library, ContentErrors> {
    Library::from_sources(&embedded())
}

/// Nội dung nhúng, phân tích một lần cho cả ứng dụng.
pub fn get() -> Result<&'static Library, &'static ContentErrors> {
    static LIB: OnceLock<Result<Library, ContentErrors>> = OnceLock::new();
    LIB.get_or_init(load).as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noi_dung_nhung_hop_le() {
        let lib = load().expect("nội dung nhúng hợp lệ");
        assert_eq!(lib.activities().len(), 4);
        let set = lib
            .sets()
            .values()
            .find(|s| s.id.as_str() == "frac-intro")
            .expect("có gói frac-intro");
        assert_eq!(set.activities.len(), 3);
    }
}
