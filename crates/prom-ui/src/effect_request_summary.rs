//! Semantic UI effect request summary scaffold.
//!
//! This module summarizes inert effect request trace reports for future
//! UI/debug/Workbench consumption. It does not implement capability admission,
//! prepared effects, committed effects, effect execution, VM/Host ABI calls,
//! audit authority, or runtime mutation.

use alloc::vec::Vec;

use crate::effect_request::{
    InteractionEffectRequestRuntimeCapability, InteractionEffectRequestUiCapability,
};
use crate::effect_request_trace::{
    InteractionEffectRequestTraceReport, InteractionEffectRequestTraceStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionEffectRequestKindCount {
    pub kind: crate::effect_request::InteractionEffectRequestKind,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionEffectRequestUiCapabilityCount {
    pub capability: InteractionEffectRequestUiCapability,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionEffectRequestRuntimeCapabilityCount {
    pub capability: InteractionEffectRequestRuntimeCapability,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionEffectRequestSummary {
    pub total: usize,
    pub described: usize,
    pub kinds: Vec<InteractionEffectRequestKindCount>,
    pub ui_capabilities: Vec<InteractionEffectRequestUiCapabilityCount>,
    pub runtime_capabilities: Vec<InteractionEffectRequestRuntimeCapabilityCount>,
}

pub fn summarize_interaction_effect_requests(
    traces: &[InteractionEffectRequestTraceReport],
) -> InteractionEffectRequestSummary {
    let mut summary = InteractionEffectRequestSummary {
        total: traces.len(),
        described: 0,
        kinds: Vec::new(),
        ui_capabilities: Vec::new(),
        runtime_capabilities: Vec::new(),
    };

    for trace in traces {
        if matches!(trace.trace_status, InteractionEffectRequestTraceStatus::Described) {
            summary.described += 1;
        }

        increment_kind_count(&mut summary.kinds, trace.requested_effect);
        increment_ui_capability_count(&mut summary.ui_capabilities, trace.required_ui_capability);
        increment_runtime_capability_count(
            &mut summary.runtime_capabilities,
            trace.required_runtime_capability,
        );
    }

    summary
}

fn increment_kind_count(
    counts: &mut Vec<InteractionEffectRequestKindCount>,
    kind: crate::effect_request::InteractionEffectRequestKind,
) {
    if let Some(entry) = counts.iter_mut().find(|entry| entry.kind == kind) {
        entry.count += 1;
    } else {
        counts.push(InteractionEffectRequestKindCount { kind, count: 1 });
    }
}

fn increment_ui_capability_count(
    counts: &mut Vec<InteractionEffectRequestUiCapabilityCount>,
    capability: InteractionEffectRequestUiCapability,
) {
    if let Some(entry) = counts
        .iter_mut()
        .find(|entry| entry.capability == capability)
    {
        entry.count += 1;
    } else {
        counts.push(InteractionEffectRequestUiCapabilityCount {
            capability,
            count: 1,
        });
    }
}

fn increment_runtime_capability_count(
    counts: &mut Vec<InteractionEffectRequestRuntimeCapabilityCount>,
    capability: InteractionEffectRequestRuntimeCapability,
) {
    if let Some(entry) = counts
        .iter_mut()
        .find(|entry| entry.capability == capability)
    {
        entry.count += 1;
    } else {
        counts.push(InteractionEffectRequestRuntimeCapabilityCount {
            capability,
            count: 1,
        });
    }
}

impl InteractionEffectRequestSummary {
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    pub const fn is_capability_admission(&self) -> bool {
        false
    }

    pub const fn is_prepared_effect(&self) -> bool {
        false
    }

    pub const fn is_committed_effect(&self) -> bool {
        false
    }

    pub const fn is_execution_authority(&self) -> bool {
        false
    }

    pub const fn is_runtime_mutation(&self) -> bool {
        false
    }

    pub const fn is_audit_authority(&self) -> bool {
        false
    }

    pub const fn calls_vm_or_host_abi(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect_request::{
        InteractionEffectRequestDenialBehavior, InteractionEffectRequestDescriptorId,
        InteractionEffectRequestKind,
        InteractionEffectRequestLifecyclePrecondition,
        InteractionEffectRequestPrepareCommitRelationship,
        InteractionEffectRequestRuntimeCapability, InteractionEffectRequestScope,
        InteractionEffectRequestTargetPolicy, InteractionEffectRequestUiCapability,
    };
    use crate::effect_request_trace::{
        InteractionEffectRequestTraceReason, InteractionEffectRequestTraceReport,
        InteractionEffectRequestTraceStatus,
    };
    use crate::action_admission::InteractionActionAdmissionPolicyGateNamespace;
    use crate::action_binding::InteractionActionName;
    use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
    use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
    use crate::admitted_action::InteractionAdmittedSemanticActionId;
    use crate::interaction::InteractionIntentKind;

    fn effect_trace(
        id: u64,
        kind: InteractionEffectRequestKind,
        ui_capability: InteractionEffectRequestUiCapability,
        runtime_capability: InteractionEffectRequestRuntimeCapability,
    ) -> InteractionEffectRequestTraceReport {
        InteractionEffectRequestTraceReport {
            descriptor_id: InteractionEffectRequestDescriptorId(id),
            source_admitted_action_id: InteractionAdmittedSemanticActionId(id),
            dispatch_record_id: InteractionSemanticActionDispatchRecordId(id),
            dispatch_route_id: InteractionSemanticActionDispatchRouteId(id),
            action: match kind {
                InteractionEffectRequestKind::PrepareEffect => InteractionActionName::PrepareEffect,
                InteractionEffectRequestKind::CommitEffect => InteractionActionName::CommitEffect,
                InteractionEffectRequestKind::RollbackEffect => {
                    InteractionActionName::RollbackEffect
                }
                InteractionEffectRequestKind::CloseWindow => InteractionActionName::CloseWindow,
                InteractionEffectRequestKind::QuarantineTarget => {
                    InteractionActionName::QuarantineTarget
                }
                InteractionEffectRequestKind::Unknown => InteractionActionName::Unknown,
            },
            source_intent: match kind {
                InteractionEffectRequestKind::CloseWindow => InteractionIntentKind::Close,
                InteractionEffectRequestKind::PrepareEffect => InteractionIntentKind::Submit,
                InteractionEffectRequestKind::CommitEffect => InteractionIntentKind::Submit,
                InteractionEffectRequestKind::RollbackEffect => InteractionIntentKind::Cancel,
                InteractionEffectRequestKind::QuarantineTarget => InteractionIntentKind::Cancel,
                InteractionEffectRequestKind::Unknown => InteractionIntentKind::Unknown,
            },
            requested_effect: kind,
            target_policy: match kind {
                InteractionEffectRequestKind::CloseWindow => InteractionEffectRequestTargetPolicy::Optional,
                InteractionEffectRequestKind::PrepareEffect
                | InteractionEffectRequestKind::CommitEffect
                | InteractionEffectRequestKind::RollbackEffect
                | InteractionEffectRequestKind::QuarantineTarget => {
                    InteractionEffectRequestTargetPolicy::Required
                }
                InteractionEffectRequestKind::Unknown => InteractionEffectRequestTargetPolicy::Ignored,
            },
            required_ui_capability: ui_capability,
            required_runtime_capability: runtime_capability,
            lifecycle_precondition: match kind {
                InteractionEffectRequestKind::CloseWindow => {
                    InteractionEffectRequestLifecyclePrecondition::WindowAlive
                }
                InteractionEffectRequestKind::PrepareEffect
                | InteractionEffectRequestKind::CommitEffect
                | InteractionEffectRequestKind::RollbackEffect => {
                    InteractionEffectRequestLifecyclePrecondition::SessionActive
                }
                InteractionEffectRequestKind::QuarantineTarget => {
                    InteractionEffectRequestLifecyclePrecondition::TargetPresent
                }
                InteractionEffectRequestKind::Unknown => {
                    InteractionEffectRequestLifecyclePrecondition::Unknown
                }
            },
            trace_requirement: crate::action_admission::InteractionActionAdmissionTraceRequirement::Required,
            denial_behavior: match kind {
                InteractionEffectRequestKind::CloseWindow
                | InteractionEffectRequestKind::PrepareEffect
                | InteractionEffectRequestKind::CommitEffect
                | InteractionEffectRequestKind::RollbackEffect
                | InteractionEffectRequestKind::QuarantineTarget => {
                    InteractionEffectRequestDenialBehavior::VisibleRequired
                }
                InteractionEffectRequestKind::Unknown => InteractionEffectRequestDenialBehavior::Unknown,
            },
            prepare_commit_relationship: match kind {
                InteractionEffectRequestKind::CommitEffect => {
                    InteractionEffectRequestPrepareCommitRelationship::CommitAfterPrepare
                }
                InteractionEffectRequestKind::RollbackEffect => {
                    InteractionEffectRequestPrepareCommitRelationship::RollbackAfterPrepare
                }
                InteractionEffectRequestKind::CloseWindow
                | InteractionEffectRequestKind::PrepareEffect
                | InteractionEffectRequestKind::QuarantineTarget => {
                    InteractionEffectRequestPrepareCommitRelationship::PrepareThenCommit
                }
                InteractionEffectRequestKind::Unknown => {
                    InteractionEffectRequestPrepareCommitRelationship::Unknown
                }
            },
            policy_gate_namespace: if id % 2 == 0 {
                InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal
            } else {
                InteractionActionAdmissionPolicyGateNamespace::CoreUi
            },
            scope: if id % 2 == 0 {
                InteractionEffectRequestScope::WorkbenchLocal
            } else {
                InteractionEffectRequestScope::CoreUi
            },
            trace_status: InteractionEffectRequestTraceStatus::Described,
            trace_reason: InteractionEffectRequestTraceReason::DescriptorRecorded,
        }
    }

    #[test]
    fn empty_summary_is_zeroed() {
        let summary = summarize_interaction_effect_requests(&[]);

        assert!(summary.is_empty());
        assert_eq!(summary.total, 0);
        assert_eq!(summary.described, 0);
        assert!(summary.kinds.is_empty());
        assert!(summary.ui_capabilities.is_empty());
        assert!(summary.runtime_capabilities.is_empty());
    }

    #[test]
    fn summary_counts_total_and_described() {
        let traces = vec![
            effect_trace(
                1,
                InteractionEffectRequestKind::PrepareEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                2,
                InteractionEffectRequestKind::CloseWindow,
                InteractionEffectRequestUiCapability::WindowLifecycle,
                InteractionEffectRequestRuntimeCapability::WindowLifecycle,
            ),
        ];

        let summary = summarize_interaction_effect_requests(&traces);

        assert_eq!(summary.total, 2);
        assert_eq!(summary.described, 2);
    }

    #[test]
    fn summary_counts_effect_request_kinds() {
        let traces = vec![
            effect_trace(
                1,
                InteractionEffectRequestKind::PrepareEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                2,
                InteractionEffectRequestKind::PrepareEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                3,
                InteractionEffectRequestKind::CloseWindow,
                InteractionEffectRequestUiCapability::WindowLifecycle,
                InteractionEffectRequestRuntimeCapability::WindowLifecycle,
            ),
        ];

        let summary = summarize_interaction_effect_requests(&traces);

        assert!(summary.kinds.contains(&InteractionEffectRequestKindCount {
            kind: InteractionEffectRequestKind::PrepareEffect,
            count: 2,
        }));
        assert!(summary.kinds.contains(&InteractionEffectRequestKindCount {
            kind: InteractionEffectRequestKind::CloseWindow,
            count: 1,
        }));
    }

    #[test]
    fn summary_counts_ui_capabilities() {
        let traces = vec![
            effect_trace(
                1,
                InteractionEffectRequestKind::PrepareEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                2,
                InteractionEffectRequestKind::RollbackEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                3,
                InteractionEffectRequestKind::CloseWindow,
                InteractionEffectRequestUiCapability::WindowLifecycle,
                InteractionEffectRequestRuntimeCapability::WindowLifecycle,
            ),
        ];

        let summary = summarize_interaction_effect_requests(&traces);

        assert!(summary.ui_capabilities.contains(&InteractionEffectRequestUiCapabilityCount {
            capability: InteractionEffectRequestUiCapability::EffectControl,
            count: 2,
        }));
        assert!(summary.ui_capabilities.contains(&InteractionEffectRequestUiCapabilityCount {
            capability: InteractionEffectRequestUiCapability::WindowLifecycle,
            count: 1,
        }));
    }

    #[test]
    fn summary_counts_runtime_capabilities() {
        let traces = vec![
            effect_trace(
                1,
                InteractionEffectRequestKind::PrepareEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                2,
                InteractionEffectRequestKind::RollbackEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                3,
                InteractionEffectRequestKind::CloseWindow,
                InteractionEffectRequestUiCapability::WindowLifecycle,
                InteractionEffectRequestRuntimeCapability::WindowLifecycle,
            ),
        ];

        let summary = summarize_interaction_effect_requests(&traces);

        assert!(summary
            .runtime_capabilities
            .contains(&InteractionEffectRequestRuntimeCapabilityCount {
                capability: InteractionEffectRequestRuntimeCapability::EffectControl,
                count: 2,
            }));
        assert!(summary
            .runtime_capabilities
            .contains(&InteractionEffectRequestRuntimeCapabilityCount {
                capability: InteractionEffectRequestRuntimeCapability::WindowLifecycle,
                count: 1,
            }));
    }

    #[test]
    fn summary_preserves_first_appearance_order() {
        let traces = vec![
            effect_trace(
                1,
                InteractionEffectRequestKind::RollbackEffect,
                InteractionEffectRequestUiCapability::TargetQuarantine,
                InteractionEffectRequestRuntimeCapability::TargetQuarantine,
            ),
            effect_trace(
                2,
                InteractionEffectRequestKind::PrepareEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                3,
                InteractionEffectRequestKind::CloseWindow,
                InteractionEffectRequestUiCapability::WindowLifecycle,
                InteractionEffectRequestRuntimeCapability::WindowLifecycle,
            ),
        ];

        let summary = summarize_interaction_effect_requests(&traces);

        assert_eq!(summary.kinds[0].kind, InteractionEffectRequestKind::RollbackEffect);
        assert_eq!(summary.kinds[1].kind, InteractionEffectRequestKind::PrepareEffect);
        assert_eq!(summary.kinds[2].kind, InteractionEffectRequestKind::CloseWindow);
        assert_eq!(
            summary.ui_capabilities[0].capability,
            InteractionEffectRequestUiCapability::TargetQuarantine
        );
        assert_eq!(
            summary.ui_capabilities[1].capability,
            InteractionEffectRequestUiCapability::EffectControl
        );
        assert_eq!(
            summary.ui_capabilities[2].capability,
            InteractionEffectRequestUiCapability::WindowLifecycle
        );
        assert_eq!(
            summary.runtime_capabilities[0].capability,
            InteractionEffectRequestRuntimeCapability::TargetQuarantine
        );
        assert_eq!(
            summary.runtime_capabilities[1].capability,
            InteractionEffectRequestRuntimeCapability::EffectControl
        );
        assert_eq!(
            summary.runtime_capabilities[2].capability,
            InteractionEffectRequestRuntimeCapability::WindowLifecycle
        );
    }

    #[test]
    fn summary_is_not_authority() {
        let traces = vec![effect_trace(
            1,
            InteractionEffectRequestKind::PrepareEffect,
            InteractionEffectRequestUiCapability::EffectControl,
            InteractionEffectRequestRuntimeCapability::EffectControl,
        )];

        let summary = summarize_interaction_effect_requests(&traces);

        assert!(!summary.is_capability_admission());
        assert!(!summary.is_prepared_effect());
        assert!(!summary.is_committed_effect());
        assert!(!summary.is_execution_authority());
        assert!(!summary.is_runtime_mutation());
        assert!(!summary.is_audit_authority());
        assert!(!summary.calls_vm_or_host_abi());
    }

    #[test]
    fn summary_generation_is_deterministic() {
        let traces = vec![
            effect_trace(
                1,
                InteractionEffectRequestKind::PrepareEffect,
                InteractionEffectRequestUiCapability::EffectControl,
                InteractionEffectRequestRuntimeCapability::EffectControl,
            ),
            effect_trace(
                2,
                InteractionEffectRequestKind::CloseWindow,
                InteractionEffectRequestUiCapability::WindowLifecycle,
                InteractionEffectRequestRuntimeCapability::WindowLifecycle,
            ),
        ];

        let first = summarize_interaction_effect_requests(&traces);
        let second = summarize_interaction_effect_requests(&traces);

        assert_eq!(first, second);
    }
}
