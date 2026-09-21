//! Regression guard for the byte-sensitive 7hell E1 diagnostic fixtures.
//!
//! The #1698 contract maps the frontend's absolute byte position into the
//! lexer-owned `SourceMark`. A CRLF checkout changes that byte stream and
//! therefore changes the reported column. Keep these fixtures LF-only on
//! every platform.
//!
//! Two checks cover two independent failure modes:
//!
//! 1. Attribute policy, via `git check-attr`, catches removal or narrowing
//!    of the `.gitattributes` rule.
//! 2. Actual working-tree bytes catch an already-affected Windows checkout
//!    that was materialized before the attribute rule existed. An
//!    attribute-only change does not retroactively rewrite an existing file.
//!    Repair from the repository root with:
//!
//!    ```text
//!    rm -rf tests/fixtures/7hell_e1
//!    git checkout -- tests/fixtures/7hell_e1
//!    ```
//!
//!    `git add --renormalize <path>` alone is not sufficient: it updates the
//!    index but does not rewrite the working-tree bytes. This guard does not
//!    normalize source inside the test runner or CLI.

use std::fs;
use std::process::Command;

const FIXTURE_DIR: &str = "tests/fixtures/7hell_e1";

fn tracked_fixture_files() -> Vec<String> {
    let output = Command::new("git")
        .args(["ls-files", "-z", "--", FIXTURE_DIR])
        .output()
        .expect("run git ls-files");
    assert!(output.status.success(), "git ls-files must succeed");
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).replace('\\', "/"))
        .collect()
}

#[test]
fn seven_hell_e1_fixtures_are_attributed_text_eol_lf() {
    let files = tracked_fixture_files();
    assert!(
        !files.is_empty(),
        "expected tracked fixture files under {FIXTURE_DIR}"
    );

    let mut args = vec![
        "check-attr".to_string(),
        "text".to_string(),
        "eol".to_string(),
        "--".to_string(),
    ];
    args.extend(files.iter().cloned());

    let output = Command::new("git")
        .args(&args)
        .output()
        .expect("run git check-attr");
    assert!(output.status.success(), "git check-attr must succeed");
    let report = String::from_utf8_lossy(&output.stdout);

    for file in &files {
        assert!(
            report.contains(&format!("{file}: text: set")),
            "expected '{file}: text: set' in git check-attr output, got:\n{report}"
        );
        assert!(
            report.contains(&format!("{file}: eol: lf")),
            "expected '{file}: eol: lf' in git check-attr output, got:\n{report}"
        );
    }
}

/// Catches an already-affected working copy that the attribute-policy check
/// above cannot see: asserts the bytes actually on disk are LF-only.
#[test]
fn seven_hell_e1_fixture_bytes_are_lf_only() {
    let files = tracked_fixture_files();
    assert!(
        !files.is_empty(),
        "expected tracked fixture files under {FIXTURE_DIR}"
    );

    for file in &files {
        let bytes = fs::read(file).unwrap_or_else(|e| panic!("read {file}: {e}"));
        assert!(
            !bytes.windows(2).any(|w| w == b"\r\n"),
            "{file} contains CRLF line endings in the working tree; \
             `git add --renormalize` alone will NOT fix this (it updates \
             the index, not the working tree). Repair with:\n  \
             rm -rf {FIXTURE_DIR}\n  git checkout -- {FIXTURE_DIR}"
        );
    }
}
