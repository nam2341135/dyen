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
use diagnose::Assessment;
use manip::{Manipulative, ModelSetup, Trace, bars};
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
                "✓ nội dung hợp lệ: {} khái niệm, {} hiểu sai, {} hoạt động, {} gói",
                lib.concepts().len(),
                lib.misconceptions().len(),
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
struct Report<'a, S> {
    activity: &'a str,
    model: &'static str,
    format: u16,
    steps: usize,
    final_state: &'a S,
    assessment: Assessment,
}

/// Chỗ duy nhất rẽ nhánh theo loại mô hình.
fn replay_activity(activity: &Activity, trace_json: &str) -> Result<String, Fail> {
    match &activity.model {
        ModelSetup::Bars(setup) => {
            let trace: Trace<bars::Action> = parse_trace(trace_json)?;
            let run = manip::replay::<bars::Bars>(setup, &trace)
                .map_err(|e| fail(2, format!("dấu vết không hợp lệ: {e}")))?;
            let assessment = diagnose::assess_bars(&activity.task, setup, &trace)
                .map_err(|e| fail(2, format!("không chấm được: {e}")))?;
            report(
                activity,
                bars::Bars::KIND,
                &trace,
                run.final_state(),
                assessment,
            )
        }
    }
}

fn parse_trace<A: serde::de::DeserializeOwned>(json: &str) -> Result<Trace<A>, Fail> {
    serde_json::from_str(json).map_err(|e| fail(2, format!("dấu vết không đọc được: {e}")))
}

fn report<A, S: Serialize>(
    activity: &Activity,
    model: &'static str,
    trace: &Trace<A>,
    final_state: &S,
    assessment: Assessment,
) -> Result<String, Fail> {
    let r = Report {
        activity: activity.id.as_str(),
        model,
        format: trace.format,
        steps: trace.entries.len(),
        final_state,
        assessment,
    };
    serde_json::to_string_pretty(&r).map_err(|e| fail(1, e.to_string()))
}
