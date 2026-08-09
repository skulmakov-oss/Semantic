#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod hello_observation_capability;

use alloc::collections::BTreeSet;
use alloc::string::String;
use prom_abi::HostCallId;
use prom_refs::CapabilityRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CapabilityKind {
    GateRead,
    GateWrite,
    PulseEmit,
    ControlledObservationSink,
    StateQuery,
    StateUpdate,
    EventPost,
    ClockRead,
    ArgsRead,
    StdinReadText,
    StdoutWrite,
    StderrWrite,
    PathInspect,
    FsRead,
    FsWrite,
    TimeDuration,
}

impl CapabilityKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::GateRead => "gate.read",
            Self::GateWrite => "gate.write",
            Self::PulseEmit => "pulse.emit",
            Self::ControlledObservationSink => "observation.controlled",
            Self::StateQuery => "state.query",
            Self::StateUpdate => "state.update",
            Self::EventPost => "event.post",
            Self::ClockRead => "clock.read",
            Self::ArgsRead => "args.read",
            Self::StdinReadText => "stdin.read_text",
            Self::StdoutWrite => "stdout.write",
            Self::StderrWrite => "stderr.write",
            Self::PathInspect => "path.inspect",
            Self::FsRead => "fs.read",
            Self::FsWrite => "fs.write",
            Self::TimeDuration => "time.duration",
        }
    }
}

/// A non-authoritative lookup record for one exact capability reference.
///
/// Successful lookup of this entry does not grant authority and does not
/// perform admission, dispatch, or effect execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityLookupEntry {
    reference: CapabilityRef,
    kind: CapabilityKind,
}

impl CapabilityLookupEntry {
    /// Creates a non-authoritative capability lookup entry from an exact
    /// reference and its taxonomy-only capability kind.
    pub const fn new(reference: CapabilityRef, kind: CapabilityKind) -> Self {
        Self { reference, kind }
    }

    /// Returns the exact stored capability reference.
    pub const fn reference(&self) -> CapabilityRef {
        self.reference
    }

    /// Returns the taxonomy-only capability kind associated with the reference.
    pub const fn kind(&self) -> CapabilityKind {
        self.kind
    }
}

/// An immutable borrowed view over a validated, strictly sorted slice of
/// non-authoritative capability lookup entries.
///
/// Construction rejects duplicate and descending adjacent full-reference keys.
/// The operation itself is allocation-free and does not claim whole-crate
/// `no_std` qualification.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityLookupView<'a> {
    entries: &'a [CapabilityLookupEntry],
}

impl<'a> CapabilityLookupView<'a> {
    /// Borrows a validated, strictly sorted entry slice without copying,
    /// mutating, or allocating.
    pub fn try_new(
        entries: &'a [CapabilityLookupEntry],
    ) -> Result<Self, CapabilityLookupBuildError> {
        validate_lookup_entries(entries)?;
        Ok(Self { entries })
    }

    /// Performs exact full-reference matching and returns the borrowed stored
    /// entry on success.
    ///
    /// Successful lookup does not grant authority. `UnknownReference` is a
    /// lookup miss and is not `CapabilityDenied`.
    pub fn lookup(
        &self,
        reference: CapabilityRef,
    ) -> Result<&CapabilityLookupEntry, CapabilityLookupError> {
        self.entries
            .iter()
            .find(|entry| entry.reference() == reference)
            .ok_or(CapabilityLookupError::UnknownReference)
    }
}

/// Public miss result for exact capability-reference lookup.
///
/// This error reports only that the supplied full reference was not present in
/// the borrowed lookup view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityLookupError {
    UnknownReference,
}

/// Public construction error for borrowed capability lookup views.
///
/// Validation is deterministic, adjacent-pair based, and allocation-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityLookupBuildError {
    DuplicateReference,
    UnsortedEntries,
}

fn compare_capability_refs(left: CapabilityRef, right: CapabilityRef) -> core::cmp::Ordering {
    let left = left.token();
    let right = right.token();

    left.issuer()
        .cmp(&right.issuer())
        .then_with(|| left.namespace().cmp(&right.namespace()))
        .then_with(|| left.generation().cmp(&right.generation()))
        .then_with(|| left.value().cmp(&right.value()))
}

fn validate_lookup_entries(
    entries: &[CapabilityLookupEntry],
) -> Result<(), CapabilityLookupBuildError> {
    for pair in entries.windows(2) {
        match compare_capability_refs(pair[0].reference(), pair[1].reference()) {
            core::cmp::Ordering::Less => {}
            core::cmp::Ordering::Equal => {
                return Err(CapabilityLookupBuildError::DuplicateReference);
            }
            core::cmp::Ordering::Greater => {
                return Err(CapabilityLookupBuildError::UnsortedEntries);
            }
        }
    }

    Ok(())
}

pub const fn required_capability_for_call(call: HostCallId) -> CapabilityKind {
    match call {
        HostCallId::GateRead => CapabilityKind::GateRead,
        HostCallId::GateWrite => CapabilityKind::GateWrite,
        HostCallId::PulseEmit => CapabilityKind::PulseEmit,
        HostCallId::StateQuery => CapabilityKind::StateQuery,
        HostCallId::StateUpdate => CapabilityKind::StateUpdate,
        HostCallId::EventPost => CapabilityKind::EventPost,
        HostCallId::ClockRead => CapabilityKind::ClockRead,
        HostCallId::ArgsRead => CapabilityKind::ArgsRead,
        HostCallId::StdinReadText => CapabilityKind::StdinReadText,
        HostCallId::StdoutWrite => CapabilityKind::StdoutWrite,
        HostCallId::StderrWrite => CapabilityKind::StderrWrite,
        HostCallId::PathInspect => CapabilityKind::PathInspect,
        HostCallId::FsRead => CapabilityKind::FsRead,
        HostCallId::FsWrite => CapabilityKind::FsWrite,
        HostCallId::TimeDuration => CapabilityKind::TimeDuration,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySurfaceClass {
    StableV1,
    FoundationApplicationV0,
    PlannedPostStable,
}

pub const fn capability_surface_class(kind: CapabilityKind) -> CapabilitySurfaceClass {
    match kind {
        CapabilityKind::GateRead => CapabilitySurfaceClass::StableV1,
        CapabilityKind::GateWrite => CapabilitySurfaceClass::StableV1,
        CapabilityKind::PulseEmit => CapabilitySurfaceClass::StableV1,
        CapabilityKind::ControlledObservationSink => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::StateQuery => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::StateUpdate => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::EventPost => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::ClockRead => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::ArgsRead
        | CapabilityKind::StdinReadText
        | CapabilityKind::StdoutWrite
        | CapabilityKind::StderrWrite
        | CapabilityKind::PathInspect
        | CapabilityKind::FsRead
        | CapabilityKind::FsWrite
        | CapabilityKind::TimeDuration => CapabilitySurfaceClass::FoundationApplicationV0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationCapabilityProfile {
    Pure,
    CliReadOnly,
    CliFileTransform,
    UiBounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityManifestVersion {
    V1,
}

impl CapabilityManifestVersion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => "v1",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityManifestMetadata {
    pub schema: String,
    pub version: CapabilityManifestVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityDeniedCode {
    MissingCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDenied {
    pub capability: CapabilityKind,
    pub call: Option<HostCallId>,
    pub code: CapabilityDeniedCode,
    pub manifest: CapabilityManifestMetadata,
    pub message: String,
}

impl CapabilityDenied {
    pub fn new(
        capability: CapabilityKind,
        call: Option<HostCallId>,
        code: CapabilityDeniedCode,
        manifest: CapabilityManifestMetadata,
        message: impl Into<String>,
    ) -> Self {
        Self {
            capability,
            call,
            code,
            manifest,
            message: message.into(),
        }
    }
}

impl core::fmt::Display for CapabilityDenied {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.call {
            Some(call) => write!(
                f,
                "capability {:?} denied for {:?} [{} {} {:?}]: {}",
                self.capability,
                call,
                self.manifest.schema,
                self.manifest.version.as_str(),
                self.code,
                self.message
            ),
            None => write!(
                f,
                "capability {:?} denied [{} {} {:?}]: {}",
                self.capability,
                self.manifest.schema,
                self.manifest.version.as_str(),
                self.code,
                self.message
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CapabilityDenied {}

pub trait CapabilityChecker {
    fn require(&self, capability: CapabilityKind) -> Result<(), CapabilityDenied>;

    fn require_call(&self, call: HostCallId) -> Result<(), CapabilityDenied> {
        self.require(required_capability_for_call(call))
            .map_err(|mut denied| {
                denied.call = Some(call);
                denied
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestValidationCode {
    UnsupportedSchema,
    UnsupportedVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestValidationReport {
    pub code: ManifestValidationCode,
    pub message: String,
}

impl ManifestValidationReport {
    pub fn new(code: ManifestValidationCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityManifest {
    schema: String,
    version: CapabilityManifestVersion,
    allowed: BTreeSet<CapabilityKind>,
}

impl CapabilityManifest {
    pub const CURRENT_SCHEMA: &'static str = "prom.cap.manifest";
    pub const CURRENT_VERSION: CapabilityManifestVersion = CapabilityManifestVersion::V1;

    pub fn new() -> Self {
        Self {
            schema: Self::CURRENT_SCHEMA.into(),
            version: Self::CURRENT_VERSION,
            allowed: BTreeSet::new(),
        }
    }

    pub fn with_contract(schema: impl Into<String>, version: CapabilityManifestVersion) -> Self {
        Self {
            schema: schema.into(),
            version,
            allowed: BTreeSet::new(),
        }
    }

    pub fn allow(&mut self, capability: CapabilityKind) {
        self.allowed.insert(capability);
    }

    pub fn allows(&self, capability: CapabilityKind) -> bool {
        self.allowed.contains(&capability)
    }

    pub fn metadata(&self) -> CapabilityManifestMetadata {
        CapabilityManifestMetadata {
            schema: self.schema.clone(),
            version: self.version,
        }
    }

    pub fn validate(&self) -> Result<(), ManifestValidationReport> {
        if self.schema != Self::CURRENT_SCHEMA {
            return Err(ManifestValidationReport::new(
                ManifestValidationCode::UnsupportedSchema,
                format!(
                    "unsupported capability manifest schema '{}'; expected '{}'",
                    self.schema,
                    Self::CURRENT_SCHEMA
                ),
            ));
        }
        if self.version != Self::CURRENT_VERSION {
            return Err(ManifestValidationReport::new(
                ManifestValidationCode::UnsupportedVersion,
                format!(
                    "unsupported capability manifest version '{}'; expected '{}'",
                    self.version.as_str(),
                    Self::CURRENT_VERSION.as_str()
                ),
            ));
        }
        Ok(())
    }

    pub fn gate_surface() -> Self {
        let mut manifest = Self::new();
        manifest.allow(CapabilityKind::GateRead);
        manifest.allow(CapabilityKind::GateWrite);
        manifest.allow(CapabilityKind::PulseEmit);
        manifest
    }

    pub fn for_application_profile(profile: ApplicationCapabilityProfile) -> Self {
        let mut manifest = Self::new();
        match profile {
            ApplicationCapabilityProfile::Pure => {}
            ApplicationCapabilityProfile::CliReadOnly
            | ApplicationCapabilityProfile::CliFileTransform => {
                for capability in [
                    CapabilityKind::ArgsRead,
                    CapabilityKind::StdinReadText,
                    CapabilityKind::StdoutWrite,
                    CapabilityKind::StderrWrite,
                    CapabilityKind::PathInspect,
                    CapabilityKind::FsRead,
                    CapabilityKind::TimeDuration,
                ] {
                    manifest.allow(capability);
                }
                if profile == ApplicationCapabilityProfile::CliFileTransform {
                    manifest.allow(CapabilityKind::FsWrite);
                }
            }
            ApplicationCapabilityProfile::UiBounded => {
                manifest.allow(CapabilityKind::ControlledObservationSink);
                manifest.allow(CapabilityKind::TimeDuration);
            }
        }
        manifest
    }
}

impl Default for CapabilityManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityChecker for CapabilityManifest {
    fn require(&self, capability: CapabilityKind) -> Result<(), CapabilityDenied> {
        self.validate().map_err(|report| {
            CapabilityDenied::new(
                capability,
                None,
                CapabilityDeniedCode::MissingCapability,
                self.metadata(),
                report.message,
            )
        })?;
        if self.allows(capability) {
            Ok(())
        } else {
            Err(CapabilityDenied::new(
                capability,
                None,
                CapabilityDeniedCode::MissingCapability,
                self.metadata(),
                "manifest does not grant this capability",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_maps_host_calls_to_capabilities() {
        assert_eq!(
            required_capability_for_call(HostCallId::GateRead),
            CapabilityKind::GateRead
        );
        assert_eq!(
            required_capability_for_call(HostCallId::PulseEmit),
            CapabilityKind::PulseEmit
        );
        assert_eq!(
            required_capability_for_call(HostCallId::StateUpdate),
            CapabilityKind::StateUpdate
        );
    }

    #[test]
    fn capability_surface_class_keeps_planned_calls_outside_v1() {
        assert_eq!(
            capability_surface_class(CapabilityKind::GateRead),
            CapabilitySurfaceClass::StableV1
        );
        assert_eq!(
            capability_surface_class(CapabilityKind::ControlledObservationSink),
            CapabilitySurfaceClass::PlannedPostStable
        );
        assert_eq!(
            capability_surface_class(CapabilityKind::ClockRead),
            CapabilitySurfaceClass::PlannedPostStable
        );
    }

    #[test]
    fn gate_surface_remains_narrow_v1_only() {
        let manifest = CapabilityManifest::gate_surface();
        assert!(manifest.allows(CapabilityKind::GateRead));
        assert!(!manifest.allows(CapabilityKind::StateQuery));
        assert!(!manifest.allows(CapabilityKind::ControlledObservationSink));
        assert!(!manifest.allows(CapabilityKind::ClockRead));
    }

    #[test]
    fn manifest_can_allow_controlled_observation_sink_only_when_explicitly_granted() {
        let mut manifest = CapabilityManifest::new();
        assert!(!manifest.allows(CapabilityKind::ControlledObservationSink));
        manifest.allow(CapabilityKind::ControlledObservationSink);
        assert!(manifest.allows(CapabilityKind::ControlledObservationSink));
        manifest
            .require(CapabilityKind::ControlledObservationSink)
            .expect("explicitly granted capability must admit");
    }

    #[test]
    fn manifest_denies_missing_capability() {
        let manifest = CapabilityManifest::new();
        let denied = manifest
            .require(CapabilityKind::GateWrite)
            .expect_err("must deny");
        assert_eq!(denied.capability, CapabilityKind::GateWrite);
        assert_eq!(denied.code, CapabilityDeniedCode::MissingCapability);
        assert_eq!(denied.manifest.schema, CapabilityManifest::CURRENT_SCHEMA);
    }

    #[test]
    fn manifest_exposes_current_contract_metadata() {
        let manifest = CapabilityManifest::gate_surface();
        let metadata = manifest.metadata();
        assert_eq!(metadata.schema, CapabilityManifest::CURRENT_SCHEMA);
        assert_eq!(metadata.version, CapabilityManifestVersion::V1);
    }

    #[test]
    fn manifest_validate_rejects_unknown_schema() {
        let manifest =
            CapabilityManifest::with_contract("prom.cap.legacy", CapabilityManifestVersion::V1);
        let report = manifest
            .validate()
            .expect_err("schema mismatch must reject");
        assert_eq!(report.code, ManifestValidationCode::UnsupportedSchema);
    }

    #[test]
    fn require_call_attaches_host_call_context() {
        let manifest = CapabilityManifest::new();
        let denied = manifest
            .require_call(HostCallId::PulseEmit)
            .expect_err("must deny");
        assert_eq!(denied.call, Some(HostCallId::PulseEmit));
        assert_eq!(denied.capability, CapabilityKind::PulseEmit);
    }

    #[test]
    fn application_capability_ids_are_exact() {
        assert_eq!(CapabilityKind::ArgsRead.id(), "args.read");
        assert_eq!(CapabilityKind::StdinReadText.id(), "stdin.read_text");
        assert_eq!(CapabilityKind::StdoutWrite.id(), "stdout.write");
        assert_eq!(CapabilityKind::StderrWrite.id(), "stderr.write");
        assert_eq!(CapabilityKind::PathInspect.id(), "path.inspect");
        assert_eq!(CapabilityKind::FsRead.id(), "fs.read");
        assert_eq!(CapabilityKind::FsWrite.id(), "fs.write");
        assert_eq!(CapabilityKind::TimeDuration.id(), "time.duration");
    }

    #[test]
    fn application_profiles_are_deny_by_default_and_write_is_narrow() {
        let pure = CapabilityManifest::for_application_profile(ApplicationCapabilityProfile::Pure);
        assert!(!pure.allows(CapabilityKind::ArgsRead));

        let read_only =
            CapabilityManifest::for_application_profile(ApplicationCapabilityProfile::CliReadOnly);
        assert!(read_only.allows(CapabilityKind::FsRead));
        assert!(!read_only.allows(CapabilityKind::FsWrite));

        let transform = CapabilityManifest::for_application_profile(
            ApplicationCapabilityProfile::CliFileTransform,
        );
        assert!(transform.allows(CapabilityKind::FsRead));
        assert!(transform.allows(CapabilityKind::FsWrite));
    }
}
