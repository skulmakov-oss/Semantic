use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// These lowering stability tests use current executable bridge syntax only.
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

fn compile_fixture_to_bytes(rel: &str, out_tag: &str) -> (PathBuf, PathBuf, Vec<u8>) {
    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc3_text_lowering_stability");
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
    let bytes = std::fs::read(&out)
        .unwrap_or_else(|err| panic!("compiled text artifact for {input} was not readable: {err}"));
    (dir, out, bytes)
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
}

fn assert_positive_text_fixture_emits_stable_semcode(rel: &str) {
    run_public_text_pipeline(rel);

    let (dir_a, out_a, bytes_a) = compile_fixture_to_bytes(rel, "first");
    let (dir_b, out_b, bytes_b) = compile_fixture_to_bytes(rel, "second");

    assert!(
        !bytes_a.is_empty(),
        "expected first emitted text artifact for {rel} to be non-empty"
    );
    assert!(
        !bytes_b.is_empty(),
        "expected second emitted text artifact for {rel} to be non-empty"
    );
    assert_eq!(
        bytes_a, bytes_b,
        "emitted SemCode drifted across repeated compiles for {rel}"
    );

    let out_a_arg = out_a.to_string_lossy().replace('\\', "/");
    let out_b_arg = out_b.to_string_lossy().replace('\\', "/");
    cli_ok(
        vec!["verify".to_string(), out_a_arg.clone()],
        &format!("smc verify for {out_a_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_a_arg.clone()],
        &format!("smc run-smc for {out_a_arg}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_b_arg.clone()],
        &format!("smc verify for {out_b_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_b_arg.clone()],
        &format!("smc run-smc for {out_b_arg}"),
    );

    let _ = std::fs::remove_dir_all(dir_a);
    let _ = std::fs::remove_dir_all(dir_b);
}

fn assert_invalid_text_source_does_not_verify(rel: &str) {
    let input = fixture_path(rel);
    let check_err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        check_err.contains("Error ["),
        "expected invalid text source {rel} to fail check, got: {check_err}"
    );

    let dir = mk_temp_dir("pcc3_text_lowering_stability_invalid");
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
                "expected verify to reject invalid text artifact for {rel}, got: {verify_err}"
            );
        }
        Err(err) => {
            assert!(
                err.contains("Error [")
                    || err.contains("type mismatch")
                    || err.contains("if condition must be bool")
                    || err.contains("while condition must be bool")
                    || err.contains("text concatenation currently admits only text + text operands")
                    || err.contains("cannot compare Text and I32")
                    || err.contains("cannot compare Text and Bool")
                    || err.contains("builtin 'to_text' does not yet support record type 'Probe'"),
                "expected compile failure to mention the current text diagnostic surface for {rel}, got: {err}"
            );
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc3_positive_text_fixtures_emit_stable_semcode() {
    for rel in [
        "positive_text_literal_binding.sm",
        "positive_text_equality.sm",
        "positive_text_concat.sm",
        "positive_to_text_identity.sm",
        "positive_to_text_scalars.sm",
    ] {
        assert_positive_text_fixture_emits_stable_semcode(rel);
    }
}

#[test]
fn pcc3_invalid_text_sources_do_not_verify() {
    for rel in [
        "negative_text_assigned_from_i32.sm",
        "negative_text_assigned_from_bool.sm",
        "negative_i32_assigned_from_text.sm",
        "negative_bool_assigned_from_text.sm",
        "negative_if_text_condition.sm",
        "negative_while_text_condition.sm",
        "negative_text_plus_i32.sm",
        "negative_text_assigned_from_quad.sm",
        "negative_quad_assigned_from_text.sm",
        "negative_text_plus_bool.sm",
        "negative_text_plus_quad.sm",
        "negative_text_comparison_against_i32.sm",
        "negative_text_comparison_against_bool.sm",
        "negative_to_text_record.sm",
    ] {
        assert_invalid_text_source_does_not_verify(rel);
    }
}
