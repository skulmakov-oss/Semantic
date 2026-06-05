//! Semantic UI interaction action binding contract draft.
//!
//! This module records inert candidate bindings between interaction intents
//! and future action names. It does not dispatch actions, perform admission,
//! request effects, call the VM, call Host ABI, or mutate runtime state.

#![allow(clippy::too_many_arguments)]

use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionActionBindingId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionName {
    CloseWindow,
    OpenInspector,
    SelectTraceEvent,
    FocusModule,
    ExpandCapabilityGate,
    PrepareEffect,
    CommitEffect,
    RollbackEffect,
    AcknowledgeError,
    OpenDenialReason,
    PinTrace,
    QuarantineTarget,
    Unknown,
}

impl InteractionActionName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CloseWindow => "ui.action.close_window",
            Self::OpenInspector => "ui.action.open_inspector",
            Self::SelectTraceEvent => "ui.action.select_trace_event",
            Self::FocusModule => "ui.action.focus_module",
            Self::ExpandCapabilityGate => "ui.action.expand_capability_gate",
            Self::PrepareEffect => "ui.action.prepare_effect",
            Self::CommitEffect => "ui.action.commit_effect",
            Self::RollbackEffect => "ui.action.rollback_effect",
            Self::AcknowledgeError => "ui.action.acknowledge_error",
            Self::OpenDenialReason => "ui.action.open_denial_reason",
            Self::PinTrace => "ui.action.pin_trace",
            Self::QuarantineTarget => "ui.action.quarantine_target",
            Self::Unknown => "ui.action.unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionBindingScope {
    CoreUi,
    WorkbenchLocal,
    DiagnosticOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionTargetPolicy {
    RequiresTarget,
    AllowsMissingTarget,
    IgnoresTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionPolicy {
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionEffectPolicy {
    NoEffect,
    MayRequestEffect,
    RequiresSeparateEffectAdmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionTracePolicy {
    Required,
    OptionalForDecorativeOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionActionBindingDescriptor {
    pub id: InteractionActionBindingId,
    pub source_intent: InteractionIntentKind,
    pub action: InteractionActionName,
    pub scope: InteractionActionBindingScope,
    pub target_policy: InteractionActionTargetPolicy,
    pub admission_policy: InteractionActionAdmissionPolicy,
    pub effect_policy: InteractionActionEffectPolicy,
    pub trace_policy: InteractionActionTracePolicy,
}

impl InteractionActionBindingDescriptor {
    pub const fn new(
        id: InteractionActionBindingId,
        source_intent: InteractionIntentKind,
        action: InteractionActionName,
        scope: InteractionActionBindingScope,
        target_policy: InteractionActionTargetPolicy,
        admission_policy: InteractionActionAdmissionPolicy,
        effect_policy: InteractionActionEffectPolicy,
        trace_policy: InteractionActionTracePolicy,
    ) -> Self {
        Self {
            id,
            source_intent,
            action,
            scope,
            target_policy,
            admission_policy,
            effect_policy,
            trace_policy,
        }
    }

    pub const fn requires_target(&self) -> bool {
        matches!(
            self.target_policy,
            InteractionActionTargetPolicy::RequiresTarget
        )
    }

    pub const fn requires_admission(&self) -> bool {
        matches!(
            self.admission_policy,
            InteractionActionAdmissionPolicy::Required
        )
    }

    pub const fn may_request_effect(&self) -> bool {
        matches!(
            self.effect_policy,
            InteractionActionEffectPolicy::MayRequestEffect
                | InteractionActionEffectPolicy::RequiresSeparateEffectAdmission
        )
    }

    pub const fn requires_trace(&self) -> bool {
        matches!(self.trace_policy, InteractionActionTracePolicy::Required)
    }
}

pub const INTERACTION_ACTION_BINDINGS: &[InteractionActionBindingDescriptor] = &[
    InteractionActionBindingDescriptor::new(
        InteractionActionBindingId(1),
        InteractionIntentKind::Close,
        InteractionActionName::CloseWindow,
        InteractionActionBindingScope::CoreUi,
        InteractionActionTargetPolicy::IgnoresTarget,
        InteractionActionAdmissionPolicy::Required,
        InteractionActionEffectPolicy::NoEffect,
        InteractionActionTracePolicy::Required,
    ),
    InteractionActionBindingDescriptor::new(
        InteractionActionBindingId(2),
        InteractionIntentKind::Select,
        InteractionActionName::SelectTraceEvent,
        InteractionActionBindingScope::CoreUi,
        InteractionActionTargetPolicy::RequiresTarget,
        InteractionActionAdmissionPolicy::Required,
        InteractionActionEffectPolicy::NoEffect,
        InteractionActionTracePolicy::Required,
    ),
    InteractionActionBindingDescriptor::new(
        InteractionActionBindingId(3),
        InteractionIntentKind::Navigate,
        InteractionActionName::FocusModule,
        InteractionActionBindingScope::CoreUi,
        InteractionActionTargetPolicy::AllowsMissingTarget,
        InteractionActionAdmissionPolicy::Required,
        InteractionActionEffectPolicy::NoEffect,
        InteractionActionTracePolicy::Required,
    ),
    InteractionActionBindingDescriptor::new(
        InteractionActionBindingId(4),
        InteractionIntentKind::Submit,
        InteractionActionName::OpenInspector,
        InteractionActionBindingScope::WorkbenchLocal,
        InteractionActionTargetPolicy::RequiresTarget,
        InteractionActionAdmissionPolicy::Required,
        InteractionActionEffectPolicy::NoEffect,
        InteractionActionTracePolicy::Required,
    ),
    InteractionActionBindingDescriptor::new(
        InteractionActionBindingId(5),
        InteractionIntentKind::Cancel,
        InteractionActionName::AcknowledgeError,
        InteractionActionBindingScope::CoreUi,
        InteractionActionTargetPolicy::AllowsMissingTarget,
        InteractionActionAdmissionPolicy::Required,
        InteractionActionEffectPolicy::NoEffect,
        InteractionActionTracePolicy::Required,
    ),
];

pub const fn interaction_action_bindings() -> &'static [InteractionActionBindingDescriptor] {
    INTERACTION_ACTION_BINDINGS
}

pub fn find_interaction_action_binding_by_action(
    action: InteractionActionName,
) -> Option<&'static InteractionActionBindingDescriptor> {
    INTERACTION_ACTION_BINDINGS
        .iter()
        .find(|binding| binding.action == action)
}

pub fn find_interaction_action_binding_by_intent(
    intent: InteractionIntentKind,
) -> Option<&'static InteractionActionBindingDescriptor> {
    INTERACTION_ACTION_BINDINGS
        .iter()
        .find(|binding| binding.source_intent == intent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_binding_registry_is_not_empty() {
        assert!(!interaction_action_bindings().is_empty());
    }

    #[test]
    fn all_bindings_require_admission() {
        for binding in interaction_action_bindings() {
            assert!(binding.requires_admission());
        }
    }

    #[test]
    fn action_names_have_ui_action_prefix() {
        for binding in interaction_action_bindings() {
            assert!(binding.action.as_str().starts_with("ui.action."));
        }
    }

    #[test]
    fn close_intent_maps_to_close_window_candidate() {
        let binding = find_interaction_action_binding_by_intent(InteractionIntentKind::Close)
            .expect("close binding should exist");

        assert_eq!(binding.action, InteractionActionName::CloseWindow);
        assert_eq!(
            binding.target_policy,
            InteractionActionTargetPolicy::IgnoresTarget
        );
        assert!(binding.requires_trace());
    }

    #[test]
    fn select_trace_event_requires_target() {
        let binding =
            find_interaction_action_binding_by_action(InteractionActionName::SelectTraceEvent)
                .expect("select trace event binding should exist");

        assert_eq!(binding.source_intent, InteractionIntentKind::Select);
        assert!(binding.requires_target());
    }

    #[test]
    fn initial_bindings_do_not_produce_effects() {
        for binding in interaction_action_bindings() {
            assert_eq!(
                binding.effect_policy,
                InteractionActionEffectPolicy::NoEffect
            );
            assert!(!binding.may_request_effect());
        }
    }

    #[test]
    fn binding_ids_are_unique() {
        let bindings = interaction_action_bindings();

        for (index, left) in bindings.iter().enumerate() {
            for right in bindings.iter().skip(index + 1) {
                assert_ne!(left.id, right.id);
            }
        }
    }

    #[test]
    fn unknown_action_is_not_registered() {
        assert!(
            find_interaction_action_binding_by_action(InteractionActionName::Unknown).is_none()
        );
    }
}
