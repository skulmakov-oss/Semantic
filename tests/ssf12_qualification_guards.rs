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
        "35. Stable Foundation Promotion Decision",
        "36. Human Decision",
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
    assert!(harness.contains("stable_promotion: true"));
    assert!(harness.contains("release_authorized: true"));
    assert!(harness.contains("tag_authorized: true"));

    let verdict = read(VERDICT_FILE);
    assert!(verdict.contains("Stable Foundation Promotion Decision"));
    assert!(verdict.contains("PROMOTE WITH EXPLICIT LIMITS"));
    assert!(!verdict.contains("```text\nPROMOTE\n```"));
}

#[test]
fn ssf12_summary_counts_agree_across_all_artifacts() {
    let verdict = read(VERDICT_FILE);
    let matrix = read(MATRIX_FILE);
    let manifest = read(MANIFEST_FILE);

    // Summary counts in verdict section 9
    assert!(verdict.contains("- **Total Gates**: 78"));
    assert!(verdict.contains("- **PASS**: 77"));
    assert!(verdict.contains("- **FAIL**: 0"));
    assert!(verdict.contains("- **BLOCKED**: 0"));
    assert!(verdict.contains("- **NOT_APPLICABLE**: 1 (`G-08`)"));

    // Summary counts in matrix section 3
    assert!(matrix.contains("- **Total Qualification Gates**: 78"));
    assert!(matrix.contains("- **PASS**: 77"));
    assert!(matrix.contains("- **FAIL**: 0"));
    assert!(matrix.contains("- **BLOCKED**: 0"));
    assert!(matrix.contains("- **NOT_APPLICABLE**: 1 (`G-08`"));

    // Summary counts in manifest json
    assert!(manifest.contains("\"total\": 78"));
    assert!(manifest.contains("\"pass\": 77"));
    assert!(manifest.contains("\"fail\": 0"));
    assert!(manifest.contains("\"blocked\": 0"));
    assert!(manifest.contains("\"not_applicable\": 1"));
    assert!(manifest.contains("\"skipped\": 0"));
    assert!(manifest.contains("\"inconclusive\": 0"));
}

#[test]
fn ssf12_manifest_gate_level_integrity() {
    let manifest_str = read(MANIFEST_FILE);
    let json: serde_json::Value =
        serde_json::from_str(&manifest_str).expect("manifest must be valid JSON");

    let gates = json["gates"].as_array().expect("gates must be an array");
    assert_eq!(gates.len(), 78, "manifest must contain exactly 78 gates");

    let mut ids = std::collections::HashSet::new();
    let mut pass = 0;
    let mut fail = 0;
    let mut blocked = 0;
    let mut na = 0;

    for g in gates {
        let id = g["id"].as_str().expect("gate must have id string");
        assert!(ids.insert(id.to_string()), "duplicate gate id: {id}");
        match g["status"].as_str().expect("status string") {
            "PASS" => pass += 1,
            "FAIL" => fail += 1,
            "BLOCKED" => blocked += 1,
            "NOT_APPLICABLE" => {
                na += 1;
                assert_eq!(id, "G-08", "only G-08 may be NOT_APPLICABLE");
            }
            other => panic!("unexpected gate status {other} for gate {id}"),
        }
    }

    assert_eq!(pass, 77, "derived pass count must be 77");
    assert_eq!(fail, 0, "derived fail count must be 0");
    assert_eq!(blocked, 0, "derived blocked count must be 0");
    assert_eq!(na, 1, "derived not_applicable count must be 1");
}

#[test]
fn ssf12_stale_c0_claims_are_prohibited_in_active_verdict() {
    let verdict = read(VERDICT_FILE);

    // Active C1 sections must not have stale C0 failure counts or blockers
    assert!(
        !verdict.contains("- **FAIL**: 6"),
        "verdict contains stale C0 fail count"
    );
    assert!(
        !verdict.contains("FAIL = 6"),
        "verdict contains stale C0 fail equation"
    );
    assert!(
        !verdict.contains("DEFECT-SSF12-001 blocks promotion"),
        "verdict contains stale C0 blocker claim"
    );
    assert!(
        !verdict.contains("10 failing editor tests"),
        "verdict contains stale C0 test failure phrasing"
    );

    // Ensure candidate commit subject is accurate
    assert!(
        verdict.contains("fix(ssf12): reuse Windows-safe workspace fmt gate in full 7hell (#1982)"),
        "verdict missing candidate C1 commit message"
    );
}

#[test]
fn ssf12_q02_smoke_script_uses_dynamic_toolchain_semcode_format() {
    let script = read("scripts/verify_release_assets.ps1");

    // Script must not use legacy hardcoded format headers as expected signals
    assert!(
        !script.contains("\"SEMCODE0\""),
        "verify_release_assets.ps1 must not hardcode SEMCODE0 as expected signal"
    );
    assert!(
        !script.contains("\"SEMCODE1\""),
        "verify_release_assets.ps1 must not hardcode SEMCODE1 as expected signal"
    );

    // Script must extract semcode_format from release toolchain version metadata
    assert!(
        script.contains(".semcode_format"),
        "verify_release_assets.ps1 must extract semcode_format from toolchain metadata"
    );
    assert!(
        script.contains("$expectedSemcodeFormat"),
        "verify_release_assets.ps1 must bind expected signals to $expectedSemcodeFormat"
    );
}
