use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("semantic-tests").join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed)
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

    let dir = mk_temp_dir("smc_pcc6_option_acceptance");
    let out = dir.join("out.smc");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("mkdir artifact parent");
    }
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
fn pcc6_option_some_match_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc6_option/positive_option_some_match.sm");
}

#[test]
fn pcc6_option_none_match_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc6_option/positive_option_none_match.sm");
}

#[test]
fn pcc6_option_function_boundary_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc6_option/positive_option_function_boundary.sm");
}
