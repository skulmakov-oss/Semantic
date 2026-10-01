//! Compatibility dimensions, artifact staleness detection, and migration preview for Semantic.
//!
//! Enforces explicit, fail-closed compatibility rules across language, manifest,
//! format, verifier, and runtime boundaries per SSF-10.

use crate::artifact_identity::ArtifactIdentity;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Canonical compatibility dimensions defined by Semantic Stable Foundation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityDimensions {
    pub source_version: &'static str,
    pub manifest_version: u32,
    pub diagnostic_schema: &'static str,
    pub stdlib_version: u32,
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
            diagnostic_schema: "sm-diagnostic-v1",
            stdlib_version: 0,
            semcode_format: "SEMCOD22",
            semcode_epoch: 0,
            semcode_revision: 23,
            verifier_profile: "VerifiedLocal",
            runtime_engine: "SVM-Deterministic-v1",
        }
    }
}

/// Explicit compatibility classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            // Older revisions (SEMCODE0..SEMCOD19) are deprecated
            CompatibilityClassification::Deprecated
        }
        _ => CompatibilityClassification::Incompatible,
    }
}

/// Staleness assessment between artifact and source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StalenessStatus {
    /// Artifact is fresh and up-to-date with source.
    Fresh,
    /// Source file was modified after artifact compilation.
    StaleSourceModified {
        artifact_mtime: SystemTime,
        source_mtime: SystemTime,
    },
    /// Source file could not be found.
    MissingSource(PathBuf),
    /// Artifact could not be found.
    MissingArtifact(PathBuf),
}

impl StalenessStatus {
    pub fn is_stale(&self) -> bool {
        matches!(self, Self::StaleSourceModified { .. })
    }

    pub fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh)
    }
}

/// Detect whether a compiled artifact is stale relative to its source.
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

    let art_meta = fs::metadata(artifact_path).map_err(|e| {
        format!(
            "failed to read metadata for '{}': {}",
            artifact_path.display(),
            e
        )
    })?;
    let src_meta = fs::metadata(source_path).map_err(|e| {
        format!(
            "failed to read metadata for '{}': {}",
            source_path.display(),
            e
        )
    })?;

    let art_mtime = art_meta.modified().map_err(|e| {
        format!(
            "failed to read mtime for '{}': {}",
            artifact_path.display(),
            e
        )
    })?;
    let src_mtime = src_meta.modified().map_err(|e| {
        format!(
            "failed to read mtime for '{}': {}",
            source_path.display(),
            e
        )
    })?;

    if src_mtime > art_mtime {
        Ok(StalenessStatus::StaleSourceModified {
            artifact_mtime: art_mtime,
            source_mtime: src_mtime,
        })
    } else {
        Ok(StalenessStatus::Fresh)
    }
}

/// Individual migration check finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationCheckFinding {
    pub file: PathBuf,
    pub line: usize,
    pub category: String,
    pub message: String,
    pub recommendation: String,
}

/// Summary report for non-destructive migration dry-run.
#[derive(Debug, Clone, PartialEq, Eq)]
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
        let manifest = target.join("semantic.toml");
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
                recommendation: "Migrate to canonical semantic.toml manifest format".to_string(),
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

            if trimmed.starts_with("format 0") {
                findings.push(MigrationCheckFinding {
                    file: file.clone(),
                    line: line_num,
                    category: "manifest_version".to_string(),
                    message: "legacy format version 0 declared".to_string(),
                    recommendation: "Upgrade format declaration to 'format 1'".to_string(),
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
