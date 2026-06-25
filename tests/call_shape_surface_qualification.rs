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
        "call_shape_surface_qualification_{}_{}",
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
fn call_shape_surface_qualification_pack_is_covered_by_smoke_and_execution_checks() {
    let positive_cases = [
        "examples/qualification/call_shape_surface/positive_basic_function_call/src/main.sm",
        "examples/qualification/call_shape_surface/positive_nested_function_call/src/main.sm",
        "examples/qualification/call_shape_surface/positive_builtin_call/src/main.sm",
        "examples/qualification/call_shape_surface/positive_named_args_call/src/main.sm",
    ];

    for rel in positive_cases {
        check_compile_verify_run(rel);
    }

    let negative_cases = [
        (
            "examples/qualification/call_shape_surface/negative_unknown_function/src/main.sm",
            "unknown function 'missing_fn'",
        ),
        (
            "examples/qualification/call_shape_surface/negative_wrong_arity_too_few/src/main.sm",
            "function 'add' is missing argument for parameter 'b'",
        ),
        (
            "examples/qualification/call_shape_surface/negative_wrong_arity_too_many/src/main.sm",
            "function 'one' expects 1 args, got 2",
        ),
        (
            "examples/qualification/call_shape_surface/negative_wrong_argument_type/src/main.sm",
            "arg 0 for 'inc' has type Bool, expected I32",
        ),
        (
            "examples/qualification/call_shape_surface/negative_builtin_wrong_type/src/main.sm",
            "builtin 'len' expects a Sequence argument, got I32",
        ),
        (
            "examples/qualification/call_shape_surface/negative_builtin_named_args_rejected/src/main.sm",
            "named arguments are not supported for builtin 'sqrt'",
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
