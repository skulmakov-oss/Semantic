//! Crate-private adapter from caller-supplied Semantic reference observations.
#![allow(dead_code)]

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::binding_graph::{BindingGraphDocument, BindingId, BindingValueDomain};
use crate::binding_graph_observation::{
    BindingObservation, BindingObservationScope, BindingObservationSet, BindingQuadState,
};
use crate::contract_primitives::{Epoch, Revision};
use crate::semantic_refs::SemanticValueRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SemanticBindingObservation {
    semantic_value_ref: SemanticValueRef,
    epoch: Epoch,
    revision: Revision,
    quad_state: Option<BindingQuadState>,
}

impl SemanticBindingObservation {
    pub(crate) const fn new(
        semantic_value_ref: SemanticValueRef,
        epoch: Epoch,
        revision: Revision,
        quad_state: Option<BindingQuadState>,
    ) -> Self {
        Self {
            semantic_value_ref,
            epoch,
            revision,
            quad_state,
        }
    }

    pub(crate) const fn semantic_value_ref(self) -> SemanticValueRef {
        self.semantic_value_ref
    }

    pub(crate) const fn epoch(self) -> Epoch {
        self.epoch
    }

    pub(crate) const fn revision(self) -> Revision {
        self.revision
    }

    pub(crate) const fn quad_state(self) -> Option<BindingQuadState> {
        self.quad_state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SemanticBindingObservationSet {
    observations: Vec<SemanticBindingObservation>,
}

impl SemanticBindingObservationSet {
    pub(crate) fn new(observations: Vec<SemanticBindingObservation>) -> Self {
        Self { observations }
    }

    pub(crate) fn observations(&self) -> &[SemanticBindingObservation] {
        &self.observations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SemanticBindingAdapterLimits {
    max_scoped_bindings: usize,
    max_observations_per_snapshot: usize,
    max_indexed_semantic_references: usize,
    max_fanout_mappings: usize,
    max_mapped_observations_per_snapshot: usize,
}

impl SemanticBindingAdapterLimits {
    pub(crate) const fn new(
        max_scoped_bindings: usize,
        max_observations_per_snapshot: usize,
        max_indexed_semantic_references: usize,
        max_fanout_mappings: usize,
        max_mapped_observations_per_snapshot: usize,
    ) -> Self {
        Self {
            max_scoped_bindings,
            max_observations_per_snapshot,
            max_indexed_semantic_references,
            max_fanout_mappings,
            max_mapped_observations_per_snapshot,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SemanticBindingAdapterStage {
    ResourcePreflight,
    ScopeValidation,
    ReferenceIndexValidation,
    SnapshotStructuralValidation,
    DomainCarrierValidation,
    CanonicalObservationMapping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SemanticBindingAdapterErrorKind {
    ResourceLimitExceeded,
    DuplicateScopeBinding,
    UnknownScopeBinding,
    BindingMissingSemanticRef,
    IncompatibleFanoutDomain,
    DuplicateSemanticObservation,
    UnknownSemanticRef,
    ObservationOutOfScope,
    MissingQuadState,
    UnexpectedQuadState,
    FanoutLimitExceeded,
    MappedObservationLimitExceeded,
}

impl SemanticBindingAdapterErrorKind {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::ResourceLimitExceeded => "BGSA_RESOURCE_LIMIT_EXCEEDED",
            Self::DuplicateScopeBinding => "BGSA_DUPLICATE_SCOPE_BINDING",
            Self::UnknownScopeBinding => "BGSA_UNKNOWN_SCOPE_BINDING",
            Self::BindingMissingSemanticRef => "BGSA_BINDING_MISSING_SEMANTIC_REF",
            Self::IncompatibleFanoutDomain => "BGSA_INCOMPATIBLE_FANOUT_DOMAIN",
            Self::DuplicateSemanticObservation => "BGSA_DUPLICATE_SEMANTIC_OBSERVATION",
            Self::UnknownSemanticRef => "BGSA_UNKNOWN_SEMANTIC_REF",
            Self::ObservationOutOfScope => "BGSA_OBSERVATION_OUT_OF_SCOPE",
            Self::MissingQuadState => "BGSA_MISSING_QUAD_STATE",
            Self::UnexpectedQuadState => "BGSA_UNEXPECTED_QUAD_STATE",
            Self::FanoutLimitExceeded => "BGSA_FANOUT_LIMIT_EXCEEDED",
            Self::MappedObservationLimitExceeded => "BGSA_MAPPED_OBSERVATION_LIMIT_EXCEEDED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SemanticBindingAdapterLimitKind {
    ScopedBindings,
    PreviousObservations,
    CurrentObservations,
    CombinedInputs,
    IndexedSemanticReferences,
    FanoutMappings,
    MappedPreviousObservations,
    MappedCurrentObservations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SemanticBindingSnapshotSide {
    Previous,
    Current,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SemanticBindingAdapterError {
    stage: SemanticBindingAdapterStage,
    kind: SemanticBindingAdapterErrorKind,
    snapshot_side: Option<SemanticBindingSnapshotSide>,
    semantic_value_ref: Option<SemanticValueRef>,
    binding_id: Option<BindingId>,
    limit_kind: Option<SemanticBindingAdapterLimitKind>,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl SemanticBindingAdapterError {
    const fn content(
        stage: SemanticBindingAdapterStage,
        kind: SemanticBindingAdapterErrorKind,
        snapshot_side: Option<SemanticBindingSnapshotSide>,
        semantic_value_ref: Option<SemanticValueRef>,
        binding_id: Option<BindingId>,
    ) -> Self {
        Self {
            stage,
            kind,
            snapshot_side,
            semantic_value_ref,
            binding_id,
            limit_kind: None,
            actual: None,
            maximum: None,
        }
    }

    const fn limit(
        stage: SemanticBindingAdapterStage,
        kind: SemanticBindingAdapterErrorKind,
        snapshot_side: Option<SemanticBindingSnapshotSide>,
        semantic_value_ref: Option<SemanticValueRef>,
        binding_id: Option<BindingId>,
        limit: (SemanticBindingAdapterLimitKind, usize, usize),
    ) -> Self {
        Self {
            stage,
            kind,
            snapshot_side,
            semantic_value_ref,
            binding_id,
            limit_kind: Some(limit.0),
            actual: Some(limit.1),
            maximum: Some(limit.2),
        }
    }

    pub(crate) const fn stage(self) -> SemanticBindingAdapterStage {
        self.stage
    }

    pub(crate) const fn kind(self) -> SemanticBindingAdapterErrorKind {
        self.kind
    }

    pub(crate) const fn code(self) -> &'static str {
        self.kind.code()
    }

    pub(crate) const fn snapshot_side(self) -> Option<SemanticBindingSnapshotSide> {
        self.snapshot_side
    }

    pub(crate) const fn semantic_value_ref(self) -> Option<SemanticValueRef> {
        self.semantic_value_ref
    }

    pub(crate) const fn binding_id(self) -> Option<BindingId> {
        self.binding_id
    }

    pub(crate) const fn limit_kind(self) -> Option<SemanticBindingAdapterLimitKind> {
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
pub(crate) struct AdaptedBindingObservationInputs {
    scope: BindingObservationScope,
    previous: BindingObservationSet,
    current: BindingObservationSet,
    fanout_work: usize,
}

impl AdaptedBindingObservationInputs {
    pub(crate) fn scope(&self) -> &BindingObservationScope {
        &self.scope
    }

    pub(crate) fn previous(&self) -> &BindingObservationSet {
        &self.previous
    }

    pub(crate) fn current(&self) -> &BindingObservationSet {
        &self.current
    }

    pub(crate) const fn fanout_work(&self) -> usize {
        self.fanout_work
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        BindingObservationScope,
        BindingObservationSet,
        BindingObservationSet,
    ) {
        (self.scope, self.previous, self.current)
    }
}

pub(crate) fn adapt_semantic_observations(
    graph: &BindingGraphDocument,
    scope: BindingObservationScope,
    previous: SemanticBindingObservationSet,
    current: SemanticBindingObservationSet,
    limits: SemanticBindingAdapterLimits,
) -> Result<AdaptedBindingObservationInputs, Vec<SemanticBindingAdapterError>> {
    let errors = resource_preflight(&scope, &previous, &current, limits);
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    let mut scoped_ids = scope.binding_ids().to_vec();
    scoped_ids.sort();
    let errors = validate_scope(graph, &scoped_ids);
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    let graph_refs = index_graph_references(graph, limits.max_indexed_semantic_references)?;
    let (fanout, fanout_work) =
        build_scoped_fanout(graph, &scoped_ids, limits.max_fanout_mappings)?;

    let mut previous_observations = previous.observations;
    let mut current_observations = current.observations;
    previous_observations.sort_by_key(|observation| observation.semantic_value_ref);
    current_observations.sort_by_key(|observation| observation.semantic_value_ref);

    let mut errors = validate_snapshot_structure(
        &previous_observations,
        &graph_refs,
        &fanout,
        SemanticBindingSnapshotSide::Previous,
    );
    errors.extend(validate_snapshot_structure(
        &current_observations,
        &graph_refs,
        &fanout,
        SemanticBindingSnapshotSide::Current,
    ));
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    let mut errors = validate_domain_carriers(
        &previous_observations,
        &fanout,
        SemanticBindingSnapshotSide::Previous,
    );
    errors.extend(validate_domain_carriers(
        &current_observations,
        &fanout,
        SemanticBindingSnapshotSide::Current,
    ));
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    let previous_mapped = map_snapshot(
        &previous_observations,
        &fanout,
        limits.max_mapped_observations_per_snapshot,
        SemanticBindingSnapshotSide::Previous,
    );
    let current_mapped = map_snapshot(
        &current_observations,
        &fanout,
        limits.max_mapped_observations_per_snapshot,
        SemanticBindingSnapshotSide::Current,
    );
    let mut mapping_errors = Vec::new();
    let previous_mapped = match previous_mapped {
        Ok(mapped) => Some(mapped),
        Err(error) => {
            mapping_errors.push(error);
            None
        }
    };
    let current_mapped = match current_mapped {
        Ok(mapped) => Some(mapped),
        Err(error) => {
            mapping_errors.push(error);
            None
        }
    };
    if !mapping_errors.is_empty() {
        return Err(canonical_errors(mapping_errors));
    }

    Ok(AdaptedBindingObservationInputs {
        scope: BindingObservationScope::new(scoped_ids),
        previous: BindingObservationSet::new(previous_mapped.unwrap_or_default()),
        current: BindingObservationSet::new(current_mapped.unwrap_or_default()),
        fanout_work,
    })
}

fn resource_preflight(
    scope: &BindingObservationScope,
    previous: &SemanticBindingObservationSet,
    current: &SemanticBindingObservationSet,
    limits: SemanticBindingAdapterLimits,
) -> Vec<SemanticBindingAdapterError> {
    let mut errors = Vec::new();
    push_preflight_limit(
        scope.binding_ids().len(),
        limits.max_scoped_bindings,
        SemanticBindingAdapterLimitKind::ScopedBindings,
        None,
        &mut errors,
    );
    push_preflight_limit(
        previous.observations.len(),
        limits.max_observations_per_snapshot,
        SemanticBindingAdapterLimitKind::PreviousObservations,
        Some(SemanticBindingSnapshotSide::Previous),
        &mut errors,
    );
    push_preflight_limit(
        current.observations.len(),
        limits.max_observations_per_snapshot,
        SemanticBindingAdapterLimitKind::CurrentObservations,
        Some(SemanticBindingSnapshotSide::Current),
        &mut errors,
    );
    if scope
        .binding_ids()
        .len()
        .checked_add(previous.observations.len())
        .and_then(|count| count.checked_add(current.observations.len()))
        .is_none()
    {
        errors.push(SemanticBindingAdapterError::limit(
            SemanticBindingAdapterStage::ResourcePreflight,
            SemanticBindingAdapterErrorKind::ResourceLimitExceeded,
            None,
            None,
            None,
            (
                SemanticBindingAdapterLimitKind::CombinedInputs,
                usize::MAX,
                usize::MAX,
            ),
        ));
    }
    errors
}

fn push_preflight_limit(
    actual: usize,
    maximum: usize,
    limit_kind: SemanticBindingAdapterLimitKind,
    snapshot_side: Option<SemanticBindingSnapshotSide>,
    errors: &mut Vec<SemanticBindingAdapterError>,
) {
    if actual > maximum {
        errors.push(SemanticBindingAdapterError::limit(
            SemanticBindingAdapterStage::ResourcePreflight,
            SemanticBindingAdapterErrorKind::ResourceLimitExceeded,
            snapshot_side,
            None,
            None,
            (limit_kind, actual, maximum),
        ));
    }
}

fn validate_scope(
    graph: &BindingGraphDocument,
    scoped_ids: &[BindingId],
) -> Vec<SemanticBindingAdapterError> {
    let mut errors = Vec::new();
    for pair in scoped_ids.windows(2) {
        if pair[0] == pair[1] {
            errors.push(SemanticBindingAdapterError::content(
                SemanticBindingAdapterStage::ScopeValidation,
                SemanticBindingAdapterErrorKind::DuplicateScopeBinding,
                None,
                None,
                Some(pair[0]),
            ));
        }
    }
    for binding_id in scoped_ids.iter().copied() {
        match graph
            .bindings
            .iter()
            .find(|binding| binding.id == binding_id)
        {
            None => errors.push(SemanticBindingAdapterError::content(
                SemanticBindingAdapterStage::ScopeValidation,
                SemanticBindingAdapterErrorKind::UnknownScopeBinding,
                None,
                None,
                Some(binding_id),
            )),
            Some(binding) if binding.semantic_value.is_none() => {
                errors.push(SemanticBindingAdapterError::content(
                    SemanticBindingAdapterStage::ScopeValidation,
                    SemanticBindingAdapterErrorKind::BindingMissingSemanticRef,
                    None,
                    None,
                    Some(binding_id),
                ));
            }
            Some(_) => {}
        }
    }
    errors
}

fn index_graph_references(
    graph: &BindingGraphDocument,
    maximum: usize,
) -> Result<BTreeSet<SemanticValueRef>, Vec<SemanticBindingAdapterError>> {
    let mut references = BTreeSet::new();
    for binding in &graph.bindings {
        let Some(semantic_ref) = binding.semantic_value else {
            continue;
        };
        if references.contains(&semantic_ref) {
            continue;
        }
        let actual = checked_increment(references.len());
        if actual > maximum {
            return Err(alloc::vec![SemanticBindingAdapterError::limit(
                SemanticBindingAdapterStage::ReferenceIndexValidation,
                SemanticBindingAdapterErrorKind::ResourceLimitExceeded,
                None,
                Some(semantic_ref),
                Some(binding.id),
                (
                    SemanticBindingAdapterLimitKind::IndexedSemanticReferences,
                    actual,
                    maximum,
                ),
            )]);
        }
        references.insert(semantic_ref);
    }
    Ok(references)
}

type FanoutIndex = BTreeMap<SemanticValueRef, Vec<(BindingId, BindingValueDomain)>>;

fn build_scoped_fanout(
    graph: &BindingGraphDocument,
    scoped_ids: &[BindingId],
    maximum: usize,
) -> Result<(FanoutIndex, usize), Vec<SemanticBindingAdapterError>> {
    let mut fanout = FanoutIndex::new();
    let mut work = 0usize;
    for binding_id in scoped_ids.iter().copied() {
        let Some(binding) = graph
            .bindings
            .iter()
            .find(|binding| binding.id == binding_id)
        else {
            continue;
        };
        let Some(semantic_ref) = binding.semantic_value else {
            continue;
        };
        let actual = checked_increment(work);
        if actual > maximum {
            return Err(alloc::vec![SemanticBindingAdapterError::limit(
                SemanticBindingAdapterStage::ReferenceIndexValidation,
                SemanticBindingAdapterErrorKind::FanoutLimitExceeded,
                None,
                Some(semantic_ref),
                Some(binding_id),
                (
                    SemanticBindingAdapterLimitKind::FanoutMappings,
                    actual,
                    maximum,
                ),
            )]);
        }
        work = actual;
        fanout
            .entry(semantic_ref)
            .or_default()
            .push((binding_id, binding.domain));
    }

    let errors: Vec<_> = fanout
        .iter()
        .filter_map(|(semantic_ref, bindings)| {
            let has_quad = bindings
                .iter()
                .any(|(_, domain)| *domain == BindingValueDomain::Quad);
            let has_non_quad = bindings
                .iter()
                .any(|(_, domain)| *domain != BindingValueDomain::Quad);
            (has_quad && has_non_quad).then(|| {
                SemanticBindingAdapterError::content(
                    SemanticBindingAdapterStage::ReferenceIndexValidation,
                    SemanticBindingAdapterErrorKind::IncompatibleFanoutDomain,
                    None,
                    Some(*semantic_ref),
                    None,
                )
            })
        })
        .collect();
    if errors.is_empty() {
        Ok((fanout, work))
    } else {
        Err(canonical_errors(errors))
    }
}

fn validate_snapshot_structure(
    observations: &[SemanticBindingObservation],
    graph_refs: &BTreeSet<SemanticValueRef>,
    fanout: &FanoutIndex,
    side: SemanticBindingSnapshotSide,
) -> Vec<SemanticBindingAdapterError> {
    let mut errors = Vec::new();
    for pair in observations.windows(2) {
        if pair[0].semantic_value_ref == pair[1].semantic_value_ref {
            errors.push(SemanticBindingAdapterError::content(
                SemanticBindingAdapterStage::SnapshotStructuralValidation,
                SemanticBindingAdapterErrorKind::DuplicateSemanticObservation,
                Some(side),
                Some(pair[0].semantic_value_ref),
                None,
            ));
        }
    }
    for observation in observations {
        let semantic_ref = observation.semantic_value_ref;
        let kind = if !graph_refs.contains(&semantic_ref) {
            Some(SemanticBindingAdapterErrorKind::UnknownSemanticRef)
        } else if !fanout.contains_key(&semantic_ref) {
            Some(SemanticBindingAdapterErrorKind::ObservationOutOfScope)
        } else {
            None
        };
        if let Some(kind) = kind {
            errors.push(SemanticBindingAdapterError::content(
                SemanticBindingAdapterStage::SnapshotStructuralValidation,
                kind,
                Some(side),
                Some(semantic_ref),
                None,
            ));
        }
    }
    errors
}

fn validate_domain_carriers(
    observations: &[SemanticBindingObservation],
    fanout: &FanoutIndex,
    side: SemanticBindingSnapshotSide,
) -> Vec<SemanticBindingAdapterError> {
    observations
        .iter()
        .filter_map(|observation| {
            let bindings = fanout.get(&observation.semantic_value_ref)?;
            let is_quad = bindings.first()?.1 == BindingValueDomain::Quad;
            let kind = match (is_quad, observation.quad_state) {
                (true, None) => Some(SemanticBindingAdapterErrorKind::MissingQuadState),
                (false, Some(_)) => Some(SemanticBindingAdapterErrorKind::UnexpectedQuadState),
                _ => None,
            };
            kind.map(|kind| {
                SemanticBindingAdapterError::content(
                    SemanticBindingAdapterStage::DomainCarrierValidation,
                    kind,
                    Some(side),
                    Some(observation.semantic_value_ref),
                    None,
                )
            })
        })
        .collect()
}

fn map_snapshot(
    observations: &[SemanticBindingObservation],
    fanout: &FanoutIndex,
    maximum: usize,
    side: SemanticBindingSnapshotSide,
) -> Result<Vec<BindingObservation>, SemanticBindingAdapterError> {
    let mut mapped = Vec::new();
    for observation in observations {
        let Some(bindings) = fanout.get(&observation.semantic_value_ref) else {
            continue;
        };
        for (binding_id, _) in bindings {
            let actual = checked_increment(mapped.len());
            if actual > maximum {
                let limit_kind = match side {
                    SemanticBindingSnapshotSide::Previous => {
                        SemanticBindingAdapterLimitKind::MappedPreviousObservations
                    }
                    SemanticBindingSnapshotSide::Current => {
                        SemanticBindingAdapterLimitKind::MappedCurrentObservations
                    }
                };
                return Err(SemanticBindingAdapterError::limit(
                    SemanticBindingAdapterStage::CanonicalObservationMapping,
                    SemanticBindingAdapterErrorKind::MappedObservationLimitExceeded,
                    Some(side),
                    Some(observation.semantic_value_ref),
                    Some(*binding_id),
                    (limit_kind, actual, maximum),
                ));
            }
            mapped.push(BindingObservation::new(
                *binding_id,
                observation.epoch,
                observation.revision,
                observation.quad_state,
            ));
        }
    }
    mapped.sort_by_key(|observation| observation.binding_id());
    Ok(mapped)
}

fn checked_increment(value: usize) -> usize {
    match value.checked_add(1) {
        Some(next) => next,
        None => usize::MAX,
    }
}

fn canonical_errors(
    mut errors: Vec<SemanticBindingAdapterError>,
) -> Vec<SemanticBindingAdapterError> {
    errors.sort();
    errors.dedup();
    errors
}
