use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::atomic::{AtomicUsize, Ordering};

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

fn run_check_trace(dir: &Path, input: &str) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    Command::new(exe)
        .arg("check")
        .arg(input)
        .arg("--trace-cache")
        .current_dir(dir)
        .output()
        .expect("run smc check --trace-cache")
}

fn run_hash_smc_trace(dir: &Path, input: &str) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    Command::new(exe)
        .arg("hash-smc")
        .arg(input)
        .arg("--profile")
        .arg("rust")
        .arg("--trace-cache")
        .current_dir(dir)
        .output()
        .expect("run smc hash-smc --trace-cache")
}

fn assert_trace(stderr: &str, event: &str, reason: &str, pack_kind: &str) {
    assert!(
        stderr.contains(&format!("\"event\":\"{}\"", event)),
        "expected event {} in trace, got:\n{}",
        event,
        stderr
    );
    assert!(
        stderr.contains(&format!("\"reason\":\"{}\"", reason)),
        "expected reason {} in trace, got:\n{}",
        reason,
        stderr
    );
    assert!(
        stderr.contains(&format!("\"pack_kind\":\"{}\"", pack_kind)),
        "expected pack_kind {} in trace, got:\n{}",
        pack_kind,
        stderr
    );
}

#[test]
fn semantic_check_reuses_pack_on_unchanged_rerun() {
    let dir = mk_temp_dir("exo_cache_reuse_sem");
    std::fs::write(
        dir.join("root.sm"),
        r#"
Law "Main" [priority 1]:
    When true ->
        System.recovery()
"#,
    )
    .expect("write root");

    let out1 = run_check_trace(&dir, "root.sm");
    assert!(
        out1.status.success(),
        "first check failed: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let out2 = run_check_trace(&dir, "root.sm");
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second check failed: {}", stderr2);
    assert_trace(&stderr2, "cache_hit", "REUSED", "SEMP");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dependency_rebuild_settles_back_to_reuse_on_next_clean_run() {
    let dir = mk_temp_dir("exo_cache_reuse_dep");
    std::fs::write(
        dir.join("root.sm"),
        r#"
Import "dep.sm"
Law "Root" [priority 1]:
    When true ->
        System.recovery()
"#,
    )
    .expect("write root");
    std::fs::write(
        dir.join("dep.sm"),
        r#"
Law "Dep" [priority 1]:
    When true ->
        System.recovery()
"#,
    )
    .expect("write dep");

    let out1 = run_check_trace(&dir, "root.sm");
    assert!(
        out1.status.success(),
        "first check failed: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    std::fs::write(
        dir.join("dep.sm"),
        r#"
Law "Dep" [priority 1]:
    When true ->
        System.recovery()
Law "Dep2" [priority 2]:
    When true ->
        System.recovery()
"#,
    )
    .expect("rewrite dep");

    let out2 = run_check_trace(&dir, "root.sm");
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second check failed: {}", stderr2);
    assert!(
        stderr2.contains("\"reason\":\"DEP_CHANGED\""),
        "expected DEP_CHANGED in trace, got:\n{}",
        stderr2
    );

    let out3 = run_check_trace(&dir, "root.sm");
    let stderr3 = String::from_utf8_lossy(&out3.stderr);
    assert!(out3.status.success(), "third check failed: {}", stderr3);
    assert_trace(&stderr3, "cache_hit", "REUSED", "SEMP");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn hash_smc_reuses_exb_pack_on_unchanged_rerun() {
    let dir = mk_temp_dir("exo_cache_reuse_exb");
    std::fs::write(
        dir.join("root.sm"),
        r#"
fn main() {
    return;
}
"#,
    )
    .expect("write root");

    let out1 = run_hash_smc_trace(&dir, "root.sm");
    assert!(
        out1.status.success(),
        "first hash-smc failed: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let out2 = run_hash_smc_trace(&dir, "root.sm");
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(out2.status.success(), "second hash-smc failed: {}", stderr2);
    assert_trace(&stderr2, "cache_hit", "REUSED", "SMCP");

    let _ = std::fs::remove_dir_all(&dir);
}
