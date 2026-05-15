use prom_abi::HostCallId;
use prom_cap::hello_observation_capability::{
    evaluate_hello_observation_capability, HelloObservationCapability,
    HelloObservationCapabilityContext, HelloObservationCapabilityDecision,
    HelloObservationCapabilityDenial, HelloObservationCapabilityPolicy,
};
use prom_cap::{CapabilityKind, required_capability_for_call};

fn allow_context() -> HelloObservationCapabilityContext {
    HelloObservationCapabilityContext {
        observation_sink_present: true,
        sink_available: true,
        requested_host_channel: None,
    }
}

fn missing_capability_context() -> HelloObservationCapabilityContext {
    HelloObservationCapabilityContext {
        observation_sink_present: false,
        sink_available: true,
        requested_host_channel: None,
    }
}

fn sink_unavailable_context() -> HelloObservationCapabilityContext {
    HelloObservationCapabilityContext {
        observation_sink_present: true,
        sink_available: false,
        requested_host_channel: None,
    }
}

fn requested_channel_context(channel: &'static str) -> HelloObservationCapabilityContext {
    HelloObservationCapabilityContext {
        observation_sink_present: true,
        sink_available: true,
        requested_host_channel: Some(channel),
    }
}

#[test]
fn hello_observation_capability_skeleton_allows_explicit_sink_access() {
    let _policy = HelloObservationCapabilityPolicy;
    let capability = HelloObservationCapability::ObservationSink;
    assert_eq!(capability, HelloObservationCapability::ObservationSink);
    assert_eq!(
        evaluate_hello_observation_capability(&allow_context()),
        HelloObservationCapabilityDecision::Allow
    );
}

#[test]
fn hello_observation_capability_skeleton_denies_missing_capability() {
    assert_eq!(
        evaluate_hello_observation_capability(&missing_capability_context()),
        HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::MissingObservationCapability,
        )
    );
}

#[test]
fn hello_observation_capability_skeleton_denies_unavailable_sink() {
    assert_eq!(
        evaluate_hello_observation_capability(&sink_unavailable_context()),
        HelloObservationCapabilityDecision::Deny(HelloObservationCapabilityDenial::SinkUnavailable)
    );
}

#[test]
fn hello_observation_capability_skeleton_denies_stdout_fallback() {
    assert_eq!(
        evaluate_hello_observation_capability(&requested_channel_context("stdout")),
        HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::StdoutNotDefaultSink,
        )
    );
}

#[test]
fn hello_observation_capability_skeleton_denies_generic_io_fallbacks() {
    for channel in ["print", "io.write", "file", "network", "stdin"] {
        assert_eq!(
            evaluate_hello_observation_capability(&requested_channel_context(channel)),
            HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed,
            ),
            "channel {channel} must not be admitted"
        );
    }
}

#[test]
fn hello_observation_capability_skeleton_keeps_existing_host_call_mapping_unchanged() {
    assert_eq!(
        required_capability_for_call(HostCallId::GateRead),
        CapabilityKind::GateRead
    );
    assert_eq!(
        required_capability_for_call(HostCallId::PulseEmit),
        CapabilityKind::PulseEmit
    );
}

