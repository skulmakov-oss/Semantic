//! S2043 CP02: diagnostics must never invent a source location.
//!
//! - A diagnostic without producer-proven position is rendered as
//!   "location unavailable" by the legacy `smc check` text, never as a
//!   fabricated `line 1:1` caret; the canonical JSON keeps `range: null`.
//! - A genuine byte offset 0 is a real position and keeps `line 1:1`.
//! - Three narrow type-check families carry a real anchor recorded by the
//!   parser: `let` binding type mismatch (binding name), unknown variable
//!   (identifier) and `if let` pattern/branch failures (`if` keyword). Legacy
//!   text, canonical human and JSON all point at the same token.

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

fn write(dir: &Path, name: &str, src: &str) -> String {
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(&path, src).expect("write case");
    path.to_str().unwrap().to_string()
}

/// 1-based line and byte column of `offset`, the convention the legacy and
/// canonical renderers share (columns count source bytes).
fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let before = &src[..offset];
    let line = before.matches('\n').count() + 1;
    let col = offset - before.rfind('\n').map(|i| i + 1).unwrap_or(0) + 1;
    (line, col)
}

fn json_report(path: &str) -> serde_json::Value {
    let (code, stdout, stderr) = smc(&["check", path, "--format", "json"]);
    assert_eq!(code, 1, "{path}: {stdout}{stderr}");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{path}: invalid json {e}: {stdout}"))
}

struct Known {
    name: &'static str,
    src: &'static str,
    /// The anchor token and which occurrence of it (0-based).
    token: &'static str,
    nth: usize,
    code: &'static str,
}

const KNOWN: &[Known] = &[
    Known {
        name: "let_mismatch",
        src: "fn main() {\n    let x: i32 = 1u32;\n    let _ = x;\n    return;\n}\n",
        token: "x",
        nth: 0,
        code: "E0201",
    },
    Known {
        name: "unknown_variable",
        src: "fn main() {\n    let x: i32 = missing_name;\n    let _ = x;\n    return;\n}\n",
        token: "missing_name",
        nth: 0,
        code: "E0201",
    },
    // Codex review (PR #2052, P2): the const-initializer path reports the
    // same unknown-variable family before type inference runs.
    Known {
        name: "const_unknown_variable",
        src: "fn main() {\n    const X: i32 = missing;\n    let _ = X;\n    return;\n}\n",
        token: "missing",
        nth: 0,
        code: "E0201",
    },
    Known {
        name: "unicode_prefix",
        src: "fn main() {\n    let s: text = \"\u{e9}\u{1f642}\";\n    let _ = s;\n    let y: i32 = missing;\n    let _ = y;\n    return;\n}\n",
        token: "missing",
        nth: 0,
        code: "E0201",
    },
    Known {
        name: "nested_unknown_variable",
        src: "fn main() {\n    if true {\n        let z: i32 = ghost;\n        let _ = z;\n    } else {\n        let w: i32 = 1;\n        let _ = w;\n    }\n    return;\n}\n",
        token: "ghost",
        nth: 0,
        code: "E0201",
    },
    Known {
        name: "if_let_branch_mismatch",
        src: "enum Wrap { Val(i32), Empty }\nfn main() {\n    let w: Wrap = Wrap::Val(5);\n    let v: i32 = if let Wrap::Val(x) = w {\n        x + 0\n    } else {\n        true\n    };\n    let _ = v;\n    return;\n}\n",
        token: "if",
        nth: 0,
        code: "E0201",
    },
    Known {
        name: "if_let_pattern_mismatch",
        src: "enum Wrap { Val(i32), Empty }\nfn main() {\n    let w: Wrap = Wrap::Val(5);\n    let v: i32 = if let Option::Some(x) = w {\n        x + 0\n    } else {\n        0\n    };\n    let _ = v;\n    return;\n}\n",
        token: "if",
        nth: 0,
        code: "E0201",
    },
    Known {
        name: "first_byte_lexical",
        src: "\u{e9}fn main() {\n    return;\n}\n",
        token: "\u{e9}",
        nth: 0,
        code: "E0001",
    },
];

fn anchor_offset(case: &Known) -> usize {
    case.src
        .match_indices(case.token)
        .filter(|(i, _)| {
            // whole-identifier occurrence only
            if !case
                .token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return true;
            }
            let before = case.src[..*i].chars().last();
            let after = case.src[i + case.token.len()..].chars().next();
            !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                && !after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        })
        .nth(case.nth)
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("{}: anchor token not found", case.name))
}

#[test]
fn proven_anchors_agree_across_legacy_human_and_json() {
    let dir = mk_temp_dir("s2043_cp02_known");
    for case in KNOWN {
        let path = write(&dir, case.name, case.src);
        let start = anchor_offset(case);
        let end = start + case.token.len();
        let (line, col) = line_col(case.src, start);

        let report = json_report(&path);
        let diag = &report["diagnostics"][0];
        assert_eq!(diag["code"], case.code, "{}: {report}", case.name);
        assert_eq!(diag["source"], 0, "{}: {report}", case.name);
        assert_eq!(
            diag["range"]["start"]["offset"].as_u64(),
            Some(start as u64),
            "{}: {report}",
            case.name
        );
        assert_eq!(
            diag["range"]["end"]["offset"].as_u64(),
            Some(end as u64),
            "{}: {report}",
            case.name
        );

        let (code, stdout, stderr) = smc(&["check", &path, "--format", "human"]);
        assert_eq!(code, 1, "{}", case.name);
        assert!(
            stdout.contains(&format!(":{line}:{col}\n")),
            "{}: expected {line}:{col} in canonical human: {stdout}{stderr}",
            case.name
        );

        let (code, stdout, stderr) = smc(&["check", &path, "--no-cache", "--color", "never"]);
        assert_eq!(code, 1, "{}", case.name);
        let legacy = format!("{stdout}{stderr}");
        assert!(
            legacy.contains(&format!("at line {line}:{col}")),
            "{}: expected legacy at line {line}:{col}: {legacy}",
            case.name
        );
        assert!(!legacy.contains("location unavailable"), "{}", case.name);
    }
}

#[test]
fn unlocated_diagnostics_never_render_a_fabricated_position() {
    let dir = mk_temp_dir("s2043_cp02_unknown");
    let cases = [
        (
            "unknown_function",
            "fn main() {\n    let x: i32 = missing_fn(1);\n    let _ = x;\n    return;\n}\n",
            "unknown function",
        ),
        (
            "unknown_assignment_target",
            "fn main() {\n    missing += 1;\n    return;\n}\n",
            "missing",
        ),
    ];
    for (name, src, fragment) in cases {
        let path = write(&dir, name, src);
        let report = json_report(&path);
        let diag = &report["diagnostics"][0];
        assert_eq!(diag["code"], "E0201", "{name}: {report}");
        assert!(diag["range"].is_null(), "{name}: {report}");

        let first = smc(&["check", &path, "--no-cache", "--color", "never"]);
        let second = smc(&["check", &path, "--no-cache", "--color", "never"]);
        assert_eq!(first, second, "{name}: legacy output must be deterministic");
        let (code, stdout, stderr) = first;
        assert_eq!(code, 1, "{name}");
        let legacy = format!("{stdout}{stderr}");
        assert!(legacy.contains(fragment), "{name}: {legacy}");
        assert!(legacy.contains("location unavailable"), "{name}: {legacy}");
        assert!(!legacy.contains("at line"), "{name}: {legacy}");
        assert!(
            !legacy.contains('^'),
            "{name}: no caret without a location: {legacy}"
        );
    }
}

#[test]
fn compile_reports_the_proven_byte_offset() {
    let dir = mk_temp_dir("s2043_cp02_compile");
    let case = &KNOWN[1];
    let path = write(&dir, case.name, case.src);
    let out = dir.join("out.smc");
    let (code, stdout, stderr) = smc(&["compile", &path, "-o", out.to_str().unwrap()]);
    assert_eq!(code, 1, "{stdout}{stderr}");
    let start = anchor_offset(case);
    assert!(
        stderr.contains(&format!("at {start}: unknown variable 'missing_name'")),
        "{stderr}"
    );
}
