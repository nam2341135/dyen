//! Công cụ dự án (ADR 0007). Chạy bằng `cargo xtask <lệnh>`.

// Công cụ dòng lệnh: in ra stdout và dừng khi lỗi là hành vi mong muốn.
#![allow(clippy::print_stdout)]

mod deps;

use std::process::{Command, ExitCode};

const HELP: &str = "\
cargo xtask <lệnh>

  ci     fmt --check, clippy -D warnings, test, deps — phải xanh trước khi báo xong
  deps   kiểm tra ranh giới crate (ADR 0006)
  fmt    tự định dạng toàn bộ code
";

fn main() -> ExitCode {
    let cmd = std::env::args().nth(1);
    let ok = match cmd.as_deref() {
        Some("ci") => ci(),
        Some("deps") => deps::check(),
        Some("fmt") => cargo(&["fmt", "--all"]),
        _ => {
            print!("{HELP}");
            return if cmd.is_none() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn ci() -> bool {
    let steps: [(&str, &dyn Fn() -> bool); 4] = [
        ("fmt", &|| cargo(&["fmt", "--all", "--check"])),
        ("clippy", &|| {
            cargo(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ])
        }),
        ("test", &|| cargo(&["test", "--workspace"])),
        ("deps", &deps::check),
    ];
    for (name, step) in steps {
        println!("\n== xtask ci: {name} ==");
        if !step() {
            println!("\n✗ xtask ci THẤT BẠI ở bước: {name}");
            return false;
        }
    }
    println!("\n✓ xtask ci: tất cả các bước đều qua");
    true
}

/// Chạy `cargo <args>` từ gốc workspace, trả về `true` nếu thành công.
pub(crate) fn cargo(args: &[&str]) -> bool {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    Command::new(cargo)
        .args(args)
        .current_dir(workspace_root())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Thư mục gốc workspace (cha của thư mục `xtask`).
pub(crate) fn workspace_root() -> std::path::PathBuf {
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    here.parent().unwrap_or(here).to_path_buf()
}
