use std::fs;
use std::path::{Path, PathBuf};

fn find_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    for entry in fs::read_dir(dir).expect("read_dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            find_rs_files(&path, files);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn vm_token_first_policy_guard() {
    let mut files = Vec::new();
    find_rs_files(Path::new("src"), &mut files);
    find_rs_files(Path::new("crates"), &mut files);

    let allowed_files = [
        // Owns the compatibility shim definitions
        Path::new("crates")
            .join("sm-vm")
            .join("src")
            .join("semcode_vm.rs"),
        // Owns public compatibility APIs
        Path::new("crates")
            .join("prom-runtime")
            .join("src")
            .join("lib.rs"),
        // Re-exports the APIs
        Path::new("src").join("lib.rs"),
        // Contains inline tests using the shims
        Path::new("crates").join("sm-vm").join("src").join("lib.rs"),
    ];

    let mut violations = Vec::new();

    for file in &files {
        // Skip allowed files
        if allowed_files.iter().any(|allowed| file.ends_with(allowed)) {
            continue;
        }

        let content = fs::read_to_string(file).expect("read_to_string");
        for (line_idx, line) in content.lines().enumerate() {
            if line.contains("run_verified_semcode")
                && !line.contains("run_verified_entry_semcode")
                && !line.trim_start().starts_with("//")
            {
                violations.push(format!(
                    "{}:{}: {}",
                    file.display(),
                    line_idx + 1,
                    line.trim()
                ));
            }
        }
    }

    if !violations.is_empty() {
        let mut msg = String::new();
        msg.push_str("VM Token-First Policy Violation Detected!\n\n");
        msg.push_str("The following production files use `run_verified_semcode*` byte-shims:\n");
        for v in violations {
            msg.push_str(&format!("  {}\n", v));
        }
        msg.push_str("\nPolicy:\n");
        msg.push_str("New production verified execution must use the canonical token path.\n");
        msg.push_str("Instead of `run_verified_semcode(bytes)`, use:\n");
        msg.push_str("  1. verify_semcode_token(bytes)?;\n");
        msg.push_str("  2. token.require_entry(\"main\")?;\n");
        msg.push_str("  3. run_verified_entry_semcode(&entry_token)\n");
        panic!("{}", msg);
    }
}
