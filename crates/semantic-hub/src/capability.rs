//! Hub-domain capability model.
//!
//! These capabilities are distinct from `prom-cap::CapabilityKind`, which
//! governs SemCode host-ABI effects (gate reads, pulses, state writes). Hub
//! capabilities govern a different domain -- what an external tool operation
//! may do -- the same way `prom-gates::GateId` and `prom-rules::RuleId` are
//! each domain-specific identifiers that sit alongside `prom-cap` rather than
//! extend it. Mirrors `prom_cap::CapabilityManifest`'s shape (schema-versioned
//! `BTreeSet`) without sharing its enum, since the two domains do not overlap.

use std::collections::BTreeSet;
use std::fmt;

/// One capability an admitted Hub invocation may exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubCapability {
    // -- bounded computational capabilities (allowed by default for in-scope tools) --
    VectorIndexCreate,
    VectorIndexRead,
    VectorIndexMutate,
    VectorSearch,
    VectorFilteredSearch,
    VectorIndexPersist,
    CpuCompute,
    MemoryAllocateBounded,
    PrivateStorageRead,
    PrivateStorageWrite,
    ClockMonotonic,

    // -- sensitive capabilities: denied by default, not grantable to any v0 tool --
    NetworkAccess,
    ArbitraryFilesystemRead,
    ArbitraryFilesystemWrite,
    ProcessSpawn,
    DeviceAccess,
    EnvironmentRead,
    SecretRead,
    ProjectMutation,
    SemanticStateMutation,
}

impl HubCapability {
    /// Sensitive capabilities are denied by default and are not granted to
    /// any Hub v0 tool, regardless of what a request asks for.
    pub const fn is_sensitive(&self) -> bool {
        matches!(
            self,
            HubCapability::NetworkAccess
                | HubCapability::ArbitraryFilesystemRead
                | HubCapability::ArbitraryFilesystemWrite
                | HubCapability::ProcessSpawn
                | HubCapability::DeviceAccess
                | HubCapability::EnvironmentRead
                | HubCapability::SecretRead
                | HubCapability::ProjectMutation
                | HubCapability::SemanticStateMutation
        )
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            HubCapability::VectorIndexCreate => "VectorIndexCreate",
            HubCapability::VectorIndexRead => "VectorIndexRead",
            HubCapability::VectorIndexMutate => "VectorIndexMutate",
            HubCapability::VectorSearch => "VectorSearch",
            HubCapability::VectorFilteredSearch => "VectorFilteredSearch",
            HubCapability::VectorIndexPersist => "VectorIndexPersist",
            HubCapability::CpuCompute => "CpuCompute",
            HubCapability::MemoryAllocateBounded => "MemoryAllocateBounded",
            HubCapability::PrivateStorageRead => "PrivateStorageRead",
            HubCapability::PrivateStorageWrite => "PrivateStorageWrite",
            HubCapability::ClockMonotonic => "ClockMonotonic",
            HubCapability::NetworkAccess => "NetworkAccess",
            HubCapability::ArbitraryFilesystemRead => "ArbitraryFilesystemRead",
            HubCapability::ArbitraryFilesystemWrite => "ArbitraryFilesystemWrite",
            HubCapability::ProcessSpawn => "ProcessSpawn",
            HubCapability::DeviceAccess => "DeviceAccess",
            HubCapability::EnvironmentRead => "EnvironmentRead",
            HubCapability::SecretRead => "SecretRead",
            HubCapability::ProjectMutation => "ProjectMutation",
            HubCapability::SemanticStateMutation => "SemanticStateMutation",
        }
    }

    /// Parse a capability's stable name (as produced by [`Self::as_str`]).
    /// Used by callers that build a `HubCapabilitySet` from external input
    /// (e.g. the CLI request file's `"capabilities"` string list). Returns
    /// `None` for any unrecognized name rather than guessing.
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "VectorIndexCreate" => HubCapability::VectorIndexCreate,
            "VectorIndexRead" => HubCapability::VectorIndexRead,
            "VectorIndexMutate" => HubCapability::VectorIndexMutate,
            "VectorSearch" => HubCapability::VectorSearch,
            "VectorFilteredSearch" => HubCapability::VectorFilteredSearch,
            "VectorIndexPersist" => HubCapability::VectorIndexPersist,
            "CpuCompute" => HubCapability::CpuCompute,
            "MemoryAllocateBounded" => HubCapability::MemoryAllocateBounded,
            "PrivateStorageRead" => HubCapability::PrivateStorageRead,
            "PrivateStorageWrite" => HubCapability::PrivateStorageWrite,
            "ClockMonotonic" => HubCapability::ClockMonotonic,
            "NetworkAccess" => HubCapability::NetworkAccess,
            "ArbitraryFilesystemRead" => HubCapability::ArbitraryFilesystemRead,
            "ArbitraryFilesystemWrite" => HubCapability::ArbitraryFilesystemWrite,
            "ProcessSpawn" => HubCapability::ProcessSpawn,
            "DeviceAccess" => HubCapability::DeviceAccess,
            "EnvironmentRead" => HubCapability::EnvironmentRead,
            "SecretRead" => HubCapability::SecretRead,
            "ProjectMutation" => HubCapability::ProjectMutation,
            "SemanticStateMutation" => HubCapability::SemanticStateMutation,
            _ => return None,
        })
    }
}

impl fmt::Display for HubCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Schema version of the `HubCapabilitySet` wire/audit shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HubCapabilitySetVersion {
    V1,
}

/// An explicit, closed set of capabilities granted to (or required by) one
/// Hub invocation. Deny-by-default: absence means denied.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HubCapabilitySet {
    granted: BTreeSet<HubCapability>,
}

impl HubCapabilitySet {
    pub const CURRENT_SCHEMA: HubCapabilitySetVersion = HubCapabilitySetVersion::V1;

    pub fn empty() -> Self {
        Self {
            granted: BTreeSet::new(),
        }
    }

    /// Grant a capability. Sensitive capabilities can be inserted into a
    /// caller-declared set (so denial is auditable and explicit) but
    /// `HubCapabilitySet::allows` for dispatch purposes is checked against
    /// the tool's fixed sensitive-deny policy separately by the admission
    /// path -- this type itself does not silently drop sensitive entries.
    pub fn grant(mut self, capability: HubCapability) -> Self {
        self.granted.insert(capability);
        self
    }

    pub fn allows(&self, capability: HubCapability) -> bool {
        self.granted.contains(&capability)
    }

    /// True only if every capability in `required` is present in `self` and
    /// none of `required` is sensitive (sensitive capabilities are never
    /// satisfiable in Hub v0, no matter what is granted).
    pub fn satisfies(&self, required: &[HubCapability]) -> bool {
        required
            .iter()
            .all(|c| !c.is_sensitive() && self.allows(*c))
    }

    pub fn iter(&self) -> impl Iterator<Item = &HubCapability> {
        self.granted.iter()
    }

    pub fn len(&self) -> usize {
        self.granted.len()
    }

    pub fn is_empty(&self) -> bool {
        self.granted.is_empty()
    }
}

impl FromIterator<HubCapability> for HubCapabilitySet {
    fn from_iter<T: IntoIterator<Item = HubCapability>>(iter: T) -> Self {
        Self {
            granted: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_set_allows_nothing() {
        let set = HubCapabilitySet::empty();
        assert!(!set.allows(HubCapability::VectorSearch));
        assert!(set.is_empty());
    }

    #[test]
    fn grant_then_allow_round_trips() {
        let set = HubCapabilitySet::empty().grant(HubCapability::VectorSearch);
        assert!(set.allows(HubCapability::VectorSearch));
        assert!(!set.allows(HubCapability::VectorIndexMutate));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn satisfies_requires_all_listed_capabilities() {
        let set = HubCapabilitySet::empty()
            .grant(HubCapability::VectorSearch)
            .grant(HubCapability::VectorFilteredSearch);
        assert!(set.satisfies(&[HubCapability::VectorSearch]));
        assert!(set.satisfies(&[
            HubCapability::VectorSearch,
            HubCapability::VectorFilteredSearch
        ]));
        assert!(!set.satisfies(&[HubCapability::VectorIndexMutate]));
    }

    #[test]
    fn satisfies_never_grants_sensitive_capabilities_even_if_present() {
        let set = HubCapabilitySet::empty().grant(HubCapability::NetworkAccess);
        assert!(set.allows(HubCapability::NetworkAccess));
        assert!(!set.satisfies(&[HubCapability::NetworkAccess]));
    }

    #[test]
    fn parse_round_trips_with_as_str_for_every_variant() {
        const ALL: &[HubCapability] = &[
            HubCapability::VectorIndexCreate,
            HubCapability::VectorIndexRead,
            HubCapability::VectorIndexMutate,
            HubCapability::VectorSearch,
            HubCapability::VectorFilteredSearch,
            HubCapability::VectorIndexPersist,
            HubCapability::CpuCompute,
            HubCapability::MemoryAllocateBounded,
            HubCapability::PrivateStorageRead,
            HubCapability::PrivateStorageWrite,
            HubCapability::ClockMonotonic,
            HubCapability::NetworkAccess,
            HubCapability::ArbitraryFilesystemRead,
            HubCapability::ArbitraryFilesystemWrite,
            HubCapability::ProcessSpawn,
            HubCapability::DeviceAccess,
            HubCapability::EnvironmentRead,
            HubCapability::SecretRead,
            HubCapability::ProjectMutation,
            HubCapability::SemanticStateMutation,
        ];
        for cap in ALL {
            assert_eq!(HubCapability::parse(cap.as_str()), Some(*cap));
        }
    }

    #[test]
    fn parse_rejects_unknown_name() {
        assert_eq!(HubCapability::parse("NotARealCapability"), None);
    }

    #[test]
    fn sensitive_classification_matches_the_documented_deny_by_default_list() {
        assert!(HubCapability::NetworkAccess.is_sensitive());
        assert!(HubCapability::ProcessSpawn.is_sensitive());
        assert!(HubCapability::SemanticStateMutation.is_sensitive());
        assert!(!HubCapability::VectorSearch.is_sensitive());
        assert!(!HubCapability::CpuCompute.is_sensitive());
    }

    #[test]
    fn display_matches_stable_names_used_in_audit_evidence() {
        assert_eq!(
            alloc_free_format(HubCapability::VectorSearch),
            "VectorSearch"
        );
    }

    fn alloc_free_format(c: HubCapability) -> String {
        format!("{c}")
    }
}
