//! Canonical compiled-artifact identity and inspection for Semantic.
//!
//! Provides deterministic artifact identity calculation, header decoding,
//! toolchain binding, and verifier admission reporting per SSF-10.

use sm_format::semcode_decode::{decode_semcode, DecodedSemCode};
use sm_format::semcode_format::*;
use sm_format::sha256::{format_hex, sha256};
use sm_verify::verify_semcode_token;
use std::path::Path;

/// Canonical build toolchain identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainIdentity {
    pub compiler_version: String,
    pub source_hash: String,
    pub enabled_features: Vec<String>,
}

impl Default for ToolchainIdentity {
    fn default() -> Self {
        Self {
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            source_hash: option_env!("SM_COMPILER_SOURCE_HASH")
                .unwrap_or("release-build")
                .to_string(),
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactHeaderSummary {
    pub magic: String,
    pub epoch: u16,
    pub revision: u16,
    pub capabilities: u32,
    pub capability_flags: Vec<String>,
}

/// Individual function summary in compiled artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactFunctionSummary {
    pub name: String,
    pub code_len: usize,
    pub string_count: usize,
    pub debug_symbol_count: usize,
    pub has_signature: bool,
    pub has_ownership: bool,
}

/// Verifier admission status bound to artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactVerifierBinding {
    pub admitted: bool,
    pub admission_code: Option<String>,
    pub diagnostics: Vec<String>,
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
            toolchain: ToolchainIdentity::default(),
            verifier: verifier_binding,
            signing: "unsigned".to_string(),
        })
    }

    /// Read file from disk and compute identity.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("failed to read artifact '{}': {}", path.display(), e))?;
        Self::from_bytes(&bytes)
    }

    /// Render human-readable inspection output.
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
        out.push_str(&format!(
            "Toolchain:      Semantic v{} (commit={}, features=[{}])\n",
            self.toolchain.compiler_version,
            self.toolchain.source_hash,
            self.toolchain.enabled_features.join(", ")
        ));
        out.push_str(&format!("Signing State:  {}\n", self.signing));
        out
    }

    /// Render deterministic JSON inspection output.
    pub fn render_json(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!(
            "  \"schema_version\": \"semantic-artifact-v1\",\n"
        ));
        json.push_str(&format!(
            "  \"artifact_hash\": \"{}\",\n",
            self.artifact_hash
        ));
        json.push_str(&format!("  \"size_bytes\": {},\n", self.size_bytes));
        json.push_str("  \"header\": {\n");
        json.push_str(&format!("    \"magic\": \"{}\",\n", self.header.magic));
        json.push_str(&format!("    \"epoch\": {},\n", self.header.epoch));
        json.push_str(&format!("    \"revision\": {},\n", self.header.revision));
        json.push_str(&format!(
            "    \"capabilities\": {},\n",
            self.header.capabilities
        ));
        json.push_str(&format!(
            "    \"capability_flags\": [{}]\n",
            self.header
                .capability_flags
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        json.push_str("  },\n");
        json.push_str(&format!("  \"function_count\": {},\n", self.function_count));
        json.push_str("  \"functions\": [\n");
        for (i, f) in self.functions.iter().enumerate() {
            let comma = if i + 1 < self.functions.len() {
                ","
            } else {
                ""
            };
            json.push_str(&format!(
                "    {{\"name\": \"{}\", \"code_len\": {}, \"string_count\": {}, \"debug_symbol_count\": {}, \"has_signature\": {}, \"has_ownership\": {}}}{}\n",
                f.name, f.code_len, f.string_count, f.debug_symbol_count, f.has_signature, f.has_ownership, comma
            ));
        }
        json.push_str("  ],\n");
        json.push_str(&format!("  \"adt_count\": {},\n", self.adt_count));
        json.push_str(&format!(
            "  \"adt_names\": [{}],\n",
            self.adt_names
                .iter()
                .map(|n| format!("\"{}\"", n))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        json.push_str(&format!(
            "  \"has_debug_symbols\": {},\n",
            self.has_debug_symbols
        ));
        json.push_str(&format!(
            "  \"has_ownership_tracks\": {},\n",
            self.has_ownership_tracks
        ));
        json.push_str(&format!("  \"has_signatures\": {},\n", self.has_signatures));
        json.push_str("  \"verifier\": {\n");
        json.push_str(&format!("    \"admitted\": {},\n", self.verifier.admitted));
        json.push_str(&format!(
            "    \"admission_code\": {},\n",
            self.verifier
                .admission_code
                .as_ref()
                .map(|c| format!("\"{}\"", c))
                .unwrap_or_else(|| "null".to_string())
        ));
        json.push_str(&format!(
            "    \"diagnostics\": [{}]\n",
            self.verifier
                .diagnostics
                .iter()
                .map(|d| format!("\"{}\"", d.replace('\"', "\\\"")))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        json.push_str("  },\n");
        json.push_str("  \"toolchain\": {\n");
        json.push_str(&format!(
            "    \"compiler_version\": \"{}\",\n",
            self.toolchain.compiler_version
        ));
        json.push_str(&format!(
            "    \"source_hash\": \"{}\",\n",
            self.toolchain.source_hash
        ));
        json.push_str(&format!(
            "    \"enabled_features\": [{}]\n",
            self.toolchain
                .enabled_features
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        json.push_str("  },\n");
        json.push_str(&format!("  \"signing\": \"{}\"\n", self.signing));
        json.push_str("}\n");
        json
    }
}
