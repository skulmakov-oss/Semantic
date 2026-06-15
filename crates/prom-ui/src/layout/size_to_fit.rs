use super::*;
use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::projection::UiProjectionArtifactId;
use crate::renderer::{UiRenderModelId, UiRenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutSizeToFitModelId {
    raw: u64,
}

impl UiLayoutSizeToFitModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutSizeToFitEntryId {
    raw: u64,
}

impl UiLayoutSizeToFitEntryId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutSizeToFitKind {
    #[default]
    DeferredIntent,
    UnavailableResult,
    AuditOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutSizeToFitState {
    #[default]
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSizeToFitEntry {
    id: UiLayoutSizeToFitEntryId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_constraint_declaration: UiLayoutConstraintId,
    source_sizing_entry: UiLayoutSizingEntryId,
    source_sizing_algorithm_entry: UiLayoutSizingAlgorithmEntryId,
    source_measuring_entry: UiLayoutMeasuringEntryId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutSizeToFitKind,
    state: UiLayoutSizeToFitState,
    order: usize,
}

impl UiLayoutSizeToFitEntry {
    pub fn id(&self) -> UiLayoutSizeToFitEntryId {
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

    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiLayoutSizeToFitKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutSizeToFitState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSizeToFitModel {
    id: UiLayoutSizeToFitModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_constraints_model: UiLayoutConstraintsModelId,
    source_sizing_model: UiLayoutSizingModelId,
    source_sizing_algorithm_model: UiLayoutSizingAlgorithmModelId,
    source_measuring_model: UiLayoutMeasuringModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    entries: Vec<UiLayoutSizeToFitEntry>,
}

impl UiLayoutSizeToFitModel {
    pub fn id(&self) -> UiLayoutSizeToFitModelId {
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

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn entries(&self) -> &[UiLayoutSizeToFitEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub fn build_layout_size_to_fit(model: &UiLayoutMeasuringModel) -> UiLayoutSizeToFitModel {
    let mut entries = Vec::with_capacity(model.entries().len());

    for (order, measuring_entry) in model.entries().iter().enumerate() {
        let kind = match order % 3 {
            0 => UiLayoutSizeToFitKind::DeferredIntent,
            1 => UiLayoutSizeToFitKind::UnavailableResult,
            _ => UiLayoutSizeToFitKind::AuditOnly,
        };

        entries.push(UiLayoutSizeToFitEntry {
            id: UiLayoutSizeToFitEntryId::new(measuring_entry.id().raw()),
            source_layout_node: measuring_entry.source_layout_node(),
            source_layout_slot: measuring_entry.source_layout_slot(),
            source_geometry_node: measuring_entry.source_geometry_node(),
            source_constraint_declaration: measuring_entry.source_constraint_declaration(),
            source_sizing_entry: measuring_entry.source_sizing_entry(),
            source_sizing_algorithm_entry: measuring_entry.source_sizing_algorithm_entry(),
            source_measuring_entry: measuring_entry.id(),
            source_render_node: measuring_entry.source_render_node(),
            source_projection_node: measuring_entry.source_projection_node(),
            source_ir_node: measuring_entry.source_ir_node(),
            kind,
            state: UiLayoutSizeToFitState::Deferred,
            order,
        });
    }

    UiLayoutSizeToFitModel {
        id: UiLayoutSizeToFitModelId::new(model.id().raw()),
        source_layout_model: model.source_layout_model(),
        source_geometry_model: model.source_geometry_model(),
        source_constraints_model: model.source_constraints_model(),
        source_sizing_model: model.source_sizing_model(),
        source_sizing_algorithm_model: model.source_sizing_algorithm_model(),
        source_measuring_model: model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        entries,
    }
}
