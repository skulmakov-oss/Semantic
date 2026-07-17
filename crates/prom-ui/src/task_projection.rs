//! Crate-private Task Projection v0 contract execution.
//!
//! This module validates caller-supplied task presentation evidence and
//! constructs one inert `ProjectionPatch`. It does not read Semantic state,
//! admit or execute actions, apply patches, mutate shell state, or own task
//! truth.
#![allow(
    dead_code,
    reason = "crate-private qualification contour is intentionally not runtime-wired before Gate D"
)]

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::binding_graph::BindingTarget;
use crate::connectivity_projection::{
    project_freshness_fragment, FreshnessProjectionDiagnostic, FreshnessProjectionLimits,
    FreshnessProjectionRoute, FreshnessState,
};
use crate::contract_primitives::{CollectionKey, Revision, StaticNodeId};
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatch, ProjectionPatchDiagnostics,
    ProjectionPatchEnvelope, ProjectionPatchOperation, ProjectionPatchValue,
};
use crate::semantic_refs::{ReferenceToken, SemanticActionRef, SemanticEvidenceRef, TaskRecordRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TaskProjectionState {
    Pending,
    Started,
    Running,
    AwaitingInput,
    Paused,
    Completing,
    Completed,
    Failed,
    Denied,
    Quarantined,
    Cancelled,
    PendingUnknown,
}

impl TaskProjectionState {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Started => "started",
            Self::Running => "running",
            Self::AwaitingInput => "awaiting_input",
            Self::Paused => "paused",
            Self::Completing => "completing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Denied => "denied",
            Self::Quarantined => "quarantined",
            Self::Cancelled => "cancelled",
            Self::PendingUnknown => "pending_unknown",
        }
    }

    const fn requires_task_evidence(self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Failed
                | Self::Denied
                | Self::Quarantined
                | Self::Cancelled
                | Self::PendingUnknown
        )
    }

    const fn requires_active_phase(self) -> bool {
        matches!(
            self,
            Self::Started | Self::Running | Self::AwaitingInput | Self::Paused | Self::Completing
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TaskPhaseStatus {
    Pending,
    Active,
    Completed,
    Blocked,
    Failed,
}

impl TaskPhaseStatus {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskPhase {
    pub(crate) id: u64,
    pub(crate) key: CollectionKey,
    pub(crate) order: u64,
    pub(crate) label: String,
    pub(crate) status: TaskPhaseStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TaskProgress {
    Indeterminate,
    Determinate { completed: u64, total: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TaskControlKind {
    Pause,
    Resume,
    Cancel,
    Retry,
    Acknowledge,
    ProvideInput,
}

impl TaskControlKind {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Cancel => "cancel",
            Self::Retry => "retry",
            Self::Acknowledge => "acknowledge",
            Self::ProvideInput => "provide_input",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskControlOffer {
    pub(crate) order: u64,
    pub(crate) key: CollectionKey,
    pub(crate) kind: TaskControlKind,
    pub(crate) action: SemanticActionRef,
    pub(crate) resume_token: Option<ReferenceToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskScopeLock {
    pub(crate) order: u64,
    pub(crate) key: CollectionKey,
    pub(crate) reference: ReferenceToken,
    pub(crate) explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskProjectionEvidence {
    pub(crate) task_ref: TaskRecordRef,
    pub(crate) previous_revision: Revision,
    pub(crate) new_revision: Revision,
    pub(crate) state: TaskProjectionState,
    pub(crate) phases: Vec<TaskPhase>,
    pub(crate) current_progress: TaskProgress,
    pub(crate) previous_progress: Option<TaskProgress>,
    pub(crate) regression_evidence: Option<SemanticEvidenceRef>,
    pub(crate) awaiting_input: Option<String>,
    pub(crate) task_evidence: Option<SemanticEvidenceRef>,
    pub(crate) freshness: FreshnessState,
    pub(crate) controls: Vec<TaskControlOffer>,
    pub(crate) locks: Vec<TaskScopeLock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTaskProjection {
    pub(crate) task_ref: TaskRecordRef,
    pub(crate) previous_revision: Revision,
    pub(crate) new_revision: Revision,
    pub(crate) state: TaskProjectionState,
    pub(crate) phases: Vec<TaskPhase>,
    pub(crate) current_progress: TaskProgress,
    pub(crate) previous_progress: Option<TaskProgress>,
    pub(crate) regression_evidence: Option<SemanticEvidenceRef>,
    pub(crate) awaiting_input: Option<String>,
    pub(crate) task_evidence: Option<SemanticEvidenceRef>,
    pub(crate) freshness: FreshnessState,
    pub(crate) controls: Vec<TaskControlOffer>,
    pub(crate) locks: Vec<TaskScopeLock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskProjectionRoutes {
    pub(crate) identity_route: BindingTarget,
    pub(crate) state_route: BindingTarget,
    pub(crate) progress_route: BindingTarget,
    pub(crate) awaiting_input_route: Option<BindingTarget>,
    pub(crate) freshness_route: FreshnessProjectionRoute,
    pub(crate) phase_collection: Option<StaticNodeId>,
    pub(crate) control_collection: Option<StaticNodeId>,
    pub(crate) scope_lock_collection: Option<StaticNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TaskProjectionLimits {
    pub(crate) phase_count: usize,
    pub(crate) control_count: usize,
    pub(crate) scope_lock_count: usize,
    pub(crate) phase_label_bytes: usize,
    pub(crate) awaiting_input_bytes: usize,
    pub(crate) lock_explanation_bytes: usize,
    pub(crate) total_projected_text_bytes: usize,
    pub(crate) total_operations: usize,
    pub(crate) freshness_limits: FreshnessProjectionLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ValidationStage {
    ResourcePreflight,
    RouteValidation,
    IdentityRevisionValidation,
    StateValidation,
    PhaseValidation,
    ProgressValidation,
    ControlValidation,
    ScopeLockValidation,
    OperationConstruction,
    PatchValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TaskProjectionDiagnosticKind {
    ResourceLimitExceeded,
    MissingTaskRef,
    NonIncreasingTaskRevision,
    MissingEvidenceRef,
    InvalidStateDetail,
    MissingAwaitingInput,
    UnexpectedAwaitingInput,
    DuplicatePhaseOrder,
    DuplicatePhaseKey,
    InvalidPhaseSet,
    InvalidProgress,
    ProgressRegressionWithoutEvidence,
    DuplicateControlOrder,
    DuplicateControlKey,
    ControlActionRefMissing,
    ResumeTokenMissing,
    StaleControlOffer,
    DuplicateLockOrder,
    DuplicateLockKey,
    EmptyLockExplanation,
    MissingPhaseRoute,
    MissingControlRoute,
    MissingLockRoute,
    MissingAwaitingInputRoute,
    OperationLimitExceeded,
    PatchRejected,
}

impl TaskProjectionDiagnosticKind {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::ResourceLimitExceeded => "TPP_RESOURCE_LIMIT_EXCEEDED",
            Self::MissingTaskRef => "TPP_MISSING_TASK_REF",
            Self::NonIncreasingTaskRevision => "TPP_NON_INCREASING_TASK_REVISION",
            Self::MissingEvidenceRef => "TPP_MISSING_EVIDENCE_REF",
            Self::InvalidStateDetail => "TPP_INVALID_STATE_DETAIL",
            Self::MissingAwaitingInput => "TPP_MISSING_AWAITING_INPUT",
            Self::UnexpectedAwaitingInput => "TPP_UNEXPECTED_AWAITING_INPUT",
            Self::DuplicatePhaseOrder => "TPP_DUPLICATE_PHASE_ORDER",
            Self::DuplicatePhaseKey => "TPP_DUPLICATE_PHASE_KEY",
            Self::InvalidPhaseSet => "TPP_INVALID_PHASE_SET",
            Self::InvalidProgress => "TPP_INVALID_PROGRESS",
            Self::ProgressRegressionWithoutEvidence => "TPP_PROGRESS_REGRESSION_WITHOUT_EVIDENCE",
            Self::DuplicateControlOrder => "TPP_DUPLICATE_CONTROL_ORDER",
            Self::DuplicateControlKey => "TPP_DUPLICATE_CONTROL_KEY",
            Self::ControlActionRefMissing => "TPP_CONTROL_ACTION_REF_MISSING",
            Self::ResumeTokenMissing => "TPP_RESUME_TOKEN_MISSING",
            Self::StaleControlOffer => "TPP_STALE_CONTROL_OFFER",
            Self::DuplicateLockOrder => "TPP_DUPLICATE_LOCK_ORDER",
            Self::DuplicateLockKey => "TPP_DUPLICATE_LOCK_KEY",
            Self::EmptyLockExplanation => "TPP_EMPTY_LOCK_EXPLANATION",
            Self::MissingPhaseRoute => "TPP_MISSING_PHASE_ROUTE",
            Self::MissingControlRoute => "TPP_MISSING_CONTROL_ROUTE",
            Self::MissingLockRoute => "TPP_MISSING_LOCK_ROUTE",
            Self::MissingAwaitingInputRoute => "TPP_MISSING_AWAITING_INPUT_ROUTE",
            Self::OperationLimitExceeded => "TPP_OPERATION_LIMIT_EXCEEDED",
            Self::PatchRejected => "TPP_PATCH_REJECTED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskProjectionDiagnostic {
    pub(crate) stage: ValidationStage,
    pub(crate) kind: TaskProjectionDiagnosticKind,
}

impl TaskProjectionDiagnostic {
    const fn new(stage: ValidationStage, kind: TaskProjectionDiagnosticKind) -> Self {
        Self { stage, kind }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TaskProjectionError {
    Task(TaskProjectionDiagnostic),
    Freshness(Vec<FreshnessProjectionDiagnostic>),
    Patch(ProjectionPatchDiagnostics),
}

#[derive(Debug)]
pub(crate) struct TaskProjectionArtifact {
    pub(crate) canonical: CanonicalTaskProjection,
    pub(crate) patch: ProjectionPatch,
}

fn task_error(stage: ValidationStage, kind: TaskProjectionDiagnosticKind) -> TaskProjectionError {
    TaskProjectionError::Task(TaskProjectionDiagnostic::new(stage, kind))
}

fn checked_text_add(total: usize, value: usize) -> Result<usize, TaskProjectionError> {
    total.checked_add(value).ok_or_else(|| {
        task_error(
            ValidationStage::ResourcePreflight,
            TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        )
    })
}

fn checked_operation_add(total: usize, value: usize) -> Result<usize, TaskProjectionError> {
    total.checked_add(value).ok_or_else(|| {
        task_error(
            ValidationStage::OperationConstruction,
            TaskProjectionDiagnosticKind::OperationLimitExceeded,
        )
    })
}

const fn u64_text_len(mut val: u64) -> usize {
    if val == 0 {
        return 1;
    }
    let mut len = 0;
    while val > 0 {
        len += 1;
        val /= 10;
    }
    len
}

fn control_value_len(control: &TaskControlOffer) -> usize {
    let mut len = control.kind.token().len() + 1 + u64_text_len(control.action.raw());
    if let Some(token) = &control.resume_token {
        len += 1
            + u64_text_len(token.issuer())
            + 1
            + u64_text_len(token.namespace() as u64)
            + 1
            + u64_text_len(token.generation() as u64)
            + 1
            + u64_text_len(token.value());
    }
    len
}

fn control_value(control: &TaskControlOffer) -> String {
    match control.resume_token {
        Some(token) => alloc::format!(
            "{}:{}:{}:{}:{}:{}",
            control.kind.token(),
            control.action.raw(),
            token.issuer(),
            token.namespace(),
            token.generation(),
            token.value()
        ),
        None => alloc::format!("{}:{}", control.kind.token(), control.action.raw()),
    }
}

pub(crate) fn project_task_state(
    envelope: ProjectionPatchEnvelope,
    evidence: TaskProjectionEvidence,
    routes: TaskProjectionRoutes,
    limits: TaskProjectionLimits,
) -> Result<TaskProjectionArtifact, TaskProjectionError> {
    // 1. ResourcePreflight
    if evidence.phases.len() > limits.phase_count
        || evidence.controls.len() > limits.control_count
        || evidence.locks.len() > limits.scope_lock_count
    {
        return Err(task_error(
            ValidationStage::ResourcePreflight,
            TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        ));
    }

    let mut total_text_bytes = 0usize;
    total_text_bytes = checked_text_add(total_text_bytes, evidence.state.token().len())?;
    total_text_bytes = checked_text_add(total_text_bytes, evidence.freshness.token().len())?;
    let progress_len = match evidence.current_progress {
        TaskProgress::Indeterminate => 13,
        TaskProgress::Determinate { completed, total } => {
            u64_text_len(completed) + 1 + u64_text_len(total)
        }
    };
    total_text_bytes = checked_text_add(total_text_bytes, progress_len)?;

    for phase in &evidence.phases {
        if phase.label.len() > limits.phase_label_bytes {
            return Err(task_error(
                ValidationStage::ResourcePreflight,
                TaskProjectionDiagnosticKind::ResourceLimitExceeded,
            ));
        }
        let phase_len =
            u64_text_len(phase.id) + 1 + phase.status.token().len() + 1 + phase.label.len();
        total_text_bytes = checked_text_add(total_text_bytes, phase_len)?;
    }
    for control in &evidence.controls {
        total_text_bytes = checked_text_add(total_text_bytes, control_value_len(control))?;
    }
    if let Some(awaiting_input) = &evidence.awaiting_input {
        if awaiting_input.len() > limits.awaiting_input_bytes {
            return Err(task_error(
                ValidationStage::ResourcePreflight,
                TaskProjectionDiagnosticKind::ResourceLimitExceeded,
            ));
        }
        total_text_bytes = checked_text_add(total_text_bytes, awaiting_input.len())?;
    }
    for lock in &evidence.locks {
        if lock.explanation.len() > limits.lock_explanation_bytes {
            return Err(task_error(
                ValidationStage::ResourcePreflight,
                TaskProjectionDiagnosticKind::ResourceLimitExceeded,
            ));
        }
        let lock_len = u64_text_len(lock.reference.issuer())
            + 1
            + u64_text_len(lock.reference.namespace() as u64)
            + 1
            + u64_text_len(lock.reference.generation() as u64)
            + 1
            + u64_text_len(lock.reference.value())
            + 1
            + lock.explanation.len();
        total_text_bytes = checked_text_add(total_text_bytes, lock_len)?;
    }
    if total_text_bytes > limits.total_projected_text_bytes {
        return Err(task_error(
            ValidationStage::ResourcePreflight,
            TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        ));
    }

    // 2. RouteValidation
    let phase_collection = routes.phase_collection.ok_or_else(|| {
        task_error(
            ValidationStage::RouteValidation,
            TaskProjectionDiagnosticKind::MissingPhaseRoute,
        )
    })?;
    let control_collection = routes.control_collection.ok_or_else(|| {
        task_error(
            ValidationStage::RouteValidation,
            TaskProjectionDiagnosticKind::MissingControlRoute,
        )
    })?;
    let scope_lock_collection = routes.scope_lock_collection.ok_or_else(|| {
        task_error(
            ValidationStage::RouteValidation,
            TaskProjectionDiagnosticKind::MissingLockRoute,
        )
    })?;
    if evidence.state == TaskProjectionState::AwaitingInput && routes.awaiting_input_route.is_none()
    {
        return Err(task_error(
            ValidationStage::RouteValidation,
            TaskProjectionDiagnosticKind::MissingAwaitingInputRoute,
        ));
    }

    // 3. IdentityRevisionValidation
    if evidence.task_ref.raw() == 0 {
        return Err(task_error(
            ValidationStage::IdentityRevisionValidation,
            TaskProjectionDiagnosticKind::MissingTaskRef,
        ));
    }
    if evidence.new_revision.raw() <= evidence.previous_revision.raw() {
        return Err(task_error(
            ValidationStage::IdentityRevisionValidation,
            TaskProjectionDiagnosticKind::NonIncreasingTaskRevision,
        ));
    }

    // 4. StateValidation
    if evidence.state.requires_task_evidence()
        && evidence
            .task_evidence
            .is_none_or(|reference| reference.raw() == 0)
    {
        return Err(task_error(
            ValidationStage::StateValidation,
            TaskProjectionDiagnosticKind::MissingEvidenceRef,
        ));
    }
    match (&evidence.state, &evidence.awaiting_input) {
        (TaskProjectionState::AwaitingInput, Some(text)) if !text.trim().is_empty() => {}
        (TaskProjectionState::AwaitingInput, _) => {
            return Err(task_error(
                ValidationStage::StateValidation,
                TaskProjectionDiagnosticKind::MissingAwaitingInput,
            ));
        }
        (_, Some(_)) => {
            return Err(task_error(
                ValidationStage::StateValidation,
                TaskProjectionDiagnosticKind::UnexpectedAwaitingInput,
            ));
        }
        (_, None) => {}
    }
    if evidence.state == TaskProjectionState::Completed
        && !matches!(
            evidence.current_progress,
            TaskProgress::Determinate { completed, total }
                if total > 0 && completed == total
        )
    {
        return Err(task_error(
            ValidationStage::StateValidation,
            TaskProjectionDiagnosticKind::InvalidStateDetail,
        ));
    }

    // 5. PhaseValidation
    let mut phase_orders = BTreeSet::new();
    let mut phase_keys = BTreeSet::new();
    let mut duplicate_order = false;
    let mut duplicate_key = false;
    let mut active_count = 0usize;
    for phase in &evidence.phases {
        if phase.id == 0 {
            return Err(task_error(
                ValidationStage::PhaseValidation,
                TaskProjectionDiagnosticKind::InvalidPhaseSet,
            ));
        }
        duplicate_order |= !phase_orders.insert(phase.order);
        duplicate_key |= !phase_keys.insert(phase.key);
        if phase.status == TaskPhaseStatus::Active {
            active_count = active_count.checked_add(1).ok_or_else(|| {
                task_error(
                    ValidationStage::PhaseValidation,
                    TaskProjectionDiagnosticKind::InvalidPhaseSet,
                )
            })?;
        }
    }
    if duplicate_order {
        return Err(task_error(
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::DuplicatePhaseOrder,
        ));
    }
    if duplicate_key {
        return Err(task_error(
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::DuplicatePhaseKey,
        ));
    }
    let valid_active_count = if evidence.state.requires_active_phase() {
        active_count == 1
    } else {
        active_count == 0
    };
    if !valid_active_count {
        return Err(task_error(
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::InvalidPhaseSet,
        ));
    }

    // 6. ProgressValidation
    if let TaskProgress::Determinate { completed, total } = evidence.current_progress {
        if total == 0 || completed > total {
            return Err(task_error(
                ValidationStage::ProgressValidation,
                TaskProjectionDiagnosticKind::InvalidProgress,
            ));
        }
    }
    if let (
        Some(TaskProgress::Determinate {
            completed: previous,
            ..
        }),
        TaskProgress::Determinate {
            completed: current, ..
        },
    ) = (evidence.previous_progress, evidence.current_progress)
    {
        if current < previous
            && evidence
                .regression_evidence
                .is_none_or(|reference| reference.raw() == 0)
        {
            return Err(task_error(
                ValidationStage::ProgressValidation,
                TaskProjectionDiagnosticKind::ProgressRegressionWithoutEvidence,
            ));
        }
    }

    // 7. ControlValidation
    let mut control_orders = BTreeSet::new();
    let mut control_keys = BTreeSet::new();
    let mut duplicate_control_order = false;
    let mut duplicate_control_key = false;
    for control in &evidence.controls {
        duplicate_control_order |= !control_orders.insert(control.order);
        duplicate_control_key |= !control_keys.insert(control.key);
    }
    if duplicate_control_order {
        return Err(task_error(
            ValidationStage::ControlValidation,
            TaskProjectionDiagnosticKind::DuplicateControlOrder,
        ));
    }
    if duplicate_control_key {
        return Err(task_error(
            ValidationStage::ControlValidation,
            TaskProjectionDiagnosticKind::DuplicateControlKey,
        ));
    }
    for control in &evidence.controls {
        if control.action.raw() == 0 {
            return Err(task_error(
                ValidationStage::ControlValidation,
                TaskProjectionDiagnosticKind::ControlActionRefMissing,
            ));
        }
        if control.kind == TaskControlKind::Resume && control.resume_token.is_none() {
            return Err(task_error(
                ValidationStage::ControlValidation,
                TaskProjectionDiagnosticKind::ResumeTokenMissing,
            ));
        }
    }
    if evidence.freshness.restricts_critical_controls() && !evidence.controls.is_empty() {
        return Err(task_error(
            ValidationStage::ControlValidation,
            TaskProjectionDiagnosticKind::StaleControlOffer,
        ));
    }

    // 8. ScopeLockValidation
    let mut lock_orders = BTreeSet::new();
    let mut lock_keys = BTreeSet::new();
    let mut duplicate_lock_order = false;
    let mut duplicate_lock_key = false;
    for lock in &evidence.locks {
        duplicate_lock_order |= !lock_orders.insert(lock.order);
        duplicate_lock_key |= !lock_keys.insert(lock.key);
    }
    if duplicate_lock_order {
        return Err(task_error(
            ValidationStage::ScopeLockValidation,
            TaskProjectionDiagnosticKind::DuplicateLockOrder,
        ));
    }
    if duplicate_lock_key {
        return Err(task_error(
            ValidationStage::ScopeLockValidation,
            TaskProjectionDiagnosticKind::DuplicateLockKey,
        ));
    }
    if evidence
        .locks
        .iter()
        .any(|lock| lock.explanation.trim().is_empty())
    {
        return Err(task_error(
            ValidationStage::ScopeLockValidation,
            TaskProjectionDiagnosticKind::EmptyLockExplanation,
        ));
    }

    let mut canonical = CanonicalTaskProjection {
        task_ref: evidence.task_ref,
        previous_revision: evidence.previous_revision,
        new_revision: evidence.new_revision,
        state: evidence.state,
        phases: evidence.phases,
        current_progress: evidence.current_progress,
        previous_progress: evidence.previous_progress,
        regression_evidence: evidence.regression_evidence,
        awaiting_input: evidence.awaiting_input,
        task_evidence: evidence.task_evidence,
        freshness: evidence.freshness,
        controls: evidence.controls,
        locks: evidence.locks,
    };
    canonical.phases.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.key.cmp(&right.key))
            .then_with(|| left.id.cmp(&right.id))
    });
    canonical.controls.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.key.cmp(&right.key))
    });
    canonical.locks.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.key.cmp(&right.key))
    });

    // 9. OperationConstruction
    let freshness_fragment = project_freshness_fragment(
        canonical.freshness,
        &routes.freshness_route,
        limits.freshness_limits,
    )
    .map_err(TaskProjectionError::Freshness)?;

    let awaiting_operation_count = match (
        canonical.state == TaskProjectionState::AwaitingInput,
        routes.awaiting_input_route.is_some(),
    ) {
        (true, true) => 2,
        (false, true) => 1,
        (_, false) => 0,
    };
    let mut operation_count = 3usize;
    operation_count = checked_operation_add(operation_count, awaiting_operation_count)?;
    operation_count = checked_operation_add(operation_count, canonical.phases.len())?;
    operation_count = checked_operation_add(operation_count, canonical.controls.len())?;
    operation_count = checked_operation_add(operation_count, canonical.locks.len())?;
    operation_count =
        checked_operation_add(operation_count, freshness_fragment.operations().len())?;
    if operation_count > limits.total_operations {
        return Err(task_error(
            ValidationStage::OperationConstruction,
            TaskProjectionDiagnosticKind::OperationLimitExceeded,
        ));
    }

    let mut operations = Vec::with_capacity(operation_count);
    operations.push(ProjectionPatchOperation::SetBindingValue {
        target: routes.identity_route,
        value: ProjectionPatchValue::UnsignedScalar(canonical.task_ref.raw()),
    });
    operations.push(ProjectionPatchOperation::SetBindingValue {
        target: routes.state_route,
        value: ProjectionPatchValue::Text(String::from(canonical.state.token())),
    });
    let progress = match canonical.current_progress {
        TaskProgress::Indeterminate => String::from("indeterminate"),
        TaskProgress::Determinate { completed, total } => {
            alloc::format!("{completed}/{total}")
        }
    };
    operations.push(ProjectionPatchOperation::SetBindingValue {
        target: routes.progress_route,
        value: ProjectionPatchValue::Text(progress),
    });

    if let Some(route) = routes.awaiting_input_route {
        if let Some(text) = &canonical.awaiting_input {
            operations.push(ProjectionPatchOperation::SetBindingValue {
                target: route.clone(),
                value: ProjectionPatchValue::Text(text.clone()),
            });
            operations.push(ProjectionPatchOperation::SetNodeAvailability {
                node: route.node,
                availability: ProjectionNodeAvailability::Available,
            });
        } else {
            operations.push(ProjectionPatchOperation::SetNodeAvailability {
                node: route.node,
                availability: ProjectionNodeAvailability::Hidden,
            });
        }
    }

    for phase in &canonical.phases {
        operations.push(ProjectionPatchOperation::CollectionInsert {
            collection: phase_collection,
            key: phase.key,
            before: None,
            value: ProjectionPatchValue::Text(alloc::format!(
                "{}:{}:{}",
                phase.id,
                phase.status.token(),
                phase.label
            )),
        });
    }
    for control in &canonical.controls {
        operations.push(ProjectionPatchOperation::CollectionInsert {
            collection: control_collection,
            key: control.key,
            before: None,
            value: ProjectionPatchValue::Text(control_value(control)),
        });
    }
    for lock in &canonical.locks {
        operations.push(ProjectionPatchOperation::CollectionInsert {
            collection: scope_lock_collection,
            key: lock.key,
            before: None,
            value: ProjectionPatchValue::Text(alloc::format!(
                "{}:{}:{}:{}:{}",
                lock.reference.issuer(),
                lock.reference.namespace(),
                lock.reference.generation(),
                lock.reference.value(),
                lock.explanation
            )),
        });
    }
    operations.extend(freshness_fragment.into_operations());
    debug_assert_eq!(operations.len(), operation_count);

    // 10. PatchValidation
    let patch = ProjectionPatch::new(envelope, operations).map_err(TaskProjectionError::Patch)?;
    Ok(TaskProjectionArtifact { canonical, patch })
}
