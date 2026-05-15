#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod hello_observation_capability;

use alloc::collections::BTreeSet;
use alloc::string::String;
use prom_abi::HostCallId;
use prom_ui::{UiCapabilityKind, UiOperationId, required_ui_capability};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CapabilityKind {
    GateRead,
    GateWrite,
    PulseEmit,
    StateQuery,
    StateUpdate,
    EventPost,
    ClockRead,
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
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySurfaceClass {
    StableV1,
    PlannedPostStable,
}

pub const fn capability_surface_class(kind: CapabilityKind) -> CapabilitySurfaceClass {
    match kind {
        CapabilityKind::GateRead => CapabilitySurfaceClass::StableV1,
        CapabilityKind::GateWrite => CapabilitySurfaceClass::StableV1,
        CapabilityKind::PulseEmit => CapabilitySurfaceClass::StableV1,
        CapabilityKind::StateQuery => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::StateUpdate => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::EventPost => CapabilitySurfaceClass::PlannedPostStable,
        CapabilityKind::ClockRead => CapabilitySurfaceClass::PlannedPostStable,
    }
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

/// Denial result for a UI capability check.
///
/// Mirrors [`CapabilityDenied`] but carries UI-specific types from `prom-ui`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCapabilityDenied {
    pub capability: UiCapabilityKind,
    pub operation: Option<UiOperationId>,
    pub code: CapabilityDeniedCode,
    pub manifest: CapabilityManifestMetadata,
    pub message: String,
}

impl UiCapabilityDenied {
    pub fn new(
        capability: UiCapabilityKind,
        operation: Option<UiOperationId>,
        manifest: CapabilityManifestMetadata,
        message: impl Into<String>,
    ) -> Self {
        Self {
            capability,
            operation,
            code: CapabilityDeniedCode::MissingCapability,
            manifest,
            message: message.into(),
        }
    }
}

impl core::fmt::Display for UiCapabilityDenied {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.operation {
            Some(op) => write!(
                f,
                "UI capability {:?} denied for {:?} [{} {} {:?}]: {}",
                self.capability,
                op,
                self.manifest.schema,
                self.manifest.version.as_str(),
                self.code,
                self.message
            ),
            None => write!(
                f,
                "UI capability {:?} denied [{} {} {:?}]: {}",
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
impl std::error::Error for UiCapabilityDenied {}

/// Checking contract for the UI application boundary capability surface.
///
/// Wired at Wave 1. This trait allows the VM host to deny UI operations
/// when the required `UiCapabilityKind` is not admitted in the manifest.
pub trait UiCapabilityChecker {
    fn require_ui(&self, capability: UiCapabilityKind) -> Result<(), UiCapabilityDenied>;

    fn require_ui_op(&self, op: UiOperationId) -> Result<(), UiCapabilityDenied> {
        self.require_ui(required_ui_capability(op)).map_err(|mut denied| {
            denied.operation = Some(op);
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
    /// Admitted UI capabilities for the application boundary surface.
    ///
    /// Empty by default. Populated by `allow_ui()` at Wave 1.
    allowed_ui: BTreeSet<UiCapabilityKind>,
}

impl CapabilityManifest {
    pub const CURRENT_SCHEMA: &'static str = "prom.cap.manifest";
    pub const CURRENT_VERSION: CapabilityManifestVersion = CapabilityManifestVersion::V1;

    pub fn new() -> Self {
        Self {
            schema: Self::CURRENT_SCHEMA.into(),
            version: Self::CURRENT_VERSION,
            allowed: BTreeSet::new(),
            allowed_ui: BTreeSet::new(),
        }
    }

    pub fn with_contract(schema: impl Into<String>, version: CapabilityManifestVersion) -> Self {
        Self {
            schema: schema.into(),
            version,
            allowed: BTreeSet::new(),
            allowed_ui: BTreeSet::new(),
        }
    }

    pub fn allow(&mut self, capability: CapabilityKind) {
        self.allowed.insert(capability);
    }

    pub fn allows(&self, capability: CapabilityKind) -> bool {
        self.allowed.contains(&capability)
    }

    /// Admit a UI capability into this manifest.
    ///
    /// UI capabilities are post-stable (Wave 1+) and must be explicitly
    /// granted; they are never included in the default or gate surfaces.
    pub fn allow_ui(&mut self, capability: UiCapabilityKind) {
        self.allowed_ui.insert(capability);
    }

    pub fn allows_ui(&self, capability: UiCapabilityKind) -> bool {
        self.allowed_ui.contains(&capability)
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

impl UiCapabilityChecker for CapabilityManifest {
    fn require_ui(&self, capability: UiCapabilityKind) -> Result<(), UiCapabilityDenied> {
        // Manifest schema/version validation applies to all capability checks.
        self.validate().map_err(|report| {
            UiCapabilityDenied::new(capability, None, self.metadata(), report.message)
        })?;
        if self.allows_ui(capability) {
            Ok(())
        } else {
            Err(UiCapabilityDenied::new(
                capability,
                None,
                self.metadata(),
                "manifest does not grant this UI capability",
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
            capability_surface_class(CapabilityKind::ClockRead),
            CapabilitySurfaceClass::PlannedPostStable
        );
    }

    #[test]
    fn gate_surface_remains_narrow_v1_only() {
        let manifest = CapabilityManifest::gate_surface();
        assert!(manifest.allows(CapabilityKind::GateRead));
        assert!(!manifest.allows(CapabilityKind::StateQuery));
        assert!(!manifest.allows(CapabilityKind::ClockRead));
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

    // --- M7 Wave 1: UI capability checker tests ---

    #[test]
    fn ui_manifest_denies_missing_ui_capability() {
        let manifest = CapabilityManifest::new();
        let denied = manifest
            .require_ui(UiCapabilityKind::DesktopSession)
            .expect_err("must deny UI capability when not granted");
        assert_eq!(denied.capability, UiCapabilityKind::DesktopSession);
        assert_eq!(denied.code, CapabilityDeniedCode::MissingCapability);
        assert!(denied.operation.is_none());
    }

    #[test]
    fn ui_manifest_admits_explicitly_granted_capability() {
        let mut manifest = CapabilityManifest::new();
        manifest.allow_ui(UiCapabilityKind::DesktopSession);
        manifest.allow_ui(UiCapabilityKind::InputPoll);
        assert!(manifest.allows_ui(UiCapabilityKind::DesktopSession));
        assert!(manifest.allows_ui(UiCapabilityKind::InputPoll));
        assert!(!manifest.allows_ui(UiCapabilityKind::FrameEmit));
        manifest.require_ui(UiCapabilityKind::DesktopSession).expect("must admit");
    }

    #[test]
    fn require_ui_op_attaches_operation_context() {
        let manifest = CapabilityManifest::new();
        let denied = manifest
            .require_ui_op(UiOperationId::WindowCreate)
            .expect_err("must deny");
        assert_eq!(denied.capability, UiCapabilityKind::DesktopSession);
        assert_eq!(denied.operation, Some(UiOperationId::WindowCreate));
    }

    #[test]
    fn gate_surface_never_includes_ui_capabilities() {
        let manifest = CapabilityManifest::gate_surface();
        assert!(!manifest.allows_ui(UiCapabilityKind::DesktopSession));
        assert!(!manifest.allows_ui(UiCapabilityKind::InputPoll));
        assert!(!manifest.allows_ui(UiCapabilityKind::FrameEmit));
    }

    #[test]
    fn ui_capability_denied_display_includes_capability_and_operation() {
        let manifest = CapabilityManifest::new();
        let denied = manifest
            .require_ui_op(UiOperationId::FrameSubmit)
            .expect_err("must deny");
        let msg = format!("{}", denied);
        assert!(msg.contains("FrameEmit"));
        assert!(msg.contains("FrameSubmit"));
    }
}
