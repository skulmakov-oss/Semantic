#[path = "support/cli_artifact_support.rs"]
mod cli_artifact_support;

use cli_artifact_support::{
    check_source, compile_source_to_artifact, run_source, temp_semcode_artifact, temp_source_file,
    verify_artifact,
};

fn check_run_compile_verify_source(name: &str, source: &str) {
    let source = temp_source_file("pcc4-records", name, source);
    check_source(&source);
    run_source(&source);

    let artifact = temp_semcode_artifact("pcc4-records", "smc_pcc4_records_acceptance");
    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
}

#[test]
fn pcc4_record_declaration_fixture_passes_full_cli_path() {
    check_run_compile_verify_source(
        "pcc4_record_declaration.sm",
        r#"
record SensorReading {
    value: i32,
    active: bool,
}

fn main() {
    return;
}
"#,
    );
}

#[test]
fn pcc4_record_construction_and_field_read_fixture_passes_full_cli_path() {
    check_run_compile_verify_source(
        "pcc4_record_construction_and_field_read.sm",
        r#"
record Pair {
    left: i32,
    right: i32,
}

fn main() {
    let pair: Pair = Pair { left: 4, right: 9 };
    assert(pair.left == 4);
    assert(pair.right == 9);
    return;
}
"#,
    );
}

#[test]
fn pcc4_record_function_boundary_fixture_passes_full_cli_path() {
    check_run_compile_verify_source(
        "pcc4_record_function_boundary.sm",
        r#"
record Sample {
    value: i32,
    enabled: bool,
}

fn sample_value(sample: Sample) -> i32 {
    return sample.value;
}

fn main() {
    let sample: Sample = Sample { value: 7, enabled: true };
    assert(sample_value(sample) == 7);
    return;
}
"#,
    );
}
