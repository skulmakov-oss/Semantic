//! Isolated Hello observation audit event skeleton.
//!
//! This module models future audit records for controlled Hello observation
//! without wiring into production audit storage or runtime routing.

use crate::{AuditEventId, AuditEventKind, AuditTrail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationAuditEventKind {
    Observation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationAuditPolicyClass {
    Required,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationAuditPayloadRef {
    LiteralTextHash(u64),
    Redacted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HelloObservationAuditSequenceIndex(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelloObservationAuditLinkage {
    pub verifier_admission_ref: Option<u64>,
    pub capability_policy_ref: Option<u64>,
    pub sink_policy_ref: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelloObservationAuditEvent {
    pub event_kind: HelloObservationAuditEventKind,
    pub operation_kind: &'static str,
    pub observation_class: &'static str,
    pub payload_ref: HelloObservationAuditPayloadRef,
    pub sequence_index: HelloObservationAuditSequenceIndex,
    pub audit_policy_class: HelloObservationAuditPolicyClass,
    pub linkage: HelloObservationAuditLinkage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledObservationAuditDecision {
    Record,
    Redact,
    NoStore,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledObservationAuditResult {
    Recorded(AuditEventId),
    NoStore,
    Denied,
}

pub fn build_hello_observation_audit_event(
    payload_hash: u64,
    sequence_index: u64,
    audit_policy_class: HelloObservationAuditPolicyClass,
    linkage: HelloObservationAuditLinkage,
) -> HelloObservationAuditEvent {
    HelloObservationAuditEvent {
        event_kind: HelloObservationAuditEventKind::Observation,
        operation_kind: "controlled_observation_text",
        observation_class: "controlled",
        payload_ref: HelloObservationAuditPayloadRef::LiteralTextHash(payload_hash),
        sequence_index: HelloObservationAuditSequenceIndex(sequence_index),
        audit_policy_class,
        linkage,
    }
}

pub fn apply_controlled_observation_audit_policy(
    trail: &mut AuditTrail,
    text_hash: u64,
    sequence_index: u64,
    policy: ControlledObservationAuditDecision,
    linkage: HelloObservationAuditLinkage,
) -> ControlledObservationAuditResult {
    match policy {
        ControlledObservationAuditDecision::Record => {
            let event_id = trail.record(AuditEventKind::ControlledObservation {
                operation_kind: "controlled_observation_text".to_string(),
                observation_class: "ControlledText".to_string(),
                payload_hash: Some(text_hash),
                redacted: false,
                sequence_index,
                policy,
                linkage,
            });
            ControlledObservationAuditResult::Recorded(event_id)
        }
        ControlledObservationAuditDecision::Redact => {
            let event_id = trail.record(AuditEventKind::ControlledObservation {
                operation_kind: "controlled_observation_text".to_string(),
                observation_class: "ControlledText".to_string(),
                payload_hash: None,
                redacted: true,
                sequence_index,
                policy,
                linkage,
            });
            ControlledObservationAuditResult::Recorded(event_id)
        }
        ControlledObservationAuditDecision::NoStore => ControlledObservationAuditResult::NoStore,
        ControlledObservationAuditDecision::Deny => ControlledObservationAuditResult::Denied,
    }
}
