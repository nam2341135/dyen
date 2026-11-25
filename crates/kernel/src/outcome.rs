//! Kết quả đánh giá một dấu vết (ADR 0010).
//!
//! Lõi không biết ý nghĩa của nhãn: mỗi loại hoạt động tự khai báo nhãn nó dùng
//! ([`crate::Kind::TAGS`], [`crate::Kind::config_tags`]), nội dung đặt tên hiển thị cho nhãn.

use serde::{Deserialize, Serialize};

use crate::TagId;

/// Kết quả đánh giá: điểm, đã hoàn thành chưa, và các phát hiện kèm bằng chứng.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// Điểm, phần nghìn.
    pub score: Score,
    /// Bài làm đã đạt yêu cầu của hoạt động chưa.
    pub completed: bool,
    /// Các phát hiện, theo thứ tự loại hoạt động đưa ra.
    pub findings: Vec<Finding>,
}

/// Một dấu hiệu rút ra từ dấu vết, gắn với một nhãn.
///
/// Lõi không biết nhãn là điều tốt hay xấu; nhãn và nghĩa của nó do loại hoạt động định nghĩa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Nhãn (phải nằm trong `Kind::TAGS` hoặc `Kind::config_tags` của loại hoạt động).
    pub tag: TagId,
    /// `true` nếu dấu hiệu đã xuất hiện rồi người học tự rời khỏi nó trong cùng bài làm.
    pub recovered: bool,
    /// Mức tin cậy.
    pub confidence: Confidence,
    /// Chỉ số các bước của người học làm bằng chứng, tăng dần, không trùng.
    pub evidence: Vec<u32>,
}

/// Mức tin cậy của một phát hiện.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Một dấu hiệu đơn lẻ.
    Tentative,
    /// Dấu hiệu rõ hoặc lặp lại.
    Strong,
}

/// Điểm tính bằng phần nghìn, `0..=1000`. Không dùng số thực (ADR 0003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Score(u16);

impl Score {
    /// Điểm 0.
    pub const ZERO: Self = Self(0);
    /// Điểm tối đa (1000‰).
    pub const FULL: Self = Self(1000);

    /// Tạo điểm từ phần nghìn; `None` nếu lớn hơn 1000.
    pub const fn from_permille(p: u16) -> Option<Self> {
        if p <= 1000 { Some(Self(p)) } else { None }
    }

    /// Giá trị phần nghìn.
    pub const fn permille(self) -> u16 {
        self.0
    }
}

/// Giá trị điểm vượt quá 1000‰.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("điểm {0}‰ vượt quá 1000‰")]
pub struct ScoreOutOfRange(pub u16);

impl TryFrom<u16> for Score {
    type Error = ScoreOutOfRange;
    fn try_from(p: u16) -> Result<Self, ScoreOutOfRange> {
        Self::from_permille(p).ok_or(ScoreOutOfRange(p))
    }
}

impl From<Score> for u16 {
    fn from(s: Score) -> u16 {
        s.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diem_trong_khoang() {
        assert_eq!(Score::from_permille(1000), Some(Score::FULL));
        assert_eq!(Score::from_permille(1001), None);
        assert!(serde_json::from_str::<Score>("1001").is_err());
        assert_eq!(
            serde_json::from_str::<Score>("500").unwrap().permille(),
            500
        );
    }

    #[test]
    fn phat_hien_serde() {
        let f = Finding {
            tag: TagId::new("x.y").unwrap(),
            recovered: true,
            confidence: Confidence::Strong,
            evidence: vec![3],
        };
        let json = serde_json::to_string(&f).unwrap();
        assert_eq!(
            json,
            r#"{"tag":"x.y","recovered":true,"confidence":"strong","evidence":[3]}"#
        );
        assert_eq!(serde_json::from_str::<Finding>(&json).unwrap(), f);
    }
}
