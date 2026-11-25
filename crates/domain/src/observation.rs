//! Kết quả chẩn đoán và điểm số (ADR 0009).

use serde::{Deserialize, Serialize};

use crate::{ConceptId, MisconceptionId};

/// Một kết luận chẩn đoán rút ra từ dấu vết, kèm bằng chứng.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// Khái niệm liên quan.
    pub concept: ConceptId,
    /// Điều quan sát được.
    pub finding: Finding,
    /// Mức tin cậy.
    pub confidence: Confidence,
    /// Chỉ số các bước trong dấu vết làm bằng chứng (tăng dần).
    pub evidence: Vec<u32>,
}

/// Điều chẩn đoán quan sát được về một khái niệm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finding {
    /// Người học thể hiện hiểu sai này.
    Misconception(MisconceptionId),
    /// Người học từng thể hiện hiểu sai này rồi tự sửa trong cùng bài làm.
    SelfCorrected(MisconceptionId),
    /// Người học thể hiện hiểu đúng khái niệm.
    Demonstrated,
}

/// Mức tin cậy của một quan sát.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Một dấu hiệu đơn lẻ; cần thêm bằng chứng.
    Tentative,
    /// Dấu hiệu rõ hoặc lặp lại.
    Strong,
}

/// Điểm số tính bằng phần nghìn, `0..=1000`. Không dùng số thực (ADR 0009).
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
    fn quan_sat_serde() {
        let o = Observation {
            concept: ConceptId::new("frac.x").unwrap(),
            finding: Finding::Misconception(MisconceptionId::new("frac.y").unwrap()),
            confidence: Confidence::Strong,
            evidence: vec![3],
        };
        let json = serde_json::to_string(&o).unwrap();
        assert_eq!(
            json,
            r#"{"concept":"frac.x","finding":{"misconception":"frac.y"},"confidence":"strong","evidence":[3]}"#
        );
        assert_eq!(serde_json::from_str::<Observation>(&json).unwrap(), o);
    }
}
