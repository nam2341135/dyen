//! Test lõi bằng một loại hoạt động đồ chơi (`Counter`), không phụ thuộc loại thật nào.
//!
//! Kiểm: phát lại, lỗi phát lại, phiên, và chính bộ kiểm hợp đồng (bắt được loại hoạt động sai).

use kernel::contract::{self, Rng, Spec};
use kernel::{
    Actor, Confidence, ConfigError, Entry, Finding, Kind, Outcome, Rejection, ReplayError, Score,
    Session, SessionError, TRACE_FORMAT, TagId, Trace, replay, run,
};
use serde::{Deserialize, Serialize};

/// Đếm lên tới `limit`; vượt thì bị từ chối.
struct Counter;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Config {
    limit: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Add(u32);

fn reached(trace: &Trace<Add>, states: &[u32], limit: u32) -> Outcome {
    let completed = states.last() == Some(&limit);
    let findings = trace
        .learner_entries()
        .last()
        .filter(|_| completed)
        .map(|(i, _)| Finding {
            tag: TagId::new("count.reached").expect("mã hợp lệ"),
            recovered: false,
            confidence: Confidence::Tentative,
            evidence: vec![i],
        })
        .into_iter()
        .collect();
    Outcome {
        score: if completed { Score::FULL } else { Score::ZERO },
        completed,
        findings,
    }
}

impl Kind for Counter {
    const KIND: &'static str = "counter";
    const TAGS: &'static [&'static str] = &["count.reached"];
    type Config = Config;
    type State = u32;
    type Input = Add;

    fn init(config: &Config) -> Result<u32, ConfigError> {
        if config.limit == 0 {
            return Err(ConfigError("limit = 0".into()));
        }
        Ok(0)
    }

    fn apply(state: &u32, input: &Add) -> Result<u32, Rejection> {
        // Giới hạn cứng 10 để bộ sinh thử được nhánh từ chối.
        match state.checked_add(input.0) {
            Some(n) if n <= 10 && input.0 > 0 => Ok(n),
            _ => Err(Rejection(format!("không cộng được {}", input.0))),
        }
    }

    fn evaluate(config: &Config, trace: &Trace<Add>, states: &[u32]) -> Outcome {
        reached(trace, states, config.limit)
    }
}

/// Sai hợp đồng: lấy bằng chứng từ cả bước hệ thống.
struct Leaky;

impl Kind for Leaky {
    const KIND: &'static str = "leaky";
    const TAGS: &'static [&'static str] = &["count.reached"];
    type Config = Config;
    type State = u32;
    type Input = Add;

    fn init(config: &Config) -> Result<u32, ConfigError> {
        Counter::init(config)
    }

    fn apply(state: &u32, input: &Add) -> Result<u32, Rejection> {
        Counter::apply(state, input)
    }

    fn evaluate(_: &Config, trace: &Trace<Add>, _: &[u32]) -> Outcome {
        Outcome {
            score: Score::ZERO,
            completed: false,
            findings: vec![Finding {
                tag: TagId::new("count.reached").expect("mã hợp lệ"),
                recovered: false,
                confidence: Confidence::Strong,
                evidence: (0..u32::try_from(trace.entries.len()).expect("ngắn")).collect(),
            }],
        }
    }
}

fn entry(at_ms: u32, n: u32) -> Entry<Add> {
    Entry {
        at_ms,
        actor: Actor::Learner,
        input: Add(n),
    }
}

fn trace(entries: Vec<Entry<Add>>) -> Trace<Add> {
    Trace {
        format: TRACE_FORMAT,
        entries,
    }
}

#[test]
fn phat_lai_va_danh_gia() {
    let config = Config { limit: 5 };
    let t = trace(vec![entry(0, 2), entry(10, 3)]);
    let (r, outcome) = run::<Counter>(&config, &t).unwrap();
    assert_eq!(r.states, vec![0, 2, 5]);
    assert_eq!(*r.final_state(), 5);
    assert!(outcome.completed);
    assert_eq!(outcome.findings[0].evidence, vec![1]);
}

#[test]
fn loi_phat_lai() {
    let config = Config { limit: 5 };
    let mut t = trace(vec![entry(0, 2)]);
    t.format = 99;
    assert_eq!(
        replay::<Counter>(&config, &t),
        Err(ReplayError::Format { found: 99 })
    );
    let t = trace(vec![entry(5, 2), entry(1, 1)]);
    assert_eq!(
        replay::<Counter>(&config, &t),
        Err(ReplayError::TimeWentBack { index: 1 })
    );
    let t = trace(vec![entry(0, 2), entry(1, 20)]);
    assert!(matches!(
        replay::<Counter>(&config, &t),
        Err(ReplayError::Rejected { index: 1, .. })
    ));
    assert!(matches!(
        replay::<Counter>(&Config { limit: 0 }, &trace(vec![])),
        Err(ReplayError::Config(_))
    ));
}

#[test]
fn phien_chi_ghi_buoc_duoc_chap_nhan() {
    let mut s = Session::<Counter>::new(Config { limit: 5 }).unwrap();
    s.act(100, Actor::Learner, Add(2)).unwrap();
    let before = (*s.state(), s.trace().clone());
    assert!(matches!(
        s.act(200, Actor::Learner, Add(20)),
        Err(SessionError::Rejected(_))
    ));
    assert!(matches!(
        s.act(50, Actor::Learner, Add(1)),
        Err(SessionError::TimeWentBack { .. })
    ));
    assert_eq!((*s.state(), s.trace().clone()), before);
}

#[test]
fn dinh_dang_json_on_dinh() {
    let t = trace(vec![entry(7, 2)]);
    let json = serde_json::to_string(&t).unwrap();
    assert_eq!(
        json,
        r#"{"format":1,"entries":[{"at_ms":7,"actor":"learner","input":2}]}"#
    );
}

fn json_roundtrip(t: &Trace<Add>) -> Result<Trace<Add>, String> {
    let s = serde_json::to_string(t).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

fn gen_add(rng: &mut Rng, _: &u32) -> Add {
    Add(rng.below(5) as u32)
}

fn no_invariant(_: &u32) -> Result<(), String> {
    Ok(())
}

#[test]
fn hop_dong_dung_voi_counter() {
    let spec = Spec::<Counter> {
        config: Config { limit: 10 },
        steps: 30,
        input: &gen_add,
        invariant: &no_invariant,
        roundtrip: &json_roundtrip,
    };
    let mut accepted = 0;
    for seed in 0..200 {
        accepted += contract::check(seed, &spec)
            .unwrap_or_else(|e| panic!("{e}"))
            .accepted;
    }
    assert!(accepted > 200, "chỉ {accepted} đầu vào được chấp nhận");
}

#[test]
fn hop_dong_bat_loai_sai() {
    let spec = Spec::<Leaky> {
        config: Config { limit: 10 },
        steps: 30,
        input: &gen_add,
        invariant: &no_invariant,
        roundtrip: &json_roundtrip,
    };
    let caught = (0..50)
        .filter(|s| contract::check(*s, &spec).is_err())
        .count();
    assert!(
        caught > 0,
        "bộ kiểm không bắt được bằng chứng là bước hệ thống"
    );
}

#[test]
fn hop_dong_bat_bat_bien_sai() {
    let spec = Spec::<Counter> {
        config: Config { limit: 10 },
        steps: 30,
        input: &gen_add,
        invariant: &|s| if *s > 6 { Err("quá 6".into()) } else { Ok(()) },
        roundtrip: &json_roundtrip,
    };
    assert!((0..50).any(|s| contract::check(s, &spec).is_err()));
}
