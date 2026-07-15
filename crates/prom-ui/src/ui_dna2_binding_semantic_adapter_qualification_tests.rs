//! Binding Graph Semantic observation adapter v0 qualification.

use alloc::vec::Vec;

use crate::binding_graph::{
    BindingDeclaration, BindingDependency, BindingGraphDocument, BindingId, BindingSlot,
    BindingTarget, BindingValueDomain,
};
use crate::binding_graph_observation::{
    derive_dirty_bindings, BindingObservationErrorKind, BindingObservationLimits,
    BindingObservationScope, BindingQuadState, DirtyDerivationResult, DirtyReasonKind,
};
use crate::binding_graph_semantic_adapter::{
    adapt_semantic_observations, AdaptedBindingObservationInputs, SemanticBindingAdapterError,
    SemanticBindingAdapterErrorKind, SemanticBindingAdapterLimitKind, SemanticBindingAdapterLimits,
    SemanticBindingAdapterStage, SemanticBindingObservation, SemanticBindingObservationSet,
    SemanticBindingSnapshotSide,
};
use crate::contract_primitives::{Epoch, Revision, StaticNodeId};
use crate::semantic_refs::SemanticValueRef;

const QUAD_STATES: [BindingQuadState; 4] = [
    BindingQuadState::N,
    BindingQuadState::F,
    BindingQuadState::T,
    BindingQuadState::S,
];

fn graph(specs: &[(u64, BindingValueDomain, &[u64], Option<u64>)]) -> BindingGraphDocument {
    let mut bindings: Vec<_> = specs
        .iter()
        .map(
            |(id, domain, dependencies, semantic_ref)| BindingDeclaration {
                id: BindingId(*id),
                target: BindingTarget {
                    node: StaticNodeId::new(*id).expect("non-zero test node"),
                    slot: BindingSlot(1),
                },
                domain: *domain,
                dependencies: dependencies
                    .iter()
                    .map(|dependency| BindingDependency {
                        target_binding: BindingId(*dependency),
                    })
                    .collect(),
                collection_key: (*domain == BindingValueDomain::Collection).then_some(*id),
                semantic_value: semantic_ref.map(SemanticValueRef::new),
                source_ref: None,
            },
        )
        .collect();
    for binding in &mut bindings {
        binding.dependencies.sort();
    }
    bindings.sort();
    BindingGraphDocument { bindings }
}

fn scope(ids: &[u64]) -> BindingObservationScope {
    BindingObservationScope::new(ids.iter().copied().map(BindingId).collect())
}

fn observation(
    semantic_ref: u64,
    epoch: u64,
    revision: u64,
    quad_state: Option<BindingQuadState>,
) -> SemanticBindingObservation {
    SemanticBindingObservation::new(
        SemanticValueRef::new(semantic_ref),
        Epoch::new(epoch),
        Revision::new(revision),
        quad_state,
    )
}

fn observations(values: Vec<SemanticBindingObservation>) -> SemanticBindingObservationSet {
    SemanticBindingObservationSet::new(values)
}

fn generous_limits() -> SemanticBindingAdapterLimits {
    SemanticBindingAdapterLimits::new(100, 100, 100, 1_000, 1_000)
}

fn adapt(
    graph: &BindingGraphDocument,
    scoped_ids: &[u64],
    previous: Vec<SemanticBindingObservation>,
    current: Vec<SemanticBindingObservation>,
) -> Result<AdaptedBindingObservationInputs, Vec<SemanticBindingAdapterError>> {
    adapt_semantic_observations(
        graph,
        scope(scoped_ids),
        observations(previous),
        observations(current),
        generous_limits(),
    )
}

fn derive(
    graph: &BindingGraphDocument,
    adapted: AdaptedBindingObservationInputs,
) -> Result<DirtyDerivationResult, Vec<crate::binding_graph_observation::BindingObservationError>> {
    let (scope, previous, current) = adapted.into_parts();
    derive_dirty_bindings(
        graph,
        scope,
        previous,
        current,
        BindingObservationLimits::new(100, 100, 100, 1_000, 1_000),
    )
}

fn dirty_ids(result: &DirtyDerivationResult) -> Vec<u64> {
    result
        .dirty_bindings()
        .iter()
        .map(|binding| binding.binding_id().0)
        .collect()
}

fn reason_kinds(result: &DirtyDerivationResult, binding_id: u64) -> Vec<DirtyReasonKind> {
    result
        .dirty_bindings()
        .iter()
        .find(|binding| binding.binding_id() == BindingId(binding_id))
        .expect("qualified dirty binding")
        .reasons()
        .iter()
        .map(|reason| reason.kind())
        .collect()
}

fn only_error(
    result: Result<AdaptedBindingObservationInputs, Vec<SemanticBindingAdapterError>>,
) -> SemanticBindingAdapterError {
    let errors = result.expect_err("adapter must reject");
    assert_eq!(errors.len(), 1);
    errors[0]
}

#[test]
fn ui_dna2_binding_semantic_adapter_empty_graph_and_scope_are_inert() {
    let graph = graph(&[]);
    let adapted = adapt(&graph, &[], alloc::vec![], alloc::vec![]).expect("empty input");

    assert!(adapted.scope().binding_ids().is_empty());
    assert!(adapted.previous().observations().is_empty());
    assert!(adapted.current().observations().is_empty());
    assert_eq!(adapted.fanout_work(), 0);
}

#[test]
fn ui_dna2_binding_semantic_adapter_scope_errors_are_canonical() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[], Some(10)),
        (2, BindingValueDomain::Text, &[], None),
    ]);
    let errors = adapt_semantic_observations(
        &graph,
        scope(&[99, 2, 1, 1]),
        observations(alloc::vec![]),
        observations(alloc::vec![]),
        generous_limits(),
    )
    .expect_err("scope errors");

    assert!(errors
        .iter()
        .all(|error| error.stage() == SemanticBindingAdapterStage::ScopeValidation));
    assert_eq!(
        errors.iter().map(|error| error.code()).collect::<Vec<_>>(),
        alloc::vec![
            "BGSA_DUPLICATE_SCOPE_BINDING",
            "BGSA_UNKNOWN_SCOPE_BINDING",
            "BGSA_BINDING_MISSING_SEMANTIC_REF",
        ]
    );
    assert_eq!(errors[0].binding_id(), Some(BindingId(1)));
}

#[test]
fn ui_dna2_binding_semantic_adapter_classifies_snapshot_references() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[], Some(10)),
        (2, BindingValueDomain::Scalar, &[], Some(20)),
    ]);
    let errors = adapt_semantic_observations(
        &graph,
        scope(&[1]),
        observations(alloc::vec![
            observation(30, 1, 1, None),
            observation(10, 1, 1, None),
            observation(10, 1, 1, None),
        ]),
        observations(alloc::vec![observation(20, 1, 1, None)]),
        generous_limits(),
    )
    .expect_err("snapshot structure");

    assert!(errors.iter().all(|error| {
        error.stage() == SemanticBindingAdapterStage::SnapshotStructuralValidation
    }));
    assert!(
        errors
            .iter()
            .any(|error| error.kind()
                == SemanticBindingAdapterErrorKind::DuplicateSemanticObservation)
    );
    assert!(errors
        .iter()
        .any(|error| error.kind() == SemanticBindingAdapterErrorKind::UnknownSemanticRef));
    assert!(errors
        .iter()
        .any(|error| error.kind() == SemanticBindingAdapterErrorKind::ObservationOutOfScope));
}

#[test]
fn ui_dna2_binding_semantic_adapter_permutations_have_identical_output() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[], Some(10)),
        (2, BindingValueDomain::Text, &[], Some(20)),
        (3, BindingValueDomain::Evidence, &[], Some(10)),
    ]);
    let first = adapt(
        &graph,
        &[3, 1, 2],
        alloc::vec![observation(20, 4, 5, None), observation(10, 2, 3, None)],
        alloc::vec![observation(10, 2, 4, None), observation(20, 4, 6, None)],
    )
    .expect("first permutation");
    let second = adapt(
        &graph,
        &[2, 3, 1],
        alloc::vec![observation(10, 2, 3, None), observation(20, 4, 5, None)],
        alloc::vec![observation(20, 4, 6, None), observation(10, 2, 4, None)],
    )
    .expect("second permutation");

    assert_eq!(first, second);
    assert_eq!(
        first.scope().binding_ids(),
        &[BindingId(1), BindingId(2), BindingId(3)]
    );
}

#[test]
fn ui_dna2_binding_semantic_adapter_preserves_epoch_revision_and_absence() {
    let graph = graph(&[(1, BindingValueDomain::Scalar, &[], Some(10))]);
    let adapted = adapt(
        &graph,
        &[1],
        alloc::vec![observation(10, 7, 11, None)],
        alloc::vec![],
    )
    .expect("mapped observation");

    let previous = adapted.previous().observations();
    assert_eq!(previous.len(), 1);
    assert_eq!(previous[0].binding_id(), BindingId(1));
    assert_eq!(previous[0].epoch().raw(), 7);
    assert_eq!(previous[0].revision().raw(), 11);
    assert!(adapted.current().observations().is_empty());
}

#[test]
fn ui_dna2_binding_semantic_adapter_fanout_is_canonical_across_non_quad_domains() {
    let graph = graph(&[
        (3, BindingValueDomain::Evidence, &[], Some(10)),
        (1, BindingValueDomain::Scalar, &[], Some(10)),
        (2, BindingValueDomain::Text, &[], Some(10)),
        (4, BindingValueDomain::Task, &[], Some(20)),
    ]);
    let adapted = adapt(
        &graph,
        &[4, 3, 2, 1],
        alloc::vec![],
        alloc::vec![observation(20, 1, 1, None), observation(10, 2, 3, None)],
    )
    .expect("non-Quad fanout");

    assert_eq!(adapted.fanout_work(), 4);
    assert_eq!(
        adapted
            .current()
            .observations()
            .iter()
            .map(|value| value.binding_id().0)
            .collect::<Vec<_>>(),
        alloc::vec![1, 2, 3, 4]
    );
}

#[test]
fn ui_dna2_binding_semantic_adapter_preserves_all_quad_states_across_fanout() {
    let graph = graph(&[
        (1, BindingValueDomain::Quad, &[], Some(10)),
        (2, BindingValueDomain::Quad, &[], Some(10)),
        (3, BindingValueDomain::Quad, &[], Some(10)),
    ]);

    for state in QUAD_STATES {
        let adapted = adapt(
            &graph,
            &[3, 1, 2],
            alloc::vec![],
            alloc::vec![observation(10, 8, 13, Some(state))],
        )
        .expect("exact Quad fanout");
        assert_eq!(adapted.fanout_work(), 3);
        assert!(adapted.current().observations().iter().all(|mapped| {
            mapped.epoch().raw() == 8
                && mapped.revision().raw() == 13
                && mapped.quad_state() == Some(state)
        }));
    }
}

#[test]
fn ui_dna2_binding_semantic_adapter_rejects_mixed_quad_fanout() {
    let graph = graph(&[
        (1, BindingValueDomain::Quad, &[], Some(10)),
        (2, BindingValueDomain::Scalar, &[], Some(10)),
    ]);
    let error = only_error(adapt(&graph, &[1, 2], alloc::vec![], alloc::vec![]));

    assert_eq!(
        error.stage(),
        SemanticBindingAdapterStage::ReferenceIndexValidation
    );
    assert_eq!(
        error.kind(),
        SemanticBindingAdapterErrorKind::IncompatibleFanoutDomain
    );
    assert_eq!(error.semantic_value_ref(), Some(SemanticValueRef::new(10)));
}

#[test]
fn ui_dna2_binding_semantic_adapter_enforces_domain_carriers() {
    let graph = graph(&[
        (1, BindingValueDomain::Quad, &[], Some(10)),
        (2, BindingValueDomain::Scalar, &[], Some(20)),
    ]);
    let errors = adapt_semantic_observations(
        &graph,
        scope(&[1, 2]),
        observations(alloc::vec![observation(10, 1, 1, None)]),
        observations(alloc::vec![observation(
            20,
            1,
            1,
            Some(BindingQuadState::S)
        )]),
        generous_limits(),
    )
    .expect_err("domain carriers");

    assert!(errors
        .iter()
        .all(|error| error.stage() == SemanticBindingAdapterStage::DomainCarrierValidation));
    assert!(errors
        .iter()
        .any(|error| error.kind() == SemanticBindingAdapterErrorKind::MissingQuadState));
    assert!(errors
        .iter()
        .any(|error| error.kind() == SemanticBindingAdapterErrorKind::UnexpectedQuadState));
}

#[test]
fn ui_dna2_binding_semantic_adapter_resource_preflight_is_earliest() {
    let graph = graph(&[(1, BindingValueDomain::Scalar, &[], Some(10))]);
    let errors = adapt_semantic_observations(
        &graph,
        scope(&[99, 1]),
        observations(alloc::vec![observation(77, 1, 1, None)]),
        observations(alloc::vec![observation(88, 1, 1, None)]),
        SemanticBindingAdapterLimits::new(1, 0, 0, 0, 0),
    )
    .expect_err("preflight");

    assert!(errors
        .iter()
        .all(|error| error.stage() == SemanticBindingAdapterStage::ResourcePreflight));
    assert_eq!(errors.len(), 3);
    assert!(errors
        .iter()
        .all(|error| { error.kind() == SemanticBindingAdapterErrorKind::ResourceLimitExceeded }));
}

#[test]
fn ui_dna2_binding_semantic_adapter_index_and_fanout_limits_are_bounded() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[], Some(10)),
        (2, BindingValueDomain::Scalar, &[], Some(20)),
        (3, BindingValueDomain::Scalar, &[], Some(20)),
    ]);
    let index_error = only_error(adapt_semantic_observations(
        &graph,
        scope(&[1]),
        observations(alloc::vec![]),
        observations(alloc::vec![]),
        SemanticBindingAdapterLimits::new(10, 10, 1, 10, 10),
    ));
    assert_eq!(
        index_error.limit_kind(),
        Some(SemanticBindingAdapterLimitKind::IndexedSemanticReferences)
    );
    assert_eq!(index_error.actual(), Some(2));
    assert_eq!(index_error.maximum(), Some(1));

    let fanout_error = only_error(adapt_semantic_observations(
        &graph,
        scope(&[3, 1, 2]),
        observations(alloc::vec![]),
        observations(alloc::vec![]),
        SemanticBindingAdapterLimits::new(10, 10, 10, 2, 10),
    ));
    assert_eq!(
        fanout_error.kind(),
        SemanticBindingAdapterErrorKind::FanoutLimitExceeded
    );
    assert_eq!(fanout_error.binding_id(), Some(BindingId(3)));
    assert_eq!(fanout_error.actual(), Some(3));
}

#[test]
fn ui_dna2_binding_semantic_adapter_mapped_snapshot_limits_are_side_specific() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[], Some(10)),
        (2, BindingValueDomain::Scalar, &[], Some(10)),
    ]);
    let errors = adapt_semantic_observations(
        &graph,
        scope(&[1, 2]),
        observations(alloc::vec![observation(10, 1, 1, None)]),
        observations(alloc::vec![observation(10, 1, 2, None)]),
        SemanticBindingAdapterLimits::new(10, 10, 10, 10, 1),
    )
    .expect_err("mapped limits");

    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| {
        error.stage() == SemanticBindingAdapterStage::CanonicalObservationMapping
            && error.kind() == SemanticBindingAdapterErrorKind::MappedObservationLimitExceeded
            && error.binding_id() == Some(BindingId(2))
            && error.actual() == Some(2)
            && error.maximum() == Some(1)
    }));
    assert_eq!(
        errors
            .iter()
            .map(|error| error.snapshot_side())
            .collect::<Vec<_>>(),
        alloc::vec![
            Some(SemanticBindingSnapshotSide::Previous),
            Some(SemanticBindingSnapshotSide::Current),
        ]
    );
}

#[test]
fn ui_dna2_binding_semantic_adapter_stage_precedence_is_explicit() {
    let missing_ref_graph = graph(&[(1, BindingValueDomain::Scalar, &[], None)]);
    let error = only_error(adapt_semantic_observations(
        &missing_ref_graph,
        scope(&[1]),
        observations(alloc::vec![observation(
            10,
            1,
            1,
            Some(BindingQuadState::N)
        )]),
        observations(alloc::vec![]),
        SemanticBindingAdapterLimits::new(10, 10, 0, 0, 0),
    ));
    assert_eq!(error.stage(), SemanticBindingAdapterStage::ScopeValidation);

    let graph = graph(&[(1, BindingValueDomain::Quad, &[], Some(10))]);
    let error = only_error(adapt_semantic_observations(
        &graph,
        scope(&[1]),
        observations(alloc::vec![observation(99, 1, 1, None)]),
        observations(alloc::vec![]),
        SemanticBindingAdapterLimits::new(10, 10, 10, 10, 0),
    ));
    assert_eq!(
        error.stage(),
        SemanticBindingAdapterStage::SnapshotStructuralValidation
    );

    let error = only_error(adapt_semantic_observations(
        &graph,
        scope(&[1]),
        observations(alloc::vec![observation(10, 1, 1, None)]),
        observations(alloc::vec![]),
        SemanticBindingAdapterLimits::new(10, 10, 10, 10, 0),
    ));
    assert_eq!(
        error.stage(),
        SemanticBindingAdapterStage::DomainCarrierValidation
    );
}

#[test]
fn ui_dna2_binding_semantic_adapter_composes_added_and_removed() {
    let graph = graph(&[(1, BindingValueDomain::Scalar, &[], Some(10))]);
    let added = derive(
        &graph,
        adapt(
            &graph,
            &[1],
            alloc::vec![],
            alloc::vec![observation(10, 1, 1, None)],
        )
        .expect("adapt added"),
    )
    .expect("derive added");
    assert_eq!(reason_kinds(&added, 1), alloc::vec![DirtyReasonKind::Added]);

    let removed = derive(
        &graph,
        adapt(
            &graph,
            &[1],
            alloc::vec![observation(10, 1, 1, None)],
            alloc::vec![],
        )
        .expect("adapt removed"),
    )
    .expect("derive removed");
    assert_eq!(
        reason_kinds(&removed, 1),
        alloc::vec![DirtyReasonKind::Removed]
    );
}

#[test]
fn ui_dna2_binding_semantic_adapter_composes_revision_and_epoch_changes() {
    let graph = graph(&[(1, BindingValueDomain::Scalar, &[], Some(10))]);
    let revision = derive(
        &graph,
        adapt(
            &graph,
            &[1],
            alloc::vec![observation(10, 2, 3, None)],
            alloc::vec![observation(10, 2, 4, None)],
        )
        .expect("adapt revision"),
    )
    .expect("derive revision");
    assert_eq!(
        reason_kinds(&revision, 1),
        alloc::vec![DirtyReasonKind::RevisionChanged]
    );

    let epoch = derive(
        &graph,
        adapt(
            &graph,
            &[1],
            alloc::vec![observation(10, 2, 50, None)],
            alloc::vec![observation(10, 3, 1, None)],
        )
        .expect("adapt epoch"),
    )
    .expect("derive epoch");
    assert!(reason_kinds(&epoch, 1).contains(&DirtyReasonKind::EpochChanged));
}

#[test]
fn ui_dna2_binding_semantic_adapter_composes_exact_quad_transitions() {
    let graph = graph(&[(1, BindingValueDomain::Quad, &[], Some(10))]);
    let transitions = [
        (BindingQuadState::N, BindingQuadState::F),
        (BindingQuadState::F, BindingQuadState::T),
        (BindingQuadState::T, BindingQuadState::S),
        (BindingQuadState::S, BindingQuadState::N),
    ];

    for (previous, current) in transitions {
        let result = derive(
            &graph,
            adapt(
                &graph,
                &[1],
                alloc::vec![observation(10, 1, 1, Some(previous))],
                alloc::vec![observation(10, 1, 2, Some(current))],
            )
            .expect("adapt Quad transition"),
        )
        .expect("derive Quad transition");
        let reasons = result.dirty_bindings()[0].reasons();
        assert!(reasons
            .iter()
            .any(|reason| reason.kind() == DirtyReasonKind::RevisionChanged));
        let quad = reasons
            .iter()
            .find(|reason| reason.kind() == DirtyReasonKind::QuadValueChanged)
            .expect("Quad reason");
        assert_eq!(quad.previous_quad_state(), Some(previous));
        assert_eq!(quad.current_quad_state(), Some(current));
    }
}

#[test]
fn ui_dna2_binding_semantic_adapter_composes_fanout_and_dependency_propagation() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[], Some(10)),
        (2, BindingValueDomain::Text, &[], Some(10)),
        (3, BindingValueDomain::Evidence, &[1], Some(30)),
        (4, BindingValueDomain::Task, &[3], Some(40)),
    ]);
    let result = derive(
        &graph,
        adapt(
            &graph,
            &[1, 2, 3, 4],
            alloc::vec![observation(10, 1, 1, None)],
            alloc::vec![observation(10, 1, 2, None)],
        )
        .expect("adapt fanout"),
    )
    .expect("derive fanout");

    assert_eq!(dirty_ids(&result), alloc::vec![1, 2, 3, 4]);
    assert!(reason_kinds(&result, 3)
        .iter()
        .all(|kind| *kind == DirtyReasonKind::RevisionChanged));
}

#[test]
fn ui_dna2_binding_semantic_adapter_leaves_temporal_rejection_to_dirty_engine() {
    let graph = graph(&[(1, BindingValueDomain::Quad, &[], Some(10))]);
    let stale = derive(
        &graph,
        adapt(
            &graph,
            &[1],
            alloc::vec![observation(10, 2, 5, Some(BindingQuadState::N))],
            alloc::vec![observation(10, 2, 4, Some(BindingQuadState::N))],
        )
        .expect("adapter transports stale tuple"),
    )
    .expect_err("engine rejects stale tuple");
    assert_eq!(
        stale[0].kind(),
        BindingObservationErrorKind::StaleObservation
    );

    let conflict = derive(
        &graph,
        adapt(
            &graph,
            &[1],
            alloc::vec![observation(10, 2, 5, Some(BindingQuadState::N))],
            alloc::vec![observation(10, 2, 5, Some(BindingQuadState::S))],
        )
        .expect("adapter transports conflicting replay"),
    )
    .expect_err("engine rejects conflicting replay");
    assert_eq!(
        conflict[0].kind(),
        BindingObservationErrorKind::ConflictingReplay
    );
}

#[test]
fn ui_dna2_binding_semantic_adapter_errors_expose_stable_evidence() {
    let graph = graph(&[(1, BindingValueDomain::Scalar, &[], Some(10))]);
    let error = only_error(adapt_semantic_observations(
        &graph,
        scope(&[1]),
        observations(alloc::vec![observation(10, 1, 1, None)]),
        observations(alloc::vec![]),
        SemanticBindingAdapterLimits::new(10, 10, 10, 10, 0),
    ));

    assert_eq!(error.code(), "BGSA_MAPPED_OBSERVATION_LIMIT_EXCEEDED");
    assert_eq!(
        error.limit_kind(),
        Some(SemanticBindingAdapterLimitKind::MappedPreviousObservations)
    );
    assert_eq!(
        error.snapshot_side(),
        Some(SemanticBindingSnapshotSide::Previous)
    );
    assert_eq!(error.semantic_value_ref(), Some(SemanticValueRef::new(10)));
    assert_eq!(error.binding_id(), Some(BindingId(1)));
    assert_eq!(error.actual(), Some(1));
    assert_eq!(error.maximum(), Some(0));
}
