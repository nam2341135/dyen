//! Property test cho bộ chấm: tính chất đúng với mọi quy tắc chẩn đoán (ADR 0009).
//!
//! - chấm tất định;
//! - bằng chứng tăng dần, không trùng, và chỉ trỏ vào bước của người học;
//! - `solved` khớp với điểm (đạt thì điểm > 0; chưa đạt thì điểm 0).
//!
//! Dấu vết ngẫu nhiên có seed cố định, sinh qua `Session` nên luôn hợp lệ.

use diagnose::{Task, assess_bars};
use domain::{Fraction, Relation};
use manip::bars::{self, Bars, TICKS};
use manip::{Actor, Answer, Session, Step};

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % n.max(1)
    }
}

fn random_trace(seed: u64, setup: &bars::Setup) -> manip::Trace<bars::Action> {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut s = Session::<Bars>::new(setup.clone()).expect("cấu hình hợp lệ");
    let mut at = 0;
    for _ in 0..30 {
        at += rng.below(2000) as u32;
        let actor = if rng.below(4) == 0 {
            Actor::System
        } else {
            Actor::Learner
        };
        let step = match rng.below(5) {
            0 => Step::Answer(Answer::Compare(
                [Relation::Less, Relation::Equal, Relation::Greater][rng.below(3) as usize],
            )),
            1 => Step::Model(bars::Action::Join {
                bar: 0,
                at: s.state().bar(0).and_then(|b| b.cuts().next()).unwrap_or(1),
            }),
            _ => Step::Model(bars::Action::Cut {
                bar: rng.below(2) as u8,
                at: 1 + rng.below(u64::from(TICKS) - 1) as u32,
            }),
        };
        let _ = s.act(at, actor, step); // bước bị từ chối không vào dấu vết
    }
    s.trace().clone()
}

#[test]
fn tinh_chat_cham() {
    let setup = bars::Setup {
        bars: 2,
        freehand: true,
        partitions: vec![3, 5],
    };
    let tasks = [
        Task::Partition { bar: 0, parts: 3 },
        Task::Partition { bar: 1, parts: 2 },
        Task::Compare {
            left: "1/3".parse::<Fraction>().expect("phân số"),
            right: "1/5".parse::<Fraction>().expect("phân số"),
        },
    ];
    for seed in 0..500 {
        let trace = random_trace(seed, &setup);
        for task in &tasks {
            let a = assess_bars(task, &setup, &trace).expect("chấm được");
            assert_eq!(
                Ok(&a),
                assess_bars(task, &setup, &trace).as_ref(),
                "seed {seed}"
            );
            assert_eq!(
                a.solved,
                a.score.permille() > 0,
                "seed {seed} {task:?}: {a:?}"
            );
            for o in &a.observations {
                assert!(
                    o.evidence.windows(2).all(|w| w[0] < w[1]),
                    "seed {seed}: bằng chứng không tăng dần {o:?}"
                );
                for i in &o.evidence {
                    let e = &trace.entries[*i as usize];
                    assert_eq!(
                        e.actor,
                        Actor::Learner,
                        "seed {seed}: bằng chứng là bước hệ thống"
                    );
                }
            }
        }
    }
}
