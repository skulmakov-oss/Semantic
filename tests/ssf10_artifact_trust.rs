//! SSF-10 Integration Tests: Compatibility, Migration, and Artifact Trust.
//!
//! Validates:
//! 1. Deterministic artifact identity (SHA-256 digest following FIPS PUB 180-4).
//! 2. Same source input -> exact same compiled artifact and identity.
//! 3. Semantically changed input -> mismatched artifact identity.
//! 4. Digest-based stale and mismatch detection (immune to mtime spoofing/backdating).
//! 5. Incompatible artifact format/version rejection (fail-closed).
//! 6. Verifier admission bound to exact artifact identity (cannot verify A and execute B).
//! 7. Artifact inspection distinguishing producer provenance from inspecting toolchain.
//! 8. Real CLI execution: `smc artifact hash` and `smc artifact inspect` (text & --json).
//! 9. Real CLI execution: `smc version` (text & --json).
//! 10. Real CLI execution: `smc migrate check|preview` with guaranteed zero mutation.
//! 11. Explicit compatibility policies (source, manifest, diagnostics, stdlib, runtime PRNG, SemCode).
//! 12. Release artifact trust model and toolchain binding.

use sm_emit::compile_program_to_semcode_with_options_debug;
use sm_format::sha256::{format_hex, sha256, sha256_prefixed_hex};
use sm_ir::{CompileProfile, OptLevel};
use sm_verify::verify_semcode_token;
use smc_cli::artifact_identity::{
    generate_and_save_companion_provenance, ArtifactIdentity, ProvenanceStatus,
};
use smc_cli::compatibility::{
    detect_artifact_staleness, CompatibilityClassification, DiagnosticCompatibilityPolicy,
    ManifestCompatibilityPolicy, RuntimeCompatibilityPolicy, SourceCompatibilityPolicy,
    StalenessStatus, StdlibCompatibilityPolicy,
};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Execute the real compiled `smc` binary via `CARGO_BIN_EXE_smc`.
fn run_smc(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("failed to execute CARGO_BIN_EXE_smc binary");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

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
fn test_4_digest_based_staleness_and_mtime_spoof_detection() {
    let source_v1 = "fn main() { let v: i32 = 1; return; }";
    let source_v2 = "fn main() { let v: i32 = 2; return; }";

    let temp_dir = std::env::temp_dir();
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let src_file = temp_dir.join(format!("ssf10_stale_src_{}.sm", unique_id));
    let art_file = temp_dir.join(format!("ssf10_stale_art_{}.smc", unique_id));

    fs::write(&src_file, source_v1).expect("write src v1");
    let artifact = compile_program_to_semcode_with_options_debug(
        source_v1,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile v1");
    fs::write(&art_file, &artifact).expect("write artifact");

    // 1. Without companion provenance: fail closed as MissingProvenance
    let status_no_prov = detect_artifact_staleness(&art_file, &src_file).expect("staleness check");
    assert!(
        matches!(status_no_prov, StalenessStatus::MissingProvenance(_)),
        "without provenance, staleness check must fail-closed as MissingProvenance"
    );

    // 2. Generate and attach canonical companion provenance
    let prov_path = generate_and_save_companion_provenance(
        &art_file,
        &artifact,
        &src_file,
        Some("test_package".to_string()),
        Some("0.1.0".to_string()),
    )
    .expect("generate provenance");
    assert!(prov_path.is_file(), "companion provenance file must exist");

    // 3. Fresh check with provenance
    let status_fresh = detect_artifact_staleness(&art_file, &src_file).expect("fresh check");
    assert!(
        status_fresh.is_fresh(),
        "matching source and artifact with provenance must be Fresh"
    );

    // 4. MTIME SPOOFING TEST:
    // Update source content to v2, but keep artifact mtime newer or equal (simulating backdated/preserved mtime)
    fs::write(&src_file, source_v2).expect("write src v2");

    // Even if filesystem mtime is not inspected or if source is newer,
    // the canonical digest check MUST detect the content change!
    let status_stale = detect_artifact_staleness(&art_file, &src_file).expect("stale check");
    assert!(
        matches!(status_stale, StalenessStatus::StaleSourceChanged { .. }),
        "digest mismatch must report StaleSourceChanged regardless of filesystem mtime"
    );
    assert!(!status_stale.is_fresh());

    // Cleanup
    let _ = fs::remove_file(src_file);
    let _ = fs::remove_file(art_file);
    let _ = fs::remove_file(prov_path);
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

    assert!(verify_semcode_token(&valid_artifact).is_ok());

    // Corrupted magic
    let mut corrupt_magic = valid_artifact.clone();
    corrupt_magic[0..8].copy_from_slice(b"CORRUPT!");
    assert!(
        verify_semcode_token(&corrupt_magic).is_err(),
        "invalid header magic must fail closed"
    );

    // Corrupted payload
    let mut corrupt_payload = valid_artifact.clone();
    let last = corrupt_payload.len() - 1;
    corrupt_payload[last] ^= 0xFF;
    assert!(
        verify_semcode_token(&corrupt_payload).is_err(),
        "corrupted payload must fail verifier admission"
    );
}

#[test]
fn test_6_verifier_result_bound_to_exact_artifact() {
    let src1 = "fn main() { let x: i32 = 1; return; }";
    let src2 = "fn main() { let x: i32 = 2; return; }";

    let art1 = compile_program_to_semcode_with_options_debug(
        src1,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile 1");
    let art2 = compile_program_to_semcode_with_options_debug(
        src2,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile 2");

    let token1 = verify_semcode_token(&art1).expect("verify 1");
    let token2 = verify_semcode_token(&art2).expect("verify 2");

    assert!(
        token1.matches_artifact(&art1),
        "verifier token 1 must match exact artifact 1"
    );
    assert!(
        !token1.matches_artifact(&art2),
        "verifier token 1 MUST NOT match artifact 2"
    );

    assert!(
        token2.matches_artifact(&art2),
        "verifier token 2 must match exact artifact 2"
    );
    assert!(
        !token2.matches_artifact(&art1),
        "verifier token 2 MUST NOT match artifact 1"
    );

    assert_eq!(token1.artifact_hash_hex(), sha256_prefixed_hex(&art1));
    assert_eq!(token2.artifact_hash_hex(), sha256_prefixed_hex(&art2));
}

#[test]
fn test_7_artifact_inspection_producer_vs_inspector_distinction() {
    let source = "fn main() { let a: i32 = 42; return; }";
    let artifact = compile_program_to_semcode_with_options_debug(
        source,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile");

    let temp_dir = std::env::temp_dir();
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let src_path = temp_dir.join(format!("inspect_test_src_{}.sm", unique_id));
    let art_path = temp_dir.join(format!("inspect_test_art_{}.smc", unique_id));

    fs::write(&src_path, source).expect("write src");
    fs::write(&art_path, &artifact).expect("write art");

    // 1. Without provenance: producer must be reported as unrecorded
    let identity_no_prov = ArtifactIdentity::from_file(&art_path).expect("inspect without prov");
    assert!(identity_no_prov.producer_provenance.is_none());
    assert_eq!(
        identity_no_prov.provenance_status,
        ProvenanceStatus::Missing
    );

    let human_no_prov = identity_no_prov.render_human(Some("inspect_test.smc"));
    assert!(
        human_no_prov.contains("[UNRECORDED - NO PROVENANCE ATTACHED]"),
        "human output must explicitly state unrecorded producer when provenance is missing"
    );
    assert!(human_no_prov.contains("Inspecting Toolchain"));

    let json_no_prov = identity_no_prov.render_json();
    assert!(
        json_no_prov.contains("\"producer_provenance\""),
        "JSON must contain producer_provenance object"
    );
    assert!(json_no_prov.contains("\"status\": \"missing\""));

    // 2. With companion provenance attached:
    let prov_path = generate_and_save_companion_provenance(
        &art_path,
        &artifact,
        &src_path,
        Some("inspect_pkg".to_string()),
        Some("1.2.3".to_string()),
    )
    .expect("generate provenance");

    let identity_with_prov = ArtifactIdentity::from_file(&art_path).expect("inspect with prov");
    assert!(identity_with_prov.producer_provenance.is_some());
    assert!(matches!(
        identity_with_prov.provenance_status,
        ProvenanceStatus::Recorded(_)
    ));

    let human_with_prov = identity_with_prov.render_human(Some("inspect_test.smc"));
    assert!(human_with_prov.contains("Producer Toolchain (from Provenance)"));
    assert!(human_with_prov.contains("Inspecting Toolchain (Current Process)"));
    assert!(human_with_prov.contains("Package:        inspect_pkg v1.2.3"));

    let json_with_prov = identity_with_prov.render_json();
    assert!(json_with_prov.contains("\"status\": \"recorded\""));
    assert!(json_with_prov.contains("\"package_name\": \"inspect_pkg\""));
    assert!(json_with_prov.contains("\"inspecting_toolchain\""));

    // Cleanup
    let _ = fs::remove_file(src_path);
    let _ = fs::remove_file(art_path);
    let _ = fs::remove_file(prov_path);
}

#[test]
fn test_8_cli_artifact_hash_and_inspect_real_binary() {
    let source = "fn main() { return; }";
    let artifact = compile_program_to_semcode_with_options_debug(
        source,
        CompileProfile::Auto,
        OptLevel::O0,
        false,
    )
    .expect("compile");

    let temp_dir = std::env::temp_dir();
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let art_path = temp_dir.join(format!("cli_test_art_{}.smc", unique_id));
    fs::write(&art_path, &artifact).expect("write artifact");

    let art_path_str = art_path.to_str().unwrap();

    // 1. `smc artifact hash` real binary invocation
    let (code_hash, stdout_hash, stderr_hash) = run_smc(&["artifact", "hash", art_path_str]);
    assert_eq!(
        code_hash, 0,
        "artifact hash must exit with 0. stderr: {}",
        stderr_hash
    );
    assert_eq!(stdout_hash.trim(), sha256_prefixed_hex(&artifact));

    // 2. `smc artifact hash --json` real binary invocation
    let (code_hash_json, stdout_hash_json, _) =
        run_smc(&["artifact", "hash", art_path_str, "--json"]);
    assert_eq!(code_hash_json, 0);
    let hash_parsed: serde_json::Value = serde_json::from_str(&stdout_hash_json)
        .expect("artifact hash --json must produce valid JSON");
    assert_eq!(
        hash_parsed["artifact_hash"].as_str().unwrap(),
        sha256_prefixed_hex(&artifact)
    );

    // 3. `smc artifact inspect` real binary invocation
    let (code_insp, stdout_insp, stderr_insp) = run_smc(&["artifact", "inspect", art_path_str]);
    assert_eq!(
        code_insp, 0,
        "artifact inspect must exit with 0. stderr: {}",
        stderr_insp
    );
    assert!(stdout_insp.contains("Canonical Hash: sha256:"));
    assert!(stdout_insp.contains("Producer Toolchain"));
    assert!(stdout_insp.contains("Inspecting Toolchain"));

    // 4. `smc artifact inspect --json` real binary invocation
    let (code_insp_json, stdout_insp_json, _) =
        run_smc(&["artifact", "inspect", art_path_str, "--json"]);
    assert_eq!(code_insp_json, 0);
    let insp_parsed: serde_json::Value = serde_json::from_str(&stdout_insp_json)
        .expect("artifact inspect --json must produce valid JSON");
    assert_eq!(insp_parsed["schema_version"], "semantic-artifact-v1");
    assert!(insp_parsed["producer_provenance"].is_object());
    assert!(insp_parsed["inspecting_toolchain"].is_object());

    let _ = fs::remove_file(art_path);
}

#[test]
fn test_9_cli_version_real_binary() {
    // 1. `smc version` real binary invocation
    let (code, stdout, stderr) = run_smc(&["version"]);
    assert_eq!(code, 0, "smc version must exit with 0. stderr: {}", stderr);
    assert!(stdout.contains("Semantic Language Toolchain v0.1.0"));
    assert!(stdout.contains("Source Fingerprint:"));
    assert!(stdout.contains("Enabled Features:"));

    // 2. `smc version --json` real binary invocation
    let (code_json, stdout_json, _) = run_smc(&["version", "--json"]);
    assert_eq!(code_json, 0);
    let ver_parsed: serde_json::Value =
        serde_json::from_str(&stdout_json).expect("version --json must produce valid JSON");
    assert_eq!(ver_parsed["schema_version"], "semantic-version-v1");
    assert_eq!(ver_parsed["toolchain_version"], "0.1.0");
    assert!(ver_parsed["source_fingerprint"].is_string());
    assert_eq!(ver_parsed["verifier_profile"], "verifier-canonical-v1");
    assert_eq!(ver_parsed["signing"], "unsigned");
}

#[test]
fn test_10_cli_migrate_preview_real_binary_zero_mutation() {
    let fixture_dir = Path::new("tests/fixtures/ssf10_compatibility/migration_preview_project");
    assert!(fixture_dir.exists(), "migration fixture must exist");

    // Capture file contents and lengths before dry-run
    let main_sm = fixture_dir.join("main.sm");
    let toml = fixture_dir.join("semantic.toml");
    let main_before = fs::read(&main_sm).expect("read main before");
    let toml_before = fs::read(&toml).expect("read toml before");

    let fixture_path_str = fixture_dir.to_str().unwrap();

    // 1. `smc migrate check` real binary invocation
    let (code_check, stdout_check, stderr_check) = run_smc(&["migrate", "check", fixture_path_str]);
    assert_eq!(
        code_check, 0,
        "migrate check must exit with 0. stderr: {}",
        stderr_check
    );
    assert!(stdout_check.contains("Migration Dry-Run Inspection"));

    // 2. `smc migrate preview --dry-run` real binary invocation
    let (code_preview, stdout_preview, stderr_preview) =
        run_smc(&["migrate", "preview", fixture_path_str, "--dry-run"]);
    assert_eq!(
        code_preview, 0,
        "migrate preview must exit with 0. stderr: {}",
        stderr_preview
    );
    assert!(stdout_preview.contains("Files Mutated:         0 (strictly non-destructive)"));

    // 3. `smc migrate preview --json` real binary invocation
    let (code_json, stdout_json, _) = run_smc(&[
        "migrate",
        "preview",
        fixture_path_str,
        "--json",
        "--dry-run",
    ]);
    assert_eq!(code_json, 0);
    let mig_parsed: serde_json::Value =
        serde_json::from_str(&stdout_json).expect("migrate preview --json must produce valid JSON");
    assert_eq!(mig_parsed["mutations_performed"], 0);

    // STRICT MUTATION VERIFICATION: verify files on disk were not touched
    let main_after = fs::read(&main_sm).expect("read main after");
    let toml_after = fs::read(&toml).expect("read toml after");
    assert_eq!(main_before, main_after, "dry-run must not mutate main.sm");
    assert_eq!(
        toml_before, toml_after,
        "dry-run must not mutate semantic.toml"
    );
}

#[test]
fn test_11_compatibility_policies_and_boundary_rejection() {
    // 1. Source Compatibility Policy
    assert_eq!(
        SourceCompatibilityPolicy::MIN_DEPRECATION_WINDOW_CYCLES,
        1,
        "deprecation window must be at least 1 cycle"
    );
    assert_eq!(
        SourceCompatibilityPolicy::assess_source_element(false, false),
        smc_cli::compatibility::CompatibilityClassification::Compatible
    );
    assert_eq!(
        SourceCompatibilityPolicy::assess_source_element(true, false),
        smc_cli::compatibility::CompatibilityClassification::Deprecated
    );
    assert_eq!(
        SourceCompatibilityPolicy::assess_source_element(false, true),
        smc_cli::compatibility::CompatibilityClassification::Incompatible
    );

    // 2. Manifest Compatibility Policy
    assert_eq!(
        ManifestCompatibilityPolicy::assess_manifest_schema(1),
        smc_cli::compatibility::CompatibilityClassification::Compatible
    );
    assert_eq!(
        ManifestCompatibilityPolicy::assess_manifest_schema(0),
        smc_cli::compatibility::CompatibilityClassification::Deprecated
    );
    assert_eq!(
        ManifestCompatibilityPolicy::assess_manifest_schema(99),
        smc_cli::compatibility::CompatibilityClassification::Unsupported
    );

    // 3. Diagnostic Compatibility Policy
    assert!(
        DiagnosticCompatibilityPolicy::is_breaking_diagnostic_change(true, false, false),
        "code change is breaking"
    );
    assert!(
        DiagnosticCompatibilityPolicy::is_breaking_diagnostic_change(false, true, false),
        "severity change is breaking"
    );
    assert!(
        DiagnosticCompatibilityPolicy::is_breaking_diagnostic_change(false, false, true),
        "structure change is breaking"
    );
    assert!(
        !DiagnosticCompatibilityPolicy::is_breaking_diagnostic_change(false, false, false),
        "identical contract is non-breaking"
    );

    // 4. Stdlib Compatibility Policy
    assert!(
        StdlibCompatibilityPolicy::is_breaking_stdlib_change(true, false),
        "signature change is breaking"
    );
    assert!(
        StdlibCompatibilityPolicy::is_breaking_stdlib_change(false, true),
        "capability widening is breaking"
    );
    assert!(
        !StdlibCompatibilityPolicy::is_breaking_stdlib_change(false, false),
        "pure additive non-conflicting builtins are non-breaking"
    );

    // 5. Runtime Compatibility Policy & Deterministic PRNG
    let seq1 = RuntimeCompatibilityPolicy::verify_deterministic_prng_seed(0x1337_CAFE, 100);
    let seq2 = RuntimeCompatibilityPolicy::verify_deterministic_prng_seed(0x1337_CAFE, 100);
    assert_eq!(
        seq1, seq2,
        "runtime PRNG contract must produce bit-for-bit identical sequences"
    );

    let seq_other = RuntimeCompatibilityPolicy::verify_deterministic_prng_seed(0xDEAD_BEEF, 100);
    assert_ne!(
        seq1, seq_other,
        "different seeds must produce distinct sequences"
    );
}

#[test]
fn test_12_release_artifact_trust_and_toolchain_binding() {
    // 1. Standard SHA-256 test vectors following FIPS PUB 180-4 algorithm
    let empty_digest = sha256(b"");
    assert_eq!(
        format_hex(&empty_digest),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    let abc_digest = sha256(b"abc");
    assert_eq!(
        format_hex(&abc_digest),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    // 2. Validate release asset verification script contract
    let script_path = Path::new("scripts/verify_release_assets.ps1");
    assert!(
        script_path.is_file(),
        "verify_release_assets.ps1 must exist"
    );
    let script_content = fs::read_to_string(script_path).expect("read script");

    // Script must enforce explicit unsigned state (no fake PKI claims)
    assert!(script_content.contains("signingState = \"unsigned\""));
    assert!(script_content.contains("digestAlgorithm = \"SHA-256\""));
    assert!(script_content.contains("release toolchain version"));
    assert!(script_content.contains("toolchainEvidence = [ordered]@{"));
}

/// Helper for creating directory symlink or junction across platforms.
fn create_dir_link(target: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_dir(target, link).is_ok() {
            return Ok(());
        }
        let status = std::process::Command::new("cmd")
            .args([
                "/c",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .output()?;
        if status.status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other(String::from_utf8_lossy(
                &status.stderr,
            )))
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }
}

#[test]
fn test_13_symlink_escaping_project_root_rejected() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test13_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    let outside_dir = temp_dir.join("outside");
    fs::create_dir_all(&project_dir).expect("create project dir");
    fs::create_dir_all(&outside_dir).expect("create outside dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"escaped_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    fs::write(project_dir.join("main.sm"), "fn main() { return; }\n").expect("write main.sm");
    fs::write(outside_dir.join("secret.sm"), "fn secret() { return; }\n").expect("write secret.sm");

    let link_path = project_dir.join("link_to_outside");
    if let Err(e) = create_dir_link(&outside_dir, &link_path) {
        eprintln!("Skipping symlink escaping test due to OS limitation: {e}");
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    let report_res = smc_cli::compatibility::inspect_migration(&project_dir);
    assert!(
        report_res.is_err(),
        "project traversal must fail-closed when symlink escapes project root"
    );
    let err_msg = report_res.unwrap_err();
    assert!(
        err_msg.contains("security violation") || err_msg.contains("escapes project root"),
        "error message must describe security violation: {err_msg}"
    );

    let _ = fs::remove_dir(&link_path).or_else(|_| fs::remove_file(&link_path));
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_14_recursive_symlink_cycle_rejected() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test14_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    let sub_dir = project_dir.join("sub");
    fs::create_dir_all(&sub_dir).expect("create sub dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"cycle_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    fs::write(project_dir.join("main.sm"), "fn main() { return; }\n").expect("write main.sm");

    let cycle_link = sub_dir.join("cycle");
    if let Err(e) = create_dir_link(&project_dir, &cycle_link) {
        eprintln!("Skipping cycle test due to OS limitation: {e}");
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    let report_res = smc_cli::compatibility::inspect_migration(&project_dir);
    assert!(
        report_res.is_err(),
        "project traversal must fail-closed on recursive directory cycles"
    );
    let err_msg = report_res.unwrap_err();
    assert!(
        err_msg.contains("security violation")
            && (err_msg.contains("cycle") || err_msg.contains("forbidden")),
        "error message must describe cycle violation: {err_msg}"
    );

    let _ = fs::remove_dir(&cycle_link).or_else(|_| fs::remove_file(&cycle_link));
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_15_nested_directory_remains_admitted() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test15_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    let nested_dir = project_dir.join("src").join("nested").join("sub");
    fs::create_dir_all(&nested_dir).expect("create nested dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"nested_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    fs::write(
        project_dir.join("src").join("main.sm"),
        "fn main() { return; }\n",
    )
    .expect("write main.sm");
    fs::write(
        nested_dir.join("helper.sm"),
        "fn helper() -> i32 { return 42; }\n",
    )
    .expect("write helper.sm");

    let report = smc_cli::compatibility::inspect_migration(&project_dir)
        .expect("migration inspection on nested directories must succeed");

    assert_eq!(
        report.classification,
        CompatibilityClassification::Compatible
    );
    let inspected_files: Vec<String> = report
        .inspected_files
        .iter()
        .map(|p| p.display().to_string().replace('\\', "/"))
        .collect();
    assert!(
        inspected_files.iter().any(|f| f.ends_with("src/main.sm")),
        "main.sm must be inspected"
    );
    assert!(
        inspected_files
            .iter()
            .any(|f| f.ends_with("src/nested/sub/helper.sm")),
        "nested helper.sm must be admitted and inspected"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_16_target_and_hidden_directories_excluded() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test16_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    let target_dir = project_dir.join("target");
    let hidden_dir = project_dir.join(".git_sub");
    fs::create_dir_all(&target_dir).expect("create target dir");
    fs::create_dir_all(&hidden_dir).expect("create hidden dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"exclusion_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    fs::write(project_dir.join("main.sm"), "fn main() { return; }\n").expect("write main.sm");

    // Intentionally invalid syntax in excluded directories to prove they are never parsed or admitted
    fs::write(
        target_dir.join("junk.sm"),
        "THIS IS COMPLETELY INVALID SYNTAX !@#$%^&*()\n",
    )
    .expect("write target junk.sm");
    fs::write(
        hidden_dir.join("secret.sm"),
        "ANOTHER SYNTAX ERROR IN HIDDEN DIR !@#$%\n",
    )
    .expect("write hidden secret.sm");

    let report = smc_cli::compatibility::inspect_migration(&project_dir)
        .expect("migration inspection must exclude target/ and hidden dirs without error");

    assert_eq!(
        report.classification,
        CompatibilityClassification::Compatible,
        "syntax errors in target/ or hidden dirs must NOT affect compatibility because they are excluded"
    );

    for file in &report.inspected_files {
        let f_str = file.display().to_string().replace('\\', "/");
        assert!(
            !f_str.contains("/target/") && !f_str.contains("/.git_sub/"),
            "excluded dirs must not appear in inspected files: {f_str}"
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_17_invalid_source_fails_migration_compatibility() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test17_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    fs::create_dir_all(&project_dir).expect("create project dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"invalid_source_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    fs::write(
        project_dir.join("broken.sm"),
        "fn broken( { syntax error unclosed brace\n",
    )
    .expect("write broken.sm");

    let report = smc_cli::compatibility::inspect_migration(&project_dir)
        .expect("inspect_migration must succeed and report incompatibility fail-closed");

    assert_eq!(
        report.classification,
        CompatibilityClassification::Incompatible,
        "invalid source must fail-closed to Incompatible"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.category == "source_error" || f.category == "syntax_error"),
        "findings must contain source_error for broken syntax: {:?}",
        report.findings
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_18_malformed_manifest_fails_migration_compatibility() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test18_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    fs::create_dir_all(&project_dir).expect("create project dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package\nthis is invalid toml content !!!\n",
    )
    .expect("write malformed manifest");
    fs::write(project_dir.join("main.sm"), "fn main() { return; }\n").expect("write main.sm");

    let report = smc_cli::compatibility::inspect_migration(&project_dir)
        .expect("inspect_migration must succeed and report incompatibility fail-closed");

    assert_eq!(
        report.classification,
        CompatibilityClassification::Incompatible,
        "malformed manifest must fail-closed to Incompatible"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.category == "manifest_error"),
        "findings must contain manifest_error for invalid toml: {:?}",
        report.findings
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_19_canonical_lowercase_manifest_precedence() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test19_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    fs::create_dir_all(&project_dir).expect("create project dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"canonical_name\"\nversion = 1\n",
    )
    .expect("write lowercase manifest");
    fs::write(project_dir.join("main.sm"), "fn main() { return; }\n").expect("write main.sm");

    let report =
        smc_cli::compatibility::inspect_migration(&project_dir).expect("inspection should succeed");

    assert_eq!(
        report.classification,
        CompatibilityClassification::Compatible
    );
    let manifest_path = report.manifest_path.expect("manifest path present");
    assert!(
        manifest_path.ends_with("semantic.toml"),
        "manifest path must reference canonical lowercase semantic.toml: {}",
        manifest_path.display()
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_20_deterministic_finding_order() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test20_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    fs::create_dir_all(&project_dir).expect("create project dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"sort_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    fs::write(project_dir.join("z_last.sm"), "fn z( { syntax error\n").expect("write z_last.sm");
    fs::write(project_dir.join("a_first.sm"), "fn a( { syntax error\n").expect("write a_first.sm");

    let report1 = smc_cli::compatibility::inspect_migration(&project_dir).expect("report 1");
    let report2 = smc_cli::compatibility::inspect_migration(&project_dir).expect("report 2");

    assert_eq!(
        report1.findings, report2.findings,
        "findings must be deterministic across runs"
    );
    assert!(report1.findings.len() >= 2, "must have at least 2 findings");

    for window in report1.findings.windows(2) {
        let cmp = (
            &window[0].file,
            window[0].line,
            &window[0].category,
            &window[0].message,
        )
            .cmp(&(
                &window[1].file,
                window[1].line,
                &window[1].category,
                &window[1].message,
            ));
        assert!(
            cmp != std::cmp::Ordering::Greater,
            "findings must be strictly sorted"
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_21_json_special_character_escaping_in_artifact_inspect() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test21_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let src_path = temp_dir.join("test_escape.sm");
    let art_path = temp_dir.join("test_escape.smc");

    fs::write(
        &src_path,
        "// test with \"quotes\" and \\backslashes\\ and newline\nfn main() { return; }\n",
    )
    .expect("write src");

    let (c_code, _, c_err) = run_smc(&[
        "compile",
        src_path.to_str().unwrap(),
        "-o",
        art_path.to_str().unwrap(),
    ]);
    assert_eq!(c_code, 0, "compile failed: {c_err}");

    let (code, stdout, stderr) =
        run_smc(&["artifact", "inspect", art_path.to_str().unwrap(), "--json"]);
    assert_eq!(code, 0, "inspect failed: {stderr}");

    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("artifact inspect --json output must be valid, well-escaped JSON");

    assert!(parsed.get("artifact_hash").is_some());
    assert!(parsed.get("producer_provenance").is_some());
    assert!(parsed.get("inspecting_toolchain").is_some());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_22_json_special_character_escaping_in_migration_report() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test22_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    fs::create_dir_all(&project_dir).expect("create temp dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"escape_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    fs::write(
        project_dir.join("main.sm"),
        "// \"quotes\" and \\escapes\\\nfn broken( { syntax error\n",
    )
    .expect("write main.sm");

    let (_code, stdout, stderr) =
        run_smc(&["migrate", "check", "--json", project_dir.to_str().unwrap()]);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "migrate check --json output must be valid JSON: {e}\nstdout: {stdout}\nstderr: {stderr}"
        )
    });

    assert!(parsed.get("classification").is_some());
    assert!(parsed.get("findings").is_some());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_23_provenance_tampering_corrupted_mismatch() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test23_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let src_path = temp_dir.join("main.sm");
    let art_path = temp_dir.join("main.smc");

    fs::write(&src_path, "fn main() { return; }\n").expect("write src");

    let (c_code, _, c_err) = run_smc(&[
        "compile",
        src_path.to_str().unwrap(),
        "-o",
        art_path.to_str().unwrap(),
    ]);
    assert_eq!(c_code, 0, "compile failed: {c_err}");

    let status_before = detect_artifact_staleness(&art_path, &src_path).expect("staleness before");
    assert!(status_before.is_fresh());

    // Tamper with 1 byte of the artifact file
    let mut bytes = fs::read(&art_path).expect("read artifact");
    let len = bytes.len();
    bytes[len - 1] ^= 0x55;
    fs::write(&art_path, &bytes).expect("write tampered artifact");

    let status_after = detect_artifact_staleness(&art_path, &src_path).expect("staleness after");
    assert!(
        matches!(status_after, StalenessStatus::CorruptedMismatch { .. }),
        "artifact digest divergence from companion provenance must yield CorruptedMismatch: {:?}",
        status_after
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_24_imported_module_change_invalidates_provenance() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test24_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"mod_change_test\"\nversion = 1\n",
    )
    .expect("write manifest");
    let main_sm = src_dir.join("main.sm");
    let helper_sm = src_dir.join("helper.sm");
    let art_path = project_dir.join("main.smc");

    fs::write(&main_sm, "fn main() { return; }\n").expect("write main.sm");
    fs::write(&helper_sm, "fn helper() -> i32 { return 1; }\n").expect("write helper.sm");

    let (c_code, _, c_err) = run_smc(&[
        "compile",
        main_sm.to_str().unwrap(),
        "-o",
        art_path.to_str().unwrap(),
    ]);
    assert_eq!(c_code, 0, "compile failed: {c_err}");

    let status_before = detect_artifact_staleness(&art_path, &main_sm).expect("staleness before");
    assert!(status_before.is_fresh());

    // Modify the helper module (not main.sm)
    fs::write(&helper_sm, "fn helper() -> i32 { return 2; }\n").expect("modify helper.sm");

    let status_after = detect_artifact_staleness(&art_path, &main_sm).expect("staleness after");
    assert!(
        matches!(status_after, StalenessStatus::StaleSourceChanged { .. }),
        "modifying helper module must invalidate provenance via project source fingerprint: {:?}",
        status_after
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_25_compatibility_dimensions_canonical_drift_guard() {
    use smc_cli::artifact_identity::ProvenanceContractIdentity;
    use smc_cli::compatibility::*;

    let dims = CompatibilityDimensions::default();
    let contract = ProvenanceContractIdentity::default();

    // 1. Cross-validate CompatibilityDimensions vs ProvenanceContractIdentity
    assert_eq!(
        dims.verifier_profile, contract.verifier_profile,
        "verifier ID must match between compatibility dimensions and provenance contract"
    );
    assert_eq!(
        dims.runtime_engine, contract.runtime_profile,
        "runtime ID must match between compatibility dimensions and provenance contract"
    );
    assert_eq!(
        dims.stdlib_version, contract.stdlib_version,
        "stdlib ID must match between compatibility dimensions and provenance contract"
    );
    assert_eq!(
        dims.diagnostic_schema, contract.diagnostic_contract,
        "diagnostic schema must match between compatibility dimensions and provenance contract"
    );
    assert_eq!(
        dims.semcode_format, contract.semcode_format,
        "format magic must match between compatibility dimensions and provenance contract"
    );
    assert_eq!(
        dims.semcode_epoch as u16, contract.semcode_epoch,
        "format epoch must match between compatibility dimensions and provenance contract"
    );
    assert_eq!(
        dims.semcode_revision as u16, contract.semcode_revision,
        "format revision must match between compatibility dimensions and provenance contract"
    );

    // 2. Validate against CANONICAL constants
    assert_eq!(dims.source_version, CANONICAL_SOURCE_VERSION);
    assert_eq!(dims.manifest_version, CANONICAL_MANIFEST_VERSION);
    assert_eq!(dims.diagnostic_schema, CANONICAL_DIAGNOSTIC_SCHEMA);
    assert_eq!(dims.stdlib_version, CANONICAL_STDLIB_VERSION);
    assert_eq!(dims.semcode_format, CANONICAL_SEMCODE_FORMAT);
    assert_eq!(dims.semcode_epoch, CANONICAL_SEMCODE_EPOCH);
    assert_eq!(dims.semcode_revision, CANONICAL_SEMCODE_REVISION);
    assert_eq!(dims.verifier_profile, CANONICAL_VERIFIER_PROFILE);
    assert_eq!(dims.runtime_engine, CANONICAL_RUNTIME_ENGINE);

    // 3. Dynamic cross-validation against `smc version --json`
    let (v_code, v_out, v_err) = run_smc(&["version", "--json"]);
    assert_eq!(v_code, 0, "smc version --json failed: {v_err}");
    let v_parsed: serde_json::Value =
        serde_json::from_str(&v_out).expect("parse smc version --json");
    assert_eq!(
        v_parsed["verifier_profile"], CANONICAL_VERIFIER_PROFILE,
        "smc version JSON verifier_profile drift"
    );
    assert_eq!(
        v_parsed["runtime_profile"], CANONICAL_RUNTIME_ENGINE,
        "smc version JSON runtime_profile drift"
    );
    assert_eq!(
        v_parsed["stdlib_version"], CANONICAL_STDLIB_VERSION,
        "smc version JSON stdlib_version drift"
    );
    assert_eq!(
        v_parsed["semcode_format"]["magic"], CANONICAL_SEMCODE_FORMAT,
        "smc version JSON semcode magic drift"
    );
    assert_eq!(
        v_parsed["semcode_format"]["epoch"], CANONICAL_SEMCODE_EPOCH,
        "smc version JSON semcode epoch drift"
    );
    assert_eq!(
        v_parsed["semcode_format"]["revision"], CANONICAL_SEMCODE_REVISION,
        "smc version JSON semcode revision drift"
    );
}
