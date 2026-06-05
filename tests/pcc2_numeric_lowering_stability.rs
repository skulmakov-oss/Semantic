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
}

fn compile_numeric_fixture_to_bytes(rel: &str, out_tag: &str) -> (PathBuf, Vec<u8>) {
    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc2_numeric_lowering_stability_bytes");
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
    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );
    (dir, bytes)
}

fn assert_positive_numeric_fixture_emits_stable_semcode(rel: &str) {
    run_public_numeric_pipeline(rel);

    let (dir_a, bytes_a) = compile_numeric_fixture_to_bytes(rel, "first");
    let (dir_b, bytes_b) = compile_numeric_fixture_to_bytes(rel, "second");

    assert!(
        !bytes_a.is_empty(),
        "expected first emitted numeric artifact for {rel} to be non-empty"
    );
    assert!(
        !bytes_b.is_empty(),
        "expected second emitted numeric artifact for {rel} to be non-empty"
    );
    assert_eq!(
        bytes_a, bytes_b,
        "emitted SemCode drifted across repeated compiles for {rel}"
    );

    let _ = std::fs::remove_dir_all(dir_a);
    let _ = std::fs::remove_dir_all(dir_b);
}

fn assert_invalid_numeric_source_does_not_verify(rel: &str) {
    let input = fixture_path(rel);
    let check_err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        check_err.contains("Error ["),
        "expected invalid numeric source {rel} to fail check, got: {check_err}"
    );

    let dir = mk_temp_dir("pcc2_numeric_lowering_stability_invalid");
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
                err.contains("Error [")
                    || err.contains("type mismatch")
                    || err.contains("arithmetic")
                    || err.contains("compare")
                    || err.contains("bool")
                    || err.contains("fx coercion"),
                "expected compile failure to mention the numeric diagnostic surface for {rel}, got: {err}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// PCC-2 numeric lowering stability maps later to:
// - Lowering Hell
// - Verifier Hell
// - VM Hell
// - Practical Hell

#[test]
fn pcc2_positive_numeric_fixtures_emit_stable_semcode() {
    for rel in [
        "positive_i32_literals_and_arithmetic.sm",
        "positive_i32_comparisons.sm",
        "positive_numeric_conditions.sm",
        "positive_u32_basic.sm",
        "positive_f64_basic.sm",
        "positive_fx_basic.sm",
    ] {
        assert_positive_numeric_fixture_emits_stable_semcode(rel);
    }
}

#[test]
fn pcc2_invalid_numeric_sources_do_not_verify() {
    for rel in [
        "negative_bool_assigned_from_i32.sm",
        "negative_i32_assigned_from_bool.sm",
        "negative_i32_plus_bool.sm",
        "negative_i32_comparison_against_bool.sm",
        "negative_if_numeric_condition.sm",
        "negative_while_numeric_condition.sm",
        "negative_i32_assigned_from_f64.sm",
        "negative_f64_assigned_from_i32.sm",
        "negative_u32_assigned_from_i32.sm",
        "negative_fx_assigned_from_f64_expr.sm",
        "negative_i32_plus_f64.sm",
        "negative_fx_plus_f64.sm",
        "negative_i32_comparison_against_f64.sm",
        "negative_f64_comparison_against_fx.sm",
    ] {
        assert_invalid_numeric_source_does_not_verify(rel);
    }
}
