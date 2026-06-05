use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_path(rel: &str) -> String {
    repo_path(&format!("tests/fixtures/pcc2_numeric/{rel}"))
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

use std::sync::atomic::{AtomicUsize, Ordering};

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

fn run_public_numeric_pipeline(rel: &str) {
    let input = fixture_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );

    let dir = mk_temp_dir("pcc2_numeric_core_gate");
    let out = dir.join(format!("{rel}.smc"));
    let out_arg = out.to_string_lossy().replace('\\', "/");
    cli_ok(
        vec![
            "compile".to_string(),
            input.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {input}"),
    );

    let bytes = std::fs::read(&out).unwrap_or_else(|err| {
        panic!("compiled numeric artifact for {input} was not readable: {err}")
    });
    assert!(
        !bytes.is_empty(),
        "expected emitted numeric artifact for {input} to be non-empty"
    );

    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn compile_numeric_fixture_to_bytes(rel: &str, out_tag: &str) -> Vec<u8> {
    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc2_numeric_core_gate_bytes");
    let out = dir.join(format!("{rel}_{out_tag}.smc"));
    let out_arg = out.to_string_lossy().replace('\\', "/");
    cli_ok(
        vec![
            "compile".to_string(),
            input.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {input}"),
    );
    let bytes = std::fs::read(&out).unwrap_or_else(|err| {
        panic!("compiled numeric artifact for {input} was not readable: {err}")
    });
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    let _ = std::fs::remove_dir_all(&dir);
    bytes
}

fn assert_stable_numeric_diagnostic(rel: &str, code: &str, needle: &str) {
    let input = fixture_path(rel);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains(&format!("Error [{code}]")),
        "expected diagnostic code {code} for {rel}, got: {err}"
    );
    assert!(
        err.contains(needle),
        "expected diagnostic containing '{needle}' for {rel}, got: {err}"
    );
}

fn assert_invalid_numeric_compile_path_does_not_verify(rel: &str, code: &str, needle: &str) {
    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc2_numeric_core_gate_invalid");
    let out = dir.join(format!("{rel}.smc"));
    let out_arg = out.to_string_lossy().replace('\\', "/");
    let compile_res = smc_cli::run(vec![
        "compile".to_string(),
        input.clone(),
        "-o".to_string(),
        out_arg.clone(),
    ]);
    match compile_res {
        Ok(()) => {
            let bytes = std::fs::read(&out).unwrap_or_else(|err| {
                panic!("compile succeeded but emitted artifact for {input} was not readable: {err}")
            });
            assert!(
                !bytes.is_empty(),
                "compile succeeded but emitted artifact for {input} was empty"
            );
            let verify_err = cli_err(
                vec!["verify".to_string(), out_arg.clone()],
                &format!("smc verify for {out_arg}"),
            );
            assert!(
                verify_err.contains("Error ["),
                "expected verify to reject invalid numeric artifact for {rel}, got: {verify_err}"
            );
        }
        Err(err) => {
            assert!(
                err.contains(&format!("Error [{code}]")) || err.contains(needle),
                "expected compile failure to mention {code} or '{needle}' for {rel}, got: {err}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// PCC-2 numeric gate maps later to:
// - Type Hell
// - Lowering Hell
// - Verifier Hell
// - VM Hell
// - Practical Hell
// - User Pain / Diagnostics Hell

#[test]
fn pcc2_i32_literals_and_arithmetic_positive_pipeline() {
    run_public_numeric_pipeline("positive_i32_literals_and_arithmetic.sm");
}

#[test]
fn pcc2_i32_comparisons_positive_pipeline() {
    run_public_numeric_pipeline("positive_i32_comparisons.sm");
}

#[test]
fn pcc2_numeric_conditions_positive_pipeline() {
    run_public_numeric_pipeline("positive_numeric_conditions.sm");
}

#[test]
fn pcc2_optional_numeric_paths_reflect_current_support() {
    for rel in [
        "positive_u32_basic.sm",
        "positive_f64_basic.sm",
        "positive_fx_basic.sm",
    ] {
        run_public_numeric_pipeline(rel);
    }
}

#[test]
fn pcc2_invalid_numeric_cases_have_stable_diagnostics() {
    let cases = [
        (
            "negative_bool_assigned_from_i32.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_i32_assigned_from_bool.sm",
            "E0201",
            "type mismatch in let",
        ),
        ("negative_i32_plus_bool.sm", "E0201", "I32"),
        (
            "negative_i32_comparison_against_bool.sm",
            "E0201",
            "cannot compare I32 and Bool",
        ),
        (
            "negative_while_numeric_condition.sm",
            "E0201",
            "while condition must be bool",
        ),
        (
            "negative_if_numeric_condition.sm",
            "E0201",
            "if condition must be bool",
        ),
    ];

    for (rel, code, needle) in cases {
        assert_stable_numeric_diagnostic(rel, code, needle);
    }
}

#[test]
fn pcc2_invalid_numeric_compile_path_does_not_verify() {
    let cases = [
        (
            "negative_bool_assigned_from_i32.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_i32_assigned_from_bool.sm",
            "E0201",
            "type mismatch in let",
        ),
        ("negative_i32_plus_bool.sm", "E0201", "I32"),
        (
            "negative_i32_comparison_against_bool.sm",
            "E0201",
            "cannot compare I32 and Bool",
        ),
        (
            "negative_while_numeric_condition.sm",
            "E0201",
            "while condition must be bool",
        ),
        (
            "negative_if_numeric_condition.sm",
            "E0201",
            "if condition must be bool",
        ),
    ];

    for (rel, code, needle) in cases {
        assert_invalid_numeric_compile_path_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc2_numeric_repeated_pipeline_is_byte_stable() {
    let first = compile_numeric_fixture_to_bytes("positive_i32_literals_and_arithmetic.sm", "a");
    let second = compile_numeric_fixture_to_bytes("positive_i32_literals_and_arithmetic.sm", "b");
    assert_eq!(
        first, second,
        "expected repeated compiles of the same numeric source to emit identical .smc bytes"
    );

    let dir = mk_temp_dir("pcc2_numeric_core_gate_repeat_run");
    let out = dir.join("stable.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");
    let input = fixture_path("positive_i32_literals_and_arithmetic.sm");
    cli_ok(
        vec![
            "compile".to_string(),
            input.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {input}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );
    let _ = std::fs::remove_dir_all(&dir);
}
