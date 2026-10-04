use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn mk_temp_dir(prefix: &str) -> std::path::PathBuf {
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

// PB-03: test-fixture migration required by restoration of the sealed
// Model-B legacy-check boundary; no cache/project semantics changed.
fn write_basic_project(dir: &std::path::Path) {
    std::fs::write(
        dir.join("root.sm"),
        r#"
fn main() {
    return;
}
"#,
    )
    .expect("write root");
}

fn write_basic_rust_project(dir: &std::path::Path) {
    std::fs::write(
        dir.join("root.sm"),
        r#"
fn main() {
    return;
}
"#,
    )
    .expect("write root rust");
}

fn run_check_trace(dir: &std::path::Path, extra_env: &[(&str, &str)]) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    let mut cmd = Command::new(exe);
    cmd.arg("check")
        .arg("root.sm")
        .arg("--trace-cache")
        .current_dir(dir);
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.output().expect("run smc")
}

fn run_hash_exb_trace(dir: &std::path::Path, extra_env: &[(&str, &str)]) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    let mut cmd = Command::new(exe);
    cmd.arg("hash-smc")
        .arg("root.sm")
        .arg("--profile")
        .arg("rust")
        .arg("--trace-cache")
        .current_dir(dir);
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.output().expect("run smc hash-smc")
}

fn assert_reason(stderr: &str, reason: &str) {
    assert!(
        stderr.contains(&format!("\"reason\":\"{}\"", reason)),
        "expected reason {} in trace, got:\n{}",
        reason,
        stderr
    );
}

#[test]
fn trace_reason_toolchain_changed() {
    let dir = mk_temp_dir("exo_trace_toolchain");
    write_basic_project(&dir);
    let out1 = run_check_trace(&dir, &[]);
    assert!(out1.status.success(), "first run failed");
    let out2 = run_check_trace(&dir, &[("SM_TOOLCHAIN_HASH", "12345")]);
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second run failed: {}", stderr2);
    assert_reason(&stderr2, "TOOLCHAIN_CHANGED");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn trace_reason_features_changed() {
    let dir = mk_temp_dir("exo_trace_features");
    write_basic_project(&dir);
    let out1 = run_check_trace(&dir, &[]);
    assert!(out1.status.success(), "first run failed");
    let out2 = run_check_trace(&dir, &[("SM_FEATURE_HASH", "777")]);
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second run failed: {}", stderr2);
    assert_reason(&stderr2, "FEATURES_CHANGED");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn trace_reason_schema_changed() {
    let dir = mk_temp_dir("exo_trace_schema");
    write_basic_project(&dir);
    let out1 = run_check_trace(&dir, &[]);
    assert!(out1.status.success(), "first run failed");
    let out2 = run_check_trace(&dir, &[("SM_CACHE_SCHEMA", "99")]);
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second run failed: {}", stderr2);
    assert_reason(&stderr2, "SCHEMA_CHANGED");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn trace_reason_corrupt_pack() {
    let dir = mk_temp_dir("exo_trace_corrupt");
    write_basic_project(&dir);
    let out1 = run_check_trace(&dir, &[]);
    assert!(out1.status.success(), "first run failed");

    let sem_dir = dir.join(".semantic-cache").join("packs").join("sem");
    let sem_file = std::fs::read_dir(&sem_dir)
        .expect("read sem dir")
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("smpack"))
        .expect("smpack exists");
    let mut bytes = std::fs::read(&sem_file).expect("read smpack");
    let idx = bytes.len().saturating_sub(1);
    bytes[idx] ^= 0x01;
    std::fs::write(&sem_file, bytes).expect("corrupt smpack");

    let out2 = run_check_trace(&dir, &[]);
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second run failed: {}", stderr2);
    assert_reason(&stderr2, "CORRUPT_PACK");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn trace_reason_caps_changed_for_exb_pack() {
    let dir = mk_temp_dir("exo_trace_caps");
    write_basic_rust_project(&dir);
    let out1 = run_hash_exb_trace(&dir, &[]);
    assert!(
        out1.status.success(),
        "first hash-smc failed: {}",
        String::from_utf8_lossy(&out1.stderr)
    );
    let out2 = run_hash_exb_trace(&dir, &[("SM_CAPS_HASH", "999")]);
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second hash-smc failed: {}", stderr2);
    assert_reason(&stderr2, "CAPS_CHANGED");
    let _ = std::fs::remove_dir_all(&dir);
}
