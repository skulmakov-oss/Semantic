#![allow(clippy::derivable_impls)]
#![allow(clippy::field_reassign_with_default)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ProfileVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProfileVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl Default for ProfileVersion {
    fn default() -> Self {
        Self::new(1, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AbiProfile {
    Core,
    GateSurface,
}

impl Default for AbiProfile {
    fn default() -> Self {
        Self::Core
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CompatibilityMode {
    Strict,
    LegacySupport,
}

impl Default for CompatibilityMode {
    fn default() -> Self {
        Self::Strict
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FeaturePolicy {
    pub allow_debug_symbols: bool,
    pub allow_f64_math: bool,
    pub allow_gate_surface: bool,
    pub allow_logos_surface: bool,
    pub allow_schema_surface: bool,
}

impl FeaturePolicy {
    pub const fn core() -> Self {
        Self {
            allow_debug_symbols: false,
            allow_f64_math: false,
            allow_gate_surface: false,
            allow_logos_surface: false,
            allow_schema_surface: false,
        }
    }
}

impl Default for FeaturePolicy {
    fn default() -> Self {
        Self::core()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CapabilityExpectations {
    pub require_debug_symbols: bool,
    pub require_f64_math: bool,
    pub require_gate_surface: bool,
}

impl CapabilityExpectations {
    pub const fn permissive() -> Self {
        Self {
            require_debug_symbols: false,
            require_f64_math: false,
            require_gate_surface: false,
        }
    }
}

impl Default for CapabilityExpectations {
    fn default() -> Self {
        Self::permissive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ParserProfile {
    pub identity: String,
    pub version: ProfileVersion,
    pub abi: AbiProfile,
    pub compatibility: CompatibilityMode,
    pub features: FeaturePolicy,
    pub capabilities: CapabilityExpectations,
    pub aliases: BTreeMap<String, String>,
}

impl Default for ParserProfile {
    fn default() -> Self {
        Self {
            identity: "semantic.core".to_string(),
            version: ProfileVersion::default(),
            abi: AbiProfile::default(),
            compatibility: CompatibilityMode::default(),
            features: FeaturePolicy::default(),
            capabilities: CapabilityExpectations::default(),
            aliases: BTreeMap::new(),
        }
    }
}

impl ParserProfile {
    pub fn foundation_default() -> Self {
        Self {
            identity: "semantic.foundation".to_string(),
            version: ProfileVersion::default(),
            abi: AbiProfile::GateSurface,
            compatibility: CompatibilityMode::LegacySupport,
            features: FeaturePolicy {
                allow_debug_symbols: true,
                allow_f64_math: true,
                allow_gate_surface: true,
                allow_logos_surface: true,
                allow_schema_surface: true,
            },
            capabilities: CapabilityExpectations::permissive(),
            aliases: BTreeMap::new(),
        }
    }

    pub fn add_alias(&mut self, raw: impl Into<String>, canonical: impl Into<String>) {
        self.aliases.insert(raw.into(), canonical.into());
    }

    pub fn normalize(&self, input: &str) -> String {
        let tokens = lex_tokens(input);
        if tokens.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        for (i, tok) in tokens.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            if let Some(mapped) = self.aliases.get(*tok) {
                out.push_str(mapped);
            } else {
                out.push_str(tok);
            }
        }
        out
    }

    #[cfg(feature = "std")]
    pub fn to_json(&self) -> Result<String, ProfileIoError> {
        serde_json::to_string_pretty(self).map_err(ProfileIoError::Json)
    }

    #[cfg(feature = "std")]
    pub fn from_json(json: &str) -> Result<Self, ProfileIoError> {
        serde_json::from_str(json).map_err(ProfileIoError::Json)
    }

    #[cfg(feature = "std")]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ProfileIoError> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(ProfileIoError::Io)
    }

    #[cfg(feature = "std")]
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ProfileIoError> {
        let json = std::fs::read_to_string(path).map_err(ProfileIoError::Io)?;
        Self::from_json(&json)
    }
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub enum ProfileIoError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

#[cfg(feature = "std")]
impl core::fmt::Display for ProfileIoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProfileIoError::Io(e) => write!(f, "I/O error: {}", e),
            ProfileIoError::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProfileIoError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrainingSample<'a> {
    pub input: &'a str,
    pub target: &'a str,
}

pub fn train_profile(samples: &[TrainingSample<'_>]) -> ParserProfile {
    let mut profile = ParserProfile::default();
    train_profile_in_place(&mut profile, samples);
    profile
}

pub fn train_profile_in_place(profile: &mut ParserProfile, samples: &[TrainingSample<'_>]) {
    for sample in samples {
        let raw_tokens = lex_tokens(sample.input);
        let target_tokens = lex_tokens(sample.target);
        if raw_tokens.len() != target_tokens.len() {
            continue;
        }

        for (raw, canonical) in raw_tokens.iter().zip(target_tokens.iter()) {
            if raw == canonical {
                continue;
            }
            if can_learn_alias(raw, canonical) {
                profile
                    .aliases
                    .entry((*raw).to_string())
                    .or_insert_with(|| (*canonical).to_string());
            }
        }
    }
}

fn can_learn_alias(raw: &str, canonical: &str) -> bool {
    is_alias_input_token(raw) && is_supported_canonical_token(canonical)
}

fn is_alias_input_token(tok: &str) -> bool {
    !tok.is_empty()
        && tok
            .as_bytes()
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
}

fn is_supported_canonical_token(tok: &str) -> bool {
    matches!(tok, "!" | "&" | "|" | "^" | "N" | "F" | "T" | "S")
}

fn lex_tokens(input: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        let ch = bytes[i];
        if ch.is_ascii_whitespace() {
            i += 1;
            continue;
        }

        if i + 1 < bytes.len() && ch == b'/' && bytes[i + 1] == b'/' {
            break;
        }
        if ch == b'#' {
            break;
        }

        if is_single_char_token(ch) {
            out.push(&input[i..i + 1]);
            i += 1;
            continue;
        }

        let start = i;
        i += 1;
        while i < bytes.len() {
            let c = bytes[i];
            if c.is_ascii_whitespace() || is_single_char_token(c) || c == b'#' {
                break;
            }
            if i + 1 < bytes.len() && c == b'/' && bytes[i + 1] == b'/' {
                break;
            }
            i += 1;
        }
        out.push(&input[start..i]);
    }

    out
}

fn is_single_char_token(ch: u8) -> bool {
    matches!(ch, b'(' | b')' | b'!' | b'&' | b'|' | b'^' | b'=')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn train_profile_learns_aliases_from_examples() {
        let samples = [
            TrainingSample {
                input: "out = a AND b",
                target: "out = a & b",
            },
            TrainingSample {
                input: "q = a OR b",
                target: "q = a | b",
            },
            TrainingSample {
                input: "x = NOT FALSE",
                target: "x = ! F",
            },
            TrainingSample {
                input: "y = TRUE XOR x",
                target: "y = T ^ x",
            },
        ];
        let profile = train_profile(&samples);

        assert_eq!(profile.identity, "semantic.core");
        assert_eq!(profile.abi, AbiProfile::Core);
        assert_eq!(profile.compatibility, CompatibilityMode::Strict);
        assert_eq!(profile.aliases.get("AND"), Some(&"&".to_string()));
        assert_eq!(profile.aliases.get("TRUE"), Some(&"T".to_string()));
        assert_eq!(profile.normalize("z = TRUE AND NOT a"), "z = T & ! a");
    }

    #[test]
    fn profile_roundtrip_keeps_contract_fields() {
        let mut profile = ParserProfile::default();
        profile.identity = "semantic.legacy".to_string();
        profile.version = ProfileVersion::new(1, 1);
        profile.abi = AbiProfile::GateSurface;
        profile.compatibility = CompatibilityMode::LegacySupport;
        profile.features.allow_debug_symbols = true;
        profile.capabilities.require_gate_surface = true;
        profile.add_alias("AND", "&");

        let json = profile.to_json().expect("serialize");
        let restored = ParserProfile::from_json(&json).expect("deserialize");

        assert_eq!(restored, profile);
    }
}
