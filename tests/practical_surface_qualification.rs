use std::path::PathBuf;

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(rel: &str) {
    let path = repo_path(rel);
    smc_cli::run(vec!["check".to_string(), path.clone()])
        .unwrap_or_else(|err| panic!("smc check unexpectedly failed for {path}: {err}"));
}

fn cli_err(rel: &str) -> String {
    let path = repo_path(rel);
    smc_cli::run(vec!["check".to_string(), path.clone()])
        .expect_err(&format!("smc check unexpectedly passed for {path}"))
}

#[test]
fn practical_surface_qualification_pack_is_covered_by_smoke_checks() {
    let positive_cases = [
        "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
        "examples/qualification/practical_surface/positive_records_and_match/src/main.sm",
        "examples/qualification/practical_surface/positive_option_result_flow/src/main.sm",
        "examples/qualification/practical_surface/positive_loops_and_sequences/src/main.sm",
        "examples/qualification/practical_surface/positive_text_and_numbers/src/main.sm",
        "examples/qualification/practical_surface/positive_module_import_surface/src/main.sm",
    ];

    for rel in positive_cases {
        cli_ok(rel);
    }

    let negative_cases = [
        (
            "examples/qualification/practical_surface/negative_bad_record_field/src/main.sm",
            "record literal 'Pair' is missing field 'right'",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_match_shape/src/main.sm",
            "match arm pattern type 'Signal' does not match scrutinee enum 'Direction'",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_collection_index/src/main.sm",
            "sequence indexing currently requires i32 index",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_option_result_use/src/main.sm",
            "Result::Ok currently requires contextual Result(T, E) type in v0",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_numeric_mismatch/src/main.sm",
            "type mismatch in let",
        ),
    ];

    for (rel, needle) in negative_cases {
        let err = cli_err(rel);
        assert!(
            err.contains(needle),
            "expected diagnostic '{needle}' for {rel}, got: {err}"
        );
    }
}
