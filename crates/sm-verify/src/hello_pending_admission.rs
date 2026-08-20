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
    GenericIoNotAllowed,
    AuditRequiredButUnavailable,
    NondeterministicSinkConfiguration,
    UnsupportedOperationKind,
    UnsupportedPayloadType,
    UnsupportedAuditPolicy,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloPendingPolicyResult {
    Admit,
    Deny { reason: HelloPendingPolicyReason },
}

#[cfg(feature = "std")]
const CONTROLLED_OPERATION_KIND: &str = "controlled_observation_text";
#[cfg(feature = "std")]
const TEXT_LITERAL_PAYLOAD_TYPE: &str = "text_literal";

#[cfg(feature = "std")]
pub fn evaluate_hello_pending_policy(input: &HelloPendingPolicyInput) -> HelloPendingPolicyResult {
    if input.capability_observation_sink != "present" || !input.sink_available {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::MissingObservationCapability,
        };
    }

    // Identity-shape checks run right after the capability gate and before
    // channel/audit/ordering policy: whether we CAN observe at all takes
    // priority over what is being observed, which in turn takes priority
    // over how it is observed. operation_kind is checked before
    // payload_type, so when both are unsupported, UnsupportedOperationKind
    // wins deterministically.
    if input.operation_kind != CONTROLLED_OPERATION_KIND {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::UnsupportedOperationKind,
        };
    }
    if input.payload_type != TEXT_LITERAL_PAYLOAD_TYPE {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::UnsupportedPayloadType,
        };
    }

    match input.requested_host_channel.as_deref() {
        Some("stdout") => {
            return HelloPendingPolicyResult::Deny {
                reason: HelloPendingPolicyReason::StdoutNotDefaultSink,
            };
        }
        Some("print") | Some("io.write") | Some("file") | Some("network") | Some("stdin") => {
            return HelloPendingPolicyResult::Deny {
                reason: HelloPendingPolicyReason::GenericIoNotAllowed,
            };
        }
        _ => {}
    }

    match input.audit_policy.as_str() {
        "required" => {
            if !input.audit_available {
                return HelloPendingPolicyResult::Deny {
                    reason: HelloPendingPolicyReason::AuditRequiredButUnavailable,
                };
            }
        }
        _ => {
            return HelloPendingPolicyResult::Deny {
                reason: HelloPendingPolicyReason::UnsupportedAuditPolicy,
            };
        }
    }

    if !input.deterministic_order {
        return HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::NondeterministicSinkConfiguration,
        };
    }

    HelloPendingPolicyResult::Admit
}
