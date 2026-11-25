//! `dyen` — công cụ dòng lệnh kiểm tra dự án không cần giao diện.
//!
//! ```text
//! dyen check  [--content <thư-mục>]
//! dyen replay <mã-hoạt-động> <dấu-vết.json> [--content <thư-mục>]
//! ```
//!
//! Mã thoát: 0 thành công, 1 lỗi dùng lệnh / đọc file / nội dung, 2 dấu vết không hợp lệ.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use content::{Activity, Library};
use serde::Serialize;

const USAGE: &str = "\
dùng:
  dyen check  [--content <thư-mục>]
  dyen replay <mã-hoạt-động> <dấu-vết.json> [--content <thư-mục>]";

/// Lỗi kèm mã thoát.
struct Fail {
    code: u8,
    message: String,
}

fn fail(code: u8, message: impl Into<String>) -> Fail {
    Fail {
        code,
        message: message.into(),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(out) => {
            let mut stdout = std::io::stdout().lock();
            if writeln!(stdout, "{out}").is_err() {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(f) => {
            eprintln!("{}", f.message);
            ExitCode::from(f.code)
        }
    }
}

/// Tách `--content <dir>` khỏi các đối số vị trí.
fn parse(args: &[String]) -> Result<(Vec<&str>, PathBuf), Fail> {
    let mut positional = Vec::new();
    let mut content = PathBuf::from("content");
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--content" {
            let dir = it
                .next()
                .ok_or_else(|| fail(1, format!("thiếu thư mục sau --content\n{USAGE}")))?;
            content = PathBuf::from(dir);
        } else {
            positional.push(a.as_str());
        }
    }
    Ok((positional, content))
}

fn run(args: &[String]) -> Result<String, Fail> {
    let (pos, content_dir) = parse(args)?;
    match pos.as_slice() {
        ["check"] => {
            let lib = load(&content_dir)?;
            Ok(format!(
                "✓ nội dung hợp lệ: {} nhãn, {} hoạt động, {} gói",
                lib.tags().len(),
                lib.activities().len(),
                lib.sets().len()
            ))
        }
        ["replay", activity, trace] => {
            let lib = load(&content_dir)?;
            let activity = lib
                .activities()
                .iter()
                .find(|(id, _)| id.as_str() == *activity)
                .map(|(_, a)| a)
                .ok_or_else(|| fail(1, format!("không có hoạt động `{activity}`")))?;
            let text = std::fs::read_to_string(trace)
                .map_err(|e| fail(1, format!("không đọc được {trace}: {e}")))?;
            replay_activity(activity, &text)
        }
        _ => Err(fail(1, USAGE)),
    }
}

fn load(dir: &Path) -> Result<Library, Fail> {
    Library::load(dir).map_err(|e| fail(1, e.to_string()))
}

/// Báo cáo phát lại — ổn định để làm snapshot.
#[derive(Serialize)]
struct Report<'a> {
    activity: &'a str,
    #[serde(flatten)]
    run: registry::Run,
}

fn replay_activity(activity: &Activity, trace_json: &str) -> Result<String, Fail> {
    let mut de = serde_json::Deserializer::from_str(trace_json);
    let run = registry::run(&activity.config, &mut de).map_err(|e| fail(2, e.to_string()))?;
    de.end()
        .map_err(|e| fail(2, format!("dấu vết không đọc được: {e}")))?;
    let r = Report {
        activity: activity.id.as_str(),
        run,
    };
    serde_json::to_string_pretty(&r).map_err(|e| fail(1, e.to_string()))
}
