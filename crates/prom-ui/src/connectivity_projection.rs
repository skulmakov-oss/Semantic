//! Crate-private freshness and critical-control projection.
//!
//! Caller-supplied freshness is presentation evidence. This module does not
//! read connection state, evaluate capability, admit actions, or mutate UI.
#![allow(dead_code)]

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::binding_graph::BindingTarget;
use crate::contract_primitives::StaticNodeId;
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatchOperation, ProjectionPatchValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum FreshnessState {
    Fresh,
    Degraded,
    Stale,
    Offline,
    Resyncing,
}

impl FreshnessState {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Degraded => "degraded",
            Self::Stale => "stale",
            Self::Offline => "offline",
            Self::Resyncing => "resyncing",
        }
    }

    pub(crate) const fn restricts_critical_controls(self) -> bool {
        matches!(self, Self::Stale | Self::Offline | Self::Resyncing)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ControlCriticality {
    Normal,
    Guarded,
    Danger,
    TaskControl,
}

impl ControlCriticality {
    const fn is_critical(self) -> bool {
        !matches!(self, Self::Normal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectedControlAvailability {
    Available,
    Unavailable,
    Hidden,
    Pending,
    Stale,
}

impl From<ProjectedControlAvailability> for ProjectionNodeAvailability {
    fn from(value: ProjectedControlAvailability) -> Self {
        match value {
            ProjectedControlAvailability::Available => Self::Available,
            ProjectedControlAvailability::Unavailable => Self::Unavailable,
            ProjectedControlAvailability::Hidden => Self::Hidden,
            ProjectedControlAvailability::Pending => Self::Pending,
            ProjectedControlAvailability::Stale => Self::Stale,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FreshnessControlRoute {
    node: StaticNodeId,
    criticality: ControlCriticality,
    availability: ProjectedControlAvailability,
}

impl FreshnessControlRoute {
    pub(crate) const fn new(
        node: StaticNodeId,
        criticality: ControlCriticality,
        availability: ProjectedControlAvailability,
    ) -> Self {
        Self {
            node,
            criticality,
            availability,
        }
    }

    pub(crate) const fn node(self) -> StaticNodeId {
        self.node
    }

    pub(crate) const fn criticality(self) -> ControlCriticality {
        self.criticality
    }

    pub(crate) const fn availability(self) -> ProjectedControlAvailability {
        self.availability
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FreshnessProjectionRoute {
    freshness_target: BindingTarget,
    controls: Vec<FreshnessControlRoute>,
}

impl FreshnessProjectionRoute {
    pub(crate) fn new(
        freshness_target: BindingTarget,
        controls: Vec<FreshnessControlRoute>,
    ) -> Self {
        Self {
            freshness_target,
            controls,
        }
    }

    pub(crate) fn freshness_target(&self) -> &BindingTarget {
        &self.freshness_target
    }

    pub(crate) fn controls(&self) -> &[FreshnessControlRoute] {
        &self.controls
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FreshnessProjectionLimits {
    max_control_routes: usize,
    max_operations: usize,
}

impl FreshnessProjectionLimits {
    pub(crate) const fn new(max_control_routes: usize, max_operations: usize) -> Self {
        Self {
            max_control_routes,
            max_operations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FreshnessProjectionFragment {
    state: FreshnessState,
    operations: Vec<ProjectionPatchOperation>,
}

impl FreshnessProjectionFragment {
    pub(crate) const fn state(&self) -> FreshnessState {
        self.state
    }

    pub(crate) fn operations(&self) -> &[ProjectionPatchOperation] {
        &self.operations
    }

    pub(crate) fn into_operations(self) -> Vec<ProjectionPatchOperation> {
        self.operations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum FreshnessProjectionDiagnosticKind {
    ResourceLimitExceeded,
    DuplicateControlNode,
    InvalidCriticalControlAvailability,
    OperationLimitExceeded,
}

impl FreshnessProjectionDiagnosticKind {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::ResourceLimitExceeded => "CFP_RESOURCE_LIMIT_EXCEEDED",
            Self::DuplicateControlNode => "CFP_DUPLICATE_CONTROL_NODE",
            Self::InvalidCriticalControlAvailability => "CFP_INVALID_CRITICAL_CONTROL_AVAILABILITY",
            Self::OperationLimitExceeded => "CFP_OPERATION_LIMIT_EXCEEDED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FreshnessProjectionDiagnostic {
    kind: FreshnessProjectionDiagnosticKind,
    node: Option<StaticNodeId>,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl FreshnessProjectionDiagnostic {
    const fn content(kind: FreshnessProjectionDiagnosticKind, node: StaticNodeId) -> Self {
        Self {
            kind,
            node: Some(node),
            actual: None,
            maximum: None,
        }
    }

    const fn limit(kind: FreshnessProjectionDiagnosticKind, actual: usize, maximum: usize) -> Self {
        Self {
            kind,
            node: None,
            actual: Some(actual),
            maximum: Some(maximum),
        }
    }

    pub(crate) const fn kind(self) -> FreshnessProjectionDiagnosticKind {
        self.kind
    }

    pub(crate) const fn code(self) -> &'static str {
        self.kind.code()
    }

    pub(crate) const fn node(self) -> Option<StaticNodeId> {
        self.node
    }

    pub(crate) const fn actual(self) -> Option<usize> {
        self.actual
    }

    pub(crate) const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

pub(crate) fn project_freshness_fragment(
    state: FreshnessState,
    route: &FreshnessProjectionRoute,
    limits: FreshnessProjectionLimits,
) -> Result<FreshnessProjectionFragment, Vec<FreshnessProjectionDiagnostic>> {
    if route.controls.len() > limits.max_control_routes {
        return Err(vec![FreshnessProjectionDiagnostic::limit(
            FreshnessProjectionDiagnosticKind::ResourceLimitExceeded,
            route.controls.len(),
            limits.max_control_routes,
        )]);
    }

    let operation_count = match route.controls.len().checked_add(1) {
        Some(count) => count,
        None => {
            return Err(vec![FreshnessProjectionDiagnostic::limit(
                FreshnessProjectionDiagnosticKind::OperationLimitExceeded,
                usize::MAX,
                limits.max_operations,
            )]);
        }
    };
    if operation_count > limits.max_operations {
        return Err(vec![FreshnessProjectionDiagnostic::limit(
            FreshnessProjectionDiagnosticKind::OperationLimitExceeded,
            operation_count,
            limits.max_operations,
        )]);
    }

    let mut controls = route.controls.clone();
    controls.sort_by_key(|control| control.node);
    let mut diagnostics = Vec::new();
    for (index, control) in controls.iter().enumerate() {
        if index > 0 && controls[index - 1].node == control.node {
            diagnostics.push(FreshnessProjectionDiagnostic::content(
                FreshnessProjectionDiagnosticKind::DuplicateControlNode,
                control.node,
            ));
        }
        if state.restricts_critical_controls()
            && control.criticality.is_critical()
            && control.availability == ProjectedControlAvailability::Available
        {
            diagnostics.push(FreshnessProjectionDiagnostic::content(
                FreshnessProjectionDiagnosticKind::InvalidCriticalControlAvailability,
                control.node,
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut operations = Vec::with_capacity(operation_count);
    operations.push(ProjectionPatchOperation::SetBindingValue {
        target: route.freshness_target.clone(),
        value: ProjectionPatchValue::Text(String::from(state.token())),
    });
    for control in controls {
        operations.push(ProjectionPatchOperation::SetNodeAvailability {
            node: control.node,
            availability: control.availability.into(),
        });
    }

    Ok(FreshnessProjectionFragment { state, operations })
}
