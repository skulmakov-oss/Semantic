use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn project(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "semantic_ssf05_utf8_{label}_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::create_dir_all(root.join("tests")).expect("create tests");
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
    root
}

fn smc(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

// Positive regression: valid UTF-8 test paths execute in normalized lexical order,
// independent of filesystem enumeration order (unaffected by this repair, but the
// contract requires it to keep holding alongside the new UTF-8 rejection).
#[test]
fn valid_utf8_test_paths_execute_in_normalized_lexical_order() {
    let root = project("lexical_order");
    for name in ["zeta.sm", "alpha.sm", "middle.sm"] {
        std::fs::write(
            root.join("tests").join(name),
            "fn main() { assert(true); return; }\n",
        )
        .expect("write test");
    }
    let root_arg = root.to_string_lossy().replace('\\', "/");

    let output = smc(&["test", &root_arg]);
    assert!(
        output.status.success(),
        "test failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ok tests/alpha.sm\nok tests/middle.sm\nok tests/zeta.sm\ntest result: ok. 3 passed\n"
    );

    std::fs::remove_dir_all(root).expect("remove project");
}

#[cfg(unix)]
#[test]
fn non_utf8_test_source_paths_reject_deterministically() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let root = project("non_utf8");
    // Two distinct invalid-UTF-8 byte sequences that both lossy-convert to the same
    // replacement-character string; a lossy sort key would treat them as equal instead
    // of rejecting them, leaving filesystem enumeration order to decide the outcome.
    let invalid_a = OsStr::from_bytes(&[0xffu8, b'a', b'.', b's', b'm']);
    let invalid_b = OsStr::from_bytes(&[0xfeu8, b'b', b'.', b's', b'm']);
    std::fs::write(
        root.join("tests").join(invalid_a),
        "fn main() { assert(true); return; }\n",
    )
    .expect("write non-utf8 test a");
    std::fs::write(
        root.join("tests").join(invalid_b),
        "fn main() { assert(true); return; }\n",
    )
    .expect("write non-utf8 test b");
    let root_arg = root.to_string_lossy().replace('\\', "/");

    let first = smc(&["test", &root_arg]);
    let second = smc(&["test", &root_arg]);
    assert!(
        !first.status.success(),
        "non-UTF-8 test source must be rejected"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "rejection must be deterministic across runs"
    );
    assert!(String::from_utf8_lossy(&first.stderr).contains("not valid UTF-8"));

    std::fs::remove_dir_all(root).expect("remove project");
}

// Verifies the shared project-entry admission authority (admit_package_entry_module,
// used by resolve_project_root_check_entry's callers) already rejects a project entry
// that resolves outside the package module root, across check/compile/run.
#[cfg(unix)]
#[test]
fn project_entry_symlink_escaping_the_module_root_is_rejected_across_check_compile_run() {
    use std::os::unix::fs::symlink;

    let root = project("entry_symlink");
    let outside = std::env::temp_dir().join(format!(
        "semantic_ssf05_entry_symlink_target_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&outside, "fn main() { assert(true); return; }\n").expect("write outside file");

    let entry = root.join("src/main.sm");
    std::fs::remove_file(&entry).expect("remove real entry");
    symlink(&outside, &entry).expect("symlink entry outside module root");
    let root_arg = root.to_string_lossy().replace('\\', "/");

    for command in ["check", "compile", "run"] {
        let args: Vec<&str> = if command == "compile" {
            vec![command, &root_arg, "-o", "/dev/null"]
        } else {
            vec![command, &root_arg]
        };
        let output = smc(&args);
        assert!(
            !output.status.success(),
            "'{command}' must reject a project entry symlinked outside the module root"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("outside admitted package module_root"),
            "'{command}' unexpected stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    std::fs::remove_file(&outside).expect("cleanup outside file");
    std::fs::remove_dir_all(root).expect("remove project");
}
