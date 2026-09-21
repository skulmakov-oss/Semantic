//! Bounded coherence guard for the `sm-sema` diagnostic families (#1704).
//!
//! The expected lists below are deliberately fixed and explicit. This is NOT a
//! global diagnostic-inventory check: other catalog gaps (for example
//! `sm-front` parser codes) are known and out of scope, so nothing here scans
//! source for code literals.

use std::{collections::BTreeSet, fs, path::Path};

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
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
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
