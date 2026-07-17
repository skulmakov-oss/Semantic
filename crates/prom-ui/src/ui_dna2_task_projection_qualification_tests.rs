

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::binding_graph::{BindingSlot, BindingTarget};
use crate::connectivity_projection::{
    project_freshness_fragment, ControlCriticality, FreshnessControlRoute,
    FreshnessProjectionLimits, FreshnessProjectionRoute, FreshnessState,
    ProjectedControlAvailability,
};
use crate::contract_primitives::{CollectionKey, Epoch, Revision, StaticDocumentId, StaticNodeId};
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatch, ProjectionPatchEnvelope, ProjectionPatchId,
    ProjectionPatchOperation, ProjectionPatchSequence, ProjectionPatchStreamId,
    ProjectionPatchValue,
};
use crate::semantic_refs::{ReferenceToken, SemanticActionRef, SemanticEvidenceRef, TaskRecordRef};
use crate::task_projection::{
    project_task_state, TaskControlKind, TaskControlOffer, TaskPhase, TaskPhaseStatus,
    TaskProgress, TaskProjectionDiagnosticKind, TaskProjectionError, TaskProjectionEvidence,
    TaskProjectionLimits, TaskProjectionRoutes, TaskProjectionState, TaskScopeLock,
    ValidationStage,
};

fn node(raw: u64) -> StaticNodeId {
    StaticNodeId::new(raw).unwrap()
}

fn key(raw: u64) -> CollectionKey {
    CollectionKey::new(raw).unwrap()
}

fn target(node_id: u64) -> BindingTarget {
    BindingTarget {
        node: node(node_id),
        slot: BindingSlot(0),
    }
}

fn envelope() -> ProjectionPatchEnvelope {
    ProjectionPatchEnvelope::new(
        ProjectionPatchId::new(1).unwrap(),
        ProjectionPatchStreamId::new(2).unwrap(),
        StaticDocumentId::new(3).unwrap(),
        None,
        Revision::new(4),
        Revision::new(5),
        Epoch::new(6),
        ProjectionPatchSequence::new(7),
    )
}

fn limits() -> TaskProjectionLimits {
    TaskProjectionLimits {
        phase_count: 8,
        control_count: 8,
        scope_lock_count: 8,
        phase_label_bytes: 64,
        awaiting_input_bytes: 64,
        lock_explanation_bytes: 64,
        total_projected_text_bytes: 256,
        total_operations: 64,
        freshness_limits: FreshnessProjectionLimits::new(8, 16),
    }
}

fn routes() -> TaskProjectionRoutes {
    TaskProjectionRoutes {
        identity_route: target(1),
        state_route: target(2),
        progress_route: target(3),
        awaiting_input_route: Some(target(4)),
        freshness_route: FreshnessProjectionRoute::new(target(5), vec![]),
        phase_collection: Some(node(6)),
        control_collection: Some(node(7)),
        scope_lock_collection: Some(node(8)),
    }
}

fn phase(id: u64, order: u64, key_raw: u64, status: TaskPhaseStatus) -> TaskPhase {
    TaskPhase {
        id,
        key: key(key_raw),
        order,
        label: alloc::format!("phase-{id}"),
        status,
    }
}

fn control(
    order: u64,
    key_raw: u64,
    kind: TaskControlKind,
    action: u64,
    token: Option<ReferenceToken>,
) -> TaskControlOffer {
    TaskControlOffer {
        order,
        key: key(key_raw),
        kind,
        action: SemanticActionRef::new(action),
        resume_token: token,
    }
}

fn lock(order: u64, key_raw: u64, token_value: u64) -> TaskScopeLock {
    TaskScopeLock {
        order,
        key: key(key_raw),
        reference: ReferenceToken::new(1, 2, 3, token_value),
        explanation: alloc::format!("lock-{token_value}"),
    }
}

fn running_evidence() -> TaskProjectionEvidence {
    TaskProjectionEvidence {
        task_ref: TaskRecordRef::new(100),
        previous_revision: Revision::new(10),
        new_revision: Revision::new(11),
        state: TaskProjectionState::Running,
        phases: vec![phase(1, 1, 1, TaskPhaseStatus::Active)],
        current_progress: TaskProgress::Determinate {
            completed: 5,
            total: 10,
        },
        previous_progress: Some(TaskProgress::Determinate {
            completed: 4,
            total: 10,
        }),
        regression_evidence: None,
        awaiting_input: None,
        task_evidence: None,
        freshness: FreshnessState::Fresh,
        controls: vec![control(1, 1, TaskControlKind::Pause, 200, None)],
        locks: vec![lock(1, 1, 300)],
    }
}

fn pending_evidence() -> TaskProjectionEvidence {
    let mut evidence = running_evidence();
    evidence.state = TaskProjectionState::Pending;
    evidence.phases.clear();
    evidence.controls.clear();
    evidence.locks.clear();
    evidence.current_progress = TaskProgress::Indeterminate;
    evidence.previous_progress = None;
    evidence
}

fn terminal_evidence(state: TaskProjectionState) -> TaskProjectionEvidence {
    let mut evidence = pending_evidence();
    evidence.state = state;
    evidence.task_evidence = Some(SemanticEvidenceRef::new(900));
    if state == TaskProjectionState::Completed {
        evidence.current_progress = TaskProgress::Determinate {
            completed: 10,
            total: 10,
        };
    }
    evidence
}

fn assert_task_error(
    result: Result<crate::task_projection::TaskProjectionArtifact, TaskProjectionError>,
    stage: ValidationStage,
    kind: TaskProjectionDiagnosticKind,
) {
    assert!(matches!(
        result,
        Err(TaskProjectionError::Task(ref diagnostic))
            if diagnostic.stage == stage && diagnostic.kind == kind
    ));
}

#[test]
fn state_tokens_are_exact() {
    let states = [
        (TaskProjectionState::Pending, "pending"),
        (TaskProjectionState::Started, "started"),
        (TaskProjectionState::Running, "running"),
        (TaskProjectionState::AwaitingInput, "awaiting_input"),
        (TaskProjectionState::Paused, "paused"),
        (TaskProjectionState::Completing, "completing"),
        (TaskProjectionState::Completed, "completed"),
        (TaskProjectionState::Failed, "failed"),
        (TaskProjectionState::Denied, "denied"),
        (TaskProjectionState::Quarantined, "quarantined"),
        (TaskProjectionState::Cancelled, "cancelled"),
        (TaskProjectionState::PendingUnknown, "pending_unknown"),
    ];
    assert_eq!(states.len(), 12);
    for (state, token) in states {
        assert_eq!(state.token(), token);
    }
}

#[test]
fn diagnostic_codes_are_exact() {
    let kinds = [
        TaskProjectionDiagnosticKind::ResourceLimitExceeded,
        TaskProjectionDiagnosticKind::MissingTaskRef,
        TaskProjectionDiagnosticKind::NonIncreasingTaskRevision,
        TaskProjectionDiagnosticKind::MissingEvidenceRef,
        TaskProjectionDiagnosticKind::InvalidStateDetail,
        TaskProjectionDiagnosticKind::MissingAwaitingInput,
        TaskProjectionDiagnosticKind::UnexpectedAwaitingInput,
        TaskProjectionDiagnosticKind::DuplicatePhaseOrder,
        TaskProjectionDiagnosticKind::DuplicatePhaseKey,
        TaskProjectionDiagnosticKind::InvalidPhaseSet,
        TaskProjectionDiagnosticKind::InvalidProgress,
        TaskProjectionDiagnosticKind::ProgressRegressionWithoutEvidence,
        TaskProjectionDiagnosticKind::DuplicateControlOrder,
        TaskProjectionDiagnosticKind::DuplicateControlKey,
        TaskProjectionDiagnosticKind::ControlActionRefMissing,
        TaskProjectionDiagnosticKind::ResumeTokenMissing,
        TaskProjectionDiagnosticKind::StaleControlOffer,
        TaskProjectionDiagnosticKind::DuplicateLockOrder,
        TaskProjectionDiagnosticKind::DuplicateLockKey,
        TaskProjectionDiagnosticKind::EmptyLockExplanation,
        TaskProjectionDiagnosticKind::MissingPhaseRoute,
        TaskProjectionDiagnosticKind::MissingControlRoute,
        TaskProjectionDiagnosticKind::MissingLockRoute,
        TaskProjectionDiagnosticKind::MissingAwaitingInputRoute,
        TaskProjectionDiagnosticKind::OperationLimitExceeded,
        TaskProjectionDiagnosticKind::PatchRejected,
    ];
    let codes: Vec<_> = kinds.into_iter().map(|kind| kind.code()).collect();
    assert_eq!(codes.len(), 26);
    assert_eq!(codes[0], "TPP_RESOURCE_LIMIT_EXCEEDED");
    assert_eq!(codes[25], "TPP_PATCH_REJECTED");
}

#[test]
fn successful_projection_returns_canonical_artifact_and_one_nonempty_patch() {
    let artifact = project_task_state(envelope(), running_evidence(), routes(), limits()).unwrap();
    assert_eq!(artifact.canonical.state, TaskProjectionState::Running);
    assert_eq!(artifact.canonical.phases.len(), 1);
    assert!(!artifact.patch.operations().is_empty());
}

#[test]
fn identity_and_revision_rules_are_enforced() {
    let mut missing = running_evidence();
    missing.task_ref = TaskRecordRef::new(0);
    assert_task_error(
        project_task_state(envelope(), missing, routes(), limits()),
        ValidationStage::IdentityRevisionValidation,
        TaskProjectionDiagnosticKind::MissingTaskRef,
    );

    let mut revision = running_evidence();
    revision.new_revision = revision.previous_revision;
    assert_task_error(
        project_task_state(envelope(), revision, routes(), limits()),
        ValidationStage::IdentityRevisionValidation,
        TaskProjectionDiagnosticKind::NonIncreasingTaskRevision,
    );
}

#[test]
fn terminal_and_uncertain_states_require_nonzero_evidence() {
    for state in [
        TaskProjectionState::Completed,
        TaskProjectionState::Failed,
        TaskProjectionState::Denied,
        TaskProjectionState::Quarantined,
        TaskProjectionState::Cancelled,
        TaskProjectionState::PendingUnknown,
    ] {
        for task_evidence in [None, Some(SemanticEvidenceRef::new(0))] {
            let mut evidence = terminal_evidence(state);
            evidence.task_evidence = task_evidence;
            assert_task_error(
                project_task_state(envelope(), evidence, routes(), limits()),
                ValidationStage::StateValidation,
                TaskProjectionDiagnosticKind::MissingEvidenceRef,
            );
        }
    }
}

#[test]
fn nonterminal_states_do_not_require_task_evidence() {
    for state in [
        TaskProjectionState::Pending,
        TaskProjectionState::Started,
        TaskProjectionState::Running,
        TaskProjectionState::AwaitingInput,
        TaskProjectionState::Paused,
        TaskProjectionState::Completing,
    ] {
        let mut evidence = pending_evidence();
        evidence.state = state;
        if matches!(
            state,
            TaskProjectionState::Started
                | TaskProjectionState::Running
                | TaskProjectionState::AwaitingInput
                | TaskProjectionState::Paused
                | TaskProjectionState::Completing
        ) {
            evidence.phases = vec![phase(1, 1, 1, TaskPhaseStatus::Active)];
        }
        if state == TaskProjectionState::AwaitingInput {
            evidence.awaiting_input = Some(String::from("provide value"));
        }
        evidence.task_evidence = None;
        assert!(project_task_state(envelope(), evidence, routes(), limits()).is_ok());
    }
}

#[test]
fn awaiting_input_state_and_route_rules_are_enforced() {
    for text in [None, Some(String::new()), Some(String::from("   "))] {
        let mut evidence = running_evidence();
        evidence.state = TaskProjectionState::AwaitingInput;
        evidence.awaiting_input = text;
        assert_task_error(
            project_task_state(envelope(), evidence, routes(), limits()),
            ValidationStage::StateValidation,
            TaskProjectionDiagnosticKind::MissingAwaitingInput,
        );
    }

    let mut unexpected = running_evidence();
    unexpected.awaiting_input = Some(String::from("unexpected"));
    assert_task_error(
        project_task_state(envelope(), unexpected, routes(), limits()),
        ValidationStage::StateValidation,
        TaskProjectionDiagnosticKind::UnexpectedAwaitingInput,
    );

    let mut running_routes = routes();
    running_routes.awaiting_input_route = None;
    assert!(project_task_state(envelope(), running_evidence(), running_routes, limits()).is_ok());

    let mut awaiting = running_evidence();
    awaiting.state = TaskProjectionState::AwaitingInput;
    awaiting.awaiting_input = Some(String::from("provide value"));
    let mut missing_route = routes();
    missing_route.awaiting_input_route = None;
    assert_task_error(
        project_task_state(envelope(), awaiting, missing_route, limits()),
        ValidationStage::RouteValidation,
        TaskProjectionDiagnosticKind::MissingAwaitingInputRoute,
    );
}

#[test]
fn collection_routes_are_required() {
    let mut missing_phase = routes();
    missing_phase.phase_collection = None;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), missing_phase, limits()),
        ValidationStage::RouteValidation,
        TaskProjectionDiagnosticKind::MissingPhaseRoute,
    );

    let mut missing_control = routes();
    missing_control.control_collection = None;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), missing_control, limits()),
        ValidationStage::RouteValidation,
        TaskProjectionDiagnosticKind::MissingControlRoute,
    );

    let mut missing_lock = routes();
    missing_lock.scope_lock_collection = None;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), missing_lock, limits()),
        ValidationStage::RouteValidation,
        TaskProjectionDiagnosticKind::MissingLockRoute,
    );
}

#[test]
fn phase_structure_rules_are_enforced() {
    let mut duplicate_order = running_evidence();
    duplicate_order
        .phases
        .push(phase(2, 1, 2, TaskPhaseStatus::Pending));
    assert_task_error(
        project_task_state(envelope(), duplicate_order, routes(), limits()),
        ValidationStage::PhaseValidation,
        TaskProjectionDiagnosticKind::DuplicatePhaseOrder,
    );

    let mut duplicate_key = running_evidence();
    duplicate_key
        .phases
        .push(phase(2, 2, 1, TaskPhaseStatus::Pending));
    assert_task_error(
        project_task_state(envelope(), duplicate_key, routes(), limits()),
        ValidationStage::PhaseValidation,
        TaskProjectionDiagnosticKind::DuplicatePhaseKey,
    );

    let mut zero_id = running_evidence();
    zero_id.phases[0].id = 0;
    assert_task_error(
        project_task_state(envelope(), zero_id, routes(), limits()),
        ValidationStage::PhaseValidation,
        TaskProjectionDiagnosticKind::InvalidPhaseSet,
    );
}

#[test]
fn phase_state_consistency_is_enforced() {
    for state in [
        TaskProjectionState::Started,
        TaskProjectionState::Running,
        TaskProjectionState::AwaitingInput,
        TaskProjectionState::Paused,
        TaskProjectionState::Completing,
    ] {
        let mut none = running_evidence();
        none.state = state;
        none.phases.clear();
        if state == TaskProjectionState::AwaitingInput {
            none.awaiting_input = Some(String::from("provide value"));
        }
        assert_task_error(
            project_task_state(envelope(), none, routes(), limits()),
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::InvalidPhaseSet,
        );
    }

    for state in [
        TaskProjectionState::Pending,
        TaskProjectionState::Completed,
        TaskProjectionState::Failed,
        TaskProjectionState::Denied,
        TaskProjectionState::Quarantined,
        TaskProjectionState::Cancelled,
        TaskProjectionState::PendingUnknown,
    ] {
        let mut evidence = terminal_evidence(state);
        if state == TaskProjectionState::Pending {
            evidence.task_evidence = None;
        }
        evidence.phases = vec![phase(1, 1, 1, TaskPhaseStatus::Active)];
        assert_task_error(
            project_task_state(envelope(), evidence, routes(), limits()),
            ValidationStage::PhaseValidation,
            TaskProjectionDiagnosticKind::InvalidPhaseSet,
        );
    }
}

#[test]
fn progress_rules_are_enforced() {
    for progress in [
        TaskProgress::Determinate {
            completed: 0,
            total: 0,
        },
        TaskProgress::Determinate {
            completed: 11,
            total: 10,
        },
    ] {
        let mut evidence = running_evidence();
        evidence.current_progress = progress;
        assert_task_error(
            project_task_state(envelope(), evidence, routes(), limits()),
            ValidationStage::ProgressValidation,
            TaskProjectionDiagnosticKind::InvalidProgress,
        );
    }

    for progress in [
        TaskProgress::Indeterminate,
        TaskProgress::Determinate {
            completed: 9,
            total: 10,
        },
    ] {
        let mut evidence = terminal_evidence(TaskProjectionState::Completed);
        evidence.current_progress = progress;
        assert_task_error(
            project_task_state(envelope(), evidence, routes(), limits()),
            ValidationStage::StateValidation,
            TaskProjectionDiagnosticKind::InvalidStateDetail,
        );
    }

    for regression_evidence in [None, Some(SemanticEvidenceRef::new(0))] {
        let mut evidence = running_evidence();
        evidence.previous_progress = Some(TaskProgress::Determinate {
            completed: 7,
            total: 10,
        });
        evidence.current_progress = TaskProgress::Determinate {
            completed: 6,
            total: 10,
        };
        evidence.regression_evidence = regression_evidence;
        assert_task_error(
            project_task_state(envelope(), evidence, routes(), limits()),
            ValidationStage::ProgressValidation,
            TaskProjectionDiagnosticKind::ProgressRegressionWithoutEvidence,
        );
    }

    let mut accepted = running_evidence();
    accepted.previous_progress = Some(TaskProgress::Determinate {
        completed: 7,
        total: 10,
    });
    accepted.current_progress = TaskProgress::Determinate {
        completed: 6,
        total: 10,
    };
    accepted.regression_evidence = Some(SemanticEvidenceRef::new(501));
    assert!(project_task_state(envelope(), accepted, routes(), limits()).is_ok());
}

#[test]
fn all_control_kinds_are_accepted_when_fresh_or_degraded() {
    let controls = vec![
        control(1, 1, TaskControlKind::Pause, 101, None),
        control(
            2,
            2,
            TaskControlKind::Resume,
            102,
            Some(ReferenceToken::new(1, 1, 1, 1)),
        ),
        control(3, 3, TaskControlKind::Cancel, 103, None),
        control(4, 4, TaskControlKind::Retry, 104, None),
        control(5, 5, TaskControlKind::Acknowledge, 105, None),
        control(6, 6, TaskControlKind::ProvideInput, 106, None),
    ];
    for freshness in [FreshnessState::Fresh, FreshnessState::Degraded] {
        let mut evidence = running_evidence();
        evidence.controls = controls.clone();
        evidence.freshness = freshness;
        assert!(project_task_state(envelope(), evidence, routes(), limits()).is_ok());
    }
}

#[test]
fn control_rules_are_enforced() {
    for freshness in [
        FreshnessState::Stale,
        FreshnessState::Offline,
        FreshnessState::Resyncing,
    ] {
        let mut evidence = running_evidence();
        evidence.freshness = freshness;
        assert_task_error(
            project_task_state(envelope(), evidence, routes(), limits()),
            ValidationStage::ControlValidation,
            TaskProjectionDiagnosticKind::StaleControlOffer,
        );
    }

    let mut duplicate_order = running_evidence();
    duplicate_order
        .controls
        .push(control(1, 2, TaskControlKind::Cancel, 201, None));
    assert_task_error(
        project_task_state(envelope(), duplicate_order, routes(), limits()),
        ValidationStage::ControlValidation,
        TaskProjectionDiagnosticKind::DuplicateControlOrder,
    );

    let mut duplicate_key = running_evidence();
    duplicate_key
        .controls
        .push(control(2, 1, TaskControlKind::Cancel, 201, None));
    assert_task_error(
        project_task_state(envelope(), duplicate_key, routes(), limits()),
        ValidationStage::ControlValidation,
        TaskProjectionDiagnosticKind::DuplicateControlKey,
    );

    let mut zero_action = running_evidence();
    zero_action.controls[0].action = SemanticActionRef::new(0);
    assert_task_error(
        project_task_state(envelope(), zero_action, routes(), limits()),
        ValidationStage::ControlValidation,
        TaskProjectionDiagnosticKind::ControlActionRefMissing,
    );

    let mut resume = running_evidence();
    resume.controls = vec![control(1, 1, TaskControlKind::Resume, 200, None)];
    assert_task_error(
        project_task_state(envelope(), resume, routes(), limits()),
        ValidationStage::ControlValidation,
        TaskProjectionDiagnosticKind::ResumeTokenMissing,
    );
}

#[test]
fn resume_token_is_retained_in_canonical_and_patch_output() {
    let project = |token_value| {
        let mut evidence = running_evidence();
        evidence.controls = vec![control(
            1,
            1,
            TaskControlKind::Resume,
            200,
            Some(ReferenceToken::new(1, 2, 3, token_value)),
        )];
        project_task_state(envelope(), evidence, routes(), limits()).unwrap()
    };
    let first = project(10);
    let second = project(11);
    assert_ne!(first.canonical.controls, second.canonical.controls);
    assert_ne!(first.patch.operations(), second.patch.operations());
}

#[test]
fn lock_rules_are_enforced() {
    let mut duplicate_order = running_evidence();
    duplicate_order.locks.push(lock(1, 2, 301));
    assert_task_error(
        project_task_state(envelope(), duplicate_order, routes(), limits()),
        ValidationStage::ScopeLockValidation,
        TaskProjectionDiagnosticKind::DuplicateLockOrder,
    );

    let mut duplicate_key = running_evidence();
    duplicate_key.locks.push(lock(2, 1, 301));
    assert_task_error(
        project_task_state(envelope(), duplicate_key, routes(), limits()),
        ValidationStage::ScopeLockValidation,
        TaskProjectionDiagnosticKind::DuplicateLockKey,
    );

    let mut empty = running_evidence();
    empty.locks[0].explanation = String::from("   ");
    assert_task_error(
        project_task_state(envelope(), empty, routes(), limits()),
        ValidationStage::ScopeLockValidation,
        TaskProjectionDiagnosticKind::EmptyLockExplanation,
    );
}

#[test]
fn caller_supplied_limits_are_enforced() {
    let mut phase_limits = limits();
    phase_limits.phase_count = 0;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), routes(), phase_limits),
        ValidationStage::ResourcePreflight,
        TaskProjectionDiagnosticKind::ResourceLimitExceeded,
    );

    let mut control_limits = limits();
    control_limits.control_count = 0;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), routes(), control_limits),
        ValidationStage::ResourcePreflight,
        TaskProjectionDiagnosticKind::ResourceLimitExceeded,
    );

    let mut lock_limits = limits();
    lock_limits.scope_lock_count = 0;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), routes(), lock_limits),
        ValidationStage::ResourcePreflight,
        TaskProjectionDiagnosticKind::ResourceLimitExceeded,
    );

    let mut text_limits = limits();
    text_limits.total_projected_text_bytes = 1;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), routes(), text_limits),
        ValidationStage::ResourcePreflight,
        TaskProjectionDiagnosticKind::ResourceLimitExceeded,
    );

    let mut operation_limits = limits();
    operation_limits.total_operations = 1;
    assert_task_error(
        project_task_state(envelope(), running_evidence(), routes(), operation_limits),
        ValidationStage::OperationConstruction,
        TaskProjectionDiagnosticKind::OperationLimitExceeded,
    );
}

fn permutation_evidence() -> TaskProjectionEvidence {
    let mut evidence = running_evidence();
    evidence.phases = vec![
        phase(3, 30, 3, TaskPhaseStatus::Pending),
        phase(1, 10, 1, TaskPhaseStatus::Active),
        phase(2, 20, 2, TaskPhaseStatus::Completed),
    ];
    evidence.controls = vec![
        control(30, 3, TaskControlKind::Cancel, 303, None),
        control(10, 1, TaskControlKind::Pause, 301, None),
        control(20, 2, TaskControlKind::Retry, 302, None),
    ];
    evidence.locks = vec![lock(30, 3, 303), lock(10, 1, 301), lock(20, 2, 302)];
    evidence
}

#[test]
fn input_permutation_does_not_change_canonical_or_patch_order() {
    let first_evidence = permutation_evidence();
    let mut second_evidence = first_evidence.clone();
    second_evidence.phases.reverse();
    second_evidence.controls.rotate_left(1);
    second_evidence.locks.swap(0, 2);

    let first = project_task_state(envelope(), first_evidence, routes(), limits()).unwrap();
    let second = project_task_state(envelope(), second_evidence, routes(), limits()).unwrap();
    assert_eq!(first.canonical, second.canonical);
    assert_eq!(first.patch.operations(), second.patch.operations());
}

#[test]
fn freshness_diagnostics_are_preserved_exactly() {
    let duplicate = FreshnessControlRoute::new(
        node(20),
        ControlCriticality::Normal,
        ProjectedControlAvailability::Available,
    );
    let freshness_route = FreshnessProjectionRoute::new(target(5), vec![duplicate, duplicate]);
    let freshness_limits = FreshnessProjectionLimits::new(8, 16);
    let expected =
        project_freshness_fragment(FreshnessState::Fresh, &freshness_route, freshness_limits)
            .unwrap_err();

    let mut routes = routes();
    routes.freshness_route = freshness_route;
    let mut limits = limits();
    limits.freshness_limits = freshness_limits;
    let actual = project_task_state(envelope(), running_evidence(), routes, limits).unwrap_err();
    assert_eq!(actual, TaskProjectionError::Freshness(expected));
}

#[test]
fn projection_patch_diagnostics_are_preserved_exactly() {
    let evidence = pending_evidence();
    let mut routes = routes();
    routes.awaiting_input_route = None;
    routes.state_route = routes.identity_route.clone();

    let expected_operations = vec![
        ProjectionPatchOperation::SetBindingValue {
            target: routes.identity_route.clone(),
            value: ProjectionPatchValue::SignedScalar(evidence.task_ref.raw() as i64),
        },
        ProjectionPatchOperation::SetBindingValue {
            target: routes.state_route.clone(),
            value: ProjectionPatchValue::Text(String::from(evidence.state.token())),
        },
        ProjectionPatchOperation::SetBindingValue {
            target: routes.progress_route.clone(),
            value: ProjectionPatchValue::Text(String::from("indeterminate")),
        },
        ProjectionPatchOperation::SetBindingValue {
            target: routes.freshness_route.freshness_target().clone(),
            value: ProjectionPatchValue::Text(String::from("fresh")),
        },
    ];
    let expected = ProjectionPatch::new(envelope(), expected_operations).unwrap_err();
    let actual = project_task_state(envelope(), evidence, routes, limits()).unwrap_err();
    assert_eq!(actual, TaskProjectionError::Patch(expected));
}

#[test]
fn awaiting_input_projection_sets_value_and_availability() {
    let mut evidence = running_evidence();
    evidence.state = TaskProjectionState::AwaitingInput;
    evidence.awaiting_input = Some(String::from("provide value"));
    let artifact = project_task_state(envelope(), evidence, routes(), limits()).unwrap();
    assert!(artifact.patch.operations().iter().any(|operation| matches!(
        operation,
        ProjectionPatchOperation::SetBindingValue {
            target: binding,
            value: ProjectionPatchValue::Text(text),
        } if binding.node == node(4) && text == "provide value"
    )));
    assert!(artifact.patch.operations().iter().any(|operation| matches!(
        operation,
        ProjectionPatchOperation::SetNodeAvailability {
            node: target_node,
            availability: ProjectionNodeAvailability::Available,
        } if *target_node == node(4)
    )));
}
