//! Semantic UI interaction action binding trace contract scaffold.
//!
//! This module explains whether an interaction intent trace has a matching
//! future action binding candidate. It does not dispatch actions, perform
//! admission, request effects, call the VM, call Host ABI, or mutate runtime
//! state.

use crate::action_binding::{
    find_interaction_action_binding_by_intent, InteractionActionAdmissionPolicy,
    InteractionActionBindingDescriptor, InteractionActionBindingId,
    InteractionActionEffectPolicy, InteractionActionName, InteractionActionTargetPolicy,
    InteractionActionTracePolicy,
};
use crate::interaction::{InteractionIntentId, InteractionIntentKind};
use crate::trace::{
    InteractionIntentTraceReport, InteractionIntentTraceStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionBindingTraceStatus {
    Bound,
    Unbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionBindingTraceReason {
    BindingFound,
    IntentUnclassified,
    NoBindingForIntent,
    BindingRequiresTarget,
    BindingAllowsMissingTarget,
    BindingIgnoresTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionActionBindingTraceReport {
    pub intent_id: InteractionIntentId,
    pub intent_kind: InteractionIntentKind,
    pub intent_status: InteractionIntentTraceStatus,
    pub binding_status: InteractionActionBindingTraceStatus,
    pub reason: InteractionActionBindingTraceReason,
    pub binding_id: Option<InteractionActionBindingId>,
    pub action: Option<InteractionActionName>,
    pub target_policy: Option<InteractionActionTargetPolicy>,
    pub admission_policy: Option<InteractionActionAdmissionPolicy>,
    pub effect_policy: Option<InteractionActionEffectPolicy>,
    pub trace_policy: Option<InteractionActionTracePolicy>,
}

pub fn trace_interaction_action_binding(
    intent_trace: &InteractionIntentTraceReport,
) -> InteractionActionBindingTraceReport {
    if matches!(intent_trace.status, InteractionIntentTraceStatus::Unclassified) {
        return unbound_report(intent_trace, InteractionActionBindingTraceReason::IntentUnclassified);
    }

    match find_interaction_action_binding_by_intent(intent_trace.intent_kind) {
        Some(binding) => bound_report(intent_trace, binding),
        None => unbound_report(intent_trace, InteractionActionBindingTraceReason::NoBindingForIntent),
    }
}

fn bound_report(
    intent_trace: &InteractionIntentTraceReport,
    binding: &InteractionActionBindingDescriptor,
) -> InteractionActionBindingTraceReport {
    InteractionActionBindingTraceReport {
        intent_id: intent_trace.intent_id,
        intent_kind: intent_trace.intent_kind,
        intent_status: intent_trace.status,
        binding_status: InteractionActionBindingTraceStatus::Bound,
        reason: reason_for_binding(binding),
        binding_id: Some(binding.id),
        action: Some(binding.action),
        target_policy: Some(binding.target_policy),
        admission_policy: Some(binding.admission_policy),
        effect_policy: Some(binding.effect_policy),
        trace_policy: Some(binding.trace_policy),
    }
}

fn unbound_report(
    intent_trace: &InteractionIntentTraceReport,
    reason: InteractionActionBindingTraceReason,
) -> InteractionActionBindingTraceReport {
    InteractionActionBindingTraceReport {
        intent_id: intent_trace.intent_id,
        intent_kind: intent_trace.intent_kind,
        intent_status: intent_trace.status,
        binding_status: InteractionActionBindingTraceStatus::Unbound,
        reason,
        binding_id: None,
        action: None,
        target_policy: None,
        admission_policy: None,
        effect_policy: None,
        trace_policy: None,
    }
}

fn reason_for_binding(
    binding: &InteractionActionBindingDescriptor,
) -> InteractionActionBindingTraceReason {
    match binding.target_policy {
        InteractionActionTargetPolicy::RequiresTarget => {
            InteractionActionBindingTraceReason::BindingRequiresTarget
        }
        InteractionActionTargetPolicy::AllowsMissingTarget => {
            InteractionActionBindingTraceReason::BindingAllowsMissingTarget
        }
        InteractionActionTargetPolicy::IgnoresTarget => {
            InteractionActionBindingTraceReason::BindingIgnoresTarget
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::{ElementId, InteractionSource, InteractionTarget};
    use crate::raw_event::{RawUiEventId, RawUiEventKind};
    use crate::trace::{
        InteractionIntentMappingRule, InteractionIntentTraceReason, InteractionIntentTraceReport,
        InteractionIntentTraceStatus,
    };

    #[test]
    fn classified_close_intent_binds_to_close_window() {
        let intent_trace = InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(1),
            intent_id: InteractionIntentId(1),
            raw_kind: RawUiEventKind::WindowCloseRequested,
            intent_kind: InteractionIntentKind::Close,
            source: InteractionSource::Window,
            target: None,
            status: InteractionIntentTraceStatus::Classified,
            reason: InteractionIntentTraceReason::WindowMapping,
            rule: InteractionIntentMappingRule::WindowClose,
        };

        let report = trace_interaction_action_binding(&intent_trace);

        assert_eq!(report.binding_status, InteractionActionBindingTraceStatus::Bound);
        assert_eq!(report.action, Some(InteractionActionName::CloseWindow));
        assert_eq!(
            report.reason,
            InteractionActionBindingTraceReason::BindingIgnoresTarget
        );
    }

    #[test]
    fn classified_select_intent_binds_and_requires_target() {
        let intent_trace = InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(2),
            intent_id: InteractionIntentId(2),
            raw_kind: RawUiEventKind::PointerDown,
            intent_kind: InteractionIntentKind::Select,
            source: InteractionSource::Pointer,
            target: Some(InteractionTarget::Element(ElementId(7))),
            status: InteractionIntentTraceStatus::Classified,
            reason: InteractionIntentTraceReason::DirectMapping,
            rule: InteractionIntentMappingRule::PointerDownActivate,
        };

        let report = trace_interaction_action_binding(&intent_trace);

        assert_eq!(report.binding_status, InteractionActionBindingTraceStatus::Bound);
        assert_eq!(report.action, Some(InteractionActionName::SelectTraceEvent));
        assert_eq!(
            report.target_policy,
            Some(InteractionActionTargetPolicy::RequiresTarget)
        );
        assert_eq!(
            report.reason,
            InteractionActionBindingTraceReason::BindingRequiresTarget
        );
    }

    #[test]
    fn unclassified_intent_does_not_bind() {
        let intent_trace = InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(3),
            intent_id: InteractionIntentId(3),
            raw_kind: RawUiEventKind::Unknown,
            intent_kind: InteractionIntentKind::Unknown,
            source: InteractionSource::Unknown,
            target: None,
            status: InteractionIntentTraceStatus::Unclassified,
            reason: InteractionIntentTraceReason::UnknownRawEvent,
            rule: InteractionIntentMappingRule::Unknown,
        };

        let report = trace_interaction_action_binding(&intent_trace);

        assert_eq!(report.binding_status, InteractionActionBindingTraceStatus::Unbound);
        assert_eq!(
            report.reason,
            InteractionActionBindingTraceReason::IntentUnclassified
        );
        assert_eq!(report.action, None);
        assert_eq!(report.binding_id, None);
    }

    #[test]
    fn classified_intent_without_binding_is_unbound() {
        let intent_trace = InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(4),
            intent_id: InteractionIntentId(4),
            raw_kind: RawUiEventKind::PointerDown,
            intent_kind: InteractionIntentKind::Activate,
            source: InteractionSource::Pointer,
            target: Some(InteractionTarget::Element(ElementId(1))),
            status: InteractionIntentTraceStatus::Classified,
            reason: InteractionIntentTraceReason::DirectMapping,
            rule: InteractionIntentMappingRule::PointerDownActivate,
        };

        let report = trace_interaction_action_binding(&intent_trace);

        assert_eq!(report.binding_status, InteractionActionBindingTraceStatus::Unbound);
        assert_eq!(
            report.reason,
            InteractionActionBindingTraceReason::NoBindingForIntent
        );
        assert_eq!(report.action, None);
    }

    #[test]
    fn bound_report_preserves_binding_policies() {
        let intent_trace = InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(5),
            intent_id: InteractionIntentId(5),
            raw_kind: RawUiEventKind::WindowCloseRequested,
            intent_kind: InteractionIntentKind::Close,
            source: InteractionSource::Window,
            target: None,
            status: InteractionIntentTraceStatus::Classified,
            reason: InteractionIntentTraceReason::WindowMapping,
            rule: InteractionIntentMappingRule::WindowClose,
        };

        let report = trace_interaction_action_binding(&intent_trace);

        assert_eq!(
            report.admission_policy,
            Some(InteractionActionAdmissionPolicy::Required)
        );
        assert_eq!(
            report.effect_policy,
            Some(InteractionActionEffectPolicy::NoEffect)
        );
        assert_eq!(
            report.trace_policy,
            Some(InteractionActionTracePolicy::Required)
        );
    }
}
