//! Loại hoạt động `choice`: chọn một hoặc nhiều đáp án trong danh sách rồi nộp.
//!
//! Đây là một **ý tưởng** cắm vào lõi, không phải một phần của lõi (ADR 0010).
//! Xóa crate này và một dòng trong `registry` là gỡ hết ý tưởng.
//!
//! Khác loại `bars`, nhãn của loại này không cố định trong code: người soạn nội dung gắn
//! nhãn vào từng lựa chọn, và [`Kind::config_tags`] công bố chúng (ADR 0011).
//!
//! Thuật ngữ riêng (lựa chọn, khóa, lần nộp) nằm ở `README.md` của crate.

use kernel::{Confidence, ConfigError, Finding, Kind, Outcome, Rejection, Score, TagId, Trace};
use serde::{Deserialize, Serialize};

/// Số lựa chọn tối đa trong một hoạt động.
pub const MAX_ITEMS: usize = 16;
/// Độ dài tối đa của khóa lựa chọn.
pub const MAX_KEY_LEN: usize = 16;

/// Loại hoạt động chọn đáp án (kiểu đánh dấu cho [`Kind`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice;

/// Cấu hình một hoạt động chọn đáp án.
///
/// TOML (trong bảng `[config]` của hoạt động):
/// ```toml
/// kind = "choice"
/// multiple = false
/// attempts = 2
///
/// [[config.items]]
/// key = "a"
/// text = "1/3"
/// correct = true
///
/// [[config.items]]
/// key = "b"
/// text = "1/5"
/// tag = "frac.larger-denominator-larger"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Cho chọn nhiều lựa chọn (`true`) hay đúng một (`false`).
    #[serde(default)]
    pub multiple: bool,
    /// Số lần nộp tối đa, `>= 1`.
    #[serde(default = "default_attempts")]
    pub attempts: u8,
    /// Các lựa chọn, theo thứ tự người soạn đặt. Bộ hiển thị có thể xáo khi vẽ.
    pub items: Vec<Item>,
}

fn default_attempts() -> u8 {
    2
}

/// Một lựa chọn. `text` là chữ của người soạn nội dung, không phải dữ liệu của người học.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    /// Khóa ổn định, dùng trong dấu vết: `a-z`, `0-9`, `-`, tối đa [`MAX_KEY_LEN`] ký tự.
    pub key: String,
    /// Nội dung hiển thị.
    pub text: String,
    /// Lựa chọn này thuộc đáp án đúng.
    #[serde(default)]
    pub correct: bool,
    /// Nhãn gắn vào phát hiện khi người học nộp bài có lựa chọn này.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<TagId>,
}

/// Một đầu vào.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    /// Chọn một lựa chọn. Khi chỉ chọn một, lựa chọn mới thay lựa chọn cũ.
    Select {
        /// Khóa lựa chọn.
        key: String,
    },
    /// Bỏ chọn một lựa chọn đang chọn.
    Deselect {
        /// Khóa lựa chọn.
        key: String,
    },
    /// Nộp các lựa chọn đang chọn (không được rỗng).
    Submit,
}

/// Trạng thái. Mang theo phần cấu hình cần để `apply` tự chấm từng lần nộp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    multiple: bool,
    attempts: u8,
    keys: Vec<String>,
    correct: Vec<String>,
    selected: Vec<String>,
    submissions: u8,
    solved: bool,
}

impl State {
    /// Có cho chọn nhiều không.
    pub fn multiple(&self) -> bool {
        self.multiple
    }

    /// Khóa các lựa chọn, theo thứ tự cấu hình.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// Các khóa đang chọn, theo thứ tự cấu hình.
    pub fn selected(&self) -> &[String] {
        &self.selected
    }

    /// Số lần đã nộp.
    pub fn submissions(&self) -> u8 {
        self.submissions
    }

    /// Số lần nộp còn lại.
    pub fn attempts_left(&self) -> u8 {
        self.attempts.saturating_sub(self.submissions)
    }

    /// Đã nộp đúng chưa.
    pub fn solved(&self) -> bool {
        self.solved
    }

    /// Hoạt động đã kết thúc (đúng, hoặc hết lượt nộp).
    pub fn finished(&self) -> bool {
        self.solved || self.attempts_left() == 0
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.keys.iter().position(|k| k == key)
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= MAX_KEY_LEN
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn reject(msg: impl Into<String>) -> Result<State, Rejection> {
    Err(Rejection(msg.into()))
}

impl Kind for Choice {
    const KIND: &'static str = "choice";
    const TAGS: &'static [&'static str] = &[];
    type Config = Config;
    type State = State;
    type Input = Input;

    fn config_tags(config: &Config) -> Vec<TagId> {
        let mut tags: Vec<TagId> = Vec::new();
        for t in config.items.iter().filter_map(|i| i.tag.as_ref()) {
            if !tags.contains(t) {
                tags.push(t.clone());
            }
        }
        tags
    }

    fn init(config: &Config) -> Result<State, ConfigError> {
        let err = |m: String| Err(ConfigError(m));
        let n = config.items.len();
        if !(2..=MAX_ITEMS).contains(&n) {
            return err(format!("cần 2..={MAX_ITEMS} lựa chọn, có {n}"));
        }
        if config.attempts == 0 {
            return err("attempts phải >= 1".into());
        }
        let mut keys: Vec<String> = Vec::with_capacity(n);
        for item in &config.items {
            if !valid_key(&item.key) {
                return err(format!("khóa `{}` không hợp lệ", item.key));
            }
            if keys.contains(&item.key) {
                return err(format!("khóa `{}` bị trùng", item.key));
            }
            if item.text.trim().is_empty() {
                return err(format!("lựa chọn `{}` không có nội dung", item.key));
            }
            keys.push(item.key.clone());
        }
        let correct: Vec<String> = config
            .items
            .iter()
            .filter(|i| i.correct)
            .map(|i| i.key.clone())
            .collect();
        match (config.multiple, correct.len()) {
            (_, 0) => return err("cần ít nhất một lựa chọn đúng".into()),
            (false, c) if c > 1 => {
                return err(format!("chọn một nhưng có {c} lựa chọn đúng"));
            }
            _ => {}
        }
        Ok(State {
            multiple: config.multiple,
            attempts: config.attempts,
            keys,
            correct,
            selected: Vec::new(),
            submissions: 0,
            solved: false,
        })
    }

    fn apply(state: &State, input: &Input) -> Result<State, Rejection> {
        if state.solved {
            return reject("đã trả lời đúng");
        }
        if state.attempts_left() == 0 {
            return reject("hết lượt nộp");
        }
        let mut next = state.clone();
        match input {
            Input::Select { key } => {
                if state.index_of(key).is_none() {
                    return reject(format!("không có lựa chọn `{key}`"));
                }
                if state.selected.contains(key) {
                    return reject(format!("`{key}` đã được chọn"));
                }
                if !state.multiple {
                    next.selected.clear();
                }
                next.selected.push(key.clone());
                // Giữ thứ tự cấu hình để trạng thái có dạng chuẩn duy nhất.
                next.selected
                    .sort_by_key(|k| state.index_of(k).unwrap_or(usize::MAX));
            }
            Input::Deselect { key } => {
                let Some(pos) = state.selected.iter().position(|k| k == key) else {
                    return reject(format!("`{key}` chưa được chọn"));
                };
                next.selected.remove(pos);
            }
            Input::Submit => {
                if state.selected.is_empty() {
                    return reject("chưa chọn gì");
                }
                next.submissions += 1;
                // `selected` và `correct` cùng theo thứ tự cấu hình nên so sánh trực tiếp được.
                next.solved = state.selected == state.correct;
            }
        }
        Ok(next)
    }

    fn evaluate(config: &Config, trace: &Trace<Input>, states: &[State]) -> Outcome {
        // Các lần nộp của người học: (chỉ số bước, trạng thái sau bước).
        let submits: Vec<(u32, &State)> = trace
            .learner_entries()
            .filter(|(_, e)| e.input == Input::Submit)
            .filter_map(|(i, _)| {
                let after = states.get(usize::try_from(i).ok()?.checked_add(1)?)?;
                Some((i, after))
            })
            .collect();
        // Lần nộp đúng (nếu có) là lần cuối, vì sau đó mọi đầu vào bị từ chối.
        let solved_at = submits.iter().find(|(_, s)| s.solved).copied();
        let completed = solved_at.is_some();
        // Điểm = 1000 / số thứ tự lần nộp đúng (tính cả lần nộp của hệ thống).
        let score = solved_at
            .and_then(|(_, s)| Score::from_permille(1000 / u16::from(s.submissions.max(1))))
            .unwrap_or(Score::ZERO);
        let last_selected = submits.last().map(|(_, s)| s.selected.as_slice());

        let mut findings = Vec::new();
        for item in &config.items {
            let Some(tag) = &item.tag else { continue };
            let mut evidence: Vec<u32> = submits
                .iter()
                .filter(|(_, s)| s.selected.contains(&item.key))
                .map(|(i, _)| *i)
                .collect();
            if evidence.is_empty() {
                continue;
            }
            let confidence = if evidence.len() >= 2 {
                Confidence::Strong
            } else {
                Confidence::Tentative
            };
            let left_behind = last_selected.is_some_and(|s| !s.contains(&item.key));
            let recovered = completed && left_behind;
            if recovered && let Some((i, _)) = solved_at {
                evidence.push(i);
            }
            findings.push(Finding {
                tag: tag.clone(),
                recovered,
                confidence,
                evidence,
            });
        }
        Outcome {
            score,
            completed,
            findings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{Actor, Session, run};

    fn item(key: &str, correct: bool, tag: Option<&str>) -> Item {
        Item {
            key: key.into(),
            text: format!("lựa chọn {key}"),
            correct,
            tag: tag.map(|t| TagId::new(t).expect("nhãn hợp lệ")),
        }
    }

    fn single() -> Config {
        Config {
            multiple: false,
            attempts: 2,
            items: vec![
                item("a", true, None),
                item("b", false, Some("x.wrong-b")),
                item("c", false, None),
            ],
        }
    }

    fn sel(k: &str) -> Input {
        Input::Select { key: k.into() }
    }

    fn play(config: Config, inputs: &[Input]) -> (Session<Choice>, Outcome) {
        let mut s = Session::<Choice>::new(config).expect("cấu hình hợp lệ");
        for (t, i) in inputs.iter().enumerate() {
            s.act(t as u32 * 1000, Actor::Learner, i.clone())
                .expect("đầu vào hợp lệ");
        }
        let (_, o) = run::<Choice>(s.config(), s.trace()).expect("phát lại được");
        (s, o)
    }

    #[test]
    fn cau_hinh_sai_bi_tu_choi() {
        let mut c = single();
        c.items[1].correct = true;
        assert!(Choice::init(&c).is_err(), "chọn một mà hai đáp án đúng");
        let mut c = single();
        c.items[0].correct = false;
        assert!(Choice::init(&c).is_err(), "không có đáp án đúng");
        let mut c = single();
        c.items[2].key = "a".into();
        assert!(Choice::init(&c).is_err(), "khóa trùng");
        let mut c = single();
        c.items[2].key = "C".into();
        assert!(Choice::init(&c).is_err(), "khóa hoa");
        let mut c = single();
        c.items.truncate(1);
        assert!(Choice::init(&c).is_err(), "một lựa chọn");
        let mut c = single();
        c.attempts = 0;
        assert!(Choice::init(&c).is_err(), "không lượt nộp");
    }

    #[test]
    fn chon_mot_thay_lua_chon_cu() {
        let s = Choice::init(&single()).expect("hợp lệ");
        let s = Choice::apply(&s, &sel("b")).expect("chọn b");
        let s = Choice::apply(&s, &sel("a")).expect("chọn a");
        assert_eq!(s.selected(), ["a"]);
        assert!(Choice::apply(&s, &sel("a")).is_err());
        assert!(Choice::apply(&s, &sel("z")).is_err());
        assert!(Choice::apply(&s, &Input::Deselect { key: "b".into() }).is_err());
    }

    #[test]
    fn chon_nhieu_giu_thu_tu_cau_hinh() {
        let mut c = single();
        c.multiple = true;
        c.items[2].correct = true;
        let s = Choice::init(&c).expect("hợp lệ");
        let s = Choice::apply(&s, &sel("c")).expect("chọn c");
        let s = Choice::apply(&s, &sel("a")).expect("chọn a");
        assert_eq!(s.selected(), ["a", "c"]);
        let s = Choice::apply(&s, &Input::Submit).expect("nộp");
        assert!(s.solved());
        assert!(Choice::apply(&s, &sel("b")).is_err(), "xong rồi thì khóa");
    }

    #[test]
    fn nop_rong_va_het_luot_bi_tu_choi() {
        let s = Choice::init(&single()).expect("hợp lệ");
        assert!(Choice::apply(&s, &Input::Submit).is_err());
        let s = Choice::apply(&s, &sel("b")).expect("chọn");
        let s = Choice::apply(&s, &Input::Submit).expect("nộp 1");
        let s = Choice::apply(&s, &Input::Submit).expect("nộp 2");
        assert!(s.finished() && !s.solved());
        assert!(Choice::apply(&s, &sel("a")).is_err());
    }

    #[test]
    fn dung_ngay_du_diem_khong_phat_hien() {
        let (_, o) = play(single(), &[sel("a"), Input::Submit]);
        assert!(o.completed);
        assert_eq!(o.score, Score::FULL);
        assert!(o.findings.is_empty());
    }

    #[test]
    fn sai_roi_sua_nua_diem_va_hoi_phuc() {
        let (_, o) = play(
            single(),
            &[sel("b"), Input::Submit, sel("a"), Input::Submit],
        );
        assert!(o.completed);
        assert_eq!(o.score.permille(), 500);
        assert_eq!(o.findings.len(), 1);
        let f = &o.findings[0];
        assert_eq!(f.tag.as_str(), "x.wrong-b");
        assert!(f.recovered);
        assert_eq!(f.confidence, Confidence::Tentative);
        assert_eq!(f.evidence, vec![1, 3]);
    }

    #[test]
    fn sai_hai_lan_chac_chan() {
        let (_, o) = play(single(), &[sel("b"), Input::Submit, Input::Submit]);
        assert!(!o.completed);
        assert_eq!(o.score, Score::ZERO);
        let f = &o.findings[0];
        assert!(!f.recovered);
        assert_eq!(f.confidence, Confidence::Strong);
        assert_eq!(f.evidence, vec![1, 2]);
    }

    #[test]
    fn nhan_cau_hinh_khong_trung_theo_thu_tu() {
        let mut c = single();
        c.items[2].tag = Some(TagId::new("x.wrong-b").expect("hợp lệ"));
        c.items[0].tag = Some(TagId::new("x.right").expect("hợp lệ"));
        let tags: Vec<String> = Choice::config_tags(&c)
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(tags, ["x.right", "x.wrong-b"]);
    }
}
