//! S2043 CP04: whole-function must-return.
//!
//! `smc check` and `smc compile` agree on whether a non-unit function can
//! fall off its end: a function is accepted only when its last statement
//! returns on every path (`return`, an `if` whose both branches end in such
//! a statement, or a `match` whose every arm and explicit `_` default do).
//! Accepted functions compile at O0 and O1 without dangling join labels,
//! pass the verifier and run; rejected ones fail closed at `check` with the
//! same message `compile` reports. No artificial `Ret` is ever inserted.

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

/// check + compile/verify/run at O0 and O1 must all succeed.
fn assert_executable(dir: &Path, name: &str, src: &str) {
    let path = write(dir, name, src);
    let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 0, "{name}: check must pass: {stdout}{stderr}");
    for level in ["O0", "O1"] {
        let out = dir.join(format!("{name}.{level}.smc"));
        let out = out.to_str().unwrap();
        let (code, stdout, stderr) = smc(&["compile", &path, "-o", out, "--opt-level", level]);
        assert_eq!(code, 0, "{name} {level}: compile: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["verify", out]);
        assert_eq!(code, 0, "{name} {level}: verify: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["run-smc", out]);
        assert_eq!(code, 0, "{name} {level}: run: {stdout}{stderr}");
    }
}

/// check and both compile levels must reject with the same must-return
/// message; no artifact is written.
fn assert_rejected(dir: &Path, name: &str, src: &str, function: &str) {
    let path = write(dir, name, src);
    let expected = format!("function '{function}' may exit without returning I32");
    let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 1, "{name}: check must reject: {stdout}{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(
        report["diagnostics"][0]["code"], "E0201",
        "{name}: {report}"
    );
    assert_eq!(
        report["diagnostics"][0]["message"], expected,
        "{name}: {report}"
    );
    for level in ["O0", "O1"] {
        let out = dir.join(format!("{name}.{level}.smc"));
        let (code, stdout, stderr) = smc(&[
            "compile",
            &path,
            "-o",
            out.to_str().unwrap(),
            "--opt-level",
            level,
        ]);
        assert_eq!(
            code, 1,
            "{name} {level}: compile must reject: {stdout}{stderr}"
        );
        assert!(stderr.contains(&expected), "{name} {level}: {stderr}");
        assert!(!out.exists(), "{name} {level}: no artifact");
    }
}

#[test]
fn branch_complete_returns_are_executable_at_o0_and_o1() {
    let dir = mk_temp_dir("s2043_cp04_ok");
    assert_executable(
        &dir,
        "both_arms",
        "fn pick(flag: bool) -> i32 {\n    if flag {\n        return 1;\n    } else {\n        return 2;\n    }\n}\nfn main() {\n    assert(pick(true) == 1);\n    assert(pick(false) == 2);\n    return;\n}\n",
    );
    assert_executable(
        &dir,
        "nested_if",
        "fn pick(a: bool, b: bool) -> i32 {\n    if a {\n        if b {\n            return 1;\n        } else {\n            return 2;\n        }\n    } else {\n        return 3;\n    }\n}\nfn main() {\n    assert(pick(true, true) == 1);\n    assert(pick(true, false) == 2);\n    assert(pick(false, true) == 3);\n    return;\n}\n",
    );
    assert_executable(
        &dir,
        "else_if_chain",
        "fn pick(x: i32) -> i32 {\n    if x == 0 {\n        return 10;\n    } else if x == 1 {\n        return 11;\n    } else {\n        return 12;\n    }\n}\nfn main() {\n    assert(pick(0) == 10);\n    assert(pick(1) == 11);\n    assert(pick(7) == 12);\n    return;\n}\n",
    );
    assert_executable(
        &dir,
        "quad_match_all",
        "fn pick(q: quad) -> i32 {\n    match q {\n        T => { return 1; }\n        F => { return 2; }\n        _ => { return 3; }\n    }\n}\nfn main() {\n    assert(pick(T) == 1);\n    assert(pick(F) == 2);\n    assert(pick(N) == 3);\n    assert(pick(S) == 3);\n    return;\n}\n",
    );
    assert_executable(
        &dir,
        "match_inside_if",
        "fn pick(flag: bool, q: quad) -> i32 {\n    if flag {\n        match q {\n            T => { return 1; }\n            _ => { return 2; }\n        }\n    } else {\n        return 3;\n    }\n}\nfn main() {\n    assert(pick(true, T) == 1);\n    assert(pick(true, S) == 2);\n    assert(pick(false, N) == 3);\n    return;\n}\n",
    );
    assert_executable(
        &dir,
        "work_before_branch",
        "fn pick(x: i32) -> i32 {\n    let y: i32 = x + 1;\n    if y > 1 {\n        return y;\n    } else {\n        return 0;\n    }\n}\nfn main() {\n    assert(pick(5) == 6);\n    assert(pick(0) == 0);\n    return;\n}\n",
    );
    assert_executable(
        &dir,
        "unit_both_arms",
        "fn touch(flag: bool) {\n    if flag {\n        return;\n    } else {\n        return;\n    }\n}\nfn main() {\n    touch(true);\n    touch(false);\n    return;\n}\n",
    );
}

#[test]
fn possible_fallthrough_is_rejected_by_check_and_compile_alike() {
    let dir = mk_temp_dir("s2043_cp04_reject");
    let main = "fn main() {\n    let r: i32 = pick(true);\n    let _ = r;\n    return;\n}\n";
    for (name, func) in [
        (
            "no_return",
            "fn pick(flag: bool) -> i32 {\n    let y: i32 = 1;\n    let _ = y;\n}\n",
        ),
        (
            "one_arm",
            "fn pick(flag: bool) -> i32 {\n    if flag {\n        return 1;\n    } else {\n        let z: i32 = 0;\n        let _ = z;\n    }\n}\n",
        ),
        (
            "nested_one_arm",
            "fn pick(flag: bool) -> i32 {\n    if flag {\n        if flag {\n            return 1;\n        } else {\n            let z: i32 = 0;\n            let _ = z;\n        }\n    } else {\n        return 2;\n    }\n}\n",
        ),
        (
            "statement_after_return",
            "fn pick(flag: bool) -> i32 {\n    return 1;\n    let z: i32 = 0;\n    let _ = z;\n}\n",
        ),
        (
            "while_may_not_run",
            "fn pick(flag: bool) -> i32 {\n    while flag {\n        return 1;\n    }\n}\n",
        ),
        (
            "guard_then_fallthrough",
            "fn pick(flag: bool) -> i32 {\n    guard flag else return 1;\n}\n",
        ),
        (
            "match_arm_falls_through",
            "fn pick(flag: bool) -> i32 {\n    let q: quad = T;\n    match q {\n        T => { return 1; }\n        _ => { let z: i32 = 0; let _ = z; }\n    }\n}\n",
        ),
    ] {
        assert_rejected(&dir, name, &format!("{func}{main}"), "pick");
    }
}

#[test]
fn branch_complete_returns_emit_no_dangling_join() {
    // The O0 IR of a both-arms-return function ends with a `Ret`; it has no
    // join label and no jump to one.
    let dir = mk_temp_dir("s2043_cp04_ir");
    let path = write(
        &dir,
        "both_arms_ir",
        "fn pick(flag: bool) -> i32 {\n    if flag {\n        return 1;\n    } else {\n        return 2;\n    }\n}\nfn main() {\n    assert(pick(true) == 1);\n    return;\n}\n",
    );
    let (code, stdout, stderr) = smc(&["dump-ir", &path, "--opt-level", "O0"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(!stdout.contains("_end"), "join label emitted: {stdout}");
}
