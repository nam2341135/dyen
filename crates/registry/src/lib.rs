//! Sổ đăng ký loại hoạt động: danh sách **đóng** các [`Kind`] mà hệ thống biết (ADR 0010).
//!
//! Đây là chỗ duy nhất trong workspace rẽ nhánh theo loại hoạt động. Thêm một loại:
//! 1. tạo crate `crates/kinds/<tên>` hiện thực [`Kind`];
//! 2. thêm một biến thể vào [`AnyConfig`] và [`AnyState`], rồi sửa các `match` mà
//!    trình biên dịch chỉ ra (mỗi `match` một dòng);
//! 3. thêm crate vào `xtask/src/deps.rs` (tự động nếu tên bắt đầu bằng `kind-`).
//!
//! Gỡ một loại: làm ngược lại. Không có cơ chế plugin động — cố ý.

use kernel::{ConfigError, Kind, Outcome, ReplayError, TagId, Trace};
use kind_bars::Bars;
use kind_choice::Choice;
use serde::{Deserialize, Deserializer, Serialize};

/// Cấu hình của mọi loại hoạt động. TOML: bảng `[config]` với `kind = "<tên loại>"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnyConfig {
    /// Thanh cắt được và thước so sánh.
    Bars(kind_bars::Config),
    /// Chọn một hoặc nhiều đáp án.
    Choice(kind_choice::Config),
}

/// Trạng thái của mọi loại hoạt động (chỉ để báo cáo; JSON không mang tên loại).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AnyState {
    /// Trạng thái thanh.
    Bars(kind_bars::State),
    /// Trạng thái chọn đáp án.
    Choice(kind_choice::State),
}

/// Tên mọi loại hoạt động đã đăng ký.
pub const KINDS: &[&str] = &[Bars::KIND, Choice::KIND];

impl AnyConfig {
    /// Tên loại hoạt động.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Bars(_) => Bars::KIND,
            Self::Choice(_) => Choice::KIND,
        }
    }

    /// Nhãn cố định mà loại hoạt động này có thể gắn vào phát hiện (`Kind::TAGS`).
    pub fn tags(&self) -> &'static [&'static str] {
        match self {
            Self::Bars(_) => Bars::TAGS,
            Self::Choice(_) => Choice::TAGS,
        }
    }

    /// Nhãn do người soạn đặt trong chính cấu hình này (`Kind::config_tags`, ADR 0011).
    pub fn config_tags(&self) -> Vec<TagId> {
        match self {
            Self::Bars(c) => Bars::config_tags(c),
            Self::Choice(c) => Choice::config_tags(c),
        }
    }

    /// Kiểm tra cấu hình bằng cách dựng thử trạng thái đầu.
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self {
            Self::Bars(c) => Bars::init(c).map(|_| ()),
            Self::Choice(c) => Choice::init(c).map(|_| ()),
        }
    }
}

/// Mọi nhãn cố định (`Kind::TAGS`) của mọi loại hoạt động, sắp xếp, không trùng.
/// Nhãn từ cấu hình xem [`AnyConfig::config_tags`].
pub fn all_tags() -> Vec<&'static str> {
    let mut tags: Vec<&'static str> = [Bars::TAGS, Choice::TAGS].concat();
    tags.sort_unstable();
    tags.dedup();
    tags
}

/// Kết quả phát lại và đánh giá một dấu vết.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Run {
    /// Tên loại hoạt động.
    pub kind: &'static str,
    /// Phiên bản định dạng dấu vết.
    pub format: u16,
    /// Số bước trong dấu vết.
    pub steps: usize,
    /// Trạng thái cuối.
    pub final_state: AnyState,
    /// Kết quả đánh giá.
    pub outcome: Outcome,
}

/// Không phát lại được.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RunError {
    /// Dấu vết không đọc được thành kiểu đầu vào của loại hoạt động.
    #[error("dấu vết không đọc được: {0}")]
    Parse(String),
    /// Dấu vết đọc được nhưng không phát lại được.
    #[error("dấu vết không hợp lệ: {0}")]
    Replay(#[from] ReplayError),
}

/// Đọc dấu vết (định dạng tùy `Deserializer`, vd. JSON), phát lại và đánh giá.
pub fn run<'de, D: Deserializer<'de>>(config: &AnyConfig, trace: D) -> Result<Run, RunError> {
    match config {
        AnyConfig::Bars(c) => run_kind::<Bars, D>(c, trace, AnyState::Bars),
        AnyConfig::Choice(c) => run_kind::<Choice, D>(c, trace, AnyState::Choice),
    }
}

fn run_kind<'de, K: Kind, D: Deserializer<'de>>(
    config: &K::Config,
    de: D,
    wrap: fn(K::State) -> AnyState,
) -> Result<Run, RunError> {
    let trace = Trace::<K::Input>::deserialize(de).map_err(|e| RunError::Parse(e.to_string()))?;
    let (replay, outcome) = kernel::run::<K>(config, &trace)?;
    Ok(Run {
        kind: K::KIND,
        format: trace.format,
        steps: trace.entries.len(),
        final_state: wrap(replay.final_state().clone()),
        outcome,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
kind = "bars"
bars = 2
partitions = [3, 5]

[task]
kind = "compare"
left = "1/3"
right = "1/5"
"#;

    fn config() -> AnyConfig {
        toml::from_str(CONFIG).unwrap()
    }

    #[test]
    fn doc_cau_hinh_toml() {
        let c = config();
        assert_eq!(c.kind(), "bars");
        assert!(c.validate().is_ok());
        assert!(
            toml::from_str::<AnyConfig>(&CONFIG.replace("bars = 2", "bars = 2\noops = 1")).is_err()
        );
        assert!(toml::from_str::<AnyConfig>(&CONFIG.replace("\"bars\"", "\"nope\"")).is_err());
        let bad: AnyConfig = toml::from_str(&CONFIG.replace("[3, 5]", "[3]")).unwrap();
        assert!(bad.validate().is_err());
    }

    #[test]
    fn nhan_sap_xep_khong_trung() {
        let tags = all_tags();
        assert!(tags.windows(2).all(|w| w[0] < w[1]));
        assert!(tags.contains(&kind_bars::tags::EQUAL_PARTS));
        assert_eq!(KINDS, &["bars", "choice"]);
    }

    #[test]
    fn chay_dau_vet_json() {
        let json = r#"{"format":1,"entries":[
            {"at_ms":10,"actor":"learner","input":{"op":"compare","answer":"greater"}}]}"#;
        let run = run(&config(), &mut serde_json::Deserializer::from_str(json)).unwrap();
        assert_eq!(run.kind, "bars");
        assert_eq!(run.steps, 1);
        assert!(run.outcome.completed);

        let bad = r#"{"format":1,"entries":[{"at_ms":0,"actor":"learner","input":{"op":"fly"}}]}"#;
        assert!(matches!(
            super::run(&config(), &mut serde_json::Deserializer::from_str(bad)),
            Err(RunError::Parse(_))
        ));
        let rejected = r#"{"format":1,"entries":[
            {"at_ms":0,"actor":"learner","input":{"op":"cut","bar":0,"at":1}}]}"#;
        assert!(matches!(
            super::run(&config(), &mut serde_json::Deserializer::from_str(rejected)),
            Err(RunError::Replay(_))
        ));
    }

    const CHOICE: &str = r#"
kind = "choice"

[[items]]
key = "a"
text = "1/3"
correct = true

[[items]]
key = "b"
text = "1/5"
tag = "frac.larger-denominator-larger"
"#;

    #[test]
    fn loai_chon_dap_an() {
        let c: AnyConfig = toml::from_str(CHOICE).unwrap();
        assert_eq!(c.kind(), "choice");
        assert!(c.validate().is_ok());
        assert!(c.tags().is_empty());
        let tags: Vec<String> = c.config_tags().into_iter().map(String::from).collect();
        assert_eq!(tags, ["frac.larger-denominator-larger"]);
        assert!(config().config_tags().is_empty());

        let json = r#"{"format":1,"entries":[
            {"at_ms":0,"actor":"learner","input":{"op":"select","key":"b"}},
            {"at_ms":1,"actor":"learner","input":{"op":"submit"}},
            {"at_ms":2,"actor":"learner","input":{"op":"select","key":"a"}},
            {"at_ms":3,"actor":"learner","input":{"op":"submit"}}]}"#;
        let run = run(&c, &mut serde_json::Deserializer::from_str(json)).unwrap();
        assert_eq!(run.kind, "choice");
        assert!(run.outcome.completed);
        assert_eq!(run.outcome.score.permille(), 500);
        assert_eq!(run.outcome.findings.len(), 1);
    }
}
