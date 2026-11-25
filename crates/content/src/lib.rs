//! Đọc và kiểm tra nội dung trong thư mục `content/` (ADR 0004, 0010).
//!
//! Bố cục:
//! - `tags.toml` — danh sách nhãn (`[[tag]]`) mà phát hiện và hoạt động dùng,
//! - `activities/<id>.toml` — một hoạt động mỗi file, cấu hình ở bảng `[config]`,
//! - `sets/<id>.toml` — một gói hoạt động mỗi file.
//!
//! Crate này không biết loại hoạt động nào: cấu hình được [`registry`] đọc và kiểm tra.
//! Phân tích ([`Library::from_sources`]) tách khỏi đọc đĩa ([`Library::load`]).
//! Mọi lỗi được gom lại và báo một lần.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use kernel::{ActivityId, ActivitySetId, TagId};
use registry::AnyConfig;
use serde::Deserialize;

/// Một nhãn: tên hiển thị cho mã mà phát hiện và hoạt động dùng.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tag {
    /// Mã.
    pub id: TagId,
    /// Tên hiển thị cho giáo viên.
    pub title: String,
}

/// Một hoạt động: chữ trình bày + cấu hình của một loại hoạt động.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    /// Mã, trùng tên file.
    pub id: ActivityId,
    /// Tên hiển thị.
    pub title: String,
    /// Lời dẫn cho người học.
    pub prompt: String,
    /// Nhãn để lọc, tìm kiếm (không ảnh hưởng chấm).
    #[serde(default)]
    pub tags: Vec<TagId>,
    /// Cấu hình, gồm `kind` chọn loại hoạt động.
    pub config: AnyConfig,
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
struct TagsFile {
    #[serde(default)]
    tag: Vec<Tag>,
}

/// Văn bản nguồn của nội dung, chưa phân tích.
///
/// Mỗi phần tử của `activities`/`sets` là `(tên file không đuôi, nội dung)`.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    /// Nội dung `tags.toml`.
    pub tags: String,
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
    tags: BTreeMap<TagId, Tag>,
    activities: BTreeMap<ActivityId, Activity>,
    sets: BTreeMap<ActivitySetId, ActivitySet>,
}

const TAGS_FILE: &str = "tags.toml";

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
            tags: BTreeMap::new(),
            activities: BTreeMap::new(),
            sets: BTreeMap::new(),
        };

        match toml::from_str::<TagsFile>(&src.tags) {
            Ok(file) => {
                for t in file.tag {
                    if lib.tags.contains_key(&t.id) {
                        errs.push(TAGS_FILE, format!("nhãn `{}` bị trùng", t.id));
                    } else {
                        lib.tags.insert(t.id.clone(), t);
                    }
                }
            }
            Err(e) => errs.push(TAGS_FILE, e.to_string()),
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
        let tags = std::fs::read_to_string(dir.join(TAGS_FILE)).unwrap_or_else(|e| {
            errs.push(TAGS_FILE, format!("không đọc được: {e}"));
            String::new()
        });
        let activities = read_toml_dir(dir, "activities", &mut errs);
        let sets = read_toml_dir(dir, "sets", &mut errs);
        if !errs.0.is_empty() {
            return Err(ContentErrors(errs.0));
        }
        Self::from_sources(&Sources {
            tags,
            activities,
            sets,
        })
    }

    /// Các kiểm tra chéo giữa các file.
    fn check(&self, errs: &mut Errors) {
        for id in registry::all_tags() {
            if !self.tags.keys().any(|k| k.as_str() == id) {
                errs.push(
                    TAGS_FILE,
                    format!("thiếu nhãn `{id}` mà một loại hoạt động dùng"),
                );
            }
        }

        for a in self.activities.values() {
            let file = activity_file(a.id.as_str());
            for t in &a.tags {
                if !self.tags.contains_key(t) {
                    errs.push(&file, format!("nhãn `{t}` không tồn tại"));
                }
            }
            for t in a.config.config_tags() {
                if !self.tags.contains_key(&t) {
                    errs.push(&file, format!("nhãn `{t}` trong cấu hình không tồn tại"));
                }
            }
            if let Err(e) = a.config.validate() {
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

    /// Các nhãn, theo mã.
    pub fn tags(&self) -> &BTreeMap<TagId, Tag> {
        &self.tags
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

    const TAGS: &str = r#"
[[tag]]
id = "frac.equal-parts"
title = "Các phần bằng nhau"

[[tag]]
id = "frac.compare-same-numerator"
title = "So sánh cùng tử số"

[[tag]]
id = "frac.unequal-parts"
title = "Chia đủ số phần nhưng không bằng nhau"

[[tag]]
id = "frac.larger-denominator-larger"
title = "Mẫu lớn hơn thì lớn hơn"

[[tag]]
id = "geo.section-only-given"
title = "Chỉ nối điểm đề cho"

[[tag]]
id = "geo.section-off-plane"
title = "Vị trí ngoài mặt cắt"
"#;

    const ACTIVITY: &str = r#"
id = "a1"
title = "Một phần ba và một phần năm"
prompt = "Phần nào lớn hơn?"
tags = ["frac.compare-same-numerator"]

[config]
kind = "bars"
bars = 2
partitions = [3, 5]

[config.task]
kind = "compare"
left = "1/3"
right = "1/5"
"#;

    const SET: &str = "id = \"s1\"\ntitle = \"G\"\nactivities = [\"a1\"]\n";

    fn sources() -> Sources {
        Sources {
            tags: TAGS.into(),
            activities: vec![("a1".into(), ACTIVITY.into())],
            sets: vec![("s1".into(), SET.into())],
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
        assert_eq!(lib.tags().len(), 6);
        assert_eq!(lib.activities().len(), 1);
        assert_eq!(lib.sets().len(), 1);
        let a = lib.activities().values().next().unwrap();
        assert_eq!(a.config.kind(), "bars");
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
    fn tham_chieu_hong() {
        let mut s = sources();
        s.activities[0].1 = ACTIVITY
            .replace("partitions = [3, 5]", "partitions = [3]")
            .replace(
                "tags = [\"frac.compare-same-numerator\"]",
                "tags = [\"zz.tag\"]",
            );
        s.sets[0].1 = SET.replace("[\"a1\"]", "[\"zz\", \"a1\", \"a1\"]");
        let e = errors(&s);
        assert!(e.iter().any(|m| m.contains("không dựng được 1/5")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("nhãn `zz.tag`")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("`zz` không tồn tại")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("lặp lại")), "{e:?}");
    }

    #[test]
    fn loai_la_va_thieu_nhan() {
        let mut s = sources();
        s.tags = TAGS.replace("frac.unequal-parts", "other.tag");
        s.activities[0].1 = ACTIVITY.replace("kind = \"bars\"", "kind = \"nope\"");
        let e = errors(&s);
        assert!(
            e.iter().any(|m| m.contains("`frac.unequal-parts`")),
            "{e:?}"
        );
        assert!(
            e.iter().any(|m| m.starts_with("activities/a1.toml")),
            "{e:?}"
        );
    }

    #[test]
    fn nhan_trong_cau_hinh_phai_ton_tai() {
        let choice = r#"
id = "c1"
title = "Chọn"
prompt = "Phân số nào lớn hơn?"

[config]
kind = "choice"

[[config.items]]
key = "a"
text = "1/3"
correct = true

[[config.items]]
key = "b"
text = "1/5"
tag = "frac.larger-denominator-larger"
"#;
        let mut s = sources();
        s.activities.push(("c1".into(), choice.into()));
        assert!(Library::from_sources(&s).is_ok());
        s.activities[1].1 = choice.replace("frac.larger-denominator-larger", "zz.missing");
        let e = errors(&s);
        assert!(
            e.iter()
                .any(|m| m.starts_with("activities/c1.toml") && m.contains("`zz.missing`")),
            "{e:?}"
        );
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
