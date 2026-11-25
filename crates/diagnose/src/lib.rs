//! Chấm (`Assessment`) và chẩn đoán (`Observation`) từ dấu vết (ADR 0009).
//!
//! Crate thuần: kết quả chỉ phụ thuộc (nhiệm vụ, cấu hình, dấu vết). Chạy giống
//! hệt ở client (phản hồi tức thì) và server (chấm có thẩm quyền).
//! Chỉ bước của `Actor::Learner` được dùng làm bằng chứng.

use domain::{
    ConceptId, Confidence, Finding, Fraction, MisconceptionId, Observation, Relation, Score,
};
use manip::bars::{self, Bars, MAX_PARTS, TICKS};
use manip::{Answer, ModelSetup, ReplayError, Step, Trace, replay};
use serde::{Deserialize, Serialize};

/// Mã khái niệm và hiểu sai mà các quy tắc chẩn đoán dùng.
///
/// `content` kiểm tra mọi mã ở [`CONCEPTS`] và [`MISCONCEPTIONS`] có trong đồ thị khái niệm.
pub mod ids {
    /// So sánh hai phân số cùng tử số.
    pub const COMPARE_SAME_NUMERATOR: &str = "frac.compare-same-numerator";
    /// Phân số cần các phần bằng nhau.
    pub const EQUAL_PARTS: &str = "frac.equal-parts";
    /// Hiểu sai: mẫu số lớn hơn thì phân số lớn hơn.
    pub const LARGER_DENOMINATOR_LARGER: &str = "frac.larger-denominator-larger";
    /// Hiểu sai: chia thành đủ số phần là được, không cần bằng nhau.
    pub const UNEQUAL_PARTS: &str = "frac.unequal-parts";
}

/// Mọi mã khái niệm các quy tắc dùng.
pub const CONCEPTS: &[&str] = &[ids::COMPARE_SAME_NUMERATOR, ids::EQUAL_PARTS];
/// Mọi mã hiểu sai các quy tắc dùng.
pub const MISCONCEPTIONS: &[&str] = &[ids::LARGER_DENOMINATOR_LARGER, ids::UNEQUAL_PARTS];

/// Sai số cho phép khi coi các mảnh cắt tự do là "bằng nhau", tính bằng phần nghìn
/// độ dài mảnh lý tưởng. Cần hiệu chỉnh sau thử nghiệm thật (ADR 0009).
pub const EQUAL_TOLERANCE_PERMILLE: u32 = 100;

/// Yêu cầu của một hoạt động — enum đóng. TOML: bảng `[task]` có `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Task {
    /// So sánh `left ? right`; người học trả lời bằng [`Answer::Compare`].
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

/// Nhiệm vụ không khớp với cấu hình mô hình.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("nhiệm vụ không hợp lệ: {0}")]
pub struct TaskError(pub String);

/// Kiểm tra nhiệm vụ làm được với cấu hình mô hình đã cho.
pub fn validate_task(task: &Task, setup: &ModelSetup) -> Result<(), TaskError> {
    let ModelSetup::Bars(s) = setup;
    match *task {
        Task::Compare { left, right } => {
            for f in [left, right] {
                if f.den() > u32::from(MAX_PARTS) || f.num() > f.den() {
                    return Err(TaskError(format!(
                        "phân số {f} phải ≤ 1 và có mẫu ≤ {MAX_PARTS} để dựng bằng thanh"
                    )));
                }
                let buildable = f.den() == 1
                    || s.freehand
                    || s.partitions.iter().any(|p| u32::from(*p) == f.den());
                if !buildable {
                    return Err(TaskError(format!(
                        "không dựng được {f}: cần freehand hoặc partitions chứa {}",
                        f.den()
                    )));
                }
            }
        }
        Task::Partition { bar, parts } => {
            if bar >= s.bars {
                return Err(TaskError(format!("không có thanh {bar}")));
            }
            if !(2..=MAX_PARTS).contains(&parts) {
                return Err(TaskError(format!("số phần phải trong 2..={MAX_PARTS}")));
            }
            if !s.freehand && !s.partitions.contains(&parts) {
                return Err(TaskError(format!(
                    "không cắt được {parts} phần: cần freehand hoặc partitions chứa {parts}"
                )));
            }
        }
    }
    Ok(())
}

/// Kết quả chấm một bài làm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    /// Điểm, phần nghìn.
    pub score: Score,
    /// Trạng thái cuối có đạt yêu cầu không.
    pub solved: bool,
    /// Các quan sát chẩn đoán.
    pub observations: Vec<Observation>,
}

/// Không chấm được.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AssessError {
    /// Nhiệm vụ không hợp lệ.
    #[error(transparent)]
    Task(#[from] TaskError),
    /// Dấu vết không phát lại được.
    #[error(transparent)]
    Replay(#[from] ReplayError),
}

/// Chấm bài làm trên mô hình thanh.
pub fn assess_bars(
    task: &Task,
    setup: &bars::Setup,
    trace: &Trace<bars::Action>,
) -> Result<Assessment, AssessError> {
    validate_task(task, &ModelSetup::Bars(setup.clone()))?;
    let run = replay::<Bars>(setup, trace)?;
    Ok(match *task {
        Task::Compare { left, right } => assess_compare(left, right, trace),
        Task::Partition { bar, parts } => assess_partition(bar, parts, trace, &run.states),
    })
}

fn concept(id: &str) -> ConceptId {
    // Mã trong `ids` là hằng hợp lệ, được kiểm tra bởi test `ma_hop_le`.
    ConceptId::new(id).unwrap_or_else(|e| unreachable!("{e}"))
}

fn misconception(id: &str) -> MisconceptionId {
    MisconceptionId::new(id).unwrap_or_else(|e| unreachable!("{e}"))
}

fn confidence(evidence: usize) -> Confidence {
    if evidence >= 2 {
        Confidence::Strong
    } else {
        Confidence::Tentative
    }
}

/// Quan sát chung: có dấu hiệu hiểu sai → `Misconception`, hoặc `SelfCorrected`
/// nếu cuối cùng làm đúng; không có dấu hiệu và làm đúng → `Demonstrated`.
fn observe(
    concept_id: &str,
    misconception_id: &str,
    wrong: Vec<u32>,
    solved_at: Option<u32>,
) -> Option<Observation> {
    let (finding, confidence, evidence) = match (wrong.is_empty(), solved_at) {
        (true, None) => return None,
        (true, Some(i)) => (Finding::Demonstrated, Confidence::Tentative, vec![i]),
        (false, None) => (
            Finding::Misconception(misconception(misconception_id)),
            confidence(wrong.len()),
            wrong,
        ),
        (false, Some(i)) => {
            let c = confidence(wrong.len());
            let mut ev = wrong;
            ev.push(i);
            (
                Finding::SelfCorrected(misconception(misconception_id)),
                c,
                ev,
            )
        }
    };
    Some(Observation {
        concept: concept(concept_id),
        finding,
        confidence,
        evidence,
    })
}

fn assess_compare(left: Fraction, right: Fraction, trace: &Trace<bars::Action>) -> Assessment {
    let correct = left.relation(right);
    let answers: Vec<(u32, Relation)> = trace
        .learner_entries()
        .filter_map(|(i, e)| match e.step {
            Step::Answer(Answer::Compare(r)) => Some((i, r)),
            _ => None,
        })
        .collect();
    let first_ok = answers.first().is_some_and(|(_, r)| *r == correct);
    let last = answers.last().copied();
    let solved = last.is_some_and(|(_, r)| r == correct);
    let score = match (first_ok, solved) {
        (true, _) => Score::FULL,
        (false, true) => Score::from_permille(500).unwrap_or(Score::ZERO),
        (false, false) => Score::ZERO,
    };

    let mut observations = Vec::new();
    // Quy tắc 1: cùng tử số, khác mẫu số — trả lời theo chiều mẫu số là dấu hiệu
    // "mẫu lớn hơn thì lớn hơn".
    if left.num() == right.num() && left.num() > 0 && left.den() != right.den() {
        let by_denominator: Relation = left.den().cmp(&right.den()).into();
        let wrong: Vec<u32> = answers
            .iter()
            .filter(|(_, r)| *r == by_denominator)
            .map(|(i, _)| *i)
            .collect();
        let solved_at = last.filter(|_| solved).map(|(i, _)| i);
        observations.extend(observe(
            ids::COMPARE_SAME_NUMERATOR,
            ids::LARGER_DENOMINATOR_LARGER,
            wrong,
            solved_at,
        ));
    }
    Assessment {
        score,
        solved,
        observations,
    }
}

/// Các mảnh của thanh có đúng `parts` mảnh và bằng nhau trong sai số cho phép không.
fn count_and_equal(state: &bars::State, bar: u8, parts: u8) -> (bool, bool) {
    let pieces = state.pieces_of(bar);
    let right_count = pieces.len() == usize::from(parts);
    let equal = pieces.iter().all(|p| {
        let diff = (p.len * u32::from(parts)).abs_diff(TICKS);
        diff * 1000 <= TICKS * EQUAL_TOLERANCE_PERMILLE
    });
    (right_count, equal)
}

fn assess_partition(
    bar: u8,
    parts: u8,
    trace: &Trace<bars::Action>,
    states: &[bars::State],
) -> Assessment {
    // Quy tắc 2: có lúc thanh được cắt thành đủ số mảnh nhưng các mảnh lệch nhau
    // quá sai số → dấu hiệu "không cần phần bằng nhau".
    let learner_cuts: Vec<u32> = trace
        .learner_entries()
        .filter(
            |(_, e)| matches!(e.step, Step::Model(bars::Action::Cut { bar: b, .. }) if b == bar),
        )
        .map(|(i, _)| i)
        .collect();
    let wrong: Vec<u32> = learner_cuts
        .iter()
        .copied()
        .filter(|i| {
            states
                .get(usize::try_from(*i).unwrap_or(usize::MAX) + 1)
                .is_some_and(|s| count_and_equal(s, bar, parts) == (true, false))
        })
        .collect();
    let solved = states
        .last()
        .is_some_and(|s| count_and_equal(s, bar, parts) == (true, true));
    let solved_at = learner_cuts.last().copied().filter(|_| solved);
    Assessment {
        score: if solved { Score::FULL } else { Score::ZERO },
        solved,
        observations: observe(ids::EQUAL_PARTS, ids::UNEQUAL_PARTS, wrong, solved_at)
            .into_iter()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use manip::bars::{Action, PieceId};
    use manip::{Actor, Entry, TRACE_FORMAT};

    fn f(s: &str) -> Fraction {
        s.parse().unwrap()
    }

    fn setup(freehand: bool) -> bars::Setup {
        bars::Setup {
            bars: 2,
            freehand,
            partitions: vec![3, 5],
        }
    }

    fn trace(steps: Vec<(Actor, Step<Action>)>) -> Trace<Action> {
        Trace {
            format: TRACE_FORMAT,
            entries: steps
                .into_iter()
                .zip(0u32..)
                .map(|((actor, step), i)| Entry {
                    at_ms: i * 1000,
                    actor,
                    step,
                })
                .collect(),
        }
    }

    fn answer(r: Relation) -> (Actor, Step<Action>) {
        (Actor::Learner, Step::Answer(Answer::Compare(r)))
    }

    fn learn(a: Action) -> (Actor, Step<Action>) {
        (Actor::Learner, Step::Model(a))
    }

    fn compare() -> Task {
        Task::Compare {
            left: f("1/3"),
            right: f("1/5"),
        }
    }

    #[test]
    fn ma_hop_le() {
        for id in CONCEPTS {
            assert!(ConceptId::new(*id).is_ok(), "{id}");
        }
        for id in MISCONCEPTIONS {
            assert!(MisconceptionId::new(*id).is_ok(), "{id}");
        }
    }

    #[test]
    fn so_sanh_dung_ngay() {
        let a = assess_bars(
            &compare(),
            &setup(false),
            &trace(vec![answer(Relation::Greater)]),
        )
        .unwrap();
        assert_eq!(a.score, Score::FULL);
        assert!(a.solved);
        assert_eq!(a.observations[0].finding, Finding::Demonstrated);
    }

    #[test]
    fn so_sanh_theo_mau_so_roi_tu_sua() {
        let t = trace(vec![
            answer(Relation::Less),
            learn(Action::Partition { bar: 0, parts: 3 }),
            learn(Action::Partition { bar: 1, parts: 5 }),
            learn(Action::Cut { bar: 0, at: 280 }),
            learn(Action::Cut { bar: 1, at: 168 }),
            learn(Action::Place {
                piece: PieceId { bar: 0, start: 0 },
            }),
            learn(Action::Place {
                piece: PieceId { bar: 1, start: 0 },
            }),
            answer(Relation::Greater),
        ]);
        let a = assess_bars(&compare(), &setup(false), &t).unwrap();
        assert_eq!(a.score.permille(), 500);
        assert!(a.solved);
        let o = &a.observations[0];
        assert_eq!(
            o.finding,
            Finding::SelfCorrected(misconception(ids::LARGER_DENOMINATOR_LARGER))
        );
        assert_eq!(o.evidence, vec![0, 7]);
        assert_eq!(o.confidence, Confidence::Tentative);
    }

    #[test]
    fn so_sanh_sai_lap_lai() {
        let t = trace(vec![answer(Relation::Less), answer(Relation::Less)]);
        let a = assess_bars(&compare(), &setup(false), &t).unwrap();
        assert_eq!(a.score, Score::ZERO);
        assert!(!a.solved);
        let o = &a.observations[0];
        assert_eq!(
            o.finding,
            Finding::Misconception(misconception(ids::LARGER_DENOMINATOR_LARGER))
        );
        assert_eq!(o.confidence, Confidence::Strong);
    }

    #[test]
    fn so_sanh_khac_tu_khong_chan_doan() {
        let task = Task::Compare {
            left: f("2/3"),
            right: f("3/5"),
        };
        let a = assess_bars(&task, &setup(false), &trace(vec![answer(Relation::Less)])).unwrap();
        assert!(!a.solved);
        assert!(a.observations.is_empty());
    }

    #[test]
    fn chia_lech_roi_sua() {
        let task = Task::Partition { bar: 0, parts: 3 };
        let t = trace(vec![
            learn(Action::Cut { bar: 0, at: 200 }),
            learn(Action::Cut { bar: 0, at: 400 }), // 200/200/440: lệch
            learn(Action::Join { bar: 0, at: 200 }),
            learn(Action::Join { bar: 0, at: 400 }),
            learn(Action::Cut { bar: 0, at: 285 }),
            learn(Action::Cut { bar: 0, at: 560 }), // 285/275/280: trong sai số
        ]);
        let a = assess_bars(&task, &setup(true), &t).unwrap();
        assert!(a.solved);
        assert_eq!(a.score, Score::FULL);
        let o = &a.observations[0];
        assert_eq!(
            o.finding,
            Finding::SelfCorrected(misconception(ids::UNEQUAL_PARTS))
        );
        assert_eq!(o.evidence, vec![1, 5]);
    }

    #[test]
    fn buoc_he_thong_khong_la_bang_chung() {
        let task = Task::Partition { bar: 0, parts: 3 };
        let t = trace(vec![
            (Actor::System, Step::Model(Action::Cut { bar: 0, at: 100 })),
            (Actor::System, Step::Model(Action::Cut { bar: 0, at: 200 })),
        ]);
        let a = assess_bars(&task, &setup(true), &t).unwrap();
        assert!(!a.solved);
        assert!(a.observations.is_empty());
    }

    #[test]
    fn nhiem_vu_khong_hop_le() {
        let s = ModelSetup::Bars(setup(false));
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
            assert!(validate_task(&bad, &s).is_err(), "{bad:?}");
        }
        assert!(validate_task(&compare(), &s).is_ok());
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
