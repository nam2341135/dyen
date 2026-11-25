//! Mã định danh ổn định cho nội dung học.
//!
//! Định dạng: 1–64 ký tự `a-z`, `0-9`, `-`, `.`; không bắt đầu hay kết thúc bằng
//! `-`/`.`, không có `..`. Ví dụ: `frac.compare-same-numerator`.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Chuỗi không đúng định dạng mã định danh.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "mã `{0}` không hợp lệ: dùng 1–64 ký tự a-z, 0-9, '-', '.'; không bắt đầu/kết thúc bằng '-' hoặc '.', không có '..'"
)]
pub struct InvalidId(pub String);

fn is_valid(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.')
        && !s.starts_with(['-', '.'])
        && !s.ends_with(['-', '.'])
        && !s.contains("..")
}

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// Tạo mã sau khi kiểm tra định dạng.
            pub fn new(s: impl Into<String>) -> Result<Self, InvalidId> {
                let s = s.into();
                if is_valid(&s) { Ok(Self(s)) } else { Err(InvalidId(s)) }
            }

            /// Chuỗi của mã.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = InvalidId;
            fn try_from(s: String) -> Result<Self, InvalidId> {
                Self::new(s)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

id_type!(
    /// Mã một khái niệm trong đồ thị kiến thức.
    ConceptId
);
id_type!(
    /// Mã một hiểu sai đã biết.
    MisconceptionId
);
id_type!(
    /// Mã một hoạt động (trùng tên file nội dung).
    ActivityId
);
id_type!(
    /// Mã một gói hoạt động.
    ActivitySetId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chap_nhan_ma_hop_le() {
        for s in ["frac", "frac.unit-fraction", "a1.b2-c3", "x"] {
            assert!(ConceptId::new(s).is_ok(), "{s}");
        }
    }

    #[test]
    fn tu_choi_ma_sai() {
        let long = "a".repeat(65);
        for s in [
            "",
            "Frac",
            "frac..x",
            ".frac",
            "frac-",
            "phân-số",
            "a b",
            long.as_str(),
        ] {
            assert!(ConceptId::new(s).is_err(), "{s:?}");
        }
    }

    #[test]
    fn serde_kiem_tra_khi_doc() {
        let ok: ConceptId = serde_json::from_str("\"frac.unit\"").unwrap();
        assert_eq!(ok.as_str(), "frac.unit");
        assert!(serde_json::from_str::<ConceptId>("\"Frac\"").is_err());
        assert_eq!(serde_json::to_string(&ok).unwrap(), "\"frac.unit\"");
    }
}
