//! Hợp đồng `Kind` cho loại thiết diện, qua bộ kiểm chung `kernel::contract`.
//!
//! Phần riêng: bộ sinh cấu hình và đầu vào ngẫu nhiên, bất biến trạng thái, và tính chất
//! `completed == (score > 0)`.

use kernel::contract::{self, Rng, Spec};
use kernel::{Kind, Outcome, Trace};
use kind_section::{Config, EdgePoint, Input, Section, State, sites};
use lib_exact::Rational;

const CASES: u64 = 300;
const STEPS: u32 = 30;

const EDGES: [&str; 12] = [
    "ab", "bc", "cd", "da", "a1b1", "b1c1", "c1d1", "d1a1", "aa1", "bb1", "cc1", "dd1",
];

fn gen_input(rng: &mut Rng, s: &State) -> Input {
    // Thiên về vị trí đúng để có lần cắt đúng; đôi khi vị trí lạ để thử nhánh từ chối.
    let site = match rng.below(4) {
        0 => {
            let all: Vec<&str> = sites().chain(["khong-co"]).collect();
            rng.pick(&all).unwrap_or("a").to_owned()
        }
        _ => {
            let right: Vec<&str> = s.section().iter().map(|c| c.site.as_str()).collect();
            rng.pick(&right).unwrap_or("a").to_owned()
        }
    };
    match rng.below(8) {
        0..=4 => Input::Mark { site },
        5 => Input::Unmark { site },
        _ => Input::Cut,
    }
}

fn invariant(s: &State) -> Result<(), String> {
    let order: Vec<&str> = sites().collect();
    let mut last = None;
    for k in s.marked() {
        let Some(i) = order.iter().position(|x| x == k) else {
            return Err(format!("đánh dấu vị trí lạ `{k}`"));
        };
        if last.is_some_and(|l| l >= i) {
            return Err(format!(
                "đánh dấu không theo thứ tự chuẩn: {:?}",
                s.marked()
            ));
        }
        last = Some(i);
    }
    if s.section().len() < 3 || s.section().len() > 6 {
        return Err(format!("thiết diện {} đỉnh", s.section().len()));
    }
    if s.solved() && s.cuts() == 0 {
        return Err("đúng mà chưa cắt".into());
    }
    Ok(())
}

fn json_roundtrip(t: &Trace<Input>) -> Result<Trace<Input>, String> {
    let s = serde_json::to_string(t).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

/// Cấu hình hợp lệ ngẫu nhiên: thử 3 điểm ngẫu nhiên đến khi `init` nhận.
fn config(seed: u64) -> Config {
    let mut rng = Rng::new(seed.wrapping_mul(7919).wrapping_add(1));
    loop {
        let points = (0..3)
            .map(|_| {
                let den = 1 + rng.below(6) as i64;
                let num = rng.below(den as u64 + 1) as i64;
                EdgePoint {
                    edge: rng.pick(&EDGES).unwrap_or("ab").to_owned(),
                    at: Rational::new(num, den).expect("mẫu khác 0"),
                }
            })
            .collect();
        let c = Config {
            attempts: 1 + (seed % 3) as u8,
            points,
        };
        if Section::init(&c).is_ok() {
            return c;
        }
    }
}

fn outcome_property(seed: u64, o: &Outcome) {
    assert_eq!(o.completed, o.score.permille() > 0, "seed {seed}: {o:?}");
}

#[test]
fn hop_dong() {
    let mut accepted = 0;
    let mut completed = 0;
    let mut with_findings = 0;
    let mut shapes = [0usize; 7];
    for seed in 0..CASES {
        let spec = Spec::<Section> {
            config: config(seed),
            steps: STEPS,
            input: &gen_input,
            invariant: &invariant,
            roundtrip: &json_roundtrip,
        };
        let checked = contract::check(seed, &spec).unwrap_or_else(|e| panic!("{e}"));
        outcome_property(seed, &checked.outcome);
        accepted += checked.accepted;
        completed += usize::from(checked.outcome.completed);
        with_findings += usize::from(!checked.outcome.findings.is_empty());
        let n = Section::init(&spec.config).expect("hợp lệ").section().len();
        shapes[n] += 1;
    }
    assert!(
        accepted > (CASES * 5) as usize,
        "chỉ {accepted} được chấp nhận"
    );
    assert!(completed > 10, "chỉ {completed} lần hoàn thành");
    assert!(with_findings > 10, "chỉ {with_findings} lần có phát hiện");
    // Bộ sinh phải chạm đủ tam giác, tứ giác, ngũ giác, lục giác.
    for (n, count) in shapes.iter().enumerate().skip(3) {
        assert!(
            *count > 0,
            "không sinh được thiết diện {n} đỉnh: {shapes:?}"
        );
    }
}
