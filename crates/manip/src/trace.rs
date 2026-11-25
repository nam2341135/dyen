//! Dấu vết, phát lại và phiên thao tác (ADR 0003, 0009).

use domain::Relation;
use serde::{Deserialize, Serialize};

use crate::{Manipulative, Rejection, SetupError};

/// Phiên bản định dạng dấu vết hiện tại. Tăng khi thay đổi phá tương thích.
pub const TRACE_FORMAT: u16 = 1;

/// Dấu vết: chuỗi bước đã được chấp nhận, có thứ tự.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace<A> {
    /// Phiên bản định dạng.
    pub format: u16,
    /// Các bước theo thứ tự thời gian.
    pub entries: Vec<Entry<A>>,
}

impl<A> Trace<A> {
    /// Dấu vết rỗng với định dạng hiện tại.
    pub fn new() -> Self {
        Self {
            format: TRACE_FORMAT,
            entries: Vec::new(),
        }
    }

    /// Duyệt các bước của người học kèm chỉ số.
    pub fn learner_entries(&self) -> impl Iterator<Item = (u32, &Entry<A>)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.actor == Actor::Learner)
            .map(|(i, e)| (index(i), e))
    }
}

impl<A> Default for Trace<A> {
    fn default() -> Self {
        Self::new()
    }
}

/// Chỉ số bước dạng `u32` (dấu vết không bao giờ dài tới 2^32 bước).
fn index(i: usize) -> u32 {
    u32::try_from(i).unwrap_or(u32::MAX)
}

/// Một bước trong dấu vết.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry<A> {
    /// Mili-giây tương đối từ lúc bắt đầu hoạt động; không giảm.
    pub at_ms: u32,
    /// Ai gây ra bước này.
    pub actor: Actor,
    /// Nội dung bước.
    pub step: Step<A>,
}

/// Ai gây ra một bước.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    /// Người học.
    Learner,
    /// Hệ thống (vd. phản hồi tự dựng lại mô hình).
    System,
}

/// Nội dung một bước: thao tác mô hình hoặc trả lời.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step<A> {
    /// Hành động trên mô hình.
    Model(A),
    /// Câu trả lời; không đổi trạng thái mô hình.
    Answer(Answer),
}

/// Câu trả lời của người học.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Answer {
    /// Chọn quan hệ `<`, `=`, `>` giữa hai đại lượng.
    Compare(Relation),
    /// Báo đã làm xong (với nhiệm vụ mà kết quả nằm trong trạng thái mô hình).
    Done,
}

/// Kết quả phát lại: chuỗi trạng thái.
///
/// `states[0]` là trạng thái đầu; `states[i + 1]` là trạng thái sau bước `i`.
#[derive(Debug, Clone, PartialEq)]
pub struct Replay<S> {
    /// Chuỗi trạng thái, dài hơn số bước đúng 1.
    pub states: Vec<S>,
}

impl<S> Replay<S> {
    /// Trạng thái cuối cùng.
    pub fn final_state(&self) -> &S {
        // `replay` luôn đẩy trạng thái đầu nên `states` không rỗng.
        &self.states[self.states.len() - 1]
    }
}

/// Dấu vết không phát lại được.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReplayError {
    /// Phiên bản định dạng không hỗ trợ.
    #[error("định dạng dấu vết {found} không được hỗ trợ (hiện tại: {TRACE_FORMAT})")]
    Format {
        /// Phiên bản tìm thấy.
        found: u16,
    },
    /// Cấu hình mô hình không hợp lệ.
    #[error(transparent)]
    Setup(#[from] SetupError),
    /// Thời gian đi lùi.
    #[error("bước {index}: thời gian đi lùi")]
    TimeWentBack {
        /// Chỉ số bước.
        index: u32,
    },
    /// Một bước bị mô hình từ chối — dấu vết bị sửa hoặc lệch phiên bản.
    #[error("bước {index}: {rejection}")]
    Rejected {
        /// Chỉ số bước.
        index: u32,
        /// Lý do.
        rejection: Rejection,
    },
}

/// Phát lại dấu vết từ cấu hình. Tất định.
pub fn replay<M: Manipulative>(
    setup: &M::Setup,
    trace: &Trace<M::Action>,
) -> Result<Replay<M::State>, ReplayError> {
    if trace.format != TRACE_FORMAT {
        return Err(ReplayError::Format {
            found: trace.format,
        });
    }
    let mut states = Vec::with_capacity(trace.entries.len() + 1);
    let mut state = M::init(setup)?;
    let mut last_ms = 0;
    for (i, entry) in trace.entries.iter().enumerate() {
        let index = index(i);
        if entry.at_ms < last_ms {
            return Err(ReplayError::TimeWentBack { index });
        }
        last_ms = entry.at_ms;
        let next = match &entry.step {
            Step::Model(action) => M::apply(&state, action)
                .map_err(|rejection| ReplayError::Rejected { index, rejection })?,
            Step::Answer(_) => state.clone(),
        };
        states.push(std::mem::replace(&mut state, next));
    }
    states.push(state);
    Ok(Replay { states })
}

/// Phiên thao tác đang diễn ra: giữ trạng thái có thẩm quyền và ghi dấu vết.
///
/// Vỏ ứng dụng giữ một `Session`; bộ hiển thị đọc [`Session::state`] và gửi bước
/// qua [`Session::act`]. Chỉ bước được chấp nhận mới vào dấu vết (ADR 0009).
pub struct Session<M: Manipulative> {
    setup: M::Setup,
    state: M::State,
    trace: Trace<M::Action>,
}

/// Bước bị phiên từ chối.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    /// Thời gian đi lùi so với bước trước.
    #[error("thời gian đi lùi ({at_ms} < {last_ms})")]
    TimeWentBack {
        /// Thời điểm bước mới.
        at_ms: u32,
        /// Thời điểm bước trước.
        last_ms: u32,
    },
    /// Mô hình từ chối hành động.
    #[error(transparent)]
    Rejected(#[from] Rejection),
}

impl<M: Manipulative> Session<M> {
    /// Bắt đầu phiên mới.
    pub fn new(setup: M::Setup) -> Result<Self, SetupError> {
        let state = M::init(&setup)?;
        Ok(Self {
            setup,
            state,
            trace: Trace::new(),
        })
    }

    /// Cấu hình của phiên.
    pub fn setup(&self) -> &M::Setup {
        &self.setup
    }

    /// Trạng thái hiện tại.
    pub fn state(&self) -> &M::State {
        &self.state
    }

    /// Dấu vết tới thời điểm này.
    pub fn trace(&self) -> &Trace<M::Action> {
        &self.trace
    }

    /// Thực hiện một bước. Nếu bị từ chối, trạng thái và dấu vết không đổi.
    pub fn act(
        &mut self,
        at_ms: u32,
        actor: Actor,
        step: Step<M::Action>,
    ) -> Result<(), SessionError> {
        let last_ms = self.trace.entries.last().map_or(0, |e| e.at_ms);
        if at_ms < last_ms {
            return Err(SessionError::TimeWentBack { at_ms, last_ms });
        }
        if let Step::Model(action) = &step {
            self.state = M::apply(&self.state, action)?;
        }
        self.trace.entries.push(Entry { at_ms, actor, step });
        Ok(())
    }
}
