//! Kiểm tra ranh giới crate (ADR 0006) bằng `cargo metadata`.
//!
//! Sửa bảng `RULES` / `PURE_ALLOWED_EXTERNAL` ở đây **cùng lúc** với ADR 0006.

use std::collections::BTreeSet;
use std::process::Command;

use serde_json::Value;

/// (crate, các crate nội bộ được phép phụ thuộc, có phải crate thuần không).
///
/// Mẫu kết thúc bằng `*` khớp theo tiền tố: `kind-*` là mọi crate loại hoạt động
/// (`crates/kinds/<tên>`, ADR 0010) — mỗi loại chỉ được phụ thuộc lõi, không phụ thuộc nhau.
const RULES: &[(&str, &[&str], bool)] = &[
    ("kernel", &[], true),
    ("kind-*", &["kernel"], true),
    ("registry", &["kernel", "kind-*"], true),
    ("content", &["kernel", "registry"], false),
    ("cli", &["kernel", "registry", "content"], false),
    ("xtask", &[], false),
];

/// `name` khớp `pattern` (chính xác, hoặc theo tiền tố nếu `pattern` kết thúc bằng `*`).
fn matches(pattern: &str, name: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => name.starts_with(prefix),
        None => pattern == name,
    }
}

/// Thư viện bên ngoài mà crate thuần được phép phụ thuộc trực tiếp.
const PURE_ALLOWED_EXTERNAL: &[&str] = &["serde", "thiserror"];

/// Trả về `true` nếu mọi ranh giới được tôn trọng.
pub(crate) fn check() -> bool {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = match Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(crate::workspace_root())
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        Ok(o) => {
            eprintln!(
                "cargo metadata lỗi:\n{}",
                String::from_utf8_lossy(&o.stderr)
            );
            return false;
        }
        Err(e) => {
            eprintln!("không chạy được cargo metadata: {e}");
            return false;
        }
    };
    let meta: Value = match serde_json::from_slice(&out) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("không đọc được cargo metadata: {e}");
            return false;
        }
    };

    let errors = violations(&meta);
    if errors.is_empty() {
        println!("✓ deps: ranh giới crate đúng theo ADR 0006");
        true
    } else {
        for e in &errors {
            eprintln!("✗ {e}");
        }
        eprintln!(
            "\nSửa phụ thuộc, hoặc nếu thay đổi là có chủ đích: cập nhật xtask/src/deps.rs và ADR 0006."
        );
        false
    }
}

fn violations(meta: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let empty = Vec::new();
    let packages = meta["packages"].as_array().unwrap_or(&empty);
    let internal: BTreeSet<&str> = packages.iter().filter_map(|p| p["name"].as_str()).collect();

    for pkg in packages {
        let Some(name) = pkg["name"].as_str() else {
            continue;
        };
        let Some(&(_, allowed, pure)) = RULES.iter().find(|(n, _, _)| matches(n, name)) else {
            errors.push(format!(
                "crate `{name}` chưa có trong bảng ranh giới (xtask/src/deps.rs, ADR 0006)"
            ));
            continue;
        };
        for dep in pkg["dependencies"].as_array().unwrap_or(&empty) {
            // Phụ thuộc dev (chỉ dùng trong test) không tính vào ranh giới.
            if dep["kind"].as_str() == Some("dev") {
                continue;
            }
            let Some(dep_name) = dep["name"].as_str() else {
                continue;
            };
            if internal.contains(dep_name) {
                if !allowed.iter().any(|a| matches(a, dep_name)) {
                    errors.push(format!(
                        "`{name}` không được phụ thuộc crate nội bộ `{dep_name}`"
                    ));
                }
            } else if pure && !PURE_ALLOWED_EXTERNAL.contains(&dep_name) {
                errors.push(format!(
                    "crate thuần `{name}` không được phụ thuộc thư viện ngoài `{dep_name}`"
                ));
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn pkg(name: &str, deps: &[(&str, Option<&str>)]) -> Value {
        let deps: Vec<Value> = deps
            .iter()
            .map(|(n, kind)| json!({ "name": n, "kind": kind }))
            .collect();
        json!({ "name": name, "dependencies": deps })
    }

    #[test]
    fn hop_le_khong_co_loi() {
        let meta = json!({ "packages": [
            pkg("kernel", &[("serde", None)]),
            pkg("kind-bars", &[("kernel", None), ("thiserror", None)]),
            pkg("kind-graph", &[("kernel", None)]),
            pkg("registry", &[("kernel", None), ("kind-bars", None), ("kind-graph", None)]),
        ]});
        assert!(violations(&meta).is_empty(), "{:?}", violations(&meta));
    }

    #[test]
    fn bat_phu_thuoc_noi_bo_nguoc_chieu() {
        let meta = json!({ "packages": [
            pkg("kernel", &[("kind-bars", None)]),
            pkg("kind-bars", &[]),
        ]});
        let errs = violations(&meta);
        assert_eq!(errs.len(), 1, "{errs:?}");
        assert!(errs[0].contains("`kernel`") && errs[0].contains("`kind-bars`"));
    }

    #[test]
    fn loai_hoat_dong_khong_phu_thuoc_nhau() {
        let meta = json!({ "packages": [
            pkg("kind-a", &[("kind-b", None)]),
            pkg("kind-b", &[]),
        ]});
        assert_eq!(violations(&meta).len(), 1);
    }

    #[test]
    fn bat_thu_vien_ngoai_trong_crate_thuan() {
        let meta = json!({ "packages": [pkg("kind-bars", &[("bevy", None)])] });
        assert_eq!(violations(&meta).len(), 1);
    }

    #[test]
    fn bo_qua_phu_thuoc_dev() {
        let meta = json!({ "packages": [pkg("kernel", &[("proptest", Some("dev"))])] });
        assert!(violations(&meta).is_empty());
    }

    #[test]
    fn bat_crate_chua_dang_ky() {
        let meta = json!({ "packages": [pkg("server", &[])] });
        assert_eq!(violations(&meta).len(), 1);
    }
}
