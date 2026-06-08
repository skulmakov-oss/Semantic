use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_path(rel: &str) -> String {
    repo_path(&format!("tests/fixtures/pcc8_stdlib_diagnostics/{rel}"))
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

fn assert_stdlib_check_error(rel: &str, code: &str, needle: &str) {
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

fn assert_invalid_stdlib_source_does_not_verify(rel: &str, code: &str, needle: &str) {
    assert_stdlib_check_error(rel, code, needle);

    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc8_stdlib_diagnostics_invalid");
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
                "expected verify to reject invalid stdlib artifact for {rel}, got: {verify_err}"
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

fn assert_stdlib_runtime_trap(rel: &str, needle: &str) {
    let input = fixture_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );

    let dir = mk_temp_dir("pcc8_stdlib_diagnostics_runtime");
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
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );

    let err = cli_err(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );
    assert!(
        err.contains(needle),
        "expected runtime trap containing '{needle}' for {rel}, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_to_text_sequence_rejects() {
    let dir = mk_temp_dir("pcc8_stdlib_diagnostics_to_text_sequence");
    let source_path = dir.join("probe.sm");
    std::fs::write(
        &source_path,
        r#"
fn main() {
    let xs = [1, 2, 3];
    to_text(xs);
    return;
}
"#,
    )
    .expect("write probe source");

    let err = cli_err(
        vec![
            "check".to_string(),
            source_path.to_string_lossy().replace('\\', "/"),
        ],
        "smc check for to_text(sequence) diagnostic",
    );
    assert!(
        err.contains("Error [E0201]"),
        "expected to_text(sequence) diagnostic code E0201, got: {err}"
    );
    assert!(
        err.contains("builtin 'to_text' currently supports"),
        "expected to_text(sequence) diagnostic to mention supported inputs, got: {err}"
    );
    assert!(
        err.contains("got Sequence"),
        "expected to_text(sequence) diagnostic to mention Sequence input, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc8_assert_false_traps_deterministically() {
    assert_stdlib_runtime_trap("negative_assert_false_trap.sm", "assertion failed");
}

#[test]
fn pcc8_print_non_text_rejects_and_does_not_verify() {
    assert_invalid_stdlib_source_does_not_verify(
        "negative_print_non_text.sm",
        "E0201",
        "builtin 'print' expects text",
    );
}

#[test]
fn pcc8_to_text_record_rejects_and_does_not_verify() {
    assert_invalid_stdlib_source_does_not_verify(
        "negative_to_text_record.sm",
        "E0201",
        "builtin 'to_text' does not yet support record type 'Probe'",
    );
}

#[test]
fn pcc8_assert_wrong_arity_rejects_and_does_not_verify() {
    assert_invalid_stdlib_source_does_not_verify(
        "negative_assert_wrong_arity.sm",
        "E0201",
        "assert builtin expects 1 arg",
    );
}

#[test]
fn pcc8_assert_wrong_argument_type_rejects_and_does_not_verify() {
    assert_invalid_stdlib_source_does_not_verify(
        "negative_assert_wrong_argument_type.sm",
        "E0201",
        "assert builtin requires bool condition",
    );
}

#[test]
fn pcc8_to_text_wrong_arity_rejects_and_does_not_verify() {
    assert_invalid_stdlib_source_does_not_verify(
        "negative_to_text_wrong_arity.sm",
        "E0201",
        "builtin 'to_text' takes exactly one positional argument",
    );
}

#[test]
fn pcc8_to_text_sequence_rejects_and_does_not_verify() {
    assert_to_text_sequence_rejects();
}
