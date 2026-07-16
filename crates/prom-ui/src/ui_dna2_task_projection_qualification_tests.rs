#![cfg(test)]
#![allow(unused_imports)]

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::binding_graph::{BindingSlot, BindingTarget};
use crate::connectivity_projection::{
    FreshnessProjectionLimits, FreshnessProjectionRoute, FreshnessState,
};
use crate::contract_primitives::{CollectionKey, Epoch, Revision, StaticDocumentId, StaticNodeId};
use crate::projection_patch::{
    ProjectionPatchEnvelope, ProjectionPatchId, ProjectionPatchOperation, ProjectionPatchSequence,
    ProjectionPatchStreamId,
};
use crate::semantic_refs::{ReferenceToken, SemanticActionRef, SemanticEvidenceRef, TaskRecordRef};
use crate::task_projection::{
    project_task_state, TaskControlKind, TaskControlOffer, TaskPhase, TaskPhaseStatus,
    TaskProgress, TaskProjectionDiagnosticKind, TaskProjectionEvidence, TaskProjectionLimits,
    TaskProjectionRoutes, TaskProjectionState, TaskScopeLock, ValidationStage,
};

fn default_limits() -> TaskProjectionLimits {
    TaskProjectionLimits {
        phase_count: 5,
        control_count: 5,
        scope_lock_count: 5,
        phase_label_bytes: 100,
        awaiting_input_bytes: 100,
        lock_explanation_bytes: 100,
        total_projected_text_bytes: 500,
        total_operations: 50,
        freshness_limits: FreshnessProjectionLimits::new(10, 20),
    }
}

fn default_routes() -> TaskProjectionRoutes {
    TaskProjectionRoutes {
        identity_route: BindingTarget {
            node: StaticNodeId::new(1).unwrap(),
            slot: BindingSlot(0),
        },
        state_route: BindingTarget {
            node: StaticNodeId::new(2).unwrap(),
            slot: BindingSlot(0),
        },
        progress_route: BindingTarget {
            node: StaticNodeId::new(3).unwrap(),
            slot: BindingSlot(0),
        },
        awaiting_input_route: Some(BindingTarget {
            node: StaticNodeId::new(4).unwrap(),
            slot: BindingSlot(0),
        }),
        freshness_route: FreshnessProjectionRoute::new(
            BindingTarget {
                node: StaticNodeId::new(5).unwrap(),
                slot: BindingSlot(0),
            },
            vec![],
        ),
        phase_collection: Some(StaticNodeId::new(6).unwrap()),
        control_collection: Some(StaticNodeId::new(7).unwrap()),
        scope_lock_collection: Some(StaticNodeId::new(8).unwrap()),
    }
}

fn default_envelope() -> ProjectionPatchEnvelope {
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

fn default_evidence() -> TaskProjectionEvidence {
    TaskProjectionEvidence {
        task_ref: TaskRecordRef::new(100),
        previous_revision: Revision::new(10),
        new_revision: Revision::new(11),
        state: TaskProjectionState::Running,
        phases: vec![TaskPhase {
            id: 1,
            key: CollectionKey::new(1).unwrap(),
            order: 1,
            label: String::from("Phase 1"),
            status: TaskPhaseStatus::Active,
        }],
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
        controls: vec![TaskControlOffer {
            order: 1,
            key: CollectionKey::new(1).unwrap(),
            kind: TaskControlKind::Pause,
            action: SemanticActionRef::new(200),
            resume_token: None,
        }],
        locks: vec![TaskScopeLock {
            order: 1,
            key: CollectionKey::new(1).unwrap(),
            reference: ReferenceToken::new(1, 1, 1, 300),
            explanation: String::from("Lock 1"),
        }],
    }
}

#[test]
fn test_successful_projection() {
    let result = project_task_state(
        default_envelope(),
        default_evidence(),
        default_routes(),
        default_limits(),
    );
    assert!(result.is_ok());
}

#[test]
fn test_resource_preflight_limit() {
    let mut limits = default_limits();
    limits.phase_count = 0;
    let result = project_task_state(
        default_envelope(),
        default_evidence(),
        default_routes(),
        limits,
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::ResourcePreflight && diag.kind == TaskProjectionDiagnosticKind::ResourceLimitExceeded
    ));
}

#[test]
fn test_route_validation_missing_phase() {
    let mut routes = default_routes();
    routes.phase_collection = None;
    let result = project_task_state(
        default_envelope(),
        default_evidence(),
        routes,
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::RouteValidation && diag.kind == TaskProjectionDiagnosticKind::MissingPhaseRoute
    ));
}

#[test]
fn test_identity_revision_validation_non_increasing() {
    let mut evidence = default_evidence();
    evidence.new_revision = Revision::new(10);
    evidence.previous_revision = Revision::new(10);
    let result = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::IdentityRevisionValidation && diag.kind == TaskProjectionDiagnosticKind::NonIncreasingTaskRevision
    ));
}

#[test]
fn test_state_validation_missing_awaiting_input() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::AwaitingInput;
    evidence.awaiting_input = None;
    let result = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::StateValidation && diag.kind == TaskProjectionDiagnosticKind::MissingAwaitingInput
    ));
}

#[test]
fn test_state_validation_completed_progress_mismatch() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::Completed;
    evidence.task_evidence = Some(SemanticEvidenceRef::new(999));
    evidence.phases.clear();
    evidence.current_progress = TaskProgress::Determinate {
        completed: 5,
        total: 10,
    };
    let result = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::StateValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidStateDetail
    ));
}

#[test]
fn test_phase_validation_duplicate_order() {
    let mut evidence = default_evidence();
    evidence.phases.push(TaskPhase {
        id: 2,
        key: CollectionKey::new(2).unwrap(),
        order: 1,
        label: String::from("Phase 2"),
        status: TaskPhaseStatus::Pending,
    });
    let result = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::DuplicatePhaseOrder
    ));
}

#[test]
fn test_progress_validation_regression() {
    let mut evidence = default_evidence();
    evidence.current_progress = TaskProgress::Determinate {
        completed: 3,
        total: 10,
    };
    evidence.previous_progress = Some(TaskProgress::Determinate {
        completed: 4,
        total: 10,
    });
    let result = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::ProgressValidation && diag.kind == TaskProjectionDiagnosticKind::ProgressRegressionWithoutEvidence
    ));
}

#[test]
fn test_control_validation_stale_offer() {
    let mut evidence = default_evidence();
    evidence.freshness = FreshnessState::Stale;
    let result = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::ControlValidation && diag.kind == TaskProjectionDiagnosticKind::StaleControlOffer
    ));
}

#[test]
fn test_scope_lock_validation_empty_explanation() {
    let mut evidence = default_evidence();
    evidence.locks[0].explanation = String::new();
    let result = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    );
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::ScopeLockValidation && diag.kind == TaskProjectionDiagnosticKind::EmptyLockExplanation
    ));
}

#[test]
fn test_patch_rejected() {
    let evidence = default_evidence();
    let mut routes = default_routes();
    // Force DuplicateMutationTarget by making two routes identical
    routes.state_route = routes.identity_route.clone();

    let expected_routes = routes.clone();

    // 1. Build the exact operation set that Task Projection will construct
    let mut expected_ops = Vec::new();
    expected_ops.push(ProjectionPatchOperation::SetBindingValue {
        target: expected_routes.identity_route.clone(),
        value: crate::projection_patch::ProjectionPatchValue::SignedScalar(
            evidence.task_ref.raw() as i64
        ),
    });
    expected_ops.push(ProjectionPatchOperation::SetBindingValue {
        target: expected_routes.state_route.clone(),
        value: crate::projection_patch::ProjectionPatchValue::Text(String::from(
            evidence.state.token(),
        )),
    });
    let progress_val = match evidence.current_progress {
        TaskProgress::Indeterminate => String::from("indeterminate"),
        TaskProgress::Determinate { completed, total } => {
            alloc::format!("{}/{}", completed, total)
        }
    };
    expected_ops.push(ProjectionPatchOperation::SetBindingValue {
        target: expected_routes.progress_route.clone(),
        value: crate::projection_patch::ProjectionPatchValue::Text(progress_val),
    });
    if let Some(aw_in) = evidence.awaiting_input.clone() {
        expected_ops.push(ProjectionPatchOperation::SetBindingValue {
            target: expected_routes.awaiting_input_route.clone().unwrap(),
            value: crate::projection_patch::ProjectionPatchValue::Text(aw_in),
        });
        expected_ops.push(ProjectionPatchOperation::SetNodeAvailability {
            node: expected_routes.awaiting_input_route.unwrap().node,
            availability: crate::projection_patch::ProjectionNodeAvailability::Available,
        });
    } else {
        expected_ops.push(ProjectionPatchOperation::SetNodeAvailability {
            node: expected_routes.awaiting_input_route.unwrap().node,
            availability: crate::projection_patch::ProjectionNodeAvailability::Hidden,
        });
    }
    for phase in evidence.phases.clone() {
        expected_ops.push(ProjectionPatchOperation::CollectionInsert {
            collection: expected_routes.phase_collection.unwrap(),
            key: phase.key,
            before: None,
            value: crate::projection_patch::ProjectionPatchValue::Text(alloc::format!(
                "{}:{}:{}",
                phase.id,
                phase.status.token(),
                phase.label
            )),
        });
    }
    for control in evidence.controls.clone() {
        expected_ops.push(ProjectionPatchOperation::CollectionInsert {
            collection: expected_routes.control_collection.unwrap(),
            key: control.key,
            before: None,
            value: crate::projection_patch::ProjectionPatchValue::Text(alloc::format!(
                "{}:{}",
                control.kind.token(),
                control.action.raw()
            )),
        });
    }
    for lock in evidence.locks.clone() {
        expected_ops.push(ProjectionPatchOperation::CollectionInsert {
            collection: expected_routes.scope_lock_collection.unwrap(),
            key: lock.key,
            before: None,
            value: crate::projection_patch::ProjectionPatchValue::Text(alloc::format!(
                "{}:{}:{}:{}:{}",
                lock.reference.issuer(),
                lock.reference.namespace(),
                lock.reference.generation(),
                lock.reference.value(),
                lock.explanation
            )),
        });
    }
    let freshness_frag = crate::connectivity_projection::project_freshness_fragment(
        evidence.freshness,
        &expected_routes.freshness_route,
        default_limits().freshness_limits,
    )
    .unwrap();
    expected_ops.extend(freshness_frag.into_operations());

    // Validate directly through ProjectionPatch
    let expected_err =
        crate::projection_patch::ProjectionPatch::new(default_envelope(), expected_ops)
            .unwrap_err();

    // 2. Validate same operation set through Task Projection
    let actual_err =
        project_task_state(default_envelope(), evidence, routes, default_limits()).unwrap_err();

    // 3. Assert exact preservation
    match actual_err {
        crate::task_projection::TaskProjectionError::Patch(actual_diags) => {
            assert_eq!(
                expected_err, actual_diags,
                "ProjectionPatchDiagnostics were not exactly preserved"
            );
        }
        _ => panic!("Expected Patch diagnostic error but got {:?}", actual_err),
    }
}

#[test]
fn test_missing_task_ref() {
    let mut evidence = default_evidence();
    evidence.task_ref = TaskRecordRef::new(0);
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::IdentityRevisionValidation && diag.kind == TaskProjectionDiagnosticKind::MissingTaskRef)
    );
}

#[test]
fn test_missing_evidence_ref() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::Completed;
    evidence.task_evidence = None;
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::StateValidation && diag.kind == TaskProjectionDiagnosticKind::MissingEvidenceRef)
    );
}

#[test]
fn test_unexpected_awaiting_input() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::Running;
    evidence.awaiting_input = Some(String::from("input"));
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::StateValidation && diag.kind == TaskProjectionDiagnosticKind::UnexpectedAwaitingInput)
    );
}

#[test]
fn test_duplicate_phase_key() {
    let mut evidence = default_evidence();
    evidence.phases.push(TaskPhase {
        id: 2,
        order: 102,
        key: CollectionKey::new(1).unwrap(),
        label: String::from("P2"),
        status: TaskPhaseStatus::Pending,
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::DuplicatePhaseKey)
    );
}

#[test]
fn test_invalid_phase_set() {
    let mut evidence = default_evidence();
    evidence.phases.push(TaskPhase {
        id: 2,
        order: 102,
        key: CollectionKey::new(2).unwrap(),
        label: String::from("P2"),
        status: TaskPhaseStatus::Active,
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidPhaseSet)
    );
}

#[test]
fn test_invalid_progress() {
    let mut evidence = default_evidence();
    evidence.current_progress = TaskProgress::Determinate {
        completed: 5,
        total: 4,
    };
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::ProgressValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidProgress)
    );
}

#[test]
fn test_duplicate_control_order() {
    let mut evidence = default_evidence();
    evidence.controls.push(TaskControlOffer {
        order: 10,
        key: CollectionKey::new(2).unwrap(),
        action: SemanticActionRef::new(1),
        kind: TaskControlKind::Cancel,
        resume_token: None,
    });
    evidence.controls.push(TaskControlOffer {
        order: 10,
        key: CollectionKey::new(3).unwrap(),
        action: SemanticActionRef::new(2),
        kind: TaskControlKind::Cancel,
        resume_token: None,
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::ControlValidation && diag.kind == TaskProjectionDiagnosticKind::DuplicateControlOrder)
    );
}

#[test]
fn test_duplicate_control_key() {
    let mut evidence = default_evidence();
    evidence.controls.push(TaskControlOffer {
        order: 20,
        key: CollectionKey::new(1).unwrap(),
        action: SemanticActionRef::new(1),
        kind: TaskControlKind::Cancel,
        resume_token: None,
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::ControlValidation && diag.kind == TaskProjectionDiagnosticKind::DuplicateControlKey)
    );
}

#[test]
fn test_control_action_ref_missing() {
    let mut evidence = default_evidence();
    evidence.controls.clear();
    evidence.controls.push(TaskControlOffer {
        order: 10,
        key: CollectionKey::new(1).unwrap(),
        action: SemanticActionRef::new(0),
        kind: TaskControlKind::Cancel,
        resume_token: None,
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::ControlValidation && diag.kind == TaskProjectionDiagnosticKind::ControlActionRefMissing)
    );
}

#[test]
fn test_resume_token_missing() {
    let mut evidence = default_evidence();
    evidence.controls.clear();
    evidence.controls.push(TaskControlOffer {
        order: 10,
        key: CollectionKey::new(1).unwrap(),
        action: SemanticActionRef::new(1),
        kind: TaskControlKind::Resume,
        resume_token: None,
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::ControlValidation && diag.kind == TaskProjectionDiagnosticKind::ResumeTokenMissing)
    );
}

#[test]
fn test_duplicate_lock_order() {
    let mut evidence = default_evidence();
    evidence.locks.push(TaskScopeLock {
        order: 1,
        key: CollectionKey::new(2).unwrap(),
        reference: ReferenceToken::new(0, 0, 0, 1),
        explanation: String::from("expl"),
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::ScopeLockValidation && diag.kind == TaskProjectionDiagnosticKind::DuplicateLockOrder)
    );
}

#[test]
fn test_duplicate_lock_key() {
    let mut evidence = default_evidence();
    evidence.locks.push(TaskScopeLock {
        order: 2,
        key: CollectionKey::new(1).unwrap(),
        reference: ReferenceToken::new(0, 0, 0, 1),
        explanation: String::from("expl"),
    });
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::ScopeLockValidation && diag.kind == TaskProjectionDiagnosticKind::DuplicateLockKey)
    );
}

#[test]
fn test_missing_control_route() {
    let mut routes = default_routes();
    routes.control_collection = None;
    let err = project_task_state(
        default_envelope(),
        default_evidence(),
        routes,
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::RouteValidation && diag.kind == TaskProjectionDiagnosticKind::MissingControlRoute)
    );
}

#[test]
fn test_missing_lock_route() {
    let mut routes = default_routes();
    routes.scope_lock_collection = None;
    let err = project_task_state(
        default_envelope(),
        default_evidence(),
        routes,
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::RouteValidation && diag.kind == TaskProjectionDiagnosticKind::MissingLockRoute)
    );
}

#[test]
fn test_missing_awaiting_input_route() {
    let mut routes = default_routes();
    routes.awaiting_input_route = None;
    let err = project_task_state(
        default_envelope(),
        default_evidence(),
        routes,
        default_limits(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::RouteValidation && diag.kind == TaskProjectionDiagnosticKind::MissingAwaitingInputRoute)
    );
}

#[test]
fn test_operation_limit_exceeded() {
    let mut limits = default_limits();
    limits.total_operations = 1;
    let err = project_task_state(
        default_envelope(),
        default_evidence(),
        default_routes(),
        limits,
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::task_projection::TaskProjectionError::Task(ref diag) if diag.stage == ValidationStage::OperationConstruction && diag.kind == TaskProjectionDiagnosticKind::OperationLimitExceeded)
    );
}

// --- Expanded Permutation Tests ---

#[test]
fn test_state_evidence_requirements() {
    let terminal_states = vec![
        TaskProjectionState::Completed,
        TaskProjectionState::Failed,
        TaskProjectionState::Denied,
        TaskProjectionState::Quarantined,
        TaskProjectionState::Cancelled,
        TaskProjectionState::PendingUnknown,
    ];

    for state in terminal_states {
        let mut evidence = default_evidence();
        evidence.state = state;
        evidence.task_evidence = None;
        // Make sure we pass phase requirements for terminal states: 0 active phases
        evidence.phases.clear();

        // For completed, progress must match
        if state == TaskProjectionState::Completed {
            evidence.current_progress = TaskProgress::Determinate {
                completed: 10,
                total: 10,
            };
        }

        let result = project_task_state(
            default_envelope(),
            evidence,
            default_routes(),
            default_limits(),
        );
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                crate::task_projection::TaskProjectionError::Task(ref diag)
                if diag.stage == ValidationStage::StateValidation && diag.kind == TaskProjectionDiagnosticKind::MissingEvidenceRef
            ),
            "State {:?} failed with incorrect error: {:?}",
            state,
            err
        );
    }
}

#[test]
fn test_state_active_phase_requirements() {
    // states that REQUIRE exactly 1 active phase
    let active_req_states = vec![
        TaskProjectionState::Started,
        TaskProjectionState::Running,
        TaskProjectionState::Completing,
    ];

    for state in active_req_states {
        // Test 0 active phases -> should fail
        let mut evidence = default_evidence();
        evidence.state = state;
        evidence.phases.clear();
        let err = project_task_state(
            default_envelope(),
            evidence.clone(),
            default_routes(),
            default_limits(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            crate::task_projection::TaskProjectionError::Task(ref diag)
            if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidPhaseSet
        ));

        // Test >1 active phases -> should fail
        evidence.phases.push(TaskPhase {
            id: 1,
            key: CollectionKey::new(1).unwrap(),
            order: 1,
            label: String::from("P1"),
            status: TaskPhaseStatus::Active,
        });
        evidence.phases.push(TaskPhase {
            id: 2,
            key: CollectionKey::new(2).unwrap(),
            order: 2,
            label: String::from("P2"),
            status: TaskPhaseStatus::Active,
        });
        let err = project_task_state(
            default_envelope(),
            evidence.clone(),
            default_routes(),
            default_limits(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            crate::task_projection::TaskProjectionError::Task(ref diag)
            if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidPhaseSet
        ));
    }

    // terminal states that REQUIRE 0 active phases
    let terminal_states = vec![
        TaskProjectionState::Completed,
        TaskProjectionState::Failed,
        TaskProjectionState::Denied,
        TaskProjectionState::Quarantined,
        TaskProjectionState::Cancelled,
        TaskProjectionState::PendingUnknown,
    ];

    for state in terminal_states {
        let mut evidence = default_evidence();
        evidence.state = state;
        evidence.task_evidence = Some(SemanticEvidenceRef::new(1));
        if state == TaskProjectionState::Completed {
            evidence.current_progress = TaskProgress::Determinate {
                completed: 10,
                total: 10,
            };
        }

        // 1 active phase -> should fail
        evidence.phases.clear();
        evidence.phases.push(TaskPhase {
            id: 1,
            key: CollectionKey::new(1).unwrap(),
            order: 1,
            label: String::from("P1"),
            status: TaskPhaseStatus::Active,
        });
        let err = project_task_state(
            default_envelope(),
            evidence.clone(),
            default_routes(),
            default_limits(),
        )
        .unwrap_err();
        assert!(
            matches!(
                err,
                crate::task_projection::TaskProjectionError::Task(ref diag)
                if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidPhaseSet
            ),
            "Terminal state {:?} failed with incorrect error: {:?}",
            state,
            err
        );
    }
}

#[test]
fn test_state_active_phase_requirements_explicit_awaiting_input() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::AwaitingInput;
    evidence.awaiting_input = Some(alloc::string::String::from("input"));
    evidence.phases.clear();
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidPhaseSet
    ));
}

#[test]
fn test_state_active_phase_requirements_explicit_paused() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::Paused;
    evidence.awaiting_input = None;
    evidence.phases.clear();
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::PhaseValidation && diag.kind == TaskProjectionDiagnosticKind::InvalidPhaseSet
    ));
}

#[test]
fn test_state_awaiting_input_requires_text() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::AwaitingInput;
    evidence.awaiting_input = None;
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::StateValidation && diag.kind == TaskProjectionDiagnosticKind::MissingAwaitingInput
    ));
}

#[test]
fn test_state_paused_forbids_text() {
    let mut evidence = default_evidence();
    evidence.state = TaskProjectionState::Paused;
    evidence.awaiting_input = Some(alloc::string::String::from("text"));
    let err = project_task_state(
        default_envelope(),
        evidence,
        default_routes(),
        default_limits(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        crate::task_projection::TaskProjectionError::Task(ref diag)
        if diag.stage == ValidationStage::StateValidation && diag.kind == TaskProjectionDiagnosticKind::UnexpectedAwaitingInput
    ));
}
