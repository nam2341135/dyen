//! Mô hình thanh: trạng thái và cách đầu vào thao tác làm đổi trạng thái.
//!
//! Mỗi thanh dài 1 đơn vị = [`TICKS`] vạch. Người học có thể:
//! - **chia** một thanh thành 2–8 phần bằng nhau (chỉ đặt vạch chia, chưa cắt),
//! - **cắt** thanh tại một vạch (tại vạch chia, hoặc bất kỳ đâu nếu `freehand`),
//! - **nối** lại hai mảnh liền kề bằng cách bỏ một vết cắt,
//! - **đặt** mảnh lên thước để so sánh, hoặc **lấy** mảnh khỏi thước.
//!
//! Mảnh được định danh bằng `(thanh, vạch bắt đầu)` — [`PieceId`].

use std::collections::BTreeSet;

use kernel::{ConfigError, Rejection};
use serde::{Deserialize, Serialize};

use crate::{Config, Fraction, Input};

/// Độ dài một thanh tính bằng vạch: BCNN(1..=8), nên chia đều 2–8 phần luôn chính xác.
pub const TICKS: u32 = 840;
/// Số thanh tối đa trong một hoạt động.
pub const MAX_BARS: u8 = 4;
/// Số phần chia đều tối đa.
pub const MAX_PARTS: u8 = 8;

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

/// Trạng thái mô hình thanh. Trường riêng tư; đọc qua phương thức.
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

pub(crate) fn valid_parts(parts: u8) -> bool {
    (2..=MAX_PARTS).contains(&parts)
}

impl State {
    /// Dựng trạng thái đầu từ cấu hình (phần mô hình, không kiểm nhiệm vụ).
    pub(crate) fn init(config: &Config) -> Result<State, ConfigError> {
        if !(1..=MAX_BARS).contains(&config.bars) {
            return Err(ConfigError(format!(
                "số thanh phải trong 1..={MAX_BARS}, có {}",
                config.bars
            )));
        }
        if let Some(p) = config.partitions.iter().find(|p| !valid_parts(**p)) {
            return Err(ConfigError(format!(
                "số phần chia phải trong 2..={MAX_PARTS}, có {p}"
            )));
        }
        let mut partitions = config.partitions.clone();
        partitions.sort_unstable();
        partitions.dedup();
        Ok(State {
            freehand: config.freehand,
            partitions,
            bars: (0..config.bars).map(|_| Bar::new()).collect(),
            ruler: Vec::new(),
        })
    }

    /// Áp dụng một đầu vào. Câu trả lời (`Compare`, `Done`) không đổi trạng thái.
    pub(crate) fn apply(&self, input: &Input) -> Result<State, Rejection> {
        let mut s = self.clone();
        match *input {
            Input::Partition { bar, parts } => {
                if !s.partitions.contains(&parts) {
                    return Err(reject(format!("không được chia {parts} phần")));
                }
                let step = TICKS / u32::from(parts);
                s.bar_mut(bar)?.marks = (1..u32::from(parts)).map(|k| k * step).collect();
            }
            Input::Cut { bar, at } => {
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
            Input::Join { bar, at } => {
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
            Input::Place { piece } => {
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
            Input::Unplace { piece } => {
                let pos = s.ruler.iter().position(|p| *p == piece).ok_or_else(|| {
                    reject(format!(
                        "mảnh {}@{} không nằm trên thước",
                        piece.bar, piece.start
                    ))
                })?;
                s.ruler.remove(pos);
            }
            Input::Compare { .. } | Input::Done => {}
        }
        Ok(s)
    }
}
