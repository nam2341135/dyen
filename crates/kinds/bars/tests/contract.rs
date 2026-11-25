//! Hợp đồng `Kind` cho loại thanh, qua bộ kiểm chung `kernel::contract`.
//!
//! Phần riêng của loại thanh: bộ sinh đầu vào ngẫu nhiên, bất biến trạng thái,
//! và tính chất đánh giá `completed == (score > 0)` (đúng với cả hai nhiệm vụ hiện có).

use kernel::contract::{self, Rng, Spec};
use kernel::{Outcome, Trace};
use kind_bars::{Bars, Config, Input, PieceId, Relation, State, TICKS, Task};

const CASES: u64 = 300;
const STEPS: u32 = 60;

fn gen_input(rng: &mut Rng, s: &State) -> Input {
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
    match rng.below(8) {
        0 => Input::Partition {
            bar,
            parts: rng.below(10) as u8,
        },
        1 => Input::Cut {
            bar,
            at: mark.unwrap_or_else(|| rng.below(u64::from(TICKS) + 1) as u32),
        },
        2 => Input::Cut {
            bar,
            at: rng.below(u64::from(TICKS) + 1) as u32,
        },
        3 => Input::Join {
            bar,
            at: existing_cut.unwrap_or_else(|| rng.below(u64::from(TICKS)) as u32),
        },
        4 => Input::Place {
            piece: rng.pick(&pieces).unwrap_or(PieceId { bar, start: 7 }),
        },
        5 => Input::Unplace {
            piece: rng
                .pick(s.ruler())
                .or_else(|| rng.pick(&pieces))
                .unwrap_or(PieceId { bar, start: 0 }),
        },
        6 => Input::Compare {
            answer: rng
                .pick(&[Relation::Less, Relation::Equal, Relation::Greater])
                .unwrap_or(Relation::Equal),
        },
        _ => Input::Done,
    }
}

fn invariant(s: &State) -> Result<(), String> {
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

fn json_roundtrip(t: &Trace<Input>) -> Result<Trace<Input>, String> {
    let s = serde_json::to_string(t).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

fn outcome_property(seed: u64, o: &Outcome) {
    assert_eq!(o.completed, o.score.permille() > 0, "seed {seed}: {o:?}");
}

/// Chạy `CASES` seed, trả về tổng số đầu vào được chấp nhận.
fn check_all(seed_base: u64, config: impl Fn(u64) -> Config) -> usize {
    let mut accepted = 0;
    for seed in seed_base..seed_base + CASES {
        let spec = Spec::<Bars> {
            config: config(seed),
            steps: STEPS,
            input: &gen_input,
            invariant: &invariant,
            roundtrip: &json_roundtrip,
        };
        let checked = contract::check(seed, &spec).unwrap_or_else(|e| panic!("{e}"));
        outcome_property(seed, &checked.outcome);
        accepted += checked.accepted;
    }
    accepted
}

fn total() -> usize {
    (CASES * u64::from(STEPS)) as usize
}

#[test]
fn hop_dong_tu_do_chia_thanh() {
    let accepted = check_all(0, |seed| Config {
        bars: 1 + (seed % 4) as u8,
        freehand: true,
        partitions: vec![2, 3, 4, 5, 6, 7, 8],
        task: Task::Partition { bar: 0, parts: 3 },
    });
    // Bộ sinh phải tạo đủ đầu vào hợp lệ, nếu không test chỉ thử nhánh từ chối.
    assert!(
        accepted * 3 > total(),
        "chỉ {accepted}/{} được chấp nhận",
        total()
    );
}

#[test]
fn hop_dong_chi_cat_tai_vach_so_sanh() {
    let accepted = check_all(10_000, |_| Config {
        bars: 2,
        freehand: false,
        partitions: vec![3, 5],
        task: Task::Compare {
            left: "1/3".parse().expect("phân số"),
            right: "1/5".parse().expect("phân số"),
        },
    });
    assert!(
        accepted * 5 > total(),
        "chỉ {accepted}/{} được chấp nhận",
        total()
    );
}
