use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn write_fixture(dir: &std::path::Path, name: &str, source: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, source).expect("write PCC-4 fixture");
    path.to_string_lossy().replace('\\', "/")
}

fn check_run_compile_verify_source(name: &str, source: &str) {
    let dir = mk_temp_dir("smc_pcc4_records_acceptance");
    let input = write_fixture(&dir, name, source);

    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );

    let out = dir.join("out.smc");
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

fn sample_enabled(sample: Sample) -> bool {
    return sample.enabled;
}

fn main() {
    let sample: Sample = Sample { value: 7, enabled: true };
    assert(sample_value(sample) == 7);
    assert(sample_enabled(sample) == true);
    return;
}
"#,
    );
}
