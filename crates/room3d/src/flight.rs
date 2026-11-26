//! Chuyển động camera: tư thế, hàm ease và chuyến bay có thể đổi đích giữa chừng.

use bevy::math::Vec3;

/// Tư thế camera: mắt và điểm nhìn (mét).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    /// Vị trí mắt.
    pub eye: Vec3,
    /// Điểm nhìn tới.
    pub look: Vec3,
}

/// Ease-in-out (smoothstep), cắt về [0, 1].
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sai số cho phép khi so thời điểm (f32).
const EPS: f32 = 1e-4;

/// Một chuyến bay camera từ tư thế này sang tư thế khác.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flight {
    from: Pose,
    to: Pose,
    t0: f32,
    dur: f32,
}

impl Flight {
    /// Chuyến bay bắt đầu ở `now`, kéo dài `dur` giây.
    pub fn new(from: Pose, to: Pose, now: f32, dur: f32) -> Self {
        Self {
            from,
            to,
            t0: now,
            dur,
        }
    }

    /// Tư thế tại thời điểm `now`.
    pub fn pose(&self, now: f32) -> Pose {
        if self.done(now) {
            return self.to;
        }
        let k = ease_in_out((now - self.t0) / self.dur);
        Pose {
            eye: self.from.eye.lerp(self.to.eye, k),
            look: self.from.look.lerp(self.to.look, k),
        }
    }

    /// Đã tới đích chưa.
    pub fn done(&self, now: f32) -> bool {
        now + EPS >= self.t0 + self.dur
    }

    /// Đổi đích giữa chừng: bay tiếp từ tư thế hiện tại, liền mạch, cùng thời lượng.
    pub fn retarget(&self, now: f32, to: Pose) -> Flight {
        Flight::new(self.pose(now), to, now, self.dur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::pose_near;
    use bevy::math::Vec3;

    fn p(x: f32) -> Pose {
        Pose {
            eye: Vec3::new(x, 1.0, 2.0),
            look: Vec3::new(0.0, x, 0.0),
        }
    }

    #[test]
    fn ease_dau_cuoi_giua() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-6);
        let mut prev = -1.0;
        for i in 0..=100 {
            let v = ease_in_out(i as f32 / 100.0);
            assert!(v > prev, "i={i}");
            prev = v;
        }
        assert_eq!(ease_in_out(-3.0), 0.0);
        assert_eq!(ease_in_out(7.0), 1.0);
    }

    #[test]
    fn bay_dung_hai_dau() {
        let f = Flight::new(p(0.0), p(10.0), 2.0, 0.7);
        assert!(pose_near(f.pose(2.0), p(0.0)));
        assert!(pose_near(f.pose(2.7), p(10.0)));
        assert!(!f.done(2.5));
        assert!(f.done(2.7));
        assert!(f.done(5.0));
    }

    #[test]
    fn doi_dich_lien_mach() {
        let f = Flight::new(p(0.0), p(10.0), 1.0, 0.7);
        let now = 1.3;
        let g = f.retarget(now, p(-5.0));
        assert!(pose_near(f.pose(now), g.pose(now)));
        assert!(pose_near(g.pose(now + 0.7), p(-5.0)));
    }
}
