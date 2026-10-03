//! SSF-12 (#1583) qualification integrity guards for candidate C1.
//!
//! Validates that the frozen candidate identity, qualification matrix,
//! Foundation Oracle verdict, and Stable Foundation promotion recommendation
//! remain honest, coherent, and free of accidental promotion drift.

use std::{fs, path::Path};

const CANDIDATE_SHA: &str = "89641da8237f4fcefb50cf1958a50e4d4003aea7";
const VERDICT_FILE: &str = "reports/semantic_stable_foundation_final_verdict.md";
const MATRIX_FILE: &str = "reports/ssf12/qualification_matrix.md";
const MANIFEST_FILE: &str = "reports/ssf12/qualification_manifest.json";

fn read(relative: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"))
        .replace("\r\n", "\n")
}

#[test]
fn ssf12_frozen_candidate_sha_matches_across_artifacts() {
    let verdict = read(VERDICT_FILE);
    let matrix = read(MATRIX_FILE);
    let manifest = read(MANIFEST_FILE);
    let harness = read(".harness/current.task.yaml");

    assert!(
        verdict.contains(&format!("- **Candidate SHA**: `{CANDIDATE_SHA}`")),
        "verdict file missing authoritative Candidate SHA field `- **Candidate SHA**: `{CANDIDATE_SHA}``"
    );
    assert!(
        matrix.contains(&format!("Frozen Candidate SHA: `{CANDIDATE_SHA}`")),
        "matrix file missing authoritative Frozen Candidate SHA header field"
    );
    assert!(
        manifest.contains(&format!("\"candidate_sha\": \"{CANDIDATE_SHA}\"")),
        "manifest file missing authoritative candidate_sha JSON field"
    );
    assert!(
        harness.contains(&format!("candidate_sha: {CANDIDATE_SHA}")),
        "harness file missing authoritative candidate_sha YAML field"
    );
}

#[test]
fn ssf12_verdict_contains_all_mandatory_sections() {
    let verdict = read(VERDICT_FILE);

    let required_sections = [
        "1. Executive Verdict",
        "2. Frozen Candidate Identity",
        "3. Evidence Branch Identity",
        "4. Toolchain Identity",
        "5. Host Qualification Environment(s)",
        "6. Final Stable Foundation Contour",
        "7. Explicit Exclusions",
        "8. SSF-00..SSF-11 Closure Evidence Map",
        "9. Qualification Gate Matrix",
        "10. Compiler / Workspace Qualification",
        "11. Public API / Boundary Qualification",
        "12. Verifier Adversarial Qualification",
        "13. Runtime Determinism",
        "14. Trap / Quota / Ownership Qualification",
        "15. Standard Library Qualification",
        "16. Project / Package Qualification",
        "17. Capability Boundary Qualification",
        "18. Diagnostics / Formatter / LSP Qualification",
        "19. Compatibility / Migration Qualification",
        "20. Artifact Identity / Provenance Qualification",
        "21. SSF-11 Corpus Qualification",
        "22. Full 7HELL Qualification",
        "23. Release Readiness Gates",
        "24. Release Bundle Qualification",
        "25. Platform Matrix",
        "26. Pre-publication Asset Smoke",
        "27. Published Asset Smoke Status",
        "28. Clean-clone Rehearsal",
        "29. Known Limits",
        "30. Earlier-phase Returns / Blockers",
        "31. Failed Gates",
        "32. Skipped Gates",
        "33. Artifact / Evidence Hashes",
        "34. Foundation Oracle Verdict",
        "35. Stable Foundation Promotion Recommendation",
        "36. Human Decision Required",
    ];

    for section in required_sections {
        assert!(
            verdict.contains(section),
            "verdict report missing required section heading: {section}"
        );
    }
}

#[test]
fn ssf12_verdicts_are_explicit_and_honest() {
    let verdict = read(VERDICT_FILE);

    assert!(
        verdict.contains("ORACLE QUALIFIED WITH EXPLICIT LIMITS"),
        "verdict missing Foundation Oracle decision"
    );
    assert!(
        verdict.contains("PROMOTE WITH EXPLICIT LIMITS"),
        "verdict missing promotion recommendation"
    );
    assert!(
        verdict.contains("DEFECT-SSF12-001"),
        "verdict missing DEFECT-SSF12-001 verification"
    );
    assert!(
        verdict.contains("DEFECT-SSF12-002"),
        "verdict missing DEFECT-SSF12-002 verification"
    );
    assert!(
        verdict.contains("DEFECT-SSF12-003"),
        "verdict missing DEFECT-SSF12-003 verification"
    );
    assert!(
        verdict.contains("DEFECT-SSF12-004"),
        "verdict missing DEFECT-SSF12-004 verification"
    );
}

#[test]
fn ssf12_does_not_falsely_claim_promotion_or_release() {
    let harness = read(".harness/current.task.yaml");
    assert!(harness.contains("stable_promotion: false"));
    assert!(harness.contains("no_release_or_tag: true"));

    let verdict = read(VERDICT_FILE);
    assert!(!verdict.contains("```text\nPROMOTE\n```"));
    assert!(!verdict.contains("RELEASE AUTHORIZED"));
    assert!(verdict.contains("Promotion decision remains reserved to the repository owner."));
}
