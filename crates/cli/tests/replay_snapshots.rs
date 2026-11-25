//! Snapshot test cho `dyen replay` (tầng 3).
//!
//! Mỗi `traces/<hoạt-động>--<tình-huống>.json` là một dấu vết mẫu; kết quả phát lại
//! và chấm được so với `<cùng tên>.expected.json`. Đây là hàng rào giữ định dạng
//! dấu vết `format = 1` đọc được mãi (ADR 0003): nếu snapshot đổi, phải giải thích được.
//!
//! Tình huống có tên chứa `--hong-` là dấu vết hỏng: phải bị từ chối với mã thoát 2.
//!
//! Cập nhật snapshot có chủ đích: `UPDATE_SNAPSHOTS=1 cargo test -p cli`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn dyen(args: &[&str]) -> std::process::Output {
    let content = root().join("../../content");
    Command::new(env!("CARGO_BIN_EXE_dyen"))
        .args(args)
        .arg("--content")
        .arg(content)
        .output()
        .expect("chạy được dyen")
}

fn trace_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("tests/traces"))
        .expect("có thư mục tests/traces")
        .map(|e| e.expect("đọc được mục").path())
        .filter(|p| {
            p.extension().is_some_and(|x| x == "json")
                && !p.to_string_lossy().ends_with(".expected.json")
        })
        .collect();
    files.sort();
    files
}

#[test]
fn snapshot_phat_lai() {
    let update = std::env::var_os("UPDATE_SNAPSHOTS").is_some();
    let files = trace_files();
    assert!(!files.is_empty(), "chưa có dấu vết mẫu");
    let mut failures = Vec::new();
    for file in &files {
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("tên file UTF-8");
        let (activity, case) = stem
            .split_once("--")
            .expect("tên dạng <hoạt-động>--<tình-huống>");
        let out = dyen(&["replay", activity, file.to_str().expect("đường dẫn UTF-8")]);
        let stdout = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
        let stderr = String::from_utf8_lossy(&out.stderr);

        if case.starts_with("hong-") {
            if out.status.code() != Some(2) {
                failures.push(format!(
                    "{stem}: mong mã thoát 2, nhận {:?}",
                    out.status.code()
                ));
            }
            continue;
        }
        if !out.status.success() {
            failures.push(format!("{stem}: dyen lỗi: {stderr}"));
            continue;
        }
        let expected_path = file.with_extension("expected.json");
        if update {
            std::fs::write(&expected_path, &stdout).expect("ghi được snapshot");
            continue;
        }
        match std::fs::read_to_string(&expected_path) {
            Ok(expected) if expected.replace("\r\n", "\n") == stdout => {}
            Ok(expected) => failures.push(format!(
                "{stem}: snapshot khác\n--- mong đợi\n{expected}\n--- thực tế\n{stdout}"
            )),
            Err(_) => failures.push(format!(
                "{stem}: thiếu {}; chạy UPDATE_SNAPSHOTS=1 cargo test -p cli",
                expected_path.display()
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn loi_dung_lenh() {
    assert_eq!(
        dyen(&["replay", "khong-co", "x.json"]).status.code(),
        Some(1)
    );
    assert_eq!(dyen(&["lenh-la"]).status.code(), Some(1));
    let ok = dyen(&["check"]);
    assert!(
        ok.status.success(),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
}
