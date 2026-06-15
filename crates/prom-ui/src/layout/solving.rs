use super::*;
use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::projection::UiProjectionArtifactId;
use crate::renderer::{UiRenderModelId, UiRenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiLayoutSolvingModelId(u64);

impl UiLayoutSolvingModelId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiLayoutSolvingEntryId(u64);

impl UiLayoutSolvingEntryId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutSolvingKind {
    DeferredIntent,
    UnavailableResult,
    AuditOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutSolvingState {
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSolvingEntry {
    id: UiLayoutSolvingEntryId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_constraint_declaration: UiLayoutConstraintId,
    source_sizing_entry: UiLayoutSizingEntryId,
    source_sizing_algorithm_entry: UiLayoutSizingAlgorithmEntryId,
    source_measuring_entry: UiLayoutMeasuringEntryId,
    source_size_to_fit_entry: UiLayoutSizeToFitEntryId,
    source_constraint_solver_entry: UiLayoutConstraintSolverEntryId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutSolvingKind,
    state: UiLayoutSolvingState,
    order: usize,
}

impl UiLayoutSolvingEntry {
    pub fn id(&self) -> UiLayoutSolvingEntryId {
        self.id
    }

    pub fn source_layout_node(&self) -> UiLayoutNodeId {
        self.source_layout_node
    }

    pub fn source_layout_slot(&self) -> UiLayoutSlotId {
        self.source_layout_slot
    }

    pub fn source_geometry_node(&self) -> UiLayoutGeometryNodeId {
        self.source_geometry_node
    }

    pub fn source_constraint_declaration(&self) -> UiLayoutConstraintId {
        self.source_constraint_declaration
    }

    pub fn source_sizing_entry(&self) -> UiLayoutSizingEntryId {
        self.source_sizing_entry
    }

    pub fn source_sizing_algorithm_entry(&self) -> UiLayoutSizingAlgorithmEntryId {
        self.source_sizing_algorithm_entry
    }

    pub fn source_measuring_entry(&self) -> UiLayoutMeasuringEntryId {
        self.source_measuring_entry
    }

    pub fn source_size_to_fit_entry(&self) -> UiLayoutSizeToFitEntryId {
        self.source_size_to_fit_entry
    }

    pub fn source_constraint_solver_entry(&self) -> UiLayoutConstraintSolverEntryId {
        self.source_constraint_solver_entry
    }

    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiLayoutSolvingKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutSolvingState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSolvingModel {
    id: UiLayoutSolvingModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_constraints_model: UiLayoutConstraintsModelId,
    source_sizing_model: UiLayoutSizingModelId,
    source_sizing_algorithm_model: UiLayoutSizingAlgorithmModelId,
    source_measuring_model: UiLayoutMeasuringModelId,
    source_size_to_fit_model: UiLayoutSizeToFitModelId,
    source_constraint_solver_model: UiLayoutConstraintSolverModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    entries: Vec<UiLayoutSolvingEntry>,
}

impl UiLayoutSolvingModel {
    pub fn id(&self) -> UiLayoutSolvingModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }

    pub fn source_geometry_model(&self) -> UiLayoutGeometryModelId {
        self.source_geometry_model
    }

    pub fn source_constraints_model(&self) -> UiLayoutConstraintsModelId {
        self.source_constraints_model
    }

    pub fn source_sizing_model(&self) -> UiLayoutSizingModelId {
        self.source_sizing_model
    }

    pub fn source_sizing_algorithm_model(&self) -> UiLayoutSizingAlgorithmModelId {
        self.source_sizing_algorithm_model
    }

    pub fn source_measuring_model(&self) -> UiLayoutMeasuringModelId {
        self.source_measuring_model
    }

    pub fn source_size_to_fit_model(&self) -> UiLayoutSizeToFitModelId {
        self.source_size_to_fit_model
    }

    pub fn source_constraint_solver_model(&self) -> UiLayoutConstraintSolverModelId {
        self.source_constraint_solver_model
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn entries(&self) -> &[UiLayoutSolvingEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub fn build_layout_solving(model: &UiLayoutConstraintSolverModel) -> UiLayoutSolvingModel {
    let mut entries = Vec::with_capacity(model.entries().len());

    for (order, constraint_solver_entry) in model.entries().iter().enumerate() {
        let kind = match order % 3 {
            0 => UiLayoutSolvingKind::DeferredIntent,
            1 => UiLayoutSolvingKind::UnavailableResult,
            _ => UiLayoutSolvingKind::AuditOnly,
        };

        entries.push(UiLayoutSolvingEntry {
            id: UiLayoutSolvingEntryId::new(constraint_solver_entry.id().raw()),
            source_layout_node: constraint_solver_entry.source_layout_node(),
            source_layout_slot: constraint_solver_entry.source_layout_slot(),
            source_geometry_node: constraint_solver_entry.source_geometry_node(),
            source_constraint_declaration: constraint_solver_entry.source_constraint_declaration(),
            source_sizing_entry: constraint_solver_entry.source_sizing_entry(),
            source_sizing_algorithm_entry: constraint_solver_entry.source_sizing_algorithm_entry(),
            source_measuring_entry: constraint_solver_entry.source_measuring_entry(),
            source_size_to_fit_entry: constraint_solver_entry.source_size_to_fit_entry(),
            source_constraint_solver_entry: constraint_solver_entry.id(),
            source_render_node: constraint_solver_entry.source_render_node(),
            source_projection_node: constraint_solver_entry.source_projection_node(),
            source_ir_node: constraint_solver_entry.source_ir_node(),
            kind,
            state: UiLayoutSolvingState::Deferred,
            order,
        });
    }

    UiLayoutSolvingModel {
        id: UiLayoutSolvingModelId::new(model.id().raw()),
        source_layout_model: model.source_layout_model(),
        source_geometry_model: model.source_geometry_model(),
        source_constraints_model: model.source_constraints_model(),
        source_sizing_model: model.source_sizing_model(),
        source_sizing_algorithm_model: model.source_sizing_algorithm_model(),
        source_measuring_model: model.source_measuring_model(),
        source_size_to_fit_model: model.source_size_to_fit_model(),
        source_constraint_solver_model: model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        entries,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiLayoutSolvingResultModelId(u64);

impl UiLayoutSolvingResultModelId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiLayoutSolvingResultEntryId(u64);

impl UiLayoutSolvingResultEntryId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutSolvingResultKind {
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayoutSolvingResultState {
    Deferred,
    MetadataOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSolvingResultEntry {
    id: UiLayoutSolvingResultEntryId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_constraint_declaration: UiLayoutConstraintId,
    source_sizing_entry: UiLayoutSizingEntryId,
    source_sizing_algorithm_entry: UiLayoutSizingAlgorithmEntryId,
    source_measuring_entry: UiLayoutMeasuringEntryId,
    source_size_to_fit_entry: UiLayoutSizeToFitEntryId,
    source_constraint_solver_entry: UiLayoutConstraintSolverEntryId,
    source_solving_entry: UiLayoutSolvingEntryId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutSolvingResultKind,
    state: UiLayoutSolvingResultState,
    order: usize,
}

impl UiLayoutSolvingResultEntry {
    pub fn id(&self) -> UiLayoutSolvingResultEntryId {
        self.id
    }

    pub fn source_layout_node(&self) -> UiLayoutNodeId {
        self.source_layout_node
    }

    pub fn source_layout_slot(&self) -> UiLayoutSlotId {
        self.source_layout_slot
    }

    pub fn source_geometry_node(&self) -> UiLayoutGeometryNodeId {
        self.source_geometry_node
    }

    pub fn source_constraint_declaration(&self) -> UiLayoutConstraintId {
        self.source_constraint_declaration
    }

    pub fn source_sizing_entry(&self) -> UiLayoutSizingEntryId {
        self.source_sizing_entry
    }

    pub fn source_sizing_algorithm_entry(&self) -> UiLayoutSizingAlgorithmEntryId {
        self.source_sizing_algorithm_entry
    }

    pub fn source_measuring_entry(&self) -> UiLayoutMeasuringEntryId {
        self.source_measuring_entry
    }

    pub fn source_size_to_fit_entry(&self) -> UiLayoutSizeToFitEntryId {
        self.source_size_to_fit_entry
    }

    pub fn source_constraint_solver_entry(&self) -> UiLayoutConstraintSolverEntryId {
        self.source_constraint_solver_entry
    }

    pub fn source_solving_entry(&self) -> UiLayoutSolvingEntryId {
        self.source_solving_entry
    }

    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiLayoutSolvingResultKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutSolvingResultState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSolvingResultModel {
    id: UiLayoutSolvingResultModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_constraints_model: UiLayoutConstraintsModelId,
    source_sizing_model: UiLayoutSizingModelId,
    source_sizing_algorithm_model: UiLayoutSizingAlgorithmModelId,
    source_measuring_model: UiLayoutMeasuringModelId,
    source_size_to_fit_model: UiLayoutSizeToFitModelId,
    source_constraint_solver_model: UiLayoutConstraintSolverModelId,
    source_solving_model: UiLayoutSolvingModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    entries: Vec<UiLayoutSolvingResultEntry>,
}

impl UiLayoutSolvingResultModel {
    pub fn id(&self) -> UiLayoutSolvingResultModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }

    pub fn source_geometry_model(&self) -> UiLayoutGeometryModelId {
        self.source_geometry_model
    }

    pub fn source_constraints_model(&self) -> UiLayoutConstraintsModelId {
        self.source_constraints_model
    }

    pub fn source_sizing_model(&self) -> UiLayoutSizingModelId {
        self.source_sizing_model
    }

    pub fn source_sizing_algorithm_model(&self) -> UiLayoutSizingAlgorithmModelId {
        self.source_sizing_algorithm_model
    }

    pub fn source_measuring_model(&self) -> UiLayoutMeasuringModelId {
        self.source_measuring_model
    }

    pub fn source_size_to_fit_model(&self) -> UiLayoutSizeToFitModelId {
        self.source_size_to_fit_model
    }

    pub fn source_constraint_solver_model(&self) -> UiLayoutConstraintSolverModelId {
        self.source_constraint_solver_model
    }

    pub fn source_solving_model(&self) -> UiLayoutSolvingModelId {
        self.source_solving_model
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn entries(&self) -> &[UiLayoutSolvingResultEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub fn build_layout_solving_result(model: &UiLayoutSolvingModel) -> UiLayoutSolvingResultModel {
    let mut entries = Vec::with_capacity(model.entries().len());

    for (order, solving_entry) in model.entries().iter().enumerate() {
        entries.push(UiLayoutSolvingResultEntry {
            id: UiLayoutSolvingResultEntryId::new(solving_entry.id().raw()),
            source_layout_node: solving_entry.source_layout_node(),
            source_layout_slot: solving_entry.source_layout_slot(),
            source_geometry_node: solving_entry.source_geometry_node(),
            source_constraint_declaration: solving_entry.source_constraint_declaration(),
            source_sizing_entry: solving_entry.source_sizing_entry(),
            source_sizing_algorithm_entry: solving_entry.source_sizing_algorithm_entry(),
            source_measuring_entry: solving_entry.source_measuring_entry(),
            source_size_to_fit_entry: solving_entry.source_size_to_fit_entry(),
            source_constraint_solver_entry: solving_entry.source_constraint_solver_entry(),
            source_solving_entry: solving_entry.id(),
            source_render_node: solving_entry.source_render_node(),
            source_projection_node: solving_entry.source_projection_node(),
            source_ir_node: solving_entry.source_ir_node(),
            kind: UiLayoutSolvingResultKind::Derived,
            state: UiLayoutSolvingResultState::Deferred,
            order,
        });
    }

    UiLayoutSolvingResultModel {
        id: UiLayoutSolvingResultModelId::new(model.id().raw()),
        source_layout_model: model.source_layout_model(),
        source_geometry_model: model.source_geometry_model(),
        source_constraints_model: model.source_constraints_model(),
        source_sizing_model: model.source_sizing_model(),
        source_sizing_algorithm_model: model.source_sizing_algorithm_model(),
        source_measuring_model: model.source_measuring_model(),
        source_size_to_fit_model: model.source_size_to_fit_model(),
        source_constraint_solver_model: model.source_constraint_solver_model(),
        source_solving_model: model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        entries,
    }
}
