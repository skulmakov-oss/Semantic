//! Bounded ActionIntent-to-admission wiring for the UI-DNA2-10 reference
//! contour (Issue #1543).
//!
//! Reuses the existing repository-owned admission evidence chain --
//! [`prom_ui::action_admission::InteractionActionAdmissionDescriptor`],
//! [`prom_ui::action_admission_result::record_interaction_action_admitted_result`]/
//! `_denied_result`, and
//! [`prom_ui::admitted_action::build_interaction_admitted_semantic_action`]
//! -- rather than introducing a second, demo-only admission system. This
//! module adds only what the bounded reference contour genuinely needs
//! beyond `UiCapabilityEvaluator`'s narrower `evaluate_intent(&SemanticIntent)`
//! signature: revision/epoch staleness and exact-replay rejection, which
//! require invocation-scoped state no existing evaluator trait carries.
//!
//! ```text
//! hit-test result != action authorization
//! ActionIntent candidate != admitted action
//! ```
//!
//! A `SemanticIntent` reaching this module is a candidate only. Admission
//! is the only stage that may produce an `InteractionAdmittedSemanticAction`,
//! and even that action carries no dispatch, execution, or effect
//! authority (`InteractionAdmittedSemanticAction::is_dispatch_authority`
//! etc. all return `false`).
#![allow(dead_code)]

use alloc::collections::BTreeSet;
use core::cell::RefCell;

use prom_ui::action_admission::{
    InteractionActionAdmissionCapabilityRequirement, InteractionActionAdmissionDenialVisibility,
    InteractionActionAdmissionDescriptor, InteractionActionAdmissionDescriptorId,
    InteractionActionAdmissionEffectRelationship, InteractionActionAdmissionFutureResultShape,
    InteractionActionAdmissionLifecycleRequirement, InteractionActionAdmissionPolicyGateNamespace,
    InteractionActionAdmissionTargetOwnershipRequirement,
    InteractionActionAdmissionTargetRequirement, InteractionActionAdmissionTraceRequirement,
};
use prom_ui::action_admission_result::{
    record_interaction_action_admitted_result, record_interaction_action_denied_result,
    InteractionActionAdmissionDenialReason, InteractionActionAdmissionMissingRequirement,
};
use prom_ui::action_binding::{InteractionActionBindingId, InteractionActionName};
use prom_ui::admitted_action::{
    build_interaction_admitted_semantic_action, InteractionAdmittedSemanticAction,
};
use prom_ui::interaction::InteractionIntentKind;
use prom_ui::SemanticIntent;

/// One admission-invocation context for the bounded reference contour: the
/// mapped semantic intent candidate, the revision/epoch it was evaluated
/// against, and a caller-supplied monotonically-unique sequence number
/// used to detect exact replay of a prior invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceActionInvocation {
    pub intent: SemanticIntent,
    pub revision: u64,
    pub epoch: u64,
    pub sequence: u64,
}

/// Denial reasons specific to the bounded reference contour's invocation
/// guard. `MissingCapability` mirrors `IntentCapabilityError::MissingCapability`;
/// the others are invocation-context concerns this contour adds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceAdmissionDenial {
    MissingCapability,
    StaleRevisionOrEpoch,
    ReplayedInvocation,
}

/// Bounded, deterministic capability evaluator and invocation-context guard
/// for the UI-DNA2-10 reference contour. Not a general capability system:
/// `granted_action_ids` is a fixed allow-list of `InteractionActionBindingId`
/// raw values, supplied once at construction from the activated bundle's
/// own declared identities.
#[derive(Debug)]
pub struct ReferenceContourAdmission {
    granted_action_ids: BTreeSet<u64>,
    established_revision_epoch: RefCell<Option<(u64, u64)>>,
    seen_sequences: RefCell<BTreeSet<u64>>,
}

impl ReferenceContourAdmission {
    pub fn new(granted_action_ids: impl IntoIterator<Item = u64>) -> Self {
        Self {
            granted_action_ids: granted_action_ids.into_iter().collect(),
            established_revision_epoch: RefCell::new(None),
            seen_sequences: RefCell::new(BTreeSet::new()),
        }
    }

    /// Evaluates one invocation. Fails closed: no
    /// `InteractionAdmittedSemanticAction` is ever produced for a replayed
    /// sequence, a stale revision/epoch, or an action id outside the
    /// granted set.
    pub fn evaluate(
        &self,
        invocation: &ReferenceActionInvocation,
    ) -> Result<InteractionAdmittedSemanticAction, ReferenceAdmissionDenial> {
        if self.seen_sequences.borrow().contains(&invocation.sequence) {
            return Err(ReferenceAdmissionDenial::ReplayedInvocation);
        }

        {
            let mut established = self.established_revision_epoch.borrow_mut();
            if let Some((rev, epoch)) = *established {
                if (invocation.revision, invocation.epoch) < (rev, epoch) {
                    return Err(ReferenceAdmissionDenial::StaleRevisionOrEpoch);
                }
            }
            *established = Some((invocation.revision, invocation.epoch));
        }

        let descriptor = reference_descriptor(invocation.intent.action_id);
        let result = if self
            .granted_action_ids
            .contains(&invocation.intent.action_id.0)
        {
            record_interaction_action_admitted_result(&descriptor)
        } else {
            record_interaction_action_denied_result(
                &descriptor,
                InteractionActionAdmissionDenialReason::CapabilityMissing,
                InteractionActionAdmissionMissingRequirement::Capability,
            )
        };

        // A sequence is consumed only once a real admit/deny decision has
        // been reached, so it cannot be burned by an earlier-returning
        // guard (replay/staleness) that never reached a decision.
        self.seen_sequences.borrow_mut().insert(invocation.sequence);

        build_interaction_admitted_semantic_action(&result)
            .ok_or(ReferenceAdmissionDenial::MissingCapability)
    }
}

fn reference_descriptor(
    binding_id: InteractionActionBindingId,
) -> InteractionActionAdmissionDescriptor {
    InteractionActionAdmissionDescriptor {
        id: InteractionActionAdmissionDescriptorId(binding_id.0),
        action: InteractionActionName::CommitEffect,
        source_intent: InteractionIntentKind::Activate,
        binding_id,
        target_requirement: InteractionActionAdmissionTargetRequirement::Required,
        target_ownership_requirement:
            InteractionActionAdmissionTargetOwnershipRequirement::RequiredWhenTargetPresent,
        lifecycle_requirement: InteractionActionAdmissionLifecycleRequirement::SessionActive,
        capability_requirement:
            InteractionActionAdmissionCapabilityRequirement::FutureActionSpecific,
        trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
        effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
        denial_visibility: InteractionActionAdmissionDenialVisibility::Required,
        policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
        future_result_shape: InteractionActionAdmissionFutureResultShape::AdmitOrDenyWithReason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prom_ui::projection::UiProjectedNodeId;

    fn invocation(
        action_id: u64,
        revision: u64,
        epoch: u64,
        sequence: u64,
    ) -> ReferenceActionInvocation {
        ReferenceActionInvocation {
            intent: SemanticIntent::new(
                UiProjectedNodeId::new(1),
                InteractionActionBindingId(action_id),
            ),
            revision,
            epoch,
            sequence,
        }
    }

    #[test]
    fn admits_granted_action_id() {
        let admission = ReferenceContourAdmission::new([42]);
        let result = admission.evaluate(&invocation(42, 1, 1, 1));
        assert!(result.is_ok());
        assert!(!result.unwrap().is_dispatch_authority());
    }

    #[test]
    fn denies_ungranted_action_id() {
        let admission = ReferenceContourAdmission::new([42]);
        let result = admission.evaluate(&invocation(999, 1, 1, 1));
        assert_eq!(result, Err(ReferenceAdmissionDenial::MissingCapability));
    }

    #[test]
    fn denies_stale_revision() {
        let admission = ReferenceContourAdmission::new([42]);
        admission
            .evaluate(&invocation(42, 5, 5, 1))
            .expect("first invocation admits");
        let result = admission.evaluate(&invocation(42, 4, 5, 2));
        assert_eq!(result, Err(ReferenceAdmissionDenial::StaleRevisionOrEpoch));
    }

    #[test]
    fn denies_stale_epoch_with_equal_revision() {
        let admission = ReferenceContourAdmission::new([42]);
        admission
            .evaluate(&invocation(42, 5, 5, 1))
            .expect("first invocation admits");
        let result = admission.evaluate(&invocation(42, 5, 4, 2));
        assert_eq!(result, Err(ReferenceAdmissionDenial::StaleRevisionOrEpoch));
    }

    #[test]
    fn admits_equal_or_advancing_revision_epoch() {
        let admission = ReferenceContourAdmission::new([42]);
        admission
            .evaluate(&invocation(42, 5, 5, 1))
            .expect("first invocation admits");
        assert!(admission.evaluate(&invocation(42, 5, 5, 2)).is_ok());
        assert!(admission.evaluate(&invocation(42, 6, 5, 3)).is_ok());
    }

    #[test]
    fn denies_replayed_sequence() {
        let admission = ReferenceContourAdmission::new([42]);
        admission
            .evaluate(&invocation(42, 1, 1, 7))
            .expect("first invocation admits");
        let result = admission.evaluate(&invocation(42, 1, 1, 7));
        assert_eq!(result, Err(ReferenceAdmissionDenial::ReplayedInvocation));
    }

    #[test]
    fn replayed_invocation_does_not_double_admit_idempotently() {
        let admission = ReferenceContourAdmission::new([42]);
        let first = admission.evaluate(&invocation(42, 1, 1, 7));
        let second = admission.evaluate(&invocation(42, 1, 1, 7));
        assert!(first.is_ok());
        assert_eq!(second, Err(ReferenceAdmissionDenial::ReplayedInvocation));
    }

    #[test]
    fn denial_preserves_no_dispatch_or_effect_authority_semantics() {
        // A denied invocation never produces an InteractionAdmittedSemanticAction
        // at all -- there is no partially-admitted object to check.
        let admission = ReferenceContourAdmission::new([42]);
        let result = admission.evaluate(&invocation(999, 1, 1, 1));
        assert!(result.is_err());
    }

    #[test]
    fn evaluation_is_deterministic_for_independent_admission_instances() {
        let admission_a = ReferenceContourAdmission::new([42]);
        let admission_b = ReferenceContourAdmission::new([42]);
        let a = admission_a.evaluate(&invocation(42, 1, 1, 1));
        let b = admission_b.evaluate(&invocation(42, 1, 1, 1));
        assert_eq!(a, b);
    }
}
