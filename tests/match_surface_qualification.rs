use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
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

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

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

fn run_full_pipeline(rel: &str) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );

    let temp_dir = mk_temp_dir("match_surface_qualification");
    let out = temp_dir.join(
        Path::new(rel)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("out.smc")),
    );
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec![
            "compile".to_string(),
            path.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {path}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

fn assert_check_rejects(rel: &str, needle: &str) {
    let path = repo_path(rel);
    let err = cli_err(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );
    assert!(
        err.contains(needle),
        "expected diagnostic containing '{needle}' for {rel}, got: {err}"
    );
}

fn assert_compile_rejects(rel: &str, needle: &str) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );

    let temp_dir = mk_temp_dir("match_surface_qualification_compile_reject");
    let out = temp_dir.join(
        Path::new(rel)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("out.smc")),
    );
    let out_arg = out.to_string_lossy().replace('\\', "/");

    let err = cli_err(
        vec![
            "compile".to_string(),
            path.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {path}"),
    );
    assert!(
        err.contains(needle),
        "expected lowering diagnostic containing '{needle}' for {rel}, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

fn assert_runtime_traps(rel: &str, needle: &str) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );

    let temp_dir = mk_temp_dir("match_surface_qualification_runtime_trap");
    let out = temp_dir.join(
        Path::new(rel)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("out.smc")),
    );
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec![
            "compile".to_string(),
            path.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {path}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    let err = cli_err(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );
    assert!(
        err.contains(needle),
        "expected runtime trap containing '{needle}' for {rel}, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn match_surface_positive_fixtures_run_end_to_end() {
    let positive_cases = [
        "examples/qualification/match_surface/positive_enum_exhaustive_match/src/main.sm",
        "examples/qualification/match_surface/positive_scalar_literal_match_selection/src/main.sm",
        "examples/qualification/match_surface/positive_option_result_match/src/main.sm",
        "examples/qualification/match_surface/positive_match_guard_bool/src/main.sm",
        "examples/qualification/match_surface/positive_nested_match/src/main.sm",
        "examples/qualification/match_surface/positive_i32_singleton_range_match/src/main.sm",
    ];

    for rel in positive_cases {
        run_full_pipeline(rel);
    }
}

#[test]
fn match_surface_negative_fixtures_reject_deterministically() {
    let negative_cases = [
        (
            "examples/qualification/match_surface/negative_missing_variant/src/main.sm",
            "non-exhaustive match expression for enum 'Direction'; missing variants: Left",
        ),
        (
            "examples/qualification/match_surface/negative_wrong_pattern_family/src/main.sm",
            "match arm pattern type 'Option' does not match scrutinee Result(T, E)",
        ),
        (
            "examples/qualification/match_surface/negative_guard_non_bool/src/main.sm",
            "match guard condition must be bool",
        ),
        (
            "examples/qualification/match_surface/negative_match_result_type_mismatch/src/main.sm",
            "match expression branch type mismatch",
        ),
        (
            "examples/qualification/match_surface/negative_i32_range_pattern_suffixed_bound_rejected/src/main.sm",
            "range pattern bound does not accept a type suffix; use a plain integer",
        ),
    ];

    for (rel, needle) in negative_cases {
        assert_check_rejects(rel, needle);
    }
}

#[test]
fn match_surface_lowering_rejection_fixtures_reject_at_compile_phase() {
    let compile_reject_cases = [
        (
            "examples/qualification/match_surface/negative_quad_or_pattern_lowering_rejected/src/main.sm",
            "wildcard/or/range match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_i32_or_pattern_lowering_rejected/src/main.sm",
            "wildcard/or/quad match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_u32_or_pattern_lowering_rejected/src/main.sm",
            "wildcard/or/quad match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_enum_or_pattern_no_wildcard_lowering_rejected/src/main.sm",
            "non-exhaustive match expression for enum 'Flag'; missing variants: A, B",
        ),
        (
            "examples/qualification/match_surface/negative_enum_or_pattern_with_wildcard_lowering_rejected/src/main.sm",
            "quad match pattern requires quad scrutinee; enum 'Flag' needs explicit variant patterns in lowering",
        ),
        (
            "examples/qualification/match_surface/negative_i32_multivalue_range_lowering_rejected/src/main.sm",
            "integer range match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_i32_oversized_singleton_range_lowering_rejected/src/main.sm",
            "integer match pattern literal is outside i32 range",
        ),
    ];

    for (rel, needle) in compile_reject_cases {
        assert_compile_rejects(rel, needle);
    }
}

#[test]
fn match_surface_u32_match_fixtures_trap_at_runtime() {
    let runtime_trap_cases = [
        (
            "examples/qualification/match_surface/defect_u32_literal_match_runtime_trap/src/main.sm",
            "runtime type mismatch: CmpEq/CmpNe operands must have same runtime type",
        ),
        (
            "examples/qualification/match_surface/defect_u32_range_match_runtime_trap/src/main.sm",
            "runtime type mismatch: CmpEq/CmpNe operands must have same runtime type",
        ),
    ];

    for (rel, needle) in runtime_trap_cases {
        assert_runtime_traps(rel, needle);
    }
}

#[test]
fn match_surface_exclusive_singleton_range_pins_known_miscompilation() {
    // Known defect, not correct behavior: `5..5` is semantically an empty
    // range that should match nothing, but lowering ignores the
    // inclusive/exclusive flag and treats it like `5..=5`. This fixture's
    // own assertions pin the current (wrong) runtime behavior so a fix
    // shows up as a test failure here, not silent drift.
    run_full_pipeline(
        "examples/qualification/match_surface/defect_i32_exclusive_singleton_range_miscompilation/src/main.sm",
    );
}
