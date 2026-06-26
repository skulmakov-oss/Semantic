#[path = "support/cli_artifact_support.rs"]
mod cli_artifact_support;

use cli_artifact_support::{source_fixture, temp_semcode_artifact, verify_artifact};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(context)
}

fn compile_source_to_artifact(source: &str, out_path: &std::path::Path, context: &str) {
    cli_ok(
        vec![
            "compile".to_string(),
            source.to_string(),
            "-o".to_string(),
            out_path.to_string_lossy().replace('\\', "/"),
        ],
        context,
    );
}

fn temp_missing_parent_output_path() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "cli_artifact_lifecycle_{}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&root).expect("mkdir root");
    root.join("missing").join("out.smc")
}

fn artifact_bytes(path: &std::path::Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|err| panic!("read artifact {}: {err}", path.display()))
}

fn write_malformed_artifact(path: &std::path::Path) {
    std::fs::write(path, b"not a semcode artifact").expect("write malformed artifact");
}

#[test]
fn smc_compile_creates_and_verifies_explicit_output_artifact() {
    let source = source_fixture(
        "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
    );
    let artifact = temp_semcode_artifact("cli-artifact-lifecycle", "created-out.smc");

    compile_source_to_artifact(
        &source.cli_arg(),
        artifact.path(),
        "smc compile explicit output",
    );

    assert!(
        artifact.path().is_file(),
        "smc compile did not create {}",
        artifact.path().display()
    );
    assert!(
        std::fs::metadata(artifact.path())
            .expect("artifact metadata")
            .len()
            > 0,
        "smc compile produced an empty artifact: {}",
        artifact.path().display()
    );
    verify_artifact(&artifact);
}

#[test]
fn smc_compile_is_deterministic_for_the_same_source() {
    let source = source_fixture(
        "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
    );
    let first = temp_semcode_artifact("cli-artifact-lifecycle", "deterministic-a.smc");
    let second = temp_semcode_artifact("cli-artifact-lifecycle", "deterministic-b.smc");

    compile_source_to_artifact(&source.cli_arg(), first.path(), "smc compile first output");
    compile_source_to_artifact(
        &source.cli_arg(),
        second.path(),
        "smc compile second output",
    );

    let first_bytes = std::fs::read(first.path()).expect("read first artifact");
    let second_bytes = std::fs::read(second.path()).expect("read second artifact");
    assert_eq!(
        first_bytes, second_bytes,
        "smc compile output bytes must be deterministic for the same source"
    );

    verify_artifact(&first);
    verify_artifact(&second);
}

#[test]
fn smc_compile_overwrites_existing_output_deterministically() {
    let source = source_fixture(
        "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
    );
    let clean = temp_semcode_artifact("cli-artifact-lifecycle", "overwrite-clean.smc");
    compile_source_to_artifact(
        &source.cli_arg(),
        clean.path(),
        "smc compile clean baseline",
    );
    let clean_bytes = std::fs::read(clean.path()).expect("read clean artifact");

    let overwrite = temp_semcode_artifact("cli-artifact-lifecycle", "overwrite-target.smc");
    std::fs::write(overwrite.path(), b"sentinel").expect("write sentinel artifact");
    let sentinel_bytes = std::fs::read(overwrite.path()).expect("read sentinel artifact");
    assert_eq!(sentinel_bytes, b"sentinel");

    compile_source_to_artifact(
        &source.cli_arg(),
        overwrite.path(),
        "smc compile overwrite target",
    );

    let current_bytes = std::fs::read(overwrite.path()).expect("read overwritten artifact");
    assert_ne!(
        current_bytes, sentinel_bytes,
        "smc compile should overwrite existing junk output"
    );
    assert_eq!(
        current_bytes, clean_bytes,
        "smc compile overwrite output should match a clean compile for the same source"
    );
    verify_artifact(&overwrite);
}

#[test]
fn smc_compile_rejects_missing_output_parent_path() {
    let source = source_fixture(
        "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
    );
    let out_path = temp_missing_parent_output_path();
    let out_parent = out_path.parent().expect("missing output parent");

    assert!(
        !out_parent.exists(),
        "test precondition violated: parent already exists: {}",
        out_parent.display()
    );

    let err = cli_err(
        vec![
            "compile".to_string(),
            source.cli_arg(),
            "-o".to_string(),
            out_path.to_string_lossy().replace('\\', "/"),
        ],
        "smc compile missing output parent path",
    );
    assert!(
        err.contains("failed to write"),
        "expected compile output write failure, got: {err}"
    );
    assert!(
        !out_path.exists(),
        "compile should not create output when parent path is missing: {}",
        out_path.display()
    );

    let _ = std::fs::remove_dir_all(
        out_path
            .parent()
            .and_then(|p| p.parent())
            .expect("cleanup root"),
    );
}

#[test]
fn smc_verify_leaves_valid_artifact_bytes_unchanged() {
    let source = source_fixture(
        "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
    );
    let artifact = temp_semcode_artifact("cli-artifact-lifecycle", "verify-unchanged.smc");

    compile_source_to_artifact(
        &source.cli_arg(),
        artifact.path(),
        "smc compile verify non-mutation fixture",
    );
    let before = artifact_bytes(artifact.path());
    verify_artifact(&artifact);
    let after = artifact_bytes(artifact.path());

    assert_eq!(
        before, after,
        "smc verify must not mutate valid artifact bytes"
    );
}

#[test]
fn smc_run_smc_leaves_valid_artifact_bytes_unchanged() {
    let source = source_fixture(
        "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
    );
    let artifact = temp_semcode_artifact("cli-artifact-lifecycle", "run-smc-unchanged.smc");

    compile_source_to_artifact(
        &source.cli_arg(),
        artifact.path(),
        "smc compile run-smc non-mutation fixture",
    );
    let before = artifact_bytes(artifact.path());
    cli_ok(
        vec!["run-smc".to_string(), artifact.cli_arg()],
        "smc run-smc valid artifact",
    );
    let after = artifact_bytes(artifact.path());

    assert_eq!(
        before, after,
        "smc run-smc must not mutate valid artifact bytes"
    );
}

#[test]
fn smc_verify_leaves_malformed_artifact_bytes_unchanged_on_failure() {
    let artifact = temp_semcode_artifact("cli-artifact-lifecycle", "verify-malformed.smc");
    write_malformed_artifact(artifact.path());
    let before = artifact_bytes(artifact.path());

    let first_err = cli_err(
        vec!["verify".to_string(), artifact.cli_arg()],
        "smc verify malformed artifact",
    );
    let after_first = artifact_bytes(artifact.path());
    let second_err = cli_err(
        vec!["verify".to_string(), artifact.cli_arg()],
        "smc verify malformed artifact second run",
    );
    let after_second = artifact_bytes(artifact.path());

    assert_eq!(before, after_first, "smc verify mutated malformed artifact");
    assert_eq!(
        after_first, after_second,
        "smc verify mutated malformed artifact on repeated failure"
    );
    assert_eq!(
        first_err, second_err,
        "smc verify failure should be deterministic for malformed artifact"
    );
    assert!(
        !first_err.is_empty(),
        "smc verify malformed artifact should emit a deterministic error"
    );
}

#[test]
fn smc_run_smc_leaves_malformed_artifact_bytes_unchanged_on_failure() {
    let artifact = temp_semcode_artifact("cli-artifact-lifecycle", "run-smc-malformed.smc");
    write_malformed_artifact(artifact.path());
    let before = artifact_bytes(artifact.path());

    let first_err = cli_err(
        vec!["run-smc".to_string(), artifact.cli_arg()],
        "smc run-smc malformed artifact",
    );
    let after_first = artifact_bytes(artifact.path());
    let second_err = cli_err(
        vec!["run-smc".to_string(), artifact.cli_arg()],
        "smc run-smc malformed artifact second run",
    );
    let after_second = artifact_bytes(artifact.path());

    assert_eq!(
        before, after_first,
        "smc run-smc mutated malformed artifact"
    );
    assert_eq!(
        after_first, after_second,
        "smc run-smc mutated malformed artifact on repeated failure"
    );
    assert_eq!(
        first_err, second_err,
        "smc run-smc failure should be deterministic for malformed artifact"
    );
    assert!(
        !first_err.is_empty(),
        "smc run-smc malformed artifact should emit a deterministic error"
    );
}
