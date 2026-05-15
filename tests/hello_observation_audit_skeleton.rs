use prom_audit::hello_observation_audit::{
    build_hello_observation_audit_event, HelloObservationAuditEventKind,
    HelloObservationAuditLinkage, HelloObservationAuditPayloadRef,
    HelloObservationAuditPolicyClass,
};
use prom_audit::{AuditEventId, AuditEventKind, AuditSessionMetadata, AuditTrail};
use prom_cap::{CapabilityManifestMetadata, CapabilityManifestVersion};
use sm_runtime_core::ExecutionContext;

fn sample_linkage() -> HelloObservationAuditLinkage {
    HelloObservationAuditLinkage {
        verifier_admission_ref: Some(17),
        capability_policy_ref: Some(29),
        sink_policy_ref: Some(31),
    }
}

#[test]
fn hello_observation_audit_skeleton_builds_controlled_observation_event() {
    let event = build_hello_observation_audit_event(
        0xfeed_beef,
        7,
        HelloObservationAuditPolicyClass::Required,
        sample_linkage(),
    );

    assert_eq!(event.event_kind, HelloObservationAuditEventKind::Observation);
    assert_eq!(event.operation_kind, "controlled_observation_text");
    assert_eq!(event.observation_class, "controlled");
    assert_eq!(
        event.payload_ref,
        HelloObservationAuditPayloadRef::LiteralTextHash(0xfeed_beef)
    );
    assert_eq!(event.sequence_index.0, 7);
    assert_eq!(event.audit_policy_class, HelloObservationAuditPolicyClass::Required);
    assert_eq!(event.linkage, sample_linkage());
}

#[test]
fn hello_observation_audit_skeleton_has_no_wall_clock_timestamp_field() {
    let event = build_hello_observation_audit_event(
        42,
        1,
        HelloObservationAuditPolicyClass::Deferred,
        sample_linkage(),
    );

    assert_eq!(event.sequence_index.0, 1);
    assert!(matches!(
        event.payload_ref,
        HelloObservationAuditPayloadRef::LiteralTextHash(42)
    ));
}

#[test]
fn hello_observation_audit_skeleton_rejects_stdout_style_kinds_by_construction() {
    let event = build_hello_observation_audit_event(
        99,
        3,
        HelloObservationAuditPolicyClass::Required,
        sample_linkage(),
    );

    assert_eq!(event.operation_kind, "controlled_observation_text");
    assert_ne!(event.operation_kind, "stdout");
    assert_ne!(event.operation_kind, "print");
    assert_ne!(event.operation_kind, "io.write");
}

#[test]
fn hello_observation_audit_skeleton_keeps_existing_audit_trail_unchanged() {
    let session = AuditSessionMetadata {
        context: ExecutionContext::KernelBound,
        capability_manifest: CapabilityManifestMetadata {
            schema: "prom.cap.manifest".to_string(),
            version: CapabilityManifestVersion::V1,
        },
        gate_registry_bound: true,
    };
    let mut trail = AuditTrail::new(session);
    let id = trail.record(AuditEventKind::SessionFinished);
    assert_eq!(id, AuditEventId(0));
    assert_eq!(trail.events().len(), 1);
}
