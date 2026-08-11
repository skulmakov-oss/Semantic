#[path = "support/executable_bundle_support.rs"]
mod executable_bundle_support;

fn check_ok(rel: &str) {
    let path = executable_bundle_support::repo_path(rel);
    smc_cli::run(vec!["check".to_string(), path.clone()])
        .unwrap_or_else(|err| panic!("smc check unexpectedly failed for {path}: {err}"));
}

fn check_err(rel: &str) -> String {
    let path = executable_bundle_support::repo_path(rel);
    smc_cli::run(vec!["check".to_string(), path.clone()])
        .expect_err(&format!("smc check unexpectedly passed for {path}"))
}

#[test]
fn g1_frontend_positive_suite_passes() {
    for rel in [
        "examples/qualification/g1_frontend_trust/positive_sequence_and_match/src/main.sm",
        "examples/qualification/g1_frontend_trust/positive_record_iterable/src/main.sm",
        "examples/qualification/g1_frontend_trust/positive_where_clause/src/main.sm",
        "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm",
        "examples/qualification/executable_module_entry/positive_selected_import/src/main.sm",
    ] {
        check_ok(rel);
    }
}

#[test]
fn g1_frontend_negative_suite_reports_expected_diagnostics() {
    let cases = [
        (
            "examples/qualification/g1_frontend_trust/negative_top_level_import/src/main.sm",
            "top-level executable Import admits direct local-path and package-qualified helper modules plus selected local imports",
        ),
        (
            "examples/qualification/g1_frontend_trust/negative_iterable_contract/src/main.sm",
            "fn next(self: Self, index: i32) -> Option(Item)",
        ),
        (
            "examples/qualification/g1_frontend_trust/negative_adt_iterable_scope/src/main.sm",
            "direct record impls only",
        ),
        (
            "examples/qualification/g1_frontend_trust/negative_result_context/src/main.sm",
            "Result::Ok currently requires contextual Result(T, E) type in v0",
        ),
        (
            "examples/qualification/g1_frontend_trust/negative_option_match_exhaustiveness/src/main.sm",
            "non-exhaustive match expression for Option(T); missing variants: None",
        ),
    ];
    for (rel, needle) in cases {
        let err = check_err(rel);
        assert!(
            err.contains(needle),
            "expected diagnostic '{needle}' for {rel}, got: {err}"
        );
    }
}
