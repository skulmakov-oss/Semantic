use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn smc_output(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {}", path.display(), err))
}

fn assert_snapshot(path: &Path, got: &str) {
    let got = normalize(got);
    let expected = normalize(&read_text(path));
    assert_eq!(expected, got, "snapshot mismatch at {}", path.display());
}

fn fixture(rel: &str) -> String {
    repo_path(rel).to_string_lossy().replace('\\', "/")
}

#[test]
fn valid_human_snapshot() {
    let input = fixture("tests/fixtures/7hell_e1/valid_minimal.sm");
    let output = smc_output(&["7hell", &input]);
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_snapshot(
        &repo_path("tests/fixtures/7hell_e1/snapshots/valid_human.txt"),
        &String::from_utf8_lossy(&output.stdout),
    );
}

#[test]
fn valid_json_snapshot() {
    let input = fixture("tests/fixtures/7hell_e1/valid_minimal.sm");
    let output = smc_output(&["seven-hell", &input, "--json"]);
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_snapshot(
        &repo_path("tests/fixtures/7hell_e1/snapshots/valid_json.json"),
        &String::from_utf8_lossy(&output.stdout),
    );
}

#[test]
fn syntax_invalid_json_snapshot() {
    let input = fixture("tests/fixtures/7hell_e1/syntax_invalid.sm");
    let output = smc_output(&["seven-hell", &input, "--json"]);
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_snapshot(
        &repo_path("tests/fixtures/7hell_e1/snapshots/syntax_invalid_json.json"),
        &String::from_utf8_lossy(&output.stdout),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "unexpected stderr: {}", stderr);
}

#[test]
fn type_invalid_json_snapshot() {
    let input = fixture("tests/fixtures/7hell_e1/type_invalid.sm");
    let output = smc_output(&["seven-hell", &input, "--json"]);
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"stage\": \"type\""));
    assert!(stdout.contains("\"kind\": \"check-diagnostic\""));
    assert!(stdout.contains("\"code\": \"E0201\""));
    assert!(!stdout.contains("\"kind\": \"vm-trap\""));
    assert!(!stdout.contains("\"kind\": \"verifier-rejection\""));
    assert!(!stdout.contains("\"kind\": \"project-diagnostic\""));
    assert_snapshot(
        &repo_path("tests/fixtures/7hell_e1/snapshots/type_invalid_json.json"),
        &stdout,
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "unexpected stderr: {}", stderr);
}

#[test]
fn vm_trap_assert_false_json_snapshot() {
    let input = fixture("tests/fixtures/7hell_e1/vm_trap_assert_false.sm");
    let output = smc_output(&["seven-hell", &input, "--json"]);
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {}",
        normalize(&String::from_utf8_lossy(&output.stdout))
    );

    let stdout = normalize(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("\"stage\": \"vm\""));
    assert!(stdout.contains("\"kind\": \"vm-trap\""));
    assert!(stdout.contains("\"code\": \"AssertionFailed\""));
    assert!(stdout.contains("\"result\": \"fail\""));
    assert!(stdout.contains("\"key\": \"practical\""));
    assert!(stdout.contains("\"blocked_by\": \"vm\""));
    assert!(!stdout.contains("\"kind\": \"verifier-rejection\""));
    assert!(!stdout.contains("\"kind\": \"project-diagnostic\""));
    assert!(!stdout.contains("\"result\": \"pass\""));

    assert_snapshot(
        &repo_path("tests/fixtures/7hell_e1/snapshots/vm_trap_assert_false_json.json"),
        &stdout,
    );

    let stderr = normalize(&String::from_utf8_lossy(&output.stderr));
    assert!(stderr.is_empty(), "unexpected stderr: {}", stderr);
}

#[test]
fn project_flag_rejected() {
    let output = smc_output(&["7hell", "--project", "."]);
    assert!(!output.status.success(), "command unexpectedly succeeded");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown flag '--project'"),
        "missing expected rejection needle: {}",
        stderr
    );
}
