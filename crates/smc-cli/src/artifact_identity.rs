//! Canonical compiled-artifact identity, provenance, and inspection for Semantic.
//!
//! Provides deterministic artifact identity calculation, header decoding,
//! producer provenance binding, inspecting toolchain distinction, and verifier
//! admission reporting per SSF-10.

use serde::{Deserialize, Serialize};
use sm_format::semcode_decode::{decode_semcode, DecodedSemCode};
use sm_format::semcode_format::*;
use sm_format::sha256::{format_hex, sha256, sha256_prefixed_hex};
use sm_verify::verify_semcode_token;
use std::fs;
use std::path::{Path, PathBuf};

/// Canonical toolchain identity of the producing compiler stored in provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProducerToolchainIdentity {
    pub compiler_name: String,
    pub compiler_version: String,
    pub build_target: String,
    /// Build source fingerprint (derived from SM_COMPILER_SOURCE_HASH, an FNV hash of the compiler source closure).
    #[serde(alias = "commit_hash")]
    pub source_fingerprint: String,
    /// Optional Git commit SHA if built from a git checkout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    pub profile: String,
    pub enabled_features: Vec<String>,
}

impl ProducerToolchainIdentity {
    pub fn commit_hash(&self) -> &str {
        &self.source_fingerprint
    }
}

impl Default for ProducerToolchainIdentity {
    fn default() -> Self {
        Self {
            compiler_name: "smc".to_string(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            build_target: option_env!("TARGET")
                .unwrap_or("x86_64-pc-windows-msvc")
                .to_string(),
            source_fingerprint: option_env!("SM_COMPILER_SOURCE_HASH")
                .unwrap_or("release-build")
                .to_string(),
            git_commit: option_env!("SM_COMPILER_GIT_COMMIT")
                .or(option_env!("GIT_HASH"))
                .map(ToString::to_string),
            profile: if cfg!(debug_assertions) {
                "dev".to_string()
            } else {
                "release".to_string()
            },
            enabled_features: option_env!("SM_ENABLED_FEATURES")
                .unwrap_or("std,profile-rust,profile-logos,debug-symbols")
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect(),
        }
    }
}

/// Source and project identity bound into artifact provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceSourceIdentity {
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub entry_file: String,
    pub source_hash: String,
    pub manifest_hash: Option<String>,
}

/// Contract and compatibility dimensions bound into artifact provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceContractIdentity {
    pub semcode_format: String,
    pub semcode_epoch: u16,
    pub semcode_revision: u16,
    pub verifier_profile: String,
    pub runtime_profile: String,
    pub stdlib_version: String,
    pub diagnostic_contract: String,
}

impl Default for ProvenanceContractIdentity {
    fn default() -> Self {
        Self {
            semcode_format: "SEMCOD22".to_string(),
            semcode_epoch: 0,
            semcode_revision: 23,
            verifier_profile: "verifier-canonical-v1".to_string(),
            runtime_profile: "deterministic-v1".to_string(),
            stdlib_version: "semantic-stdlib-v1".to_string(),
            diagnostic_contract: "semantic.diagnostics".to_string(),
        }
    }
}

/// Canonical provenance record produced by the compiler and bound to the artifact SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactProvenance {
    pub schema_version: u32,
    pub artifact_hash: String,
    pub artifact_size_bytes: usize,
    pub producer: ProducerToolchainIdentity,
    pub source: ProvenanceSourceIdentity,
    pub contract: ProvenanceContractIdentity,
}

/// Status of provenance resolution for an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceStatus {
    /// Provenance was successfully resolved, parsed, and its artifact hash matches.
    Recorded(Box<ArtifactProvenance>),
    /// No companion provenance record was found next to the artifact.
    Missing,
    /// Provenance was found, but its recorded artifact hash did not match the artifact.
    CorruptedMismatch {
        expected_hash: String,
        actual_hash: String,
    },
    /// Provenance was found, but was malformed or unsupported.
    Unsupported(String),
}

/// Canonical toolchain identity of the currently running inspector process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainIdentity {
    pub compiler_version: String,
    #[serde(alias = "source_hash")]
    pub source_fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    pub enabled_features: Vec<String>,
}

impl ToolchainIdentity {
    pub fn source_hash(&self) -> &str {
        &self.source_fingerprint
    }
}

impl Default for ToolchainIdentity {
    fn default() -> Self {
        Self {
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            source_fingerprint: option_env!("SM_COMPILER_SOURCE_HASH")
                .unwrap_or("release-build")
                .to_string(),
            git_commit: option_env!("SM_COMPILER_GIT_COMMIT")
                .or(option_env!("GIT_HASH"))
                .map(ToString::to_string),
            enabled_features: option_env!("SM_ENABLED_FEATURES")
                .unwrap_or("std,profile-rust,profile-logos,debug-symbols")
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect(),
        }
    }
}

/// Decoded SemCode header summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactHeaderSummary {
    pub magic: String,
    pub epoch: u16,
    pub revision: u16,
    pub capabilities: u32,
    pub capability_flags: Vec<String>,
}

/// Individual function summary in compiled artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFunctionSummary {
    pub name: String,
    pub code_len: usize,
    pub string_count: usize,
    pub debug_symbol_count: usize,
    pub has_signature: bool,
    pub has_ownership: bool,
}

/// Verifier admission status bound to artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactVerifierBinding {
    pub admitted: bool,
    pub admission_code: Option<String>,
    pub diagnostics: Vec<String>,
}

/// Canonical inspection report format serialized to standard JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactInspectionReport {
    pub schema_version: String,
    pub artifact_hash: String,
    pub size_bytes: usize,
    pub header: ArtifactHeaderSummary,
    pub function_count: usize,
    pub functions: Vec<ArtifactFunctionSummary>,
    pub adt_count: usize,
    pub adt_names: Vec<String>,
    pub has_debug_symbols: bool,
    pub has_ownership_tracks: bool,
    pub has_signatures: bool,
    pub verifier: ArtifactVerifierBinding,
    pub signing: String,
    pub producer_provenance: Option<ProducerProvenanceReport>,
    pub inspecting_toolchain: ToolchainReport,
    pub toolchain: ToolchainReport,
}

/// Producer provenance section in the inspection report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProducerProvenanceReport {
    pub status: String,
    pub compiler_name: Option<String>,
    pub compiler_version: Option<String>,
    pub build_target: Option<String>,
    pub source_fingerprint: Option<String>,
    pub commit_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    pub profile: Option<String>,
    pub source_hash: Option<String>,
    pub entry_file: Option<String>,
    pub manifest_hash: Option<String>,
    pub package_name: Option<String>,
}

/// Toolchain section in the inspection report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainReport {
    pub compiler_version: String,
    pub source_fingerprint: String,
    pub source_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    pub enabled_features: Vec<String>,
}

/// Comprehensive canonical compiled-artifact identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactIdentity {
    pub artifact_hash: String,
    pub size_bytes: usize,
    pub header: ArtifactHeaderSummary,
    pub function_count: usize,
    pub functions: Vec<ArtifactFunctionSummary>,
    pub adt_count: usize,
    pub adt_names: Vec<String>,
    pub has_debug_symbols: bool,
    pub has_ownership_tracks: bool,
    pub has_signatures: bool,
    /// Provenance of the original producing compiler if attached via companion record.
    pub producer_provenance: Option<ArtifactProvenance>,
    /// Status of provenance resolution.
    pub provenance_status: ProvenanceStatus,
    /// Identity of the toolchain executing this inspection.
    pub inspecting_toolchain: ToolchainIdentity,
    /// Backwards-compatible alias for the inspecting toolchain.
    pub toolchain: ToolchainIdentity,
    pub verifier: ArtifactVerifierBinding,
    pub signing: String,
}

impl ArtifactIdentity {
    /// Compute canonical artifact identity from raw SemCode bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let digest = sha256(bytes);
        let artifact_hash = format!("sha256:{}", format_hex(&digest));
        let size_bytes = bytes.len();

        if bytes.len() < 8 {
            return Err("artifact is shorter than 8-byte SemCode header".to_string());
        }

        let magic_bytes = &bytes[0..8];
        let magic_str = String::from_utf8_lossy(magic_bytes).to_string();

        let decoded_result = decode_semcode(bytes);
        let (header_summary, function_summaries, adt_names, has_dbg, has_own, has_sig) =
            match &decoded_result {
                Ok(DecodedSemCode {
                    header,
                    adt_descriptors,
                    functions,
                }) => {
                    let mut cap_flags = Vec::new();
                    if header.capabilities & CAP_DEBUG_SYMBOLS != 0 {
                        cap_flags.push("DEBUG_SYMBOLS".to_string());
                    }
                    if header.capabilities & CAP_OWNERSHIP_PATHS != 0 {
                        cap_flags.push("OWNERSHIP_PATHS".to_string());
                    }
                    if header.capabilities & CAP_OWNERSHIP_SEQUENCE_PATHS != 0 {
                        cap_flags.push("OWNERSHIP_SEQUENCE_PATHS".to_string());
                    }
                    if header.capabilities & CAP_OWNERSHIP_ADT_BORROW_PATHS != 0 {
                        cap_flags.push("OWNERSHIP_ADT_BORROW_PATHS".to_string());
                    }
                    if header.rev >= SEMCODE_ADT_DESCRIPTOR_MIN_REVISION {
                        cap_flags.push("ADT_DESCRIPTORS".to_string());
                    }
                    if header.rev >= SEMCODE_SIGNATURE_MIN_REVISION {
                        cap_flags.push("CALLABLE_SIGNATURES".to_string());
                    }

                    let adts = adt_descriptors
                        .as_ref()
                        .map(|table| {
                            table
                                .descriptors()
                                .iter()
                                .map(|desc| desc.name().to_string())
                                .collect()
                        })
                        .unwrap_or_default();

                    let mut fn_summaries = Vec::with_capacity(functions.len());
                    let mut dbg_found = false;
                    let mut own_found = false;
                    let mut sig_found = false;

                    for f in functions {
                        if f.has_debug_section || !f.debug_symbols.is_empty() {
                            dbg_found = true;
                        }
                        if f.has_ownership_section {
                            own_found = true;
                        }
                        if f.signature.is_some() {
                            sig_found = true;
                        }
                        fn_summaries.push(ArtifactFunctionSummary {
                            name: f.name.clone(),
                            code_len: f.code_len,
                            string_count: f.strings.len(),
                            debug_symbol_count: f.debug_symbols.len(),
                            has_signature: f.signature.is_some(),
                            has_ownership: f.has_ownership_section,
                        });
                    }

                    (
                        ArtifactHeaderSummary {
                            magic: String::from_utf8_lossy(&header.magic).to_string(),
                            epoch: header.epoch,
                            revision: header.rev,
                            capabilities: header.capabilities,
                            capability_flags: cap_flags,
                        },
                        fn_summaries,
                        adts,
                        dbg_found,
                        own_found,
                        sig_found,
                    )
                }
                Err(_) => (
                    ArtifactHeaderSummary {
                        magic: magic_str,
                        epoch: 0,
                        revision: 0,
                        capabilities: 0,
                        capability_flags: Vec::new(),
                    },
                    Vec::new(),
                    Vec::new(),
                    false,
                    false,
                    false,
                ),
            };

        // Verifier admission check
        let verifier_binding = match verify_semcode_token(bytes) {
            Ok(token) => {
                assert!(token.matches_artifact(bytes));
                ArtifactVerifierBinding {
                    admitted: true,
                    admission_code: None,
                    diagnostics: Vec::new(),
                }
            }
            Err(report) => {
                let diags: Vec<String> = report
                    .diagnostics
                    .iter()
                    .map(|d| format!("{:?}: {}", d.code, d.message))
                    .collect();
                let first_code = report.diagnostics.first().map(|d| format!("{:?}", d.code));
                ArtifactVerifierBinding {
                    admitted: false,
                    admission_code: first_code,
                    diagnostics: diags,
                }
            }
        };

        let inspecting = ToolchainIdentity::default();

        Ok(ArtifactIdentity {
            artifact_hash,
            size_bytes,
            header: header_summary,
            function_count: function_summaries.len(),
            functions: function_summaries,
            adt_count: adt_names.len(),
            adt_names,
            has_debug_symbols: has_dbg,
            has_ownership_tracks: has_own,
            has_signatures: has_sig,
            producer_provenance: None,
            provenance_status: ProvenanceStatus::Missing,
            inspecting_toolchain: inspecting.clone(),
            toolchain: inspecting,
            verifier: verifier_binding,
            signing: "unsigned".to_string(),
        })
    }

    /// Read file from disk, compute identity, and resolve companion provenance if present.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path)
            .map_err(|e| format!("failed to read artifact '{}': {}", path.display(), e))?;
        let mut identity = Self::from_bytes(&bytes)?;

        if let Some(prov_path) = find_companion_provenance_path(path) {
            match fs::read_to_string(&prov_path) {
                Ok(content) => match serde_json::from_str::<ArtifactProvenance>(&content) {
                    Ok(prov) => {
                        if prov.artifact_hash == identity.artifact_hash {
                            identity.provenance_status =
                                ProvenanceStatus::Recorded(Box::new(prov.clone()));
                            identity.producer_provenance = Some(prov);
                        } else {
                            identity.provenance_status = ProvenanceStatus::CorruptedMismatch {
                                expected_hash: prov.artifact_hash,
                                actual_hash: identity.artifact_hash.clone(),
                            };
                        }
                    }
                    Err(e) => {
                        identity.provenance_status = ProvenanceStatus::Unsupported(format!(
                            "failed to parse provenance '{}': {}",
                            prov_path.display(),
                            e
                        ));
                    }
                },
                Err(e) => {
                    identity.provenance_status = ProvenanceStatus::Unsupported(format!(
                        "failed to read provenance '{}': {}",
                        prov_path.display(),
                        e
                    ));
                }
            }
        } else {
            identity.provenance_status = ProvenanceStatus::Missing;
        }

        Ok(identity)
    }

    /// Explicitly bind provenance to an identity, checking cryptographic hash match.
    pub fn with_provenance(mut self, provenance: ArtifactProvenance) -> Result<Self, String> {
        if provenance.artifact_hash != self.artifact_hash {
            return Err(format!(
                "provenance artifact hash '{}' does not match artifact identity '{}'",
                provenance.artifact_hash, self.artifact_hash
            ));
        }
        self.provenance_status = ProvenanceStatus::Recorded(Box::new(provenance.clone()));
        self.producer_provenance = Some(provenance);
        Ok(self)
    }

    /// Render human-readable inspection output, clearly distinguishing producer and inspector.
    pub fn render_human(&self, path_display: Option<&str>) -> String {
        let mut out = String::new();
        if let Some(path) = path_display {
            out.push_str(&format!("Artifact: {}\n", path));
        }
        out.push_str(&format!("Canonical Hash: {}\n", self.artifact_hash));
        out.push_str(&format!("Payload Size:   {} bytes\n", self.size_bytes));
        out.push_str(&format!(
            "SemCode Header: {} (epoch={}, rev={}, caps=0x{:08x})\n",
            self.header.magic, self.header.epoch, self.header.revision, self.header.capabilities
        ));
        if !self.header.capability_flags.is_empty() {
            out.push_str(&format!(
                "Capabilities:   {}\n",
                self.header.capability_flags.join(", ")
            ));
        }
        out.push_str(&format!("Functions:      {}\n", self.function_count));
        for f in &self.functions {
            out.push_str(&format!(
                "  - fn {} (code={}B, strings={}, debug_symbols={})\n",
                f.name, f.code_len, f.string_count, f.debug_symbol_count
            ));
        }
        out.push_str(&format!("ADT Types:      {}\n", self.adt_count));
        for name in &self.adt_names {
            out.push_str(&format!("  - adt {}\n", name));
        }
        out.push_str(&format!(
            "Verifier:       {}\n",
            if self.verifier.admitted {
                "Admitted (Pass)"
            } else {
                "Rejected (Fail)"
            }
        ));
        if !self.verifier.admitted {
            for diag in &self.verifier.diagnostics {
                out.push_str(&format!("  - diagnostic: {}\n", diag));
            }
        }
        out.push_str(&format!("Signing State:  {}\n", self.signing));

        out.push_str("\n--- Producer Toolchain (from Provenance) ---\n");
        match &self.provenance_status {
            ProvenanceStatus::Recorded(prov) => {
                out.push_str("Trust Model:    Integrity-only sidecar bound to artifact SHA-256 (unsigned)\n");
                out.push_str(&format!(
                    "Compiler:       {} v{} (target={}, fingerprint={}, profile={})\n",
                    prov.producer.compiler_name,
                    prov.producer.compiler_version,
                    prov.producer.build_target,
                    prov.producer.source_fingerprint,
                    prov.producer.profile,
                ));
                out.push_str(&format!(
                    "Features:       [{}]\n",
                    prov.producer.enabled_features.join(", ")
                ));
                out.push_str(&format!(
                    "Source Entry:   {} (source_hash={})\n",
                    prov.source.entry_file, prov.source.source_hash
                ));
                if let Some(pkg) = &prov.source.package_name {
                    out.push_str(&format!(
                        "Package:        {} v{}\n",
                        pkg,
                        prov.source.package_version.as_deref().unwrap_or("0.1.0")
                    ));
                }
                if let Some(mh) = &prov.source.manifest_hash {
                    out.push_str(&format!("Manifest Hash:  {}\n", mh));
                }
                out.push_str(&format!(
                    "Contract:       format={}, verifier={}, runtime={}, stdlib={}, diagnostics={}\n",
                    prov.contract.semcode_format,
                    prov.contract.verifier_profile,
                    prov.contract.runtime_profile,
                    prov.contract.stdlib_version,
                    prov.contract.diagnostic_contract,
                ));
            }
            ProvenanceStatus::Missing => {
                out.push_str("Status:         [UNRECORDED - NO PROVENANCE ATTACHED]\n");
                out.push_str("Note:           Artifact has no companion provenance record. Original producing toolchain cannot be verified.\n");
            }
            ProvenanceStatus::CorruptedMismatch {
                expected_hash,
                actual_hash,
            } => {
                out.push_str("Status:         [PROVENANCE MISMATCH - CORRUPTED]\n");
                out.push_str(&format!("Expected Hash:  {}\n", expected_hash));
                out.push_str(&format!("Actual Hash:    {}\n", actual_hash));
            }
            ProvenanceStatus::Unsupported(reason) => {
                out.push_str(&format!(
                    "Status:         [UNSUPPORTED PROVENANCE: {}]\n",
                    reason
                ));
            }
        }

        out.push_str("\n--- Inspecting Toolchain (Current Process) ---\n");
        out.push_str(&format!(
            "Inspector:      Semantic v{} (fingerprint={}, features=[{}])\n",
            self.inspecting_toolchain.compiler_version,
            self.inspecting_toolchain.source_fingerprint,
            self.inspecting_toolchain.enabled_features.join(", ")
        ));

        out
    }

    /// Render deterministic JSON inspection output.
    pub fn render_json(&self) -> String {
        let producer_provenance = match &self.producer_provenance {
            Some(prov) => Some(ProducerProvenanceReport {
                status: "recorded".to_string(),
                compiler_name: Some(prov.producer.compiler_name.clone()),
                compiler_version: Some(prov.producer.compiler_version.clone()),
                build_target: Some(prov.producer.build_target.clone()),
                source_fingerprint: Some(prov.producer.source_fingerprint.clone()),
                commit_hash: Some(prov.producer.source_fingerprint.clone()),
                git_commit: prov.producer.git_commit.clone(),
                profile: Some(prov.producer.profile.clone()),
                source_hash: Some(prov.source.source_hash.clone()),
                entry_file: Some(prov.source.entry_file.clone()),
                manifest_hash: prov.source.manifest_hash.clone(),
                package_name: prov.source.package_name.clone(),
            }),
            None => {
                let status_str = match &self.provenance_status {
                    ProvenanceStatus::Missing => "missing",
                    ProvenanceStatus::CorruptedMismatch { .. } => "corrupted_mismatch",
                    ProvenanceStatus::Unsupported(_) => "unsupported",
                    ProvenanceStatus::Recorded(_) => "recorded",
                };
                Some(ProducerProvenanceReport {
                    status: status_str.to_string(),
                    compiler_name: None,
                    compiler_version: None,
                    build_target: None,
                    source_fingerprint: None,
                    commit_hash: None,
                    git_commit: None,
                    profile: None,
                    source_hash: None,
                    entry_file: None,
                    manifest_hash: None,
                    package_name: None,
                })
            }
        };

        let inspecting = ToolchainReport {
            compiler_version: self.inspecting_toolchain.compiler_version.clone(),
            source_fingerprint: self.inspecting_toolchain.source_fingerprint.clone(),
            source_hash: self.inspecting_toolchain.source_fingerprint.clone(),
            git_commit: self.inspecting_toolchain.git_commit.clone(),
            enabled_features: self.inspecting_toolchain.enabled_features.clone(),
        };

        let toolchain = ToolchainReport {
            compiler_version: self.toolchain.compiler_version.clone(),
            source_fingerprint: self.toolchain.source_fingerprint.clone(),
            source_hash: self.toolchain.source_fingerprint.clone(),
            git_commit: self.toolchain.git_commit.clone(),
            enabled_features: self.toolchain.enabled_features.clone(),
        };

        let report = ArtifactInspectionReport {
            schema_version: "semantic-artifact-v1".to_string(),
            artifact_hash: self.artifact_hash.clone(),
            size_bytes: self.size_bytes,
            header: self.header.clone(),
            function_count: self.function_count,
            functions: self.functions.clone(),
            adt_count: self.adt_count,
            adt_names: self.adt_names.clone(),
            has_debug_symbols: self.has_debug_symbols,
            has_ownership_tracks: self.has_ownership_tracks,
            has_signatures: self.has_signatures,
            verifier: self.verifier.clone(),
            signing: self.signing.clone(),
            producer_provenance,
            inspecting_toolchain: inspecting,
            toolchain,
        };

        serde_json::to_string_pretty(&report)
            .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }
}

/// Compute the primary companion provenance file path for an artifact.
pub fn companion_provenance_path(artifact_path: &Path) -> PathBuf {
    let mut prov_name = artifact_path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    prov_name.push(".provenance.json");
    artifact_path.with_file_name(prov_name)
}

/// Search for companion provenance path (either `<file>.smc.provenance.json` or `<file>.provenance.json`).
pub fn find_companion_provenance_path(artifact_path: &Path) -> Option<PathBuf> {
    let direct = companion_provenance_path(artifact_path);
    if direct.is_file() {
        return Some(direct);
    }
    let stem_prov = artifact_path.with_extension("provenance.json");
    if stem_prov.is_file() {
        return Some(stem_prov);
    }
    None
}

/// Write companion provenance JSON file to disk.
pub fn save_companion_provenance(
    artifact_path: &Path,
    provenance: &ArtifactProvenance,
) -> Result<PathBuf, String> {
    let prov_path = companion_provenance_path(artifact_path);
    let json_text = serde_json::to_string_pretty(provenance)
        .map_err(|e| format!("failed to serialize provenance: {}", e))?;
    fs::write(&prov_path, json_text).map_err(|e| {
        format!(
            "failed to write provenance '{}': {}",
            prov_path.display(),
            e
        )
    })?;
    Ok(prov_path)
}

/// Find project root directory by searching ancestors for `semantic.toml` or `Semantic.toml`.
pub(crate) fn find_project_root_ancestor(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if current.join("semantic.toml").is_file() || current.join("Semantic.toml").is_file() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Generate and save companion provenance for a compiled artifact.
pub fn generate_and_save_companion_provenance(
    artifact_path: &Path,
    artifact_bytes: &[u8],
    source_path: &Path,
    package_name: Option<String>,
    package_version: Option<String>,
) -> Result<PathBuf, String> {
    let artifact_hash = sha256_prefixed_hex(artifact_bytes);

    let project_root = if source_path.is_dir() {
        Some(source_path.to_path_buf())
    } else {
        find_project_root_ancestor(source_path)
    };

    let (source_bytes, manifest_hash, resolved_package_name) = match project_root {
        Some(ref root) => {
            let mut combined = Vec::new();
            collect_project_source_bytes(root, &mut combined)?;
            let manifest_path = if root.join("semantic.toml").is_file() {
                root.join("semantic.toml")
            } else if root.join("Semantic.toml").is_file() {
                root.join("Semantic.toml")
            } else {
                PathBuf::new()
            };
            let (m_hash, pkg_name) = if manifest_path.is_file() {
                let m_bytes = fs::read(&manifest_path).map_err(|e| {
                    format!(
                        "failed to read manifest '{}': {}",
                        manifest_path.display(),
                        e
                    )
                })?;
                let mut found_pkg = None;
                if let Ok(m_str) = std::str::from_utf8(&m_bytes) {
                    for line in m_str.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("name =") {
                            let name = trimmed
                                .strip_prefix("name =")
                                .unwrap_or("")
                                .trim()
                                .trim_matches('"');
                            if !name.is_empty() {
                                found_pkg = Some(name.to_string());
                            }
                        }
                    }
                }
                (Some(sha256_prefixed_hex(&m_bytes)), found_pkg)
            } else {
                (None, None)
            };
            (combined, m_hash, package_name.or(pkg_name))
        }
        None => {
            let s_bytes = if source_path.is_file() {
                fs::read(source_path).map_err(|e| {
                    format!("failed to read source '{}': {}", source_path.display(), e)
                })?
            } else {
                let mut combined = Vec::new();
                collect_project_source_bytes(source_path, &mut combined)?;
                combined
            };
            (s_bytes, None, package_name)
        }
    };

    let source_hash = sha256_prefixed_hex(&source_bytes);

    let provenance = ArtifactProvenance {
        schema_version: 1,
        artifact_hash,
        artifact_size_bytes: artifact_bytes.len(),
        producer: ProducerToolchainIdentity::default(),
        source: ProvenanceSourceIdentity {
            package_name: resolved_package_name,
            package_version,
            entry_file: source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("entry.sm")
                .to_string(),
            source_hash,
            manifest_hash,
        },
        contract: ProvenanceContractIdentity::default(),
    };

    save_companion_provenance(artifact_path, &provenance)
}

/// Securely collect all `.sm` files under a project root, enforcing containment,
/// reparse-point rejection, and cycle prevention.
pub(crate) fn collect_project_files_secure(
    _root: &Path,
    canonical_root: &Path,
    current_dir: &Path,
    visited_dirs: &mut std::collections::HashSet<PathBuf>,
    out_sm: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let canonical_current = fs::canonicalize(current_dir).map_err(|e| {
        format!(
            "failed to canonicalize directory '{}': {}",
            current_dir.display(),
            e
        )
    })?;

    if !canonical_current.starts_with(canonical_root) {
        return Err(format!(
            "security violation: directory '{}' escapes project root '{}'",
            current_dir.display(),
            canonical_root.display()
        ));
    }

    if !visited_dirs.insert(canonical_current.clone()) {
        return Err(format!(
            "security violation: recursive directory cycle detected at '{}'",
            current_dir.display()
        ));
    }

    let read_dir = fs::read_dir(current_dir).map_err(|e| {
        format!("failed to read directory '{}': {}", current_dir.display(), e)
    })?;

    let mut entries: Vec<_> = read_dir.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip hidden entries and build target directory
        if name.starts_with('.') || name == "target" {
            continue;
        }

        let is_reparse = crate::package_manifest::path_is_reparse(&path).unwrap_or(false);
        if is_reparse {
            match fs::canonicalize(&path) {
                Ok(canon) => {
                    if !canon.starts_with(canonical_root) {
                        return Err(format!(
                            "security violation: reparse path '{}' escapes project root '{}'",
                            path.display(),
                            canonical_root.display()
                        ));
                    }
                    if path.is_dir() {
                        if visited_dirs.contains(&canon) {
                            return Err(format!(
                                "security violation: recursive symlink cycle detected at '{}'",
                                path.display()
                            ));
                        }
                        return Err(format!(
                            "security violation: symlinked directory '{}' is forbidden in project traversal",
                            path.display()
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "security violation: failed to resolve reparse path '{}': {}",
                        path.display(),
                        e
                    ));
                }
            }
        }

        if path.is_dir() {
            collect_project_files_secure(_root, canonical_root, &path, visited_dirs, out_sm)?;
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "sm") {
            out_sm.push(path);
        }
    }

    Ok(())
}

/// Collect sorted project source bytes across all `.sm` files, normalized with relative paths.
pub(crate) fn collect_project_source_bytes(dir: &Path, out: &mut Vec<u8>) -> Result<(), String> {
    let canonical_root = fs::canonicalize(dir)
        .map_err(|e| format!("failed to canonicalize directory '{}': {}", dir.display(), e))?;
    let mut visited_dirs = std::collections::HashSet::new();
    let mut entries = Vec::new();
    collect_project_files_secure(dir, &canonical_root, dir, &mut visited_dirs, &mut entries)?;
    entries.sort();

    for path in entries {
        let b =
            fs::read(&path).map_err(|e| format!("failed to read '{}': {}", path.display(), e))?;
        let rel = path.strip_prefix(dir).unwrap_or(&path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        out.extend_from_slice(rel_str.as_bytes());
        out.push(0);
        out.extend_from_slice(&b);
        out.push(0);
    }
    Ok(())
}
