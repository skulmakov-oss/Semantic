//! S2043 CP05: the already-parsed and type-checked `if let` expression is
//! executable, and a bare identifier block after an `if` condition or `if let`
//! scrutinee is a block, not a record literal.
//!
//! `if let PAT = SCRUTINEE { THEN } else { ELSE }` lowers exactly as
//! `match SCRUTINEE { PAT => { THEN } _ => { ELSE } }`: the scrutinee is
//! evaluated once, pattern bindings exist only in THEN, ELSE runs on every
//! non-matching value. No new opcode, SemCode revision or VM behavior.

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

/// check, then compile/verify/run at O0 and O1; returns the O0 run stdout.
fn run_everywhere(dir: &Path, name: &str, src: &str) -> String {
    let path = write(dir, name, src);
    let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 0, "{name}: check: {stdout}{stderr}");
    let mut first = None;
    for level in ["O0", "O1"] {
        let out = dir.join(format!("{name}.{level}.smc"));
        let out = out.to_str().unwrap();
        let (code, stdout, stderr) = smc(&["compile", &path, "-o", out, "--opt-level", level]);
        assert_eq!(code, 0, "{name} {level}: compile: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["verify", out]);
        assert_eq!(code, 0, "{name} {level}: verify: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["run-smc", out]);
        assert_eq!(code, 0, "{name} {level}: run: {stdout}{stderr}");
        match &first {
            None => first = Some(stdout),
            Some(o0) => assert_eq!(o0, &stdout, "{name}: O0/O1 output differs"),
        }
    }
    first.unwrap()
}

fn assert_check_rejects(dir: &Path, name: &str, src: &str, fragment: &str) {
    let path = write(dir, name, src);
    let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 1, "{name}: check must reject: {stdout}{stderr}");
    assert!(stdout.contains(fragment), "{name}: {stdout}");
    let out = dir.join(format!("{name}.smc"));
    let (code, stdout, stderr) = smc(&["compile", &path, "-o", out.to_str().unwrap()]);
    assert_eq!(code, 1, "{name}: compile must reject: {stdout}{stderr}");
    assert!(!out.exists(), "{name}: no artifact");
}

#[test]
fn if_let_over_option_result_and_enums_runs_at_o0_and_o1() {
    let dir = mk_temp_dir("s2043_cp05_run");
    run_everywhere(
        &dir,
        "option",
        "fn pick(o: Option(i32)) -> i32 {\n    let v: i32 = if let Option::Some(x) = o {\n        x + 1\n    } else {\n        0\n    };\n    return v;\n}\nfn main() {\n    assert(pick(Option::Some(4)) == 5);\n    let none: Option(i32) = Option::None;\n    assert(pick(none) == 0);\n    return;\n}\n",
    );
    // A `ref` capture records Borrow(AdtPayload) exactly as `match` does
    // and the artifact passes the verifier.
    run_everywhere(
        &dir,
        "ref_capture",
        "fn main() {\n    let opt: Option(i32) = Option::Some(4);\n    let v: i32 = if let Option::Some(ref value) = opt {\n        value\n    } else {\n        0\n    };\n    assert(v == 4);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "result",
        "fn pick(r: Result(i32, i32)) -> i32 {\n    let v: i32 = if let Result::Err(e) = r {\n        e * 10\n    } else {\n        1\n    };\n    return v;\n}\nfn main() {\n    let bad: Result(i32, i32) = Result::Err(7);\n    let good: Result(i32, i32) = Result::Ok(3);\n    assert(pick(bad) == 70);\n    assert(pick(good) == 1);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "enum_payload_and_unit",
        "enum Wrap { Val(i32), Empty }\nfn pick(w: Wrap) -> i32 {\n    let v: i32 = if let Wrap::Val(x) = w {\n        x + 0\n    } else {\n        -1\n    };\n    return v;\n}\nfn holds_nothing(w: Wrap) -> bool {\n    let e: bool = if let Wrap::Empty = w {\n        true\n    } else {\n        false\n    };\n    return e;\n}\nfn main() {\n    assert(pick(Wrap::Val(9)) == 9);\n    assert(pick(Wrap::Empty) == -1);\n    assert(holds_nothing(Wrap::Empty));\n    assert(!holds_nothing(Wrap::Val(1)));\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "nested",
        "fn pick(a: Option(i32), b: Option(i32)) -> i32 {\n    let v: i32 = if let Option::Some(x) = a {\n        if let Option::Some(y) = b {\n            x + y\n        } else {\n            x\n        }\n    } else {\n        0\n    };\n    return v;\n}\nfn main() {\n    let none: Option(i32) = Option::None;\n    assert(pick(Option::Some(1), Option::Some(2)) == 3);\n    assert(pick(Option::Some(1), none) == 1);\n    assert(pick(none, Option::Some(2)) == 0);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "bool_branch_values",
        "fn pick(o: Option(bool)) -> bool {\n    let v: bool = if let Option::Some(flag) = o {\n        flag\n    } else {\n        false\n    };\n    return v;\n}\nfn main() {\n    assert(pick(Option::Some(true)));\n    let none: Option(bool) = Option::None;\n    assert(!pick(none));\n    return;\n}\n",
    );
}

#[test]
fn if_let_pattern_forms_shared_with_match_run() {
    let dir = mk_temp_dir("s2043_cp05_forms");
    run_everywhere(
        &dir,
        "wildcard_is_irrefutable",
        "enum Wrap { Val(i32), Empty }\nfn main() {\n    let w: Wrap = Wrap::Empty;\n    let v: i32 = if let _ = w { 1 } else { 0 };\n    assert(v == 1);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "quad_literal",
        "fn main() {\n    let q: quad = S;\n    let v: i32 = if let S = q { 1 } else { 0 };\n    let w: i32 = if let N = q { 1 } else { 0 };\n    assert(v == 1);\n    assert(w == 0);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "int_literal",
        "fn main() {\n    let n: i32 = 4;\n    let v: i32 = if let 4 = n { 1 } else { 0 };\n    let w: i32 = if let 5 = n { 1 } else { 0 };\n    assert(v == 1);\n    assert(w == 0);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "two_payload_and_wildcard_item",
        "enum Wrap { Pair(i32, i32), Empty }\nfn main() {\n    let w: Wrap = Wrap::Pair(2, 3);\n    let v: i32 = if let Wrap::Pair(a, b) = w { a + b } else { 0 };\n    let x: Wrap = Wrap::Pair(2, 3);\n    let u: i32 = if let Wrap::Pair(_, b) = x { b } else { 0 };\n    assert(v == 5);\n    assert(u == 3);\n    return;\n}\n",
    );
}

/// Codex review (PR #2052, P1): `_` is irrefutable over every type the type
/// checker admits, including scrutinee families `match` itself rejects.
#[test]
fn wildcard_if_let_runs_over_any_scrutinee_type_once() {
    let dir = mk_temp_dir("s2043_cp05_wild_any");
    run_everywhere(
        &dir,
        "wild_any",
        "record P {\n    v: i32,\n}\nfn main() {\n    let a: i32 = if let _ = true { 1 } else { 0 };\n    let pair: (i32, i32) = (1, 2);\n    let b: i32 = if let _ = pair { 2 } else { 0 };\n    let rec: P = P { v: 3 };\n    let c: i32 = if let _ = rec { 3 } else { 0 };\n    let d: i32 = if let _ = \"text\" { 4 } else { 0 };\n    assert(a + b + c + d == 10);\n    return;\n}\n",
    );
    let stdout = run_everywhere(
        &dir,
        "wild_once",
        "fn flag() -> bool {\n    print(\"eval\");\n    return false;\n}\nfn main() {\n    let v: i32 = if let _ = flag() { 7 } else { 0 };\n    assert(v == 7);\n    return;\n}\n",
    );
    assert_eq!(stdout.matches("eval").count(), 1, "{stdout}");
}

#[test]
fn integer_range_if_let_is_rejected_at_lowering_like_match() {
    // Same documented boundary as an integer range `match` arm
    // (foundation_source_profile_v1.md): typecheck-only, rejected
    // deterministically by the IR backend. Pinned so the language catalog
    // can never claim it executable by accident.
    let dir = mk_temp_dir("s2043_cp05_range");
    let path = write(
        &dir,
        "range",
        "fn main() {\n    let n: i32 = 4;\n    let v: i32 = if let 1..=5 = n { 1 } else { 0 };\n    let _ = v;\n    return;\n}\n",
    );
    let (code, _, _) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 0);
    let out = dir.join("range.smc");
    let (code, _, stderr) = smc(&["compile", &path, "-o", out.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("integer range match pattern lowering is not yet implemented"),
        "{stderr}"
    );
}

#[test]
fn scrutinee_is_evaluated_exactly_once() {
    let dir = mk_temp_dir("s2043_cp05_once");
    for (name, arg) in [("once_some", "true"), ("once_none", "false")] {
        let stdout = run_everywhere(
            &dir,
            name,
            &format!(
                "fn make(some: bool) -> Option(i32) {{\n    print(\"eval\");\n    if some {{\n        return Option::Some(3);\n    }} else {{\n        return Option::None;\n    }}\n}}\nfn main() {{\n    let v: i32 = if let Option::Some(x) = make({arg}) {{\n        x\n    }} else {{\n        0\n    }};\n    let _ = v;\n    return;\n}}\n"
            ),
        );
        assert_eq!(stdout.matches("eval").count(), 1, "{name}: {stdout}");
    }
}

#[test]
fn bare_identifier_block_after_a_condition_is_a_block() {
    let dir = mk_temp_dir("s2043_cp05_lookahead");
    // GA-04: `w { x }` / `flag { x }` / `flag {}` used to be taken as a
    // record literal head and fail at the following `else`.
    run_everywhere(
        &dir,
        "if_let_bare_ident",
        "fn pick(w: Option(i32)) -> i32 {\n    let v: i32 = if let Option::Some(x) = w { x } else { 0 };\n    return v;\n}\nfn main() {\n    assert(pick(Option::Some(6)) == 6);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "if_bare_ident",
        "fn pick(flag: bool, x: i32) -> i32 {\n    let v: i32 = if flag { x } else { 0 };\n    return v;\n}\nfn main() {\n    assert(pick(true, 4) == 4);\n    assert(pick(false, 4) == 0);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "if_empty_blocks",
        "fn main() {\n    let flag: bool = true;\n    if flag {} else {}\n    return;\n}\n",
    );
    // Record literals keep working, including shorthand and a record literal
    // compared inside a condition.
    run_everywhere(
        &dir,
        "record_literals_unchanged",
        "record P {\n    v: i32,\n}\nfn main() {\n    let v: i32 = 2;\n    let a: P = P { v };\n    let b: P = P { v: 2 };\n    if a == P { v: 2 } {\n        assert(b.v == 2);\n    } else {\n        assert(false);\n    }\n    return;\n}\n",
    );
}

/// Adversarial review of #2052: the `else` rule is scoped to the `if` head.
/// A shorthand record literal directly before `else` in a `guard` condition
/// or a let-else initializer is still a record literal (valid on base).
#[test]
fn shorthand_record_before_else_outside_if_heads_is_unchanged() {
    let dir = mk_temp_dir("s2043_cp05_scope");
    run_everywhere(
        &dir,
        "guard_shorthand",
        "record P {\n    v: i32,\n}\nfn check(x: P) -> i32 {\n    let v: i32 = 1;\n    guard x == P { v } else return 0;\n    return 1;\n}\nfn main() {\n    assert(check(P { v: 1 }) == 1);\n    assert(check(P { v: 2 }) == 0);\n    return;\n}\n",
    );
    run_everywhere(
        &dir,
        "let_else_shorthand",
        "record P {\n    q: quad,\n}\nfn main() {\n    let q: quad = T;\n    let P { q: T } = P { q } else return;\n    return;\n}\n",
    );
    // A shorthand record literal compared in an `if` head, followed by the
    // then-block (not by `else`), stays a record literal.
    run_everywhere(
        &dir,
        "record_compare_in_if_head",
        "record P {\n    v: i32,\n}\nfn main() {\n    let v: i32 = 1;\n    let x: P = P { v: 1 };\n    let r: i32 = if x == P { v } { 1 } else { 0 };\n    assert(r == 1);\n    return;\n}\n",
    );
    // A block nested in an `if` head is not an `if` head: its let-else
    // initializer stays a record literal, so the program fails exactly as on
    // base (let-else is not a value-block statement), not as a misparse.
    assert_check_rejects(
        &dir,
        "let_else_in_block_in_if_head",
        "record P {\n    q: quad,\n}\nfn main() {\n    let q: quad = T;\n    let c: bool = true;\n    let r: i32 = if if c { let P { q: T } = P { q } else return; true } else { false } { 1 } else { 0 };\n    assert(r == 1);\n    return;\n}\n",
        "value-producing block currently supports only",
    );
}

/// check, compile and verify succeed at O0 and O1, and running traps with
/// the deterministic write/borrow overlap (R0002).
fn run_traps_on_borrow_overlap(dir: &Path, name: &str, src: &str) {
    let path = write(dir, name, src);
    let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 0, "{name}: check: {stdout}{stderr}");
    for level in ["O0", "O1"] {
        let out = dir.join(format!("{name}.{level}.smc"));
        let out = out.to_str().unwrap();
        let (code, stdout, stderr) = smc(&["compile", &path, "-o", out, "--opt-level", level]);
        assert_eq!(code, 0, "{name} {level}: compile: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["verify", out]);
        assert_eq!(code, 0, "{name} {level}: verify: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["run-smc", out]);
        assert_eq!(code, 1, "{name} {level}: run must trap: {stdout}{stderr}");
        assert!(
            format!("{stdout}{stderr}").contains("write path overlaps active borrow"),
            "{name} {level}: {stdout}{stderr}"
        );
    }
}

fn main_with(prelude: &str, expr: &str, expected: i32, epilogue: &str) -> String {
    format!(
        "fn main() {{\n{prelude}    let v: i32 = {expr};\n    assert(v == {expected});\n{epilogue}    return;\n}}\n"
    )
}

const OUTER: &str = "    let outer: Option(Option(i32)) = Option::Some(Option::Some(3));\n";
const NESTED_MATCH: &str = "match outer { Option::Some(inner) => { match inner { Option::Some(ref x) => { x } _ => { 0 } } } _ => { 0 } }";
const NESTED_IF_LET: &str = "if let Option::Some(inner) = outer { match inner { Option::Some(ref x) => { x } _ => { 0 } } } else { 0 }";

/// skulmakov-oss/Semantic#2053: a `ref` capture on a binding of an
/// enclosing arm, in that arm's tail, compiles and runs (it used to pass
/// `check` and fail `compile` with "cannot resolve lexical binding"). The
/// `match` and the equivalent `if let` produce byte-identical SemCode.
#[test]
fn nested_ref_capture_on_an_outer_arm_binding_runs_everywhere() {
    let dir = mk_temp_dir("s2053_tail");
    run_everywhere(&dir, "nested_match", &main_with(OUTER, NESTED_MATCH, 3, ""));
    run_everywhere(
        &dir,
        "nested_if_let",
        &main_with(OUTER, NESTED_IF_LET, 3, ""),
    );
    for level in ["O0", "O1"] {
        let a = std::fs::read(dir.join(format!("nested_match.{level}.smc"))).unwrap();
        let b = std::fs::read(dir.join(format!("nested_if_let.{level}.smc"))).unwrap();
        assert_eq!(a, b, "{level}: if let and its match differ");
    }
    // The same capture in an arm statement (it compiled before #2053).
    run_everywhere(
        &dir,
        "arm_statement",
        &main_with(
            OUTER,
            "match outer { Option::Some(inner) => { let y: i32 = match inner { Option::Some(ref x) => { x } _ => { 0 } }; y } _ => { 0 } }",
            3,
            "",
        ),
    );
    // A tail after the arm block's own `let`, and a plain block tail.
    run_everywhere(
        &dir,
        "arm_block_let_then_tail",
        &main_with(
            OUTER,
            "match outer { Option::Some(inner) => { let y: Option(i32) = inner; match y { Option::Some(ref x) => { x } _ => { 0 } } } _ => { 0 } }",
            3,
            "",
        ),
    );
    run_everywhere(
        &dir,
        "block_let_then_tail",
        &main_with(
            "",
            "{ let inner: Option(i32) = Option::Some(3); match inner { Option::Some(ref x) => { x } _ => { 0 } } }",
            3,
            "",
        ),
    );
}

/// #2053: several nesting levels, non-matching enclosing patterns, match
/// defaults and if-let ELSE paths, and guards that name arm bindings.
#[test]
fn nested_ref_captures_across_levels_defaults_and_guards() {
    let dir = mk_temp_dir("s2053_levels");
    let three =
        "    let o3: Option(Option(Option(i32))) = Option::Some(Option::Some(Option::Some(5)));\n";
    run_everywhere(
        &dir,
        "three_levels_match",
        &main_with(
            three,
            "match o3 { Option::Some(o2) => { match o2 { Option::Some(o1) => { match o1 { Option::Some(ref x) => { x } _ => { 0 } } } _ => { 0 } } } _ => { 0 } }",
            5,
            "",
        ),
    );
    run_everywhere(
        &dir,
        "three_levels_if_let",
        &main_with(
            three,
            "if let Option::Some(o2) = o3 { if let Option::Some(o1) = o2 { match o1 { Option::Some(ref x) => { x } _ => { 0 } } } else { 0 } } else { 0 }",
            5,
            "",
        ),
    );
    // The enclosing pattern does not match: the default / ELSE path runs,
    // and its own nested capture (on an outer-scope value) still works.
    let none = "    let outer: Option(Option(i32)) = Option::None;\n    let other: Option(i32) = Option::Some(8);\n";
    run_everywhere(
        &dir,
        "default_path",
        &main_with(
            none,
            "match outer { Option::Some(inner) => { match inner { Option::Some(ref x) => { x } _ => { 0 } } } _ => { match other { Option::Some(ref y) => { y } _ => { 0 } } } }",
            8,
            "",
        ),
    );
    run_everywhere(
        &dir,
        "else_path",
        &main_with(
            none,
            "if let Option::Some(inner) = outer { match inner { Option::Some(ref x) => { x } _ => { 0 } } } else { match other { Option::Some(ref y) => { y } _ => { 0 } } }",
            8,
            "",
        ),
    );
    // A guard that captures by reference from the arm's own binding, taken
    // and not taken.
    let guard = |n: i32| {
        format!("match outer {{ Option::Some(inner) if match inner {{ Option::Some(ref x) => {{ x == {n} }} _ => {{ false }} }} => {{ 1 }} _ => {{ 0 }} }}")
    };
    run_everywhere(&dir, "guard_taken", &main_with(OUTER, &guard(3), 1, ""));
    run_everywhere(&dir, "guard_not_taken", &main_with(OUTER, &guard(4), 0, ""));
}

/// #2053: the nested borrow is rooted at the arm's own binding, not at an
/// outer variable of the same name, so writing that outer variable is not
/// an overlap; a borrow recorded from a tail still enforces R0002 against a
/// later write of the borrowed path; other ownership events (a site-backed
/// tuple borrow in an arm statement) coexist.
#[test]
fn nested_tail_borrows_keep_their_identity_and_enforcement() {
    let dir = mk_temp_dir("s2053_enforce");
    let shadow = "    let mut inner: Option(i32) = Option::Some(9);\n    let outer: Option(Option(i32)) = Option::Some(Option::Some(3));\n";
    run_everywhere(
        &dir,
        "shadowed_outer_write_allowed",
        &main_with(
            shadow,
            NESTED_MATCH,
            3,
            "    inner = Option::None;\n    let _ = inner;\n",
        ),
    );
    let tail_on_outer = "    let mut flag: Option(i32) = Option::Some(4);\n    let outer: Option(Option(i32)) = Option::Some(Option::Some(3));\n";
    let expr = "match outer { Option::Some(inner) => { let _ = inner; match flag { Option::Some(ref x) => { x } _ => { 0 } } } _ => { 0 } }";
    run_traps_on_borrow_overlap(
        &dir,
        "tail_borrow_then_overlapping_write",
        &main_with(
            tail_on_outer,
            expr,
            4,
            "    flag = Option::None;\n    let _ = flag;\n",
        ),
    );
    // A Borrow recorded from a guard (prescanned after the arm's bindings)
    // is enforced the same way.
    run_traps_on_borrow_overlap(
        &dir,
        "guard_borrow_then_overlapping_write",
        &main_with(
            tail_on_outer,
            "match outer { Option::Some(inner) if match flag { Option::Some(ref x) => { x == 4 } _ => { false } } => { let _ = inner; 4 } _ => { 0 } }",
            4,
            "    flag = Option::None;\n    let _ = flag;\n",
        ),
    );
    run_everywhere(
        &dir,
        "tail_borrow_then_unrelated_write",
        &main_with(
            "    let mut other: i32 = 1;\n    let mut flag: Option(i32) = Option::Some(4);\n    let outer: Option(Option(i32)) = Option::Some(Option::Some(3));\n",
            expr,
            4,
            "    other = 2;\n    let _ = other;\n    let _ = flag;\n",
        ),
    );
    run_everywhere(
        &dir,
        "with_tuple_borrow_in_arm_statement",
        &main_with(
            "    let pair: (i32, i32) = (1, 2);\n    let outer: Option(Option(i32)) = Option::Some(Option::Some(3));\n",
            "match outer { Option::Some(inner) => { let (ref a, _): (i32, i32) = pair; let _ = a; match inner { Option::Some(ref x) => { x } _ => { 0 } } } _ => { 0 } }",
            3,
            "",
        ),
    );
    // Adversarial Review 3: a wildcard `if let`'s never-run ELSE still
    // records its frame-scoped Borrow, so a later write traps (unchanged).
    run_traps_on_borrow_overlap(
        &dir,
        "wildcard_else_borrow_still_enforced",
        &main_with(
            "    let mut opt: Option(i32) = Option::Some(7);\n",
            "if let _ = true { 1 } else { match opt { Option::Some(ref x) => { x } _ => { 0 } } }",
            1,
            "    opt = Option::None;\n    let _ = opt;\n",
        ),
    );
}

#[test]
fn invalid_if_let_forms_fail_closed() {
    let dir = mk_temp_dir("s2043_cp05_reject");
    assert_check_rejects(
        &dir,
        "branch_type_mismatch",
        "fn main() {\n    let o: Option(i32) = Option::Some(1);\n    let v: i32 = if let Option::Some(x) = o {\n        x\n    } else {\n        true\n    };\n    let _ = v;\n    return;\n}\n",
        "if-let branch type mismatch",
    );
    // SSF-08 #1662: the else branch's move of an outer path survives the
    // join, so a later whole use is a use after move.
    assert_check_rejects(
        &dir,
        "else_branch_move_survives_join",
        "enum Wrap { Val(i32) }\nfn main() {\n    let w: Wrap = Wrap::Val(5);\n    let pair: (i32, i32) = (1, 2);\n    let v: i32 = if let Wrap::Val(x) = w {\n        x + 0\n    } else {\n        let (moved, dummy) = pair;\n        let _ = dummy;\n        moved\n    };\n    let _ = v;\n    let (checked, dummy2) = pair;\n    let _ = checked;\n    let _ = dummy2;\n    return;\n}\n",
        "moved",
    );
    assert_check_rejects(
        &dir,
        "binding_escapes_then_branch",
        "fn main() {\n    let o: Option(i32) = Option::Some(1);\n    let v: i32 = if let Option::Some(x) = o {\n        x\n    } else {\n        x\n    };\n    let _ = v;\n    return;\n}\n",
        "unknown variable 'x'",
    );
    // The scrutinee is a call, so the then-block is unambiguous. (With a
    // bare identifier scrutinee and no `else`, `o { x }` stays a record
    // literal head - `P { v };` is valid elsewhere - and the rejection is the
    // parser's deterministic "expected '{'": a known wording limit.)
    assert_check_rejects(
        &dir,
        "missing_else",
        "fn make() -> Option(i32) {\n    return Option::Some(1);\n}\nfn main() {\n    let v: i32 = if let Option::Some(x) = make() {\n        x\n    };\n    let _ = v;\n    return;\n}\n",
        "if-let expression requires explicit else branch",
    );
    assert_check_rejects(
        &dir,
        "missing_else_bare_scrutinee",
        "fn main() {\n    let o: Option(i32) = Option::Some(1);\n    let v: i32 = if let Option::Some(x) = o {\n        x\n    };\n    let _ = v;\n    return;\n}\n",
        "expected '{'",
    );
}
