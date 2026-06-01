//! Isolated Hello observation capability skeleton.
//!
//! This module models future capability admission for controlled Hello
//! observation without wiring into production capability handling.

use super::{CapabilityChecker, CapabilityKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationCapability {
    ObservationSink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HelloObservationCapabilityPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelloObservationCapabilityContext {
    pub observation_sink_present: bool,
    pub sink_available: bool,
    pub requested_host_channel: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationCapabilityDecision {
    Allow,
    Deny(HelloObservationCapabilityDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationCapabilityDenial {
    MissingObservationCapability,
    SinkUnavailable,
    StdoutNotDefaultSink,
    GenericIoNotAllowed,
}

pub fn evaluate_hello_observation_capability(
    context: &HelloObservationCapabilityContext,
) -> HelloObservationCapabilityDecision {
    match context.requested_host_channel {
        Some("stdout") => {
            return HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::StdoutNotDefaultSink,
            );
        }
        Some("print") | Some("io.write") | Some("file") | Some("network") | Some("stdin") => {
            return HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed,
            );
        }
        _ => {}
    }

    if !context.observation_sink_present {
        return HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::MissingObservationCapability,
        );
    }

    if !context.sink_available {
        return HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::SinkUnavailable,
        );
    }

    HelloObservationCapabilityDecision::Allow
}

pub fn require_hello_observation_sink_capability<C: CapabilityChecker>(
    checker: &C,
    context: &HelloObservationCapabilityContext,
) -> HelloObservationCapabilityDecision {
    match context.requested_host_channel {
        Some("stdout") => {
            return HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::StdoutNotDefaultSink,
            );
        }
        Some("print") | Some("io.write") | Some("file") | Some("network") | Some("stdin") => {
            return HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed,
            );
        }
        _ => {}
    }

    if !context.observation_sink_present {
        return HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::MissingObservationCapability,
        );
    }

    if !context.sink_available {
        return HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::SinkUnavailable,
        );
    }

    match checker.require(CapabilityKind::ControlledObservationSink) {
        Ok(()) => HelloObservationCapabilityDecision::Allow,
        Err(_) => HelloObservationCapabilityDecision::Deny(
            HelloObservationCapabilityDenial::MissingObservationCapability,
        ),
    }
}
