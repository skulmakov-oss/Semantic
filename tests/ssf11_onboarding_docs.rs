//! SSF-11 (#1582) onboarding and drift guards.
//!
//! A small repository-local guard (no Markdown parser dependency) that keeps
//! current-facing onboarding honest: referenced paths exist, every onboarding
//! topic has a canonical owner, corpus positives are indexed, boundary and
//! historical evidence is never shown as a stable positive, SSF-12 is not
//! described as done, and retired native UI is not presented as current
//! Foundation application qualification.

use serde_json::Value;
use std::path::{Path, PathBuf};

const MATRIX: &str = "docs/roadmap/stable_foundation/ssf11_application_onboarding_matrix.md";
const CORPUS: &str = "examples/qualification/ssf11/corpus.json";
const CURRENT_FACING: [&str; 5] = [
    "docs/getting_started.md",
    "docs/examples_index.md",
    "docs/LANGUAGE.md",
    MATRIX,
    "examples/canonical/README.md",
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(repo().join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
        .replace("\r\n", "\n")
}

fn corpus() -> Value {
    serde_json::from_str(&read(CORPUS)).expect("corpus JSON")
}

/// Repository-relative path references: inline code spans that contain no
/// whitespace (so quoted command output such as `ok tests/x.sm` is not read
/// as a path) and Markdown link targets.
fn referenced_paths(document: &str) -> Vec<String> {
    let mut candidates: Vec<&str> = document
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| !span.chars().any(char::is_whitespace))
        .collect();
    for link in document.split("](").skip(1) {
        candidates.push(link.split(')').next().unwrap_or_default());
    }
    let mut found = Vec::new();
    for candidate in candidates {
        let token = candidate.trim_start_matches("../");
        let token = token.split('#').next().unwrap_or_default();
        let rooted = [
            "docs/",
            "examples/",
            "tests/",
            "reports/",
            "crates/",
            "scripts/",
        ]
        .iter()
        .any(|prefix| token.starts_with(prefix));
        if rooted && !token.contains('<') && !token.contains('*') && !token.contains('{') {
            found.push(token.to_string());
        }
    }
    found
}

fn resolve(document_path: &str, reference: &str) -> bool {
    let root = repo();
    if root.join(reference).exists() {
        return true;
    }
    // Relative Markdown links (e.g. `spec/source_style.md` from docs/).
    let base = Path::new(document_path).parent().unwrap_or(Path::new(""));
    root.join(base).join(reference).exists()
}

#[test]
fn current_facing_onboarding_references_resolve() {
    for document_path in CURRENT_FACING {
        let document = read(document_path);
        for reference in referenced_paths(&document) {
            assert!(
                resolve(document_path, &reference),
                "{document_path} references missing path {reference}"
            );
        }
    }
}

#[test]
fn every_onboarding_topic_has_an_existing_canonical_owner() {
    let matrix = read(MATRIX);
    let section = matrix
        .split("## Onboarding topic ownership")
        .nth(1)
        .expect("ownership section");
    let section = section.split("\n## ").next().unwrap_or_default();
    for topic in [
        "Getting Started",
        "Language Tour",
        "Semantic By Example",
        "Project Model",
        "Package Baseline",
        "Standard Library",
        "CLI",
        "Diagnostics",
        "Verifier / Runtime",
        "Compatibility / Migration",
        "Troubleshooting",
        "Release Status",
    ] {
        let row = section
            .lines()
            .find(|line| line.contains(&format!("| {topic} |")))
            .unwrap_or_else(|| panic!("onboarding topic {topic} has no owner row"));
        let owners = referenced_paths(row);
        assert!(!owners.is_empty(), "{topic}: owner row names no document");
        for owner in owners {
            assert!(
                repo().join(&owner).exists(),
                "{topic}: owner {owner} missing"
            );
        }
    }
    let getting_started = read("docs/getting_started.md");
    for heading in ["## Prerequisites", "## Troubleshooting", "Release status"] {
        assert!(
            getting_started.contains(heading),
            "getting started lacks {heading}"
        );
    }
    assert!(
        getting_started.contains("libopenblas-dev"),
        "Linux BLAS prerequisite"
    );
}

#[test]
fn foundation_positive_corpus_cases_are_indexed_for_users() {
    let index = read("docs/examples_index.md");
    let matrix = read(MATRIX);
    let corpus = corpus();
    for case in corpus["cases"].as_array().expect("cases") {
        let Some(entry) = case["entry_path"].as_str() else {
            continue;
        };
        let case_id = case["case_id"].as_str().unwrap();
        if case["kind"] == "positive" {
            let directory = Path::new(entry)
                .parent()
                .expect("entry parent")
                .to_string_lossy()
                .replace('\\', "/");
            let named = matrix.contains(entry)
                || matrix.contains(&directory)
                || directory
                    .rsplit('/')
                    .take(2)
                    .any(|segment| matrix.contains(&format!("`{segment}")));
            assert!(
                named,
                "{case_id}: positive {entry} is not named in the matrix"
            );
        }
    }
    for family in corpus["families"].as_array().expect("families") {
        let id = family["family_id"].as_str().unwrap();
        assert!(
            index.contains(&format!("| {id} ")),
            "examples index has no row for {id}"
        );
        let disposition = family["disposition"].as_str().unwrap();
        let matrix_row = matrix
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("matrix has no row for {id}"));
        assert!(
            matrix_row.contains(disposition),
            "{id}: matrix disposition disagrees with corpus ({disposition})"
        );
    }
}

#[test]
fn excluded_and_historical_rows_are_never_presented_as_stable_positives() {
    let index = read("docs/examples_index.md");
    for line in index.lines().filter(|line| line.starts_with("| F")) {
        let boundary = line.contains("(boundary)") || line.contains("F12");
        if boundary {
            assert!(
                line.contains("Roadmap")
                    || line.contains("Out of scope")
                    || line.contains("negative")
                    || line.contains("Boundary"),
                "boundary/historical row lacks honest maturity: {line}"
            );
            assert!(!line.contains("Published stable"), "{line}");
        }
    }
    let corpus = corpus();
    for case in corpus["cases"].as_array().unwrap() {
        if case["maturity"] == "Roadmap" || case["maturity"] == "Out of scope" {
            assert_ne!(case["kind"], "positive", "{}", case["case_id"]);
        }
    }
}

#[test]
fn ssf12_is_not_described_as_completed_or_promoted() {
    let dependencies = read("docs/roadmap/stable_foundation/stable_foundation_dependency_map.md");
    let ssf12 = dependencies
        .lines()
        .find(|line| line.starts_with("| SSF-12 / #1583 |"))
        .expect("SSF-12 row");
    assert!(
        ssf12.contains("Blocked by SSF-11"),
        "SSF-12 must stay blocked: {ssf12}"
    );
    let ssf11 = dependencies
        .lines()
        .find(|line| line.starts_with("| SSF-11 / #1582 |"))
        .expect("SSF-11 row");
    assert!(
        ssf11.contains("**Active**"),
        "SSF-11 must be the active phase"
    );

    for document_path in CURRENT_FACING {
        let lower = read(document_path).to_ascii_lowercase();
        for claim in [
            "semantic is now stable",
            "semantic 1.0 is released",
            "ssf-12 completed",
            "ssf-12 is complete",
            "verdict: promote",
            "all main features are stable",
        ] {
            assert!(!lower.contains(claim), "{document_path} claims `{claim}`");
        }
    }
    let getting_started = read("docs/getting_started.md");
    assert!(getting_started.contains("**not** a published stable release"));
}

#[test]
fn retired_native_ui_is_not_current_foundation_application_evidence() {
    let matrix = read(MATRIX);
    let f12 = matrix
        .lines()
        .find(|line| line.starts_with("| F12 |"))
        .expect("F12 row");
    assert!(f12.contains("HISTORICAL-NON-QUALIFYING"), "{f12}");
    assert!(f12.contains("Historical only"), "{f12}");
    let corpus = corpus();
    for case in corpus["cases"].as_array().unwrap() {
        let text = case.to_string();
        if text.contains("workbench") || text.contains("prom-ui") || text.contains("apps/") {
            assert_eq!(case["kind"], "historical", "{}", case["case_id"]);
        }
        if let Some(entry) = case["entry_path"].as_str() {
            for retired in ["examples/workbench_semantic", "apps/", "crates/prom-ui"] {
                assert!(!entry.starts_with(retired), "{entry} revives retired UI");
            }
        }
    }
}
