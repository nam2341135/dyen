//! `dyen` — công cụ dòng lệnh kiểm tra dự án không cần giao diện.
//!
//! Tầng 3 sẽ thêm `dyen replay <dấu-vết>` và `dyen content check`.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!(
        "dyen {} — chưa có lệnh nào (xem docs/README.md, tầng 3)",
        env!("CARGO_PKG_VERSION")
    );
    ExitCode::SUCCESS
}
