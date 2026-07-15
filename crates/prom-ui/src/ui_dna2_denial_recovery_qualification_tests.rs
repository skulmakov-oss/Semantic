//! UI-DNA2 denial, recovery, batch, and freshness v0 qualification.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::binding_graph::{BindingSlot, BindingTarget};
use crate::connectivity_projection::{
    project_freshness_fragment, ControlCriticality, FreshnessControlRoute,
    FreshnessProjectionDiagnosticKind, FreshnessProjectionLimits, FreshnessProjectionRoute,
    FreshnessState, ProjectedControlAvailability,
};
use crate::contract_primitives::{
    CollectionKey, Epoch, Revision, StaticDocumentId, StaticNodeId, StaticSurfaceId,
};
use crate::denial_recovery::{
    project_denial_recovery, BatchMode, BatchProjection, BatchProjectionItem,
    DenialRecoveryEvidence, DenialRecoveryLimitKind, DenialRecoveryLimits,
    DenialRecoveryProjectionDiagnostic, DenialRecoveryProjectionDiagnosticKind,
    DenialRecoveryProjectionFailure, DenialRecoveryProjectionStage, DenialRecoveryRoute,
    ProjectionResultCategory, RecoveryKind, RecoveryOffer,
};
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatchEnvelope, ProjectionPatchId,
    ProjectionPatchOperation, ProjectionPatchSequence, ProjectionPatchStreamId,
    ProjectionPatchValue,
};
use crate::semantic_refs::{ReferenceToken, SemanticActionRef, SemanticEvidenceRef};

fn node(raw: u64) -> StaticNodeId {
    StaticNodeId::new(raw).expect("node")
}

fn key(raw: u64) -> CollectionKey {
    CollectionKey::new(raw).expect("key")
}

fn target(node_raw: u64) -> BindingTarget {
    BindingTarget {
        node: node(node_raw),
        slot: BindingSlot(1),
    }
}

fn envelope() -> ProjectionPatchEnvelope {
    ProjectionPatchEnvelope::new(
        ProjectionPatchId::new(1).expect("patch"),
        ProjectionPatchStreamId::new(1).expect("stream"),
        StaticDocumentId::new(1).expect("document"),
        Some(StaticSurfaceId::new(1).expect("surface")),
        Revision::new(0),
        Revision::new(1),
        Epoch::new(1),
        ProjectionPatchSequence::new(0),
    )
}

fn controls() -> Vec<FreshnessControlRoute> {
    vec![FreshnessControlRoute::new(
        node(10),
        ControlCriticality::Normal,
        ProjectedControlAvailability::Available,
    )]
}

fn route_with_controls(controls: Vec<FreshnessControlRoute>) -> DenialRecoveryRoute {
    DenialRecoveryRoute::new(
        target(1),
        target(2),
        Some(node(20)),
        Some(node(30)),
        FreshnessProjectionRoute::new(target(3), controls),
    )
}

fn route() -> DenialRecoveryRoute {
    route_with_controls(controls())
}

fn evidence(result: ProjectionResultCategory, freshness: FreshnessState) -> DenialRecoveryEvidence {
    DenialRecoveryEvidence::new(
        envelope(),
        result,
        if result == ProjectionResultCategory::Accepted {
            String::new()
        } else {
            "reason".to_string()
        },
        Some(SemanticActionRef::new(7)),
        Some(SemanticEvidenceRef::new(8)),
        freshness,
        false,
        None,
        Vec::new(),
    )
}

fn limits() -> DenialRecoveryLimits {
    DenialRecoveryLimits::new(256, 32, 32, 32, 128, 4096)
}

fn action_ref() -> SemanticActionRef {
    SemanticActionRef::new(11)
}

fn resume_token() -> ReferenceToken {
    ReferenceToken::new(1, 2, 3, 4)
}

fn batch_item(order: u32, key_raw: u64, result: ProjectionResultCategory) -> BatchProjectionItem {
    BatchProjectionItem::new(order, key(key_raw), result)
}

fn recovery_offer(order: u32, key_raw: u64, kind: RecoveryKind) -> RecoveryOffer {
    RecoveryOffer::new(
        order,
        key(key_raw),
        kind,
        (!matches!(kind, RecoveryKind::Dismiss)).then_some(action_ref()),
        (kind == RecoveryKind::Resume).then_some(resume_token()),
    )
}

fn diagnostics(
    result: Result<
        crate::denial_recovery::DenialRecoveryProjectionArtifact,
        DenialRecoveryProjectionFailure,
    >,
) -> Vec<DenialRecoveryProjectionDiagnostic> {
    match result.expect_err("expected rejection") {
        DenialRecoveryProjectionFailure::Diagnostics(diagnostics) => diagnostics,
        other => panic!("expected DRP diagnostics, got {other:?}"),
    }
}

#[test]
fn ui_dna2_denial_recovery_result_taxonomy_has_exact_distinct_tokens() {
    let cases = [
        (ProjectionResultCategory::Accepted, "accepted"),
        (ProjectionResultCategory::Denied, "denied"),
        (ProjectionResultCategory::LocalDenied, "local_denied"),
        (
            ProjectionResultCategory::AdmissionDenied,
            "admission_denied",
        ),
        (
            ProjectionResultCategory::CapabilityRejected,
            "capability_rejected",
        ),
        (ProjectionResultCategory::StaleRejected, "stale_rejected"),
        (
            ProjectionResultCategory::FreshnessRejected,
            "freshness_rejected",
        ),
        (ProjectionResultCategory::PartialDenied, "partial_denied"),
        (ProjectionResultCategory::NotApplied, "not_applied"),
        (ProjectionResultCategory::BatchBreak, "batch_break"),
        (ProjectionResultCategory::PendingUnknown, "pending_unknown"),
        (ProjectionResultCategory::Quarantined, "quarantined"),
    ];
    let tokens: Vec<_> = cases
        .iter()
        .map(|(category, expected)| {
            assert_eq!(category.token(), *expected);
            category.token()
        })
        .collect();
    let mut unique = tokens.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), tokens.len());
}

#[test]
fn ui_dna2_denial_recovery_reason_and_evidence_requirements_are_explicit() {
    assert!(project_denial_recovery(
        &route(),
        &evidence(ProjectionResultCategory::Accepted, FreshnessState::Fresh),
        limits(),
    )
    .is_ok());

    let mut missing_reason = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    missing_reason.reason.clear();
    let errors = diagnostics(project_denial_recovery(&route(), &missing_reason, limits()));
    assert_eq!(errors[0].code(), "DRP_MISSING_REASON");

    for category in [
        ProjectionResultCategory::AdmissionDenied,
        ProjectionResultCategory::CapabilityRejected,
        ProjectionResultCategory::Quarantined,
    ] {
        let mut missing_evidence = evidence(category, FreshnessState::Fresh);
        missing_evidence.evidence_ref = None;
        let errors = diagnostics(project_denial_recovery(
            &route(),
            &missing_evidence,
            limits(),
        ));
        assert!(errors
            .iter()
            .any(|error| error.code() == "DRP_MISSING_EVIDENCE_REF"));
    }
}

#[test]
fn ui_dna2_denial_recovery_freshness_tokens_cover_all_states() {
    let cases = [
        (FreshnessState::Fresh, "fresh"),
        (FreshnessState::Degraded, "degraded"),
        (FreshnessState::Stale, "stale"),
        (FreshnessState::Offline, "offline"),
        (FreshnessState::Resyncing, "resyncing"),
    ];
    for (state, token) in cases {
        let fragment = project_freshness_fragment(
            state,
            &FreshnessProjectionRoute::new(target(3), Vec::new()),
            FreshnessProjectionLimits::new(0, 1),
        )
        .expect("valid fragment");
        assert_eq!(fragment.state(), state);
        assert!(matches!(
            &fragment.operations()[0],
            ProjectionPatchOperation::SetBindingValue {
                value: ProjectionPatchValue::Text(value), ..
            } if value == token
        ));
    }
}

#[test]
fn ui_dna2_denial_recovery_freshness_rejects_duplicate_controls_canonically() {
    let route = FreshnessProjectionRoute::new(
        target(3),
        vec![
            FreshnessControlRoute::new(
                node(12),
                ControlCriticality::Normal,
                ProjectedControlAvailability::Available,
            ),
            FreshnessControlRoute::new(
                node(12),
                ControlCriticality::Normal,
                ProjectedControlAvailability::Unavailable,
            ),
        ],
    );
    let errors = project_freshness_fragment(
        FreshnessState::Fresh,
        &route,
        FreshnessProjectionLimits::new(2, 3),
    )
    .expect_err("duplicate");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code(), "CFP_DUPLICATE_CONTROL_NODE");
    assert_eq!(errors[0].node(), Some(node(12)));
}

#[test]
fn ui_dna2_denial_recovery_stale_states_reject_available_critical_controls() {
    for state in [
        FreshnessState::Stale,
        FreshnessState::Offline,
        FreshnessState::Resyncing,
    ] {
        for criticality in [
            ControlCriticality::Guarded,
            ControlCriticality::Danger,
            ControlCriticality::TaskControl,
        ] {
            let route = FreshnessProjectionRoute::new(
                target(3),
                vec![FreshnessControlRoute::new(
                    node(10),
                    criticality,
                    ProjectedControlAvailability::Available,
                )],
            );
            let errors =
                project_freshness_fragment(state, &route, FreshnessProjectionLimits::new(1, 2))
                    .expect_err("critical availability");
            assert_eq!(
                errors[0].kind(),
                FreshnessProjectionDiagnosticKind::InvalidCriticalControlAvailability
            );
        }
    }
}

#[test]
fn ui_dna2_denial_recovery_normal_controls_remain_caller_supplied() {
    let fragment = project_freshness_fragment(
        FreshnessState::Offline,
        &FreshnessProjectionRoute::new(target(3), controls()),
        FreshnessProjectionLimits::new(1, 2),
    )
    .expect("normal control remains available");
    assert!(matches!(
        fragment.operations()[1],
        ProjectionPatchOperation::SetNodeAvailability {
            availability: ProjectionNodeAvailability::Available,
            ..
        }
    ));
}

#[test]
fn ui_dna2_denial_recovery_critical_stale_attempt_requires_local_denial() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Stale);
    supplied.critical_attempt = true;
    let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
    assert_eq!(
        errors[0].stage(),
        DenialRecoveryProjectionStage::FreshnessValidation
    );
    assert_eq!(errors[0].code(), "DRP_INVALID_FRESHNESS_OUTCOME");

    supplied.result = ProjectionResultCategory::LocalDenied;
    assert!(project_denial_recovery(&route(), &supplied, limits()).is_ok());
}

#[test]
fn ui_dna2_denial_recovery_recovery_taxonomy_has_exact_tokens() {
    let cases = [
        (RecoveryKind::Dismiss, "dismiss"),
        (RecoveryKind::Acknowledge, "acknowledge"),
        (RecoveryKind::Retry, "retry"),
        (RecoveryKind::Resume, "resume"),
        (RecoveryKind::CancelSuffix, "cancel_suffix"),
    ];
    for (kind, token) in cases {
        assert_eq!(kind.token(), token);
    }
}

#[test]
fn ui_dna2_denial_recovery_action_and_resume_references_are_required() {
    for kind in [
        RecoveryKind::Acknowledge,
        RecoveryKind::Retry,
        RecoveryKind::Resume,
        RecoveryKind::CancelSuffix,
    ] {
        let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
        supplied.recovery_offers = vec![RecoveryOffer::new(1, key(1), kind, None, None)];
        let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
        assert!(errors
            .iter()
            .any(|error| error.code() == "DRP_RECOVERY_ACTION_REF_MISSING"));
        if kind == RecoveryKind::Resume {
            assert!(errors
                .iter()
                .any(|error| error.code() == "DRP_RESUME_TOKEN_MISSING"));
        }
    }
}

#[test]
fn ui_dna2_denial_recovery_recovery_duplicates_are_deterministic() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.recovery_offers = vec![
        recovery_offer(2, 2, RecoveryKind::Retry),
        recovery_offer(2, 1, RecoveryKind::Acknowledge),
        recovery_offer(3, 1, RecoveryKind::CancelSuffix),
    ];
    let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
    assert_eq!(
        errors.iter().map(|error| error.code()).collect::<Vec<_>>(),
        vec!["DRP_DUPLICATE_RECOVERY_ORDER", "DRP_DUPLICATE_RECOVERY_KEY"]
    );
}

#[test]
fn ui_dna2_denial_recovery_nonempty_recovery_requires_route() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.recovery_offers = vec![recovery_offer(1, 1, RecoveryKind::Dismiss)];
    let no_recovery_route = DenialRecoveryRoute::new(
        target(1),
        target(2),
        Some(node(20)),
        None,
        FreshnessProjectionRoute::new(target(3), controls()),
    );
    let errors = diagnostics(project_denial_recovery(
        &no_recovery_route,
        &supplied,
        limits(),
    ));
    assert_eq!(
        errors[0].stage(),
        DenialRecoveryProjectionStage::RouteValidation
    );
    assert_eq!(errors[0].code(), "DRP_MISSING_RECOVERY_ROUTE");
}

#[test]
fn ui_dna2_denial_recovery_recovery_order_is_canonical_and_inert() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.recovery_offers = vec![
        recovery_offer(3, 3, RecoveryKind::Retry),
        recovery_offer(1, 1, RecoveryKind::Dismiss),
        recovery_offer(2, 2, RecoveryKind::Acknowledge),
    ];
    let artifact = project_denial_recovery(&route(), &supplied, limits()).expect("projection");
    assert_eq!(
        artifact
            .recovery_offers()
            .iter()
            .map(|offer| offer.kind())
            .collect::<Vec<_>>(),
        vec![
            RecoveryKind::Dismiss,
            RecoveryKind::Acknowledge,
            RecoveryKind::Retry
        ]
    );
    assert_eq!(artifact.action_ref(), supplied.action_ref);
    assert_eq!(artifact.evidence_ref(), supplied.evidence_ref);
}

#[test]
fn ui_dna2_denial_recovery_atomic_acceptance_is_all_accepted() {
    let mut supplied = evidence(ProjectionResultCategory::Accepted, FreshnessState::Fresh);
    supplied.batch = Some(BatchProjection::new(
        BatchMode::Atomic,
        None,
        vec![
            batch_item(2, 2, ProjectionResultCategory::Accepted),
            batch_item(1, 1, ProjectionResultCategory::Accepted),
        ],
    ));
    let artifact = project_denial_recovery(&route(), &supplied, limits()).expect("atomic success");
    assert_eq!(artifact.batch().expect("batch").items()[0].order(), 1);
}

#[test]
fn ui_dna2_denial_recovery_atomic_denial_has_no_accepted_claim() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.batch = Some(BatchProjection::new(
        BatchMode::Atomic,
        None,
        vec![
            batch_item(1, 1, ProjectionResultCategory::Denied),
            batch_item(2, 2, ProjectionResultCategory::NotApplied),
        ],
    ));
    assert!(project_denial_recovery(&route(), &supplied, limits()).is_ok());
}

#[test]
fn ui_dna2_denial_recovery_atomic_mixed_acceptance_is_invalid() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.batch = Some(BatchProjection::new(
        BatchMode::Atomic,
        None,
        vec![
            batch_item(1, 1, ProjectionResultCategory::Accepted),
            batch_item(2, 2, ProjectionResultCategory::Denied),
        ],
    ));
    let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
    assert_eq!(errors[0].code(), "DRP_INVALID_ATOMIC_BATCH");
}

#[test]
fn ui_dna2_denial_recovery_ordered_partial_accepts_exact_prefix_break_suffix() {
    let mut supplied = evidence(
        ProjectionResultCategory::PartialDenied,
        FreshnessState::Fresh,
    );
    supplied.batch = Some(BatchProjection::new(
        BatchMode::OrderedPartial,
        Some(2),
        vec![
            batch_item(3, 3, ProjectionResultCategory::NotApplied),
            batch_item(1, 1, ProjectionResultCategory::Accepted),
            batch_item(2, 2, ProjectionResultCategory::Denied),
        ],
    ));
    let artifact = project_denial_recovery(&route(), &supplied, limits()).expect("partial batch");
    assert_eq!(
        artifact
            .batch()
            .expect("batch")
            .items()
            .iter()
            .map(|item| item.result())
            .collect::<Vec<_>>(),
        vec![
            ProjectionResultCategory::Accepted,
            ProjectionResultCategory::Denied,
            ProjectionResultCategory::NotApplied,
        ]
    );
}

#[test]
fn ui_dna2_denial_recovery_ordered_partial_rejects_missing_or_multiple_breaks() {
    for items in [
        vec![batch_item(1, 1, ProjectionResultCategory::Accepted)],
        vec![
            batch_item(1, 1, ProjectionResultCategory::Denied),
            batch_item(2, 2, ProjectionResultCategory::LocalDenied),
        ],
    ] {
        let mut supplied = evidence(
            ProjectionResultCategory::PartialDenied,
            FreshnessState::Fresh,
        );
        supplied.batch = Some(BatchProjection::new(
            BatchMode::OrderedPartial,
            Some(1),
            items,
        ));
        let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
        assert_eq!(errors[0].code(), "DRP_INVALID_ORDERED_PARTIAL_BATCH");
    }
}

#[test]
fn ui_dna2_denial_recovery_ordered_partial_rejects_not_applied_prefix() {
    let mut supplied = evidence(
        ProjectionResultCategory::PartialDenied,
        FreshnessState::Fresh,
    );
    supplied.batch = Some(BatchProjection::new(
        BatchMode::OrderedPartial,
        Some(2),
        vec![
            batch_item(1, 1, ProjectionResultCategory::NotApplied),
            batch_item(2, 2, ProjectionResultCategory::Denied),
        ],
    ));
    let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
    assert_eq!(errors[0].code(), "DRP_INVALID_ORDERED_PARTIAL_BATCH");
}

#[test]
fn ui_dna2_denial_recovery_break_order_must_match_denial_item() {
    let mut supplied = evidence(ProjectionResultCategory::BatchBreak, FreshnessState::Fresh);
    supplied.batch = Some(BatchProjection::new(
        BatchMode::OrderedPartial,
        Some(9),
        vec![batch_item(2, 2, ProjectionResultCategory::Denied)],
    ));
    let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
    assert_eq!(errors[0].code(), "DRP_BATCH_BREAK_MISMATCH");
    assert_eq!(errors[0].primary(), 9);
    assert_eq!(errors[0].secondary(), 2);
}

#[test]
fn ui_dna2_denial_recovery_batch_duplicate_order_and_key_are_canonical() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.batch = Some(BatchProjection::new(
        BatchMode::Atomic,
        None,
        vec![
            batch_item(1, 2, ProjectionResultCategory::Denied),
            batch_item(1, 1, ProjectionResultCategory::Denied),
            batch_item(2, 1, ProjectionResultCategory::NotApplied),
        ],
    ));
    let errors = diagnostics(project_denial_recovery(&route(), &supplied, limits()));
    assert_eq!(
        errors.iter().map(|error| error.code()).collect::<Vec<_>>(),
        vec!["DRP_DUPLICATE_BATCH_ORDER", "DRP_DUPLICATE_BATCH_KEY"]
    );
}

#[test]
fn ui_dna2_denial_recovery_batch_storage_permutation_is_invariant() {
    let items = vec![
        batch_item(1, 1, ProjectionResultCategory::Accepted),
        batch_item(2, 2, ProjectionResultCategory::Denied),
        batch_item(3, 3, ProjectionResultCategory::NotApplied),
    ];
    let mut a = evidence(
        ProjectionResultCategory::PartialDenied,
        FreshnessState::Fresh,
    );
    a.batch = Some(BatchProjection::new(
        BatchMode::OrderedPartial,
        Some(2),
        items.clone(),
    ));
    let mut b = evidence(
        ProjectionResultCategory::PartialDenied,
        FreshnessState::Fresh,
    );
    b.batch = Some(BatchProjection::new(
        BatchMode::OrderedPartial,
        Some(2),
        vec![items[2].clone(), items[0].clone(), items[1].clone()],
    ));
    let a = project_denial_recovery(&route(), &a, limits()).expect("a");
    let b = project_denial_recovery(&route(), &b, limits()).expect("b");
    assert_eq!(a.patch().operations(), b.patch().operations());
}

#[test]
fn ui_dna2_denial_recovery_patch_mapping_preserves_envelope_reason_and_order() {
    let control_routes = vec![
        FreshnessControlRoute::new(
            node(12),
            ControlCriticality::Guarded,
            ProjectedControlAvailability::Unavailable,
        ),
        FreshnessControlRoute::new(
            node(11),
            ControlCriticality::Normal,
            ProjectedControlAvailability::Available,
        ),
    ];
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.reason = " exact reason \n".to_string();
    supplied.batch = Some(BatchProjection::new(
        BatchMode::Atomic,
        None,
        vec![batch_item(1, 41, ProjectionResultCategory::Denied)],
    ));
    supplied.recovery_offers = vec![recovery_offer(1, 51, RecoveryKind::Dismiss)];
    let artifact =
        project_denial_recovery(&route_with_controls(control_routes), &supplied, limits())
            .expect("projection");
    assert_eq!(artifact.patch().envelope(), supplied.envelope);
    assert_eq!(artifact.operation_count(), 7);
    let operations = artifact.patch().operations();
    assert!(matches!(
        &operations[0],
        ProjectionPatchOperation::SetBindingValue { value: ProjectionPatchValue::Text(value), .. }
            if value == "denied"
    ));
    assert!(matches!(
        &operations[1],
        ProjectionPatchOperation::SetBindingValue { value: ProjectionPatchValue::Text(value), .. }
            if value == " exact reason \n"
    ));
    assert!(matches!(
        &operations[2],
        ProjectionPatchOperation::SetBindingValue { value: ProjectionPatchValue::Text(value), .. }
            if value == "fresh"
    ));
    assert!(matches!(
        operations[3],
        ProjectionPatchOperation::CollectionInsert { key: item_key, .. } if item_key == key(41)
    ));
    assert!(matches!(
        operations[4],
        ProjectionPatchOperation::SetNodeAvailability { node: control, .. } if control == node(11)
    ));
    assert!(matches!(
        operations[5],
        ProjectionPatchOperation::SetNodeAvailability { node: control, .. } if control == node(12)
    ));
    assert!(matches!(
        operations[6],
        ProjectionPatchOperation::CollectionInsert { key: offer_key, .. } if offer_key == key(51)
    ));
}

#[test]
fn ui_dna2_denial_recovery_original_patch_diagnostics_are_retained() {
    let duplicate_target_route = DenialRecoveryRoute::new(
        target(1),
        target(1),
        Some(node(20)),
        Some(node(30)),
        FreshnessProjectionRoute::new(target(3), Vec::new()),
    );
    let supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    match project_denial_recovery(&duplicate_target_route, &supplied, limits())
        .expect_err("patch rejection")
    {
        DenialRecoveryProjectionFailure::PatchRejected {
            diagnostic,
            patch_diagnostics,
        } => {
            assert_eq!(diagnostic.code(), "DRP_PATCH_REJECTED");
            assert_eq!(
                diagnostic.stage(),
                DenialRecoveryProjectionStage::PatchValidation
            );
            assert_eq!(
                patch_diagnostics[0].coordinate().code().as_str(),
                "PP_DUP_MUTATION_TARGET"
            );
        }
        other => panic!("unexpected failure {other:?}"),
    }
}

#[test]
fn ui_dna2_denial_recovery_every_preflight_limit_is_qualified() {
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    supplied.batch = Some(BatchProjection::new(
        BatchMode::Atomic,
        None,
        vec![batch_item(1, 1, ProjectionResultCategory::Denied)],
    ));
    supplied.recovery_offers = vec![recovery_offer(1, 2, RecoveryKind::Dismiss)];
    let errors = diagnostics(project_denial_recovery(
        &route(),
        &supplied,
        DenialRecoveryLimits::new(0, 0, 0, 0, 128, 0),
    ));
    assert_eq!(errors.len(), 5);
    assert_eq!(
        errors
            .iter()
            .map(|error| error.limit_kind().expect("limit"))
            .collect::<Vec<_>>(),
        vec![
            DenialRecoveryLimitKind::ReasonBytes,
            DenialRecoveryLimitKind::BatchItems,
            DenialRecoveryLimitKind::RecoveryOffers,
            DenialRecoveryLimitKind::ControlRoutes,
            DenialRecoveryLimitKind::TotalProjectedTextBytes,
        ]
    );
    assert!(errors
        .iter()
        .all(|error| error.actual().expect("actual") > 0));
    assert!(errors.iter().all(|error| error.maximum() == Some(0)));
}

#[test]
fn ui_dna2_denial_recovery_operation_limit_reports_first_exceeding_count() {
    let supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Fresh);
    let errors = diagnostics(project_denial_recovery(
        &route(),
        &supplied,
        DenialRecoveryLimits::new(256, 32, 32, 32, 3, 4096),
    ));
    assert_eq!(errors[0].code(), "DRP_OPERATION_LIMIT_EXCEEDED");
    assert_eq!(
        errors[0].stage(),
        DenialRecoveryProjectionStage::OperationConstruction
    );
    assert_eq!(errors[0].actual(), Some(4));
    assert_eq!(errors[0].maximum(), Some(3));
}

#[test]
fn ui_dna2_denial_recovery_freshness_limits_are_bounded() {
    let route = FreshnessProjectionRoute::new(target(3), controls());
    let resource = project_freshness_fragment(
        FreshnessState::Fresh,
        &route,
        FreshnessProjectionLimits::new(0, 10),
    )
    .expect_err("control limit");
    assert_eq!(resource[0].code(), "CFP_RESOURCE_LIMIT_EXCEEDED");
    assert_eq!(resource[0].actual(), Some(1));
    assert_eq!(resource[0].maximum(), Some(0));

    let operations = project_freshness_fragment(
        FreshnessState::Fresh,
        &route,
        FreshnessProjectionLimits::new(1, 1),
    )
    .expect_err("operation limit");
    assert_eq!(operations[0].code(), "CFP_OPERATION_LIMIT_EXCEEDED");
    assert_eq!(operations[0].actual(), Some(2));
}

#[test]
fn ui_dna2_denial_recovery_stage_precedence_is_resource_then_freshness_then_outcome() {
    let invalid_controls = vec![FreshnessControlRoute::new(
        node(10),
        ControlCriticality::Danger,
        ProjectedControlAvailability::Available,
    )];
    let mut supplied = evidence(ProjectionResultCategory::Denied, FreshnessState::Stale);
    supplied.reason.clear();
    supplied.critical_attempt = true;

    let resource = diagnostics(project_denial_recovery(
        &route_with_controls(invalid_controls.clone()),
        &supplied,
        DenialRecoveryLimits::new(0, 32, 32, 0, 128, 4096),
    ));
    assert!(resource
        .iter()
        .all(|error| error.stage() == DenialRecoveryProjectionStage::ResourcePreflight));

    match project_denial_recovery(&route_with_controls(invalid_controls), &supplied, limits())
        .expect_err("freshness before outcome")
    {
        DenialRecoveryProjectionFailure::Freshness(errors) => assert_eq!(
            errors[0].code(),
            "CFP_INVALID_CRITICAL_CONTROL_AVAILABILITY"
        ),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn ui_dna2_denial_recovery_batch_precedes_recovery_and_operation_validation() {
    let mut supplied = evidence(
        ProjectionResultCategory::PartialDenied,
        FreshnessState::Fresh,
    );
    supplied.batch = Some(BatchProjection::new(
        BatchMode::OrderedPartial,
        Some(1),
        vec![batch_item(1, 1, ProjectionResultCategory::Accepted)],
    ));
    supplied.recovery_offers = vec![RecoveryOffer::new(
        1,
        key(2),
        RecoveryKind::Retry,
        None,
        None,
    )];
    let errors = diagnostics(project_denial_recovery(
        &route(),
        &supplied,
        DenialRecoveryLimits::new(256, 32, 32, 32, 0, 4096),
    ));
    assert_eq!(
        errors[0].stage(),
        DenialRecoveryProjectionStage::BatchValidation
    );
}

#[test]
fn ui_dna2_denial_recovery_all_stable_diagnostic_codes_are_frozen() {
    let drp = [
        DenialRecoveryProjectionDiagnosticKind::ResourceLimitExceeded,
        DenialRecoveryProjectionDiagnosticKind::MissingReason,
        DenialRecoveryProjectionDiagnosticKind::MissingEvidenceRef,
        DenialRecoveryProjectionDiagnosticKind::InvalidFreshnessOutcome,
        DenialRecoveryProjectionDiagnosticKind::DuplicateBatchOrder,
        DenialRecoveryProjectionDiagnosticKind::DuplicateBatchKey,
        DenialRecoveryProjectionDiagnosticKind::InvalidAtomicBatch,
        DenialRecoveryProjectionDiagnosticKind::InvalidOrderedPartialBatch,
        DenialRecoveryProjectionDiagnosticKind::BatchBreakMismatch,
        DenialRecoveryProjectionDiagnosticKind::MissingBatchRoute,
        DenialRecoveryProjectionDiagnosticKind::DuplicateRecoveryOrder,
        DenialRecoveryProjectionDiagnosticKind::DuplicateRecoveryKey,
        DenialRecoveryProjectionDiagnosticKind::MissingRecoveryRoute,
        DenialRecoveryProjectionDiagnosticKind::RecoveryActionRefMissing,
        DenialRecoveryProjectionDiagnosticKind::ResumeTokenMissing,
        DenialRecoveryProjectionDiagnosticKind::OperationLimitExceeded,
        DenialRecoveryProjectionDiagnosticKind::PatchRejected,
    ];
    assert_eq!(drp.len(), 17);
    assert!(drp.iter().all(|kind| kind.code().starts_with("DRP_")));

    let cfp = [
        FreshnessProjectionDiagnosticKind::ResourceLimitExceeded,
        FreshnessProjectionDiagnosticKind::DuplicateControlNode,
        FreshnessProjectionDiagnosticKind::InvalidCriticalControlAvailability,
        FreshnessProjectionDiagnosticKind::OperationLimitExceeded,
    ];
    assert_eq!(cfp.len(), 4);
    assert!(cfp.iter().all(|kind| kind.code().starts_with("CFP_")));
}
