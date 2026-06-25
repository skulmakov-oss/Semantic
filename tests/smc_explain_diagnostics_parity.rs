use std::process::{Command, Output};

fn smc_output(args: &[&str]) -> Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    Command::new(exe).args(args).output().expect("run smc")
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_check_rejects_with_code_and_fragment(path: &str, fragment: &str) {
    let output = smc_output(&["check", path]);
    let text = output_text(&output);
    assert!(
        !output.status.success(),
        "expected smc check to fail for {path}, got:\n{text}"
    );
    assert!(
        text.contains("Error [E0201]"),
        "expected E0201 for {path}, got:\n{text}"
    );
    assert!(
        text.contains(fragment),
        "expected diagnostic fragment '{fragment}' for {path}, got:\n{text}"
    );
}

#[test]
fn smc_explain_known_code_returns_help() {
    let output = smc_output(&["explain", "E0201"]);
    let text = output_text(&output);

    assert!(output.status.success(), "smc explain E0201 failed:\n{text}");
    assert!(
        text.contains("E0201:"),
        "expected code prefix in explain output, got:\n{text}"
    );
    assert!(
        text.contains("Type mismatch"),
        "expected stable help fragment in explain output, got:\n{text}"
    );
}

#[test]
fn smc_explain_unknown_code_is_deterministic() {
    let first = smc_output(&["explain", "E9999"]);
    let second = smc_output(&["explain", "E9999"]);

    let first_text = output_text(&first);
    let second_text = output_text(&second);

    assert!(
        !first.status.success(),
        "unknown explain code unexpectedly succeeded:\n{first_text}"
    );
    assert!(
        first_text.contains("unknown diagnostic code 'E9999'"),
        "expected deterministic unknown-code message, got:\n{first_text}"
    );
    assert_eq!(
        first_text, second_text,
        "unknown-code output changed between runs"
    );
}

#[test]
fn smc_check_diagnostics_have_explainable_help_for_common_authoring_errors() {
    let cases = [
        (
            "examples/qualification/practical_surface/negative_bad_record_field/src/main.sm",
            "record literal 'Pair' is missing field 'right'",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_collection_index/src/main.sm",
            "sequence indexing currently requires i32 index",
        ),
        (
            "examples/qualification/mutable_binding_surface/negative_const_reassignment/src/main.sm",
            "cannot assign to const binding 'value'",
        ),
    ];

    for (path, fragment) in cases {
        assert_check_rejects_with_code_and_fragment(path, fragment);
    }

    let explain = smc_output(&["explain", "E0201"]);
    let explain_text = output_text(&explain);
    assert!(
        explain.status.success(),
        "smc explain E0201 failed:\n{explain_text}"
    );
    assert!(
        explain_text.contains("Type mismatch"),
        "expected explain output to map to the shared type-mismatch catalog, got:\n{explain_text}"
    );
}
