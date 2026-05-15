use std::vec::Vec;

use prom_cap::hello_observation_capability::{
    evaluate_hello_observation_capability, HelloObservationCapabilityContext,
    HelloObservationCapabilityDecision, HelloObservationCapabilityDenial,
};
use sm_runtime_core::hello_observation_route::{
    route_hello_observation_to_sink, HelloObservationRouteError,
    HelloObservationRouteInput, HelloObservationRouteResult,
};
use sm_runtime_core::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink, HelloObservationSinkError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapabilityGatedRouteResult {
    Routed,
    CapabilityDenied(HelloObservationCapabilityDenial),
    RouteDenied(HelloObservationRouteError),
}

#[derive(Default)]
struct InMemorySink {
    events: Vec<HelloObservationEvent>,
    reject: bool,
}

impl HelloObservationSink for InMemorySink {
    fn observe(
        &mut self,
        event: HelloObservationEvent,
    ) -> Result<(), HelloObservationSinkError> {
        if self.reject {
            return Err(HelloObservationSinkError::Denied);
        }
        self.events.push(event);
        Ok(())
    }
}

fn capability_gate_then_route<S: HelloObservationSink>(
    capability_context: HelloObservationCapabilityContext,
    route_input: HelloObservationRouteInput,
    sink: &mut S,
) -> CapabilityGatedRouteResult {
    match evaluate_hello_observation_capability(&capability_context) {
        HelloObservationCapabilityDecision::Allow => {
            match route_hello_observation_to_sink(route_input, sink) {
                HelloObservationRouteResult::Routed => CapabilityGatedRouteResult::Routed,
                HelloObservationRouteResult::NotRouted(err) => {
                    CapabilityGatedRouteResult::RouteDenied(err)
                }
            }
        }
        HelloObservationCapabilityDecision::Deny(reason) => {
            CapabilityGatedRouteResult::CapabilityDenied(reason)
        }
    }
}

fn allowed_capability_context() -> HelloObservationCapabilityContext {
    HelloObservationCapabilityContext {
        observation_sink_present: true,
        sink_available: true,
        requested_host_channel: None,
    }
}

fn admitted_route_input() -> HelloObservationRouteInput {
    HelloObservationRouteInput {
        admitted: true,
        text: "Hello, World!".to_string(),
        sequence_index: HelloObservationSequenceIndex(0),
    }
}

#[test]
fn hello_capability_gated_route_routes_admitted_hello_only_after_allow() {
    let mut sink = InMemorySink::default();
    let result = capability_gate_then_route(
        allowed_capability_context(),
        admitted_route_input(),
        &mut sink,
    );

    assert!(matches!(result, CapabilityGatedRouteResult::Routed));
    assert_eq!(sink.events.len(), 1);
    let event = &sink.events[0];
    assert_eq!(event.operation_kind, "controlled_observation_text");
    assert_eq!(event.observation_class, HelloObservationClass::ControlledText);
    assert_eq!(event.text, "Hello, World!");
    assert_eq!(event.sequence_index, HelloObservationSequenceIndex(0));
}

#[test]
fn hello_capability_gated_route_denies_before_route_when_capability_denied() {
    for (context, expected_reason) in [
        (
            HelloObservationCapabilityContext {
            observation_sink_present: false,
            sink_available: true,
            requested_host_channel: None,
        },
            HelloObservationCapabilityDenial::MissingObservationCapability,
        ),
        (
            HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: false,
            requested_host_channel: None,
        },
            HelloObservationCapabilityDenial::SinkUnavailable,
        ),
        (
            HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: Some("stdout"),
        },
            HelloObservationCapabilityDenial::StdoutNotDefaultSink,
        ),
        (
            HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: Some("print"),
        },
            HelloObservationCapabilityDenial::GenericIoNotAllowed,
        ),
        (
            HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: Some("io.write"),
        },
            HelloObservationCapabilityDenial::GenericIoNotAllowed,
        ),
        (
            HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: Some("file"),
        },
            HelloObservationCapabilityDenial::GenericIoNotAllowed,
        ),
        (
            HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: Some("network"),
        },
            HelloObservationCapabilityDenial::GenericIoNotAllowed,
        ),
        (
            HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: Some("stdin"),
        },
            HelloObservationCapabilityDenial::GenericIoNotAllowed,
        ),
    ] {
        let mut sink = InMemorySink::default();
        let result = capability_gate_then_route(context, admitted_route_input(), &mut sink);
        assert_eq!(
            result,
            CapabilityGatedRouteResult::CapabilityDenied(expected_reason)
        );
        assert!(sink.events.is_empty());
    }
}

#[test]
fn hello_capability_gated_route_respects_route_rejection_and_non_admitted_inputs() {
    let mut sink = InMemorySink::default();

    let not_admitted = HelloObservationRouteInput {
        admitted: false,
        text: "Hello, World!".to_string(),
        sequence_index: HelloObservationSequenceIndex(1),
    };
    let result = capability_gate_then_route(allowed_capability_context(), not_admitted, &mut sink);
    assert!(matches!(
        result,
        CapabilityGatedRouteResult::RouteDenied(HelloObservationRouteError::NotAdmitted)
    ));
    assert!(sink.events.is_empty());

    for forbidden in ["stdout", "print", "io.write", "file", "network", "stdin"] {
        let result = capability_gate_then_route(
            allowed_capability_context(),
            HelloObservationRouteInput {
                admitted: true,
                text: forbidden.to_string(),
                sequence_index: HelloObservationSequenceIndex(2),
            },
            &mut sink,
        );
        assert!(matches!(
            result,
            CapabilityGatedRouteResult::RouteDenied(
                HelloObservationRouteError::ForbiddenHostOutput
            )
        ));
    }

    let non_controlled = capability_gate_then_route(
        allowed_capability_context(),
        HelloObservationRouteInput {
            admitted: true,
            text: "Not Hello".to_string(),
            sequence_index: HelloObservationSequenceIndex(3),
        },
        &mut sink,
    );
    assert!(matches!(
        non_controlled,
        CapabilityGatedRouteResult::RouteDenied(HelloObservationRouteError::NonControlledText)
    ));
}
