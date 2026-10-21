//! #2057: the ownership prescan of a lifted lambda body resolves names in
//! the lambda's own environment (its captures and parameter), so a `ref`
//! capture in a `match` on the parameter compiles and runs, and its Borrow
//! lives in the lambda's frame.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sm_format::semcode_decode::{DecodedAccessPathComponent, DecodedBorrowActivation};

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

/// check, compile, verify at O0 and O1; `run-smc` exits 0, or traps with
/// R0002 when `trap`. Returns the O0 and O1 artifact bytes.
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

const LAMBDA: &str = "(o => match o { Option::Some(ref x) => { x } _ => { 0 } })";

fn assert_child_borrow(artifacts: &[Vec<u8>], root: &str) {
    for bytes in artifacts {
        let decoded = sm_format::semcode_decode::decode_semcode(bytes).expect("decode OWN0");
        let main = decoded.functions.iter().find(|f| f.name == "main").unwrap();
        assert!(main.borrowed_paths.is_empty());
        let child = decoded
            .functions
            .iter()
            .find(|f| f.name.starts_with("__closure"))
            .unwrap();
        assert_eq!(child.borrowed_paths.len(), 1);
        let borrow = &child.borrowed_paths[0];
        assert_eq!(child.strings[borrow.root_symbol_id as usize], root);
        assert!(matches!(
            borrow.components.as_slice(),
            [DecodedAccessPathComponent::AdtPayload { index: 0, .. }]
        ));
        assert!(matches!(
            borrow.activation,
            Some(DecodedBorrowActivation::FrameEntry)
        ));
    }
}

/// The #2057 reproduction (it used to pass `check` and fail `compile` with
/// "cannot resolve lexical binding 'o'"), repeated calls, and the
/// equivalent named-function form.
#[test]
fn lambda_ref_capture_on_its_parameter_runs_everywhere() {
    let dir = mk_temp_dir("s2057_issue");
    let source = "fn main() {\n    let f: Closure(Option(i32) -> i32) =\n        (o => match o {\n            Option::Some(ref x) => { x }\n            _ => { 0 }\n        });\n\n    let v: i32 = f(Option::Some(5));\n    assert(v == 5);\n    return;\n}\n";
    let artifacts = pipeline(&dir, "issue_2057", source, false);
    assert_child_borrow(&artifacts, "__sm_local_0_o");
    assert_eq!(
        artifacts,
        pipeline(&dir, "issue_2057_repeat", source, false,),
        "each optimization level is byte-deterministic"
    );
    pipeline(
        &dir,
        "repeated_calls",
        &format!("fn main() {{\n    let f: Closure(Option(i32) -> i32) = {LAMBDA};\n    assert(f(Option::Some(5)) == 5);\n    assert(f(Option::None) == 0);\n    assert(f(Option::Some(8)) == 8);\n    return;\n}}\n"),
        false,
    );
    pipeline(
        &dir,
        "named_function_equivalent",
        "fn pick(o: Option(i32)) -> i32 {\n    let v: i32 = match o { Option::Some(ref x) => { x } _ => { 0 } };\n    return v;\n}\nfn main() {\n    assert(pick(Option::Some(5)) == 5);\n    return;\n}\n",
        false,
    );
}

/// A capture next to the parameter, a lambda-local `let` scrutinized by
/// reference, and a parameter shadowing an outer variable of the same
/// name: the outer variable is a different binding in a different frame, so
/// writing it is not a conflict.
#[test]
fn lambda_bindings_keep_their_lexical_identity() {
    let dir = mk_temp_dir("s2057_identity");
    let captured = pipeline(
        &dir,
        "capture_and_parameter",
        "fn main() {\n    let k: i32 = 10;\n    let f: Closure(Option(i32) -> i32) = (o => match o { Option::Some(ref x) => { x + k } _ => { k } });\n    assert(f(Option::Some(5)) == 15);\n    assert(f(Option::None) == 10);\n    return;\n}\n",
        false,
    );
    assert_child_borrow(&captured, "__sm_local_1_o");
    let local = pipeline(
        &dir,
        "lambda_local_let",
        "fn main() {\n    let f: Closure(Option(i32) -> i32) = (o => { let y: Option(i32) = o; match y { Option::Some(ref x) => { x } _ => { 0 } } });\n    assert(f(Option::Some(5)) == 5);\n    return;\n}\n",
        false,
    );
    assert_child_borrow(&local, "__sm_local_1_y");
    let shadowed = pipeline(
        &dir,
        "parameter_shadows_outer_then_outer_write",
        &format!("fn main() {{\n    let pad: i32 = 42;\n    let mut o: Option(i32) = Option::Some(9);\n    let f: Closure(Option(i32) -> i32) = {LAMBDA};\n    assert(f(Option::Some(5)) == 5);\n    o = Option::None;\n    let _ = o;\n    let _ = pad;\n    return;\n}}\n"),
        false,
    );
    assert_child_borrow(&shadowed, "__sm_local_0_o");
}

/// R0002 is unchanged next to a lambda: a statement-form `ref` in the
/// enclosing frame still traps a later write of its scrutinee, and an
/// unrelated write is still permitted.
#[test]
fn borrow_enforcement_is_unchanged_next_to_a_lambda() {
    let dir = mk_temp_dir("s2057_enforce");
    let prelude = format!("    let mut opt: Option(i32) = Option::Some(7);\n    let mut other: i32 = 1;\n    let f: Closure(Option(i32) -> i32) = {LAMBDA};\n    assert(f(Option::Some(5)) == 5);\n    match opt {{\n        Option::Some(ref x) => {{ assert(x == 7); }}\n        _ => {{}}\n    }}\n");
    pipeline(
        &dir,
        "enclosing_write_traps",
        &format!(
            "fn main() {{\n{prelude}    opt = Option::None;\n    let _ = opt;\n    return;\n}}\n"
        ),
        true,
    );
    pipeline(
        &dir,
        "unrelated_write_allowed",
        &format!("fn main() {{\n{prelude}    other = 2;\n    let _ = other;\n    return;\n}}\n"),
        false,
    );
}

/// An undefined name in a lambda body is still rejected by `check` and
/// `compile`; it is never resolved against an enclosing binding.
#[test]
fn undefined_name_in_a_lambda_body_is_rejected() {
    let dir = mk_temp_dir("s2057_undefined");
    let path = dir.join("undefined.sm");
    std::fs::write(
        &path,
        "fn main() {\n    let missing_outer: Option(i32) = Option::Some(1);\n    let f: Closure(Option(i32) -> i32) = (o => match missing { Option::Some(ref x) => { x } _ => { 0 } });\n    assert(f(Option::Some(5)) == 5);\n    let _ = missing_outer;\n    return;\n}\n",
    )
    .unwrap();
    let path = path.to_str().unwrap();
    let (code, out) = smc(&["check", path]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("missing"), "{out}");
    let art = dir.join("undefined.smc");
    let (code, out) = smc(&["compile", path, "-o", art.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(!art.exists(), "no artifact");
}
