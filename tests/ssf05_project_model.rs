use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn project() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "semantic_ssf05_project_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::create_dir_all(root.join("tests/nested")).expect("create tests");
    std::fs::write(
        root.join("semantic.toml"),
        "[package]\nname = \"ssf05\"\n\n[project]\nentry = \"src/main.sm\"\n",
    )
    .expect("write manifest");
    std::fs::write(
        root.join("src/main.sm"),
        "fn main() { assert(true); return; }\n",
    )
    .expect("write main");
    std::fs::write(
        root.join("tests/zeta.sm"),
        "fn main() { assert(true); return; }\n",
    )
    .expect("write zeta test");
    std::fs::write(
        root.join("tests/nested/alpha.sm"),
        "fn main() { assert(true); return; }\n",
    )
    .expect("write alpha test");
    root
}

fn smc(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

#[test]
fn project_verify_and_tests_use_the_canonical_root() {
    let root = project();
    let root_arg = root.to_string_lossy().replace('\\', "/");

    let verify = smc(&["verify", &root_arg]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stdout).contains("header=SEMCODE0"));

    let first = smc(&["test", &root_arg]);
    let second = smc(&["test", &root_arg]);
    assert!(
        first.status.success(),
        "test failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        String::from_utf8_lossy(&first.stdout),
        "ok tests/nested/alpha.sm\nok tests/zeta.sm\ntest result: ok. 2 passed\n"
    );

    std::fs::remove_dir_all(root).expect("remove project");
}

#[test]
fn project_test_fails_explicitly_when_no_tests_exist() {
    let root = project();
    std::fs::remove_dir_all(root.join("tests")).expect("remove tests");
    let root_arg = root.to_string_lossy().replace('\\', "/");

    let output = smc(&["test", &root_arg]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("contains no tests/*.sm programs"));

    std::fs::remove_dir_all(root).expect("remove project");
}

#[test]
fn project_test_propagates_a_deterministic_runtime_failure() {
    let root = project();
    std::fs::write(
        root.join("tests/nested/alpha.sm"),
        "fn main() { assert(false); return; }\n",
    )
    .expect("write failing test");
    let root_arg = root.to_string_lossy().replace('\\', "/");

    let first = smc(&["test", &root_arg]);
    let second = smc(&["test", &root_arg]);
    assert!(!first.status.success());
    assert_eq!(first.stderr, second.stderr);
    assert!(String::from_utf8_lossy(&first.stderr).contains("alpha.sm"));

    std::fs::remove_dir_all(root).expect("remove project");
}

#[test]
fn project_test_rejects_a_non_directory_test_root() {
    let root = project();
    std::fs::remove_dir_all(root.join("tests")).expect("remove tests");
    std::fs::write(root.join("tests"), "not a directory\n").expect("write test-root file");
    let root_arg = root.to_string_lossy().replace('\\', "/");

    let output = smc(&["test", &root_arg]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must be a real directory"));

    std::fs::remove_dir_all(root).expect("remove project");
}
