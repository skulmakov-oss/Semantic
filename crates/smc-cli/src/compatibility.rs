//! Compatibility dimensions, digest-based artifact staleness detection, and
//! non-destructive migration preview for Semantic.
//!
//! Enforces explicit, fail-closed compatibility rules across language, manifest,
//! format, verifier, stdlib, and runtime boundaries per SSF-10.

use crate::artifact_identity::{
    find_companion_provenance_path, ArtifactIdentity, ArtifactProvenance,
};
use serde::{Deserialize, Serialize};
use sm_format::sha256::sha256_prefixed_hex;
use std::fs;
use std::path::{Path, PathBuf};

/// Canonical compatibility dimensions defined by Semantic Stable Foundation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityDimensions {
    pub source_version: &'static str,
    pub manifest_version: u32,
    pub diagnostic_schema: &'static str,
    pub stdlib_version: &'static str,
    pub semcode_format: &'static str,
    pub semcode_epoch: u8,
    pub semcode_revision: u8,
    pub verifier_profile: &'static str,
    pub runtime_engine: &'static str,
}

impl Default for CompatibilityDimensions {
    fn default() -> Self {
        Self {
            source_version: "0.1.0",
            manifest_version: 1,
            diagnostic_schema: "semantic.diagnostics/v1",
            stdlib_version: "0.1.0",
            semcode_format: "SEMCOD22",
            semcode_epoch: 0,
            semcode_revision: 23,
            verifier_profile: "VerifiedLocal",
            runtime_engine: "SVM-Deterministic-v1",
        }
    }
}

/// Explicit compatibility classification across Semantic contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityClassification {
    /// Fully compatible with current canonical contracts.
    Compatible,
    /// Admitted under backward-compatibility window; migration recommended.
    Deprecated,
    /// Hard contract violation or structurally rejected.
    Incompatible,
    /// Future or unrecognized version beyond toolchain support.
    Unsupported,
}

impl CompatibilityClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "Compatible",
            Self::Deprecated => "Deprecated",
            Self::Incompatible => "Incompatible",
            Self::Unsupported => "Unsupported",
        }
    }
}

// ============================================================================
// SSF-10 Section 5: Explicit Compatibility Policies
// ============================================================================

/// Explicit policy governing Semantic source language stability and evolution.
pub struct SourceCompatibilityPolicy;

impl SourceCompatibilityPolicy {
    /// Minimum deprecation window policy: deprecated syntax/attributes must remain
    /// admitted with compiler diagnostic warnings for at least 1 full minor/epoch cycle
    /// before transition to Removed.
    pub const MIN_DEPRECATION_WINDOW_CYCLES: u32 = 1;

    /// Evaluates a source language construct state.
    pub fn assess_source_element(
        is_deprecated: bool,
        is_removed: bool,
    ) -> CompatibilityClassification {
        if is_removed {
            CompatibilityClassification::Incompatible
        } else if is_deprecated {
            CompatibilityClassification::Deprecated
        } else {
            CompatibilityClassification::Compatible
        }
    }
}

/// Explicit policy governing package manifests (`Semantic.toml`).
pub struct ManifestCompatibilityPolicy;

impl ManifestCompatibilityPolicy {
    /// Supported manifest schema versions. Manifests are versioned independently
    /// from source language compatibility.
    pub const SUPPORTED_SCHEMA_VERSIONS: &'static [u32] = &[1];

    /// Evaluate manifest schema version.
    /// Schema 1: current canonical format.
    /// Schema 0: legacy format, admitted with deprecation.
    /// Others: unsupported fail-closed.
    pub fn assess_manifest_schema(schema_version: u32) -> CompatibilityClassification {
        if Self::SUPPORTED_SCHEMA_VERSIONS.contains(&schema_version) {
            CompatibilityClassification::Compatible
        } else if schema_version == 0 {
            CompatibilityClassification::Deprecated
        } else {
            CompatibilityClassification::Unsupported
        }
    }
}

/// Explicit policy governing diagnostic schemas and machine contracts.
pub struct DiagnosticCompatibilityPolicy;

impl DiagnosticCompatibilityPolicy {
    pub const CANONICAL_SCHEMA: &'static str = "semantic.diagnostics";
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    /// Evaluates whether a change to diagnostics constitutes a breaking contract change.
    /// Breaking changes require incrementing the schema version (`v2`):
    /// - Changing or retiring an existing diagnostic error code.
    /// - Altering diagnostic severity from warning to error (or vice-versa).
    /// - Modifying JSON carrier structure or removing established keys.
    pub fn is_breaking_diagnostic_change(
        code_changed: bool,
        severity_changed: bool,
        schema_structure_changed: bool,
    ) -> bool {
        code_changed || severity_changed || schema_structure_changed
    }
}

/// Explicit policy governing Standard Library and Builtins.
pub struct StdlibCompatibilityPolicy;

impl StdlibCompatibilityPolicy {
    pub const CURRENT_STDLIB_VERSION: &'static str = "0.1.0";

    /// Evaluates whether a stdlib modification is breaking.
    /// Behavior-preserving: adding pure builtins without signature conflicts.
    /// Breaking: changing argument counts, types, or widening capability requirements.
    pub fn is_breaking_stdlib_change(signature_changed: bool, capability_widened: bool) -> bool {
        signature_changed || capability_widened
    }
}

/// Explicit policy governing Runtime execution and Total Determinism.
pub struct RuntimeCompatibilityPolicy;

impl RuntimeCompatibilityPolicy {
    pub const CANONICAL_RUNTIME_PROFILE: &'static str = "deterministic-v1";

    /// Total Determinism PRNG Rule:
    /// Any supported runtime PRNG contract must produce bit-for-bit identical
    /// pseudo-random sequences across repeated executions with the identical seed.
    pub fn verify_deterministic_prng_seed(seed: u64, iterations: usize) -> Vec<u64> {
        let mut state = seed;
        let mut out = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            out.push(state.wrapping_mul(0x2545_F491_4F6C_DD1D));
        }
        out
    }
}

/// Explicit policy governing SemCode wire format and verifier admission.
pub struct SemCodeVerifierPolicy;

impl SemCodeVerifierPolicy {
    pub const CANONICAL_VERIFIER_PROFILE: &'static str = "canonical-v1";
    pub const CURRENT_SEMCODE_MAGIC: &'static [u8; 8] = b"SEMCOD22";
    pub const CURRENT_SEMCODE_REVISION: u16 = 23;
}

// ============================================================================
// Artifact Staleness & Digest-Based Mismatch Authority
// ============================================================================

/// Staleness and mismatch assessment between compiled artifact, provenance, and source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StalenessStatus {
    /// Artifact is cryptographically fresh; source and manifest digests match provenance.
    Fresh {
        source_hash: String,
        artifact_hash: String,
        mtime_hint_match: bool,
    },
    /// The source content hash has changed relative to provenance.
    /// This is detected even when filesystem mtime was backdated or preserved!
    StaleSourceChanged {
        expected_source_hash: String,
        actual_source_hash: String,
        mtime_indicated_stale: bool,
    },
    /// The artifact belongs to a different project or package name.
    ProjectMismatch {
        expected_project: String,
        actual_project: String,
    },
    /// The project manifest (Semantic.toml) hash has changed relative to provenance.
    ManifestMismatch {
        expected_manifest_hash: String,
        actual_manifest_hash: String,
    },
    /// The toolchain that produced the artifact has an incompatible major version.
    ToolchainMismatch {
        producer_compiler: String,
        current_compiler: String,
        reason: String,
    },
    /// Companion provenance record is missing; fail-closed cannot prove provenance.
    MissingProvenance(PathBuf),
    /// Companion provenance record is malformed or invalid JSON.
    UnsupportedProvenance(String),
    /// Source file or project directory does not exist.
    MissingSource(PathBuf),
    /// Artifact file does not exist.
    MissingArtifact(PathBuf),
}

impl StalenessStatus {
    pub fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh { .. })
    }

    pub fn is_stale(&self) -> bool {
        matches!(
            self,
            Self::StaleSourceChanged { .. }
                | Self::ProjectMismatch { .. }
                | Self::ManifestMismatch { .. }
                | Self::ToolchainMismatch { .. }
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fresh { .. } => "Fresh",
            Self::StaleSourceChanged { .. } => "StaleSourceChanged",
            Self::ProjectMismatch { .. } => "ProjectMismatch",
            Self::ManifestMismatch { .. } => "ManifestMismatch",
            Self::ToolchainMismatch { .. } => "ToolchainMismatch",
            Self::MissingProvenance(_) => "MissingProvenance",
            Self::UnsupportedProvenance(_) => "UnsupportedProvenance",
            Self::MissingSource(_) => "MissingSource",
            Self::MissingArtifact(_) => "MissingArtifact",
        }
    }
}

/// Detect whether a compiled artifact is stale or mismatched relative to its source.
///
/// Canonical correctness is governed by cryptographic digest comparison,
/// making mismatch detection immune to preserved or backdated filesystem `mtime`.
pub fn detect_artifact_staleness(
    artifact_path: &Path,
    source_path: &Path,
) -> Result<StalenessStatus, String> {
    if !artifact_path.exists() {
        return Ok(StalenessStatus::MissingArtifact(
            artifact_path.to_path_buf(),
        ));
    }
    if !source_path.exists() {
        return Ok(StalenessStatus::MissingSource(source_path.to_path_buf()));
    }

    // 1. Calculate actual artifact hash
    let artifact_bytes = fs::read(artifact_path).map_err(|e| {
        format!(
            "failed to read artifact '{}': {}",
            artifact_path.display(),
            e
        )
    })?;
    let actual_artifact_hash = sha256_prefixed_hex(&artifact_bytes);

    // 2. Calculate actual source content hash
    let actual_source_hash = if source_path.is_file() {
        let s_bytes = fs::read(source_path)
            .map_err(|e| format!("failed to read source '{}': {}", source_path.display(), e))?;
        sha256_prefixed_hex(&s_bytes)
    } else {
        let mut combined = Vec::new();
        collect_project_source_bytes(source_path, &mut combined)?;
        sha256_prefixed_hex(&combined)
    };

    // 3. Inspect mtime as a secondary diagnostic hint
    let art_meta = fs::metadata(artifact_path).ok();
    let src_meta = fs::metadata(source_path).ok();
    let art_mtime = art_meta.and_then(|m| m.modified().ok());
    let src_mtime = src_meta.and_then(|m| m.modified().ok());
    let mtime_indicated_stale = match (art_mtime, src_mtime) {
        (Some(a), Some(s)) => s > a,
        _ => false,
    };

    // 4. Resolve companion provenance record
    let prov_path = match find_companion_provenance_path(artifact_path) {
        Some(p) => p,
        None => {
            let expected_prov = crate::artifact_identity::companion_provenance_path(artifact_path);
            return Ok(StalenessStatus::MissingProvenance(expected_prov));
        }
    };

    let prov_content = match fs::read_to_string(&prov_path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(StalenessStatus::UnsupportedProvenance(format!(
                "cannot read provenance '{}': {}",
                prov_path.display(),
                e
            )));
        }
    };

    let provenance: ArtifactProvenance = match serde_json::from_str(&prov_content) {
        Ok(p) => p,
        Err(e) => {
            return Ok(StalenessStatus::UnsupportedProvenance(format!(
                "cannot parse provenance JSON '{}': {}",
                prov_path.display(),
                e
            )));
        }
    };

    // 5. Cryptographic validation of artifact hash
    if provenance.artifact_hash != actual_artifact_hash {
        return Ok(StalenessStatus::StaleSourceChanged {
            expected_source_hash: provenance.source.source_hash,
            actual_source_hash,
            mtime_indicated_stale: true,
        });
    }

    // 6. Cryptographic validation of source hash (DIGEST AUTHORITY - immune to mtime backdating!)
    if provenance.source.source_hash != actual_source_hash {
        return Ok(StalenessStatus::StaleSourceChanged {
            expected_source_hash: provenance.source.source_hash,
            actual_source_hash,
            mtime_indicated_stale,
        });
    }

    // 7. Check manifest hash if manifest was recorded in provenance
    let current_manifest_path = if source_path.is_dir() {
        source_path.join("Semantic.toml")
    } else if let Some(parent) = source_path.parent() {
        parent.join("Semantic.toml")
    } else {
        PathBuf::from("Semantic.toml")
    };

    if let Some(expected_m_hash) = &provenance.source.manifest_hash {
        if current_manifest_path.is_file() {
            let m_bytes = fs::read(&current_manifest_path).map_err(|e| {
                format!(
                    "failed to read manifest '{}': {}",
                    current_manifest_path.display(),
                    e
                )
            })?;
            let actual_m_hash = sha256_prefixed_hex(&m_bytes);
            if expected_m_hash != &actual_m_hash {
                return Ok(StalenessStatus::ManifestMismatch {
                    expected_manifest_hash: expected_m_hash.clone(),
                    actual_manifest_hash: actual_m_hash,
                });
            }
        } else {
            return Ok(StalenessStatus::ManifestMismatch {
                expected_manifest_hash: expected_m_hash.clone(),
                actual_manifest_hash: "missing".to_string(),
            });
        }
    }

    // 8. Check project identity mismatch
    if let Some(expected_pkg) = &provenance.source.package_name {
        if current_manifest_path.is_file() {
            if let Ok(manifest_text) = fs::read_to_string(&current_manifest_path) {
                for line in manifest_text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("name =") {
                        let actual_pkg = trimmed
                            .strip_prefix("name =")
                            .unwrap_or("")
                            .trim()
                            .trim_matches('"')
                            .to_string();
                        if !actual_pkg.is_empty() && &actual_pkg != expected_pkg {
                            return Ok(StalenessStatus::ProjectMismatch {
                                expected_project: expected_pkg.clone(),
                                actual_project: actual_pkg,
                            });
                        }
                    }
                }
            }
        }
    }

    // 9. Check toolchain major version compatibility
    let current_version = env!("CARGO_PKG_VERSION");
    let current_major = current_version.split('.').next().unwrap_or("0");
    let producer_major = provenance
        .producer
        .compiler_version
        .split('.')
        .next()
        .unwrap_or("0");
    if current_major != producer_major && (current_major != "0" || producer_major != "0") {
        return Ok(StalenessStatus::ToolchainMismatch {
            producer_compiler: provenance.producer.compiler_version.clone(),
            current_compiler: current_version.to_string(),
            reason: format!(
                "incompatible toolchain major version: producer v{} vs current v{}",
                provenance.producer.compiler_version, current_version
            ),
        });
    }

    // All cryptographic checks match
    Ok(StalenessStatus::Fresh {
        source_hash: actual_source_hash,
        artifact_hash: actual_artifact_hash,
        mtime_hint_match: !mtime_indicated_stale,
    })
}

fn collect_project_source_bytes(dir: &Path, out: &mut Vec<u8>) -> Result<(), String> {
    let mut entries = Vec::new();
    collect_sm_files_sorted(dir, &mut entries)?;
    for path in entries {
        let b =
            fs::read(&path).map_err(|e| format!("failed to read '{}': {}", path.display(), e))?;
        out.extend_from_slice(path.to_string_lossy().as_bytes());
        out.push(0);
        out.extend_from_slice(&b);
        out.push(0);
    }
    Ok(())
}

fn collect_sm_files_sorted(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let read_dir = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory '{}': {}", dir.display(), e))?;
    let mut entries: Vec<_> = read_dir.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "target" {
                collect_sm_files_sorted(&path, out)?;
            }
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "sm") {
            out.push(path);
        }
    }
    Ok(())
}

/// Assess compatibility of a compiled SemCode artifact against canonical contracts.
pub fn assess_artifact_compatibility(identity: &ArtifactIdentity) -> CompatibilityClassification {
    // 1. Verifier admission check
    if !identity.verifier.admitted {
        return CompatibilityClassification::Incompatible;
    }

    // 2. Format / header revision evaluation
    match identity.header.magic.as_str() {
        "SEMCOD22" => {
            if identity.header.revision == 23 {
                CompatibilityClassification::Compatible
            } else if identity.header.revision > 23 {
                CompatibilityClassification::Unsupported
            } else {
                CompatibilityClassification::Deprecated
            }
        }
        "SEMCOD20" | "SEMCOD21" => CompatibilityClassification::Deprecated,
        magic if magic.starts_with("SEMCOD") || magic.starts_with("SEMCODE") => {
            CompatibilityClassification::Deprecated
        }
        _ => CompatibilityClassification::Incompatible,
    }
}

// ============================================================================
// Non-Destructive Migration Inspection (Dry-Run Authority)
// ============================================================================

/// Individual migration check finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationCheckFinding {
    pub file: PathBuf,
    pub line: usize,
    pub category: String,
    pub message: String,
    pub recommendation: String,
}

/// Summary report for non-destructive migration dry-run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReport {
    pub target: PathBuf,
    pub classification: CompatibilityClassification,
    pub findings: Vec<MigrationCheckFinding>,
    /// Must be guaranteed to be 0 for dry-run inspection.
    pub mutations_performed: usize,
    pub status: String,
}

impl MigrationReport {
    pub fn render_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Migration Dry-Run Inspection: {}\n",
            self.target.display()
        ));
        out.push_str(&format!(
            "Overall Compatibility: {}\n",
            self.classification.as_str()
        ));
        out.push_str(&format!(
            "Files Mutated:         {} (strictly non-destructive)\n",
            self.mutations_performed
        ));
        out.push_str(&format!("Findings Count:        {}\n", self.findings.len()));

        if self.findings.is_empty() {
            out.push_str("No deprecated constructs or migration requirements found.\n");
        } else {
            for f in &self.findings {
                out.push_str(&format!(
                    "  [{}:{}] {}: {}\n    Recommendation: {}\n",
                    f.file.display(),
                    f.line,
                    f.category,
                    f.message,
                    f.recommendation
                ));
            }
        }
        out.push_str(&format!("Status: {}\n", self.status));
        out
    }

    pub fn render_json(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!(
            "  \"target\": \"{}\",\n",
            self.target.display().to_string().replace('\\', "/")
        ));
        json.push_str(&format!(
            "  \"classification\": \"{}\",\n",
            self.classification.as_str()
        ));
        json.push_str(&format!(
            "  \"mutations_performed\": {},\n",
            self.mutations_performed
        ));
        json.push_str(&format!("  \"findings_count\": {},\n", self.findings.len()));
        json.push_str("  \"findings\": [\n");
        for (i, f) in self.findings.iter().enumerate() {
            let comma = if i + 1 < self.findings.len() { "," } else { "" };
            json.push_str(&format!(
                "    {{\"file\": \"{}\", \"line\": {}, \"category\": \"{}\", \"message\": \"{}\", \"recommendation\": \"{}\"}}{}\n",
                f.file.display().to_string().replace('\\', "/"),
                f.line,
                f.category,
                f.message.replace('\"', "\\\""),
                f.recommendation.replace('\"', "\\\""),
                comma
            ));
        }
        json.push_str("  ],\n");
        json.push_str(&format!("  \"status\": \"{}\"\n", self.status));
        json.push_str("}\n");
        json
    }
}

/// Perform non-destructive migration inspection on a target source file or project.
///
/// Strictly guarantees ZERO disk mutation.
pub fn inspect_migration(target: &Path) -> Result<MigrationReport, String> {
    if !target.exists() {
        return Err(format!("target path '{}' does not exist", target.display()));
    }

    let mut files_to_check = Vec::new();
    if target.is_dir() {
        collect_sm_files(target, &mut files_to_check)?;
        let manifest = target.join("Semantic.toml");
        if manifest.is_file() {
            files_to_check.push(manifest);
        }
        let legacy_manifest = target.join("package.manifest");
        if legacy_manifest.is_file() {
            files_to_check.push(legacy_manifest);
        }
    } else {
        files_to_check.push(target.to_path_buf());
    }

    let mut findings = Vec::new();

    for file in &files_to_check {
        let content = fs::read_to_string(file)
            .map_err(|e| format!("failed to read '{}': {}", file.display(), e))?;

        if file.file_name().is_some_and(|n| n == "package.manifest") {
            findings.push(MigrationCheckFinding {
                file: file.clone(),
                line: 1,
                category: "manifest_deprecation".to_string(),
                message: "package.manifest uses legacy package manifest format".to_string(),
                recommendation: "Migrate to canonical Semantic.toml manifest format".to_string(),
            });
        }

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("#[deprecated") || trimmed.contains("// @deprecated") {
                findings.push(MigrationCheckFinding {
                    file: file.clone(),
                    line: line_num,
                    category: "deprecated_attribute".to_string(),
                    message: "explicitly deprecated item in source".to_string(),
                    recommendation: "Review deprecation note and replace with current API"
                        .to_string(),
                });
            }

            if trimmed.starts_with("format = 0") || trimmed.starts_with("format 0") {
                findings.push(MigrationCheckFinding {
                    file: file.clone(),
                    line: line_num,
                    category: "manifest_version".to_string(),
                    message: "legacy format version 0 declared".to_string(),
                    recommendation: "Upgrade format declaration to 'format = 1'".to_string(),
                });
            }
        }
    }

    let classification = if findings.iter().any(|f| f.category.contains("error")) {
        CompatibilityClassification::Incompatible
    } else if !findings.is_empty() {
        CompatibilityClassification::Deprecated
    } else {
        CompatibilityClassification::Compatible
    };

    Ok(MigrationReport {
        target: target.to_path_buf(),
        classification,
        findings,
        mutations_performed: 0,
        status: "dry-run preview completed successfully with zero mutations".to_string(),
    })
}

fn collect_sm_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory '{}': {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "target" {
                collect_sm_files(&path, out)?;
            }
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "sm") {
            out.push(path);
        }
    }
    Ok(())
}
