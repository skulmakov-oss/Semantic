use std::{fs, path::Path};

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

/// Collapses newlines/indentation to single spaces so prose assertions don't depend on
/// exact markdown line wrapping.
fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

const MAP_PROGRAM: &str = r#"
fn main() {
    let m: Map(i32, i32) = map_empty();
    let m2: Map(i32, i32) = map_set(m, 1, 2);
    assert(map_contains(m2, 1) == true);
    assert(map_get(m2, 1, 0) == 2);
    return;
}
"#;

#[test]
fn map_program_selects_the_semcod14_header() {
    let bytes = sm_emit::compile_program_to_semcode(MAP_PROGRAM).expect("compile Map program");
    // #1773 (FA-09-005): SEMCOD19 is now the floor for every compiled
    // artifact regardless of which opcodes it uses (was SEMCOD14, Map's own
    // promotion floor - Map still needs at least that revision internally,
    // this just isn't independently observable from `compile_program_to_semcode`'s
    // header output anymore).
    assert_eq!(
        &bytes[..8],
        b"SEMCOD19",
        "a program that actually uses Map(K, V) must select a header at or above SEMCOD14"
    );
}

#[test]
fn map_program_is_verifier_admitted_and_vm_executed() {
    let bytes = sm_emit::compile_program_to_semcode(MAP_PROGRAM).expect("compile Map program");
    sm_vm::run_verified_semcode(&bytes)
        .expect("verifier must admit and VM must execute the SEMCOD14 artifact");
}

#[test]
fn foundation_source_profile_and_semcode_spec_document_semcod14_for_map() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let profile = normalize(&read(root, "docs/spec/foundation_source_profile_v1.md"));
    assert!(
        profile.contains("SEMCODE0` through `SEMCOD14`"),
        "foundation_source_profile_v1.md must document SEMCOD14 as the supported cap now that Map(K, V) is included"
    );
    assert!(
        !profile.contains("SEMCODE0` through `SEMCOD13`"),
        "foundation_source_profile_v1.md must not still cap the supported family at SEMCOD13"
    );

    let semcode_spec = read(root, "docs/spec/semcode.md");
    assert!(
        semcode_spec.contains("`SEMCOD14`"),
        "semcode.md must list SEMCOD14 in the versioned header family"
    );
    assert!(
        semcode_spec.contains("CAP_MAP_VALUES"),
        "semcode.md must document CAP_MAP_VALUES as a canonical capability family"
    );

    // stable_public_language_contract.md is an SSF-01 evidence/closure record, but its pinned
    // base commit postdates SEMCOD14's introduction, so it must not undercount the header cap.
    let contract = normalize(&read(
        root,
        "docs/roadmap/stable_foundation/stable_public_language_contract.md",
    ));
    assert!(
        !contract.contains("SEMCODE0` through `SEMCOD13`"),
        "stable_public_language_contract.md must not still cap the supported family at SEMCOD13"
    );
}
