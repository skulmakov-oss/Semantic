//! Classification enums attached to tools, requests, and replies.

use std::fmt;

/// Where/how a tool worker executes. Hub v0 implements only `InProcess`; the
/// other variants exist so the wire/API contract does not need to change
/// when subprocess/WASM execution is added later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubExecutionMode {
    InProcess,
    Subprocess,
    Wasm,
    Remote,
}

impl HubExecutionMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HubExecutionMode::InProcess => "InProcess",
            HubExecutionMode::Subprocess => "Subprocess",
            HubExecutionMode::Wasm => "Wasm",
            HubExecutionMode::Remote => "Remote",
        }
    }
}

impl fmt::Display for HubExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Honest determinism classification. Never inferred from a tool's name --
/// only assigned after the adapter's qualification evidence supports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubDeterminismClass {
    Deterministic,
    DeterministicWithSeed,
    EnvironmentDependent,
    Unknown,
}

impl HubDeterminismClass {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HubDeterminismClass::Deterministic => "Deterministic",
            HubDeterminismClass::DeterministicWithSeed => "DeterministicWithSeed",
            HubDeterminismClass::EnvironmentDependent => "EnvironmentDependent",
            HubDeterminismClass::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for HubDeterminismClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How much the Hub and the caller should trust a tool's output.
/// `InProcessUnisolated` is the honest label for Hub v0's only execution
/// mode: it does not claim hard memory-corruption isolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubTrustClass {
    InProcessUnisolated,
    ProcessIsolated,
    SandboxIsolated,
}

impl HubTrustClass {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HubTrustClass::InProcessUnisolated => "InProcessUnisolated",
            HubTrustClass::ProcessIsolated => "ProcessIsolated",
            HubTrustClass::SandboxIsolated => "SandboxIsolated",
        }
    }
}

impl fmt::Display for HubTrustClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Privacy classification carried by a request/reply/audit record. Mirrors
/// the vocabulary defined by issue #1372 for Studio/ALM data governance so
/// Hub evidence can be reasoned about with the same classes, without Hub
/// reimplementing that policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubPrivacyClass {
    PublicSafe,
    ProjectLocal,
    PrivateSource,
    OrganizationPrivate,
    SecretSuspected,
}

impl HubPrivacyClass {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HubPrivacyClass::PublicSafe => "PublicSafe",
            HubPrivacyClass::ProjectLocal => "ProjectLocal",
            HubPrivacyClass::PrivateSource => "PrivateSource",
            HubPrivacyClass::OrganizationPrivate => "OrganizationPrivate",
            HubPrivacyClass::SecretSuspected => "SecretSuspected",
        }
    }

    /// Whether data at this class may appear in default (non-redacted) audit
    /// evidence. Hub v0 stores digests rather than raw payloads regardless,
    /// but this flag documents the intended propagation policy explicitly.
    pub const fn exportable_by_default(&self) -> bool {
        matches!(self, HubPrivacyClass::PublicSafe)
    }

    /// Parse a privacy class's stable name (as produced by [`Self::as_str`]).
    /// Used by callers building a request from external input (e.g. the CLI
    /// request file's `"privacy_class"` field).
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "PublicSafe" => HubPrivacyClass::PublicSafe,
            "ProjectLocal" => HubPrivacyClass::ProjectLocal,
            "PrivateSource" => HubPrivacyClass::PrivateSource,
            "OrganizationPrivate" => HubPrivacyClass::OrganizationPrivate,
            "SecretSuspected" => HubPrivacyClass::SecretSuspected,
            _ => return None,
        })
    }
}

impl fmt::Display for HubPrivacyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_mode_display_is_stable() {
        assert_eq!(HubExecutionMode::InProcess.to_string(), "InProcess");
    }

    #[test]
    fn determinism_class_display_is_stable() {
        assert_eq!(
            HubDeterminismClass::EnvironmentDependent.to_string(),
            "EnvironmentDependent"
        );
    }

    #[test]
    fn trust_class_does_not_claim_isolation_for_in_process() {
        assert_eq!(
            HubTrustClass::InProcessUnisolated.as_str(),
            "InProcessUnisolated"
        );
    }

    #[test]
    fn only_public_safe_is_exportable_by_default() {
        assert!(HubPrivacyClass::PublicSafe.exportable_by_default());
        assert!(!HubPrivacyClass::ProjectLocal.exportable_by_default());
        assert!(!HubPrivacyClass::SecretSuspected.exportable_by_default());
    }

    #[test]
    fn privacy_class_parse_round_trips_with_as_str() {
        const ALL: &[HubPrivacyClass] = &[
            HubPrivacyClass::PublicSafe,
            HubPrivacyClass::ProjectLocal,
            HubPrivacyClass::PrivateSource,
            HubPrivacyClass::OrganizationPrivate,
            HubPrivacyClass::SecretSuspected,
        ];
        for class in ALL {
            assert_eq!(HubPrivacyClass::parse(class.as_str()), Some(*class));
        }
        assert_eq!(HubPrivacyClass::parse("NotAClass"), None);
    }
}
