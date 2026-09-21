//! Bounded coherence guard for the `sm-sema` diagnostic families (#1704).
//!
//! The expected lists below are deliberately fixed and explicit. This is NOT a
//! global diagnostic-inventory check: other catalog gaps (for example
//! `sm-front` parser codes) are known and out of scope, so nothing here scans
//! source for code literals.

use std::{collections::BTreeSet, fs, path::Path};

use sm_sema::{check_source, diagnostic_help_core};
use smc_cli::CliPipeline;
use ton618_core::diagnostics::diagnostic_catalog;

/// The bounded `sm-sema` families covered by this guard. `E0240` is listed
/// because it is catalogued today; it currently has no production
/// construction site and its text is intentionally not asserted here.
const BOUNDED_CODES: [&str; 14] = [
    "E0238", "E0239", "E0240", "E0241", "E0242", "E0243", "E0244", "E0245", "W0240", "W0241",
    "W0250", "W0251", "W0252", "W0253",
];

/// Production-emitted import/export codes that were missing from the catalog.
const FORMERLY_MISSING_CODES: [&str; 4] = ["E0242", "E0243", "E0244", "E0245"];

fn read(relative: &str) -> String {
    // Normalize line endings: Windows checkouts may materialize CRLF.
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
        .replace("\r\n", "\n")
}

#[test]
fn catalog_contains_every_bounded_sema_code() {
    let catalog: BTreeSet<&str> = diagnostic_catalog().iter().map(|(code, _)| *code).collect();
    for code in BOUNDED_CODES {
        assert!(catalog.contains(code), "diagnostic_catalog() lacks {code}");
    }
}

#[test]
fn catalog_codes_are_unique_and_sorted() {
    let codes: Vec<&str> = diagnostic_catalog().iter().map(|(code, _)| *code).collect();
    for pair in codes.windows(2) {
        assert!(
            pair[0] < pair[1],
            "catalog must be strictly ascending with no duplicates: {} then {}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn every_bounded_code_is_explainable() {
    for code in BOUNDED_CODES {
        let text = CliPipeline::explain(code)
            .unwrap_or_else(|| panic!("`smc explain {code}` has no catalog entry"));
        assert!(!text.trim().is_empty(), "empty explain text for {code}");
    }
    for code in FORMERLY_MISSING_CODES {
        let lowercase = code.to_ascii_lowercase();
        assert!(
            CliPipeline::explain(&lowercase).is_some(),
            "explain must stay case-insensitive for {code}"
        );
    }
}

#[test]
fn error_codes_mirror_lists_every_catalog_code() {
    let mirror = read("docs/ERROR_CODES.md");
    for (code, _) in diagnostic_catalog() {
        assert!(
            mirror.contains(&format!("- `{code}`:")),
            "docs/ERROR_CODES.md must mirror catalog code {code}"
        );
    }
    assert!(
        mirror.contains("crates/ton618-core/src/diagnostics.rs"),
        "the mirror's maintenance note must name the actual catalog owner"
    );
    assert!(
        !mirror.contains("src/bin/smc.rs"),
        "the mirror must not point maintainers at the stale catalog location"
    );
}

#[test]
fn dead_when_and_constant_fold_warnings_have_help() {
    // Pin the exact mappings so a swap or a wrong-code mapping is caught.
    assert_eq!(
        diagnostic_help_core("W0240"),
        Some("Remove or revise the branch whose When condition is always false.")
    );
    assert_eq!(
        diagnostic_help_core("W0241"),
        Some("Consider replacing the literal-only fx.* call with its precomputed constant.")
    );

    // Neighbouring help entries are unchanged.
    assert_eq!(
        diagnostic_help_core("E0242"),
        Some("Rename with 'as' or export symbols selectively to avoid collisions.")
    );
    assert_eq!(
        diagnostic_help_core("W0250"),
        Some("Use UpperCamelCase names for laws to keep style consistent.")
    );
    assert_eq!(diagnostic_help_core("W9999"), None);
}

#[test]
fn rendered_dead_when_and_constant_fold_warnings_carry_help() {
    let cases = [
        (
            "W0240",
            "Entity A:\n    state x: quad\nLaw \"L\" [priority 1]:\n    When N ->\n        Pulse.emit(\"x\")\n",
        ),
        (
            "W0241",
            "Law \"L\" [priority 1]:\n    When true -> fx.add(1.0, 2.0)\n",
        ),
    ];
    for (code, src) in cases {
        let report = check_source(src).unwrap_or_else(|e| panic!("{code} fixture failed: {e}"));
        let warning = report
            .warnings
            .iter()
            .find(|w| w.code == code)
            .unwrap_or_else(|| panic!("fixture did not emit {code}"));
        let help = diagnostic_help_core(code).expect("help entry");
        assert!(
            warning.rendered.contains(&format!("help: {help}")),
            "{code} rendered text must carry its help line: {}",
            warning.rendered
        );
    }
}

#[test]
fn spec_warning_families_list_every_bounded_warning() {
    let spec = read("docs/spec/diagnostics.md");
    let intro = spec
        .find("Current warning families include:")
        .expect("spec must have a warning families list");
    // The bullet list starts after the intro sentence's blank line and ends
    // at the next blank line.
    let after_intro = &spec[intro..];
    let bullets_at = after_intro.find("\n\n").expect("blank line after intro") + 2;
    let bullets = &after_intro[bullets_at..];
    let list = &bullets[..bullets.find("\n\n").unwrap_or(bullets.len())];
    for code in ["W0240", "W0241", "W0250", "W0251", "W0252", "W0253"] {
        assert!(
            list.contains(&format!("- `{code}`")),
            "spec warning families list must include {code}:\n{list}"
        );
    }
}
