//! PB-03 Model-B contract preservation exposed by FA-03-002 (#1671).
//!
//! The two `smc check` surfaces intentionally differ for Logos source:
//! unformatted `smc check` is executable-source admission and must reject
//! Logos; `smc check --format human|json` (canonical SSF-09 diagnostics,
//! shared with `smc lsp`) analyzes Logos for diagnostics. A future cleanup
//! must not collapse one into the other.

use std::path::PathBuf;
use std::process::Command;

const LOGOS_EXAMPLE: &str = "examples/canonical/quad_cycle_logos/src/main.sm";
const BOUNDARY: &str =
    "SOURCE SURFACE BOUNDARY: `smc check` admits executable RustLike source only";

fn smc(args: &[&str]) -> (i32, String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .current_dir(&root)
        .args(args)
        .output()
        .expect("run smc");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn legacy_check_rejects_logos_deterministically_even_when_cached() {
    for attempt in 0..2 {
        let (code, stdout, stderr) = smc(&["check", LOGOS_EXAMPLE]);
        assert_ne!(
            code, 0,
            "attempt {attempt}: Logos must not pass legacy check"
        );
        assert!(stderr.contains(BOUNDARY), "attempt {attempt}: {stderr}");
        assert!(
            !stdout.contains("smc check passed"),
            "attempt {attempt}: {stdout}"
        );
    }
}

#[test]
fn canonical_check_formats_keep_logos_diagnostics_behavior() {
    let (code, stdout, stderr) = smc(&["check", LOGOS_EXAMPLE, "--format", "human"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(stdout.contains("check passed"), "{stdout}");
    assert!(!stdout.contains(BOUNDARY) && !stderr.contains(BOUNDARY));

    let (code, stdout, stderr) = smc(&["check", LOGOS_EXAMPLE, "--format", "json"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(stdout.contains("\"passed\""), "{stdout}");
    assert!(!stdout.contains(BOUNDARY) && !stderr.contains(BOUNDARY));
}

#[test]
fn logos_inspection_stays_available_and_execution_stays_rejected() {
    assert_eq!(smc(&["dump-ast", LOGOS_EXAMPLE]).0, 0);
    assert_eq!(smc(&["dump-ir", LOGOS_EXAMPLE, "--profile", "logos"]).0, 0);
    assert_ne!(
        smc(&["dump-bytecode", LOGOS_EXAMPLE, "--profile", "logos"]).0,
        0
    );
    assert_ne!(smc(&["run", LOGOS_EXAMPLE]).0, 0);
}

#[test]
fn rustlike_legacy_check_is_unaffected() {
    let (code, stdout, stderr) =
        smc(&["check", "examples/canonical/match_control_flow/src/main.sm"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(!stderr.contains(BOUNDARY));
}
