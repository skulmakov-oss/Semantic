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

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn assert_stable_numeric_check_diagnostic(rel: &str, code: &str, needle: &str) {
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

fn assert_stable_numeric_check_diagnostics(rel: &str, code: &str, needles: &[&str]) {
    let input = fixture_path(rel);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains(&format!("Error [{code}]")),
        "expected diagnostic code {code} for {rel}, got: {err}"
    );
    for needle in needles {
        assert!(
            err.contains(needle),
            "expected diagnostic containing '{needle}' for {rel}, got: {err}"
        );
    }
}

fn assert_invalid_numeric_compile_path_does_not_verify(rel: &str) {
    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc2_numeric_diagnostics_invalid");
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
                verify_err.contains("Error [") || verify_err.contains("failed"),
                "expected verify to reject invalid numeric artifact for {rel}, got: {verify_err}"
            );
        }
        Err(err) => {
            assert!(
                err.contains("Error [E0201]")
                    || err.contains("type mismatch")
                    || err.contains("fx coercion")
                    || err.contains("operator type mismatch")
                    || err.contains("f64 arithmetic requires f64 operands")
                    || err.contains("cannot compare")
                    || err.contains("while condition must be bool")
                    || err.contains("if condition must be bool"),
                "expected compile failure to mention the numeric diagnostic surface for {rel}, got: {err}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// PCC-2 numeric diagnostics map later to:
// - Type Hell
// - User Pain / Diagnostics Hell
// - Lowering Hell guard, because invalid numeric programs must not emit valid SemCode

#[test]
fn pcc2_numeric_assignment_mismatches_have_stable_diagnostics() {
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
        (
            "negative_i32_assigned_from_f64.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_f64_assigned_from_i32.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_u32_assigned_from_i32.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_fx_assigned_from_f64_expr.sm",
            "E0201",
            "fx coercion from non-literal numeric expressions",
        ),
    ];

    for (rel, code, needle) in cases {
        assert_stable_numeric_check_diagnostic(rel, code, needle);
    }
}

#[test]
fn pcc2_numeric_arithmetic_mismatches_have_stable_diagnostics() {
    let cases = [
        (
            "negative_i32_plus_bool.sm",
            "E0201",
            &["I32", "Bool", "arithmetic"][..],
        ),
        (
            "negative_i32_plus_f64.sm",
            "E0201",
            &["I32", "F64", "arithmetic"][..],
        ),
        (
            "negative_fx_plus_f64.sm",
            "E0201",
            &["Fx", "F64", "arithmetic"][..],
        ),
    ];

    for (rel, code, needles) in cases {
        assert_stable_numeric_check_diagnostics(rel, code, needles);
    }
}

#[test]
fn pcc2_numeric_comparison_mismatches_have_stable_diagnostics() {
    let cases = [
        (
            "negative_i32_comparison_against_bool.sm",
            "E0201",
            "cannot compare I32 and Bool",
        ),
        (
            "negative_i32_comparison_against_f64.sm",
            "E0201",
            "cannot compare I32 and F64",
        ),
        (
            "negative_f64_comparison_against_fx.sm",
            "E0201",
            "cannot compare F64 and Fx",
        ),
    ];

    for (rel, code, needle) in cases {
        assert_stable_numeric_check_diagnostic(rel, code, needle);
    }
}

#[test]
fn pcc2_numeric_condition_mistakes_have_stable_diagnostics() {
    let cases = [
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
        assert_stable_numeric_check_diagnostic(rel, code, needle);
    }
}

#[test]
fn pcc2_invalid_numeric_compile_path_does_not_verify() {
    for rel in [
        "negative_bool_assigned_from_i32.sm",
        "negative_i32_assigned_from_bool.sm",
        "negative_i32_assigned_from_f64.sm",
        "negative_f64_assigned_from_i32.sm",
        "negative_u32_assigned_from_i32.sm",
        "negative_fx_assigned_from_f64_expr.sm",
        "negative_i32_plus_bool.sm",
        "negative_i32_plus_f64.sm",
        "negative_fx_plus_f64.sm",
        "negative_i32_comparison_against_bool.sm",
        "negative_i32_comparison_against_f64.sm",
        "negative_f64_comparison_against_fx.sm",
        "negative_while_numeric_condition.sm",
        "negative_if_numeric_condition.sm",
    ] {
        assert_invalid_numeric_compile_path_does_not_verify(rel);
    }
}
