use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// These fixtures use current executable bridge syntax only.
// They do not define canonical Semantic surface vocabulary.
// Canonical vocabulary remains guarded by PCC-3-0 and future #478/#479 work.

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_path(rel: &str) -> String {
    repo_path(&format!("tests/fixtures/pcc3_text/{rel}"))
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

fn run_public_text_pipeline(rel: &str) {
    let input = fixture_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );

    let dir = mk_temp_dir("pcc3_text_core_gate");
    let out = dir.join("out.smc");
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

    let bytes = std::fs::read(&out)
        .unwrap_or_else(|err| panic!("compiled text artifact for {input} was not readable: {err}"));
    assert!(
        !bytes.is_empty(),
        "expected emitted text artifact for {input} to be non-empty"
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

fn assert_stable_text_check_diagnostic(rel: &str, code: &str, needle: &str) {
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

fn assert_invalid_text_source_does_not_verify(rel: &str, code: &str, needle: &str) {
    assert_stable_text_check_diagnostic(rel, code, needle);

    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc3_text_core_gate_invalid");
    let out = dir.join("out.smc");
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
                "expected verify to reject invalid text artifact for {rel}, got: {verify_err}"
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

#[test]
fn pcc3_text_positive_public_pipeline() {
    for rel in [
        "positive_text_literal_binding.sm",
        "positive_text_equality.sm",
        "positive_text_concat.sm",
        "positive_to_text_identity.sm",
        "positive_to_text_scalars.sm",
    ] {
        run_public_text_pipeline(rel);
    }
}

#[test]
fn pcc3_text_assignment_mismatches_have_stable_diagnostics() {
    for (rel, code, needle) in [
        (
            "negative_text_assigned_from_i32.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_text_assigned_from_bool.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_i32_assigned_from_text.sm",
            "E0201",
            "type mismatch in let",
        ),
        (
            "negative_bool_assigned_from_text.sm",
            "E0201",
            "type mismatch in let",
        ),
    ] {
        assert_invalid_text_source_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc3_text_condition_mismatches_have_stable_diagnostics() {
    for (rel, code, needle) in [
        (
            "negative_if_text_condition.sm",
            "E0201",
            "if condition must be bool",
        ),
        (
            "negative_while_text_condition.sm",
            "E0201",
            "while condition must be bool",
        ),
    ] {
        assert_invalid_text_source_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc3_text_concatenation_with_scalar_rejects() {
    assert_invalid_text_source_does_not_verify(
        "negative_text_plus_i32.sm",
        "E0201",
        "text concatenation currently admits only text + text operands",
    );
}
