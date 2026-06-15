use super::*;
use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::projection::UiProjectionArtifactId;
use crate::renderer::{UiRenderModelId, UiRenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutSizingModelId {
    raw: u64,
}

impl UiLayoutSizingModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutSizingEntryId {
    raw: u64,
}

impl UiLayoutSizingEntryId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutSizingKind {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutSizingState {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSizingEntry {
    id: UiLayoutSizingEntryId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_constraint_declaration: UiLayoutConstraintId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutSizingKind,
    state: UiLayoutSizingState,
    order: usize,
}

impl UiLayoutSizingEntry {
    pub fn id(&self) -> UiLayoutSizingEntryId {
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

    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiLayoutSizingKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutSizingState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutSizingModel {
    id: UiLayoutSizingModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_constraints_model: UiLayoutConstraintsModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    entries: Vec<UiLayoutSizingEntry>,
}

impl UiLayoutSizingModel {
    pub fn id(&self) -> UiLayoutSizingModelId {
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

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn entries(&self) -> &[UiLayoutSizingEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub fn build_layout_sizing(model: &UiLayoutModel) -> UiLayoutSizingModel {
    let geometry_model = build_layout_geometry(model);
    let constraints_model = build_layout_constraints(model);
    let mut entries = Vec::with_capacity(constraints_model.declarations().len());

    for (index, ((declaration, layout_node), geometry_node)) in constraints_model
        .declarations()
        .iter()
        .zip(model.nodes())
        .zip(geometry_model.nodes())
        .enumerate()
    {
        entries.push(UiLayoutSizingEntry {
            id: UiLayoutSizingEntryId::new(declaration.id().raw()),
            source_layout_node: layout_node.id(),
            source_layout_slot: layout_node.slot(),
            source_geometry_node: geometry_node.id(),
            source_constraint_declaration: declaration.id(),
            source_render_node: layout_node.source_render_node(),
            source_projection_node: layout_node.source_projection_node(),
            source_ir_node: layout_node.source_ir_node(),
            kind: UiLayoutSizingKind::Unresolved,
            state: UiLayoutSizingState::Unresolved,
            order: index,
        });
    }

    UiLayoutSizingModel {
        id: UiLayoutSizingModelId::new(model.id().raw()),
        source_layout_model: model.id(),
        source_geometry_model: geometry_model.id(),
        source_constraints_model: constraints_model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        entries,
    }
}
