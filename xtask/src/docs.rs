//! Kiểm tra tài liệu: những điều người (hoặc AI) đến sau dựa vào để biết phải làm gì.
//!
//! Tài liệu cũ đi lặng lẽ thì người đến sau làm theo điều sai. Ở đây chỉ kiểm những gì
//! máy kiểm được chắc chắn:
//! - mọi liên kết tương đối trong file `.md` trỏ tới file/thư mục có thật;
//! - bảng ADR trong `docs/README.md` liệt kê đủ mọi ADR, đúng trạng thái;
//! - `CLAUDE.md` nhập `AGENTS.md` (một nguồn hướng dẫn, không hai bản chép tay);
//! - `docs/STATUS.md` có đủ các mục mà `docs/playbooks/ban-giao.md` dựa vào;
//! - mỗi loại hoạt động `crates/kinds/<tên>` có README và test hợp đồng.

use std::fs;
use std::path::{Path, PathBuf};

use crate::workspace_root;

/// Thư mục không quét (sản phẩm build, nháp cục bộ không nằm trong git).
const SKIP_DIRS: &[&str] = &["target", "dist", ".git", ".superpowers", "node_modules"];
/// Các mục bắt buộc trong `docs/STATUS.md`.
const STATUS_SECTIONS: &[&str] = &["## Đang làm", "## Chờ người quyết", "## Tiếp theo"];
/// Các trạng thái ADR, theo mẫu `docs/adr/0000-mau.md`.
const ADR_STATUSES: &[&str] = &["Đề xuất", "Chấp nhận", "Bị thay thế", "Thay thế", "Bỏ"];

/// Chạy mọi kiểm tra, in lỗi, trả về `true` nếu không có lỗi.
pub fn check() -> bool {
    let root = workspace_root();
    let mut errors = Vec::new();
    check_links(&root, &mut errors);
    check_adr_table(&root, &mut errors);
    check_claude_imports_agents(&root, &mut errors);
    check_status(&root, &mut errors);
    check_kinds(&root, &mut errors);
    if errors.is_empty() {
        println!(
            "✓ docs: liên kết, bảng ADR, AGENTS/CLAUDE, STATUS, README loại hoạt động đều đúng"
        );
        true
    } else {
        for e in &errors {
            println!("✗ docs: {e}");
        }
        false
    }
}

fn read(path: &Path, errors: &mut Vec<String>) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) => {
            errors.push(format!("không đọc được {}: {e}", path.display()));
            None
        }
    }
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Mọi file `.md` dưới `dir`, bỏ qua [`SKIP_DIRS`], theo thứ tự tên.
fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        if path.is_dir() {
            if !name.is_some_and(|n| SKIP_DIRS.contains(&n.as_str())) {
                out.extend(markdown_files(&path));
            }
        } else if path.extension().is_some_and(|x| x == "md") {
            out.push(path);
        }
    }
    out
}

/// Đích của các liên kết Markdown `[chữ](đích)` cần kiểm: bỏ URL, neo `#…` trong trang,
/// và phần `#neo` / `:dòng` ở cuối. Bỏ qua khối code.
pub(crate) fn link_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let mut rest = line;
        while let Some(i) = rest.find("](") {
            rest = &rest[i + 2..];
            let Some(end) = rest.find(')') else { break };
            let target = rest[..end].split_whitespace().next().unwrap_or("");
            rest = &rest[end..];
            if target.is_empty()
                || target.starts_with('#')
                || target.contains("://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            let path = target.split('#').next().unwrap_or(target);
            // `file.rs:42` (liên kết tới dòng) → `file.rs`.
            let path = match path.rsplit_once(':') {
                Some((p, line)) if !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()) => {
                    p
                }
                _ => path,
            };
            out.push(path.to_owned());
        }
    }
    out
}

fn check_links(root: &Path, errors: &mut Vec<String>) {
    for file in markdown_files(root) {
        let Some(text) = read(&file, errors) else {
            continue;
        };
        let dir = file.parent().unwrap_or(root);
        for target in link_targets(&text) {
            if !dir.join(&target).exists() {
                errors.push(format!("{}: liên kết hỏng → {target}", rel(root, &file)));
            }
        }
    }
}

/// Trạng thái chuẩn ở đầu một chuỗi (bỏ `*` và khoảng trắng): `"**Chấp nhận** cho…"` → `Chấp nhận`.
pub(crate) fn status_of(s: &str) -> Option<&'static str> {
    let s = s.trim_start_matches(['*', ' ']);
    ADR_STATUSES.iter().copied().find(|st| s.starts_with(st))
}

/// Dòng trạng thái trong file ADR: `- **Trạng thái:** …`.
pub(crate) fn adr_file_status(text: &str) -> Option<&'static str> {
    text.lines()
        .find_map(|l| l.split_once("**Trạng thái:**").map(|(_, s)| s))
        .and_then(status_of)
}

/// Hàng của `file` trong bảng ADR: trạng thái ở cột cuối.
pub(crate) fn adr_table_status(readme: &str, file: &str) -> Option<Option<&'static str>> {
    let link = format!("(adr/{file})");
    readme
        .lines()
        .find(|l| l.starts_with('|') && l.contains(&link))
        .map(|row| {
            let cells: Vec<&str> = row.trim().trim_matches('|').split('|').collect();
            cells.last().and_then(|c| status_of(c))
        })
}

fn check_adr_table(root: &Path, errors: &mut Vec<String>) {
    let Some(readme) = read(&root.join("docs/README.md"), errors) else {
        return;
    };
    let Ok(entries) = fs::read_dir(root.join("docs/adr")) else {
        errors.push("không đọc được docs/adr".into());
        return;
    };
    let mut files: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".md") && n.as_bytes().first().is_some_and(u8::is_ascii_digit))
        .filter(|n| !n.starts_with("0000-"))
        .collect();
    files.sort();
    for file in files {
        let Some(text) = read(&root.join("docs/adr").join(&file), errors) else {
            continue;
        };
        let Some(want) = adr_file_status(&text) else {
            errors.push(format!(
                "docs/adr/{file}: không có dòng `**Trạng thái:**` hợp lệ"
            ));
            continue;
        };
        match adr_table_status(&readme, &file) {
            None => errors.push(format!("docs/README.md: bảng ADR thiếu {file}")),
            Some(got) if got != Some(want) => errors.push(format!(
                "docs/README.md: {file} ghi trạng thái {got:?}, file ghi {want:?}"
            )),
            Some(_) => {}
        }
    }
}

fn check_claude_imports_agents(root: &Path, errors: &mut Vec<String>) {
    let Some(text) = read(&root.join("CLAUDE.md"), errors) else {
        return;
    };
    if text.lines().find(|l| !l.trim().is_empty()).map(str::trim) != Some("@AGENTS.md") {
        errors.push(
            "CLAUDE.md: dòng đầu phải là `@AGENTS.md` — hướng dẫn chung viết ở AGENTS.md".into(),
        );
    }
}

fn check_status(root: &Path, errors: &mut Vec<String>) {
    let Some(text) = read(&root.join("docs/STATUS.md"), errors) else {
        return;
    };
    for s in STATUS_SECTIONS {
        if !text.lines().any(|l| l.trim_end() == *s) {
            errors.push(format!("docs/STATUS.md: thiếu mục `{s}`"));
        }
    }
}

fn check_kinds(root: &Path, errors: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root.join("crates/kinds")) else {
        errors.push("không đọc được crates/kinds".into());
        return;
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for dir in dirs {
        for need in ["README.md", "tests/contract.rs"] {
            if !dir.join(need).exists() {
                errors.push(format!("{}: thiếu {need}", rel(root, &dir)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lien_ket_bo_url_neo_va_dong() {
        let md = "xem [a](docs/a.md), [b](b.rs:42), [c](https://x.y), [d](#neo), [e](e.md#muc)\n\
                  ```\n[f](trong-code.md)\n```\n[g](g/ \"tiêu đề\")";
        assert_eq!(link_targets(md), ["docs/a.md", "b.rs", "e.md", "g/"]);
    }

    #[test]
    fn trang_thai_adr() {
        assert_eq!(status_of(" **Chấp nhận** cho Leptos"), Some("Chấp nhận"));
        assert_eq!(status_of("Bị thay thế bởi 0010"), Some("Bị thay thế"));
        assert_eq!(status_of("chưa rõ"), None);
        let adr = "# ADR\n\n- **Trạng thái:** Đề xuất\n- **Ngày:** 2026-11-26\n";
        assert_eq!(adr_file_status(adr), Some("Đề xuất"));
    }

    #[test]
    fn bang_adr() {
        let readme = "| ADR | Tiêu đề | Trạng thái |\n|---|---|---|\n\
                      | [0001](adr/0001-a.md) | A | Chấp nhận |\n\
                      | [0002](adr/0002-b.md) | B \\| C | Đề xuất |\n";
        assert_eq!(
            adr_table_status(readme, "0001-a.md"),
            Some(Some("Chấp nhận"))
        );
        assert_eq!(adr_table_status(readme, "0002-b.md"), Some(Some("Đề xuất")));
        assert_eq!(adr_table_status(readme, "0003-c.md"), None);
    }
}
