//! Nhiệm vụ và đánh giá của loại hoạt động thanh.
//!
//! Chỉ bước của `Actor::Learner` được dùng làm bằng chứng.

use kernel::{Confidence, Finding, Outcome, Score, TagId, Trace};
use serde::{Deserialize, Serialize};

use crate::model::valid_parts;
use crate::{Config, Fraction, Input, MAX_PARTS, Relation, State, TICKS};

/// Nhãn mà loại hoạt động thanh gắn vào phát hiện.
///
/// Nội dung phải đặt tên hiển thị cho mọi nhãn ở [`ALL`](tags::ALL) (`content/tags.toml`).
pub mod tags {
    /// So sánh hai phân số cùng tử số.
    pub const COMPARE_SAME_NUMERATOR: &str = "frac.compare-same-numerator";
    /// Phân số cần các phần bằng nhau.
    pub const EQUAL_PARTS: &str = "frac.equal-parts";
    /// Lối nghĩ: mẫu số lớn hơn thì phân số lớn hơn.
    pub const LARGER_DENOMINATOR_LARGER: &str = "frac.larger-denominator-larger";
    /// Lối nghĩ: chia đủ số phần là được, không cần bằng nhau.
    pub const UNEQUAL_PARTS: &str = "frac.unequal-parts";
    /// Mọi nhãn.
    pub const ALL: &[&str] = &[
        COMPARE_SAME_NUMERATOR,
        EQUAL_PARTS,
        LARGER_DENOMINATOR_LARGER,
        UNEQUAL_PARTS,
    ];
}

/// Sai số cho phép khi coi các mảnh cắt tự do là "bằng nhau", tính bằng phần nghìn
/// độ dài mảnh lý tưởng. Cần hiệu chỉnh sau thử nghiệm thật.
pub const EQUAL_TOLERANCE_PERMILLE: u32 = 100;

/// Yêu cầu của một hoạt động thanh. TOML: bảng `[config.task]` có `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Task {
    /// So sánh `left ? right`; người học trả lời bằng [`Input::Compare`].
    Compare {
        /// Vế trái.
        left: Fraction,
        /// Vế phải.
        right: Fraction,
    },
    /// Chia thanh `bar` thành `parts` phần bằng nhau bằng cách cắt.
    Partition {
        /// Chỉ số thanh.
        bar: u8,
        /// Số phần.
        parts: u8,
    },
}

/// Kiểm tra nhiệm vụ làm được với phần mô hình của cấu hình.
pub(crate) fn validate_task(c: &Config) -> Result<(), String> {
    match c.task {
        Task::Compare { left, right } => {
            for f in [left, right] {
                if f.den() > u32::from(MAX_PARTS) || f.num() > f.den() {
                    return Err(format!(
                        "phân số {f} phải ≤ 1 và có mẫu ≤ {MAX_PARTS} để dựng bằng thanh"
                    ));
                }
                let buildable = f.den() == 1
                    || c.freehand
                    || c.partitions.iter().any(|p| u32::from(*p) == f.den());
                if !buildable {
                    return Err(format!(
                        "không dựng được {f}: cần freehand hoặc partitions chứa {}",
                        f.den()
                    ));
                }
            }
        }
        Task::Partition { bar, parts } => {
            if bar >= c.bars {
                return Err(format!("nhiệm vụ: không có thanh {bar}"));
            }
            if !valid_parts(parts) {
                return Err(format!("nhiệm vụ: số phần phải trong 2..={MAX_PARTS}"));
            }
            if !c.freehand && !c.partitions.contains(&parts) {
                return Err(format!(
                    "không cắt được {parts} phần: cần freehand hoặc partitions chứa {parts}"
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn evaluate(task: &Task, trace: &Trace<Input>, states: &[State]) -> Outcome {
    match *task {
        Task::Compare { left, right } => evaluate_compare(left, right, trace),
        Task::Partition { bar, parts } => evaluate_partition(bar, parts, trace, states),
    }
}

fn tag(id: &str) -> TagId {
    // Mã trong `tags` là hằng hợp lệ, được kiểm tra bởi test `nhan_hop_le`.
    TagId::new(id).unwrap_or_else(|e| unreachable!("{e}"))
}

fn confidence(evidence: usize) -> Confidence {
    if evidence >= 2 {
        Confidence::Strong
    } else {
        Confidence::Tentative
    }
}

/// Phát hiện chung cho một cặp (kỹ năng, lối nghĩ sai):
/// - không có dấu hiệu sai và làm đúng → nhãn kỹ năng;
/// - có dấu hiệu sai, chưa làm đúng → nhãn lối nghĩ sai;
/// - có dấu hiệu sai, cuối cùng làm đúng → nhãn lối nghĩ sai, `recovered`.
fn observe(skill: &str, pitfall: &str, wrong: Vec<u32>, solved_at: Option<u32>) -> Option<Finding> {
    let (tag_id, recovered, confidence, evidence) = match (wrong.is_empty(), solved_at) {
        (true, None) => return None,
        (true, Some(i)) => (skill, false, Confidence::Tentative, vec![i]),
        (false, None) => (pitfall, false, confidence(wrong.len()), wrong),
        (false, Some(i)) => {
            let c = confidence(wrong.len());
            let mut ev = wrong;
            ev.push(i);
            (pitfall, true, c, ev)
        }
    };
    Some(Finding {
        tag: tag(tag_id),
        recovered,
        confidence,
        evidence,
    })
}

fn evaluate_compare(left: Fraction, right: Fraction, trace: &Trace<Input>) -> Outcome {
    let correct = left.relation(right);
    let answers: Vec<(u32, Relation)> = trace
        .learner_entries()
        .filter_map(|(i, e)| match e.input {
            Input::Compare { answer } => Some((i, answer)),
            _ => None,
        })
        .collect();
    let last = answers.last().copied();
    let completed = last.is_some_and(|(_, r)| r == correct);
    // Điểm theo câu trả lời cuối: chưa đúng → 0; đúng và chưa từng sai → đủ điểm;
    // đúng sau khi đã sai → một nửa. (Đúng rồi đổi sang sai vẫn là 0.)
    let never_wrong = answers.iter().all(|(_, r)| *r == correct);
    let score = match (completed, never_wrong) {
        (false, _) => Score::ZERO,
        (true, true) => Score::FULL,
        (true, false) => Score::from_permille(500).unwrap_or(Score::ZERO),
    };

    let mut findings = Vec::new();
    // Cùng tử số, khác mẫu số — trả lời theo chiều mẫu số là dấu hiệu
    // "mẫu lớn hơn thì lớn hơn".
    if left.num() == right.num() && left.num() > 0 && left.den() != right.den() {
        let by_denominator: Relation = left.den().cmp(&right.den()).into();
        let wrong: Vec<u32> = answers
            .iter()
            .filter(|(_, r)| *r == by_denominator)
            .map(|(i, _)| *i)
            .collect();
        let solved_at = last.filter(|_| completed).map(|(i, _)| i);
        findings.extend(observe(
            tags::COMPARE_SAME_NUMERATOR,
            tags::LARGER_DENOMINATOR_LARGER,
            wrong,
            solved_at,
        ));
    }
    Outcome {
        score,
        completed,
        findings,
    }
}

/// Các mảnh của thanh có đúng `parts` mảnh và bằng nhau trong sai số cho phép không.
fn count_and_equal(state: &State, bar: u8, parts: u8) -> (bool, bool) {
    let pieces = state.pieces_of(bar);
    let right_count = pieces.len() == usize::from(parts);
    let equal = pieces.iter().all(|p| {
        let diff = (p.len * u32::from(parts)).abs_diff(TICKS);
        diff * 1000 <= TICKS * EQUAL_TOLERANCE_PERMILLE
    });
    (right_count, equal)
}

fn evaluate_partition(bar: u8, parts: u8, trace: &Trace<Input>, states: &[State]) -> Outcome {
    // Có lúc thanh được cắt thành đủ số mảnh nhưng các mảnh lệch nhau quá sai số
    // → dấu hiệu "không cần phần bằng nhau".
    let learner_cuts: Vec<u32> = trace
        .learner_entries()
        .filter(|(_, e)| matches!(e.input, Input::Cut { bar: b, .. } if b == bar))
        .map(|(i, _)| i)
        .collect();
    let wrong: Vec<u32> = learner_cuts
        .iter()
        .copied()
        .filter(|i| {
            usize::try_from(*i)
                .ok()
                .and_then(|i| states.get(i + 1))
                .is_some_and(|s| count_and_equal(s, bar, parts) == (true, false))
        })
        .collect();
    let completed = states
        .last()
        .is_some_and(|s| count_and_equal(s, bar, parts) == (true, true));
    let solved_at = learner_cuts.last().copied().filter(|_| completed);
    Outcome {
        score: if completed { Score::FULL } else { Score::ZERO },
        completed,
        findings: observe(tags::EQUAL_PARTS, tags::UNEQUAL_PARTS, wrong, solved_at)
            .into_iter()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bars, PieceId};
    use kernel::{Actor, Entry, Kind, TRACE_FORMAT, run};

    fn f(s: &str) -> Fraction {
        s.parse().unwrap()
    }

    fn config(freehand: bool, task: Task) -> Config {
        Config {
            bars: 2,
            freehand,
            partitions: vec![3, 5],
            task,
        }
    }

    fn trace(steps: Vec<(Actor, Input)>) -> Trace<Input> {
        Trace {
            format: TRACE_FORMAT,
            entries: steps
                .into_iter()
                .zip(0u32..)
                .map(|((actor, input), i)| Entry {
                    at_ms: i * 1000,
                    actor,
                    input,
                })
                .collect(),
        }
    }

    fn answer(r: Relation) -> (Actor, Input) {
        (Actor::Learner, Input::Compare { answer: r })
    }

    fn learn(i: Input) -> (Actor, Input) {
        (Actor::Learner, i)
    }

    fn compare() -> Task {
        Task::Compare {
            left: f("1/3"),
            right: f("1/5"),
        }
    }

    fn outcome(c: &Config, t: &Trace<Input>) -> Outcome {
        run::<Bars>(c, t).unwrap().1
    }

    #[test]
    fn nhan_hop_le() {
        for id in tags::ALL {
            assert!(TagId::new(*id).is_ok(), "{id}");
        }
        assert_eq!(Bars::TAGS, tags::ALL);
    }

    #[test]
    fn so_sanh_dung_ngay() {
        let o = outcome(
            &config(false, compare()),
            &trace(vec![answer(Relation::Greater)]),
        );
        assert_eq!(o.score, Score::FULL);
        assert!(o.completed);
        assert_eq!(o.findings[0].tag.as_str(), tags::COMPARE_SAME_NUMERATOR);
        assert!(!o.findings[0].recovered);
    }

    #[test]
    fn so_sanh_theo_mau_so_roi_tu_sua() {
        let t = trace(vec![
            answer(Relation::Less),
            learn(Input::Partition { bar: 0, parts: 3 }),
            learn(Input::Partition { bar: 1, parts: 5 }),
            learn(Input::Cut { bar: 0, at: 280 }),
            learn(Input::Cut { bar: 1, at: 168 }),
            learn(Input::Place {
                piece: PieceId { bar: 0, start: 0 },
            }),
            learn(Input::Place {
                piece: PieceId { bar: 1, start: 0 },
            }),
            answer(Relation::Greater),
        ]);
        let o = outcome(&config(false, compare()), &t);
        assert_eq!(o.score.permille(), 500);
        assert!(o.completed);
        let x = &o.findings[0];
        assert_eq!(x.tag.as_str(), tags::LARGER_DENOMINATOR_LARGER);
        assert!(x.recovered);
        assert_eq!(x.evidence, vec![0, 7]);
        assert_eq!(x.confidence, Confidence::Tentative);
    }

    #[test]
    fn so_sanh_sai_lap_lai() {
        let t = trace(vec![answer(Relation::Less), answer(Relation::Less)]);
        let o = outcome(&config(false, compare()), &t);
        assert_eq!(o.score, Score::ZERO);
        assert!(!o.completed);
        let x = &o.findings[0];
        assert_eq!(x.tag.as_str(), tags::LARGER_DENOMINATOR_LARGER);
        assert!(!x.recovered);
        assert_eq!(x.confidence, Confidence::Strong);
    }

    #[test]
    fn dung_roi_doi_sang_sai_la_chua_dat() {
        let t = trace(vec![answer(Relation::Greater), answer(Relation::Less)]);
        let o = outcome(&config(false, compare()), &t);
        assert!(!o.completed);
        assert_eq!(o.score, Score::ZERO);
    }

    #[test]
    fn so_sanh_khac_tu_khong_co_phat_hien() {
        let task = Task::Compare {
            left: f("2/3"),
            right: f("3/5"),
        };
        let o = outcome(&config(false, task), &trace(vec![answer(Relation::Less)]));
        assert!(!o.completed);
        assert!(o.findings.is_empty());
    }

    #[test]
    fn chia_lech_roi_sua() {
        let task = Task::Partition { bar: 0, parts: 3 };
        let t = trace(vec![
            learn(Input::Cut { bar: 0, at: 200 }),
            learn(Input::Cut { bar: 0, at: 400 }), // 200/200/440: lệch
            learn(Input::Join { bar: 0, at: 200 }),
            learn(Input::Join { bar: 0, at: 400 }),
            learn(Input::Cut { bar: 0, at: 285 }),
            learn(Input::Cut { bar: 0, at: 560 }), // 285/275/280: trong sai số
        ]);
        let o = outcome(&config(true, task), &t);
        assert!(o.completed);
        assert_eq!(o.score, Score::FULL);
        let x = &o.findings[0];
        assert_eq!(x.tag.as_str(), tags::UNEQUAL_PARTS);
        assert!(x.recovered);
        assert_eq!(x.evidence, vec![1, 5]);
    }

    #[test]
    fn buoc_he_thong_khong_la_bang_chung() {
        let task = Task::Partition { bar: 0, parts: 3 };
        let t = trace(vec![
            (Actor::System, Input::Cut { bar: 0, at: 100 }),
            (Actor::System, Input::Cut { bar: 0, at: 200 }),
        ]);
        let o = outcome(&config(true, task), &t);
        assert!(!o.completed);
        assert!(o.findings.is_empty());
    }

    #[test]
    fn nhiem_vu_khong_hop_le() {
        for bad in [
            Task::Compare {
                left: f("1/4"),
                right: f("1/3"),
            }, // 4 không có trong partitions
            Task::Compare {
                left: f("4/3"),
                right: f("1/3"),
            },
            Task::Partition { bar: 2, parts: 3 },
            Task::Partition { bar: 0, parts: 4 },
            Task::Partition { bar: 0, parts: 9 },
        ] {
            assert!(Bars::init(&config(false, bad.clone())).is_err(), "{bad:?}");
        }
        assert!(Bars::init(&config(false, compare())).is_ok());
    }

    #[test]
    fn task_json() {
        let t: Task =
            serde_json::from_str(r#"{"kind":"compare","left":"1/3","right":"1/5"}"#).unwrap();
        assert_eq!(t, compare());
        assert!(serde_json::from_str::<Task>(r#"{"kind":"partition","bar":0}"#).is_err());
        assert!(
            serde_json::from_str::<Task>(r#"{"kind":"partition","bar":0,"parts":3,"x":1}"#)
                .is_err()
        );
    }
}
