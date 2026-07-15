//! Crate-private denial, batch, recovery, and freshness patch composition.
//!
//! Inputs are caller-supplied evidence. Successful projection constructs an
//! inert `ProjectionPatch`; it does not admit, dispatch, recover, or mutate UI.
#![allow(dead_code)]

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::binding_graph::BindingTarget;
use crate::connectivity_projection::{
    project_freshness_fragment, FreshnessProjectionDiagnostic, FreshnessProjectionLimits,
    FreshnessProjectionRoute, FreshnessState,
};
use crate::contract_primitives::{CollectionKey, StaticNodeId};
use crate::projection_patch::{
    ProjectionPatch, ProjectionPatchDiagnostics, ProjectionPatchEnvelope, ProjectionPatchOperation,
    ProjectionPatchValue,
};
use crate::semantic_refs::{ReferenceToken, SemanticActionRef, SemanticEvidenceRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionResultCategory {
    Accepted,
    Denied,
    LocalDenied,
    AdmissionDenied,
    CapabilityRejected,
    StaleRejected,
    FreshnessRejected,
    PartialDenied,
    NotApplied,
    BatchBreak,
    PendingUnknown,
    Quarantined,
}

impl ProjectionResultCategory {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Denied => "denied",
            Self::LocalDenied => "local_denied",
            Self::AdmissionDenied => "admission_denied",
            Self::CapabilityRejected => "capability_rejected",
            Self::StaleRejected => "stale_rejected",
            Self::FreshnessRejected => "freshness_rejected",
            Self::PartialDenied => "partial_denied",
            Self::NotApplied => "not_applied",
            Self::BatchBreak => "batch_break",
            Self::PendingUnknown => "pending_unknown",
            Self::Quarantined => "quarantined",
        }
    }

    const fn is_denial_like(self) -> bool {
        matches!(
            self,
            Self::Denied
                | Self::LocalDenied
                | Self::AdmissionDenied
                | Self::CapabilityRejected
                | Self::StaleRejected
                | Self::FreshnessRejected
                | Self::Quarantined
        )
    }

    const fn requires_batch(self) -> bool {
        matches!(
            self,
            Self::PartialDenied | Self::BatchBreak | Self::NotApplied
        )
    }

    const fn requires_evidence(self) -> bool {
        matches!(
            self,
            Self::AdmissionDenied | Self::CapabilityRejected | Self::Quarantined
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BatchMode {
    Atomic,
    OrderedPartial,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BatchProjectionItem {
    order: u32,
    key: CollectionKey,
    result: ProjectionResultCategory,
}

impl BatchProjectionItem {
    pub(crate) const fn new(
        order: u32,
        key: CollectionKey,
        result: ProjectionResultCategory,
    ) -> Self {
        Self { order, key, result }
    }

    pub(crate) const fn order(&self) -> u32 {
        self.order
    }

    pub(crate) const fn key(&self) -> CollectionKey {
        self.key
    }

    pub(crate) const fn result(&self) -> ProjectionResultCategory {
        self.result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BatchProjection {
    mode: BatchMode,
    declared_break_order: Option<u32>,
    items: Vec<BatchProjectionItem>,
}

impl BatchProjection {
    pub(crate) fn new(
        mode: BatchMode,
        declared_break_order: Option<u32>,
        items: Vec<BatchProjectionItem>,
    ) -> Self {
        Self {
            mode,
            declared_break_order,
            items,
        }
    }

    pub(crate) const fn mode(&self) -> BatchMode {
        self.mode
    }

    pub(crate) const fn declared_break_order(&self) -> Option<u32> {
        self.declared_break_order
    }

    pub(crate) fn items(&self) -> &[BatchProjectionItem] {
        &self.items
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum RecoveryKind {
    Dismiss,
    Acknowledge,
    Retry,
    Resume,
    CancelSuffix,
}

impl RecoveryKind {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Dismiss => "dismiss",
            Self::Acknowledge => "acknowledge",
            Self::Retry => "retry",
            Self::Resume => "resume",
            Self::CancelSuffix => "cancel_suffix",
        }
    }

    const fn requires_action_ref(self) -> bool {
        !matches!(self, Self::Dismiss)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RecoveryOffer {
    order: u32,
    key: CollectionKey,
    kind: RecoveryKind,
    action_ref: Option<SemanticActionRef>,
    resume_token: Option<ReferenceToken>,
}

impl RecoveryOffer {
    pub(crate) const fn new(
        order: u32,
        key: CollectionKey,
        kind: RecoveryKind,
        action_ref: Option<SemanticActionRef>,
        resume_token: Option<ReferenceToken>,
    ) -> Self {
        Self {
            order,
            key,
            kind,
            action_ref,
            resume_token,
        }
    }

    pub(crate) const fn order(self) -> u32 {
        self.order
    }

    pub(crate) const fn key(self) -> CollectionKey {
        self.key
    }

    pub(crate) const fn kind(self) -> RecoveryKind {
        self.kind
    }

    pub(crate) const fn action_ref(self) -> Option<SemanticActionRef> {
        self.action_ref
    }

    pub(crate) const fn resume_token(self) -> Option<ReferenceToken> {
        self.resume_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DenialRecoveryRoute {
    result_target: BindingTarget,
    reason_target: BindingTarget,
    batch_collection: Option<StaticNodeId>,
    recovery_collection: Option<StaticNodeId>,
    freshness_route: FreshnessProjectionRoute,
}

impl DenialRecoveryRoute {
    pub(crate) fn new(
        result_target: BindingTarget,
        reason_target: BindingTarget,
        batch_collection: Option<StaticNodeId>,
        recovery_collection: Option<StaticNodeId>,
        freshness_route: FreshnessProjectionRoute,
    ) -> Self {
        Self {
            result_target,
            reason_target,
            batch_collection,
            recovery_collection,
            freshness_route,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DenialRecoveryEvidence {
    pub(crate) envelope: ProjectionPatchEnvelope,
    pub(crate) result: ProjectionResultCategory,
    pub(crate) reason: String,
    pub(crate) action_ref: Option<SemanticActionRef>,
    pub(crate) evidence_ref: Option<SemanticEvidenceRef>,
    pub(crate) freshness: FreshnessState,
    pub(crate) critical_attempt: bool,
    pub(crate) batch: Option<BatchProjection>,
    pub(crate) recovery_offers: Vec<RecoveryOffer>,
}

impl DenialRecoveryEvidence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        envelope: ProjectionPatchEnvelope,
        result: ProjectionResultCategory,
        reason: String,
        action_ref: Option<SemanticActionRef>,
        evidence_ref: Option<SemanticEvidenceRef>,
        freshness: FreshnessState,
        critical_attempt: bool,
        batch: Option<BatchProjection>,
        recovery_offers: Vec<RecoveryOffer>,
    ) -> Self {
        Self {
            envelope,
            result,
            reason,
            action_ref,
            evidence_ref,
            freshness,
            critical_attempt,
            batch,
            recovery_offers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DenialRecoveryLimits {
    max_reason_bytes: usize,
    max_batch_items: usize,
    max_recovery_offers: usize,
    max_control_routes: usize,
    max_total_operations: usize,
    max_total_projected_text_bytes: usize,
}

impl DenialRecoveryLimits {
    pub(crate) const fn new(
        max_reason_bytes: usize,
        max_batch_items: usize,
        max_recovery_offers: usize,
        max_control_routes: usize,
        max_total_operations: usize,
        max_total_projected_text_bytes: usize,
    ) -> Self {
        Self {
            max_reason_bytes,
            max_batch_items,
            max_recovery_offers,
            max_control_routes,
            max_total_operations,
            max_total_projected_text_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DenialRecoveryProjectionStage {
    ResourcePreflight,
    RouteValidation,
    FreshnessValidation,
    OutcomeValidation,
    BatchValidation,
    RecoveryValidation,
    OperationConstruction,
    PatchValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DenialRecoveryProjectionDiagnosticKind {
    ResourceLimitExceeded,
    MissingReason,
    MissingEvidenceRef,
    InvalidFreshnessOutcome,
    DuplicateBatchOrder,
    DuplicateBatchKey,
    InvalidAtomicBatch,
    InvalidOrderedPartialBatch,
    BatchBreakMismatch,
    MissingBatchRoute,
    DuplicateRecoveryOrder,
    DuplicateRecoveryKey,
    MissingRecoveryRoute,
    RecoveryActionRefMissing,
    ResumeTokenMissing,
    OperationLimitExceeded,
    PatchRejected,
}

impl DenialRecoveryProjectionDiagnosticKind {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::ResourceLimitExceeded => "DRP_RESOURCE_LIMIT_EXCEEDED",
            Self::MissingReason => "DRP_MISSING_REASON",
            Self::MissingEvidenceRef => "DRP_MISSING_EVIDENCE_REF",
            Self::InvalidFreshnessOutcome => "DRP_INVALID_FRESHNESS_OUTCOME",
            Self::DuplicateBatchOrder => "DRP_DUPLICATE_BATCH_ORDER",
            Self::DuplicateBatchKey => "DRP_DUPLICATE_BATCH_KEY",
            Self::InvalidAtomicBatch => "DRP_INVALID_ATOMIC_BATCH",
            Self::InvalidOrderedPartialBatch => "DRP_INVALID_ORDERED_PARTIAL_BATCH",
            Self::BatchBreakMismatch => "DRP_BATCH_BREAK_MISMATCH",
            Self::MissingBatchRoute => "DRP_MISSING_BATCH_ROUTE",
            Self::DuplicateRecoveryOrder => "DRP_DUPLICATE_RECOVERY_ORDER",
            Self::DuplicateRecoveryKey => "DRP_DUPLICATE_RECOVERY_KEY",
            Self::MissingRecoveryRoute => "DRP_MISSING_RECOVERY_ROUTE",
            Self::RecoveryActionRefMissing => "DRP_RECOVERY_ACTION_REF_MISSING",
            Self::ResumeTokenMissing => "DRP_RESUME_TOKEN_MISSING",
            Self::OperationLimitExceeded => "DRP_OPERATION_LIMIT_EXCEEDED",
            Self::PatchRejected => "DRP_PATCH_REJECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DenialRecoveryLimitKind {
    ReasonBytes,
    BatchItems,
    RecoveryOffers,
    ControlRoutes,
    TotalProjectedTextBytes,
    TotalOperations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DenialRecoveryProjectionDiagnostic {
    stage: DenialRecoveryProjectionStage,
    kind: DenialRecoveryProjectionDiagnosticKind,
    primary: u64,
    secondary: u64,
    limit_kind: Option<DenialRecoveryLimitKind>,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl DenialRecoveryProjectionDiagnostic {
    const fn content(
        stage: DenialRecoveryProjectionStage,
        kind: DenialRecoveryProjectionDiagnosticKind,
        primary: u64,
        secondary: u64,
    ) -> Self {
        Self {
            stage,
            kind,
            primary,
            secondary,
            limit_kind: None,
            actual: None,
            maximum: None,
        }
    }

    const fn limit(
        stage: DenialRecoveryProjectionStage,
        kind: DenialRecoveryProjectionDiagnosticKind,
        limit_kind: DenialRecoveryLimitKind,
        actual: usize,
        maximum: usize,
    ) -> Self {
        Self {
            stage,
            kind,
            primary: 0,
            secondary: 0,
            limit_kind: Some(limit_kind),
            actual: Some(actual),
            maximum: Some(maximum),
        }
    }

    pub(crate) const fn stage(self) -> DenialRecoveryProjectionStage {
        self.stage
    }

    pub(crate) const fn kind(self) -> DenialRecoveryProjectionDiagnosticKind {
        self.kind
    }

    pub(crate) const fn code(self) -> &'static str {
        self.kind.code()
    }

    pub(crate) const fn primary(self) -> u64 {
        self.primary
    }

    pub(crate) const fn secondary(self) -> u64 {
        self.secondary
    }

    pub(crate) const fn limit_kind(self) -> Option<DenialRecoveryLimitKind> {
        self.limit_kind
    }

    pub(crate) const fn actual(self) -> Option<usize> {
        self.actual
    }

    pub(crate) const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DenialRecoveryProjectionFailure {
    Diagnostics(Vec<DenialRecoveryProjectionDiagnostic>),
    Freshness(Vec<FreshnessProjectionDiagnostic>),
    PatchRejected {
        diagnostic: DenialRecoveryProjectionDiagnostic,
        patch_diagnostics: ProjectionPatchDiagnostics,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DenialRecoveryProjectionArtifact {
    patch: ProjectionPatch,
    action_ref: Option<SemanticActionRef>,
    evidence_ref: Option<SemanticEvidenceRef>,
    recovery_offers: Vec<RecoveryOffer>,
    batch: Option<BatchProjection>,
    operation_count: usize,
}

impl DenialRecoveryProjectionArtifact {
    pub(crate) const fn patch(&self) -> &ProjectionPatch {
        &self.patch
    }

    pub(crate) const fn action_ref(&self) -> Option<SemanticActionRef> {
        self.action_ref
    }

    pub(crate) const fn evidence_ref(&self) -> Option<SemanticEvidenceRef> {
        self.evidence_ref
    }

    pub(crate) fn recovery_offers(&self) -> &[RecoveryOffer] {
        &self.recovery_offers
    }

    pub(crate) const fn batch(&self) -> Option<&BatchProjection> {
        self.batch.as_ref()
    }

    pub(crate) const fn operation_count(&self) -> usize {
        self.operation_count
    }
}

pub(crate) fn project_denial_recovery(
    route: &DenialRecoveryRoute,
    evidence: &DenialRecoveryEvidence,
    limits: DenialRecoveryLimits,
) -> Result<DenialRecoveryProjectionArtifact, DenialRecoveryProjectionFailure> {
    validate_resource_preflight(route, evidence, limits)?;
    validate_routes(route, evidence)?;

    let freshness_fragment = project_freshness_fragment(
        evidence.freshness,
        &route.freshness_route,
        FreshnessProjectionLimits::new(limits.max_control_routes, usize::MAX),
    )
    .map_err(DenialRecoveryProjectionFailure::Freshness)?;
    if evidence.critical_attempt
        && evidence.freshness.restricts_critical_controls()
        && evidence.result != ProjectionResultCategory::LocalDenied
    {
        return Err(diagnostics_failure(vec![
            DenialRecoveryProjectionDiagnostic::content(
                DenialRecoveryProjectionStage::FreshnessValidation,
                DenialRecoveryProjectionDiagnosticKind::InvalidFreshnessOutcome,
                0,
                0,
            ),
        ]));
    }

    validate_outcome(evidence)?;
    let batch = validate_batch(evidence)?;
    let recovery_offers = validate_recovery(evidence)?;

    let operation_count = predicted_operation_count(route, evidence).ok_or_else(|| {
        diagnostics_failure(vec![DenialRecoveryProjectionDiagnostic::limit(
            DenialRecoveryProjectionStage::OperationConstruction,
            DenialRecoveryProjectionDiagnosticKind::OperationLimitExceeded,
            DenialRecoveryLimitKind::TotalOperations,
            usize::MAX,
            limits.max_total_operations,
        )])
    })?;
    if operation_count > limits.max_total_operations {
        return Err(diagnostics_failure(vec![
            DenialRecoveryProjectionDiagnostic::limit(
                DenialRecoveryProjectionStage::OperationConstruction,
                DenialRecoveryProjectionDiagnosticKind::OperationLimitExceeded,
                DenialRecoveryLimitKind::TotalOperations,
                operation_count,
                limits.max_total_operations,
            ),
        ]));
    }

    let mut operations = Vec::with_capacity(operation_count);
    operations.push(ProjectionPatchOperation::SetBindingValue {
        target: route.result_target.clone(),
        value: ProjectionPatchValue::Text(evidence.result.token().to_string()),
    });
    if !evidence.reason.is_empty() {
        operations.push(ProjectionPatchOperation::SetBindingValue {
            target: route.reason_target.clone(),
            value: ProjectionPatchValue::Text(evidence.reason.clone()),
        });
    }

    let mut freshness_operations = freshness_fragment.into_operations().into_iter();
    if let Some(freshness) = freshness_operations.next() {
        operations.push(freshness);
    }
    if let (Some(collection), Some(batch)) = (route.batch_collection, batch.as_ref()) {
        for item in &batch.items {
            operations.push(ProjectionPatchOperation::CollectionInsert {
                collection,
                key: item.key,
                before: None,
                value: ProjectionPatchValue::Text(item.result.token().to_string()),
            });
        }
    }
    operations.extend(freshness_operations);
    if let Some(collection) = route.recovery_collection {
        for offer in &recovery_offers {
            operations.push(ProjectionPatchOperation::CollectionInsert {
                collection,
                key: offer.key,
                before: None,
                value: ProjectionPatchValue::Text(offer.kind.token().to_string()),
            });
        }
    }

    let patch =
        ProjectionPatch::new(evidence.envelope, operations).map_err(|patch_diagnostics| {
            DenialRecoveryProjectionFailure::PatchRejected {
                diagnostic: DenialRecoveryProjectionDiagnostic::content(
                    DenialRecoveryProjectionStage::PatchValidation,
                    DenialRecoveryProjectionDiagnosticKind::PatchRejected,
                    evidence.envelope.patch_id().raw(),
                    0,
                ),
                patch_diagnostics,
            }
        })?;

    Ok(DenialRecoveryProjectionArtifact {
        patch,
        action_ref: evidence.action_ref,
        evidence_ref: evidence.evidence_ref,
        recovery_offers,
        batch,
        operation_count,
    })
}

fn validate_resource_preflight(
    route: &DenialRecoveryRoute,
    evidence: &DenialRecoveryEvidence,
    limits: DenialRecoveryLimits,
) -> Result<(), DenialRecoveryProjectionFailure> {
    let batch_len = evidence.batch.as_ref().map_or(0, |batch| batch.items.len());
    let controls_len = route.freshness_route.controls().len();
    let checks = [
        (
            DenialRecoveryLimitKind::ReasonBytes,
            evidence.reason.len(),
            limits.max_reason_bytes,
        ),
        (
            DenialRecoveryLimitKind::BatchItems,
            batch_len,
            limits.max_batch_items,
        ),
        (
            DenialRecoveryLimitKind::RecoveryOffers,
            evidence.recovery_offers.len(),
            limits.max_recovery_offers,
        ),
        (
            DenialRecoveryLimitKind::ControlRoutes,
            controls_len,
            limits.max_control_routes,
        ),
    ];
    let mut diagnostics = Vec::new();
    for (kind, actual, maximum) in checks {
        if actual > maximum {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::limit(
                DenialRecoveryProjectionStage::ResourcePreflight,
                DenialRecoveryProjectionDiagnosticKind::ResourceLimitExceeded,
                kind,
                actual,
                maximum,
            ));
        }
    }

    match projected_text_bytes(evidence) {
        Some(actual) if actual > limits.max_total_projected_text_bytes => {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::limit(
                DenialRecoveryProjectionStage::ResourcePreflight,
                DenialRecoveryProjectionDiagnosticKind::ResourceLimitExceeded,
                DenialRecoveryLimitKind::TotalProjectedTextBytes,
                actual,
                limits.max_total_projected_text_bytes,
            ));
        }
        None => diagnostics.push(DenialRecoveryProjectionDiagnostic::limit(
            DenialRecoveryProjectionStage::ResourcePreflight,
            DenialRecoveryProjectionDiagnosticKind::ResourceLimitExceeded,
            DenialRecoveryLimitKind::TotalProjectedTextBytes,
            usize::MAX,
            limits.max_total_projected_text_bytes,
        )),
        Some(_) => {}
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics_failure(diagnostics))
    }
}

fn validate_routes(
    route: &DenialRecoveryRoute,
    evidence: &DenialRecoveryEvidence,
) -> Result<(), DenialRecoveryProjectionFailure> {
    let mut diagnostics = Vec::new();
    if evidence
        .batch
        .as_ref()
        .is_some_and(|batch| !batch.items.is_empty())
        && route.batch_collection.is_none()
    {
        diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
            DenialRecoveryProjectionStage::RouteValidation,
            DenialRecoveryProjectionDiagnosticKind::MissingBatchRoute,
            0,
            0,
        ));
    }
    if !evidence.recovery_offers.is_empty() && route.recovery_collection.is_none() {
        diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
            DenialRecoveryProjectionStage::RouteValidation,
            DenialRecoveryProjectionDiagnosticKind::MissingRecoveryRoute,
            0,
            0,
        ));
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics_failure(diagnostics))
    }
}

fn validate_outcome(
    evidence: &DenialRecoveryEvidence,
) -> Result<(), DenialRecoveryProjectionFailure> {
    let mut diagnostics = Vec::new();
    if evidence.result != ProjectionResultCategory::Accepted && evidence.reason.is_empty() {
        diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
            DenialRecoveryProjectionStage::OutcomeValidation,
            DenialRecoveryProjectionDiagnosticKind::MissingReason,
            0,
            0,
        ));
    }
    if evidence.result.requires_evidence() && evidence.evidence_ref.is_none() {
        diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
            DenialRecoveryProjectionStage::OutcomeValidation,
            DenialRecoveryProjectionDiagnosticKind::MissingEvidenceRef,
            0,
            0,
        ));
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics_failure(diagnostics))
    }
}

fn validate_batch(
    evidence: &DenialRecoveryEvidence,
) -> Result<Option<BatchProjection>, DenialRecoveryProjectionFailure> {
    let Some(mut batch) = evidence.batch.clone() else {
        if evidence.result.requires_batch() {
            return Err(diagnostics_failure(vec![
                DenialRecoveryProjectionDiagnostic::content(
                    DenialRecoveryProjectionStage::BatchValidation,
                    DenialRecoveryProjectionDiagnosticKind::InvalidOrderedPartialBatch,
                    0,
                    0,
                ),
            ]));
        }
        return Ok(None);
    };

    batch.items.sort_by_key(|item| (item.order, item.key));
    let mut diagnostics = duplicate_batch_diagnostics(&batch.items);
    if diagnostics.is_empty() {
        match batch.mode {
            BatchMode::Atomic => {
                let all_accepted = batch
                    .items
                    .iter()
                    .all(|item| item.result == ProjectionResultCategory::Accepted);
                let all_denied_or_not_applied = batch.items.iter().all(|item| {
                    item.result.is_denial_like()
                        || item.result == ProjectionResultCategory::NotApplied
                });
                if !all_accepted && !all_denied_or_not_applied {
                    diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                        DenialRecoveryProjectionStage::BatchValidation,
                        DenialRecoveryProjectionDiagnosticKind::InvalidAtomicBatch,
                        0,
                        0,
                    ));
                }
            }
            BatchMode::OrderedPartial => {
                let mut phase = 0_u8;
                let mut break_order = None;
                let mut valid = true;
                for item in &batch.items {
                    match (phase, item.result) {
                        (0, ProjectionResultCategory::Accepted) => {}
                        (0, result) if result.is_denial_like() => {
                            phase = 1;
                            break_order = Some(item.order);
                        }
                        (1, ProjectionResultCategory::NotApplied) => phase = 2,
                        (2, ProjectionResultCategory::NotApplied) => {}
                        _ => valid = false,
                    }
                }
                if break_order.is_none() || !valid {
                    diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                        DenialRecoveryProjectionStage::BatchValidation,
                        DenialRecoveryProjectionDiagnosticKind::InvalidOrderedPartialBatch,
                        0,
                        0,
                    ));
                } else if batch.declared_break_order != break_order {
                    diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                        DenialRecoveryProjectionStage::BatchValidation,
                        DenialRecoveryProjectionDiagnosticKind::BatchBreakMismatch,
                        u64::from(batch.declared_break_order.unwrap_or(u32::MAX)),
                        u64::from(break_order.unwrap_or(u32::MAX)),
                    ));
                }
            }
        }
    }

    if evidence.result == ProjectionResultCategory::Accepted
        && batch
            .items
            .iter()
            .any(|item| item.result != ProjectionResultCategory::Accepted)
    {
        diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
            DenialRecoveryProjectionStage::BatchValidation,
            DenialRecoveryProjectionDiagnosticKind::InvalidAtomicBatch,
            0,
            0,
        ));
    }

    if diagnostics.is_empty() {
        Ok(Some(batch))
    } else {
        Err(diagnostics_failure(diagnostics))
    }
}

fn duplicate_batch_diagnostics(
    items: &[BatchProjectionItem],
) -> Vec<DenialRecoveryProjectionDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut orders = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for item in items {
        if !orders.insert(item.order) {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                DenialRecoveryProjectionStage::BatchValidation,
                DenialRecoveryProjectionDiagnosticKind::DuplicateBatchOrder,
                u64::from(item.order),
                item.key.raw(),
            ));
        }
        if !keys.insert(item.key) {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                DenialRecoveryProjectionStage::BatchValidation,
                DenialRecoveryProjectionDiagnosticKind::DuplicateBatchKey,
                item.key.raw(),
                u64::from(item.order),
            ));
        }
    }
    diagnostics
}

fn validate_recovery(
    evidence: &DenialRecoveryEvidence,
) -> Result<Vec<RecoveryOffer>, DenialRecoveryProjectionFailure> {
    let mut offers = evidence.recovery_offers.clone();
    offers.sort_by_key(|offer| (offer.order, offer.key, offer.kind));
    let mut diagnostics = Vec::new();
    let mut orders = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for offer in &offers {
        if !orders.insert(offer.order) {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                DenialRecoveryProjectionStage::RecoveryValidation,
                DenialRecoveryProjectionDiagnosticKind::DuplicateRecoveryOrder,
                u64::from(offer.order),
                offer.key.raw(),
            ));
        }
        if !keys.insert(offer.key) {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                DenialRecoveryProjectionStage::RecoveryValidation,
                DenialRecoveryProjectionDiagnosticKind::DuplicateRecoveryKey,
                offer.key.raw(),
                u64::from(offer.order),
            ));
        }
        if offer.kind.requires_action_ref() && offer.action_ref.is_none() {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                DenialRecoveryProjectionStage::RecoveryValidation,
                DenialRecoveryProjectionDiagnosticKind::RecoveryActionRefMissing,
                u64::from(offer.order),
                offer.key.raw(),
            ));
        }
        if offer.kind == RecoveryKind::Resume && offer.resume_token.is_none() {
            diagnostics.push(DenialRecoveryProjectionDiagnostic::content(
                DenialRecoveryProjectionStage::RecoveryValidation,
                DenialRecoveryProjectionDiagnosticKind::ResumeTokenMissing,
                u64::from(offer.order),
                offer.key.raw(),
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(offers)
    } else {
        Err(diagnostics_failure(diagnostics))
    }
}

fn predicted_operation_count(
    route: &DenialRecoveryRoute,
    evidence: &DenialRecoveryEvidence,
) -> Option<usize> {
    let mut count = 2_usize;
    if !evidence.reason.is_empty() {
        count = count.checked_add(1)?;
    }
    count = count.checked_add(route.freshness_route.controls().len())?;
    count = count.checked_add(evidence.batch.as_ref().map_or(0, |batch| batch.items.len()))?;
    count.checked_add(evidence.recovery_offers.len())
}

fn projected_text_bytes(evidence: &DenialRecoveryEvidence) -> Option<usize> {
    let mut bytes = evidence
        .result
        .token()
        .len()
        .checked_add(evidence.freshness.token().len())?;
    bytes = bytes.checked_add(evidence.reason.len())?;
    if let Some(batch) = &evidence.batch {
        for item in &batch.items {
            bytes = bytes.checked_add(item.result.token().len())?;
        }
    }
    for offer in &evidence.recovery_offers {
        bytes = bytes.checked_add(offer.kind.token().len())?;
    }
    Some(bytes)
}

fn diagnostics_failure(
    mut diagnostics: Vec<DenialRecoveryProjectionDiagnostic>,
) -> DenialRecoveryProjectionFailure {
    diagnostics.sort();
    diagnostics.dedup();
    DenialRecoveryProjectionFailure::Diagnostics(diagnostics)
}
