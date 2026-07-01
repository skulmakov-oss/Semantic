use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn smc_output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

#[test]
fn work_prove_source_compiles_and_verifies() {
    let output = smc_output(&[
        "work",
        "examples/semantic_policy_overdrive_trace.sm",
        "prove",
    ]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("verified 'examples/semantic_policy_overdrive_trace.sm'"));
    assert!(stdout.contains("function(s)"));
}

#[test]
fn work_check_accepts_optional_modifiers_without_forwarding_flags() {
    let output = smc_output(&[
        "work",
        "examples/semantic_policy_overdrive_trace.sm",
        "check",
        "to",
        "ignored.smc",
        "with",
        "strict",
    ]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("smc check passed"));
    assert!(!stderr.contains("unknown flag"));
    assert!(!stderr.contains("Unexpected token"));
}

#[test]
fn work_seal_writes_artifact_and_verifies() {
    let target = temp_path("sealed");
    let target_arg = target.to_string_lossy().replace('\\', "/");
    let output = smc_output(&[
        "work",
        "examples/semantic_policy_overdrive_trace.sm",
        "seal",
        "to",
        &target_arg,
        "with",
        "rust",
    ]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(target.exists(), "sealed artifact was not written");

    let verify = smc_output(&["verify", &target_arg]);
    assert!(
        verify.status.success(),
        "verify stdout:\n{}\nverify stderr:\n{}",
        String::from_utf8_lossy(&verify.stdout),
        String::from_utf8_lossy(&verify.stderr)
    );

    let _ = fs::remove_file(&target);
}

fn temp_path(prefix: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "smc_work_layer_guard_{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
