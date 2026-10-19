//! S2043 CP01: a numeric literal followed by a multibyte UTF-8 scalar used to
//! panic the frontend lexer (`lexer.rs` numeric suffix scan sliced `&str` at
//! byte offsets). Every public entry point must now report the ordinary
//! deterministic E0001 for the offending scalar instead of exiting with a
//! Rust panic.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn smc(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn write_case(dir: &Path, name: &str, literal: &str) -> PathBuf {
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(
        &path,
        format!("fn main() {{\n    let x: i32 = {literal};\n    let _ = x;\n    return;\n}}\n"),
    )
    .expect("write case");
    path
}

/// (case name, literal, offending scalar). The first five panicked on the
/// research baseline 743a3209 at lexer.rs:313/316.
const CASES: &[(&str, &str, &str)] = &[
    ("emoji", "1\u{1f642}", "\u{1f642}"),
    ("euro", "1\u{20ac}", "\u{20ac}"),
    ("u_emoji", "1u\u{1f642}", "\u{1f642}"),
    ("e_acute_twice", "1\u{e9}\u{e9}", "\u{e9}"),
    ("fu_emoji", "1fu\u{1f642}", "\u{1f642}"),
    ("e_acute_semi", "1\u{e9}", "\u{e9}"),
    ("fx_emoji", "1.0fx\u{1f642}", "\u{1f642}"),
    ("u32_emoji", "1u32\u{1f642}", "\u{1f642}"),
    ("i32_euro", "1i32\u{20ac}", "\u{20ac}"),
];

#[test]
fn check_json_reports_e0001_with_exact_range_instead_of_panicking() {
    let dir = mk_temp_dir("s2043_cp01_json");
    for (name, literal, scalar) in CASES {
        let path = write_case(&dir, name, literal);
        let src = std::fs::read_to_string(&path).expect("read case");
        let (code, stdout, stderr) = smc(&["check", path.to_str().unwrap(), "--format", "json"]);
        assert_eq!(code, 1, "{name}: {stdout}{stderr}");
        assert!(!stderr.contains("panicked"), "{name}: {stderr}");
        let report: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("{name}: invalid json ({e}): {stdout}"));
        let diag = &report["diagnostics"][0];
        assert_eq!(diag["code"], "E0001", "{name}: {stdout}");
        let start = diag["range"]["start"]["offset"]
            .as_u64()
            .unwrap_or_else(|| panic!("{name}: no range start: {stdout}"))
            as usize;
        let end = diag["range"]["end"]["offset"].as_u64().unwrap() as usize;
        assert_eq!(&src[start..end], *scalar, "{name}");
    }
}

#[test]
fn legacy_check_and_compile_fail_closed_without_panicking() {
    let dir = mk_temp_dir("s2043_cp01_cli");
    for (name, literal, _) in CASES {
        let path = write_case(&dir, name, literal);
        let p = path.to_str().unwrap();
        let out = dir.join(format!("{name}.smc"));
        for args in [
            vec!["check", p, "--no-cache"],
            vec![
                "compile",
                p,
                "-o",
                out.to_str().unwrap(),
                "--opt-level",
                "O0",
            ],
            vec![
                "compile",
                p,
                "-o",
                out.to_str().unwrap(),
                "--opt-level",
                "O1",
            ],
        ] {
            let (code, stdout, stderr) = smc(&args);
            assert_eq!(code, 1, "{name} {args:?}: {stdout}{stderr}");
            assert!(!stderr.contains("panicked"), "{name} {args:?}: {stderr}");
            assert!(
                stderr.contains("E0001") || stdout.contains("E0001"),
                "{name} {args:?}: {stdout}{stderr}"
            );
            assert!(!out.exists(), "{name}: no artifact on lexical failure");
        }
    }
}
