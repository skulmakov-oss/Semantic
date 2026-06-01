use prom_abi::HostCallId;
use prom_cap::hello_observation_capability::{
    evaluate_hello_observation_capability, require_hello_observation_sink_capability,
    HelloObservationCapability, HelloObservationCapabilityContext,
    HelloObservationCapabilityDecision, HelloObservationCapabilityDenial,
    HelloObservationCapabilityPolicy,
};
use prom_cap::{
    capability_surface_class, required_capability_for_call, CapabilityKind, CapabilityManifest,
    CapabilitySurfaceClass,
};

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
fn hello_observation_capability_skeleton_requires_explicit_controlled_observation_sink_capability()
{
    let mut manifest = CapabilityManifest::new();
    let context = allow_context();

    assert_eq!(
        require_hello_observation_sink_capability(&manifest, &context),
        HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::MissingObservationCapability,
        )
    );

    manifest.allow(CapabilityKind::ControlledObservationSink);
    assert_eq!(
        require_hello_observation_sink_capability(&manifest, &context),
        HelloObservationCapabilityDecision::Allow
    );

    let gate_surface = CapabilityManifest::gate_surface();
    assert!(!gate_surface.allows(CapabilityKind::ControlledObservationSink));
    assert_eq!(
        capability_surface_class(CapabilityKind::ControlledObservationSink),
        CapabilitySurfaceClass::PlannedPostStable
    );
}

#[test]
fn hello_observation_capability_skeleton_keeps_controlled_observation_separate_from_stdout() {
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::ControlledObservationSink);

    for channel in ["stdout", "print", "io.write", "file", "network", "stdin"] {
        let context = requested_channel_context(channel);
        assert_eq!(
            require_hello_observation_sink_capability(&manifest, &context),
            if channel == "stdout" {
                HelloObservationCapabilityDecision::Deny(
                    HelloObservationCapabilityDenial::StdoutNotDefaultSink,
                )
            } else {
                HelloObservationCapabilityDecision::Deny(
                    HelloObservationCapabilityDenial::GenericIoNotAllowed,
                )
            },
            "channel {channel} must be denied"
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
