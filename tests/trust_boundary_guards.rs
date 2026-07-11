use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Helper to run a cargo command and get output
fn cargo_tree_deps(crate_name: &str) -> String {
    let output = Command::new("cargo")
        .args(["tree", "--edges", "normal", "-p", crate_name])
        .output()
        .expect("failed to execute cargo tree");

    assert!(
        output.status.success(),
        "cargo tree failed for {}",
        crate_name
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn prom_cap_dependency_boundaries() {
    let tree = cargo_tree_deps("prom-cap");
    assert!(
        !tree.contains("prom-ui"),
        "prom-cap MUST NOT depend on prom-ui"
    );

    // prom-cap may depend only on prom-abi and prom-refs among prom-* crates.
    for line in tree.lines() {
        if line.contains("prom-") && !line.contains("prom-cap") {
            assert!(
                line.contains("prom-abi") || line.contains("prom-refs"),
                "prom-cap is only allowed to depend on prom-abi and prom-refs among prom-* crates, but found: {}",
                line
            );
        }
    }
}

#[test]
fn sm_vm_dependency_boundaries() {
    let tree = cargo_tree_deps("sm-vm");
    assert!(
        !tree.contains("prom-ui"),
        "sm-vm MUST NOT depend on prom-ui"
    );
    assert!(!tree.contains("sm-ir"), "sm-vm MUST NOT depend on sm-ir");
    assert!(
        !tree.contains("sm-emit"),
        "sm-vm MUST NOT depend on sm-emit in the normal graph"
    );
}

#[test]
fn sm_format_dependency_boundaries() {
    let tree = cargo_tree_deps("sm-format");
    assert!(
        !tree.contains("sm-ir"),
        "sm-format MUST NOT depend on sm-ir"
    );
}

#[test]
fn sm_verify_dependency_boundaries() {
    let tree = cargo_tree_deps("sm-verify");
    assert!(
        !tree.contains("sm-ir"),
        "sm-verify MUST NOT depend on sm-ir"
    );
    assert!(
        !tree.contains("sm-emit"),
        "sm-verify MUST NOT depend on sm-emit in the normal graph"
    );
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }
    for entry in fs::read_dir(dir).expect("read_dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn source_level_no_prom_ui_imports() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("crates/sm-vm"), &mut files);
    collect_rs_files(Path::new("crates/prom-cap"), &mut files);

    let mut violations = Vec::new();

    for file in files {
        let content = String::from_utf8_lossy(&fs::read(&file).expect("read file")).into_owned();
        for (i, line) in content.lines().enumerate() {
            if line.contains("prom_ui") || line.contains("prom-ui") {
                violations.push(format!("{}:{}: {}", file.display(), i + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Source-level trust boundary violation detected. Found 'prom_ui' or 'prom-ui' references in core crates:\n{:#?}", 
        violations
    );
}
