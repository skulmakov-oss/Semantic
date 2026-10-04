#![allow(clippy::derivable_impls)]
#![allow(clippy::field_reassign_with_default)]
#![cfg_attr(not(feature = "std"), no_std)]

//! `ParserProfile`: the canonical policy contract for Semantic
//! language-surface acceptance (see `docs/spec/profile.md`).
//!
//! PB-01 contract (`docs/roadmap/phase_b/pb01_profile_contract_decision.md`):
//!
//! - one public baseline: `ParserProfile::default() == foundation_default()`;
//! - one validator: [`ParserProfile::validate`] (structural admission, used by
//!   `from_json`/`load_from_file`/`to_json`) and
//!   [`ParserProfile::validate_for_canonical_source`] (what canonical Semantic
//!   source admission additionally requires);
//! - one alias rule: [`validate_alias`], used by every alias entry path;
//! - `aliases` and every RESERVED policy field are non-authoritative for
//!   canonical Semantic source. Aliases belong to the retained TON618
//!   compatibility perimeter (the TON618 compatibility binary (`src/bin/`)), which owns legacy
//!   alias training and line normalization.

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct ProfileVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProfileVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// The only profile contract version this crate admits. Any other
    /// major is incompatible; a higher minor may carry semantics this
    /// build does not understand, so it also fails closed.
    pub const SUPPORTED: Self = Self::new(1, 0);
}

impl Default for ProfileVersion {
    fn default() -> Self {
        Self::SUPPORTED
    }
}

/// RESERVED / NON-AUTHORITATIVE. No production ABI boundary consumes this
/// value. Canonical source admission requires the baseline
/// (`GateSurface`) so a profile cannot appear to select a different ABI.
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

/// ACTIVE: gates legacy Logos directives in `sm-front`.
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

/// Surface policy.
///
/// ACTIVE (enforced by `sm-front`): `allow_f64_math`, `allow_logos_surface`,
/// `allow_schema_surface`.
///
/// RESERVED / NON-AUTHORITATIVE: `allow_debug_symbols`, `allow_gate_surface`.
/// They have no production consumer; canonical source admission requires
/// them to hold the baseline value `true`, so a profile can never appear to
/// deny something the compiler would still accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct FeaturePolicy {
    pub allow_debug_symbols: bool,
    pub allow_f64_math: bool,
    pub allow_gate_surface: bool,
    pub allow_logos_surface: bool,
    pub allow_schema_surface: bool,
}

impl FeaturePolicy {
    /// Every ACTIVE surface disabled; RESERVED fields at baseline.
    pub const fn core() -> Self {
        Self {
            allow_debug_symbols: true,
            allow_f64_math: false,
            allow_gate_surface: true,
            allow_logos_surface: false,
            allow_schema_surface: false,
        }
    }

    pub const fn foundation() -> Self {
        Self {
            allow_debug_symbols: true,
            allow_f64_math: true,
            allow_gate_surface: true,
            allow_logos_surface: true,
            allow_schema_surface: true,
        }
    }
}

impl Default for FeaturePolicy {
    fn default() -> Self {
        Self::core()
    }
}

/// RESERVED / NON-AUTHORITATIVE declarative metadata. Never a capability
/// grant: SemCode capabilities are derived from actual program usage only.
/// No `require_*` is enforced, so canonical source admission requires every
/// field to be `false` (the baseline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
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
#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct ParserProfile {
    pub identity: String,
    pub version: ProfileVersion,
    pub abi: AbiProfile,
    pub compatibility: CompatibilityMode,
    pub features: FeaturePolicy,
    pub capabilities: CapabilityExpectations,
    /// TON618 legacy alias vocabulary. RESERVED for canonical Semantic:
    /// canonical source admission rejects a non-empty map. Mutate through
    /// [`ParserProfile::add_alias`]; direct edits are re-checked by
    /// [`ParserProfile::validate`] at every serialization/consumer boundary.
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_unique_aliases")
    )]
    pub aliases: BTreeMap<String, String>,
}

/// The one public baseline: identical to [`ParserProfile::foundation_default`].
impl Default for ParserProfile {
    fn default() -> Self {
        Self::foundation_default()
    }
}

/// Rejected alias pair. Shared by every alias entry path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasError {
    /// Raw side is not a word (`[A-Za-z0-9_]+`) or is itself a canonical token.
    InvalidRaw { raw: String },
    /// Canonical side is outside the TON618 canonical vocabulary.
    InvalidCanonical { raw: String, canonical: String },
    /// `raw` is already mapped to a different canonical token.
    Conflict {
        raw: String,
        existing: String,
        proposed: String,
    },
}

impl core::fmt::Display for AliasError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AliasError::InvalidRaw { raw } => write!(f, "invalid alias raw token '{raw}'"),
            AliasError::InvalidCanonical { raw, canonical } => {
                write!(f, "invalid alias target '{canonical}' for '{raw}'")
            }
            AliasError::Conflict {
                raw,
                existing,
                proposed,
            } => write!(
                f,
                "conflicting alias '{raw}': already '{existing}', proposed '{proposed}'"
            ),
        }
    }
}

/// Profile admission failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    EmptyIdentity,
    UnsupportedVersion(ProfileVersion),
    Alias(AliasError),
    /// Canonical Semantic source admission does not apply aliases.
    AliasesNotCanonical,
    /// A RESERVED field holds a non-baseline value at canonical admission.
    ReservedField(&'static str),
}

impl core::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProfileError::EmptyIdentity => write!(f, "profile identity must not be empty"),
            ProfileError::UnsupportedVersion(v) => write!(
                f,
                "unsupported profile version {}.{} (supported: {}.{})",
                v.major,
                v.minor,
                ProfileVersion::SUPPORTED.major,
                ProfileVersion::SUPPORTED.minor
            ),
            ProfileError::Alias(e) => write!(f, "{e}"),
            ProfileError::AliasesNotCanonical => write!(
                f,
                "profile aliases are a TON618 legacy feature and are not supported by canonical Semantic source admission"
            ),
            ProfileError::ReservedField(name) => write!(
                f,
                "profile field '{name}' is reserved and non-authoritative; it must hold its baseline value"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AliasError {}
#[cfg(feature = "std")]
impl std::error::Error for ProfileError {}

/// TON618 canonical alias vocabulary.
fn is_canonical_alias_token(tok: &str) -> bool {
    matches!(tok, "!" | "&" | "|" | "^" | "N" | "F" | "T" | "S")
}

/// The one alias validity rule (pair-local; conflicts are checked against a
/// map by [`ParserProfile::add_alias`]).
pub fn validate_alias(raw: &str, canonical: &str) -> Result<(), AliasError> {
    let raw_is_word =
        !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
    if !raw_is_word || is_canonical_alias_token(raw) {
        return Err(AliasError::InvalidRaw {
            raw: raw.to_string(),
        });
    }
    if !is_canonical_alias_token(canonical) {
        return Err(AliasError::InvalidCanonical {
            raw: raw.to_string(),
            canonical: canonical.to_string(),
        });
    }
    Ok(())
}

impl ParserProfile {
    pub fn foundation_default() -> Self {
        Self {
            identity: "semantic.foundation".to_string(),
            version: ProfileVersion::SUPPORTED,
            abi: AbiProfile::GateSurface,
            compatibility: CompatibilityMode::LegacySupport,
            features: FeaturePolicy::foundation(),
            capabilities: CapabilityExpectations::permissive(),
            aliases: BTreeMap::new(),
        }
    }

    /// Explicit strict profile: every ACTIVE surface disabled and strict
    /// compatibility. (Formerly the implicit `Default`.)
    pub fn core() -> Self {
        Self {
            identity: "semantic.core".to_string(),
            compatibility: CompatibilityMode::Strict,
            features: FeaturePolicy::core(),
            ..Self::foundation_default()
        }
    }

    /// Insert an alias. Exact duplicates are idempotent; a different target
    /// for an existing raw token is a [`AliasError::Conflict`]. Never
    /// overwrites.
    pub fn add_alias(
        &mut self,
        raw: impl Into<String>,
        canonical: impl Into<String>,
    ) -> Result<(), AliasError> {
        let (raw, canonical) = (raw.into(), canonical.into());
        validate_alias(&raw, &canonical)?;
        match self.aliases.get(&raw) {
            Some(existing) if *existing == canonical => Ok(()),
            Some(existing) => Err(AliasError::Conflict {
                existing: existing.clone(),
                raw,
                proposed: canonical,
            }),
            None => {
                self.aliases.insert(raw, canonical);
                Ok(())
            }
        }
    }

    /// Structural admission: identity, version, alias validity.
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.identity.is_empty() {
            return Err(ProfileError::EmptyIdentity);
        }
        if self.version != ProfileVersion::SUPPORTED {
            return Err(ProfileError::UnsupportedVersion(self.version));
        }
        for (raw, canonical) in &self.aliases {
            validate_alias(raw, canonical).map_err(ProfileError::Alias)?;
        }
        Ok(())
    }

    /// What canonical Semantic source admission (`sm-front`) requires on top
    /// of [`ParserProfile::validate`]: no aliases, RESERVED fields at baseline.
    pub fn validate_for_canonical_source(&self) -> Result<(), ProfileError> {
        self.validate()?;
        if !self.aliases.is_empty() {
            return Err(ProfileError::AliasesNotCanonical);
        }
        let reserved = [
            ("abi", self.abi == AbiProfile::GateSurface),
            (
                "features.allow_debug_symbols",
                self.features.allow_debug_symbols,
            ),
            (
                "features.allow_gate_surface",
                self.features.allow_gate_surface,
            ),
            (
                "capabilities.require_debug_symbols",
                !self.capabilities.require_debug_symbols,
            ),
            (
                "capabilities.require_f64_math",
                !self.capabilities.require_f64_math,
            ),
            (
                "capabilities.require_gate_surface",
                !self.capabilities.require_gate_surface,
            ),
        ];
        match reserved.iter().find(|(_, at_baseline)| !at_baseline) {
            Some((name, _)) => Err(ProfileError::ReservedField(name)),
            None => Ok(()),
        }
    }

    #[cfg(feature = "std")]
    pub fn to_json(&self) -> Result<String, ProfileIoError> {
        self.validate().map_err(ProfileIoError::Invalid)?;
        serde_json::to_string_pretty(self).map_err(ProfileIoError::Json)
    }

    /// Semantic admission, not a raw decode: unknown fields (at any level),
    /// duplicate alias keys, unsupported versions and invalid aliases fail.
    #[cfg(feature = "std")]
    pub fn from_json(json: &str) -> Result<Self, ProfileIoError> {
        let profile: Self = serde_json::from_str(json).map_err(ProfileIoError::Json)?;
        profile.validate().map_err(ProfileIoError::Invalid)?;
        Ok(profile)
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

/// serde's map decoding is last-key-wins; a duplicated alias key in JSON is
/// conflicting evidence, so it fails closed instead.
#[cfg(feature = "std")]
fn deserialize_unique_aliases<'de, D>(de: D) -> Result<BTreeMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("an alias map without duplicate keys")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some((k, v)) = map.next_entry::<String, String>()? {
                if out.contains_key(&k) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate alias key '{k}'"
                    )));
                }
                out.insert(k, v);
            }
            Ok(out)
        }
    }
    de.deserialize_map(V)
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub enum ProfileIoError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(ProfileError),
}

#[cfg(feature = "std")]
impl core::fmt::Display for ProfileIoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProfileIoError::Io(e) => write!(f, "I/O error: {}", e),
            ProfileIoError::Json(e) => write!(f, "JSON error: {}", e),
            ProfileIoError::Invalid(e) => write!(f, "invalid profile: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProfileIoError {}
