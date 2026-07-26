//! Regression guard for the strict canonical UI frame inspection fixtures
//! (Issue #1365 follow-up).
//!
//! `smc look ui frame`'s Frame Snapshot v0 / Event Script v0 decoders are
//! strict: they require an exact trailing `\n`, not `\r\n`. Without a
//! `.gitattributes` override these fixtures fall under the repo's generic
//! `* text=auto` rule, so a checkout with `core.autocrlf=true` (a common
//! Windows Git default) rewrites their line endings to CRLF, which the
//! decoders then correctly reject as noncanonical.
//!
//! Two checks, for two different failure modes:
//!
//! 1. Attribute policy, via `git check-attr` -- catches a *policy*
//!    regression (someone removing or narrowing the `.gitattributes` rule).
//!    `core.autocrlf` doesn't apply on the Linux CI runners this repo uses,
//!    so a bytes-only check would never catch a policy regression there;
//!    `git check-attr` reports what `.gitattributes` would do regardless of
//!    the checking-out platform.
//! 2. Actual working-tree bytes -- catches the case `git check-attr` alone
//!    cannot: an *already-affected* Windows clone (checked out before this
//!    `.gitattributes` rule existed) that pulls this fix. Git only
//!    re-applies line-ending filters when a blob's content changes or the
//!    file is freshly checked out; an attribute-only change does not
//!    retroactively re-smudge files git already considers up to date, so a
//!    pre-existing CRLF-corrupted working copy stays corrupted after
//!    pulling this commit even though `git check-attr` now correctly
//!    reports `eol: lf`. If this second check ever fails locally, delete
//!    and re-checkout the directory to repair the working copy -- e.g.
//!    (from the repo root):
//!
//!    ```text
//!    rm -rf tests/fixtures/ui_frame_inspection
//!    git checkout -- tests/fixtures/ui_frame_inspection
//!    ```
//!
//!    `git add --renormalize <path>` alone is *not* sufficient: it
//!    re-applies the clean filter into the index, but never rewrites the
//!    working-tree file, so the on-disk bytes stay CRLF and this check
//!    would still fail (verified empirically).

use std::fs;
use std::process::Command;

const FIXTURE_DIR: &str = "tests/fixtures/ui_frame_inspection";

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
fn ui_frame_inspection_fixtures_are_attributed_text_eol_lf() {
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
fn ui_frame_inspection_fixture_bytes_are_lf_only() {
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
