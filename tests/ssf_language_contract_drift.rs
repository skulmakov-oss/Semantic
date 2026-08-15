use std::{fs, path::Path};

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

/// Extracts the value of a `Label: \`value\`` field. Used to cross-check that
/// the normative profile and its evidence map agree on the contract
/// identifier, instead of independently pinning the same literal in two
/// places (which let them silently drift during SSF-07's 1.0 -> 1.1 -> 1.2
/// bumps).
fn extract_backtick_field<'a>(doc: &'a str, label: &str) -> &'a str {
    let marker = format!("{label}: `");
    let start = doc
        .find(&marker)
        .unwrap_or_else(|| panic!("missing '{label}:' field"))
        + marker.len();
    let rest = &doc[start..];
    let end = rest
        .find('`')
        .unwrap_or_else(|| panic!("unterminated '{label}:' field"));
    &rest[..end]
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
        "semantic.foundation.source/1.2",
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

    let profile_contract_id = extract_backtick_field(&profile, "Contract identifier");
    let evidence_contract_id = extract_backtick_field(&evidence, "Contract identifier");
    assert_eq!(
        profile_contract_id, evidence_contract_id,
        "the normative profile and its SSF-01 evidence map disagree on the \
         contract identifier -- one was bumped without the other"
    );

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

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every current-facing doc (as opposed to a frozen historical decision
/// record) that names the Foundation Source contract identifier or the
/// `match` scrutinee allowlist must agree with the normative profile. This
/// guard exists because review repeatedly found one more doc pinned to a
/// superseded contract version or a stale allowlist after the previous one
/// was fixed (SSF-07, PR #1615, rounds 12/21). Deliberately bounded to a
/// fixed file list and a handful of literal substrings, not a general prose
/// parser, per this repo's drift-guard discipline.
#[test]
fn ssf_07_pattern_surface_authorities_stay_in_sync() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = read(root, "docs/spec/foundation_source_profile_v1.md");
    let live_contract_id = extract_backtick_field(&profile, "Contract identifier");

    // Every current-facing authority that names the contract identifier at
    // all must name the live one -- never a superseded "semantic.foundation.
    // source/<older>" or "Foundation Source <older>" spelling.
    let current_facing_docs = [
        "README.md",
        "docs/spec/foundation_source_profile_v1.md",
        "docs/spec/syntax.md",
        "docs/spec/types.md",
        "docs/spec/source_semantics.md",
        "docs/spec/source_style.md",
        "docs/spec/diagnostics.md",
        "docs/spec/foundation_stdlib_v0.md",
        "docs/spec/controlled_application_boundary_v0.md",
        "docs/spec/logos.md",
        "docs/roadmap/stable_foundation/stable_public_language_contract.md",
        "docs/roadmap/stable_foundation/rustlike_logos_coherence_decision.md",
        "docs/roadmap/stable_foundation/standard_library_v0_evidence.md",
        "examples/canonical/stdlib_v0_helpers/README.md",
    ];
    let stale_identifier_markers = [
        "semantic.foundation.source/1.0",
        "semantic.foundation.source/1.1",
        "Foundation Source 1.0",
        "Foundation Source 1.1",
        "Foundation Source Profile 1.0",
        "Foundation Source Profile 1.1",
    ];
    for relative in current_facing_docs {
        let text = read(root, relative);
        for marker in stale_identifier_markers {
            assert!(
                !text.contains(marker),
                "{relative} still contains the superseded marker '{marker}'; \
                 current-facing docs must track the live contract identifier \
                 ({live_contract_id})"
            );
        }
    }

    // Every doc that documents the `match` scrutinee allowlist must list all
    // six admitted families, not a stale subset that predates a family being
    // added (e.g. i32/u32 were added to this allowlist after syntax.md,
    // types.md, source_semantics.md, and source_style.md already existed).
    let match_allowlist_docs = [
        "docs/spec/types.md",
        "docs/spec/syntax.md",
        "docs/spec/source_semantics.md",
        "docs/spec/source_style.md",
    ];
    for relative in match_allowlist_docs {
        let text = read(root, relative);
        for family in ["quad", "enum", "Option", "Result", "i32", "u32"] {
            assert!(
                text.contains(family),
                "{relative} match-scrutinee-allowlist coverage is missing \
                 '{family}'"
            );
        }
    }

    // u32 match became fully Included (not typecheck-only) as of Foundation
    // Source 1.2 (SSF-07). No current-facing doc may still describe it as
    // typecheck-only or as failing to execute correctly.
    let stale_u32_markers = [
        "u32 is typecheck-only",
        "u32` is typecheck-only",
        "does not execute correctly",
    ];
    for relative in current_facing_docs {
        let text = normalize_whitespace(&read(root, relative));
        for marker in stale_u32_markers {
            assert!(
                !text.contains(marker),
                "{relative} still describes u32 match with the stale marker \
                 '{marker}', but u32 match is fully Included as of Foundation \
                 Source 1.2"
            );
        }
    }
}
