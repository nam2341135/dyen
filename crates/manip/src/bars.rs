//! Mô hình thanh cắt được và thước so sánh.
//!
//! Mỗi thanh dài 1 đơn vị = [`TICKS`] vạch. Người học có thể:
//! - **chia** một thanh thành 2–8 phần bằng nhau (chỉ đặt vạch chia, chưa cắt),
//! - **cắt** thanh tại một vạch (tại vạch chia, hoặc bất kỳ đâu nếu `freehand`),
//! - **nối** lại hai mảnh liền kề bằng cách bỏ một vết cắt,
//! - **đặt** mảnh lên thước để so sánh, hoặc **lấy** mảnh khỏi thước.
//!
//! Mảnh được định danh bằng `(thanh, vạch bắt đầu)` — [`PieceId`].

use std::collections::BTreeSet;

use domain::Fraction;
use serde::{Deserialize, Serialize};

use crate::{Manipulative, Rejection, SetupError};

/// Độ dài một thanh tính bằng vạch: BCNN(1..=8), nên chia đều 2–8 phần luôn chính xác.
pub const TICKS: u32 = 840;
/// Số thanh tối đa trong một hoạt động.
pub const MAX_BARS: u8 = 4;
/// Số phần chia đều tối đa.
pub const MAX_PARTS: u8 = 8;

/// Mô hình thanh (kiểu đánh dấu cho [`Manipulative`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bars;

/// Cấu hình mô hình thanh.
///
/// TOML: `[model]` `kind = "bars"`, `bars = 2`, `partitions = [3, 5]`, `freehand = false`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    /// Số thanh, `1..=MAX_BARS`, mỗi thanh dài 1 đơn vị.
    pub bars: u8,
    /// Cho phép cắt tại vạch bất kỳ; nếu `false` chỉ cắt được tại vạch chia.
    #[serde(default)]
    pub freehand: bool,
    /// Các số phần được phép chia đều (mỗi số trong `2..=MAX_PARTS`).
    #[serde(default)]
    pub partitions: Vec<u8>,
}

/// Mã một mảnh: thanh chứa nó và vạch bắt đầu (0 hoặc một vết cắt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PieceId {
    /// Chỉ số thanh.
    pub bar: u8,
    /// Vạch bắt đầu.
    pub start: u32,
}

/// Một mảnh hiện có.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    /// Mã mảnh.
    pub id: PieceId,
    /// Độ dài tính bằng vạch.
    pub len: u32,
}

impl Piece {
    /// Độ dài dưới dạng phân số của một đơn vị.
    pub fn fraction(self) -> Fraction {
        // `TICKS > 0` nên không thể lỗi.
        Fraction::new(self.len, TICKS).unwrap_or(Fraction::ZERO)
    }
}

/// Một hành động trên mô hình thanh. JSON/TOML: trường `op` chọn loại.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Action {
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
}

/// Một thanh: vạch chia và vết cắt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bar {
    marks: BTreeSet<u32>,
    cuts: BTreeSet<u32>,
}

impl Bar {
    fn new() -> Self {
        Self {
            marks: BTreeSet::new(),
            cuts: BTreeSet::new(),
        }
    }

    /// Vị trí các vạch chia, tăng dần.
    pub fn marks(&self) -> impl Iterator<Item = u32> + '_ {
        self.marks.iter().copied()
    }

    /// Vị trí các vết cắt, tăng dần.
    pub fn cuts(&self) -> impl Iterator<Item = u32> + '_ {
        self.cuts.iter().copied()
    }

    /// Các đoạn `(bắt đầu, độ dài)` theo thứ tự từ trái sang phải.
    fn segments(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        let starts = std::iter::once(0).chain(self.cuts.iter().copied());
        let ends = self.cuts.iter().copied().chain(std::iter::once(TICKS));
        starts.zip(ends).map(|(s, e)| (s, e - s))
    }

    /// Vạch bắt đầu của mảnh chứa điểm `at` (với `0 < at < TICKS`).
    fn start_of_piece_containing(&self, at: u32) -> u32 {
        self.cuts.range(..at).next_back().copied().unwrap_or(0)
    }

    fn has_piece_at(&self, start: u32) -> bool {
        start == 0 || self.cuts.contains(&start)
    }
}

/// Trạng thái mô hình thanh. Trường riêng tư; đọc qua phương thức (ADR 0009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    freehand: bool,
    partitions: Vec<u8>,
    bars: Vec<Bar>,
    ruler: Vec<PieceId>,
}

impl State {
    /// Số thanh.
    pub fn bar_count(&self) -> usize {
        self.bars.len()
    }

    /// Thanh thứ `bar`, nếu có.
    pub fn bar(&self, bar: u8) -> Option<&Bar> {
        self.bars.get(usize::from(bar))
    }

    /// Mọi mảnh hiện có, theo thanh rồi theo vị trí.
    pub fn pieces(&self) -> Vec<Piece> {
        (0u8..)
            .zip(&self.bars)
            .flat_map(|(i, b)| {
                b.segments().map(move |(start, len)| Piece {
                    id: PieceId { bar: i, start },
                    len,
                })
            })
            .collect()
    }

    /// Các mảnh của một thanh, từ trái sang phải.
    pub fn pieces_of(&self, bar: u8) -> Vec<Piece> {
        self.bar(bar)
            .map(|b| {
                b.segments()
                    .map(|(start, len)| Piece {
                        id: PieceId { bar, start },
                        len,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Tra một mảnh theo mã.
    pub fn piece(&self, id: PieceId) -> Option<Piece> {
        self.pieces_of(id.bar).into_iter().find(|p| p.id == id)
    }

    /// Các mảnh đang nằm trên thước, theo thứ tự đặt.
    pub fn ruler(&self) -> &[PieceId] {
        &self.ruler
    }

    fn bar_mut(&mut self, bar: u8) -> Result<&mut Bar, Rejection> {
        self.bars
            .get_mut(usize::from(bar))
            .ok_or_else(|| reject(format!("không có thanh {bar}")))
    }

    fn ensure_not_on_ruler(&self, id: PieceId) -> Result<(), Rejection> {
        if self.ruler.contains(&id) {
            Err(reject(format!(
                "mảnh {}@{} đang nằm trên thước",
                id.bar, id.start
            )))
        } else {
            Ok(())
        }
    }
}

fn reject(msg: String) -> Rejection {
    Rejection(msg)
}

fn valid_parts(parts: u8) -> bool {
    (2..=MAX_PARTS).contains(&parts)
}

impl Manipulative for Bars {
    const KIND: &'static str = "bars";
    type Setup = Setup;
    type State = State;
    type Action = Action;

    fn init(setup: &Setup) -> Result<State, SetupError> {
        if !(1..=MAX_BARS).contains(&setup.bars) {
            return Err(SetupError(format!(
                "số thanh phải trong 1..={MAX_BARS}, có {}",
                setup.bars
            )));
        }
        if let Some(p) = setup.partitions.iter().find(|p| !valid_parts(**p)) {
            return Err(SetupError(format!(
                "số phần chia phải trong 2..={MAX_PARTS}, có {p}"
            )));
        }
        let mut partitions = setup.partitions.clone();
        partitions.sort_unstable();
        partitions.dedup();
        Ok(State {
            freehand: setup.freehand,
            partitions,
            bars: (0..setup.bars).map(|_| Bar::new()).collect(),
            ruler: Vec::new(),
        })
    }

    fn apply(state: &State, action: &Action) -> Result<State, Rejection> {
        let mut s = state.clone();
        match *action {
            Action::Partition { bar, parts } => {
                if !s.partitions.contains(&parts) {
                    return Err(reject(format!("không được chia {parts} phần")));
                }
                let step = TICKS / u32::from(parts);
                s.bar_mut(bar)?.marks = (1..u32::from(parts)).map(|k| k * step).collect();
            }
            Action::Cut { bar, at } => {
                if at == 0 || at >= TICKS {
                    return Err(reject(format!("vị trí cắt {at} nằm ngoài thanh")));
                }
                let b = s
                    .bar(bar)
                    .ok_or_else(|| reject(format!("không có thanh {bar}")))?;
                if b.cuts.contains(&at) {
                    return Err(reject(format!("đã có vết cắt tại {at}")));
                }
                if !s.freehand && !b.marks.contains(&at) {
                    return Err(reject(format!(
                        "chỉ được cắt tại vạch chia, {at} không phải"
                    )));
                }
                let start = b.start_of_piece_containing(at);
                s.ensure_not_on_ruler(PieceId { bar, start })?;
                s.bar_mut(bar)?.cuts.insert(at);
            }
            Action::Join { bar, at } => {
                let b = s
                    .bar(bar)
                    .ok_or_else(|| reject(format!("không có thanh {bar}")))?;
                if !b.cuts.contains(&at) {
                    return Err(reject(format!("không có vết cắt tại {at}")));
                }
                let left = b.start_of_piece_containing(at);
                s.ensure_not_on_ruler(PieceId { bar, start: left })?;
                s.ensure_not_on_ruler(PieceId { bar, start: at })?;
                s.bar_mut(bar)?.cuts.remove(&at);
            }
            Action::Place { piece } => {
                let b = s
                    .bar(piece.bar)
                    .ok_or_else(|| reject(format!("không có thanh {}", piece.bar)))?;
                if !b.has_piece_at(piece.start) {
                    return Err(reject(format!(
                        "không có mảnh bắt đầu tại {} trên thanh {}",
                        piece.start, piece.bar
                    )));
                }
                s.ensure_not_on_ruler(piece)?;
                s.ruler.push(piece);
            }
            Action::Unplace { piece } => {
                let pos = s.ruler.iter().position(|p| *p == piece).ok_or_else(|| {
                    reject(format!(
                        "mảnh {}@{} không nằm trên thước",
                        piece.bar, piece.start
                    ))
                })?;
                s.ruler.remove(pos);
            }
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Actor, Session, SessionError, Step, Trace, replay};

    fn setup(freehand: bool) -> Setup {
        Setup {
            bars: 2,
            freehand,
            partitions: vec![3, 5],
        }
    }

    fn run(setup: &Setup, actions: &[Action]) -> Result<State, Rejection> {
        let mut s = Bars::init(setup).unwrap();
        for a in actions {
            s = Bars::apply(&s, a)?;
        }
        Ok(s)
    }

    #[test]
    fn chia_roi_cat_tai_vach() {
        let s = run(
            &setup(false),
            &[
                Action::Partition { bar: 0, parts: 3 },
                Action::Cut { bar: 0, at: 280 },
                Action::Cut { bar: 0, at: 560 },
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
            &setup(false),
            &[
                Action::Partition { bar: 0, parts: 3 },
                Action::Cut { bar: 0, at: 300 },
            ],
        );
        assert!(err.is_err());
        // Tự do thì được, và mảnh lệch được giữ nguyên độ dài.
        let s = run(&setup(true), &[Action::Cut { bar: 0, at: 300 }]).unwrap();
        assert_eq!(s.pieces_of(0)[0].len, 300);
    }

    #[test]
    fn tu_choi_hanh_dong_sai() {
        let s = setup(true);
        for bad in [
            Action::Partition { bar: 0, parts: 4 }, // không có trong partitions
            Action::Partition { bar: 9, parts: 3 },
            Action::Cut { bar: 0, at: 0 },
            Action::Cut { bar: 0, at: TICKS },
            Action::Join { bar: 0, at: 100 },
            Action::Place {
                piece: PieceId { bar: 0, start: 5 },
            },
            Action::Unplace {
                piece: PieceId { bar: 0, start: 0 },
            },
        ] {
            assert!(run(&s, &[bad]).is_err(), "{bad:?}");
        }
        assert!(
            run(
                &s,
                &[
                    Action::Cut { bar: 0, at: 10 },
                    Action::Cut { bar: 0, at: 10 }
                ]
            )
            .is_err()
        );
    }

    #[test]
    fn thuoc_khoa_manh_dang_dat() {
        let s = setup(true);
        let p0 = PieceId { bar: 0, start: 0 };
        let p1 = PieceId { bar: 0, start: 420 };
        let base = [Action::Cut { bar: 0, at: 420 }, Action::Place { piece: p1 }];
        assert!(run(&s, &[base[0], base[1], Action::Cut { bar: 0, at: 600 }]).is_err());
        assert!(run(&s, &[base[0], base[1], Action::Join { bar: 0, at: 420 }]).is_err());
        assert!(run(&s, &[base[0], base[1], Action::Place { piece: p1 }]).is_err());
        // Mảnh khác vẫn cắt được.
        let st = run(&s, &[base[0], base[1], Action::Cut { bar: 0, at: 200 }]).unwrap();
        assert_eq!(st.ruler(), &[p1]);
        let st = Bars::apply(&st, &Action::Place { piece: p0 }).unwrap();
        let st = Bars::apply(&st, &Action::Unplace { piece: p1 }).unwrap();
        assert_eq!(st.ruler(), &[p0]);
        let st = Bars::apply(&st, &Action::Join { bar: 0, at: 420 }).unwrap();
        assert_eq!(st.pieces_of(0).len(), 2);
    }

    #[test]
    fn cau_hinh_sai() {
        for bad in [
            Setup {
                bars: 0,
                freehand: false,
                partitions: vec![],
            },
            Setup {
                bars: 5,
                freehand: false,
                partitions: vec![],
            },
            Setup {
                bars: 1,
                freehand: false,
                partitions: vec![1],
            },
            Setup {
                bars: 1,
                freehand: false,
                partitions: vec![9],
            },
        ] {
            assert!(Bars::init(&bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn dinh_dang_json_on_dinh() {
        let a = Action::Cut { bar: 0, at: 280 };
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, r#"{"op":"cut","bar":0,"at":280}"#);
        let p = Action::Place {
            piece: PieceId { bar: 1, start: 0 },
        };
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            r#"{"op":"place","piece":{"bar":1,"start":0}}"#
        );

        let m: crate::ModelSetup =
            serde_json::from_str(r#"{"kind":"bars","bars":1,"partitions":[3]}"#).unwrap();
        assert_eq!(m.kind(), "bars");
        assert!(m.validate().is_ok());
        assert!(
            serde_json::from_str::<crate::ModelSetup>(r#"{"kind":"bars","bars":1,"oops":1}"#)
                .is_err()
        );
    }

    fn session_mau() -> Session<Bars> {
        let mut s = Session::<Bars>::new(setup(false)).unwrap();
        s.act(
            0,
            Actor::Learner,
            Step::Model(Action::Partition { bar: 0, parts: 3 }),
        )
        .unwrap();
        s.act(
            900,
            Actor::Learner,
            Step::Model(Action::Cut { bar: 0, at: 280 }),
        )
        .unwrap();
        s.act(
            1500,
            Actor::Learner,
            Step::Answer(crate::Answer::Compare(domain::Relation::Greater)),
        )
        .unwrap();
        s
    }

    #[test]
    fn phien_chi_ghi_buoc_duoc_chap_nhan() {
        let mut s = session_mau();
        let before = (s.state().clone(), s.trace().clone());
        assert!(matches!(
            s.act(
                2000,
                Actor::Learner,
                Step::Model(Action::Cut { bar: 0, at: 1 })
            ),
            Err(SessionError::Rejected(_))
        ));
        assert!(matches!(
            s.act(
                10,
                Actor::Learner,
                Step::Model(Action::Cut { bar: 0, at: 560 })
            ),
            Err(SessionError::TimeWentBack { .. })
        ));
        assert_eq!((s.state().clone(), s.trace().clone()), before);
        assert_eq!(s.trace().entries.len(), 3);
    }

    #[test]
    fn phat_lai_tat_dinh_va_qua_json() {
        let s = session_mau();
        let json = serde_json::to_string(s.trace()).unwrap();
        let trace: Trace<Action> = serde_json::from_str(&json).unwrap();
        let r1 = replay::<Bars>(s.setup(), &trace).unwrap();
        let r2 = replay::<Bars>(s.setup(), s.trace()).unwrap();
        assert_eq!(r1, r2);
        assert_eq!(r1.states.len(), 4);
        assert_eq!(r1.final_state(), s.state());
        // Câu trả lời không đổi trạng thái.
        assert_eq!(r1.states[2], r1.states[3]);
        assert!(json.starts_with(r#"{"format":1,"#));
    }

    #[test]
    fn phat_lai_bat_dau_vet_hong() {
        let s = session_mau();
        let mut t = s.trace().clone();
        t.format = 99;
        assert!(matches!(
            replay::<Bars>(s.setup(), &t),
            Err(crate::ReplayError::Format { found: 99 })
        ));
        let mut t = s.trace().clone();
        t.entries[1].step = Step::Model(Action::Cut { bar: 0, at: 300 });
        assert!(matches!(
            replay::<Bars>(s.setup(), &t),
            Err(crate::ReplayError::Rejected { index: 1, .. })
        ));
        let mut t = s.trace().clone();
        t.entries[2].at_ms = 1;
        assert!(matches!(
            replay::<Bars>(s.setup(), &t),
            Err(crate::ReplayError::TimeWentBack { index: 2 })
        ));
    }
}
