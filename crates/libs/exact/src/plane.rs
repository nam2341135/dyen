//! Mặt phẳng [`Plane`].

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{ExactError, Rational, Vec3};

/// Mặt phẳng `normal · p = offset`, `normal` khác 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plane {
    normal: Vec3,
    offset: Rational,
}

impl Plane {
    /// Mặt phẳng qua ba điểm; `Ok(None)` nếu ba điểm thẳng hàng (kể cả trùng nhau).
    pub fn through(a: Vec3, b: Vec3, c: Vec3) -> Result<Option<Self>, ExactError> {
        let normal = b.checked_sub(a)?.cross(c.checked_sub(a)?)?;
        if normal.is_zero() {
            return Ok(None);
        }
        Ok(Some(Self {
            normal,
            offset: normal.dot(a)?,
        }))
    }

    /// Vectơ pháp tuyến (không chuẩn hóa).
    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    /// `normal · p - offset`: bằng 0 khi `p` nằm trên mặt; dấu cho biết phía.
    /// Độ lớn tỉ lệ với khoảng cách, không phải chính khoảng cách.
    pub fn eval(&self, p: Vec3) -> Result<Rational, ExactError> {
        self.normal.dot(p)?.checked_sub(self.offset)
    }

    /// Phía của `p`: `Equal` nếu nằm trên mặt; `Less`/`Greater` là hai phía.
    pub fn side(&self, p: Vec3) -> Result<Ordering, ExactError> {
        Ok(self.eval(p)?.signum())
    }

    /// Giao của mặt với đoạn `a → b`, dạng tham số `t ∈ [0, 1]` (điểm `a + t·(b - a)`).
    /// `None` nếu đoạn không cắt mặt hoặc nằm hẳn trên mặt.
    pub fn cut_segment(&self, a: Vec3, b: Vec3) -> Result<Option<Rational>, ExactError> {
        let (ea, eb) = (self.eval(a)?, self.eval(b)?);
        if ea.signum() == eb.signum() {
            // Cùng phía, hoặc cả hai đầu trên mặt.
            return Ok(None);
        }
        // Khác dấu nên ea != eb: mẫu khác 0.
        Ok(Some(ea.checked_div(ea.checked_sub(eb)?)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> Rational {
        s.parse().unwrap()
    }

    fn diagonal() -> Plane {
        Plane::through(Vec3::int(1, 0, 0), Vec3::int(0, 1, 0), Vec3::int(0, 0, 1))
            .unwrap()
            .unwrap()
    }

    #[test]
    fn qua_ba_diem() {
        let p = diagonal();
        assert_eq!(p.side(Vec3::ZERO).unwrap(), Ordering::Less);
        assert_eq!(p.side(Vec3::int(1, 1, 1)).unwrap(), Ordering::Greater);
        let on = Vec3::new(r("1/2"), r("1/2"), r("0"));
        assert_eq!(p.side(on).unwrap(), Ordering::Equal);
    }

    #[test]
    fn thang_hang_khong_co_mat() {
        let a = Vec3::ZERO;
        assert_eq!(
            Plane::through(a, Vec3::int(1, 1, 1), Vec3::int(2, 2, 2)),
            Ok(None)
        );
        assert_eq!(Plane::through(a, a, Vec3::int(1, 0, 0)), Ok(None));
    }

    #[test]
    fn cat_doan() {
        let p = diagonal();
        let t = p.cut_segment(Vec3::ZERO, Vec3::int(3, 0, 0)).unwrap();
        assert_eq!(t, Some(r("1/3")));
        assert_eq!(p.cut_segment(Vec3::ZERO, Vec3::ZERO).unwrap(), None);
        // Đầu mút nằm trên mặt: t = 0.
        let t = p
            .cut_segment(Vec3::int(1, 0, 0), Vec3::int(2, 0, 0))
            .unwrap();
        assert_eq!(t, Some(Rational::ZERO));
        // Đoạn nằm hẳn trên mặt: không có giao điểm duy nhất.
        let t = p
            .cut_segment(Vec3::int(1, 0, 0), Vec3::int(0, 1, 0))
            .unwrap();
        assert_eq!(t, None);
    }
}
