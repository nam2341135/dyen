//! Đọc và kiểm tra nội dung học trong thư mục `content/` (ADR 0004, 0009).
//!
//! Bố cục:
//! - `concepts.toml` — đồ thị khái niệm (`[[concept]]`) và hiểu sai (`[[misconception]]`),
//! - `activities/<id>.toml` — một hoạt động mỗi file,
//! - `sets/<id>.toml` — một gói hoạt động mỗi file.
//!
//! Phân tích ([`Library::from_sources`]) tách khỏi đọc đĩa ([`Library::load`]).
//! Mọi lỗi được gom lại và báo một lần.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use diagnose::Task;
use domain::{ActivityId, ActivitySetId, ConceptId, MisconceptionId};
use manip::ModelSetup;
use serde::Deserialize;

/// Một khái niệm trong đồ thị kiến thức.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    /// Mã.
    pub id: ConceptId,
    /// Tên hiển thị.
    pub title: String,
    /// Khái niệm tiên quyết.
    #[serde(default)]
    pub requires: Vec<ConceptId>,
}

/// Một hiểu sai đã biết, gắn với một khái niệm.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Misconception {
    /// Mã.
    pub id: MisconceptionId,
    /// Khái niệm liên quan.
    pub concept: ConceptId,
    /// Mô tả ngắn cho giáo viên.
    pub title: String,
}

/// Một hoạt động: mô hình + nhiệm vụ.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    /// Mã, trùng tên file.
    pub id: ActivityId,
    /// Tên hiển thị.
    pub title: String,
    /// Lời dẫn cho người học.
    pub prompt: String,
    /// Khái niệm hoạt động luyện (ít nhất một).
    pub concepts: Vec<ConceptId>,
    /// Cấu hình mô hình.
    pub model: ModelSetup,
    /// Yêu cầu.
    pub task: Task,
}

/// Một gói hoạt động theo thứ tự.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivitySet {
    /// Mã, trùng tên file.
    pub id: ActivitySetId,
    /// Tên hiển thị.
    pub title: String,
    /// Các hoạt động, theo thứ tự.
    pub activities: Vec<ActivityId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConceptsFile {
    #[serde(default)]
    concept: Vec<Concept>,
    #[serde(default)]
    misconception: Vec<Misconception>,
}

/// Văn bản nguồn của nội dung, chưa phân tích.
///
/// Mỗi phần tử của `activities`/`sets` là `(tên file không đuôi, nội dung)`.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    /// Nội dung `concepts.toml`.
    pub concepts: String,
    /// Các file hoạt động.
    pub activities: Vec<(String, String)>,
    /// Các file gói hoạt động.
    pub sets: Vec<(String, String)>,
}

/// Một lỗi nội dung, kèm file gây lỗi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentError {
    /// Đường dẫn tương đối trong thư mục nội dung.
    pub file: String,
    /// Mô tả lỗi.
    pub message: String,
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.message)
    }
}

/// Tất cả lỗi nội dung tìm được (luôn khác rỗng).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub struct ContentErrors(pub Vec<ContentError>);

impl fmt::Display for ContentErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} lỗi nội dung:", self.0.len())?;
        for e in &self.0 {
            writeln!(f, "  {e}")?;
        }
        Ok(())
    }
}

/// Toàn bộ nội dung đã kiểm tra.
#[derive(Debug, Clone, PartialEq)]
pub struct Library {
    concepts: BTreeMap<ConceptId, Concept>,
    misconceptions: BTreeMap<MisconceptionId, Misconception>,
    activities: BTreeMap<ActivityId, Activity>,
    sets: BTreeMap<ActivitySetId, ActivitySet>,
}

const CONCEPTS_FILE: &str = "concepts.toml";

#[derive(Default)]
struct Errors(Vec<ContentError>);

impl Errors {
    fn push(&mut self, file: &str, message: impl Into<String>) {
        self.0.push(ContentError {
            file: file.to_owned(),
            message: message.into(),
        });
    }
}

fn activity_file(stem: &str) -> String {
    format!("activities/{stem}.toml")
}

fn set_file(stem: &str) -> String {
    format!("sets/{stem}.toml")
}

impl Library {
    /// Phân tích và kiểm tra nội dung từ văn bản nguồn. Thuần, không IO.
    pub fn from_sources(src: &Sources) -> Result<Self, ContentErrors> {
        let mut errs = Errors::default();
        let mut lib = Self {
            concepts: BTreeMap::new(),
            misconceptions: BTreeMap::new(),
            activities: BTreeMap::new(),
            sets: BTreeMap::new(),
        };

        match toml::from_str::<ConceptsFile>(&src.concepts) {
            Ok(file) => {
                for c in file.concept {
                    if lib.concepts.contains_key(&c.id) {
                        errs.push(CONCEPTS_FILE, format!("khái niệm `{}` bị trùng", c.id));
                    } else {
                        lib.concepts.insert(c.id.clone(), c);
                    }
                }
                for m in file.misconception {
                    if lib.misconceptions.contains_key(&m.id) {
                        errs.push(CONCEPTS_FILE, format!("hiểu sai `{}` bị trùng", m.id));
                    } else {
                        lib.misconceptions.insert(m.id.clone(), m);
                    }
                }
            }
            Err(e) => errs.push(CONCEPTS_FILE, e.to_string()),
        }

        for (stem, text) in &src.activities {
            let file = activity_file(stem);
            match toml::from_str::<Activity>(text) {
                Ok(a) if a.id.as_str() != stem => {
                    errs.push(&file, format!("mã `{}` không trùng tên file", a.id));
                }
                Ok(a) if lib.activities.contains_key(&a.id) => {
                    errs.push(&file, format!("hoạt động `{}` bị trùng", a.id));
                }
                Ok(a) => {
                    lib.activities.insert(a.id.clone(), a);
                }
                Err(e) => errs.push(&file, e.to_string()),
            }
        }

        for (stem, text) in &src.sets {
            let file = set_file(stem);
            match toml::from_str::<ActivitySet>(text) {
                Ok(s) if s.id.as_str() != stem => {
                    errs.push(&file, format!("mã `{}` không trùng tên file", s.id));
                }
                Ok(s) if lib.sets.contains_key(&s.id) => {
                    errs.push(&file, format!("gói `{}` bị trùng", s.id));
                }
                Ok(s) => {
                    lib.sets.insert(s.id.clone(), s);
                }
                Err(e) => errs.push(&file, e.to_string()),
            }
        }

        lib.check(&mut errs);
        if errs.0.is_empty() {
            Ok(lib)
        } else {
            Err(ContentErrors(errs.0))
        }
    }

    /// Đọc thư mục nội dung rồi gọi [`Library::from_sources`].
    ///
    /// Thư mục `activities/` hoặc `sets/` không tồn tại được coi là rỗng.
    pub fn load(dir: &Path) -> Result<Self, ContentErrors> {
        let mut errs = Errors::default();
        let concepts = std::fs::read_to_string(dir.join(CONCEPTS_FILE)).unwrap_or_else(|e| {
            errs.push(CONCEPTS_FILE, format!("không đọc được: {e}"));
            String::new()
        });
        let activities = read_toml_dir(dir, "activities", &mut errs);
        let sets = read_toml_dir(dir, "sets", &mut errs);
        if !errs.0.is_empty() {
            return Err(ContentErrors(errs.0));
        }
        Self::from_sources(&Sources {
            concepts,
            activities,
            sets,
        })
    }

    /// Các kiểm tra chéo giữa các file.
    fn check(&self, errs: &mut Errors) {
        for c in self.concepts.values() {
            for r in &c.requires {
                if !self.concepts.contains_key(r) {
                    errs.push(
                        CONCEPTS_FILE,
                        format!("khái niệm `{}` cần `{r}` không tồn tại", c.id),
                    );
                }
            }
        }
        if let Some(id) = self.find_cycle() {
            errs.push(
                CONCEPTS_FILE,
                format!("đồ thị khái niệm có chu trình qua `{id}`"),
            );
        }
        for m in self.misconceptions.values() {
            if !self.concepts.contains_key(&m.concept) {
                errs.push(
                    CONCEPTS_FILE,
                    format!(
                        "hiểu sai `{}` gắn với khái niệm `{}` không tồn tại",
                        m.id, m.concept
                    ),
                );
            }
        }
        for id in diagnose::CONCEPTS {
            if !self.concepts.keys().any(|k| k.as_str() == *id) {
                errs.push(
                    CONCEPTS_FILE,
                    format!("thiếu khái niệm `{id}` mà bộ chẩn đoán dùng"),
                );
            }
        }
        for id in diagnose::MISCONCEPTIONS {
            if !self.misconceptions.keys().any(|k| k.as_str() == *id) {
                errs.push(
                    CONCEPTS_FILE,
                    format!("thiếu hiểu sai `{id}` mà bộ chẩn đoán dùng"),
                );
            }
        }

        for a in self.activities.values() {
            let file = activity_file(a.id.as_str());
            if a.concepts.is_empty() {
                errs.push(&file, "phải luyện ít nhất một khái niệm");
            }
            for c in &a.concepts {
                if !self.concepts.contains_key(c) {
                    errs.push(&file, format!("khái niệm `{c}` không tồn tại"));
                }
            }
            if let Err(e) = a.model.validate() {
                errs.push(&file, e.to_string());
            } else if let Err(e) = diagnose::validate_task(&a.task, &a.model) {
                errs.push(&file, e.to_string());
            }
        }

        for s in self.sets.values() {
            let file = set_file(s.id.as_str());
            if s.activities.is_empty() {
                errs.push(&file, "gói rỗng");
            }
            let mut seen = BTreeSet::new();
            for a in &s.activities {
                if !self.activities.contains_key(a) {
                    errs.push(&file, format!("hoạt động `{a}` không tồn tại"));
                }
                if !seen.insert(a) {
                    errs.push(&file, format!("hoạt động `{a}` lặp lại"));
                }
            }
        }
    }

    /// Tìm một khái niệm nằm trên chu trình `requires`, nếu có.
    fn find_cycle(&self) -> Option<&ConceptId> {
        // 0 = chưa thăm, 1 = đang thăm, 2 = xong.
        let mut mark: BTreeMap<&ConceptId, u8> = BTreeMap::new();
        fn visit<'a>(
            lib: &'a Library,
            id: &'a ConceptId,
            mark: &mut BTreeMap<&'a ConceptId, u8>,
        ) -> Option<&'a ConceptId> {
            match mark.get(id) {
                Some(1) => return Some(id),
                Some(_) => return None,
                None => {}
            }
            mark.insert(id, 1);
            for r in lib
                .concepts
                .get(id)
                .map(|c| c.requires.as_slice())
                .unwrap_or(&[])
            {
                if let Some(found) = visit(lib, r, mark) {
                    return Some(found);
                }
            }
            mark.insert(id, 2);
            None
        }
        self.concepts
            .keys()
            .find_map(|id| visit(self, id, &mut mark))
    }

    /// Các khái niệm, theo mã.
    pub fn concepts(&self) -> &BTreeMap<ConceptId, Concept> {
        &self.concepts
    }

    /// Các hiểu sai, theo mã.
    pub fn misconceptions(&self) -> &BTreeMap<MisconceptionId, Misconception> {
        &self.misconceptions
    }

    /// Các hoạt động, theo mã.
    pub fn activities(&self) -> &BTreeMap<ActivityId, Activity> {
        &self.activities
    }

    /// Các gói hoạt động, theo mã.
    pub fn sets(&self) -> &BTreeMap<ActivitySetId, ActivitySet> {
        &self.sets
    }
}

/// Đọc mọi file `*.toml` trong `dir/sub`, sắp theo tên để kết quả tất định.
fn read_toml_dir(dir: &Path, sub: &str, errs: &mut Errors) -> Vec<(String, String)> {
    let path = dir.join(sub);
    if !path.exists() {
        return Vec::new();
    }
    let entries = match std::fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => {
            errs.push(sub, format!("không đọc được thư mục: {e}"));
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for entry in entries {
        let p = match entry {
            Ok(e) => e.path(),
            Err(e) => {
                errs.push(sub, e.to_string());
                continue;
            }
        };
        if p.extension().and_then(|x| x.to_str()) != Some("toml") {
            continue;
        }
        let Some(stem) = p.file_stem().and_then(|s| s.to_str()).map(str::to_owned) else {
            errs.push(sub, format!("tên file không hợp lệ: {}", p.display()));
            continue;
        };
        match std::fs::read_to_string(&p) {
            Ok(text) => out.push((stem, text)),
            Err(e) => errs.push(
                &format!("{sub}/{stem}.toml"),
                format!("không đọc được: {e}"),
            ),
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONCEPTS: &str = r#"
[[concept]]
id = "frac.equal-parts"
title = "Các phần bằng nhau"

[[concept]]
id = "frac.compare-same-numerator"
title = "So sánh cùng tử số"
requires = ["frac.equal-parts"]

[[misconception]]
id = "frac.unequal-parts"
concept = "frac.equal-parts"
title = "Chia đủ số phần nhưng không bằng nhau"

[[misconception]]
id = "frac.larger-denominator-larger"
concept = "frac.compare-same-numerator"
title = "Mẫu lớn hơn thì lớn hơn"
"#;

    const ACTIVITY: &str = r#"
id = "a1"
title = "Một phần ba và một phần năm"
prompt = "Phần nào lớn hơn?"
concepts = ["frac.compare-same-numerator"]

[model]
kind = "bars"
bars = 2
partitions = [3, 5]

[task]
kind = "compare"
left = "1/3"
right = "1/5"
"#;

    fn sources() -> Sources {
        Sources {
            concepts: CONCEPTS.into(),
            activities: vec![("a1".into(), ACTIVITY.into())],
            sets: vec![(
                "s1".into(),
                "id = \"s1\"\ntitle = \"G\"\nactivities = [\"a1\"]\n".into(),
            )],
        }
    }

    fn errors(src: &Sources) -> Vec<String> {
        Library::from_sources(src)
            .unwrap_err()
            .0
            .into_iter()
            .map(|e| e.to_string())
            .collect()
    }

    #[test]
    fn hop_le() {
        let lib = Library::from_sources(&sources()).unwrap();
        assert_eq!(lib.concepts().len(), 2);
        assert_eq!(lib.activities().len(), 1);
        assert_eq!(lib.sets().len(), 1);
    }

    #[test]
    fn ma_khac_ten_file_va_truong_la() {
        let mut s = sources();
        s.activities[0].0 = "a2".into();
        s.sets[0].1.push_str("extra = 1\n");
        let e = errors(&s);
        assert!(
            e.iter().any(|m| m.contains("không trùng tên file")),
            "{e:?}"
        );
        assert!(e.iter().any(|m| m.starts_with("sets/s1.toml")), "{e:?}");
    }

    #[test]
    fn tham_chieu_hong_va_chu_trinh() {
        let mut s = sources();
        s.concepts = s.concepts.replace(
            "title = \"Các phần bằng nhau\"",
            "title = \"x\"\nrequires = [\"frac.compare-same-numerator\"]",
        );
        s.activities[0].1 = ACTIVITY.replace("partitions = [3, 5]", "partitions = [3]");
        s.sets[0].1 = "id = \"s1\"\ntitle = \"G\"\nactivities = [\"zz\", \"a1\", \"a1\"]\n".into();
        let e = errors(&s);
        assert!(e.iter().any(|m| m.contains("chu trình")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("không dựng được 1/5")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("`zz` không tồn tại")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("lặp lại")), "{e:?}");
    }

    #[test]
    fn thieu_ma_chan_doan() {
        let mut s = sources();
        s.concepts = CONCEPTS.split("[[misconception]]").next().unwrap().into();
        let e = errors(&s);
        assert!(e.iter().any(|m| m.contains("frac.unequal-parts")), "{e:?}");
    }

    #[test]
    fn gom_moi_loi() {
        let mut s = sources();
        s.activities.push(("bad".into(), "id = ".into()));
        s.sets.push((
            "s2".into(),
            "id = \"s2\"\ntitle = \"\"\nactivities = []\n".into(),
        ));
        let e = Library::from_sources(&s).unwrap_err();
        assert_eq!(e.0.len(), 2, "{e}");
    }
}
