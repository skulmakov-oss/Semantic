use prom_cap::{CapabilityDeniedCode, CapabilityManifest, UiCapabilityChecker};
use prom_ui::{
    required_ui_capability, ui_capability_surface_class, UiCapabilityKind,
    UiCapabilitySurfaceClass, UiOperationId,
};

fn all_ui_ops() -> [UiOperationId; 5] {
    [
        UiOperationId::WindowCreate,
        UiOperationId::WindowRun,
        UiOperationId::WindowClose,
        UiOperationId::EventPoll,
        UiOperationId::FrameSubmit,
    ]
}

fn all_ui_caps() -> [UiCapabilityKind; 3] {
    [
        UiCapabilityKind::DesktopSession,
        UiCapabilityKind::InputPoll,
        UiCapabilityKind::FrameEmit,
    ]
}

#[test]
fn ui_operation_capability_mapping_is_canonical() {
    assert_eq!(
        required_ui_capability(UiOperationId::WindowCreate),
        UiCapabilityKind::DesktopSession
    );
    assert_eq!(
        required_ui_capability(UiOperationId::WindowRun),
        UiCapabilityKind::DesktopSession
    );
    assert_eq!(
        required_ui_capability(UiOperationId::WindowClose),
        UiCapabilityKind::DesktopSession
    );
    assert_eq!(
        required_ui_capability(UiOperationId::EventPoll),
        UiCapabilityKind::InputPoll
    );
    assert_eq!(
        required_ui_capability(UiOperationId::FrameSubmit),
        UiCapabilityKind::FrameEmit
    );
}

#[test]
fn default_manifest_denies_all_ui_operations() {
    let manifest = CapabilityManifest::new();

    for op in all_ui_ops() {
        let denied = manifest
            .require_ui_op(op)
            .expect_err("default manifest must deny UI operations");
        assert_eq!(denied.capability, required_ui_capability(op));
        assert_eq!(denied.operation, Some(op));
        assert_eq!(denied.code, CapabilityDeniedCode::MissingCapability);
        assert_eq!(denied.manifest.schema, CapabilityManifest::CURRENT_SCHEMA);
    }
}

#[test]
fn gate_surface_does_not_grant_ui_capabilities() {
    let manifest = CapabilityManifest::gate_surface();

    for capability in all_ui_caps() {
        assert!(
            !manifest.allows_ui(capability),
            "gate_surface must not grant {:?}",
            capability
        );
    }

    for op in all_ui_ops() {
        let denied = manifest
            .require_ui_op(op)
            .expect_err("gate surface must still deny UI operations");
        assert_eq!(denied.capability, required_ui_capability(op));
        assert_eq!(denied.operation, Some(op));
    }
}

#[test]
fn explicit_ui_capability_admits_matching_operations() {
    let mut manifest = CapabilityManifest::new();
    manifest.allow_ui(UiCapabilityKind::DesktopSession);
    manifest.allow_ui(UiCapabilityKind::InputPoll);
    manifest.allow_ui(UiCapabilityKind::FrameEmit);

    for capability in all_ui_caps() {
        assert!(manifest.allows_ui(capability));
    }

    for op in all_ui_ops() {
        manifest
            .require_ui_op(op)
            .expect("explicit UI capability must admit matching operation");
    }
}

#[test]
fn ui_denial_carries_operation_context() {
    let manifest = CapabilityManifest::new();
    let denied = manifest
        .require_ui_op(UiOperationId::FrameSubmit)
        .expect_err("missing UI capability must fail closed");

    assert_eq!(denied.operation, Some(UiOperationId::FrameSubmit));
    assert_eq!(denied.capability, UiCapabilityKind::FrameEmit);
    assert_eq!(denied.code, CapabilityDeniedCode::MissingCapability);
    assert!(denied
        .message
        .contains("manifest does not grant this UI capability"));
}

#[test]
fn ui_surface_remains_post_stable() {
    for capability in all_ui_caps() {
        assert_eq!(
            ui_capability_surface_class(capability),
            UiCapabilitySurfaceClass::PostStableFirstWave
        );
    }
}
