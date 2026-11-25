//! Phân số không âm, số học nguyên tất định (ADR 0003).

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Phân số không âm, luôn ở dạng tối giản, mẫu số > 0.
///
/// Biểu diễn văn bản (và serde) là `"tử/mẫu"`, ví dụ `"1/3"`; số nguyên viết `"2"`.
/// So sánh bằng nhân chéo trên `u64`, không dùng số thực.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Fraction {
    num: u32,
    den: u32,
}

/// Lỗi tạo hoặc đọc phân số.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FractionError {
    /// Mẫu số bằng 0.
    #[error("mẫu số bằng 0")]
    ZeroDenominator,
    /// Chuỗi không có dạng `tử/mẫu`.
    #[error("`{0}` không phải phân số dạng tử/mẫu")]
    Parse(String),
}

impl Fraction {
    /// Số 0.
    pub const ZERO: Self = Self { num: 0, den: 1 };

    /// Tạo phân số và rút gọn.
    pub fn new(num: u32, den: u32) -> Result<Self, FractionError> {
        if den == 0 {
            return Err(FractionError::ZeroDenominator);
        }
        let g = gcd(num, den);
        Ok(Self {
            num: num / g,
            den: den / g,
        })
    }

    /// Tử số (sau rút gọn).
    pub fn num(self) -> u32 {
        self.num
    }

    /// Mẫu số (sau rút gọn).
    pub fn den(self) -> u32 {
        self.den
    }

    /// So sánh với phân số khác, trả về quan hệ `self ? other`.
    pub fn relation(self, other: Self) -> Relation {
        self.cmp(&other).into()
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> Ordering {
        (u64::from(self.num) * u64::from(other.den))
            .cmp(&(u64::from(other.num) * u64::from(self.den)))
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl FromStr for Fraction {
    type Err = FractionError;

    fn from_str(s: &str) -> Result<Self, FractionError> {
        let bad = || FractionError::Parse(s.to_owned());
        let (n, d) = match s.trim().split_once('/') {
            Some((n, d)) => (n.trim(), d.trim()),
            None => (s.trim(), "1"),
        };
        let num = n.parse().map_err(|_| bad())?;
        let den = d.parse().map_err(|_| bad())?;
        Self::new(num, den)
    }
}

impl TryFrom<String> for Fraction {
    type Error = FractionError;
    fn try_from(s: String) -> Result<Self, FractionError> {
        s.parse()
    }
}

impl From<Fraction> for String {
    fn from(f: Fraction) -> String {
        f.to_string()
    }
}

/// Quan hệ so sánh giữa hai đại lượng — dùng cho câu trả lời `<`, `=`, `>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// Bé hơn.
    Less,
    /// Bằng.
    Equal,
    /// Lớn hơn.
    Greater,
}

impl Relation {
    /// Quan hệ ngược chiều (`<` ↔ `>`; `=` giữ nguyên).
    pub fn reversed(self) -> Self {
        match self {
            Self::Less => Self::Greater,
            Self::Equal => Self::Equal,
            Self::Greater => Self::Less,
        }
    }
}

impl From<Ordering> for Relation {
    fn from(o: Ordering) -> Self {
        match o {
            Ordering::Less => Self::Less,
            Ordering::Equal => Self::Equal,
            Ordering::Greater => Self::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(s: &str) -> Fraction {
        s.parse().unwrap()
    }

    #[test]
    fn rut_gon_khi_tao() {
        let x = Fraction::new(280, 840).unwrap();
        assert_eq!((x.num(), x.den()), (1, 3));
        assert_eq!(f("2/4"), f("1/2"));
        assert_eq!(f("0/5"), f("0/1"));
    }

    #[test]
    fn so_sanh_bang_nhan_cheo() {
        assert_eq!(f("1/3").relation(f("1/5")), Relation::Greater);
        assert_eq!(f("2/6").relation(f("1/3")), Relation::Equal);
        assert_eq!(f("3/4").relation(f("4/5")), Relation::Less);
        // Không tràn số với giá trị lớn.
        assert_eq!(
            Fraction::new(u32::MAX, u32::MAX - 1)
                .unwrap()
                .relation(f("1")),
            Relation::Greater
        );
    }

    #[test]
    fn doc_va_ghi_chuoi() {
        assert_eq!(f(" 3 / 4 ").to_string(), "3/4");
        assert_eq!(f("4/2").to_string(), "2");
        assert_eq!(f("7").to_string(), "7");
        assert_eq!(
            "1/0".parse::<Fraction>(),
            Err(FractionError::ZeroDenominator)
        );
        assert!("a/3".parse::<Fraction>().is_err());
        assert!("-1/3".parse::<Fraction>().is_err());
    }

    #[test]
    fn serde_dung_chuoi() {
        let x: Fraction = serde_json::from_str("\"2/6\"").unwrap();
        assert_eq!(serde_json::to_string(&x).unwrap(), "\"1/3\"");
    }

    #[test]
    fn quan_he_nguoc() {
        assert_eq!(Relation::Less.reversed(), Relation::Greater);
        assert_eq!(Relation::Equal.reversed(), Relation::Equal);
    }
}
