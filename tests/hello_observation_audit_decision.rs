use std::vec::Vec;

use prom_audit::hello_observation_audit::{
    build_hello_observation_audit_event, HelloObservationAuditEvent,
    HelloObservationAuditEventKind, HelloObservationAuditLinkage, HelloObservationAuditPayloadRef,
    HelloObservationAuditPolicyClass,
};
use sm_runtime_core::hello_observation_route::{
    route_hello_observation_to_sink, HelloObservationRouteError, HelloObservationRouteInput,
    HelloObservationRouteResult,
};
use sm_runtime_core::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink, HelloObservationSinkError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum HelloObservationAuditDecision {
    Recorded(HelloObservationAuditEvent),
    Deferred,
    NotRecorded(&'static str),
}

#[derive(Default)]
struct InMemorySink {
    events: Vec<HelloObservationEvent>,
    reject: bool,
}

impl HelloObservationSink for InMemorySink {
    fn observe(&mut self, event: HelloObservationEvent) -> Result<(), HelloObservationSinkError> {
        if self.reject {
            return Err(HelloObservationSinkError::Denied);
        }
        self.events.push(event);
        Ok(())
    }
}

fn deterministic_text_hash(text: &str, sequence_index: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325u64 ^ sequence_index;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn route_with_sink(
    admitted: bool,
    text: &str,
    sequence_index: u64,
    sink: &mut InMemorySink,
) -> HelloObservationRouteResult {
    route_hello_observation_to_sink(
        HelloObservationRouteInput {
            admitted,
            text: text.to_string(),
            sequence_index: HelloObservationSequenceIndex(sequence_index),
        },
        sink,
    )
}

fn audit_after_route(
    route_result: HelloObservationRouteResult,
    text: &str,
    sequence_index: u64,
    audit_policy_class: HelloObservationAuditPolicyClass,
    linkage: HelloObservationAuditLinkage,
) -> HelloObservationAuditDecision {
    match route_result {
        HelloObservationRouteResult::Routed => match audit_policy_class {
            HelloObservationAuditPolicyClass::Deferred => HelloObservationAuditDecision::Deferred,
            HelloObservationAuditPolicyClass::Required => {
                HelloObservationAuditDecision::Recorded(build_hello_observation_audit_event(
                    deterministic_text_hash(text, sequence_index),
                    sequence_index,
                    HelloObservationAuditPolicyClass::Required,
                    linkage,
                ))
            }
        },
        HelloObservationRouteResult::NotRouted(_) => {
            HelloObservationAuditDecision::NotRecorded("route_not_routed")
        }
    }
}

#[test]
fn hello_observation_audit_decision_records_required_routed_observation() {
    let mut sink = InMemorySink::default();
    let route_result = route_with_sink(true, "Hello, World!", 0, &mut sink);
    assert_eq!(route_result, HelloObservationRouteResult::Routed);
    assert_eq!(sink.events.len(), 1);
    let event = &sink.events[0];
    assert_eq!(event.operation_kind, "controlled_observation_text");
    assert_eq!(
        event.observation_class,
        HelloObservationClass::ControlledText
    );
    assert_eq!(event.text, "Hello, World!");
    assert_eq!(event.sequence_index, HelloObservationSequenceIndex(0));

    let decision = audit_after_route(
        route_result,
        "Hello, World!",
        0,
        HelloObservationAuditPolicyClass::Required,
        HelloObservationAuditLinkage {
            verifier_admission_ref: Some(17),
            capability_policy_ref: Some(23),
            sink_policy_ref: Some(31),
        },
    );

    match decision {
        HelloObservationAuditDecision::Recorded(recorded) => {
            assert_eq!(
                recorded.event_kind,
                HelloObservationAuditEventKind::Observation
            );
            assert_eq!(recorded.operation_kind, "controlled_observation_text");
            assert_eq!(recorded.observation_class, "controlled");
            assert_eq!(
                recorded.payload_ref,
                HelloObservationAuditPayloadRef::LiteralTextHash(deterministic_text_hash(
                    "Hello, World!",
                    0,
                ))
            );
            assert_eq!(
                recorded.sequence_index,
                prom_audit::hello_observation_audit::HelloObservationAuditSequenceIndex(0)
            );
            assert_eq!(
                recorded.audit_policy_class,
                HelloObservationAuditPolicyClass::Required
            );
            assert_eq!(
                recorded.linkage,
                HelloObservationAuditLinkage {
                    verifier_admission_ref: Some(17),
                    capability_policy_ref: Some(23),
                    sink_policy_ref: Some(31),
                }
            );
        }
        other => panic!("expected recorded audit event, got {other:?}"),
    }
}

#[test]
fn hello_observation_audit_decision_deferred_stays_local() {
    let mut sink = InMemorySink::default();
    let route_result = route_with_sink(true, "Hello, World!", 0, &mut sink);
    assert_eq!(route_result, HelloObservationRouteResult::Routed);

    let decision = audit_after_route(
        route_result,
        "Hello, World!",
        0,
        HelloObservationAuditPolicyClass::Deferred,
        HelloObservationAuditLinkage {
            verifier_admission_ref: Some(2),
            capability_policy_ref: Some(3),
            sink_policy_ref: Some(5),
        },
    );

    assert_eq!(decision, HelloObservationAuditDecision::Deferred);
    assert_eq!(sink.events.len(), 1);
}

#[test]
fn hello_observation_audit_decision_does_not_record_for_not_routed_variants() {
    let mut sink = InMemorySink::default();

    let not_admitted = route_with_sink(false, "Hello, World!", 0, &mut sink);
    assert_eq!(
        not_admitted,
        HelloObservationRouteResult::NotRouted(HelloObservationRouteError::NotAdmitted)
    );
    assert_eq!(
        audit_after_route(
            not_admitted,
            "Hello, World!",
            0,
            HelloObservationAuditPolicyClass::Required,
            HelloObservationAuditLinkage {
                verifier_admission_ref: None,
                capability_policy_ref: None,
                sink_policy_ref: None,
            },
        ),
        HelloObservationAuditDecision::NotRecorded("route_not_routed")
    );

    for forbidden in ["stdout", "print", "io.write", "file", "network", "stdin"] {
        let route_result = route_with_sink(true, forbidden, 1, &mut sink);
        assert_eq!(
            route_result,
            HelloObservationRouteResult::NotRouted(HelloObservationRouteError::ForbiddenHostOutput)
        );
        assert_eq!(
            audit_after_route(
                route_result,
                forbidden,
                1,
                HelloObservationAuditPolicyClass::Required,
                HelloObservationAuditLinkage {
                    verifier_admission_ref: None,
                    capability_policy_ref: None,
                    sink_policy_ref: None,
                },
            ),
            HelloObservationAuditDecision::NotRecorded("route_not_routed")
        );
    }

    let sink_rejected = route_with_sink(
        true,
        "Hello, World!",
        2,
        &mut InMemorySink {
            events: Vec::new(),
            reject: true,
        },
    );
    assert_eq!(
        sink_rejected,
        HelloObservationRouteResult::NotRouted(HelloObservationRouteError::SinkRejected)
    );
    assert_eq!(
        audit_after_route(
            sink_rejected,
            "Hello, World!",
            2,
            HelloObservationAuditPolicyClass::Required,
            HelloObservationAuditLinkage {
                verifier_admission_ref: None,
                capability_policy_ref: None,
                sink_policy_ref: None,
            },
        ),
        HelloObservationAuditDecision::NotRecorded("route_not_routed")
    );

    let non_controlled = route_with_sink(true, "Not Hello", 3, &mut sink);
    assert_eq!(
        non_controlled,
        HelloObservationRouteResult::NotRouted(HelloObservationRouteError::NonControlledText)
    );
    assert_eq!(
        audit_after_route(
            non_controlled,
            "Not Hello",
            3,
            HelloObservationAuditPolicyClass::Required,
            HelloObservationAuditLinkage {
                verifier_admission_ref: None,
                capability_policy_ref: None,
                sink_policy_ref: None,
            },
        ),
        HelloObservationAuditDecision::NotRecorded("route_not_routed")
    );
}
