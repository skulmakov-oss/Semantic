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
