use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!(
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

fn check_run_compile_verify(rel: &str) {
    let input = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );

    let dir = mk_temp_dir("smc_pcc7_sequence_acceptance");
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
        &format!("smc verify for {input}"),
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc7_sequence_indexing_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_indexing.sm");
}

#[test]
fn pcc7_sequence_iteration_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_iteration.sm");
}

#[test]
fn pcc7_sequence_len_empty_contains_fixture_passes_full_cli_path() {
    check_run_compile_verify(
        "tests/fixtures/pcc7_sequence/positive_sequence_len_empty_contains.sm",
    );
}

#[test]
fn pcc7_sequence_push_prepend_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_push_prepend.sm");
}

#[test]
fn pcc7_sequence_pop_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_pop.sm");
}

#[test]
fn pcc7_sequence_function_boundary_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_function_boundary.sm");
}
