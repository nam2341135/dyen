//! Bộ kiểm hợp đồng [`Kind`] dùng lại cho mọi loại hoạt động.
//!
//! Mỗi crate loại hoạt động gọi [`check`] từ test của mình với một bộ sinh đầu vào
//! ngẫu nhiên và một bất biến riêng. Số ngẫu nhiên có seed ([`Rng`]) nên test tất định;
//! khi lỗi, thông báo có seed để tái hiện.
//!
//! Không panic: trả `Err(mô tả)` để test bên gọi báo lỗi.

use std::collections::BTreeSet;

use crate::{Actor, Kind, Outcome, Session, Trace, replay, run};

/// Bộ sinh số giả ngẫu nhiên xorshift64 có seed — đủ cho test, không cần thư viện ngoài.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// Tạo từ seed (mọi seed đều dùng được, kể cả 0).
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// Số tiếp theo.
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Số trong `0..n` (`n = 0` coi như 1).
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }

    /// Chọn một phần tử, `None` nếu rỗng.
    pub fn pick<T: Copy>(&mut self, xs: &[T]) -> Option<T> {
        let n = u64::try_from(xs.len()).ok()?;
        if n == 0 {
            return None;
        }
        let i = usize::try_from(self.below(n)).ok()?;
        xs.get(i).copied()
    }
}

/// Ghi rồi đọc lại một dấu vết; `Err` mang mô tả lỗi.
pub type Roundtrip<I> = dyn Fn(&Trace<I>) -> Result<Trace<I>, String>;

/// Mô tả một lần kiểm hợp đồng.
pub struct Spec<'a, K: Kind> {
    /// Cấu hình hợp lệ để bắt đầu phiên.
    pub config: K::Config,
    /// Số đầu vào sinh ra (cả bị từ chối).
    pub steps: u32,
    /// Sinh một đầu vào từ trạng thái hiện tại. Nên đôi khi sinh đầu vào sai để thử nhánh từ chối.
    pub input: &'a dyn Fn(&mut Rng, &K::State) -> K::Input,
    /// Bất biến riêng của loại hoạt động, phải đúng ở mọi trạng thái.
    pub invariant: &'a dyn Fn(&K::State) -> Result<(), String>,
    /// Ghi rồi đọc lại dấu vết (vd. qua JSON). Lõi không phụ thuộc định dạng cụ thể.
    pub roundtrip: &'a Roundtrip<K::Input>,
}

/// Kết quả một lần kiểm thành công.
pub struct Checked<K: Kind> {
    /// Số đầu vào được chấp nhận.
    pub accepted: usize,
    /// Dấu vết của phiên.
    pub trace: Trace<K::Input>,
    /// Kết quả đánh giá dấu vết.
    pub outcome: Outcome,
}

/// Kiểm hợp đồng với một seed:
/// 1. `apply` tất định;
/// 2. đầu vào bị từ chối không đổi trạng thái lẫn dấu vết;
/// 3. bất biến riêng đúng ở mọi trạng thái;
/// 4. phát lại dấu vết cho đúng chuỗi trạng thái của phiên;
/// 5. dấu vết khứ hồi qua `roundtrip` không đổi;
/// 6. `evaluate` tất định; bằng chứng tăng dần, chỉ trỏ vào bước người học; nhãn nằm trong
///    `K::TAGS` hoặc `K::config_tags(config)`.
pub fn check<K: Kind>(seed: u64, spec: &Spec<'_, K>) -> Result<Checked<K>, String> {
    let fail = |msg: String| Err(format!("seed {seed}: {msg}"));
    let mut rng = Rng::new(seed);
    let mut session =
        Session::<K>::new(spec.config.clone()).map_err(|e| format!("seed {seed}: {e}"))?;
    let mut states = vec![session.state().clone()];
    let mut at_ms = 0u32;

    for _ in 0..spec.steps {
        let input = (spec.input)(&mut rng, session.state());
        let before = session.state().clone();
        let before_len = session.trace().entries.len();
        let once = K::apply(&before, &input);
        if once != K::apply(&before, &input) {
            return fail(format!("apply không tất định với {input:?}"));
        }
        at_ms = at_ms.saturating_add(u32::try_from(rng.below(3000)).unwrap_or(0));
        let actor = if rng.below(5) == 0 {
            Actor::System
        } else {
            Actor::Learner
        };
        match session.act(at_ms, actor, input.clone()) {
            Ok(()) => {
                if Ok(session.state()) != once.as_ref() {
                    return fail(format!("phiên khác apply với {input:?}"));
                }
                states.push(session.state().clone());
            }
            Err(_) => {
                if once.is_ok() {
                    return fail(format!("phiên từ chối điều apply chấp nhận: {input:?}"));
                }
                if session.state() != &before || session.trace().entries.len() != before_len {
                    return fail(format!("từ chối {input:?} mà phiên đổi"));
                }
            }
        }
        if let Err(e) = (spec.invariant)(session.state()) {
            return fail(format!("vi phạm bất biến sau {input:?}: {e}"));
        }
    }

    let trace = session.trace().clone();
    let replayed = replay::<K>(&spec.config, &trace).map_err(|e| format!("seed {seed}: {e}"))?;
    if replayed.states != states {
        return fail("phát lại khác chuỗi trạng thái của phiên".into());
    }

    let back = (spec.roundtrip)(&trace).map_err(|e| format!("seed {seed}: {e}"))?;
    if back != trace {
        return fail("dấu vết không khứ hồi".into());
    }

    let (_, outcome) = run::<K>(&spec.config, &trace).map_err(|e| format!("seed {seed}: {e}"))?;
    let (_, again) = run::<K>(&spec.config, &back).map_err(|e| format!("seed {seed}: {e}"))?;
    if outcome != again {
        return fail("evaluate không tất định".into());
    }
    check_outcome::<K>(&spec.config, &trace, &outcome).map_err(|e| format!("seed {seed}: {e}"))?;

    Ok(Checked {
        accepted: trace.entries.len(),
        trace,
        outcome,
    })
}

/// Các tính chất mọi kết quả đánh giá phải có.
fn check_outcome<K: Kind>(
    config: &K::Config,
    trace: &Trace<K::Input>,
    outcome: &Outcome,
) -> Result<(), String> {
    let from_config = K::config_tags(config);
    let tags: BTreeSet<&str> = K::TAGS
        .iter()
        .copied()
        .chain(from_config.iter().map(|t| t.as_str()))
        .collect();
    for f in &outcome.findings {
        if !tags.contains(f.tag.as_str()) {
            return Err(format!(
                "nhãn `{}` không có trong TAGS hay config_tags",
                f.tag
            ));
        }
        if !f.evidence.windows(2).all(|w| w[0] < w[1]) {
            return Err(format!("bằng chứng không tăng dần: {f:?}"));
        }
        for i in &f.evidence {
            let entry = usize::try_from(*i)
                .ok()
                .and_then(|i| trace.entries.get(i))
                .ok_or_else(|| format!("bằng chứng trỏ ra ngoài dấu vết: {f:?}"))?;
            if entry.actor != Actor::Learner {
                return Err(format!("bằng chứng là bước hệ thống: {f:?}"));
            }
        }
    }
    Ok(())
}
