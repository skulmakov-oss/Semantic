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
