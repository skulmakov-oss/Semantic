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

pub const CANONICAL_SOURCE_VERSION: &str = "0.1.0";
pub const CANONICAL_MANIFEST_VERSION: u32 = 1;
pub const CANONICAL_DIAGNOSTIC_SCHEMA: &str = "semantic.diagnostics";
pub const CANONICAL_STDLIB_VERSION: &str = "semantic-stdlib-v1";
pub const CANONICAL_SEMCODE_FORMAT: &str = "SEMCOD22";
pub const CANONICAL_SEMCODE_EPOCH: u8 = 0;
pub const CANONICAL_SEMCODE_REVISION: u8 = 23;
pub const CANONICAL_VERIFIER_PROFILE: &str = "verifier-canonical-v1";
pub const CANONICAL_RUNTIME_ENGINE: &str = "deterministic-v1";

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
            source_version: CANONICAL_SOURCE_VERSION,
            manifest_version: CANONICAL_MANIFEST_VERSION,
            diagnostic_schema: CANONICAL_DIAGNOSTIC_SCHEMA,
            stdlib_version: CANONICAL_STDLIB_VERSION,
            semcode_format: CANONICAL_SEMCODE_FORMAT,
            semcode_epoch: CANONICAL_SEMCODE_EPOCH,
            semcode_revision: CANONICAL_SEMCODE_REVISION,
            verifier_profile: CANONICAL_VERIFIER_PROFILE,
            runtime_engine: CANONICAL_RUNTIME_ENGINE,
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
    pub const CANONICAL_RUNTIME_PROFILE: &'static str = CANONICAL_RUNTIME_ENGINE;

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
    pub const CANONICAL_VERIFIER_PROFILE: &'static str = CANONICAL_VERIFIER_PROFILE;
    pub const CURRENT_SEMCODE_MAGIC: &'static [u8; 8] = b"SEMCOD22";
    pub const CURRENT_SEMCODE_REVISION: u16 = CANONICAL_SEMCODE_REVISION as u16;
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
    /// The artifact bytes do not match the cryptographic hash recorded in companion provenance.
    CorruptedMismatch {
        expected_artifact_hash: String,
        actual_artifact_hash: String,
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
                | Self::CorruptedMismatch { .. }
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fresh { .. } => "Fresh",
            Self::StaleSourceChanged { .. } => "StaleSourceChanged",
            Self::ProjectMismatch { .. } => "ProjectMismatch",
            Self::ManifestMismatch { .. } => "ManifestMismatch",
            Self::ToolchainMismatch { .. } => "ToolchainMismatch",
            Self::CorruptedMismatch { .. } => "CorruptedMismatch",
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
    let project_root = if source_path.is_dir() {
        Some(source_path.to_path_buf())
    } else {
        crate::artifact_identity::find_project_root_ancestor(source_path)
    };

    let actual_source_hash = match project_root {
        Some(ref root) => {
            let mut combined = Vec::new();
            crate::artifact_identity::collect_project_source_bytes(root, &mut combined)?;
            sha256_prefixed_hex(&combined)
        }
        None => {
            let s_bytes = if source_path.is_file() {
                fs::read(source_path).map_err(|e| {
                    format!("failed to read source '{}': {}", source_path.display(), e)
                })?
            } else {
                let mut combined = Vec::new();
                crate::artifact_identity::collect_project_source_bytes(source_path, &mut combined)?;
                combined
            };
            sha256_prefixed_hex(&s_bytes)
        }
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
        return Ok(StalenessStatus::CorruptedMismatch {
            expected_artifact_hash: provenance.artifact_hash,
            actual_artifact_hash,
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
    let current_manifest_path = match project_root {
        Some(ref root) => {
            if root.join("semantic.toml").is_file() {
                root.join("semantic.toml")
            } else if root.join("Semantic.toml").is_file() {
                root.join("Semantic.toml")
            } else {
                root.join("semantic.toml")
            }
        }
        None => {
            let base_dir = if source_path.is_dir() {
                source_path
            } else if let Some(parent) = source_path.parent() {
                parent
            } else {
                Path::new(".")
            };
            if base_dir.join("semantic.toml").is_file() {
                base_dir.join("semantic.toml")
            } else if base_dir.join("Semantic.toml").is_file() {
                base_dir.join("Semantic.toml")
            } else {
                base_dir.join("semantic.toml")
            }
        }
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

#[derive(Serialize, Deserialize)]
struct MigrationReportJson<'a> {
    target: String,
    classification: &'a str,
    mutations_performed: usize,
    findings_count: usize,
    inspected_files_count: usize,
    manifest_path: Option<String>,
    findings: Vec<MigrationCheckFindingJson>,
    status: &'a str,
}

#[derive(Serialize, Deserialize)]
struct MigrationCheckFindingJson {
    file: String,
    line: usize,
    category: String,
    message: String,
    recommendation: String,
}

/// Summary report for non-destructive migration dry-run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReport {
    pub target: PathBuf,
    pub classification: CompatibilityClassification,
    pub findings: Vec<MigrationCheckFinding>,
    pub inspected_files: Vec<PathBuf>,
    pub manifest_path: Option<PathBuf>,
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
        out.push_str(&format!(
            "Inspected Files:       {}\n",
            self.inspected_files.len()
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
        let findings_json: Vec<MigrationCheckFindingJson> = self
            .findings
            .iter()
            .map(|f| MigrationCheckFindingJson {
                file: f.file.display().to_string().replace('\\', "/"),
                line: f.line,
                category: f.category.clone(),
                message: f.message.clone(),
                recommendation: f.recommendation.clone(),
            })
            .collect();

        let report_json = MigrationReportJson {
            target: self.target.display().to_string().replace('\\', "/"),
            classification: self.classification.as_str(),
            mutations_performed: self.mutations_performed,
            findings_count: self.findings.len(),
            inspected_files_count: self.inspected_files.len(),
            manifest_path: self
                .manifest_path
                .as_ref()
                .map(|p| p.display().to_string().replace('\\', "/")),
            findings: findings_json,
            status: &self.status,
        };

        serde_json::to_string_pretty(&report_json)
            .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }
}

fn offset_to_line(content: &str, pos: usize) -> usize {
    let safe_pos = pos.min(content.len());
    content[..safe_pos].chars().filter(|&c| c == '\n').count() + 1
}

/// Perform non-destructive migration inspection on a target source file or project.
///
/// Strictly guarantees ZERO disk mutation.
pub fn inspect_migration(target: &Path) -> Result<MigrationReport, String> {
    if !target.exists() {
        return Err(format!("target path '{}' does not exist", target.display()));
    }

    let mut files_to_check = Vec::new();
    let mut discovered_manifest = None;
    if target.is_dir() {
        let canonical_target = fs::canonicalize(target)
            .map_err(|e| format!("failed to canonicalize '{}': {}", target.display(), e))?;
        let mut visited_dirs = std::collections::HashSet::new();
        crate::artifact_identity::collect_project_files_secure(
            target,
            &canonical_target,
            target,
            &mut visited_dirs,
            &mut files_to_check,
        )?;

        // Canonical manifest search: check semantic.toml first, then Semantic.toml
        discovered_manifest = if target.join("semantic.toml").is_file() {
            Some(target.join("semantic.toml"))
        } else if target.join("Semantic.toml").is_file() {
            Some(target.join("Semantic.toml"))
        } else {
            None
        };
        if let Some(ref m) = discovered_manifest {
            if crate::package_manifest::path_is_reparse(m).unwrap_or(false) {
                let m_canon = fs::canonicalize(m).map_err(|e| {
                    format!("failed to canonicalize manifest '{}': {}", m.display(), e)
                })?;
                if !m_canon.starts_with(&canonical_target) {
                    return Err(format!(
                        "security violation: manifest '{}' escapes project root",
                        m.display()
                    ));
                }
            }
            files_to_check.push(m.clone());
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

        let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name == "package.manifest" {
            findings.push(MigrationCheckFinding {
                file: file.clone(),
                line: 1,
                category: "manifest_deprecation".to_string(),
                message: "package.manifest uses legacy package manifest format".to_string(),
                recommendation: "Migrate to canonical semantic.toml manifest format".to_string(),
            });
        } else if file_name == "semantic.toml" || file_name == "Semantic.toml" {
            // Validate manifest structure using canonical package manifest parser
            if let Err(err) = crate::package_manifest::parse_semantic_toml_manifest(file, &content)
            {
                findings.push(MigrationCheckFinding {
                    file: file.clone(),
                    line: 1,
                    category: "manifest_error".to_string(),
                    message: format!("malformed package manifest: {}", err.message),
                    recommendation: "Fix syntax error in semantic.toml manifest".to_string(),
                });
            }

            for (line_idx, line) in content.lines().enumerate() {
                let line_num = line_idx + 1;
                let trimmed = line.trim();
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
        } else if file.extension().is_some_and(|ext| ext == "sm") {
            // Canonical source admission and checking
            match crate::executable_bundle::prepare_source_text(content.clone()) {
                Ok((_, prepared)) => {
                    match prepared {
                        crate::executable_bundle::PreparedSource::RustLikeOwned(Err(e)) => {
                            let line = offset_to_line(&content, e.pos);
                            findings.push(MigrationCheckFinding {
                                file: file.clone(),
                                line,
                                category: "source_error".to_string(),
                                message: format!("source syntax error: {}", e.message),
                                recommendation:
                                    "Fix syntax error before checking migration compatibility"
                                        .to_string(),
                            });
                        }
                        crate::executable_bundle::PreparedSource::LogosOwned(Err(e)) => {
                            let line = offset_to_line(&content, e.pos);
                            findings.push(MigrationCheckFinding {
                                file: file.clone(),
                                line,
                                category: "source_error".to_string(),
                                message: format!("source syntax error: {}", e.message),
                                recommendation:
                                    "Fix syntax error before checking migration compatibility"
                                        .to_string(),
                            });
                        }
                        crate::executable_bundle::PreparedSource::Ambiguous { logos, rustlike }
                            if logos.is_err() && rustlike.is_err() =>
                        {
                            findings.push(MigrationCheckFinding {
                                file: file.clone(),
                                line: 1,
                                category: "source_error".to_string(),
                                message: "source syntax error: failed to parse under both grammars"
                                    .to_string(),
                                recommendation:
                                    "Fix syntax error before checking migration compatibility"
                                        .to_string(),
                            });
                        }
                        crate::executable_bundle::PreparedSource::NoSurfaceClaim
                            if !content.trim().is_empty() && !content.trim().starts_with("//") =>
                        {
                            findings.push(MigrationCheckFinding {
                            file: file.clone(),
                            line: 1,
                            category: "source_error".to_string(),
                            message: "source does not contain recognized Semantic grammar declarations".to_string(),
                            recommendation: "Ensure file contains valid Semantic declarations".to_string(),
                        });
                        }
                        _ => {}
                    }
                }
                Err(crate::executable_bundle::PrepareSourceError::Lex { error, .. }) => {
                    let line = offset_to_line(&content, error.pos);
                    findings.push(MigrationCheckFinding {
                        file: file.clone(),
                        line,
                        category: "source_error".to_string(),
                        message: format!("lexical error: {}", error.message),
                        recommendation: "Fix lexical errors before checking migration compatibility".to_string(),
                    });
                }
                Err(crate::executable_bundle::PrepareSourceError::Read(e)) => {
                    findings.push(MigrationCheckFinding {
                        file: file.clone(),
                        line: 1,
                        category: "source_error".to_string(),
                        message: format!("failed to read source: {}", e),
                        recommendation: "Verify file permissions and accessibility".to_string(),
                    });
                }
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
            }
        }
    }

    // Sort findings deterministically
    findings.sort_by(|a, b| {
        (&a.file, a.line, &a.category, &a.message).cmp(&(&b.file, b.line, &b.category, &b.message))
    });

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
        inspected_files: files_to_check,
        manifest_path: discovered_manifest,
        mutations_performed: 0,
        status: "dry-run preview completed successfully with zero mutations".to_string(),
    })
}
