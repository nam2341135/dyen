//! Nhúng thư mục `content/` vào crate lúc biên dịch: trình duyệt không đọc được file,
//! nên `web` mang sẵn văn bản nguồn và tự dựng `content::Library` lúc chạy.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    println!("cargo:rerun-if-changed={}", root.display());
    for sub in ["activities", "sets"] {
        println!("cargo:rerun-if-changed={}", root.join(sub).display());
    }

    let tags = fs::read_to_string(root.join("tags.toml")).expect("đọc được content/tags.toml");
    let mut out = String::new();
    writeln!(out, "/// Nội dung `tags.toml`.").expect("ghi chuỗi");
    writeln!(out, "pub const TAGS: &str = {tags:?};").expect("ghi chuỗi");
    for (name, sub) in [("ACTIVITIES", "activities"), ("SETS", "sets")] {
        writeln!(
            out,
            "/// Các file `{sub}/*.toml`: (tên không đuôi, nội dung)."
        )
        .expect("ghi chuỗi");
        writeln!(out, "pub const {name}: &[(&str, &str)] = &[").expect("ghi chuỗi");
        for (stem, path) in toml_files(&root.join(sub)) {
            let text = fs::read_to_string(&path).expect("đọc được file nội dung");
            println!("cargo:rerun-if-changed={}", path.display());
            writeln!(out, "    ({stem:?}, {text:?}),").expect("ghi chuỗi");
        }
        writeln!(out, "];").expect("ghi chuỗi");
    }
    let dest =
        PathBuf::from(std::env::var("OUT_DIR").expect("cargo đặt OUT_DIR")).join("content.rs");
    fs::write(dest, out).expect("ghi được content.rs");
}

/// Các file `*.toml` trong `dir`, sắp theo tên để bản build tất định.
fn toml_files(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut files: Vec<(String, PathBuf)> = fs::read_dir(dir)
        .expect("đọc được thư mục nội dung")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .filter_map(|p| Some((p.file_stem()?.to_str()?.to_owned(), p.clone())))
        .collect();
    files.sort();
    files
}
