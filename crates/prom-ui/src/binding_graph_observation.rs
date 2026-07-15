//! Crate-private Binding Graph observation and dirty derivation.
#![allow(dead_code)]

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::vec::Vec;

use crate::binding_graph::{BindingGraphDocument, BindingId, BindingValueDomain};
use crate::contract_primitives::{Epoch, Revision};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BindingObservationScope {
    binding_ids: Vec<BindingId>,
}

impl BindingObservationScope {
    pub(crate) fn new(binding_ids: Vec<BindingId>) -> Self {
        Self { binding_ids }
    }

    pub(crate) fn binding_ids(&self) -> &[BindingId] {
        &self.binding_ids
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BindingQuadState {
    N,
    F,
    T,
    S,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BindingObservation {
    binding_id: BindingId,
    epoch: Epoch,
    revision: Revision,
    quad_state: Option<BindingQuadState>,
}

impl BindingObservation {
    pub(crate) const fn new(
        binding_id: BindingId,
        epoch: Epoch,
        revision: Revision,
        quad_state: Option<BindingQuadState>,
    ) -> Self {
        Self {
            binding_id,
            epoch,
            revision,
            quad_state,
        }
    }

    pub(crate) const fn binding_id(self) -> BindingId {
        self.binding_id
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
pub(crate) struct BindingObservationSet {
    observations: Vec<BindingObservation>,
}

impl BindingObservationSet {
    pub(crate) fn new(observations: Vec<BindingObservation>) -> Self {
        Self { observations }
    }

    pub(crate) fn observations(&self) -> &[BindingObservation] {
        &self.observations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BindingObservationLimits {
    max_scoped_bindings: usize,
    max_observations_per_snapshot: usize,
    max_dirty_bindings: usize,
    max_propagation_work: usize,
    max_retained_reasons: usize,
}

impl BindingObservationLimits {
    pub(crate) const fn new(
        max_scoped_bindings: usize,
        max_observations_per_snapshot: usize,
        max_dirty_bindings: usize,
        max_propagation_work: usize,
        max_retained_reasons: usize,
    ) -> Self {
        Self {
            max_scoped_bindings,
            max_observations_per_snapshot,
            max_dirty_bindings,
            max_propagation_work,
            max_retained_reasons,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BindingObservationStage {
    ResourcePreflight,
    StructuralValidation,
    DomainValidation,
    TemporalValidation,
    DirtySeedDerivation,
    DependencyPropagation,
    CanonicalResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BindingObservationErrorKind {
    ResourceLimitExceeded,
    DuplicateScopeBinding,
    UnknownScopeBinding,
    DuplicateObservation,
    UnknownBinding,
    ObservationOutOfScope,
    MissingQuadState,
    UnexpectedQuadState,
    StaleObservation,
    ConflictingReplay,
    DirtyResultLimitExceeded,
    PropagationWorkLimitExceeded,
    RetainedReasonLimitExceeded,
}

impl BindingObservationErrorKind {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::ResourceLimitExceeded => "BGOD_RESOURCE_LIMIT_EXCEEDED",
            Self::DuplicateScopeBinding => "BGOD_DUPLICATE_SCOPE_BINDING",
            Self::UnknownScopeBinding => "BGOD_UNKNOWN_SCOPE_BINDING",
            Self::DuplicateObservation => "BGOD_DUPLICATE_OBSERVATION",
            Self::UnknownBinding => "BGOD_UNKNOWN_BINDING",
            Self::ObservationOutOfScope => "BGOD_OBSERVATION_OUT_OF_SCOPE",
            Self::MissingQuadState => "BGOD_MISSING_QUAD_STATE",
            Self::UnexpectedQuadState => "BGOD_UNEXPECTED_QUAD_STATE",
            Self::StaleObservation => "BGOD_STALE_OBSERVATION",
            Self::ConflictingReplay => "BGOD_CONFLICTING_REPLAY",
            Self::DirtyResultLimitExceeded => "BGOD_DIRTY_RESULT_LIMIT_EXCEEDED",
            Self::PropagationWorkLimitExceeded => "BGOD_PROPAGATION_WORK_LIMIT_EXCEEDED",
            Self::RetainedReasonLimitExceeded => "BGOD_RETAINED_REASON_LIMIT_EXCEEDED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BindingObservationLimitKind {
    ScopedBindings,
    PreviousObservations,
    CurrentObservations,
    CombinedInputs,
    DirtyBindings,
    PropagationWork,
    RetainedReasons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BindingObservationError {
    stage: BindingObservationStage,
    binding_id: Option<BindingId>,
    kind: BindingObservationErrorKind,
    related_binding_id: Option<BindingId>,
    limit_kind: Option<BindingObservationLimitKind>,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl BindingObservationError {
    const fn content(
        stage: BindingObservationStage,
        kind: BindingObservationErrorKind,
        binding_id: BindingId,
    ) -> Self {
        Self {
            stage,
            binding_id: Some(binding_id),
            kind,
            related_binding_id: None,
            limit_kind: None,
            actual: None,
            maximum: None,
        }
    }

    const fn limit(
        stage: BindingObservationStage,
        kind: BindingObservationErrorKind,
        binding_id: Option<BindingId>,
        limit_kind: BindingObservationLimitKind,
        actual: usize,
        maximum: usize,
    ) -> Self {
        Self {
            stage,
            binding_id,
            kind,
            related_binding_id: None,
            limit_kind: Some(limit_kind),
            actual: Some(actual),
            maximum: Some(maximum),
        }
    }

    pub(crate) const fn stage(self) -> BindingObservationStage {
        self.stage
    }

    pub(crate) const fn kind(self) -> BindingObservationErrorKind {
        self.kind
    }

    pub(crate) const fn code(self) -> &'static str {
        self.kind.code()
    }

    pub(crate) const fn binding_id(self) -> Option<BindingId> {
        self.binding_id
    }

    pub(crate) const fn related_binding_id(self) -> Option<BindingId> {
        self.related_binding_id
    }

    pub(crate) const fn limit_kind(self) -> Option<BindingObservationLimitKind> {
        self.limit_kind
    }

    pub(crate) const fn actual(self) -> Option<usize> {
        self.actual
    }

    pub(crate) const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DirtyReasonKind {
    Added,
    Removed,
    RevisionChanged,
    EpochChanged,
    QuadValueChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DirtyReason {
    kind: DirtyReasonKind,
    originating_binding_id: BindingId,
    previous_quad_state: Option<BindingQuadState>,
    current_quad_state: Option<BindingQuadState>,
}

impl DirtyReason {
    const fn new(
        kind: DirtyReasonKind,
        originating_binding_id: BindingId,
        previous_quad_state: Option<BindingQuadState>,
        current_quad_state: Option<BindingQuadState>,
    ) -> Self {
        Self {
            kind,
            originating_binding_id,
            previous_quad_state,
            current_quad_state,
        }
    }

    pub(crate) const fn kind(self) -> DirtyReasonKind {
        self.kind
    }

    pub(crate) const fn originating_binding_id(self) -> BindingId {
        self.originating_binding_id
    }

    pub(crate) const fn previous_quad_state(self) -> Option<BindingQuadState> {
        self.previous_quad_state
    }

    pub(crate) const fn current_quad_state(self) -> Option<BindingQuadState> {
        self.current_quad_state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirtyBinding {
    binding_id: BindingId,
    reasons: Vec<DirtyReason>,
}

impl DirtyBinding {
    pub(crate) const fn binding_id(&self) -> BindingId {
        self.binding_id
    }

    pub(crate) fn reasons(&self) -> &[DirtyReason] {
        &self.reasons
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirtyDerivationResult {
    dirty_bindings: Vec<DirtyBinding>,
    propagation_work: usize,
}

impl DirtyDerivationResult {
    pub(crate) fn dirty_bindings(&self) -> &[DirtyBinding] {
        &self.dirty_bindings
    }

    pub(crate) const fn propagation_work(&self) -> usize {
        self.propagation_work
    }
}

pub(crate) fn derive_dirty_bindings(
    graph: &BindingGraphDocument,
    scope: BindingObservationScope,
    previous: BindingObservationSet,
    current: BindingObservationSet,
    limits: BindingObservationLimits,
) -> Result<DirtyDerivationResult, Vec<BindingObservationError>> {
    let mut errors = resource_preflight(&scope, &previous, &current, limits);
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    let mut scoped_ids = scope.binding_ids;
    let mut previous_observations = previous.observations;
    let mut current_observations = current.observations;
    scoped_ids.sort();
    previous_observations.sort_by_key(|observation| observation.binding_id);
    current_observations.sort_by_key(|observation| observation.binding_id);

    let graph_domains: BTreeMap<_, _> = graph
        .bindings
        .iter()
        .map(|binding| (binding.id, binding.domain))
        .collect();
    let scoped_set: BTreeSet<_> = scoped_ids.iter().copied().collect();

    collect_duplicates(
        &scoped_ids,
        BindingObservationStage::StructuralValidation,
        BindingObservationErrorKind::DuplicateScopeBinding,
        &mut errors,
    );
    for binding_id in scoped_set.iter().copied() {
        if !graph_domains.contains_key(&binding_id) {
            errors.push(BindingObservationError::content(
                BindingObservationStage::StructuralValidation,
                BindingObservationErrorKind::UnknownScopeBinding,
                binding_id,
            ));
        }
    }
    validate_observation_structure(
        &previous_observations,
        &graph_domains,
        &scoped_set,
        &mut errors,
    );
    validate_observation_structure(
        &current_observations,
        &graph_domains,
        &scoped_set,
        &mut errors,
    );
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    validate_domains(&previous_observations, &graph_domains, &mut errors);
    validate_domains(&current_observations, &graph_domains, &mut errors);
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    let previous_by_id: BTreeMap<_, _> = previous_observations
        .into_iter()
        .map(|observation| (observation.binding_id, observation))
        .collect();
    let current_by_id: BTreeMap<_, _> = current_observations
        .into_iter()
        .map(|observation| (observation.binding_id, observation))
        .collect();
    let mut seed_reasons = BTreeMap::<BindingId, Vec<DirtyReason>>::new();

    for binding_id in scoped_set.iter().copied() {
        let previous = previous_by_id.get(&binding_id).copied();
        let current = current_by_id.get(&binding_id).copied();
        match (previous, current) {
            (None, None) => {}
            (None, Some(current)) => {
                seed_reasons.insert(
                    binding_id,
                    alloc::vec![DirtyReason::new(
                        DirtyReasonKind::Added,
                        binding_id,
                        None,
                        current.quad_state,
                    )],
                );
            }
            (Some(previous), None) => {
                seed_reasons.insert(
                    binding_id,
                    alloc::vec![DirtyReason::new(
                        DirtyReasonKind::Removed,
                        binding_id,
                        previous.quad_state,
                        None,
                    )],
                );
            }
            (Some(previous), Some(current)) => {
                let previous_tuple = (previous.epoch, previous.revision);
                let current_tuple = (current.epoch, current.revision);
                if current_tuple < previous_tuple {
                    errors.push(BindingObservationError::content(
                        BindingObservationStage::TemporalValidation,
                        BindingObservationErrorKind::StaleObservation,
                        binding_id,
                    ));
                } else if current_tuple == previous_tuple {
                    if current.quad_state != previous.quad_state {
                        errors.push(BindingObservationError::content(
                            BindingObservationStage::TemporalValidation,
                            BindingObservationErrorKind::ConflictingReplay,
                            binding_id,
                        ));
                    }
                } else {
                    let mut reasons = Vec::new();
                    if current.revision != previous.revision {
                        reasons.push(DirtyReason::new(
                            DirtyReasonKind::RevisionChanged,
                            binding_id,
                            previous.quad_state,
                            current.quad_state,
                        ));
                    }
                    if current.epoch != previous.epoch {
                        reasons.push(DirtyReason::new(
                            DirtyReasonKind::EpochChanged,
                            binding_id,
                            previous.quad_state,
                            current.quad_state,
                        ));
                    }
                    if current.quad_state != previous.quad_state {
                        reasons.push(DirtyReason::new(
                            DirtyReasonKind::QuadValueChanged,
                            binding_id,
                            previous.quad_state,
                            current.quad_state,
                        ));
                    }
                    reasons.sort();
                    reasons.dedup();
                    if !reasons.is_empty() {
                        seed_reasons.insert(binding_id, reasons);
                    }
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(canonical_errors(errors));
    }

    if seed_reasons.len() > limits.max_dirty_bindings {
        return Err(alloc::vec![BindingObservationError::limit(
            BindingObservationStage::DirtySeedDerivation,
            BindingObservationErrorKind::DirtyResultLimitExceeded,
            seed_reasons.keys().nth(limits.max_dirty_bindings).copied(),
            BindingObservationLimitKind::DirtyBindings,
            seed_reasons.len(),
            limits.max_dirty_bindings,
        )]);
    }

    let mut dirty_ids: BTreeSet<_> = seed_reasons.keys().copied().collect();
    let mut retained_reason_count = 0usize;
    let mut retained_reason_overflow = false;
    let mut reasons_by_binding = BTreeMap::<BindingId, BTreeSet<DirtyReason>>::new();
    for (binding_id, reasons) in &seed_reasons {
        retain_reasons(
            *binding_id,
            reasons,
            limits.max_retained_reasons,
            &mut retained_reason_count,
            &mut retained_reason_overflow,
            &mut reasons_by_binding,
        );
    }

    let mut reverse_dependencies = BTreeMap::<BindingId, Vec<BindingId>>::new();
    for binding in &graph.bindings {
        for dependency in &binding.dependencies {
            reverse_dependencies
                .entry(dependency.target_binding)
                .or_default()
                .push(binding.id);
        }
    }
    for dependents in reverse_dependencies.values_mut() {
        dependents.sort();
        dependents.dedup();
    }

    let mut propagation_work = 0usize;
    let mut propagation_work_overflow = false;
    for (origin, reasons) in &seed_reasons {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        visited.insert(*origin);
        queue.push_back(*origin);

        while let Some(binding_id) = queue.pop_front() {
            if let Some(dependents) = reverse_dependencies.get(&binding_id) {
                for dependent in dependents.iter().copied() {
                    propagation_work = propagation_work.checked_add(1).unwrap_or(usize::MAX);
                    if propagation_work > limits.max_propagation_work {
                        propagation_work_overflow = true;
                    }

                    if visited.insert(dependent) {
                        if !dirty_ids.contains(&dependent)
                            && dirty_ids.len() >= limits.max_dirty_bindings
                        {
                            let actual = dirty_ids.len().checked_add(1).unwrap_or(usize::MAX);
                            return Err(alloc::vec![BindingObservationError::limit(
                                BindingObservationStage::DependencyPropagation,
                                BindingObservationErrorKind::DirtyResultLimitExceeded,
                                Some(dependent),
                                BindingObservationLimitKind::DirtyBindings,
                                actual,
                                limits.max_dirty_bindings,
                            )]);
                        }
                        dirty_ids.insert(dependent);
                        queue.push_back(dependent);
                        retain_reasons(
                            dependent,
                            reasons,
                            limits.max_retained_reasons,
                            &mut retained_reason_count,
                            &mut retained_reason_overflow,
                            &mut reasons_by_binding,
                        );
                    }
                }
            }
        }
    }

    if propagation_work_overflow {
        return Err(alloc::vec![BindingObservationError::limit(
            BindingObservationStage::DependencyPropagation,
            BindingObservationErrorKind::PropagationWorkLimitExceeded,
            None,
            BindingObservationLimitKind::PropagationWork,
            propagation_work,
            limits.max_propagation_work,
        )]);
    }

    if retained_reason_overflow {
        return Err(alloc::vec![BindingObservationError::limit(
            BindingObservationStage::CanonicalResult,
            BindingObservationErrorKind::RetainedReasonLimitExceeded,
            None,
            BindingObservationLimitKind::RetainedReasons,
            retained_reason_count,
            limits.max_retained_reasons,
        )]);
    }

    let dirty_bindings = dirty_ids
        .into_iter()
        .map(|binding_id| DirtyBinding {
            binding_id,
            reasons: reasons_by_binding
                .remove(&binding_id)
                .unwrap_or_default()
                .into_iter()
                .collect(),
        })
        .collect();

    Ok(DirtyDerivationResult {
        dirty_bindings,
        propagation_work,
    })
}

fn resource_preflight(
    scope: &BindingObservationScope,
    previous: &BindingObservationSet,
    current: &BindingObservationSet,
    limits: BindingObservationLimits,
) -> Vec<BindingObservationError> {
    let mut errors = Vec::new();
    push_input_limit_error(
        scope.binding_ids.len(),
        limits.max_scoped_bindings,
        BindingObservationLimitKind::ScopedBindings,
        &mut errors,
    );
    push_input_limit_error(
        previous.observations.len(),
        limits.max_observations_per_snapshot,
        BindingObservationLimitKind::PreviousObservations,
        &mut errors,
    );
    push_input_limit_error(
        current.observations.len(),
        limits.max_observations_per_snapshot,
        BindingObservationLimitKind::CurrentObservations,
        &mut errors,
    );

    if scope
        .binding_ids
        .len()
        .checked_add(previous.observations.len())
        .and_then(|count| count.checked_add(current.observations.len()))
        .is_none()
    {
        errors.push(BindingObservationError::limit(
            BindingObservationStage::ResourcePreflight,
            BindingObservationErrorKind::ResourceLimitExceeded,
            None,
            BindingObservationLimitKind::CombinedInputs,
            usize::MAX,
            usize::MAX,
        ));
    }
    errors
}

fn push_input_limit_error(
    actual: usize,
    maximum: usize,
    limit_kind: BindingObservationLimitKind,
    errors: &mut Vec<BindingObservationError>,
) {
    if actual > maximum {
        errors.push(BindingObservationError::limit(
            BindingObservationStage::ResourcePreflight,
            BindingObservationErrorKind::ResourceLimitExceeded,
            None,
            limit_kind,
            actual,
            maximum,
        ));
    }
}

fn collect_duplicates(
    binding_ids: &[BindingId],
    stage: BindingObservationStage,
    kind: BindingObservationErrorKind,
    errors: &mut Vec<BindingObservationError>,
) {
    for pair in binding_ids.windows(2) {
        if pair[0] == pair[1] {
            errors.push(BindingObservationError::content(stage, kind, pair[0]));
        }
    }
}

fn validate_observation_structure(
    observations: &[BindingObservation],
    graph_domains: &BTreeMap<BindingId, BindingValueDomain>,
    scoped_ids: &BTreeSet<BindingId>,
    errors: &mut Vec<BindingObservationError>,
) {
    let observation_ids: Vec<_> = observations
        .iter()
        .map(|observation| observation.binding_id)
        .collect();
    collect_duplicates(
        &observation_ids,
        BindingObservationStage::StructuralValidation,
        BindingObservationErrorKind::DuplicateObservation,
        errors,
    );
    for binding_id in observation_ids {
        if !graph_domains.contains_key(&binding_id) {
            errors.push(BindingObservationError::content(
                BindingObservationStage::StructuralValidation,
                BindingObservationErrorKind::UnknownBinding,
                binding_id,
            ));
        } else if !scoped_ids.contains(&binding_id) {
            errors.push(BindingObservationError::content(
                BindingObservationStage::StructuralValidation,
                BindingObservationErrorKind::ObservationOutOfScope,
                binding_id,
            ));
        }
    }
}

fn validate_domains(
    observations: &[BindingObservation],
    graph_domains: &BTreeMap<BindingId, BindingValueDomain>,
    errors: &mut Vec<BindingObservationError>,
) {
    for observation in observations {
        match (
            graph_domains.get(&observation.binding_id),
            observation.quad_state,
        ) {
            (Some(BindingValueDomain::Quad), None) => {
                errors.push(BindingObservationError::content(
                    BindingObservationStage::DomainValidation,
                    BindingObservationErrorKind::MissingQuadState,
                    observation.binding_id,
                ));
            }
            (Some(domain), Some(_)) if *domain != BindingValueDomain::Quad => {
                errors.push(BindingObservationError::content(
                    BindingObservationStage::DomainValidation,
                    BindingObservationErrorKind::UnexpectedQuadState,
                    observation.binding_id,
                ));
            }
            _ => {}
        }
    }
}

fn retain_reasons(
    binding_id: BindingId,
    reasons: &[DirtyReason],
    maximum: usize,
    actual: &mut usize,
    overflow: &mut bool,
    reasons_by_binding: &mut BTreeMap<BindingId, BTreeSet<DirtyReason>>,
) {
    *actual = actual.checked_add(reasons.len()).unwrap_or(usize::MAX);
    if *actual > maximum {
        *overflow = true;
        return;
    }
    reasons_by_binding
        .entry(binding_id)
        .or_default()
        .extend(reasons.iter().copied());
}

fn canonical_errors(mut errors: Vec<BindingObservationError>) -> Vec<BindingObservationError> {
    errors.sort();
    errors.dedup();
    errors
}
