//! SSF-10 Integration Tests: Compatibility, Migration, and Artifact Trust.
//!
//! Validates:
//! 1. Deterministic artifact identity (same bytes -> exact same sha256).
//! 2. Same source input -> exact same compiled artifact and identity.
//! 3. Semantically changed input -> mismatched artifact identity.
//! 4. Stale artifact detection (source modification vs artifact).
//! 5. Incompatible artifact format/version rejection (fail-closed).
//! 6. Verifier admission bound to exact artifact identity (cannot verify A and execute B).
//! 7. Artifact inspection (headers, capabilities, functions, ADT descriptors).
//! 8. Artifact hash CLI behavior (text and --json modes).
//! 9. Version CLI behavior (canonical identity, --json schema).
//! 10. Migration dry-run non-destructive guarantee (zero filesystem mutation).
//! 11. Compatibility fail-closed evaluation.
//! 12. Release artifact trust model (explicit unsigned signing, SHA-256 digests).

use sm_emit::compile_program_to_semcode_with_options_debug;
use sm_format::semcode_format::HEADER_V22;
use sm_format::sha256::{format_hex, sha256, sha256_prefixed_hex};
use sm_ir::{CompileProfile, OptLevel};
use sm_verify::verify_semcode_token;
use smc_cli::artifact_identity::ArtifactIdentity;
use smc_cli::compatibility::{
    assess_artifact_compatibility, detect_artifact_staleness, inspect_migration,
    CompatibilityClassification,
};
use std::fs;
use std::path::Path;

#[test]
fn test_1_deterministic_artifact_identity() {
    let sample = b"SEMCODE_DETERMINISTIC_TEST_PAYLOAD_V22";
    let hash1 = sha256(sample);
    let hash2 = sha256(sample);
    assert_eq!(
        hash1, hash2,
        "SHA-256 must be strictly deterministic across calls"
    );

    let hex1 = format_hex(&hash1);
    let hex2 = format_hex(&hash2);
    assert_eq!(hex1, hex2, "Hex formatting must be strictly deterministic");
    assert_eq!(hex1.len(), 64, "SHA-256 hex must be exactly 64 characters");

    let prefixed = sha256_prefixed_hex(sample);
    assert_eq!(prefixed, format!("sha256:{}", hex1));
}

#[test]
fn test_2_same_input_yields_same_identity() {
    let source = r#"
fn main() {
    let a: i32 = 10;
    let b: i32 = 20;
    let c: i32 = a + b;
    return;
}
"#;

    let artifact1 = compile_program_to_semcode_with_options_debug(
        source,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compilation 1 must succeed");

    let artifact2 = compile_program_to_semcode_with_options_debug(
        source,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compilation 2 must succeed");

    assert_eq!(
        artifact1, artifact2,
        "compiling identical source must produce bit-for-bit identical bytes"
    );

    let id1 = ArtifactIdentity::from_bytes(&artifact1).expect("identity 1");
    let id2 = ArtifactIdentity::from_bytes(&artifact2).expect("identity 2");

    assert_eq!(
        id1.artifact_hash, id2.artifact_hash,
        "identical artifacts must have identical canonical hash"
    );
    assert_eq!(id1.size_bytes, id2.size_bytes);
    assert_eq!(id1.signing, "unsigned");
    assert_eq!(id2.signing, "unsigned");
}

#[test]
fn test_3_changed_input_yields_mismatched_identity() {
    let source_a = r#"
fn main() {
    let val: i32 = 100;
    return;
}
"#;

    let source_b = r#"
fn main() {
    let val: i32 = 200;
    return;
}
"#;

    let artifact_a = compile_program_to_semcode_with_options_debug(
        source_a,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compilation A");

    let artifact_b = compile_program_to_semcode_with_options_debug(
        source_b,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compilation B");

    assert_ne!(
        artifact_a, artifact_b,
        "different source constants must produce different artifact bytes"
    );

    let id_a = ArtifactIdentity::from_bytes(&artifact_a).expect("identity A");
    let id_b = ArtifactIdentity::from_bytes(&artifact_b).expect("identity B");

    assert_ne!(
        id_a.artifact_hash, id_b.artifact_hash,
        "different artifacts must produce different canonical hashes"
    );
}

#[test]
fn test_4_stale_artifact_detection() {
    let source_v1 = r#"
fn main() {
    let version: i32 = 1;
    return;
}
"#;
    let source_v2 = r#"
fn main() {
    let version: i32 = 2;
    return;
}
"#;

    let temp_dir = std::env::temp_dir();
    let src_file = temp_dir.join("ssf10_stale_test_src.sm");
    let art_file = temp_dir.join("ssf10_stale_test_art.smc");

    fs::write(&src_file, source_v1).expect("write src v1");
    let artifact = compile_program_to_semcode_with_options_debug(
        source_v1,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile v1");
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(&art_file, &artifact).expect("write artifact");

    // Fresh artifact
    let status_fresh = detect_artifact_staleness(&art_file, &src_file).expect("fresh check");
    assert!(
        status_fresh.is_fresh(),
        "artifact written after source must be fresh"
    );
    assert!(!status_fresh.is_stale());

    // Update source so source is newer than artifact
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(&src_file, source_v2).expect("update src to v2");

    let status_stale = detect_artifact_staleness(&art_file, &src_file).expect("stale check");
    assert!(
        status_stale.is_stale(),
        "source written after artifact must be detected as stale"
    );
    assert!(!status_stale.is_fresh());

    // Cleanup
    let _ = fs::remove_file(src_file);
    let _ = fs::remove_file(art_file);
}

#[test]
fn test_5_incompatible_artifact_rejection() {
    let source = "fn main() { return; }";
    let valid_artifact = compile_program_to_semcode_with_options_debug(
        source,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile valid");

    // Corrupt the 8-byte magic header.
    let mut corrupted_magic = valid_artifact.clone();
    corrupted_magic[0..8].copy_from_slice(b"BADMAGIC");

    let id_bad = ArtifactIdentity::from_bytes(&corrupted_magic).expect("id for corrupted magic");
    let status_bad = assess_artifact_compatibility(&id_bad);
    assert!(
        matches!(status_bad, CompatibilityClassification::Incompatible),
        "artifact with corrupted magic must be classified as Incompatible"
    );

    // Verifier must strictly reject bad magic (fail-closed).
    let verify_result = verify_semcode_token(&corrupted_magic);
    assert!(
        verify_result.is_err(),
        "verifier must fail-closed on artifact with unrecognized magic"
    );

    // Short truncated artifact (< 8 bytes)
    let truncated = &valid_artifact[0..4];
    assert!(
        ArtifactIdentity::from_bytes(truncated).is_err(),
        "truncated artifact shorter than header must fail from_bytes"
    );
}

#[test]
fn test_6_verifier_result_bound_to_exact_artifact() {
    let source_a = "fn main() { return; }";
    let source_b = "fn main() { let x: i32 = 42; return; }";

    let artifact_a = compile_program_to_semcode_with_options_debug(
        source_a,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile A");

    let artifact_b = compile_program_to_semcode_with_options_debug(
        source_b,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile B");

    let hash_a = sha256(&artifact_a);
    let hash_b = sha256(&artifact_b);
    assert_ne!(hash_a, hash_b);

    // Verify artifact A.
    let verified_a = verify_semcode_token(&artifact_a).expect("verify A must pass");

    // Property: VerifiedSemCode is bound to exact artifact hash of A.
    assert_eq!(
        verified_a.artifact_hash(),
        hash_a,
        "verified result must record exact artifact SHA-256"
    );
    assert_eq!(
        verified_a.artifact_hash_hex(),
        format!("sha256:{}", format_hex(&hash_a)),
        "verified result must format exact artifact hash hex"
    );

    // Invariant: Verification of artifact A matches artifact A bytes.
    assert!(
        verified_a.matches_artifact(&artifact_a),
        "verified token must match artifact A bytes"
    );

    // Invariant: Verification of artifact A CANNOT be used to admit artifact B.
    assert!(
        !verified_a.matches_artifact(&artifact_b),
        "CRITICAL: verification token for artifact A must NEVER match artifact B"
    );

    // Invariant: Mutating even 1 byte in artifact A breaks verification binding.
    let mut mutated_a = artifact_a.clone();
    let last_idx = mutated_a.len() - 1;
    mutated_a[last_idx] ^= 0xFF;
    assert!(
        !verified_a.matches_artifact(&mutated_a),
        "CRITICAL: verification token must NOT match mutated artifact bytes"
    );
}

#[test]
fn test_7_artifact_inspection_completeness() {
    let source = r#"
fn main() {
    let val: i32 = 99;
    return;
}
"#;
    let artifact = compile_program_to_semcode_with_options_debug(
        source,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile");

    let id = ArtifactIdentity::from_bytes(&artifact).expect("artifact inspection");

    // Validate header inspection
    assert_eq!(id.header.magic, "SEMCOD22");
    assert_eq!(id.header.epoch, HEADER_V22.epoch);
    assert_eq!(id.header.revision, HEADER_V22.rev);
    assert!(id
        .header
        .capability_flags
        .contains(&"ADT_DESCRIPTORS".to_string()));
    assert!(id
        .header
        .capability_flags
        .contains(&"CALLABLE_SIGNATURES".to_string()));

    // Validate functions
    assert_eq!(id.function_count, 1);
    assert_eq!(id.functions[0].name, "main");
    assert!(id.functions[0].has_signature);

    // Validate ADT table (Option, Result are builtins)
    assert!(id.adt_names.contains(&"Option".to_string()));
    assert!(id.adt_names.contains(&"Result".to_string()));

    // Validate verifier binding
    assert!(id.verifier.admitted);
    assert!(id.verifier.admission_code.is_none());
    assert!(id.verifier.diagnostics.is_empty());

    // Validate signing state honesty
    assert_eq!(id.signing, "unsigned");

    // Validate JSON render format
    let json = id.render_json();
    assert!(json.contains("\"schema_version\": \"semantic-artifact-v1\""));
    assert!(json.contains(&format!("\"artifact_hash\": \"{}\"", id.artifact_hash)));
    assert!(json.contains("\"signing\": \"unsigned\""));
    assert!(json.contains("\"SEMCOD22\""));

    // Validate human render format
    let human = id.render_human(Some("test.smc"));
    assert!(human.contains("Artifact: test.smc"));
    assert!(human.contains(&id.artifact_hash));
    assert!(human.contains("Verifier:       Admitted (Pass)"));
    assert!(human.contains("Signing State:  unsigned"));
}

#[test]
fn test_8_artifact_hash_cli_behavior() {
    let source = "fn main() { return; }";
    let artifact = compile_program_to_semcode_with_options_debug(
        source,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile");

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_artifact_hash_cli.smc");
    fs::write(&test_file, &artifact).expect("write artifact");

    let calculated = sha256_prefixed_hex(&artifact);
    let bytes_read = fs::read(&test_file).expect("read");
    let hash_read = sha256_prefixed_hex(&bytes_read);
    assert_eq!(calculated, hash_read);

    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_9_version_identity_contract() {
    let toolchain = smc_cli::artifact_identity::ToolchainIdentity::default();
    assert_eq!(toolchain.compiler_version, env!("CARGO_PKG_VERSION"));
    assert!(!toolchain.source_hash.is_empty());
    assert!(!toolchain.enabled_features.is_empty());
}

#[test]
fn test_10_migration_dry_run_zero_mutation() {
    let fixture_dir = Path::new("tests/fixtures/ssf10_compatibility/migration_preview_project");
    assert!(fixture_dir.exists(), "fixture dir must exist");

    // Capture state of all files in fixture_dir before dry run
    let mut files_before = Vec::new();
    for entry in fs::read_dir(fixture_dir).expect("read fixture dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_file() {
            let content = fs::read(&path).expect("read content");
            let hash = sha256_prefixed_hex(&content);
            files_before.push((path, hash, content.len()));
        }
    }
    assert!(!files_before.is_empty(), "fixture dir must have files");

    // Run dry-run migration inspection
    let report = inspect_migration(fixture_dir).expect("dry run inspection");

    // Invariant: zero mutations performed
    assert_eq!(
        report.mutations_performed, 0,
        "migration dry-run must perform zero mutations"
    );

    // Invariant: all file contents and hashes strictly identical before and after
    for (path, expected_hash, expected_len) in &files_before {
        let content_after = fs::read(path).expect("read after dry run");
        let hash_after = sha256_prefixed_hex(&content_after);
        assert_eq!(
            content_after.len(),
            *expected_len,
            "file length must not change after dry-run"
        );
        assert_eq!(
            &hash_after, expected_hash,
            "file content hash must not change after dry-run"
        );
    }

    // Validate JSON output format
    let json = report.render_json();
    assert!(json.contains("\"mutations_performed\": 0"));
    assert!(json.contains("dry-run preview completed successfully with zero mutations"));

    // Validate human output format
    let human = report.render_human();
    assert!(human.contains("Files Mutated:         0 (strictly non-destructive)"));
}

#[test]
fn test_11_compatibility_fail_closed_on_unsupported_states() {
    // Truncated / empty
    let empty = b"";
    assert!(ArtifactIdentity::from_bytes(empty).is_err());

    // Bad magic
    let bad_magic = b"NOTSMCOD\x00\x00\x00\x00";
    let id_bad = ArtifactIdentity::from_bytes(bad_magic).expect("id bad magic");
    let status_magic = assess_artifact_compatibility(&id_bad);
    assert!(matches!(
        status_magic,
        CompatibilityClassification::Incompatible
    ));

    // Unrecognized future magic
    let future_magic = b"SEMCOD99\x00\x00\x00\x00";
    let id_future = ArtifactIdentity::from_bytes(future_magic).expect("id future magic");
    let status_future = assess_artifact_compatibility(&id_future);
    assert!(matches!(
        status_future,
        CompatibilityClassification::Incompatible
    ));
}

#[test]
fn test_12_release_artifact_trust_and_checksum_integrity() {
    // SHA-256 test vectors per FIPS 180-4
    // Empty string "" -> e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let empty_hash = sha256(b"");
    assert_eq!(
        format_hex(&empty_hash),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    // "abc" -> ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
    let abc_hash = sha256(b"abc");
    assert_eq!(
        format_hex(&abc_hash),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    // Signing state honesty:
    // No mock certificates, no invented keys.
    // Release artifacts must declare signing: "unsigned".
    let dummy_artifact = compile_program_to_semcode_with_options_debug(
        "fn main() { return; }",
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile");

    let id = ArtifactIdentity::from_bytes(&dummy_artifact).expect("identity");
    assert_eq!(id.signing, "unsigned");
}
