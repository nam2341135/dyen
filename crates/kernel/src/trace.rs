//! Dấu vết, phát lại và phiên (ADR 0003, 0010).

use serde::{Deserialize, Serialize};

use crate::{ConfigError, Kind, Outcome, Rejection};

/// Phiên bản định dạng dấu vết hiện tại. Tăng khi thay đổi phá tương thích.
pub const TRACE_FORMAT: u16 = 1;

/// Dấu vết: chuỗi đầu vào đã được chấp nhận, có thứ tự.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace<I> {
    /// Phiên bản định dạng.
    pub format: u16,
    /// Các bước theo thứ tự thời gian.
    pub entries: Vec<Entry<I>>,
}

impl<I> Trace<I> {
    /// Dấu vết rỗng với định dạng hiện tại.
    pub fn new() -> Self {
        Self {
            format: TRACE_FORMAT,
            entries: Vec::new(),
        }
    }

    /// Duyệt các bước của người học kèm chỉ số.
    pub fn learner_entries(&self) -> impl Iterator<Item = (u32, &Entry<I>)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.actor == Actor::Learner)
            .map(|(i, e)| (index(i), e))
    }
}

impl<I> Default for Trace<I> {
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
pub struct Entry<I> {
    /// Mili-giây tương đối từ lúc bắt đầu hoạt động; không giảm.
    pub at_ms: u32,
    /// Ai gây ra bước này.
    pub actor: Actor,
    /// Đầu vào, kiểu do loại hoạt động định nghĩa.
    pub input: I,
}

/// Ai gây ra một bước.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    /// Người học. Chỉ bước của người học được làm bằng chứng.
    Learner,
    /// Hệ thống (vd. phản hồi tự thao tác lại).
    System,
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
    /// Cấu hình không hợp lệ.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Thời gian đi lùi.
    #[error("bước {index}: thời gian đi lùi")]
    TimeWentBack {
        /// Chỉ số bước.
        index: u32,
    },
    /// Một bước bị từ chối — dấu vết bị sửa hoặc lệch phiên bản.
    #[error("bước {index}: {rejection}")]
    Rejected {
        /// Chỉ số bước.
        index: u32,
        /// Lý do.
        rejection: Rejection,
    },
}

/// Phát lại dấu vết từ cấu hình. Tất định.
pub fn replay<K: Kind>(
    config: &K::Config,
    trace: &Trace<K::Input>,
) -> Result<Replay<K::State>, ReplayError> {
    if trace.format != TRACE_FORMAT {
        return Err(ReplayError::Format {
            found: trace.format,
        });
    }
    let mut states = Vec::with_capacity(trace.entries.len() + 1);
    let mut state = K::init(config)?;
    let mut last_ms = 0;
    for (i, entry) in trace.entries.iter().enumerate() {
        let index = index(i);
        if entry.at_ms < last_ms {
            return Err(ReplayError::TimeWentBack { index });
        }
        last_ms = entry.at_ms;
        let next = K::apply(&state, &entry.input)
            .map_err(|rejection| ReplayError::Rejected { index, rejection })?;
        states.push(std::mem::replace(&mut state, next));
    }
    states.push(state);
    Ok(Replay { states })
}

/// Phát lại rồi đánh giá. Cách duy nhất nên dùng để chấm (client và server như nhau).
pub fn run<K: Kind>(
    config: &K::Config,
    trace: &Trace<K::Input>,
) -> Result<(Replay<K::State>, Outcome), ReplayError> {
    let replay = replay::<K>(config, trace)?;
    let outcome = K::evaluate(config, trace, &replay.states);
    Ok((replay, outcome))
}

/// Phiên đang diễn ra: giữ trạng thái có thẩm quyền và ghi dấu vết.
///
/// Vỏ ứng dụng giữ một `Session`; bộ hiển thị đọc [`Session::state`] và gửi đầu vào
/// qua [`Session::act`]. Chỉ đầu vào được chấp nhận mới vào dấu vết.
pub struct Session<K: Kind> {
    config: K::Config,
    state: K::State,
    trace: Trace<K::Input>,
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
    /// Loại hoạt động từ chối đầu vào.
    #[error(transparent)]
    Rejected(#[from] Rejection),
}

impl<K: Kind> Session<K> {
    /// Bắt đầu phiên mới.
    pub fn new(config: K::Config) -> Result<Self, ConfigError> {
        let state = K::init(&config)?;
        Ok(Self {
            config,
            state,
            trace: Trace::new(),
        })
    }

    /// Cấu hình của phiên.
    pub fn config(&self) -> &K::Config {
        &self.config
    }

    /// Trạng thái hiện tại.
    pub fn state(&self) -> &K::State {
        &self.state
    }

    /// Dấu vết tới thời điểm này.
    pub fn trace(&self) -> &Trace<K::Input> {
        &self.trace
    }

    /// Thực hiện một bước. Nếu bị từ chối, trạng thái và dấu vết không đổi.
    pub fn act(&mut self, at_ms: u32, actor: Actor, input: K::Input) -> Result<(), SessionError> {
        let last_ms = self.trace.entries.last().map_or(0, |e| e.at_ms);
        if at_ms < last_ms {
            return Err(SessionError::TimeWentBack { at_ms, last_ms });
        }
        self.state = K::apply(&self.state, &input)?;
        self.trace.entries.push(Entry {
            at_ms,
            actor,
            input,
        });
        Ok(())
    }
}
