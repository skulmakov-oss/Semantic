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
    detect_artifact_staleness, inspect_migration, CompatibilityClassification,
    DiagnosticCompatibilityPolicy, ManifestCompatibilityPolicy, RuntimeCompatibilityPolicy,
    SourceCompatibilityPolicy, StalenessStatus, StdlibCompatibilityPolicy,
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

    // 5. Runtime Compatibility Policy
    assert_eq!(
        RuntimeCompatibilityPolicy::assess_runtime_profile(
            RuntimeCompatibilityPolicy::CANONICAL_RUNTIME_PROFILE
        ),
        smc_cli::compatibility::CompatibilityClassification::Compatible
    );
    assert_eq!(
        RuntimeCompatibilityPolicy::assess_runtime_profile("experimental-v2"),
        smc_cli::compatibility::CompatibilityClassification::Incompatible
    );
    assert_eq!(
        RuntimeCompatibilityPolicy::assess_runtime_profile("deterministic-v0"),
        smc_cli::compatibility::CompatibilityClassification::Incompatible
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

#[test]
fn test_26_read_dir_error_handling_fails_closed() {
    use smc_cli::artifact_identity::collect_project_files_secure;
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test26_{}", std::process::id()));
    let non_existent = temp_dir.join("non_existent_subdir");
    let mut visited = std::collections::HashSet::new();
    let mut out_sm = Vec::new();
    let res = collect_project_files_secure(
        &temp_dir,
        &temp_dir,
        &non_existent,
        &mut visited,
        &mut out_sm,
    );
    assert!(
        res.is_err(),
        "collect_project_files_secure must fail closed on missing or unreadable dir"
    );
}

#[test]
fn test_27_reparse_check_fails_closed() {
    use smc_cli::artifact_identity::collect_project_files_secure;
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test27_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let canonical = fs::canonicalize(&temp_dir).expect("canonicalize");
    let outside_dir = std::env::temp_dir().join(format!("ssf10_test27_out_{}", std::process::id()));
    fs::create_dir_all(&outside_dir).expect("create outside dir");
    let canonical_out = fs::canonicalize(&outside_dir).expect("canonicalize outside");

    let mut visited = std::collections::HashSet::new();
    let mut out_sm = Vec::new();
    let res = collect_project_files_secure(
        &temp_dir,
        &canonical,
        &canonical_out,
        &mut visited,
        &mut out_sm,
    );
    assert!(
        res.is_err(),
        "collect_project_files_secure must fail closed when directory escapes canonical root"
    );
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::remove_dir_all(&outside_dir);
}

#[test]
fn test_28_prefix_free_source_framing() {
    use smc_cli::artifact_identity::collect_project_source_bytes;
    let temp_dir_a = std::env::temp_dir().join(format!("ssf10_test28a_{}", std::process::id()));
    let temp_dir_b = std::env::temp_dir().join(format!("ssf10_test28b_{}", std::process::id()));
    fs::create_dir_all(&temp_dir_a).expect("create a");
    fs::create_dir_all(&temp_dir_b).expect("create b");

    // Project A: file "a.sm" content "bc", file "d.sm" content "e"
    fs::write(temp_dir_a.join("a.sm"), "bc").expect("write a/a.sm");
    fs::write(temp_dir_a.join("d.sm"), "e").expect("write a/d.sm");

    // Project B: file "a.sm" content "b", file "cd.sm" content "e"
    // Under un-prefixed concatenation these could collide, but prefix-free framing distinguishes them
    fs::write(temp_dir_b.join("a.sm"), "b").expect("write b/a.sm");
    fs::write(temp_dir_b.join("cd.sm"), "e").expect("write b/cd.sm");

    let mut out_a = Vec::new();
    let mut out_b = Vec::new();
    collect_project_source_bytes(&temp_dir_a, &mut out_a).expect("collect a");
    collect_project_source_bytes(&temp_dir_b, &mut out_b).expect("collect b");

    assert_ne!(
        out_a, out_b,
        "prefix-free framing must prevent boundary shift collisions"
    );

    let _ = fs::remove_dir_all(&temp_dir_a);
    let _ = fs::remove_dir_all(&temp_dir_b);
}

#[test]
fn test_29_manifest_reparse_error_propagation() {
    use smc_cli::compatibility::inspect_migration;
    let missing_path = Path::new("non_existent_directory_for_migration_check_99999");
    let res = inspect_migration(missing_path);
    assert!(
        res.is_err(),
        "inspect_migration must fail closed if target does not exist"
    );
}

#[test]
fn test_30_verify_release_assets_powershell_syntax() {
    let script_path = Path::new("scripts/verify_release_assets.ps1");
    assert!(
        script_path.is_file(),
        "verify_release_assets.ps1 must exist"
    );
    let content = fs::read_to_string(script_path).expect("read script");
    assert!(
        !content.contains("    $sourceFingerprint = if"),
        "hashtable must not contain inline variable assignment"
    );
}

#[test]
fn test_31_compile_package_version_from_manifest() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test31_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    // Case 1: manifest has version = "1.2.3"
    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"ver_test\"\nversion = \"1.2.3\"\n",
    )
    .expect("write manifest");
    let main_sm = src_dir.join("main.sm");
    let art_path = project_dir.join("main.smc");
    fs::write(&main_sm, "fn main() { return; }\n").expect("write main.sm");

    let (c_code, _, c_err) = run_smc(&[
        "compile",
        main_sm.to_str().unwrap(),
        "-o",
        art_path.to_str().unwrap(),
    ]);
    assert_eq!(c_code, 0, "compile failed: {c_err}");

    let (i_code, i_out, i_err) = run_smc(&["artifact", "inspect", art_path.to_str().unwrap()]);
    assert_eq!(i_code, 0, "inspect failed: {i_err}");
    assert!(
        i_out.contains("Package:        ver_test v1.2.3"),
        "inspected output must show manifest package version: {i_out}"
    );

    // Case 2: manifest has NO version
    fs::write(
        project_dir.join("semantic.toml"),
        "[package]\nname = \"ver_test_none\"\n",
    )
    .expect("write manifest without version");
    let (c_code2, _, c_err2) = run_smc(&[
        "compile",
        main_sm.to_str().unwrap(),
        "-o",
        art_path.to_str().unwrap(),
    ]);
    assert_eq!(c_code2, 0, "compile without version failed: {c_err2}");

    let (i_code2, i_out2, i_err2) = run_smc(&["artifact", "inspect", art_path.to_str().unwrap()]);
    assert_eq!(i_code2, 0, "inspect failed: {i_err2}");
    assert!(
        i_out2.contains("Package:        ver_test_none\n"),
        "inspected output must NOT fabricate v0.1.0 when version is omitted: {i_out2}"
    );
    assert!(
        !i_out2.contains("ver_test_none v0.1.0"),
        "must not fabricate fallback 0.1.0"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_32_migrate_requires_subcommand() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test32_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let test_file = temp_dir.join("test.sm");
    fs::write(&test_file, "fn main() { return; }\n").expect("write test.sm");

    // 1. Invoking without subcommand must fail
    let (code_no_sub, _, err_no_sub) = run_smc(&["migrate", test_file.to_str().unwrap()]);
    assert_ne!(
        code_no_sub, 0,
        "invoking migrate without subcommand must fail"
    );
    assert!(
        err_no_sub.contains("missing required subcommand 'check' or 'preview'"),
        "error must explain missing subcommand: {err_no_sub}"
    );

    // 2. Invoking with check on valid file must succeed (Compatible -> exit 0)
    let (code_ok, out_ok, _) = run_smc(&["migrate", "check", test_file.to_str().unwrap()]);
    assert_eq!(code_ok, 0, "migrate check on valid file must succeed");
    assert!(out_ok.contains("Compatible"));

    // 3. Invoking with check on syntax error file must fail with non-zero exit code (Incompatible -> exit 1)
    let err_file = temp_dir.join("error.sm");
    fs::write(&err_file, "fn main() { invalid syntax !! }\n").expect("write error.sm");
    let (code_err, _, err_out) = run_smc(&["migrate", "check", err_file.to_str().unwrap()]);
    assert_ne!(
        code_err, 0,
        "migrate check on incompatible syntax must exit non-zero"
    );
    assert!(
        err_out.contains("migration check failed with classification: Incompatible"),
        "must report classification failure: {err_out}"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_33_producer_toolchain_target_forwarding() {
    use smc_cli::artifact_identity::ProducerToolchainIdentity;
    let id = ProducerToolchainIdentity::default();
    assert!(
        !id.build_target.is_empty(),
        "build_target must not be empty"
    );
    if let Ok(expected) = std::env::var("TARGET") {
        assert_eq!(id.build_target, expected);
    }
}

#[test]
fn test_34_provenance_schema_version_validation() {
    use smc_cli::artifact_identity::{ArtifactIdentity, ArtifactProvenance, ProvenanceStatus};
    use smc_cli::compatibility::{detect_artifact_staleness, StalenessStatus};

    let temp_dir = std::env::temp_dir().join(format!("ssf10_test34_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create dir");
    let src_file = temp_dir.join("main.sm");
    let art_file = temp_dir.join("main.smc");
    let prov_file = temp_dir.join("main.smc.provenance.json");
    fs::write(&src_file, "fn main() { return; }\n").expect("write src");

    let (c_code, _, _) = run_smc(&[
        "compile",
        src_file.to_str().unwrap(),
        "-o",
        art_file.to_str().unwrap(),
    ]);
    assert_eq!(c_code, 0);

    // Read generated valid provenance and tamper schema_version to 99
    let mut prov: ArtifactProvenance =
        serde_json::from_str(&fs::read_to_string(&prov_file).expect("read prov")).expect("parse");
    prov.schema_version = 99;
    fs::write(&prov_file, serde_json::to_string_pretty(&prov).unwrap())
        .expect("write tampered prov");

    // 1. ArtifactIdentity::from_file must report Unsupported
    let ident = ArtifactIdentity::from_file(&art_file).expect("from_file");
    assert!(
        matches!(ident.provenance_status, ProvenanceStatus::Unsupported(_)),
        "unsupported schema version must yield ProvenanceStatus::Unsupported: {:?}",
        ident.provenance_status
    );

    // 2. ArtifactIdentity::with_provenance must return Err
    let from_bytes = ArtifactIdentity::from_bytes(&fs::read(&art_file).unwrap()).unwrap();
    assert!(
        from_bytes.with_provenance(prov.clone()).is_err(),
        "with_provenance must reject schema_version != 1"
    );

    // 3. detect_artifact_staleness must report UnsupportedProvenance
    let staleness = detect_artifact_staleness(&art_file, &src_file).expect("staleness");
    assert!(
        matches!(staleness, StalenessStatus::UnsupportedProvenance(_)),
        "unsupported schema version must yield StalenessStatus::UnsupportedProvenance: {:?}",
        staleness
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_35_deterministic_prng_alignment_with_vm() {
    // 1. Architectural Guard: ensure smc-cli does NOT implement runtime PRNG algorithm.
    // Sole semantic authority belongs to sm-vm.
    let compat_path = Path::new("crates/smc-cli/src/compatibility.rs");
    let compat_src = fs::read_to_string(compat_path).expect("read compatibility.rs");
    assert!(
        !compat_src.contains("verify_deterministic_prng_seed"),
        "smc-cli must not expose duplicate verify_deterministic_prng_seed authority"
    );
    assert!(
        !compat_src.contains("state ^= state << 13"),
        "smc-cli must not contain duplicate xorshift64 bitshift algorithm"
    );

    // 2. Canonical Runtime Transition in sm-vm
    // Zero-seed normalization: maps to 1 to avoid zero fixed-point
    assert_eq!(
        sm_vm::deterministic_prng_next(0),
        sm_vm::deterministic_prng_next(1),
        "seed 0 must be normalized to seed 1 to avoid fixed point"
    );

    // Exact frozen sequence from canonical contract (docs/spec/foundation_stdlib_v0.md:181-192)
    // Seed 1 transitions:
    assert_eq!(
        sm_vm::deterministic_prng_next(1),
        1082269761u64,
        "first transition from seed 1"
    );
    assert_eq!(
        sm_vm::deterministic_prng_next(1082269761u64),
        1152992998833853505u64,
        "second transition from seed 1"
    );
    assert_eq!(
        sm_vm::deterministic_prng_next(1152992998833853505u64),
        11177516664432764457u64,
        "third transition from seed 1"
    );

    // Seed 42 transitions:
    assert_eq!(
        sm_vm::deterministic_prng_next(42),
        45454805674u64,
        "first transition from seed 42"
    );
    assert_eq!(
        sm_vm::deterministic_prng_next(45454805674u64),
        11532217803599905471u64,
        "second transition from seed 42"
    );

    // deterministic_prng_step advances mutable state
    let mut state = 42u64;
    let s1 = sm_vm::deterministic_prng_step(&mut state);
    assert_eq!(s1, 45454805674u64);
    assert_eq!(state, 45454805674u64);
    let s2 = sm_vm::deterministic_prng_step(&mut state);
    assert_eq!(s2, 11532217803599905471u64);
    assert_eq!(state, 11532217803599905471u64);

    // 3. Authoritative VM Execution via SemCode
    let program = r#"
fn run_prng(seed: i32, lo: i32, hi: i32) -> i32 {
    random_seed(seed);
    let v: i32 = random_next_i32(lo, hi);
    return v;
}
fn main() { return; }
"#;
    let semcode_bytes =
        sm_emit::compile_program_to_semcode(program).expect("compile PRNG program to SemCode");
    let token = verify_semcode_token(&semcode_bytes).expect("verify SemCode token");
    let entry = token
        .require_entry("run_prng")
        .expect("require run_prng entry");

    // Frozen contract output for seed 42 in range [0, 1000):
    // raw = 45454805674, offset = raw % 1000 = 674, result = 0 + 674 = 674.
    let res42 = sm_vm::run_verified_function_semcode_with_args(
        &entry,
        vec![
            sm_vm::Value::I32(42),
            sm_vm::Value::I32(0),
            sm_vm::Value::I32(1000),
        ],
    )
    .expect("execute VM run_prng(42, 0, 1000)");
    assert_eq!(
        res42,
        sm_vm::Value::I32(674),
        "VM execution for seed 42 in [0, 1000) must yield exact frozen contract value 674"
    );

    // Repeatability: executing again with same seed yields identical result
    let res42_repeat = sm_vm::run_verified_function_semcode_with_args(
        &entry,
        vec![
            sm_vm::Value::I32(42),
            sm_vm::Value::I32(0),
            sm_vm::Value::I32(1000),
        ],
    )
    .expect("execute VM run_prng(42, 0, 1000) repeat");
    assert_eq!(
        res42, res42_repeat,
        "deterministic VM execution must yield identical outputs for identical seed"
    );

    // Seed 0 normalization to 1:
    // raw = 1082269761, offset = raw % 1000 = 761, result = 0 + 761 = 761.
    let res0 = sm_vm::run_verified_function_semcode_with_args(
        &entry,
        vec![
            sm_vm::Value::I32(0),
            sm_vm::Value::I32(0),
            sm_vm::Value::I32(1000),
        ],
    )
    .expect("execute VM run_prng(0, 0, 1000)");
    let res1 = sm_vm::run_verified_function_semcode_with_args(
        &entry,
        vec![
            sm_vm::Value::I32(1),
            sm_vm::Value::I32(0),
            sm_vm::Value::I32(1000),
        ],
    )
    .expect("execute VM run_prng(1, 0, 1000)");
    assert_eq!(
        res0,
        sm_vm::Value::I32(761),
        "VM execution for seed 0 must yield exact frozen contract value 761"
    );
    assert_eq!(
        res0, res1,
        "VM execution for seed 0 and seed 1 must yield identical output"
    );

    // Cross-zero range: [-2000000000, 2000000000)
    let res_span = sm_vm::run_verified_function_semcode_with_args(
        &entry,
        vec![
            sm_vm::Value::I32(1),
            sm_vm::Value::I32(-2000000000),
            sm_vm::Value::I32(2000000000),
        ],
    )
    .expect("execute VM run_prng(1, -2000000000, 2000000000)");
    if let sm_vm::Value::I32(val) = res_span {
        assert!(
            (-2000000000..2000000000).contains(&val),
            "output must lie strictly within [lo, hi): got {val}"
        );
    } else {
        panic!("expected i32 return value");
    }
}

#[test]
fn test_36_assess_artifact_compatibility_branches() {
    use smc_cli::artifact_identity::ArtifactIdentity;
    use smc_cli::compatibility::{assess_artifact_compatibility, CompatibilityClassification};

    let temp_dir = std::env::temp_dir().join(format!("ssf10_test36_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create dir");
    let src_file = temp_dir.join("main.sm");
    let art_file = temp_dir.join("main.smc");
    fs::write(&src_file, "fn main() { return; }\n").expect("write src");

    let (c_code, _, _) = run_smc(&[
        "compile",
        src_file.to_str().unwrap(),
        "-o",
        art_file.to_str().unwrap(),
    ]);
    assert_eq!(c_code, 0);

    let valid_bytes = fs::read(&art_file).expect("read artifact");

    // 1. Current canonical rev 23 -> Compatible
    let ident_current = ArtifactIdentity::from_bytes(&valid_bytes).expect("ident current");
    assert_eq!(
        assess_artifact_compatibility(&ident_current),
        CompatibilityClassification::Compatible
    );

    // 2. Legacy rev < 23 -> Deprecated
    let mut legacy_bytes = valid_bytes.clone();
    // Offset 10 is revision in 8-byte magic + 2-byte epoch + 2-byte rev
    legacy_bytes[10] = 22;
    legacy_bytes[11] = 0;
    let mut ident_legacy = ArtifactIdentity::from_bytes(&legacy_bytes).expect("ident legacy");
    ident_legacy.verifier.admitted = true; // Test compatibility classification logic
    assert_eq!(
        assess_artifact_compatibility(&ident_legacy),
        CompatibilityClassification::Deprecated
    );

    // 3. Future rev > 23 -> Unsupported
    let mut future_bytes = valid_bytes.clone();
    future_bytes[10] = 24;
    future_bytes[11] = 0;
    let mut ident_future = ArtifactIdentity::from_bytes(&future_bytes).expect("ident future");
    ident_future.verifier.admitted = true;
    assert_eq!(
        assess_artifact_compatibility(&ident_future),
        CompatibilityClassification::Unsupported
    );

    // 4. Verifier rejected or corrupted -> Incompatible
    let mut rejected = ident_current.clone();
    rejected.verifier.admitted = false;
    assert_eq!(
        assess_artifact_compatibility(&rejected),
        CompatibilityClassification::Incompatible
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_37_uppercase_manifest_deprecation_and_format_recommendation() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test37_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    fs::create_dir_all(&project_dir).expect("create dir");

    // Create uppercase Semantic.toml with legacy format = 0
    fs::write(
        project_dir.join("Semantic.toml"),
        "[package]\nname = \"legacy_manifest\"\nformat = 0\n",
    )
    .expect("write uppercase manifest");
    fs::write(project_dir.join("main.sm"), "fn main() { return; }\n").expect("write main.sm");

    let report = inspect_migration(&project_dir).expect("inspect_migration");

    // Verify finding for uppercase Semantic.toml
    let has_deprecated_manifest = report
        .findings
        .iter()
        .any(|f| f.category == "manifest_deprecated" && f.message.contains("Semantic.toml"));
    assert!(
        has_deprecated_manifest,
        "inspect_migration must emit manifest_deprecated for uppercase Semantic.toml: {:?}",
        report.findings
    );

    // Verify finding for format = 0 does NOT recommend format = 1
    let format_finding = report
        .findings
        .iter()
        .find(|f| f.category == "manifest_version")
        .expect("must find manifest_version finding");
    assert!(
        !format_finding.recommendation.contains("format = 1"),
        "recommendation must not propose format = 1 in semantic.toml: {}",
        format_finding.recommendation
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_38_manifest_lowercase_precedence_and_atomic_compile_safety() {
    let temp_dir = std::env::temp_dir().join(format!("ssf10_test38_{}", std::process::id()));
    let project_dir = temp_dir.join("project");
    fs::create_dir_all(&project_dir).expect("create project dir");

    // 1. Verify atomic compile safety: failed compilation preserves pre-existing valid artifact
    let out_smc = project_dir.join("output.smc");
    let original_payload = b"ORIGINAL_VALID_ARTIFACT_PRESERVED";
    fs::write(&out_smc, original_payload).expect("write original valid artifact");

    let bad_sm = project_dir.join("bad.sm");
    fs::write(&bad_sm, "fn invalid_syntax { non_existent }\n").expect("write bad.sm");

    let (c_code, _, _) = run_smc(&[
        "compile",
        bad_sm.to_str().unwrap(),
        "-o",
        out_smc.to_str().unwrap(),
    ]);
    assert_ne!(c_code, 0, "compilation with bad syntax must fail");
    assert!(
        out_smc.is_file(),
        "pre-existing output file must not be removed on compile failure"
    );
    let remaining_bytes = fs::read(&out_smc).expect("read remaining bytes");
    assert_eq!(
        remaining_bytes, original_payload,
        "pre-existing output file contents must remain unchanged after compile failure"
    );

    // 2. Verify non-destructive failure when provenance sidecar destination fails (e.g. is a directory)
    let fail_sidecar_dir = project_dir.join("sidecar_dest_fail.smc.provenance.json");
    let sidecar_out_smc = project_dir.join("sidecar_dest_fail.smc");
    fs::write(&sidecar_out_smc, b"PREEXISTING_VALID_OUTPUT_UNTOUCHED")
        .expect("write pre-existing output");
    fs::create_dir_all(&fail_sidecar_dir).expect("create blocking directory at sidecar path");

    let valid_sm = project_dir.join("valid.sm");
    fs::write(&valid_sm, "fn main() { return; }\n").expect("write valid.sm");

    let (c_code_sc, _, _) = run_smc(&[
        "compile",
        valid_sm.to_str().unwrap(),
        "-o",
        sidecar_out_smc.to_str().unwrap(),
    ]);
    assert_ne!(
        c_code_sc, 0,
        "compile must fail when companion provenance sidecar destination cannot be written"
    );
    let preserved_sc_bytes = fs::read(&sidecar_out_smc).expect("read preserved output");
    assert_eq!(
        preserved_sc_bytes, b"PREEXISTING_VALID_OUTPUT_UNTOUCHED",
        "pre-existing output must NOT be replaced when provenance sidecar saving fails"
    );
    let _ = fs::remove_dir_all(&fail_sidecar_dir);

    // 2b. Verify sidecar restoration and rollback when artifact destination fails (e.g. artifact destination is a directory)
    let fail_art_dir = project_dir.join("art_dest_fail.smc");
    let fail_art_prov = project_dir.join("art_dest_fail.smc.provenance.json");
    fs::create_dir_all(&fail_art_dir).expect("create blocking directory at artifact path");
    let initial_prov_bytes = b"{\"preexisting_sidecar\": true}";
    fs::write(&fail_art_prov, initial_prov_bytes).expect("write pre-existing sidecar");

    let (c_code_art, _, _) = run_smc(&[
        "compile",
        valid_sm.to_str().unwrap(),
        "-o",
        fail_art_dir.to_str().unwrap(),
    ]);
    assert_ne!(
        c_code_art, 0,
        "compile must fail when artifact destination cannot be replaced (e.g. is a directory)"
    );
    let restored_prov_bytes = fs::read(&fail_art_prov).expect("read restored sidecar");
    assert_eq!(
        restored_prov_bytes, initial_prov_bytes,
        "pre-existing provenance sidecar must be preserved/restored when artifact replacement fails"
    );
    let _ = fs::remove_file(&fail_art_prov);
    let _ = fs::remove_dir_all(&fail_art_dir);

    // 2c. Direct unit test of save_artifact_and_companion_provenance_atomic rollback:
    // When artifact replacement fails, an existing sidecar is restored intact,
    // and when no sidecar previously existed, no orphaned sidecar is left behind.
    let unit_art_dir = project_dir.join("unit_fail_art.smc");
    let unit_art_prov = smc_cli::artifact_identity::companion_provenance_path(&unit_art_dir);
    fs::create_dir_all(&unit_art_dir).expect("create blocking directory at artifact path");

    let dummy_prov = smc_cli::artifact_identity::generate_companion_provenance(
        b"payload",
        &project_dir,
        None,
        None,
    )
    .expect("generate dummy provenance");

    // Case 2c-1: No pre-existing sidecar -> must not leave orphaned sidecar on failure
    let res_none = smc_cli::artifact_identity::save_artifact_and_companion_provenance_atomic(
        &unit_art_dir,
        b"payload",
        &dummy_prov,
    );
    assert!(
        res_none.is_err(),
        "must return error when artifact cannot be replaced"
    );
    assert!(
        !unit_art_prov.exists(),
        "must not leave orphaned sidecar behind when no sidecar existed prior to failure"
    );

    // Case 2c-2: Pre-existing sidecar -> must restore original sidecar on failure
    fs::write(&unit_art_prov, b"ORIGINAL_SIDECAR_PAYLOAD").expect("write pre-existing sidecar");
    let res_existing = smc_cli::artifact_identity::save_artifact_and_companion_provenance_atomic(
        &unit_art_dir,
        b"payload",
        &dummy_prov,
    );
    assert!(
        res_existing.is_err(),
        "must return error when artifact cannot be replaced"
    );
    let preserved_sidecar = fs::read(&unit_art_prov).expect("read preserved sidecar");
    assert_eq!(
        preserved_sidecar, b"ORIGINAL_SIDECAR_PAYLOAD",
        "pre-existing sidecar must be restored to original contents after artifact replacement failure"
    );
    let prov_dir = unit_art_prov.parent().unwrap();
    let has_dangling_backup = fs::read_dir(prov_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().contains(".bak."));
    assert!(
        !has_dangling_backup,
        "backup file must be cleaned up when restoration succeeds"
    );
    let _ = fs::remove_file(&unit_art_prov);
    let _ = fs::remove_dir_all(&unit_art_dir);

    // 3. Direct atomic write verification
    let atomic_target = project_dir.join("atomic_target.bin");
    fs::write(&atomic_target, b"initial").expect("write initial");
    smc_cli::artifact_identity::write_file_atomic(&atomic_target, b"replaced")
        .expect("write_file_atomic should succeed");
    let updated_bytes = fs::read(&atomic_target).expect("read atomic target");
    assert_eq!(updated_bytes, b"replaced");

    // 4. Manifest lowercase precedence in inspect_migration with both casings exercised
    let sub_project = temp_dir.join("manifest_sub");
    fs::create_dir_all(&sub_project).expect("create manifest_sub");
    fs::write(
        sub_project.join("semantic.toml"),
        "[package]\nname = \"canonical_lowercase\"\n",
    )
    .expect("write lowercase manifest");
    fs::write(sub_project.join("main.sm"), "fn main() { return; }\n").expect("write main.sm");

    // Probe filesystem case-sensitivity
    let probe_a = sub_project.join("case_probe_a.tmp");
    let probe_b = sub_project.join("CASE_PROBE_A.tmp");
    let _ = fs::write(&probe_a, b"a");
    let is_case_sensitive =
        fs::write(&probe_b, b"b").is_ok() && fs::read(&probe_a).map(|c| c == b"a").unwrap_or(false);
    let _ = fs::remove_file(&probe_a);
    let _ = fs::remove_file(&probe_b);

    if is_case_sensitive {
        // Exercise coexistence of both manifest casings in the same directory
        fs::write(
            sub_project.join("Semantic.toml"),
            "[package]\nname = \"legacy_uppercase_manifest\"\n",
        )
        .expect("write uppercase manifest alongside lowercase");
    }

    let report = inspect_migration(&sub_project).expect("inspect_migration");
    assert_eq!(
        report.classification,
        CompatibilityClassification::Compatible
    );
    let m_path = report.manifest_path.expect("manifest_path present");
    assert!(
        m_path.ends_with("semantic.toml"),
        "must select canonical lowercase semantic.toml: {}",
        m_path.display()
    );
    // Findings must NOT report deprecated uppercase name because canonical lowercase is present and prioritized
    let has_uppercase_finding = report
        .findings
        .iter()
        .any(|f| f.message.contains("Semantic.toml"));
    assert!(
        !has_uppercase_finding,
        "must not emit uppercase finding when canonical lowercase manifest is prioritized"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
