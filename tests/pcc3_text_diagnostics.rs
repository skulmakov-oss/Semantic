#![allow(dead_code)]
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// These diagnostics use current executable bridge syntax only.
// They do not define canonical Semantic surface vocabulary.
// PCC-3-0 remains authoritative: no Hello World, no print/observe, no canonical fn main/return/assert.

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

fn assert_text_check_error(rel: &str, code: &str, needle: &str) {
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
    assert_text_check_error(rel, code, needle);

    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc3_text_diagnostics_invalid");
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
fn pcc3_text_assignment_mismatch_diagnostics_are_stable() {
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
            "negative_text_assigned_from_quad.sm",
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
        (
            "negative_quad_assigned_from_text.sm",
            "E0201",
            "type mismatch in let",
        ),
    ] {
        assert_invalid_text_source_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc3_text_control_flow_condition_diagnostics_are_stable() {
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
fn pcc3_text_concatenation_boundary_diagnostics_are_stable() {
    for (rel, code, needle) in [
        (
            "negative_text_plus_i32.sm",
            "E0201",
            "text concatenation currently admits only text + text operands",
        ),
        (
            "negative_text_plus_bool.sm",
            "E0201",
            "text concatenation currently admits only text + text operands",
        ),
        (
            "negative_text_plus_quad.sm",
            "E0201",
            "text concatenation currently admits only text + text operands",
        ),
    ] {
        assert_invalid_text_source_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc3_text_comparison_mismatch_diagnostics_are_stable() {
    for (rel, code, needle) in [
        (
            "negative_text_comparison_against_i32.sm",
            "E0201",
            "cannot compare Text and I32",
        ),
        (
            "negative_text_comparison_against_bool.sm",
            "E0201",
            "cannot compare Text and Bool",
        ),
    ] {
        assert_invalid_text_source_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc3_text_to_text_unsupported_argument_rejects_and_does_not_verify() {
    assert_invalid_text_source_does_not_verify(
        "negative_to_text_record.sm",
        "E0201",
        "builtin 'to_text' does not yet support record type 'Probe'",
    );
}
