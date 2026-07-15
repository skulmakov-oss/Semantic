//! Binding Graph observation/dirty engine v0 qualification.

use alloc::vec::Vec;

use crate::binding_graph::{
    BindingDeclaration, BindingDependency, BindingGraphDocument, BindingId, BindingSlot,
    BindingTarget, BindingValueDomain,
};
use crate::binding_graph_observation::{
    derive_dirty_bindings, BindingObservation, BindingObservationError,
    BindingObservationErrorKind, BindingObservationLimitKind, BindingObservationLimits,
    BindingObservationScope, BindingObservationSet, BindingObservationStage, BindingQuadState,
    DirtyDerivationResult, DirtyReasonKind,
};
use crate::contract_primitives::{Epoch, Revision, StaticNodeId};

const QUAD_STATES: [BindingQuadState; 4] = [
    BindingQuadState::N,
    BindingQuadState::F,
    BindingQuadState::T,
    BindingQuadState::S,
];

fn graph(specs: &[(u64, BindingValueDomain, &[u64])]) -> BindingGraphDocument {
    let mut bindings: Vec<_> = specs
        .iter()
        .map(|(id, domain, dependencies)| BindingDeclaration {
            id: BindingId(*id),
            target: BindingTarget {
                node: StaticNodeId::new(*id).expect("non-zero node"),
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
            semantic_value: None,
            source_ref: None,
        })
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
    id: u64,
    epoch: u64,
    revision: u64,
    quad_state: Option<BindingQuadState>,
) -> BindingObservation {
    BindingObservation::new(
        BindingId(id),
        Epoch::new(epoch),
        Revision::new(revision),
        quad_state,
    )
}

fn observations(values: Vec<BindingObservation>) -> BindingObservationSet {
    BindingObservationSet::new(values)
}

fn generous_limits() -> BindingObservationLimits {
    BindingObservationLimits::new(100, 100, 100, 1_000, 1_000)
}

fn derive(
    graph: &BindingGraphDocument,
    scope: &[u64],
    previous: Vec<BindingObservation>,
    current: Vec<BindingObservation>,
) -> Result<DirtyDerivationResult, Vec<BindingObservationError>> {
    derive_dirty_bindings(
        graph,
        self::scope(scope),
        observations(previous),
        observations(current),
        generous_limits(),
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
        .expect("dirty binding")
        .reasons()
        .iter()
        .map(|reason| reason.kind())
        .collect()
}

fn only_error(
    result: Result<DirtyDerivationResult, Vec<BindingObservationError>>,
) -> BindingObservationError {
    let errors = result.expect_err("derivation must reject");
    assert_eq!(errors.len(), 1);
    errors[0]
}

#[test]
fn ui_dna2_binding_observation_empty_scope_and_absent_replay_are_clean() {
    let empty = graph(&[]);
    let result = derive(&empty, &[], alloc::vec![], alloc::vec![]).expect("empty scope");
    assert!(result.dirty_bindings().is_empty());
    assert_eq!(result.propagation_work(), 0);

    let scalar = graph(&[(1, BindingValueDomain::Scalar, &[])]);
    let result = derive(&scalar, &[1], alloc::vec![], alloc::vec![]).expect("absent replay");
    assert!(result.dirty_bindings().is_empty());
}

#[test]
fn ui_dna2_binding_observation_structural_matrix_is_canonical() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[]),
    ]);
    let errors = derive_dirty_bindings(
        &graph,
        scope(&[99, 1, 1]),
        observations(alloc::vec![
            observation(77, 1, 1, None),
            observation(1, 1, 1, None),
            observation(1, 1, 1, None),
        ]),
        observations(alloc::vec![
            observation(2, 1, 1, None),
            observation(2, 1, 1, None),
        ]),
        generous_limits(),
    )
    .expect_err("structural errors");

    assert!(errors
        .iter()
        .all(|error| error.stage() == BindingObservationStage::StructuralValidation));
    let codes: Vec<_> = errors.iter().map(|error| error.code()).collect();
    assert_eq!(
        codes,
        alloc::vec![
            "BGOD_DUPLICATE_SCOPE_BINDING",
            "BGOD_DUPLICATE_OBSERVATION",
            "BGOD_DUPLICATE_OBSERVATION",
            "BGOD_OBSERVATION_OUT_OF_SCOPE",
            "BGOD_UNKNOWN_BINDING",
            "BGOD_UNKNOWN_SCOPE_BINDING",
        ]
    );
    assert_eq!(errors[0].binding_id(), Some(BindingId(1)));
    assert_eq!(errors[0].related_binding_id(), None);
}

#[test]
fn ui_dna2_binding_observation_domain_matrix_covers_quad_and_every_non_quad_domain() {
    let domains = [
        BindingValueDomain::Quad,
        BindingValueDomain::Scalar,
        BindingValueDomain::Text,
        BindingValueDomain::Evidence,
        BindingValueDomain::Collection,
        BindingValueDomain::Task,
        BindingValueDomain::Connectivity,
        BindingValueDomain::ActionAvailability,
    ];
    let specs: Vec<_> = domains
        .iter()
        .enumerate()
        .map(|(index, domain)| ((index + 1) as u64, *domain, &[][..]))
        .collect();
    let graph = graph(&specs);
    let scope_ids: Vec<_> = (1..=8).collect();
    let values = domains
        .iter()
        .enumerate()
        .map(|(index, domain)| {
            observation(
                (index + 1) as u64,
                1,
                1,
                (*domain != BindingValueDomain::Quad).then_some(BindingQuadState::N),
            )
        })
        .collect();

    let errors =
        derive(&graph, &scope_ids, values, alloc::vec![]).expect_err("domain errors must reject");
    assert_eq!(errors.len(), 8);
    assert_eq!(errors[0].code(), "BGOD_MISSING_QUAD_STATE");
    assert!(errors[1..]
        .iter()
        .all(|error| error.code() == "BGOD_UNEXPECTED_QUAD_STATE"));
    assert!(errors
        .iter()
        .all(|error| error.stage() == BindingObservationStage::DomainValidation));
}

#[test]
fn ui_dna2_binding_observation_scope_and_storage_permutations_are_invariant() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[1]),
        (3, BindingValueDomain::Scalar, &[2]),
    ]);
    let previous = alloc::vec![observation(1, 1, 1, None), observation(2, 1, 1, None),];
    let current = alloc::vec![observation(1, 1, 2, None), observation(2, 1, 1, None),];
    let first = derive(&graph, &[1, 2, 3], previous.clone(), current.clone()).unwrap();
    let second = derive(
        &graph,
        &[3, 1, 2],
        previous.into_iter().rev().collect(),
        current.into_iter().rev().collect(),
    )
    .unwrap();
    assert_eq!(first, second);
}

#[test]
fn ui_dna2_binding_observation_added_removed_and_exact_replay_are_distinct() {
    let graph = graph(&[(1, BindingValueDomain::Scalar, &[])]);
    let value = observation(1, 1, 1, None);

    let added = derive(&graph, &[1], alloc::vec![], alloc::vec![value]).unwrap();
    assert_eq!(reason_kinds(&added, 1), alloc::vec![DirtyReasonKind::Added]);

    let removed = derive(&graph, &[1], alloc::vec![value], alloc::vec![]).unwrap();
    assert_eq!(
        reason_kinds(&removed, 1),
        alloc::vec![DirtyReasonKind::Removed]
    );

    let replay = derive(&graph, &[1], alloc::vec![value], alloc::vec![value]).unwrap();
    assert!(replay.dirty_bindings().is_empty());
}

#[test]
fn ui_dna2_binding_observation_revision_epoch_and_reset_reasons_are_exact() {
    let graph = graph(&[(1, BindingValueDomain::Scalar, &[])]);

    let revision = derive(
        &graph,
        &[1],
        alloc::vec![observation(1, 1, 1, None)],
        alloc::vec![observation(1, 1, 2, None)],
    )
    .unwrap();
    assert_eq!(
        reason_kinds(&revision, 1),
        alloc::vec![DirtyReasonKind::RevisionChanged]
    );

    let epoch = derive(
        &graph,
        &[1],
        alloc::vec![observation(1, 1, 1, None)],
        alloc::vec![observation(1, 2, 1, None)],
    )
    .unwrap();
    assert_eq!(
        reason_kinds(&epoch, 1),
        alloc::vec![DirtyReasonKind::EpochChanged]
    );

    let reset = derive(
        &graph,
        &[1],
        alloc::vec![observation(1, 1, 9, None)],
        alloc::vec![observation(1, 2, 0, None)],
    )
    .unwrap();
    assert_eq!(
        reason_kinds(&reset, 1),
        alloc::vec![
            DirtyReasonKind::RevisionChanged,
            DirtyReasonKind::EpochChanged,
        ]
    );
}

#[test]
fn ui_dna2_binding_observation_stale_revision_and_epoch_reject_canonically() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[]),
    ]);
    let errors = derive(
        &graph,
        &[1, 2],
        alloc::vec![observation(1, 2, 5, None), observation(2, 2, 1, None),],
        alloc::vec![observation(1, 2, 4, None), observation(2, 1, 99, None),],
    )
    .expect_err("stale observations");
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| {
        error.kind() == BindingObservationErrorKind::StaleObservation
            && error.stage() == BindingObservationStage::TemporalValidation
    }));
    assert_eq!(errors[0].binding_id(), Some(BindingId(1)));
    assert_eq!(errors[1].binding_id(), Some(BindingId(2)));
}

#[test]
fn ui_dna2_binding_observation_temporal_validation_completes_before_seed_limits() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[]),
        (3, BindingValueDomain::Quad, &[]),
    ]);
    let errors = derive_dirty_bindings(
        &graph,
        scope(&[3, 1, 2]),
        observations(alloc::vec![
            observation(2, 1, 2, None),
            observation(3, 1, 1, Some(BindingQuadState::N)),
        ]),
        observations(alloc::vec![
            observation(1, 1, 1, None),
            observation(2, 1, 1, None),
            observation(3, 1, 1, Some(BindingQuadState::S)),
        ]),
        BindingObservationLimits::new(3, 3, 0, 0, 0),
    )
    .expect_err("temporal errors must precede direct seed capacity");

    assert_eq!(errors.len(), 2);
    assert!(errors
        .iter()
        .all(|error| error.stage() == BindingObservationStage::TemporalValidation));
    assert_eq!(errors[0].binding_id(), Some(BindingId(2)));
    assert_eq!(errors[0].code(), "BGOD_STALE_OBSERVATION");
    assert_eq!(errors[1].binding_id(), Some(BindingId(3)));
    assert_eq!(errors[1].code(), "BGOD_CONFLICTING_REPLAY");
}

#[test]
fn ui_dna2_binding_observation_equal_tuple_quad_replay_and_conflict_are_exhaustive() {
    let graph = graph(&[(1, BindingValueDomain::Quad, &[])]);
    for previous_state in QUAD_STATES {
        for current_state in QUAD_STATES {
            let result = derive(
                &graph,
                &[1],
                alloc::vec![observation(1, 7, 9, Some(previous_state))],
                alloc::vec![observation(1, 7, 9, Some(current_state))],
            );
            if previous_state == current_state {
                assert!(result.unwrap().dirty_bindings().is_empty());
            } else {
                let error = only_error(result);
                assert_eq!(error.code(), "BGOD_CONFLICTING_REPLAY");
                assert_eq!(error.stage(), BindingObservationStage::TemporalValidation);
            }
        }
    }
}

#[test]
fn ui_dna2_binding_observation_quad_four_by_four_preserves_exact_states() {
    let graph = graph(&[(1, BindingValueDomain::Quad, &[])]);
    for previous_state in QUAD_STATES {
        for current_state in QUAD_STATES {
            let result = derive(
                &graph,
                &[1],
                alloc::vec![observation(1, 1, 1, Some(previous_state))],
                alloc::vec![observation(1, 1, 2, Some(current_state))],
            )
            .unwrap();
            let binding = &result.dirty_bindings()[0];
            let expected = if previous_state == current_state {
                alloc::vec![DirtyReasonKind::RevisionChanged]
            } else {
                alloc::vec![
                    DirtyReasonKind::RevisionChanged,
                    DirtyReasonKind::QuadValueChanged,
                ]
            };
            assert_eq!(reason_kinds(&result, 1), expected);
            for reason in binding.reasons() {
                assert_eq!(reason.originating_binding_id(), BindingId(1));
                assert_eq!(reason.previous_quad_state(), Some(previous_state));
                assert_eq!(reason.current_quad_state(), Some(current_state));
            }
        }
    }
}

#[test]
fn ui_dna2_binding_observation_reverse_dependency_closure_and_work_are_exact() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[1]),
        (3, BindingValueDomain::Scalar, &[1]),
        (4, BindingValueDomain::Scalar, &[2, 3]),
    ]);
    let result = derive(
        &graph,
        &[1, 2, 3, 4],
        alloc::vec![observation(1, 1, 1, None)],
        alloc::vec![observation(1, 1, 2, None)],
    )
    .unwrap();
    assert_eq!(dirty_ids(&result), alloc::vec![1, 2, 3, 4]);
    assert_eq!(result.propagation_work(), 4);
    for binding in result.dirty_bindings() {
        assert_eq!(binding.reasons().len(), 1);
        assert_eq!(binding.reasons()[0].originating_binding_id(), BindingId(1));
    }
}

#[test]
fn ui_dna2_binding_observation_multiple_origins_and_reasons_coalesce_canonically() {
    let graph = graph(&[
        (1, BindingValueDomain::Quad, &[]),
        (2, BindingValueDomain::Scalar, &[]),
        (3, BindingValueDomain::Scalar, &[1, 2]),
    ]);
    let result = derive(
        &graph,
        &[1, 2, 3],
        alloc::vec![observation(1, 1, 9, Some(BindingQuadState::N))],
        alloc::vec![
            observation(2, 1, 1, None),
            observation(1, 2, 0, Some(BindingQuadState::S)),
        ],
    )
    .unwrap();
    assert_eq!(dirty_ids(&result), alloc::vec![1, 2, 3]);
    assert_eq!(result.propagation_work(), 2);
    let converged = result
        .dirty_bindings()
        .iter()
        .find(|binding| binding.binding_id() == BindingId(3))
        .unwrap();
    let reason_keys: Vec<_> = converged
        .reasons()
        .iter()
        .map(|reason| (reason.kind(), reason.originating_binding_id().0))
        .collect();
    assert_eq!(
        reason_keys,
        alloc::vec![
            (DirtyReasonKind::Added, 2),
            (DirtyReasonKind::RevisionChanged, 1),
            (DirtyReasonKind::EpochChanged, 1),
            (DirtyReasonKind::QuadValueChanged, 1),
        ]
    );
}

#[test]
fn ui_dna2_binding_observation_graph_storage_order_does_not_change_propagation() {
    let first = BindingGraphDocument {
        bindings: graph(&[
            (1, BindingValueDomain::Scalar, &[]),
            (2, BindingValueDomain::Scalar, &[1]),
            (3, BindingValueDomain::Scalar, &[2]),
        ])
        .bindings,
    };
    let mut reversed = first.clone();
    reversed.bindings.reverse();
    let previous = alloc::vec![observation(1, 1, 1, None)];
    let current = alloc::vec![observation(1, 1, 2, None)];
    assert_eq!(
        derive(&first, &[1, 2, 3], previous.clone(), current.clone()).unwrap(),
        derive(&reversed, &[3, 2, 1], previous, current).unwrap()
    );
}

#[test]
fn ui_dna2_binding_observation_resource_preflight_and_combined_accounting_are_stable() {
    let graph = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[]),
    ]);
    let values = alloc::vec![observation(1, 1, 1, None), observation(2, 1, 1, None),];
    let errors = derive_dirty_bindings(
        &graph,
        scope(&[1, 2]),
        observations(values.clone()),
        observations(values),
        BindingObservationLimits::new(1, 1, 10, 10, 10),
    )
    .expect_err("preflight limits");
    assert_eq!(errors.len(), 3);
    assert!(errors.iter().all(|error| {
        error.stage() == BindingObservationStage::ResourcePreflight
            && error.code() == "BGOD_RESOURCE_LIMIT_EXCEEDED"
            && error.actual() == Some(2)
            && error.maximum() == Some(1)
    }));
    assert_eq!(
        errors
            .iter()
            .map(|error| error.limit_kind().unwrap())
            .collect::<Vec<_>>(),
        alloc::vec![
            BindingObservationLimitKind::ScopedBindings,
            BindingObservationLimitKind::PreviousObservations,
            BindingObservationLimitKind::CurrentObservations,
        ]
    );

    let values = alloc::vec![observation(1, 1, 1, None), observation(2, 1, 1, None),];
    let accepted = derive_dirty_bindings(
        &graph,
        scope(&[1, 2]),
        observations(values.clone()),
        observations(values),
        BindingObservationLimits::new(2, 2, 2, 0, 0),
    )
    .expect("checked combined input accounting accepts representable totals");
    assert!(accepted.dirty_bindings().is_empty());
}

#[test]
fn ui_dna2_binding_observation_direct_and_propagated_dirty_limits_are_distinct() {
    let independent = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[]),
    ]);
    let first_direct = only_error(derive_dirty_bindings(
        &independent,
        scope(&[2, 1]),
        observations(alloc::vec![]),
        observations(alloc::vec![
            observation(2, 1, 1, None),
            observation(1, 1, 1, None)
        ]),
        BindingObservationLimits::new(2, 2, 0, 0, 0),
    ));
    assert_eq!(
        first_direct.stage(),
        BindingObservationStage::DirtySeedDerivation
    );
    assert_eq!(first_direct.code(), "BGOD_DIRTY_RESULT_LIMIT_EXCEEDED");
    assert_eq!(first_direct.binding_id(), Some(BindingId(1)));
    assert_eq!(first_direct.actual(), Some(1));
    assert_eq!(first_direct.maximum(), Some(0));

    let second_direct = only_error(derive_dirty_bindings(
        &independent,
        scope(&[1, 2]),
        observations(alloc::vec![]),
        observations(alloc::vec![
            observation(2, 1, 1, None),
            observation(1, 1, 1, None),
        ]),
        BindingObservationLimits::new(2, 2, 1, 0, 0),
    ));
    assert_eq!(
        second_direct.stage(),
        BindingObservationStage::DirtySeedDerivation
    );
    assert_eq!(second_direct.code(), "BGOD_DIRTY_RESULT_LIMIT_EXCEEDED");
    assert_eq!(second_direct.binding_id(), Some(BindingId(2)));
    assert_eq!(second_direct.actual(), Some(2));
    assert_eq!(second_direct.maximum(), Some(1));

    let chain = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[1]),
    ]);
    let propagated = only_error(derive_dirty_bindings(
        &chain,
        scope(&[1, 2]),
        observations(alloc::vec![observation(1, 1, 1, None)]),
        observations(alloc::vec![observation(1, 1, 2, None)]),
        BindingObservationLimits::new(2, 1, 1, 0, 0),
    ));
    assert_eq!(
        propagated.stage(),
        BindingObservationStage::DependencyPropagation
    );
    assert_eq!(propagated.code(), "BGOD_DIRTY_RESULT_LIMIT_EXCEEDED");
    assert_eq!(propagated.binding_id(), Some(BindingId(2)));
}

#[test]
fn ui_dna2_binding_observation_work_and_retained_reason_limits_are_stage_owned() {
    let chain = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[1]),
        (3, BindingValueDomain::Scalar, &[2]),
        (4, BindingValueDomain::Scalar, &[3]),
        (5, BindingValueDomain::Scalar, &[4]),
    ]);
    let work = only_error(derive_dirty_bindings(
        &chain,
        scope(&[1, 2, 3, 4, 5]),
        observations(alloc::vec![observation(1, 1, 1, None)]),
        observations(alloc::vec![observation(1, 1, 2, None)]),
        BindingObservationLimits::new(5, 1, 5, 1, 100),
    ));
    assert_eq!(work.code(), "BGOD_PROPAGATION_WORK_LIMIT_EXCEEDED");
    assert_eq!(work.actual(), Some(2));
    assert_eq!(work.maximum(), Some(1));

    let quad = graph(&[(1, BindingValueDomain::Quad, &[])]);
    let retained = only_error(derive_dirty_bindings(
        &quad,
        scope(&[1]),
        observations(alloc::vec![
            observation(1, 1, 9, Some(BindingQuadState::N),)
        ]),
        observations(alloc::vec![
            observation(1, 2, 0, Some(BindingQuadState::S),)
        ]),
        BindingObservationLimits::new(1, 1, 1, 0, 2),
    ));
    assert_eq!(retained.stage(), BindingObservationStage::CanonicalResult);
    assert_eq!(retained.code(), "BGOD_RETAINED_REASON_LIMIT_EXCEEDED");
    assert_eq!(retained.actual(), Some(3));
}

#[test]
fn ui_dna2_binding_observation_error_precedence_matches_the_frozen_stages() {
    let scalar_chain = graph(&[
        (1, BindingValueDomain::Scalar, &[]),
        (2, BindingValueDomain::Scalar, &[1]),
    ]);
    let structural = only_error(derive_dirty_bindings(
        &scalar_chain,
        scope(&[1, 1]),
        observations(alloc::vec![]),
        observations(alloc::vec![observation(1, 1, 1, None)]),
        BindingObservationLimits::new(2, 1, 0, 0, 0),
    ));
    assert_eq!(
        structural.stage(),
        BindingObservationStage::StructuralValidation
    );

    let quad = graph(&[(1, BindingValueDomain::Quad, &[])]);
    let domain = only_error(derive_dirty_bindings(
        &quad,
        scope(&[1]),
        observations(alloc::vec![observation(1, 1, 1, None)]),
        observations(alloc::vec![observation(1, 1, 2, None)]),
        BindingObservationLimits::new(1, 1, 1, 0, 0),
    ));
    assert_eq!(domain.stage(), BindingObservationStage::DomainValidation);

    let temporal = only_error(derive_dirty_bindings(
        &scalar_chain,
        scope(&[1, 2]),
        observations(alloc::vec![observation(1, 1, 2, None)]),
        observations(alloc::vec![observation(1, 1, 1, None)]),
        BindingObservationLimits::new(2, 1, 0, 0, 0),
    ));
    assert_eq!(
        temporal.stage(),
        BindingObservationStage::TemporalValidation
    );

    let dirty = only_error(derive_dirty_bindings(
        &scalar_chain,
        scope(&[1, 2]),
        observations(alloc::vec![observation(1, 1, 1, None)]),
        observations(alloc::vec![observation(1, 1, 2, None)]),
        BindingObservationLimits::new(2, 1, 1, 0, 0),
    ));
    assert_eq!(dirty.code(), "BGOD_DIRTY_RESULT_LIMIT_EXCEEDED");
    assert_eq!(
        dirty.stage(),
        BindingObservationStage::DependencyPropagation
    );
    assert_eq!(dirty.binding_id(), Some(BindingId(2)));
    assert_eq!(dirty.actual(), Some(2));
    assert_eq!(dirty.maximum(), Some(1));

    let work = only_error(derive_dirty_bindings(
        &scalar_chain,
        scope(&[1, 2]),
        observations(alloc::vec![observation(1, 1, 1, None)]),
        observations(alloc::vec![observation(1, 1, 2, None)]),
        BindingObservationLimits::new(2, 1, 2, 0, 0),
    ));
    assert_eq!(work.code(), "BGOD_PROPAGATION_WORK_LIMIT_EXCEEDED");
}

#[test]
fn ui_dna2_binding_observation_result_is_inert_evidence_only() {
    let graph = graph(&[(1, BindingValueDomain::Quad, &[])]);
    let result = derive(
        &graph,
        &[1],
        alloc::vec![observation(1, 1, 1, Some(BindingQuadState::F))],
        alloc::vec![observation(1, 1, 2, Some(BindingQuadState::T))],
    )
    .unwrap();
    assert_eq!(result.dirty_bindings().len(), 1);
    assert_eq!(result.propagation_work(), 0);
    assert_eq!(
        reason_kinds(&result, 1),
        alloc::vec![
            DirtyReasonKind::RevisionChanged,
            DirtyReasonKind::QuadValueChanged,
        ]
    );
}
