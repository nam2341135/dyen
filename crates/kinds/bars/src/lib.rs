//! Loại hoạt động `bars`: thanh cắt được, thước so sánh, và hai nhiệm vụ về phân số.
//!
//! Đây là một **ý tưởng** cắm vào lõi, không phải một phần của lõi (ADR 0010).
//! Xóa crate này và một dòng trong `registry` là gỡ hết ý tưởng.
//!
//! Thuật ngữ riêng của loại này (vạch, mảnh, thước) nằm ở `README.md` của crate.

mod evaluate;
mod fraction;
mod model;

use kernel::{ConfigError, Kind, Outcome, Rejection, Trace};
use serde::{Deserialize, Serialize};

pub use evaluate::{EQUAL_TOLERANCE_PERMILLE, Task, tags};
pub use fraction::{Fraction, FractionError, Relation};
pub use model::{Bar, MAX_BARS, MAX_PARTS, Piece, PieceId, State, TICKS};

/// Loại hoạt động thanh (kiểu đánh dấu cho [`Kind`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bars;

/// Cấu hình một hoạt động thanh.
///
/// TOML (trong bảng `[config]` của hoạt động):
/// ```toml
/// kind = "bars"
/// bars = 2
/// partitions = [3, 5]
/// freehand = false
///
/// [config.task]
/// kind = "compare"
/// left = "1/3"
/// right = "1/5"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Số thanh, `1..=MAX_BARS`, mỗi thanh dài 1 đơn vị.
    pub bars: u8,
    /// Cho phép cắt tại vạch bất kỳ; nếu `false` chỉ cắt được tại vạch chia.
    #[serde(default)]
    pub freehand: bool,
    /// Các số phần được phép chia đều (mỗi số trong `2..=MAX_PARTS`).
    #[serde(default)]
    pub partitions: Vec<u8>,
    /// Yêu cầu của hoạt động; quyết định cách đánh giá.
    pub task: Task,
}

/// Một đầu vào. JSON: trường `op` chọn loại.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Input {
    /// Đặt vạch chia đều thanh `bar` thành `parts` phần (thay vạch chia cũ).
    Partition {
        /// Chỉ số thanh.
        bar: u8,
        /// Số phần.
        parts: u8,
    },
    /// Cắt thanh `bar` tại vạch `at`.
    Cut {
        /// Chỉ số thanh.
        bar: u8,
        /// Vị trí cắt, `1..TICKS`.
        at: u32,
    },
    /// Bỏ vết cắt tại vạch `at`, nối hai mảnh liền kề.
    Join {
        /// Chỉ số thanh.
        bar: u8,
        /// Vị trí vết cắt.
        at: u32,
    },
    /// Đặt một mảnh lên cuối thước.
    Place {
        /// Mảnh được đặt.
        piece: PieceId,
    },
    /// Lấy một mảnh khỏi thước.
    Unplace {
        /// Mảnh được lấy ra.
        piece: PieceId,
    },
    /// Trả lời so sánh `left ? right`. Không đổi trạng thái.
    Compare {
        /// Quan hệ người học chọn.
        answer: Relation,
    },
    /// Báo đã làm xong. Không đổi trạng thái.
    Done,
}

impl Kind for Bars {
    const KIND: &'static str = "bars";
    const TAGS: &'static [&'static str] = tags::ALL;
    type Config = Config;
    type State = State;
    type Input = Input;

    fn init(config: &Config) -> Result<State, ConfigError> {
        let state = State::init(config)?;
        evaluate::validate_task(config).map_err(ConfigError)?;
        Ok(state)
    }

    fn apply(state: &State, input: &Input) -> Result<State, Rejection> {
        state.apply(input)
    }

    fn evaluate(config: &Config, trace: &Trace<Input>, states: &[State]) -> Outcome {
        evaluate::evaluate(&config.task, trace, states)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{Actor, ReplayError, Session, SessionError, replay};

    fn config(freehand: bool) -> Config {
        Config {
            bars: 2,
            freehand,
            partitions: vec![3, 5],
            task: Task::Partition { bar: 0, parts: 3 },
        }
    }

    fn run(config: &Config, inputs: &[Input]) -> Result<State, Rejection> {
        let mut s = Bars::init(config).unwrap();
        for i in inputs {
            s = Bars::apply(&s, i)?;
        }
        Ok(s)
    }

    #[test]
    fn chia_roi_cat_tai_vach() {
        let s = run(
            &config(false),
            &[
                Input::Partition { bar: 0, parts: 3 },
                Input::Cut { bar: 0, at: 280 },
                Input::Cut { bar: 0, at: 560 },
            ],
        )
        .unwrap();
        let pieces = s.pieces_of(0);
        assert_eq!(pieces.len(), 3);
        assert!(
            pieces
                .iter()
                .all(|p| p.fraction() == "1/3".parse().unwrap())
        );
        assert_eq!(s.pieces().len(), 4); // thanh 1 còn nguyên
    }

    #[test]
    fn tu_choi_cat_ngoai_vach_khi_khong_tu_do() {
        let err = run(
            &config(false),
            &[
                Input::Partition { bar: 0, parts: 3 },
                Input::Cut { bar: 0, at: 300 },
            ],
        );
        assert!(err.is_err());
        // Tự do thì được, và mảnh lệch được giữ nguyên độ dài.
        let s = run(&config(true), &[Input::Cut { bar: 0, at: 300 }]).unwrap();
        assert_eq!(s.pieces_of(0)[0].len, 300);
    }

    #[test]
    fn tu_choi_dau_vao_sai() {
        let c = config(true);
        for bad in [
            Input::Partition { bar: 0, parts: 4 }, // không có trong partitions
            Input::Partition { bar: 9, parts: 3 },
            Input::Cut { bar: 0, at: 0 },
            Input::Cut { bar: 0, at: TICKS },
            Input::Join { bar: 0, at: 100 },
            Input::Place {
                piece: PieceId { bar: 0, start: 5 },
            },
            Input::Unplace {
                piece: PieceId { bar: 0, start: 0 },
            },
        ] {
            assert!(run(&c, &[bad]).is_err(), "{bad:?}");
        }
        assert!(
            run(
                &c,
                &[Input::Cut { bar: 0, at: 10 }, Input::Cut { bar: 0, at: 10 }]
            )
            .is_err()
        );
    }

    #[test]
    fn thuoc_khoa_manh_dang_dat() {
        let c = config(true);
        let p0 = PieceId { bar: 0, start: 0 };
        let p1 = PieceId { bar: 0, start: 420 };
        let base = [Input::Cut { bar: 0, at: 420 }, Input::Place { piece: p1 }];
        assert!(run(&c, &[base[0], base[1], Input::Cut { bar: 0, at: 600 }]).is_err());
        assert!(run(&c, &[base[0], base[1], Input::Join { bar: 0, at: 420 }]).is_err());
        assert!(run(&c, &[base[0], base[1], Input::Place { piece: p1 }]).is_err());
        // Mảnh khác vẫn cắt được.
        let st = run(&c, &[base[0], base[1], Input::Cut { bar: 0, at: 200 }]).unwrap();
        assert_eq!(st.ruler(), &[p1]);
        let st = Bars::apply(&st, &Input::Place { piece: p0 }).unwrap();
        let st = Bars::apply(&st, &Input::Unplace { piece: p1 }).unwrap();
        assert_eq!(st.ruler(), &[p0]);
        let st = Bars::apply(&st, &Input::Join { bar: 0, at: 420 }).unwrap();
        assert_eq!(st.pieces_of(0).len(), 2);
    }

    #[test]
    fn tra_loi_khong_doi_trang_thai() {
        let s = Bars::init(&config(false)).unwrap();
        for i in [
            Input::Compare {
                answer: Relation::Less,
            },
            Input::Done,
        ] {
            assert_eq!(Bars::apply(&s, &i), Ok(s.clone()));
        }
    }

    #[test]
    fn cau_hinh_sai() {
        let task = Task::Partition { bar: 0, parts: 3 };
        for (bars, partitions) in [(0, vec![]), (5, vec![]), (1, vec![1]), (1, vec![9])] {
            let c = Config {
                bars,
                freehand: true,
                partitions,
                task: task.clone(),
            };
            assert!(Bars::init(&c).is_err(), "{c:?}");
        }
    }

    #[test]
    fn dinh_dang_json_on_dinh() {
        let cases = [
            (
                Input::Cut { bar: 0, at: 280 },
                r#"{"op":"cut","bar":0,"at":280}"#,
            ),
            (
                Input::Place {
                    piece: PieceId { bar: 1, start: 0 },
                },
                r#"{"op":"place","piece":{"bar":1,"start":0}}"#,
            ),
            (
                Input::Compare {
                    answer: Relation::Greater,
                },
                r#"{"op":"compare","answer":"greater"}"#,
            ),
            (Input::Done, r#"{"op":"done"}"#),
        ];
        for (input, json) in cases {
            assert_eq!(serde_json::to_string(&input).unwrap(), json);
            assert_eq!(serde_json::from_str::<Input>(json).unwrap(), input);
        }
        assert!(
            serde_json::from_str::<Config>(
                r#"{"bars":1,"oops":1,"task":{"kind":"partition","bar":0,"parts":2}}"#
            )
            .is_err()
        );
    }

    fn session_mau() -> Session<Bars> {
        let mut s = Session::<Bars>::new(config(false)).unwrap();
        s.act(0, Actor::Learner, Input::Partition { bar: 0, parts: 3 })
            .unwrap();
        s.act(900, Actor::Learner, Input::Cut { bar: 0, at: 280 })
            .unwrap();
        s.act(
            1500,
            Actor::Learner,
            Input::Compare {
                answer: Relation::Greater,
            },
        )
        .unwrap();
        s
    }

    #[test]
    fn phien_chi_ghi_buoc_duoc_chap_nhan() {
        let mut s = session_mau();
        let before = (s.state().clone(), s.trace().clone());
        assert!(matches!(
            s.act(2000, Actor::Learner, Input::Cut { bar: 0, at: 1 }),
            Err(SessionError::Rejected(_))
        ));
        assert_eq!((s.state().clone(), s.trace().clone()), before);
        assert_eq!(s.trace().entries.len(), 3);
    }

    #[test]
    fn phat_lai_bat_dau_vet_hong() {
        let s = session_mau();
        let mut t = s.trace().clone();
        t.entries[1].input = Input::Cut { bar: 0, at: 300 };
        assert!(matches!(
            replay::<Bars>(s.config(), &t),
            Err(ReplayError::Rejected { index: 1, .. })
        ));
    }
}
