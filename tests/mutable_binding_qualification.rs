use std::path::PathBuf;

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

fn check_compile_verify_run(rel: &str) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "mutable_binding_qualification_{}_{}",
        std::process::id(),
        rel.replace(['/', '\\'], "_")
    ));
    std::fs::create_dir_all(&temp_dir).expect("mkdir");
    let out = temp_dir.join("out.smc");
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

#[test]
fn mutable_binding_qualification_pack_is_covered_by_smoke_and_execution_checks() {
    let positive_cases = [
        "examples/qualification/mutable_binding_surface/positive_basic_reassignment/src/main.sm",
        "examples/qualification/mutable_binding_surface/positive_loop_reassignment/src/main.sm",
        "examples/qualification/mutable_binding_surface/positive_text_reassignment/src/main.sm",
        "examples/qualification/mutable_binding_surface/positive_sequence_reassignment/src/main.sm",
        "examples/qualification/mutable_binding_surface/positive_record_reassignment/src/main.sm",
    ];

    for rel in positive_cases {
        check_compile_verify_run(rel);
    }

    let negative_cases = [
        (
            "examples/qualification/mutable_binding_surface/negative_const_reassignment/src/main.sm",
            "cannot assign to const binding 'value'",
        ),
        (
            "examples/qualification/mutable_binding_surface/negative_reassignment_type_mismatch/src/main.sm",
            "type mismatch in assignment to 'value'",
        ),
        (
            "examples/qualification/mutable_binding_surface/negative_unknown_assignment_target/src/main.sm",
            "unknown assignment target 'missing'",
        ),
    ];

    for (rel, needle) in negative_cases {
        let err = cli_err(
            vec!["check".to_string(), repo_path(rel)],
            &format!("smc check for {rel}"),
        );
        assert!(
            err.contains(needle),
            "expected diagnostic '{needle}' for {rel}, got: {err}"
        );
    }
}
