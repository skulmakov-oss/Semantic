//! #2056: a statement-form `match` whose arm captures an ADT payload by
//! `ref` records the same `Borrow` as the expression form, so a later
//! overlapping write traps (R0002) instead of being silently allowed.

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

fn smc(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc");
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// check, compile and verify succeed at O0 and O1; `run-smc` exits `run`
/// (1 = R0002 trap). Returns the O0 and O1 artifact bytes.
fn pipeline(dir: &Path, name: &str, src: &str, trap: bool) -> Vec<Vec<u8>> {
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(&path, src).unwrap();
    let path = path.to_str().unwrap();
    let (code, out) = smc(&["check", path]);
    assert_eq!(code, 0, "{name}: check: {out}");
    let mut artifacts = Vec::new();
    for level in ["O0", "O1"] {
        let art = dir.join(format!("{name}.{level}.smc"));
        let art = art.to_str().unwrap();
        let (code, out) = smc(&["compile", path, "-o", art, "--opt-level", level]);
        assert_eq!(code, 0, "{name} {level}: compile: {out}");
        let (code, out) = smc(&["verify", art]);
        assert_eq!(code, 0, "{name} {level}: verify: {out}");
        let (code, out) = smc(&["run-smc", art]);
        if trap {
            assert_eq!(code, 1, "{name} {level}: run must trap: {out}");
            assert!(
                out.contains("write path overlaps active borrow"),
                "{name} {level}: {out}"
            );
        } else {
            assert_eq!(code, 0, "{name} {level}: run: {out}");
        }
        artifacts.push(std::fs::read(art).unwrap());
    }
    artifacts
}

fn prog(prelude: &str, body: &str, epilogue: &str) -> String {
    format!("fn main() {{\n{prelude}{body}{epilogue}    return;\n}}\n")
}

const OPT: &str = "    let mut opt: Option(i32) = Option::Some(7);\n";
const STMT_MATCH: &str =
    "    match opt {\n        Option::Some(ref x) => {\n            assert(x == 7);\n        }\n        _ => {}\n    }\n";
const WRITE_OPT: &str = "    opt = Option::None;\n    let _ = opt;\n";

/// The #2056 reproduction now traps exactly like the expression form, and
/// the borrow is not an artefact of the write: without it, both run.
#[test]
fn statement_match_ref_capture_is_enforced_like_the_expression_form() {
    let dir = mk_temp_dir("s2056_basic");
    pipeline(
        &dir,
        "stmt_then_write",
        &prog(OPT, STMT_MATCH, WRITE_OPT),
        true,
    );
    pipeline(
        &dir,
        "expr_then_write",
        &prog(
            OPT,
            "    let v: i32 = match opt { Option::Some(ref x) => { x } _ => { 0 } };\n    assert(v == 7);\n",
            WRITE_OPT,
        ),
        true,
    );
    pipeline(&dir, "stmt_no_write", &prog(OPT, STMT_MATCH, ""), false);
    // A write to an unrelated variable is still permitted.
    pipeline(
        &dir,
        "stmt_unrelated_write",
        &prog(
            &format!("{OPT}    let mut other: i32 = 1;\n"),
            STMT_MATCH,
            "    other = 2;\n    let _ = other;\n    let _ = opt;\n",
        ),
        false,
    );
}

/// Several arms with distinct captures, nested statement matches, a
/// statement match inside `while` and inside a `loop` expression, a guard
/// borrow, and a non-matching arm under the frame-scoped model.
#[test]
fn statement_match_borrows_across_arms_nesting_loops_and_guards() {
    let dir = mk_temp_dir("s2056_shapes");
    let result = "    let mut r: Result(i32, i32) = Result::Err(4);\n";
    pipeline(
        &dir,
        "two_arms_two_captures",
        &prog(
            result,
            "    match r {\n        Result::Ok(ref a) => { assert(a == 0); }\n        Result::Err(ref e) => { assert(e == 4); }\n    }\n",
            "    r = Result::Ok(1);\n    let _ = r;\n",
        ),
        true,
    );
    let nested = "    let mut outer: Option(Option(i32)) = Option::Some(Option::Some(3));\n";
    pipeline(
        &dir,
        "nested_statement_matches",
        &prog(
            nested,
            "    match outer {\n        Option::Some(inner) => {\n            match inner {\n                Option::Some(ref x) => { assert(x == 3); }\n                _ => {}\n            }\n        }\n        _ => {}\n    }\n",
            "",
        ),
        false,
    );
    pipeline(
        &dir,
        "outer_statement_match_ref_then_write",
        &prog(
            nested,
            "    match outer {\n        Option::Some(ref inner) => { let _ = inner; }\n        _ => {}\n    }\n",
            "    outer = Option::None;\n    let _ = outer;\n",
        ),
        true,
    );
    pipeline(
        &dir,
        "statement_match_in_while",
        &prog(
            &format!("{OPT}    let mut i: i32 = 0;\n"),
            &format!(
                "    while i < 2 {{\n    {}        i = i + 1;\n    }}\n",
                STMT_MATCH.replace('\n', "\n    ")
            ),
            WRITE_OPT,
        ),
        true,
    );
    pipeline(
        &dir,
        "statement_match_in_loop_expression",
        &prog(
            OPT,
            "    let n: i32 = loop {\n        match opt {\n            Option::Some(ref x) => { break x; }\n            _ => { break 0; }\n        }\n    };\n    assert(n == 7);\n",
            WRITE_OPT,
        ),
        true,
    );
    pipeline(
        &dir,
        "statement_match_guard_borrow",
        &prog(
            &format!("{OPT}    let mut flag: Option(i32) = Option::Some(1);\n"),
            "    match opt {\n        Option::Some(v) if match flag { Option::Some(ref f) => { f == 1 } _ => { false } } => { assert(v == 7); }\n        _ => {}\n    }\n",
            "    flag = Option::None;\n    let _ = flag;\n",
        ),
        true,
    );
    // The arm is never taken; the frame-scoped Borrow v0 still applies.
    pipeline(
        &dir,
        "non_matching_arm_still_borrows",
        &prog(
            "    let mut opt: Option(i32) = Option::None;\n",
            "    match opt {\n        Option::Some(ref x) => { assert(x == 0); }\n        _ => {}\n    }\n",
            WRITE_OPT,
        ),
        true,
    );
}
