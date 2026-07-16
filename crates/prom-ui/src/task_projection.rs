//! Crate-private Task Projection v0 contract execution.
//!
//! Evaluates task projection boundaries, freshness integration, and limits
//! before generating deterministic inert operations to apply.
#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::binding_graph::BindingTarget;
use crate::connectivity_projection::{
    project_freshness_fragment, FreshnessProjectionDiagnostic, FreshnessProjectionLimits,
    FreshnessProjectionRoute, FreshnessState,
};
use crate::contract_primitives::{CollectionKey, Revision, StaticNodeId};
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatch, ProjectionPatchEnvelope, ProjectionPatchOperation,
    ProjectionPatchValue,
};
use crate::semantic_refs::{SemanticActionRef, SemanticEvidenceRef, TaskRecordRef};

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

    const fn is_terminal_or_uncertain(self) -> bool {
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

    const fn requires_evidence(self) -> bool {
        self.is_terminal_or_uncertain()
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
    pub(crate) resume_token: Option<crate::semantic_refs::ReferenceToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskScopeLock {
    pub(crate) order: u64,
    pub(crate) key: CollectionKey,
    pub(crate) reference: crate::semantic_refs::ReferenceToken,
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

#[derive(Debug)]
pub(crate) enum TaskProjectionError {
    Task(TaskProjectionDiagnostic),
    Freshness(Vec<FreshnessProjectionDiagnostic>),
    Patch(crate::projection_patch::ProjectionPatchDiagnostics),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskProjectionDiagnostic {
    pub(crate) stage: ValidationStage,
    pub(crate) kind: TaskProjectionDiagnosticKind,
}

impl TaskProjectionDiagnostic {
    pub(crate) const fn new(stage: ValidationStage, kind: TaskProjectionDiagnosticKind) -> Self {
        Self { stage, kind }
    }
}

#[derive(Debug)]
pub(crate) struct TaskProjectionArtifact {
    pub(crate) patch: ProjectionPatch,
}

pub(crate) fn project_task_state(
    envelope: ProjectionPatchEnvelope,
    evidence: TaskProjectionEvidence,
    routes: TaskProjectionRoutes,
    limits: TaskProjectionLimits,
) -> Result<TaskProjectionArtifact, TaskProjectionError> {
    // 1. ResourcePreflight
    let mut total_text_bytes = 0;
    if evidence.phases.len() > limits.phase_count {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::ResourcePreflight,
            TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        )));
    }
    if evidence.controls.len() > limits.control_count {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::ResourcePreflight,
            TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        )));
    }
    if evidence.locks.len() > limits.scope_lock_count {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::ResourcePreflight,
            TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        )));
    }
    for phase in &evidence.phases {
        total_text_bytes += phase.label.len();
        if phase.label.len() > limits.phase_label_bytes {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ResourcePreflight,
                TaskProjectionDiagnosticKind::ResourceLimitExceeded,
            )));
        }
    }
    if let Some(aw_in) = &evidence.awaiting_input {
        total_text_bytes += aw_in.len();
        if aw_in.len() > limits.awaiting_input_bytes {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ResourcePreflight,
                TaskProjectionDiagnosticKind::ResourceLimitExceeded,
            )));
        }
    }
    for lock in &evidence.locks {
        total_text_bytes += lock.explanation.len();
        if lock.explanation.len() > limits.lock_explanation_bytes {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ResourcePreflight,
                TaskProjectionDiagnosticKind::ResourceLimitExceeded,
            )));
        }
    }
    if total_text_bytes > limits.total_projected_text_bytes {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::ResourcePreflight,
            TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        )));
    }

    // 2. RouteValidation
    let phase_collection = match routes.phase_collection {
        Some(col) => col,
        None => {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::RouteValidation,
                TaskProjectionDiagnosticKind::MissingPhaseRoute,
            )));
        }
    };
    let control_collection = match routes.control_collection {
        Some(col) => col,
        None => {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::RouteValidation,
                TaskProjectionDiagnosticKind::MissingControlRoute,
            )));
        }
    };
    let scope_lock_collection = match routes.scope_lock_collection {
        Some(col) => col,
        None => {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::RouteValidation,
                TaskProjectionDiagnosticKind::MissingLockRoute,
            )));
        }
    };
    let awaiting_input_route = match routes.awaiting_input_route {
        Some(route) => route,
        None => {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::RouteValidation,
                TaskProjectionDiagnosticKind::MissingAwaitingInputRoute,
            )));
        }
    };

    // 3. IdentityRevisionValidation
    if evidence.task_ref.raw() == 0 {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::IdentityRevisionValidation,
            TaskProjectionDiagnosticKind::MissingTaskRef,
        )));
    }
    if evidence.new_revision.raw() <= evidence.previous_revision.raw() {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::IdentityRevisionValidation,
            TaskProjectionDiagnosticKind::NonIncreasingTaskRevision,
        )));
    }

    // 4. StateValidation
    if evidence.state.requires_evidence() && evidence.task_evidence.map_or(0, |e| e.raw()) == 0 {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::StateValidation,
            TaskProjectionDiagnosticKind::MissingEvidenceRef,
        )));
    }
    if evidence.state == TaskProjectionState::AwaitingInput && evidence.awaiting_input.is_none() {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::StateValidation,
            TaskProjectionDiagnosticKind::MissingAwaitingInput,
        )));
    }
    if evidence.state != TaskProjectionState::AwaitingInput && evidence.awaiting_input.is_some() {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::StateValidation,
            TaskProjectionDiagnosticKind::UnexpectedAwaitingInput,
        )));
    }
    if evidence.state == TaskProjectionState::Completed {
        if let TaskProgress::Determinate { completed, total } = evidence.current_progress {
            if completed != total {
                return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                    ValidationStage::StateValidation,
                    TaskProjectionDiagnosticKind::InvalidStateDetail,
                )));
            }
        }
    }

    // 5. PhaseValidation
    let mut phase_orders = Vec::new();
    let mut phase_keys = Vec::new();
    let mut active_count = 0;
    for phase in &evidence.phases {
        if phase.id == 0 {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::PhaseValidation,
                TaskProjectionDiagnosticKind::InvalidPhaseSet,
            )));
        }
        if phase_orders.contains(&phase.order) {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::PhaseValidation,
                TaskProjectionDiagnosticKind::DuplicatePhaseOrder,
            )));
        }
        phase_orders.push(phase.order);

        if phase_keys.contains(&phase.key) {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::PhaseValidation,
                TaskProjectionDiagnosticKind::DuplicatePhaseKey,
            )));
        }
        phase_keys.push(phase.key);

        if phase.status == TaskPhaseStatus::Active {
            active_count += 1;
        }
    }
    if active_count > 1 {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::InvalidPhaseSet,
        )));
    }
    if evidence.state.is_terminal_or_uncertain() && active_count > 0 {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::InvalidPhaseSet,
        )));
    }
    if matches!(
        evidence.state,
        TaskProjectionState::Started
            | TaskProjectionState::Running
            | TaskProjectionState::AwaitingInput
            | TaskProjectionState::Paused
            | TaskProjectionState::Completing
    ) && active_count != 1
    {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::InvalidPhaseSet,
        )));
    }

    // 6. ProgressValidation
    if let TaskProgress::Determinate { completed, total } = evidence.current_progress {
        if total == 0 || completed > total {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ProgressValidation,
                TaskProjectionDiagnosticKind::InvalidProgress,
            )));
        }
        if let Some(TaskProgress::Determinate {
            completed: prev_comp,
            ..
        }) = evidence.previous_progress
        {
            if completed < prev_comp && evidence.regression_evidence.is_none() {
                return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                    ValidationStage::ProgressValidation,
                    TaskProjectionDiagnosticKind::ProgressRegressionWithoutEvidence,
                )));
            }
        }
    }

    // 7. ControlValidation
    let mut control_orders = Vec::new();
    let mut control_keys = Vec::new();
    let restricts_controls = matches!(
        evidence.freshness,
        FreshnessState::Stale | FreshnessState::Offline | FreshnessState::Resyncing
    );
    if restricts_controls && !evidence.controls.is_empty() {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::ControlValidation,
            TaskProjectionDiagnosticKind::StaleControlOffer,
        )));
    }
    for control in &evidence.controls {
        if control_orders.contains(&control.order) {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ControlValidation,
                TaskProjectionDiagnosticKind::DuplicateControlOrder,
            )));
        }
        control_orders.push(control.order);

        if control_keys.contains(&control.key) {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ControlValidation,
                TaskProjectionDiagnosticKind::DuplicateControlKey,
            )));
        }
        control_keys.push(control.key);

        if control.action.raw() == 0 {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ControlValidation,
                TaskProjectionDiagnosticKind::ControlActionRefMissing,
            )));
        }

        if control.kind == TaskControlKind::Resume && control.resume_token.is_none() {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ControlValidation,
                TaskProjectionDiagnosticKind::ResumeTokenMissing,
            )));
        }
    }

    // 8. ScopeLockValidation
    let mut lock_orders = Vec::new();
    let mut lock_keys = Vec::new();
    for lock in &evidence.locks {
        if lock_orders.contains(&lock.order) {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ScopeLockValidation,
                TaskProjectionDiagnosticKind::DuplicateLockOrder,
            )));
        }
        lock_orders.push(lock.order);

        if lock_keys.contains(&lock.key) {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ScopeLockValidation,
                TaskProjectionDiagnosticKind::DuplicateLockKey,
            )));
        }
        lock_keys.push(lock.key);

        if lock.explanation.is_empty() {
            return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
                ValidationStage::ScopeLockValidation,
                TaskProjectionDiagnosticKind::EmptyLockExplanation,
            )));
        }
    }

    // 9. OperationConstruction
    let freshness_frag = match project_freshness_fragment(
        evidence.freshness,
        &routes.freshness_route,
        limits.freshness_limits,
    ) {
        Ok(frag) => frag,
        Err(diags) => {
            return Err(TaskProjectionError::Freshness(diags));
        }
    };

    let mut ops = Vec::new();

    // Identity, state, progress
    ops.push(ProjectionPatchOperation::SetBindingValue {
        target: routes.identity_route,
        value: ProjectionPatchValue::SignedScalar(evidence.task_ref.raw() as i64),
    });
    ops.push(ProjectionPatchOperation::SetBindingValue {
        target: routes.state_route,
        value: ProjectionPatchValue::Text(String::from(evidence.state.token())),
    });

    let progress_val = match evidence.current_progress {
        TaskProgress::Indeterminate => String::from("indeterminate"),
        TaskProgress::Determinate { completed, total } => {
            alloc::format!("{}/{}", completed, total)
        }
    };
    ops.push(ProjectionPatchOperation::SetBindingValue {
        target: routes.progress_route,
        value: ProjectionPatchValue::Text(progress_val),
    });

    if let Some(aw_in) = evidence.awaiting_input {
        ops.push(ProjectionPatchOperation::SetBindingValue {
            target: awaiting_input_route.clone(),
            value: ProjectionPatchValue::Text(aw_in),
        });
        ops.push(ProjectionPatchOperation::SetNodeAvailability {
            node: awaiting_input_route.node,
            availability: ProjectionNodeAvailability::Available,
        });
    } else {
        ops.push(ProjectionPatchOperation::SetNodeAvailability {
            node: awaiting_input_route.node,
            availability: ProjectionNodeAvailability::Hidden,
        });
    }

    // Phases
    for phase in evidence.phases {
        ops.push(ProjectionPatchOperation::CollectionInsert {
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

    // Controls
    for control in evidence.controls {
        ops.push(ProjectionPatchOperation::CollectionInsert {
            collection: control_collection,
            key: control.key,
            before: None,
            value: ProjectionPatchValue::Text(alloc::format!(
                "{}:{}",
                control.kind.token(),
                control.action.raw()
            )),
        });
    }

    // Scope locks
    for lock in evidence.locks {
        ops.push(ProjectionPatchOperation::CollectionInsert {
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

    // Compose freshness operations
    ops.extend(freshness_frag.into_operations());

    if ops.len() > limits.total_operations {
        return Err(TaskProjectionError::Task(TaskProjectionDiagnostic::new(
            ValidationStage::OperationConstruction,
            TaskProjectionDiagnosticKind::OperationLimitExceeded,
        )));
    }

    // 10. PatchValidation
    match ProjectionPatch::new(envelope, ops) {
        Ok(patch) => Ok(TaskProjectionArtifact { patch }),
        Err(diags) => Err(TaskProjectionError::Patch(diags)),
    }
}
