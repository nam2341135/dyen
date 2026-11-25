//! Vectơ/điểm 3D [`Vec3`] với tọa độ hữu tỉ.

use serde::{Deserialize, Serialize};

use crate::{ExactError, Rational};

/// Vectơ (hoặc điểm) 3D, tọa độ [`Rational`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vec3 {
    /// Hoành độ.
    pub x: Rational,
    /// Tung độ.
    pub y: Rational,
    /// Cao độ.
    pub z: Rational,
}

impl Vec3 {
    /// Vectơ 0.
    pub const ZERO: Self = Self::int(0, 0, 0);

    /// Từ ba tọa độ.
    pub const fn new(x: Rational, y: Rational, z: Rational) -> Self {
        Self { x, y, z }
    }

    /// Từ ba tọa độ nguyên.
    pub const fn int(x: i64, y: i64, z: i64) -> Self {
        Self::new(Rational::int(x), Rational::int(y), Rational::int(z))
    }

    /// `self + rhs`.
    pub fn checked_add(self, rhs: Self) -> Result<Self, ExactError> {
        Ok(Self::new(
            self.x.checked_add(rhs.x)?,
            self.y.checked_add(rhs.y)?,
            self.z.checked_add(rhs.z)?,
        ))
    }

    /// `self - rhs`.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ExactError> {
        Ok(Self::new(
            self.x.checked_sub(rhs.x)?,
            self.y.checked_sub(rhs.y)?,
            self.z.checked_sub(rhs.z)?,
        ))
    }

    /// `k · self`.
    pub fn scale(self, k: Rational) -> Result<Self, ExactError> {
        Ok(Self::new(
            self.x.checked_mul(k)?,
            self.y.checked_mul(k)?,
            self.z.checked_mul(k)?,
        ))
    }

    /// Tích vô hướng.
    pub fn dot(self, rhs: Self) -> Result<Rational, ExactError> {
        self.x
            .checked_mul(rhs.x)?
            .checked_add(self.y.checked_mul(rhs.y)?)?
            .checked_add(self.z.checked_mul(rhs.z)?)
    }

    /// Tích có hướng.
    pub fn cross(self, rhs: Self) -> Result<Self, ExactError> {
        Ok(Self::new(
            self.y
                .checked_mul(rhs.z)?
                .checked_sub(self.z.checked_mul(rhs.y)?)?,
            self.z
                .checked_mul(rhs.x)?
                .checked_sub(self.x.checked_mul(rhs.z)?)?,
            self.x
                .checked_mul(rhs.y)?
                .checked_sub(self.y.checked_mul(rhs.x)?)?,
        ))
    }

    /// Điểm trên đoạn `self → to` ứng với tham số `t` (`t = 0` là `self`, `t = 1` là `to`).
    pub fn lerp(self, to: Self, t: Rational) -> Result<Self, ExactError> {
        self.checked_add(to.checked_sub(self)?.scale(t)?)
    }

    /// Là vectơ 0.
    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> Rational {
        s.parse().unwrap()
    }

    #[test]
    fn phep_tinh_vecto() {
        let a = Vec3::int(1, 0, 0);
        let b = Vec3::int(0, 1, 0);
        assert_eq!(a.cross(b).unwrap(), Vec3::int(0, 0, 1));
        assert_eq!(a.dot(b).unwrap(), Rational::ZERO);
        assert_eq!(
            a.lerp(b, r("1/3")).unwrap(),
            Vec3::new(r("2/3"), r("1/3"), Rational::ZERO)
        );
        assert!(a.cross(a).unwrap().is_zero());
    }

    #[test]
    fn serde() {
        let v = Vec3::new(r("1/2"), r("-1"), r("0"));
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, r#"{"x":"1/2","y":"-1","z":"0"}"#);
        assert_eq!(serde_json::from_str::<Vec3>(&json).unwrap(), v);
    }
}
