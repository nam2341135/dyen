//! Số hữu tỉ [`Rational`].

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ExactError;

/// Số hữu tỉ, luôn tối giản, mẫu > 0. Bằng nhau theo giá trị nhờ dạng chuẩn duy nhất.
///
/// Serde dạng chuỗi: `"-3/4"`, `"2"`. Đọc chấp nhận dạng chưa tối giản (`"2/4"`) và tối giản lại.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rational {
    num: i64,
    den: i64,
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Rational {
    /// Số 0.
    pub const ZERO: Self = Self { num: 0, den: 1 };
    /// Số 1.
    pub const ONE: Self = Self { num: 1, den: 1 };

    /// Số nguyên.
    pub const fn int(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    /// `num / den`, tối giản.
    pub fn new(num: i64, den: i64) -> Result<Self, ExactError> {
        Self::from_i128(i128::from(num), i128::from(den))
    }

    fn from_i128(num: i128, den: i128) -> Result<Self, ExactError> {
        if den == 0 {
            return Err(ExactError::DivisionByZero);
        }
        // g >= 1 vì den != 0, và g <= |den| nên vừa i128.
        let g = i128::try_from(gcd(num.unsigned_abs(), den.unsigned_abs()))
            .map_err(|_| ExactError::Overflow)?;
        let (mut n, mut d) = (num / g, den / g);
        if d < 0 {
            n = n.checked_neg().ok_or(ExactError::Overflow)?;
            d = d.checked_neg().ok_or(ExactError::Overflow)?;
        }
        Ok(Self {
            num: i64::try_from(n).map_err(|_| ExactError::Overflow)?,
            den: i64::try_from(d).map_err(|_| ExactError::Overflow)?,
        })
    }

    /// Tử số (đã tối giản, mang dấu).
    pub const fn numer(self) -> i64 {
        self.num
    }

    /// Mẫu số (đã tối giản, > 0).
    pub const fn denom(self) -> i64 {
        self.den
    }

    /// Bằng 0.
    pub const fn is_zero(self) -> bool {
        self.num == 0
    }

    /// Dấu: `Less` nếu âm, `Equal` nếu 0, `Greater` nếu dương.
    pub fn signum(self) -> Ordering {
        self.num.cmp(&0)
    }

    /// `self + rhs`.
    pub fn checked_add(self, rhs: Self) -> Result<Self, ExactError> {
        let (a, b, c, d) = self.wide(rhs);
        // |a·d|, |c·b| <= 2^63·(2^63 - 1) nên tổng vừa i128; vẫn kiểm cho chắc.
        let n = (a * d).checked_add(c * b).ok_or(ExactError::Overflow)?;
        Self::from_i128(n, b * d)
    }

    /// `self - rhs`.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ExactError> {
        let (a, b, c, d) = self.wide(rhs);
        let n = (a * d).checked_sub(c * b).ok_or(ExactError::Overflow)?;
        Self::from_i128(n, b * d)
    }

    /// `self * rhs`.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, ExactError> {
        let (a, b, c, d) = self.wide(rhs);
        Self::from_i128(a * c, b * d)
    }

    /// `self / rhs`.
    pub fn checked_div(self, rhs: Self) -> Result<Self, ExactError> {
        let (a, b, c, d) = self.wide(rhs);
        Self::from_i128(a * d, b * c)
    }

    /// `-self`.
    pub fn checked_neg(self) -> Result<Self, ExactError> {
        Ok(Self {
            num: self.num.checked_neg().ok_or(ExactError::Overflow)?,
            den: self.den,
        })
    }

    /// Hai số dưới dạng `i128`: `(tử self, mẫu self, tử rhs, mẫu rhs)`.
    /// Tích của hai giá trị `i64` luôn vừa `i128`.
    fn wide(self, rhs: Self) -> (i128, i128, i128, i128) {
        (
            i128::from(self.num),
            i128::from(self.den),
            i128::from(rhs.num),
            i128::from(rhs.den),
        )
    }

    /// Giá trị gần đúng — **chỉ để vẽ**, không dùng cho quyết định logic.
    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        // Tích hai giá trị i64 vừa i128: không tràn.
        (i128::from(self.num) * i128::from(other.den))
            .cmp(&(i128::from(other.num) * i128::from(self.den)))
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<i64> for Rational {
    fn from(n: i64) -> Self {
        Self::int(n)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl fmt::Debug for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Chuỗi không phải số hữu tỉ dạng `p` hoặc `p/q`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` không phải số hữu tỉ dạng `p` hoặc `p/q`")]
pub struct ParseRationalError(pub String);

impl FromStr for Rational {
    type Err = ParseRationalError;

    fn from_str(s: &str) -> Result<Self, ParseRationalError> {
        let err = || ParseRationalError(s.to_owned());
        let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
        let (n, d) = s.split_once('/').unwrap_or((s, "1"));
        if !digits(n.strip_prefix('-').unwrap_or(n)) || !digits(d) {
            return Err(err());
        }
        let n: i64 = n.parse().map_err(|_| err())?;
        let d: i64 = d.parse().map_err(|_| err())?;
        Self::new(n, d).map_err(|_| err())
    }
}

impl TryFrom<String> for Rational {
    type Error = ParseRationalError;
    fn try_from(s: String) -> Result<Self, ParseRationalError> {
        s.parse()
    }
}

impl From<Rational> for String {
    fn from(r: Rational) -> String {
        r.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> Rational {
        s.parse().unwrap()
    }

    #[test]
    fn toi_gian_va_dau() {
        assert_eq!(r("2/4"), r("1/2"));
        assert_eq!(Rational::new(3, -6).unwrap().to_string(), "-1/2");
        assert_eq!(r("-0/5"), Rational::ZERO);
        assert_eq!(r("6/3").to_string(), "2");
        assert_eq!(Rational::new(1, 0), Err(ExactError::DivisionByZero));
    }

    #[test]
    fn doc_chuoi() {
        for bad in [
            "", "1/", "/2", "+1", "1/-2", "a", "1.5", "1/0", " 1", "--1", "-",
        ] {
            assert!(bad.parse::<Rational>().is_err(), "`{bad}`");
        }
        assert_eq!(r("-3/4").to_string(), "-3/4");
    }

    #[test]
    fn phep_tinh() {
        assert_eq!(r("1/2").checked_add(r("1/3")).unwrap(), r("5/6"));
        assert_eq!(r("1/2").checked_sub(r("1/3")).unwrap(), r("1/6"));
        assert_eq!(r("2/3").checked_mul(r("-3/4")).unwrap(), r("-1/2"));
        assert_eq!(r("2/3").checked_div(r("4/9")).unwrap(), r("3/2"));
        assert_eq!(
            r("1").checked_div(Rational::ZERO),
            Err(ExactError::DivisionByZero)
        );
    }

    #[test]
    fn so_sanh() {
        assert!(r("1/3") > r("1/5"));
        assert!(r("-1/2") < r("-1/3"));
        assert_eq!(r("-1/2").signum(), Ordering::Less);
        assert_eq!(Rational::ZERO.signum(), Ordering::Equal);
        assert!(Rational::int(i64::MAX) > Rational::int(i64::MAX - 1));
        assert!(Rational::int(i64::MIN) < Rational::new(i64::MIN + 1, i64::MAX).unwrap());
    }

    #[test]
    fn tran_so_bao_loi_khong_panic() {
        let max = Rational::int(i64::MAX);
        let min = Rational::int(i64::MIN);
        assert_eq!(max.checked_add(Rational::ONE), Err(ExactError::Overflow));
        assert_eq!(max.checked_mul(max), Err(ExactError::Overflow));
        assert_eq!(min.checked_neg(), Err(ExactError::Overflow));
        assert_eq!(min.checked_sub(max), Err(ExactError::Overflow));
        let tiny = Rational::new(1, i64::MAX).unwrap();
        assert_eq!(tiny.checked_mul(tiny), Err(ExactError::Overflow));
        assert_eq!(Rational::new(i64::MIN, -1), Err(ExactError::Overflow));
        // Trung gian gần 2^127 vẫn vừa i128, kết quả không vừa i64: báo lỗi, không panic.
        let a = Rational::new(i64::MIN, i64::MAX).unwrap();
        let b = Rational::new(i64::MIN, i64::MAX - 1).unwrap();
        assert_eq!(a.checked_add(b), Err(ExactError::Overflow));
    }

    #[test]
    fn serde_chuoi() {
        let json = serde_json::to_string(&r("-3/4")).unwrap();
        assert_eq!(json, r#""-3/4""#);
        assert_eq!(serde_json::from_str::<Rational>(&json).unwrap(), r("-3/4"));
        assert!(serde_json::from_str::<Rational>(r#""1/0""#).is_err());
        assert!(serde_json::from_str::<Rational>("1").is_err());
    }
}
