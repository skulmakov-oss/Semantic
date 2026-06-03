use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    let dir = std::env::temp_dir()
        .join("semantic-tests")
        .join("pcc5-match")
        .join(format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed),
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

    let dir = mk_temp_dir("smc_pcc5_match_acceptance");
    let out = dir.join("out.smc");
    std::fs::create_dir_all(
        out.parent()
            .expect("out.smc should always have an output directory"),
    )
    .expect("mkdir out parent");
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
        vec!["verify".to_string(), out_arg],
        &format!("smc verify for {input}"),
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc5_match_unit_enum_label_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc5_match/positive_match_unit_enum_label.sm");
}

#[test]
fn pcc5_match_function_boundary_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc5_match/positive_match_function_boundary.sm");
}

#[test]
fn pcc5_match_constructor_from_function_fixture_passes_full_cli_path() {
    check_run_compile_verify(
        "tests/fixtures/pcc5_match/positive_match_constructor_from_function.sm",
    );
}
