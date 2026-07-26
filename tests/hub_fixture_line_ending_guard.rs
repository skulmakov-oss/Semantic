//! Regression guard for the strict canonical Semantic Hub request fixtures
//! (Issue #1553), mirroring `tests/ui_frame_fixture_line_ending_guard.rs`
//! (Issue #1365 / PR #1552) exactly -- do not repeat the mistake that PR
//! fixed.
//!
//! These fixtures are read verbatim as request-file bytes by
//! `smc hub invoke --input <fixture>`, which parses them with `serde_json`.
//! `serde_json` itself tolerates either line ending, so this guard is not
//! about decoder strictness the way the UI frame fixtures are -- it exists
//! so this fixture directory never silently regresses to CRLF on a Windows
//! checkout with `core.autocrlf=true`, which would make every fixture byte
//! count and content differ from what is actually committed and reviewed,
//! and would make `tests/fixtures/hub/reject_malformed_truncated.json`
//! (whose entire point is an exact truncation point) no longer test the
//! exact truncation being reviewed in the PR diff.
//!
//! Two checks, for two different failure modes (see the UI frame guard's
//! module docs for the full rationale -- repeated verbatim here):
//!
//! 1. Attribute policy, via `git check-attr` -- catches a *policy*
//!    regression (someone removing or narrowing the `.gitattributes` rule).
//! 2. Actual working-tree bytes -- catches an *already-affected* Windows
//!    clone that `git check-attr` alone cannot see, since Git only
//!    re-applies line-ending filters on fresh checkout or content change.
//!    Repair with (from the repo root):
//!
//!    ```text
//!    rm -rf tests/fixtures/hub
//!    git checkout -- tests/fixtures/hub
//!    ```
//!
//!    `git add --renormalize <path>` alone is *not* sufficient (verified
//!    empirically for the UI frame fixtures): it updates the index only,
//!    never the working-tree file.

use std::fs;
use std::process::Command;

const FIXTURE_DIR: &str = "tests/fixtures/hub";

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
fn hub_fixtures_are_attributed_text_eol_lf() {
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
/// above cannot see (see module docs): asserts the bytes actually on disk,
/// right now, are LF-only, regardless of what `.gitattributes` currently says.
#[test]
fn hub_fixture_bytes_are_lf_only() {
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
