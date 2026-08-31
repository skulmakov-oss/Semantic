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

#[test]
fn match_surface_positive_fixtures_run_end_to_end() {
    let positive_cases = [
        "examples/qualification/match_surface/positive_enum_exhaustive_match/src/main.sm",
        "examples/qualification/match_surface/positive_scalar_literal_match_selection/src/main.sm",
        "examples/qualification/match_surface/positive_option_result_match/src/main.sm",
        "examples/qualification/match_surface/positive_match_guard_bool/src/main.sm",
        "examples/qualification/match_surface/positive_nested_match/src/main.sm",
        "examples/qualification/match_surface/positive_i32_singleton_range_match/src/main.sm",
        "examples/qualification/match_surface/positive_i32_plain_literal_match/src/main.sm",
        "examples/qualification/match_surface/positive_u32_match_full_domain/src/main.sm",
        "examples/qualification/match_surface/positive_int_match_inside_value_producing_loop/src/main.sm",
        "examples/qualification/match_surface/positive_exhaustive_enum_match_inside_value_producing_loop/src/main.sm",
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
        (
            "examples/qualification/match_surface/negative_i32_negative_bound_range_pattern_rejected/src/main.sm",
            "expected match pattern",
        ),
        // or-patterns: SSF-07 rejects `A | B` match arms deterministically at
        // typecheck, before lowering, with one unified diagnostic regardless
        // of scrutinee family or wildcard presence. See
        // build_and_apply_match_plan in crates/sm-front/src/typecheck.rs.
        (
            "examples/qualification/match_surface/negative_quad_or_pattern_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_i32_or_pattern_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_u32_or_pattern_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_enum_or_pattern_no_wildcard_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_enum_or_pattern_with_wildcard_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_option_or_pattern_no_wildcard_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_option_or_pattern_with_wildcard_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_result_or_pattern_no_wildcard_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        (
            "examples/qualification/match_surface/negative_result_or_pattern_with_wildcard_lowering_rejected/src/main.sm",
            "or-pattern match arms ('A | B') are not supported",
        ),
        // FA-02-005 / FA-02-006 (#1637 / #1638): a second default '_' arm
        // must reject deterministically at parse time in both the
        // expression-producing and statement forms of match, rather than
        // silently overwriting the first parsed default arm.
        (
            "examples/qualification/match_surface/negative_duplicate_default_expression_arm_rejected/src/main.sm",
            "match cannot have more than one default '_' arm",
        ),
        (
            "examples/qualification/match_surface/negative_duplicate_default_statement_arm_rejected/src/main.sm",
            "match cannot have more than one default '_' arm",
        ),
    ];

    for (rel, needle) in negative_cases {
        assert_check_rejects(rel, needle);
    }
}

#[test]
fn match_surface_range_lowering_rejection_fixtures_reject_at_compile_phase() {
    let compile_reject_cases = [
        (
            "examples/qualification/match_surface/negative_i32_multivalue_range_lowering_rejected/src/main.sm",
            "integer range match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_i32_exclusive_multivalue_range_lowering_rejected/src/main.sm",
            "integer range match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_i32_exclusive_singleton_range_lowering_rejected/src/main.sm",
            "integer range match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_i32_oversized_singleton_range_lowering_rejected/src/main.sm",
            "integer match pattern literal is outside i32 range",
        ),
        (
            "examples/qualification/match_surface/negative_u32_multivalue_range_lowering_rejected/src/main.sm",
            "integer range match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_u32_exclusive_singleton_range_lowering_rejected/src/main.sm",
            "integer range match pattern lowering is not yet implemented in the IR backend",
        ),
        (
            "examples/qualification/match_surface/negative_u32_oversized_singleton_range_lowering_rejected/src/main.sm",
            "integer match pattern literal is outside u32 range",
        ),
    ];

    for (rel, needle) in compile_reject_cases {
        assert_compile_rejects(rel, needle);
    }
}
