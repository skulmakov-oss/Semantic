#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloPendingPolicyInput {
    pub operation_kind: String,
    pub payload_type: String,
    pub capability_observation_sink: String,
    pub sink_available: bool,
    pub audit_policy: String,
    pub audit_available: bool,
    pub deterministic_order: bool,
    pub requested_host_channel: Option<String>,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloPendingPolicyReason {
    MissingObservationCapability,
    StdoutNotDefaultSink,
    AuditRequiredButUnavailable,
    NondeterministicSinkConfiguration,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloPendingPolicyResult {
    Admit,
    Deny { reason: HelloPendingPolicyReason },
}

#[cfg(feature = "std")]
pub fn evaluate_hello_pending_policy(input: &HelloPendingPolicyInput) -> HelloPendingPolicyResult {
    if input.capability_observation_sink != "present" || !input.sink_available {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::MissingObservationCapability,
        };
    }

    if matches!(input.requested_host_channel.as_deref(), Some("stdout")) {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::StdoutNotDefaultSink,
        };
    }

    if input.audit_policy == "required" && !input.audit_available {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::AuditRequiredButUnavailable,
        };
    }

    if !input.deterministic_order {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::NondeterministicSinkConfiguration,
        };
    }

    HelloPendingPolicyResult::Admit
}
