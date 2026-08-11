use std::{fs, path::Path};

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

#[test]
fn ssf_01_language_contract_keeps_its_version_and_evidence_map() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = read(root, "docs/spec/foundation_source_profile_v1.md");
    let evidence = read(
        root,
        "docs/roadmap/stable_foundation/stable_public_language_contract.md",
    );

    for required in [
        "semantic.foundation.source/1.0",
        "semantic.foundation`/`1.0",
        "Included executable surface",
        "Experimental but currently accepted extensions",
        "Deterministically unsupported forms",
        "Source-to-SemCode relationship",
        "SEMCODE0` through `SEMCOD14",
        "not published stable",
    ] {
        assert!(
            profile.contains(required),
            "Foundation Source Profile is missing {required}"
        );
    }

    for required in [
        "Included stable candidate",
        "Experimental",
        "Deferred",
        "SSF-02 entry conditions",
        "4de0b6eb1cd5d8e5dc37989e9b9b95a5a8e07e57",
    ] {
        assert!(
            evidence.contains(required),
            "SSF-01 evidence map is missing {required}"
        );
    }

    for relative in [
        "tests/practical_surface_execution_qualification.rs",
        "tests/call_shape_surface_qualification.rs",
        "tests/mutable_binding_qualification.rs",
        "tests/match_surface_qualification.rs",
        "tests/pcc1_control_flow_gate.rs",
        "tests/pcc2_numeric_core_gate.rs",
        "tests/pcc3_text_core_gate.rs",
        "tests/pcc4_records_acceptance.rs",
        "tests/pcc5_adt_acceptance.rs",
        "tests/pcc6_option_acceptance.rs",
        "tests/pcc6_result_acceptance.rs",
        "tests/pcc7_sequence_acceptance.rs",
        "tests/pcc7_map_acceptance.rs",
        "tests/short_lambda_surface_qualification.rs",
        "tests/import_surface_qualification.rs",
        "tests/canonical_examples.rs",
    ] {
        assert!(
            root.join(relative).is_file(),
            "missing evidence file {relative}"
        );
        assert!(evidence.contains(relative), "evidence map omits {relative}");
    }
}
