//! Property test cho hợp đồng `Manipulative` / `Session` / `replay` (ADR 0003, 0009).
//!
//! Phần kiểm tra hợp đồng (`check_contract`) chỉ dùng trait, không biết mô hình cụ thể.
//! Mô hình thanh được cắm vào qua một bộ sinh hành động ngẫu nhiên và một bất biến.
//! Số ngẫu nhiên có seed cố định (xorshift) nên test tất định; khi lỗi, thông báo in seed.

use manip::bars::{self, Bars, PieceId, TICKS};
use manip::{Actor, Manipulative, Session, Step, Trace, replay};

const CASES: u64 = 300;
const STEPS: u32 = 60;

/// Bộ sinh số giả ngẫu nhiên xorshift64 — đủ cho test, không cần thư viện ngoài.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    fn pick<T: Copy>(&mut self, xs: &[T]) -> Option<T> {
        if xs.is_empty() {
            None
        } else {
            Some(xs[self.below(xs.len() as u64) as usize])
        }
    }
}

/// Hợp đồng chung, đúng cho mọi mô hình:
/// 1. `apply` tất định;
/// 2. bước bị từ chối không đổi trạng thái lẫn dấu vết;
/// 3. phát lại dấu vết của phiên cho đúng trạng thái của phiên, từng bước một;
/// 4. dấu vết qua JSON rồi phát lại cho cùng kết quả;
/// 5. bất biến riêng của mô hình đúng ở mọi trạng thái.
fn check_contract<M: Manipulative>(
    seed: u64,
    setup: M::Setup,
    mut gen_action: impl FnMut(&mut Rng, &M::State) -> M::Action,
    invariant: impl Fn(&M::State) -> Result<(), String>,
) -> usize {
    let mut rng = Rng::new(seed);
    let mut session = Session::<M>::new(setup).expect("cấu hình hợp lệ");
    let mut at_ms = 0u32;
    let mut accepted = 0;
    for _ in 0..STEPS {
        let action = gen_action(&mut rng, session.state());
        let before_state = session.state().clone();
        let before_len = session.trace().entries.len();

        let once = M::apply(&before_state, &action);
        assert_eq!(
            once,
            M::apply(&before_state, &action),
            "seed {seed}: apply không tất định"
        );

        at_ms += rng.below(3000) as u32;
        let actor = if rng.below(5) == 0 {
            Actor::System
        } else {
            Actor::Learner
        };
        match session.act(at_ms, actor, Step::Model(action.clone())) {
            Ok(()) => {
                accepted += 1;
                assert_eq!(
                    Ok(session.state()),
                    once.as_ref(),
                    "seed {seed}: phiên khác apply"
                );
            }
            Err(_) => {
                assert!(
                    once.is_err(),
                    "seed {seed}: phiên từ chối điều apply chấp nhận"
                );
                assert_eq!(
                    session.state(),
                    &before_state,
                    "seed {seed}: từ chối mà trạng thái đổi"
                );
                assert_eq!(session.trace().entries.len(), before_len, "seed {seed}");
            }
        }
        if let Err(e) = invariant(session.state()) {
            panic!("seed {seed}: vi phạm bất biến sau {action:?}: {e}");
        }
    }
    assert_eq!(session.trace().entries.len(), accepted);

    let run = replay::<M>(session.setup(), session.trace()).expect("phát lại được");
    assert_eq!(run.states.len(), accepted + 1, "seed {seed}");
    assert_eq!(run.final_state(), session.state(), "seed {seed}");
    for (i, e) in session.trace().entries.iter().enumerate() {
        if let Step::Model(a) = &e.step {
            assert_eq!(
                M::apply(&run.states[i], a).as_ref(),
                Ok(&run.states[i + 1]),
                "seed {seed}: bước {i}"
            );
        }
    }

    let json = serde_json::to_string(session.trace()).expect("ghi JSON");
    let back: Trace<M::Action> = serde_json::from_str(&json).expect("đọc JSON");
    assert_eq!(&back, session.trace(), "seed {seed}: JSON không khứ hồi");
    assert_eq!(
        replay::<M>(session.setup(), &back).expect("phát lại"),
        run,
        "seed {seed}"
    );
    accepted
}

// ---- Mô hình thanh: bộ sinh và bất biến ----

fn bars_action(rng: &mut Rng, s: &bars::State) -> bars::Action {
    let bar_count = s.bar_count() as u64;
    // Đôi khi chọn thanh không tồn tại để thử nhánh từ chối.
    let bar = rng.below(bar_count + 1) as u8;
    let pieces: Vec<PieceId> = s.pieces().into_iter().map(|p| p.id).collect();
    let existing_cut = s
        .bar(bar)
        .map(|b| b.cuts().collect::<Vec<_>>())
        .and_then(|c| rng.pick(&c));
    let mark = s
        .bar(bar)
        .map(|b| b.marks().collect::<Vec<_>>())
        .and_then(|m| rng.pick(&m));
    match rng.below(6) {
        0 => bars::Action::Partition {
            bar,
            parts: rng.below(10) as u8,
        },
        1 => bars::Action::Cut {
            bar,
            at: mark.unwrap_or_else(|| rng.below(u64::from(TICKS) + 1) as u32),
        },
        2 => bars::Action::Cut {
            bar,
            at: rng.below(u64::from(TICKS) + 1) as u32,
        },
        3 => bars::Action::Join {
            bar,
            at: existing_cut.unwrap_or_else(|| rng.below(u64::from(TICKS)) as u32),
        },
        4 => bars::Action::Place {
            piece: rng.pick(&pieces).unwrap_or(PieceId { bar, start: 7 }),
        },
        _ => bars::Action::Unplace {
            piece: rng
                .pick(s.ruler())
                .or_else(|| rng.pick(&pieces))
                .unwrap_or(PieceId { bar, start: 0 }),
        },
    }
}

fn bars_invariant(s: &bars::State) -> Result<(), String> {
    {
        for i in 0..s.bar_count() as u8 {
            let total: u32 = s.pieces_of(i).iter().map(|p| p.len).sum();
            if total != TICKS {
                return Err(format!("thanh {i} dài {total} vạch"));
            }
            if s.pieces_of(i).iter().any(|p| p.len == 0) {
                return Err(format!("thanh {i} có mảnh rỗng"));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for p in s.ruler() {
            if s.piece(*p).is_none() {
                return Err(format!("thước chứa mảnh không tồn tại {p:?}"));
            }
            if !seen.insert(*p) {
                return Err(format!("thước chứa {p:?} hai lần"));
            }
        }
        Ok(())
    }
}

#[test]
fn hop_dong_thanh_tu_do() {
    let mut accepted = 0;
    for seed in 0..CASES {
        let setup = bars::Setup {
            bars: 1 + (seed % 4) as u8,
            freehand: true,
            partitions: vec![2, 3, 4, 5, 6, 7, 8],
        };
        accepted += check_contract::<Bars>(seed, setup, bars_action, bars_invariant);
    }
    // Bộ sinh phải tạo đủ hành động hợp lệ, nếu không test chỉ thử nhánh từ chối.
    let total = (CASES * u64::from(STEPS)) as usize;
    assert!(
        accepted * 3 > total,
        "chỉ {accepted}/{total} hành động được chấp nhận"
    );
}

#[test]
fn hop_dong_thanh_chi_cat_tai_vach() {
    let mut accepted = 0;
    for seed in 0..CASES {
        let setup = bars::Setup {
            bars: 2,
            freehand: false,
            partitions: vec![3, 4],
        };
        accepted += check_contract::<Bars>(seed + 10_000, setup, bars_action, bars_invariant);
    }
    let total = (CASES * u64::from(STEPS)) as usize;
    assert!(
        accepted * 5 > total,
        "chỉ {accepted}/{total} hành động được chấp nhận"
    );
}
