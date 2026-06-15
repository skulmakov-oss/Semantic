use super::*;
use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::projection::UiProjectionArtifactId;
use crate::renderer::{UiRenderModelId, UiRenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutConstraintsModelId {
    raw: u64,
}

impl UiLayoutConstraintsModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutConstraintId {
    raw: u64,
}

impl UiLayoutConstraintId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutConstraintKind {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutConstraintState {
    #[default]
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutConstraintDeclaration {
    id: UiLayoutConstraintId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutConstraintKind,
    state: UiLayoutConstraintState,
    order: usize,
}

impl UiLayoutConstraintDeclaration {
    pub fn id(&self) -> UiLayoutConstraintId {
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

    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiLayoutConstraintKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutConstraintState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutConstraintsModel {
    id: UiLayoutConstraintsModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    declarations: Vec<UiLayoutConstraintDeclaration>,
}

impl UiLayoutConstraintsModel {
    pub fn id(&self) -> UiLayoutConstraintsModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }

    pub fn source_geometry_model(&self) -> UiLayoutGeometryModelId {
        self.source_geometry_model
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

    pub fn declarations(&self) -> &[UiLayoutConstraintDeclaration] {
        &self.declarations
    }

    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.declarations.len()
    }
}

pub fn build_layout_constraints(model: &UiLayoutModel) -> UiLayoutConstraintsModel {
    let geometry_model = build_layout_geometry(model);
    let mut declarations = Vec::with_capacity(geometry_model.nodes().len());

    for geometry_node in geometry_model.nodes() {
        declarations.push(UiLayoutConstraintDeclaration {
            id: UiLayoutConstraintId::new(geometry_node.id().raw()),
            source_layout_node: geometry_node.source_layout_node(),
            source_layout_slot: geometry_node.source_layout_slot(),
            source_geometry_node: geometry_node.id(),
            source_render_node: geometry_node.source_render_node(),
            source_projection_node: geometry_node.source_projection_node(),
            source_ir_node: geometry_node.source_ir_node(),
            kind: UiLayoutConstraintKind::Unresolved,
            state: UiLayoutConstraintState::Unresolved,
            order: geometry_node.order(),
        });
    }

    UiLayoutConstraintsModel {
        id: UiLayoutConstraintsModelId::new(model.id().raw()),
        source_layout_model: model.id(),
        source_geometry_model: geometry_model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        declarations,
    }
}
