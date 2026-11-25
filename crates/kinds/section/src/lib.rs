//! Loại hoạt động `section`: tìm thiết diện của hình lập phương cắt bởi mặt phẳng qua 3 điểm.
//!
//! Đây là một **ý tưởng** cắm vào lõi, không phải một phần của lõi (ADR 0010).
//! Xóa crate này và một dòng trong `registry` là gỡ hết ý tưởng.
//!
//! Đề cho 3 điểm trên cạnh của lập phương `ABCD.A'B'C'D'`. Người học đánh dấu mọi
//! **vị trí** (đỉnh hoặc cạnh của lập phương) nơi có một đỉnh của thiết diện, rồi cắt.
//! Mọi quyết định dùng số hữu tỉ chính xác (`lib-exact`, ADR 0012).
//!
//! Thuật ngữ riêng (vị trí, đỉnh thiết diện, lần cắt) nằm ở `README.md` của crate.

use std::cmp::Ordering;

use kernel::{Confidence, ConfigError, Finding, Kind, Outcome, Rejection, Score, TagId, Trace};
use lib_exact::{ExactError, Plane, Rational, Vec3};
use serde::{Deserialize, Serialize};

/// Mẫu số lớn nhất cho tham số điểm trên cạnh. Giữ mọi phép tính xa ngưỡng tràn `i64`.
pub const MAX_DENOM: i64 = 1000;

/// Nhãn mà loại này có thể gắn vào phát hiện.
pub mod tags {
    /// Chỉ nối các điểm đề cho, quên tìm giao tuyến với các mặt khác.
    pub const ONLY_GIVEN: &str = "geo.section-only-given";
    /// Đánh dấu một vị trí mà mặt phẳng không đi qua.
    pub const OFF_PLANE: &str = "geo.section-off-plane";
}

/// Đỉnh của lập phương: khóa và tọa độ (cạnh dài 1). `a1` là `A'`.
const VERTICES: [(&str, [i64; 3]); 8] = [
    ("a", [0, 0, 0]),
    ("b", [1, 0, 0]),
    ("c", [1, 1, 0]),
    ("d", [0, 1, 0]),
    ("a1", [0, 0, 1]),
    ("b1", [1, 0, 1]),
    ("c1", [1, 1, 1]),
    ("d1", [0, 1, 1]),
];

/// Cạnh của lập phương: khóa, đỉnh đầu, đỉnh cuối (chỉ số trong [`VERTICES`]).
/// Tham số `at` của điểm trên cạnh tính từ đỉnh đầu.
const EDGES: [(&str, usize, usize); 12] = [
    ("ab", 0, 1),
    ("bc", 1, 2),
    ("cd", 2, 3),
    ("da", 3, 0),
    ("a1b1", 4, 5),
    ("b1c1", 5, 6),
    ("c1d1", 6, 7),
    ("d1a1", 7, 4),
    ("aa1", 0, 4),
    ("bb1", 1, 5),
    ("cc1", 2, 6),
    ("dd1", 3, 7),
];

/// Mọi vị trí theo thứ tự chuẩn: 8 đỉnh rồi 12 cạnh.
pub fn sites() -> impl Iterator<Item = &'static str> {
    VERTICES
        .iter()
        .map(|(k, _)| *k)
        .chain(EDGES.iter().map(|(k, _, _)| *k))
}

fn site_index(key: &str) -> Option<usize> {
    sites().position(|k| k == key)
}

fn vertex(i: usize) -> Vec3 {
    let [x, y, z] = VERTICES[i].1;
    Vec3::int(x, y, z)
}

/// Loại hoạt động thiết diện (kiểu đánh dấu cho [`Kind`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section;

/// Cấu hình một hoạt động thiết diện.
///
/// TOML (trong bảng `[config]` của hoạt động):
/// ```toml
/// kind = "section"
/// attempts = 3
/// points = [
///   { edge = "bc", at = "1/2" },
///   { edge = "cd", at = "1/2" },
///   { edge = "a1b1", at = "1/2" },
/// ]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Số lần cắt tối đa, `>= 1`.
    #[serde(default = "default_attempts")]
    pub attempts: u8,
    /// Đúng 3 điểm xác định mặt phẳng cắt.
    pub points: Vec<EdgePoint>,
}

fn default_attempts() -> u8 {
    3
}

/// Một điểm trên cạnh của lập phương.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgePoint {
    /// Khóa cạnh, vd. `"ab"`, `"a1b1"`, `"cc1"`.
    pub edge: String,
    /// Vị trí trên cạnh tính từ đỉnh đầu, `0..=1`, mẫu số `<= MAX_DENOM`.
    /// `0` hoặc `1` là chính đỉnh của lập phương.
    pub at: Rational,
}

/// Một đầu vào.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    /// Đánh dấu một vị trí là có đỉnh thiết diện.
    Mark {
        /// Khóa vị trí (đỉnh hoặc cạnh).
        site: String,
    },
    /// Bỏ đánh dấu.
    Unmark {
        /// Khóa vị trí.
        site: String,
    },
    /// Cắt: nộp tập vị trí đang đánh dấu (ít nhất 3).
    Cut,
}

/// Một đỉnh của thiết diện: vị trí trên lập phương và tọa độ chính xác.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Corner {
    /// Khóa vị trí.
    pub site: String,
    /// Tọa độ (cạnh lập phương dài 1).
    pub point: Vec3,
}

/// Trạng thái. Mang theo lời giải để `apply` tự chấm từng lần cắt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    attempts: u8,
    points: Vec<Vec3>,
    given_sites: Vec<String>,
    section: Vec<Corner>,
    marked: Vec<String>,
    cuts: u8,
    solved: bool,
}

impl State {
    /// Tọa độ 3 điểm đề cho.
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// Vị trí của các điểm đề cho, theo thứ tự chuẩn, không trùng.
    pub fn given_sites(&self) -> &[String] {
        &self.given_sites
    }

    /// Lời giải: các đỉnh thiết diện theo thứ tự chuẩn của vị trí (không theo vòng đa giác;
    /// bộ hiển thị tự sắp khi vẽ). Chỉ nên hiện sau khi cắt.
    pub fn section(&self) -> &[Corner] {
        &self.section
    }

    /// Các vị trí đang đánh dấu, theo thứ tự chuẩn.
    pub fn marked(&self) -> &[String] {
        &self.marked
    }

    /// Số lần đã cắt.
    pub fn cuts(&self) -> u8 {
        self.cuts
    }

    /// Số lần cắt còn lại.
    pub fn attempts_left(&self) -> u8 {
        self.attempts.saturating_sub(self.cuts)
    }

    /// Đã cắt đúng chưa.
    pub fn solved(&self) -> bool {
        self.solved
    }

    /// Hoạt động đã kết thúc (đúng, hoặc hết lượt cắt).
    pub fn finished(&self) -> bool {
        self.solved || self.attempts_left() == 0
    }

    /// Dấu hiệu `geo.section-only-given`: sai, và chỉ đánh dấu vị trí của điểm đề cho.
    fn only_given(&self) -> bool {
        !self.solved && self.marked.iter().all(|m| self.given_sites.contains(m))
    }

    /// Dấu hiệu `geo.section-off-plane`: có vị trí đánh dấu không chứa đỉnh thiết diện.
    fn off_plane(&self) -> bool {
        self.marked
            .iter()
            .any(|m| !self.section.iter().any(|c| c.site == *m))
    }

    fn is_correct(&self) -> bool {
        self.marked.len() == self.section.len()
            && self
                .marked
                .iter()
                .zip(&self.section)
                .all(|(m, c)| *m == c.site)
    }
}

/// Một dấu hiệu sai lầm, xét trên trạng thái sau một lần cắt.
type Sign = fn(&State) -> bool;

fn reject(msg: impl Into<String>) -> Result<State, Rejection> {
    Err(Rejection(msg.into()))
}

fn overflow(e: ExactError) -> ConfigError {
    ConfigError(format!("lỗi số học khi dựng thiết diện: {e}"))
}

/// Vị trí và tọa độ của một điểm trên cạnh.
fn locate(p: &EdgePoint) -> Result<(String, Vec3), ConfigError> {
    let Some(&(_, from, to)) = EDGES.iter().find(|(k, _, _)| *k == p.edge) else {
        return Err(ConfigError(format!("không có cạnh `{}`", p.edge)));
    };
    if p.at < Rational::ZERO || p.at > Rational::ONE {
        return Err(ConfigError(format!("`at = {}` phải trong [0, 1]", p.at)));
    }
    if p.at.denom() > MAX_DENOM {
        return Err(ConfigError(format!(
            "`at = {}` có mẫu số lớn hơn {MAX_DENOM}",
            p.at
        )));
    }
    let site = if p.at == Rational::ZERO {
        VERTICES[from].0
    } else if p.at == Rational::ONE {
        VERTICES[to].0
    } else {
        p.edge.as_str()
    };
    let point = vertex(from).lerp(vertex(to), p.at).map_err(overflow)?;
    Ok((site.to_owned(), point))
}

/// Các đỉnh thiết diện: đỉnh lập phương nằm trên mặt, và giao điểm với cạnh có hai đầu
/// nằm hẳn hai phía. Đó đúng là các đỉnh của đa giác thiết diện (lập phương lồi).
fn section_of(plane: &Plane) -> Result<Vec<Corner>, ExactError> {
    let mut sides = [Ordering::Equal; 8];
    for (i, s) in sides.iter_mut().enumerate() {
        *s = plane.side(vertex(i))?;
    }
    let mut corners = Vec::new();
    for (i, (key, _)) in VERTICES.iter().enumerate() {
        if sides[i] == Ordering::Equal {
            corners.push(Corner {
                site: (*key).to_owned(),
                point: vertex(i),
            });
        }
    }
    for (key, from, to) in EDGES {
        if sides[from] != Ordering::Equal
            && sides[to] != Ordering::Equal
            && sides[from] != sides[to]
        {
            let (a, b) = (vertex(from), vertex(to));
            if let Some(t) = plane.cut_segment(a, b)? {
                corners.push(Corner {
                    site: key.to_owned(),
                    point: a.lerp(b, t)?,
                });
            }
        }
    }
    Ok(corners)
}

impl Kind for Section {
    const KIND: &'static str = "section";
    const TAGS: &'static [&'static str] = &[tags::ONLY_GIVEN, tags::OFF_PLANE];
    type Config = Config;
    type State = State;
    type Input = Input;

    fn init(config: &Config) -> Result<State, ConfigError> {
        if config.attempts == 0 {
            return Err(ConfigError("attempts phải >= 1".into()));
        }
        let [p, q, r] = config.points.as_slice() else {
            return Err(ConfigError(format!(
                "cần đúng 3 điểm, có {}",
                config.points.len()
            )));
        };
        let (located, points): (Vec<String>, Vec<Vec3>) = [p, q, r]
            .into_iter()
            .map(locate)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        let plane = Plane::through(points[0], points[1], points[2])
            .map_err(overflow)?
            .ok_or_else(|| ConfigError("3 điểm thẳng hàng hoặc trùng nhau".into()))?;
        let mut above = false;
        let mut below = false;
        for i in 0..VERTICES.len() {
            match plane.side(vertex(i)).map_err(overflow)? {
                Ordering::Greater => above = true,
                Ordering::Less => below = true,
                Ordering::Equal => {}
            }
        }
        if !(above && below) {
            return Err(ConfigError(
                "mặt phẳng không cắt vào trong lập phương (chỉ chạm hoặc trùng một mặt)".into(),
            ));
        }
        let section = section_of(&plane).map_err(overflow)?;
        let mut given_sites: Vec<String> = Vec::new();
        for key in sites() {
            if located.iter().any(|s| s == key) {
                given_sites.push(key.to_owned());
            }
        }
        Ok(State {
            attempts: config.attempts,
            points,
            given_sites,
            section,
            marked: Vec::new(),
            cuts: 0,
            solved: false,
        })
    }

    fn apply(state: &State, input: &Input) -> Result<State, Rejection> {
        if state.solved {
            return reject("đã cắt đúng");
        }
        if state.attempts_left() == 0 {
            return reject("hết lượt cắt");
        }
        let mut next = state.clone();
        match input {
            Input::Mark { site } => {
                if site_index(site).is_none() {
                    return reject(format!("không có vị trí `{site}`"));
                }
                if state.marked.contains(site) {
                    return reject(format!("`{site}` đã được đánh dấu"));
                }
                next.marked.push(site.clone());
                // Giữ thứ tự chuẩn để trạng thái có dạng duy nhất.
                next.marked
                    .sort_by_key(|k| site_index(k).unwrap_or(usize::MAX));
            }
            Input::Unmark { site } => {
                let Some(pos) = state.marked.iter().position(|k| k == site) else {
                    return reject(format!("`{site}` chưa được đánh dấu"));
                };
                next.marked.remove(pos);
            }
            Input::Cut => {
                if state.marked.len() < 3 {
                    return reject("thiết diện cần ít nhất 3 đỉnh");
                }
                next.cuts += 1;
                next.solved = state.is_correct();
            }
        }
        Ok(next)
    }

    fn evaluate(_config: &Config, trace: &Trace<Input>, states: &[State]) -> Outcome {
        // Các lần cắt của người học: (chỉ số bước, trạng thái sau bước).
        let cuts: Vec<(u32, &State)> = trace
            .learner_entries()
            .filter(|(_, e)| e.input == Input::Cut)
            .filter_map(|(i, _)| {
                let after = states.get(usize::try_from(i).ok()?.checked_add(1)?)?;
                Some((i, after))
            })
            .collect();
        // Lần cắt đúng (nếu có) là lần cuối, vì sau đó mọi đầu vào bị từ chối.
        let solved_at = cuts.iter().find(|(_, s)| s.solved).copied();
        let completed = solved_at.is_some();
        // Điểm = 1000 / số thứ tự lần cắt đúng (tính cả lần cắt của hệ thống).
        let score = solved_at
            .and_then(|(_, s)| Score::from_permille(1000 / u16::from(s.cuts.max(1))))
            .unwrap_or(Score::ZERO);

        let checks: [(&str, Sign); 2] = [
            (tags::ONLY_GIVEN, State::only_given),
            (tags::OFF_PLANE, State::off_plane),
        ];

        let mut findings = Vec::new();
        for (tag, test) in checks {
            let mut evidence: Vec<u32> = cuts
                .iter()
                .filter(|(_, s)| test(s))
                .map(|(i, _)| *i)
                .collect();
            if evidence.is_empty() {
                continue;
            }
            let confidence = if evidence.len() >= 2 {
                Confidence::Strong
            } else {
                Confidence::Tentative
            };
            // Lần cắt đúng không mang dấu hiệu nào, nên cắt đúng về sau là đã rời khỏi nó.
            if let Some((i, _)) = solved_at {
                evidence.push(i);
            }
            findings.push(Finding {
                tag: TagId::new(tag).unwrap_or_else(|_| unreachable!("nhãn hằng hợp lệ")),
                recovered: completed,
                confidence,
                evidence,
            });
        }
        Outcome {
            score,
            completed,
            findings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{Actor, Session, run};

    fn pt(edge: &str, at: &str) -> EdgePoint {
        EdgePoint {
            edge: edge.into(),
            at: at.parse().expect("số hữu tỉ"),
        }
    }

    fn config(points: &[(&str, &str)]) -> Config {
        Config {
            attempts: 3,
            points: points.iter().map(|(e, a)| pt(e, a)).collect(),
        }
    }

    /// Lục giác đều: mặt phẳng `x + y + z = 3/2` qua trung điểm 6 cạnh.
    fn hexagon() -> Config {
        config(&[("bc", "1/2"), ("cd", "1/2"), ("a1b1", "1/2")])
    }

    fn sites_of(s: &State) -> Vec<&str> {
        s.section().iter().map(|c| c.site.as_str()).collect()
    }

    fn mark(site: &str) -> Input {
        Input::Mark { site: site.into() }
    }

    fn play(config: Config, inputs: &[Input]) -> Outcome {
        let mut s = Session::<Section>::new(config).expect("cấu hình hợp lệ");
        for (t, i) in inputs.iter().enumerate() {
            s.act(t as u32 * 1000, Actor::Learner, i.clone())
                .expect("đầu vào hợp lệ");
        }
        run::<Section>(s.config(), s.trace())
            .expect("phát lại được")
            .1
    }

    #[test]
    fn luc_giac_qua_trung_diem() {
        let s = Section::init(&hexagon()).expect("hợp lệ");
        assert_eq!(sites_of(&s), ["bc", "cd", "a1b1", "d1a1", "bb1", "dd1"]);
        assert_eq!(s.given_sites(), ["bc", "cd", "a1b1"]);
        let half: Rational = "1/2".parse().expect("số");
        let bb1 = &s.section()[4];
        assert_eq!(bb1.point, Vec3::new(Rational::ONE, Rational::ZERO, half));
    }

    #[test]
    fn tam_giac_o_goc_va_qua_dinh() {
        let s = Section::init(&config(&[("ab", "1/3"), ("da", "1/2"), ("aa1", "1/4")]))
            .expect("hợp lệ");
        assert_eq!(sites_of(&s), ["ab", "da", "aa1"]);
        // B, D, A': điểm ở đầu cạnh là chính đỉnh.
        let s = Section::init(&config(&[("ab", "1"), ("da", "0"), ("aa1", "1")])).expect("hợp lệ");
        assert_eq!(sites_of(&s), ["b", "d", "a1"]);
        assert_eq!(s.given_sites(), ["b", "d", "a1"]);
    }

    #[test]
    fn hinh_chu_nhat_qua_canh() {
        // Mặt chéo ACC'A': qua 4 đỉnh.
        let s = Section::init(&config(&[("ab", "0"), ("bc", "1"), ("cc1", "1")])).expect("hợp lệ");
        assert_eq!(sites_of(&s), ["a", "c", "a1", "c1"]);
    }

    #[test]
    fn ngu_giac() {
        // Qua trung điểm AB, AD và điểm trên CC' gần C': cắt 5 cạnh.
        let s = Section::init(&config(&[("ab", "1/2"), ("da", "1/2"), ("cc1", "3/4")]))
            .expect("hợp lệ");
        assert_eq!(s.section().len(), 5, "{:?}", sites_of(&s));
    }

    #[test]
    fn cau_hinh_sai_bi_tu_choi() {
        let bad = [
            config(&[("ab", "1/2"), ("bc", "1/2")]),
            config(&[("ab", "0"), ("ab", "1/2"), ("ab", "1")]),
            config(&[("ab", "0"), ("ab", "1"), ("bc", "1")]),
            config(&[("ab", "1"), ("bc", "0"), ("bb1", "0")]),
            config(&[("xy", "1/2"), ("bc", "1/2"), ("cc1", "1/2")]),
            config(&[("ab", "3/2"), ("bc", "1/2"), ("cc1", "1/2")]),
            config(&[("ab", "-1/2"), ("bc", "1/2"), ("cc1", "1/2")]),
            config(&[("ab", "1/1001"), ("bc", "1/2"), ("cc1", "1/2")]),
        ];
        for c in bad {
            assert!(Section::init(&c).is_err(), "{c:?}");
        }
        let mut c = hexagon();
        c.attempts = 0;
        assert!(Section::init(&c).is_err());
    }

    #[test]
    fn mau_so_sat_bien_khong_tran() {
        let s = Section::init(&config(&[
            ("ab", "997/1000"),
            ("bc", "1/999"),
            ("cc1", "998/999"),
        ]))
        .expect("hợp lệ");
        assert!(s.section().len() >= 3);
        for c in s.section() {
            for v in [c.point.x, c.point.y, c.point.z] {
                assert!(v >= Rational::ZERO && v <= Rational::ONE, "{c:?}");
            }
        }
    }

    #[test]
    fn danh_dau_va_tu_choi() {
        let s = Section::init(&hexagon()).expect("hợp lệ");
        assert!(Section::apply(&s, &mark("zz")).is_err());
        assert!(Section::apply(&s, &Input::Cut).is_err(), "chưa đủ 3 đỉnh");
        let s = Section::apply(&s, &mark("dd1")).expect("đánh dấu");
        let s = Section::apply(&s, &mark("b")).expect("đánh dấu");
        assert_eq!(s.marked(), ["b", "dd1"]);
        assert!(Section::apply(&s, &mark("b")).is_err());
        assert!(Section::apply(&s, &Input::Unmark { site: "c".into() }).is_err());
        let s = Section::apply(&s, &Input::Unmark { site: "b".into() }).expect("bỏ");
        assert_eq!(s.marked(), ["dd1"]);
    }

    #[test]
    fn dung_ngay_du_diem() {
        let mut inputs: Vec<Input> = ["bc", "cd", "d1a1", "a1b1", "bb1", "dd1"]
            .into_iter()
            .map(mark)
            .collect();
        inputs.push(Input::Cut);
        let o = play(hexagon(), &inputs);
        assert!(o.completed);
        assert_eq!(o.score, Score::FULL);
        assert!(o.findings.is_empty());
    }

    #[test]
    fn chi_noi_diem_cho_roi_sua() {
        let o = play(
            hexagon(),
            &[
                mark("bc"),
                mark("cd"),
                mark("a1b1"),
                Input::Cut,
                mark("bb1"),
                mark("dd1"),
                mark("d1a1"),
                Input::Cut,
            ],
        );
        assert!(o.completed);
        assert_eq!(o.score.permille(), 500);
        assert_eq!(o.findings.len(), 1);
        let f = &o.findings[0];
        assert_eq!(f.tag.as_str(), tags::ONLY_GIVEN);
        assert!(f.recovered);
        assert_eq!(f.evidence, vec![3, 7]);
    }

    #[test]
    fn danh_dau_ngoai_mat_phang() {
        let o = play(
            hexagon(),
            &[
                mark("bc"),
                mark("cd"),
                mark("a1b1"),
                mark("cc1"),
                Input::Cut,
                Input::Cut,
            ],
        );
        assert!(!o.completed);
        assert_eq!(o.score, Score::ZERO);
        assert_eq!(o.findings.len(), 1);
        let f = &o.findings[0];
        assert_eq!(f.tag.as_str(), tags::OFF_PLANE);
        assert!(!f.recovered);
        assert_eq!(f.confidence, Confidence::Strong);
        assert_eq!(f.evidence, vec![4, 5]);
    }

    #[test]
    fn nhan_hang_hop_le() {
        for t in Section::TAGS {
            assert!(TagId::new(*t).is_ok(), "{t}");
        }
    }
}
