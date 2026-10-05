//! Isolated Hello observation capability skeleton.
//!
//! This module models future capability admission for controlled Hello
//! observation without wiring into production capability handling.

use super::{CapabilityChecker, CapabilityDeniedCode, CapabilityKind};

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
    /// The checker's capability contract is itself invalid (#1780); this is
    /// not an ordinary missing `ControlledObservationSink` grant.
    InvalidCapabilityContract,
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
        Some(_) => {
            return HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed,
            );
        }
        None => {}
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
        Some(_) => {
            return HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed,
            );
        }
        None => {}
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
        Err(denied) => HelloObservationCapabilityDecision::Deny(match denied.code {
            CapabilityDeniedCode::MissingCapability => {
                HelloObservationCapabilityDenial::MissingObservationCapability
            }
            CapabilityDeniedCode::InvalidManifestContract(_) => {
                HelloObservationCapabilityDenial::InvalidCapabilityContract
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CapabilityManifest;

    #[test]
    fn unknown_host_channel_is_denied() {
        let context = HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: Some("socket"),
        };

        assert_eq!(
            evaluate_hello_observation_capability(&context),
            HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed
            )
        );

        let mut manifest = CapabilityManifest::new();
        manifest.allow(CapabilityKind::ControlledObservationSink);
        assert_eq!(
            require_hello_observation_sink_capability(&manifest, &context),
            HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed
            )
        );
    }

    // #1781 (resolved by PR #1973): every unknown channel is denied by both
    // evaluators, and `None` proceeds to the ordinary controlled-sink checks.
    #[test]
    fn pb07_unknown_host_channels_fail_closed_and_none_reaches_sink_checks() {
        let mut manifest = CapabilityManifest::new();
        manifest.allow(CapabilityKind::ControlledObservationSink);
        for channel in ["socket", "udp", "stderr2", "pb07-arbitrary-future-channel"] {
            let context = HelloObservationCapabilityContext {
                observation_sink_present: true,
                sink_available: true,
                requested_host_channel: Some(channel),
            };
            let denied = HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::GenericIoNotAllowed,
            );
            assert_eq!(
                evaluate_hello_observation_capability(&context),
                denied,
                "{channel}"
            );
            assert_eq!(
                require_hello_observation_sink_capability(&manifest, &context),
                denied,
                "{channel}"
            );
        }
        let none = HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: None,
        };
        assert_eq!(
            require_hello_observation_sink_capability(&manifest, &none),
            HelloObservationCapabilityDecision::Allow
        );
        let unavailable = HelloObservationCapabilityContext {
            sink_available: false,
            ..none
        };
        assert_eq!(
            require_hello_observation_sink_capability(&manifest, &unavailable),
            HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::SinkUnavailable
            )
        );
    }

    // #1780: an invalid checker contract is not a missing observation grant.
    #[test]
    fn pb07_invalid_checker_contract_is_distinct_from_missing_sink_capability() {
        let context = HelloObservationCapabilityContext {
            observation_sink_present: true,
            sink_available: true,
            requested_host_channel: None,
        };
        let mut invalid =
            CapabilityManifest::with_contract("other.schema", crate::CapabilityManifestVersion::V1);
        invalid.allow(CapabilityKind::ControlledObservationSink);
        assert_eq!(
            require_hello_observation_sink_capability(&invalid, &context),
            HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::InvalidCapabilityContract
            )
        );
        assert_eq!(
            require_hello_observation_sink_capability(&CapabilityManifest::new(), &context),
            HelloObservationCapabilityDecision::Deny(
                HelloObservationCapabilityDenial::MissingObservationCapability
            )
        );
    }
}
