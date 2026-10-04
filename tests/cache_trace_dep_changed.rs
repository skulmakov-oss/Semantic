use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_check(dir: &std::path::Path, input: &str) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    Command::new(exe)
        .arg("check")
        .arg(input)
        .arg("--trace-cache")
        .current_dir(dir)
        .output()
        .expect("run smc check --trace-cache")
}

#[test]
fn trace_cache_reports_dep_changed_on_import_update() {
    // PB-03: test-fixture migration required by restoration of the sealed
    // Model-B legacy-check boundary; no cache/project semantics changed.
    let base = std::env::temp_dir().join(format!(
        "exo_cache_dep_changed_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&base).expect("mkdir");

    let root = base.join("root.sm");
    let dep = base.join("dep.sm");
    std::fs::write(
        &root,
        r#"
Import "dep.sm"

fn main() {
    let value: i32 = score(1);
    assert(value == 1);
    return;
}
"#,
    )
    .expect("write root");
    std::fs::write(
        &dep,
        r#"
fn score(value: i32) -> i32 {
    return value;
}
"#,
    )
    .expect("write dep");

    let out1 = run_check(&base, "root.sm");
    assert!(
        out1.status.success(),
        "first check failed: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let out2 = run_check(&base, "root.sm");
    assert!(
        out2.status.success(),
        "second check failed: {}",
        String::from_utf8_lossy(&out2.stderr)
    );

    std::fs::write(
        &dep,
        r#"
fn score(value: i32) -> i32 {
    return value;
}

fn score_twice(value: i32) -> i32 {
    return value + value;
}
"#,
    )
    .expect("rewrite dep");

    let out3 = run_check(&base, "root.sm");
    let stderr3 = String::from_utf8_lossy(&out3.stderr);
    assert!(out3.status.success(), "third check failed: {stderr3}");
    assert!(
        stderr3.contains("\"reason\":\"DEP_CHANGED\""),
        "expected DEP_CHANGED in trace, got:\n{}",
        stderr3
    );

    let _ = std::fs::remove_dir_all(&base);
}
