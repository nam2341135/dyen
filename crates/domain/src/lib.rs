//! Kiểu dùng chung cho mọi crate: mã định danh, phân số, quan sát chẩn đoán, điểm số.
//!
//! Crate thuần (ADR 0006): không IO, không mạng, không đồng hồ.
//! Thuật ngữ theo `docs/glossary.md`; hợp đồng theo ADR 0009.

mod fraction;
mod id;
mod observation;

pub use fraction::{Fraction, FractionError, Relation};
pub use id::{ActivityId, ActivitySetId, ConceptId, InvalidId, MisconceptionId};
pub use observation::{Confidence, Finding, Observation, Score, ScoreOutOfRange};
