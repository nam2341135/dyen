//! Hợp đồng `Kind` cho loại chọn đáp án, qua bộ kiểm chung `kernel::contract`.
//!
//! Phần riêng: bộ sinh đầu vào ngẫu nhiên, bất biến trạng thái, và tính chất
//! `completed == (score > 0)`.

use kernel::contract::{self, Rng, Spec};
use kernel::{Outcome, TagId, Trace};
use kind_choice::{Choice, Config, Input, Item, State};

const CASES: u64 = 300;
const STEPS: u32 = 20;

fn gen_input(rng: &mut Rng, s: &State) -> Input {
    // Đôi khi dùng khóa không tồn tại để thử nhánh từ chối.
    let mut keys: Vec<&str> = s.keys().iter().map(String::as_str).collect();
    keys.push("khong-co");
    let key = rng.pick(&keys).unwrap_or("a").to_owned();
    match rng.below(5) {
        0 | 1 => Input::Select { key },
        2 => Input::Deselect { key },
        _ => Input::Submit,
    }
}

fn invariant(s: &State) -> Result<(), String> {
    let mut last = None;
    for k in s.selected() {
        let Some(i) = s.keys().iter().position(|x| x == k) else {
            return Err(format!("chọn khóa lạ `{k}`"));
        };
        if last.is_some_and(|l| l >= i) {
            return Err(format!(
                "lựa chọn không theo thứ tự cấu hình: {:?}",
                s.selected()
            ));
        }
        last = Some(i);
    }
    if !s.multiple() && s.selected().len() > 1 {
        return Err(format!("chọn một mà đang chọn {:?}", s.selected()));
    }
    if s.solved() && s.submissions() == 0 {
        return Err("đúng mà chưa nộp".into());
    }
    Ok(())
}

fn json_roundtrip(t: &Trace<Input>) -> Result<Trace<Input>, String> {
    let s = serde_json::to_string(t).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

fn config(seed: u64) -> Config {
    let n = 2 + (seed % 5) as usize;
    let multiple = seed % 2 == 1;
    let items = (0..n)
        .map(|i| Item {
            key: format!("k{i}"),
            text: format!("lựa chọn {i}"),
            // Chọn một: chỉ k0 đúng. Chọn nhiều: các khóa chẵn đúng.
            correct: if multiple { i % 2 == 0 } else { i == 0 },
            tag: (i % 3 == 1).then(|| TagId::new(format!("t.item-{i}")).expect("nhãn hợp lệ")),
        })
        .collect();
    Config {
        multiple,
        attempts: 1 + (seed % 3) as u8,
        items,
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
    for seed in 0..CASES {
        let spec = Spec::<Choice> {
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
    }
    // Phiên kết thúc sau vài lần nộp nên phần lớn đầu vào về sau bị từ chối là đúng;
    // chỉ đòi trung bình mỗi phiên có vài đầu vào được chấp nhận, và đi qua cả nhánh
    // hoàn thành lẫn nhánh có phát hiện.
    assert!(
        accepted > (CASES * 3) as usize,
        "chỉ {accepted} được chấp nhận"
    );
    assert!(completed > 10, "chỉ {completed} lần hoàn thành");
    assert!(with_findings > 10, "chỉ {with_findings} lần có phát hiện");
}
