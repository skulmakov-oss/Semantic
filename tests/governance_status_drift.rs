//! Lightweight guards against governance/status drift reconciled by Issue
//! #1969. They anchor a few stable facts with small strings rather than
//! snapshotting prose:
//!
//! - the native UI / Workbench / Semantic Studio retirement decision exists,
//!   is honest (retired, not fixed), and is linked from the current-facing
//!   status authorities;
//! - no retired contour is wired into CI or the local admission guard as a
//!   required gate;
//! - current-facing status documents carry no machine-local paths;
//! - release-facing "landed, not yet promised" lists do not re-list surfaces
//!   that the same documents name as qualified (FND-064, FND-065).

use std::{fs, path::Path};

const RETIREMENT_DOC: &str = "docs/roadmap/ui_workbench_studio_retirement.md";

fn read(relative: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
        .replace("\r\n", "\n")
}

/// Text of the markdown section whose heading line starts with `heading`,
/// up to the next heading of the same or higher level.
fn section<'a>(document: &'a str, heading: &str) -> &'a str {
    let start = document
        .find(heading)
        .unwrap_or_else(|| panic!("missing section {heading}"));
    let level = heading.chars().take_while(|c| *c == '#').count();
    let body = &document[start + heading.len()..];
    let end = body
        .match_indices("\n#")
        .find(|(index, _)| {
            let hashes = body[index + 1..].chars().take_while(|c| *c == '#').count();
            hashes <= level
        })
        .map(|(index, _)| index)
        .unwrap_or(body.len());
    &body[..end]
}

#[test]
fn retirement_decision_is_explicit_and_not_a_fix_claim() {
    let decision = read(RETIREMENT_DOC);
    for anchor in [
        "Semantic no longer owns a native UI product roadmap.",
        "Its unresolved historical findings are not represented as fixed",
        "| defects fixed | **No.**",
        "| stable / published | **No.**",
        "| completed | **No.**",
        "#1968",
        "#1862",
        "#1910",
        "C0 → C1 → C2",
    ] {
        assert!(
            decision.contains(anchor),
            "{RETIREMENT_DOC} is missing anchor {anchor:?}"
        );
    }
}

#[test]
fn current_facing_authorities_link_the_retirement_decision() {
    for relative in [
        "README.md",
        "docs/roadmap/public_status_model.md",
        "docs/roadmap/v1_readiness.md",
        "docs/roadmap/backlog.md",
        "docs/roadmap/milestones.md",
        "docs/roadmap/wbs.md",
        "docs/status/feature_maturity_matrix.md",
        "docs/roadmap/stable_foundation/semantic_stable_foundation_matrix.md",
        "docs/roadmap/stable_foundation/stable_foundation_target_contract.md",
        "docs/wiki/current_status.md",
    ] {
        assert!(
            read(relative).contains("ui_workbench_studio_retirement.md"),
            "{relative} must point at the retirement decision"
        );
    }
}

#[test]
fn retired_contour_is_not_a_required_ci_or_admission_gate() {
    for relative in [
        ".github/workflows/ci.yml",
        ".github/workflows/7hell-full.yml",
        "scripts/admission_guard.ps1",
        "scripts/admission_guard_lib.ps1",
        "scripts/local_ci.ps1",
        "tools/7hell/run_ci.ps1",
    ] {
        let text = read(relative).to_ascii_lowercase();
        for retired in [
            "workbench",
            "semantic studio",
            "workbench_native_launch_smoke",
        ] {
            assert!(
                !text.contains(retired),
                "{relative} references retired contour {retired:?} as part of a gate"
            );
        }
    }
    let smoke = read("scripts/workbench_native_launch_smoke.ps1");
    assert!(
        smoke.starts_with("# RETIRED CONTOUR"),
        "the historical Workbench smoke script must be marked retired"
    );
}

#[test]
fn current_facing_status_docs_have_no_machine_local_paths() {
    let patterns = ["c:\\users\\", "c:/users/", "/c/users/", "/users/", "/home/"];
    for relative in [
        "README.md",
        "docs/roadmap/public_status_model.md",
        "docs/roadmap/v1_readiness.md",
        "docs/roadmap/backlog.md",
        "docs/roadmap/milestones.md",
        "docs/roadmap/wbs.md",
        "docs/status/feature_maturity_matrix.md",
        "docs/wiki/current_status.md",
        "docs/roadmap/repository_truth_audit_2026-04-22.md",
        "docs/roadmap/pcc/collections_core_audit.md",
        "docs/roadmap/pcc/text_core_audit.md",
        "docs/roadmap/pcc/control_flow_core_audit.md",
        "docs/roadmap/pcc/cli_public_sample_qualification_audit.md",
        RETIREMENT_DOC,
    ] {
        let text = read(relative).to_ascii_lowercase();
        for pattern in patterns {
            assert!(
                !text.contains(pattern),
                "{relative} contains a machine-local path ({pattern})"
            );
        }
    }
}

#[test]
fn landed_lists_do_not_relist_qualified_surfaces() {
    let readiness = read("docs/roadmap/v1_readiness.md");
    let readiness_landed = section(&readiness, "## Landed On `main`, Not Yet Promised");
    let backlog = read("docs/roadmap/backlog.md");
    let backlog_landed = section(&backlog, "## Landed On `main`, Not Yet Promised");

    for (relative, landed) in [
        ("docs/roadmap/v1_readiness.md", readiness_landed),
        ("docs/roadmap/backlog.md", backlog_landed),
    ] {
        for bullet in [
            "\n- selected-import executable module entry\n",
            "\n- built-in iterable surface and direct-record iterable dispatch\n",
            "\n- iterable surface\n",
            "\n- first-wave UI application boundary\n",
        ] {
            assert!(
                !landed.contains(bullet),
                "{relative} lists {:?} as landed-not-promised",
                bullet.trim()
            );
        }
    }

    for relative in ["docs/roadmap/v1_readiness.md", "docs/roadmap/backlog.md"] {
        let text = read(relative);
        assert!(
            text.contains("direct-record user-defined `Iterable` dispatch")
                || text.contains("direct-record user-defined `Iterable`"),
            "{relative} must keep direct-record Iterable dispatch in the qualified contour"
        );
    }
}
