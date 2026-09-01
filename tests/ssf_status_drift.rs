use std::{fs, path::Path};

const BASE_SHA: &str = "89a014b66e7c1e40502dbd764c94bf5f9445677f";

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

#[test]
fn ssf_00_status_authorities_do_not_drift() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let matrix = read(
        root,
        "docs/roadmap/stable_foundation/semantic_stable_foundation_matrix.md",
    );
    let target = read(
        root,
        "docs/roadmap/stable_foundation/stable_foundation_target_contract.md",
    );
    let dependencies = read(
        root,
        "docs/roadmap/stable_foundation/stable_foundation_dependency_map.md",
    );

    for document in [&matrix, &target, &dependencies] {
        assert!(
            document.contains(BASE_SHA),
            "SSF-00 authority is missing base SHA {BASE_SHA}"
        );
    }

    for status in [
        "Published stable",
        "Qualified limited release",
        "Landed and qualified on `main`",
        "Landed but unqualified",
        "Experimental",
        "Roadmap",
        "Out of scope",
    ] {
        assert!(matrix.contains(status), "matrix is missing status {status}");
    }

    for phase in 1..=12 {
        let phase_id = format!("SSF-{phase:02}");
        assert!(
            dependencies.contains(&phase_id),
            "dependency map is missing {phase_id}"
        );
    }

    for relative in [
        "README.md",
        "docs/roadmap/v1_readiness.md",
        "docs/roadmap/stable_release_policy.md",
        "docs/roadmap/compatibility_statement.md",
        "docs/status/feature_maturity_matrix.md",
        "docs/release_artifact_model.md",
        "docs/wiki/current_status.md",
    ] {
        let current_facing = read(root, relative)
            .replace("\r\n", "\n")
            .to_ascii_lowercase();
        assert!(
            !current_facing.contains("published stable line is `v1.1.1`")
                && !current_facing.contains("published stable line remains:\n\n- `v1.1.1`")
                && !current_facing.contains("current published stable line is:\n\n- `v1.1.1`")
                && !current_facing
                    .contains("published stable line is currently:\n\n```text\nv1.1.1\n```"),
            "stale published-stable assertion remains in {relative}"
        );
    }
}

/// Added after SSF-07 #1578 was found closed (2026-08-30) while
/// `stable_foundation_dependency_map.md` and `.harness/current.task.yaml`
/// both still said SSF-07 was Active -- the dependency map's `Current state`
/// column and the harness's `active_phase`/`issue` fields are two
/// independent authorities for "which SSF phase is active right now" and
/// nothing previously checked they agreed. This closes that gap: exactly
/// one dependency-map row may be marked `**Active**`, and its `SSF-NN /
/// #NNNN` phase id must match the harness's `active_phase`/`issue` fields
/// exactly.
#[test]
fn ssf_active_phase_matches_across_dependency_map_and_harness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dependencies = read(
        root,
        "docs/roadmap/stable_foundation/stable_foundation_dependency_map.md",
    );
    let harness = read(root, ".harness/current.task.yaml");

    let active_rows: Vec<&str> = dependencies
        .lines()
        .filter(|line| line.contains("| **Active** |"))
        .collect();
    assert_eq!(
        active_rows.len(),
        1,
        "expected exactly one dependency-map phase row marked Active, found {}: {active_rows:?}",
        active_rows.len()
    );
    let phase_id = active_rows[0]
        .trim_start_matches('|')
        .split('|')
        .next()
        .unwrap_or_default()
        .trim();
    let (ssf_phase, issue_part) = phase_id
        .split_once('/')
        .unwrap_or_else(|| panic!("could not parse phase id from dependency-map row: {phase_id}"));
    let ssf_phase = ssf_phase.trim();
    let issue_number = issue_part.trim().trim_start_matches('#').trim();

    let harness_active_phase = harness
        .lines()
        .find_map(|line| line.trim().strip_prefix("active_phase:"))
        .map(str::trim)
        .unwrap_or_else(|| panic!("`.harness/current.task.yaml` is missing `active_phase:`"));
    let harness_issue = harness
        .lines()
        .find_map(|line| line.trim().strip_prefix("issue:"))
        .map(str::trim)
        .unwrap_or_else(|| panic!("`.harness/current.task.yaml` is missing `issue:`"));

    assert_eq!(
        ssf_phase, harness_active_phase,
        "dependency map marks {ssf_phase} Active but harness active_phase is \
         {harness_active_phase} -- these two authorities have drifted"
    );
    assert_eq!(
        issue_number, harness_issue,
        "dependency map's Active phase is issue #{issue_number} but harness issue is \
         {harness_issue} -- these two authorities have drifted"
    );
}

/// Added when SSF-08 selected Position A (bounded deterministic VM language,
/// no Rust-equivalent lifetime/region claim) in
/// `docs/roadmap/stable_foundation/ssf08_ownership_position_decision.md`.
/// Three independent authorities each carry that decision: the decision
/// record itself, `.harness/current.task.yaml`, and the frozen runtime
/// ownership spec's non-goal boundary. Nothing previously checked they
/// agreed. This uses small, stable anchor strings rather than a large prose
/// snapshot, so it does not accidentally freeze unrelated wording.
#[test]
fn ssf08_ownership_position_matches_across_decision_record_harness_and_spec() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let decision = read(
        root,
        "docs/roadmap/stable_foundation/ssf08_ownership_position_decision.md",
    );
    let harness = read(root, ".harness/current.task.yaml");
    let runtime_ownership = read(root, "docs/spec/runtime_ownership.md");

    assert!(
        decision.contains("Decision: **Position A — bounded deterministic VM language**"),
        "decision record is missing its own Position A anchor"
    );
    assert!(
        harness.contains("ownership_position: A"),
        ".harness/current.task.yaml is missing `ownership_position: A`"
    );
    assert!(
        harness.contains("no_rust_equivalent_lifetime_region_claim: true"),
        ".harness/current.task.yaml is missing the no-Rust-equivalent-claim invariant"
    );
    assert!(
        runtime_ownership.contains("Position A — bounded deterministic VM language"),
        "runtime_ownership.md's Public Position section is missing the Position A anchor"
    );
    assert!(
        runtime_ownership.contains("does not claim a general runtime borrow checker"),
        "runtime_ownership.md is missing its general-borrow-checker non-claim boundary"
    );
}
