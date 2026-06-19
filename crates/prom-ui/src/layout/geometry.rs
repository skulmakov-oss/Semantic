use super::*;
use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::projection::UiProjectionArtifactId;
use crate::renderer::{UiRenderModelId, UiRenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutGeometryModelId {
    raw: u64,
}

impl UiLayoutGeometryModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutGeometryNodeId {
    raw: u64,
}

impl UiLayoutGeometryNodeId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiPoint {
    pub x: i32,
    pub y: i32,
}

impl UiPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiSize {
    pub width: u32,
    pub height: u32,
}

impl UiSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiRect {
    pub origin: UiPoint,
    pub size: UiSize,
}

impl UiRect {
    pub const fn new(origin: UiPoint, size: UiSize) -> Self {
        Self { origin, size }
    }

    pub const fn from_xywh(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            origin: UiPoint::new(x, y),
            size: UiSize::new(width, height),
        }
    }

    pub const fn x(&self) -> i32 {
        self.origin.x
    }

    pub const fn y(&self) -> i32 {
        self.origin.y
    }

    pub const fn width(&self) -> u32 {
        self.size.width
    }

    pub const fn height(&self) -> u32 {
        self.size.height
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutGeometryNode {
    id: UiLayoutGeometryNodeId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    rect: UiRect,
    order: usize,
}

impl UiLayoutGeometryNode {
    pub fn id(&self) -> UiLayoutGeometryNodeId {
        self.id
    }

    pub fn source_layout_node(&self) -> UiLayoutNodeId {
        self.source_layout_node
    }

    pub fn source_layout_slot(&self) -> UiLayoutSlotId {
        self.source_layout_slot
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

    pub fn rect(&self) -> UiRect {
        self.rect
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn with_rect(mut self, rect: UiRect) -> Self {
        self.rect = rect;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutGeometryModel {
    id: UiLayoutGeometryModelId,
    source_layout_model: UiLayoutModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    nodes: Vec<UiLayoutGeometryNode>,
}

impl UiLayoutGeometryModel {
    pub fn id(&self) -> UiLayoutGeometryModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
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

    pub fn nodes(&self) -> &[UiLayoutGeometryNode] {
        &self.nodes
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn with_nodes(mut self, nodes: Vec<UiLayoutGeometryNode>) -> Self {
        self.nodes = nodes;
        self
    }
}

pub fn build_layout_geometry(model: &UiLayoutModel) -> UiLayoutGeometryModel {
    let mut nodes = Vec::with_capacity(model.nodes().len());

    for layout_node in model.nodes() {
        nodes.push(UiLayoutGeometryNode {
            id: UiLayoutGeometryNodeId::new(layout_node.id().raw()),
            source_layout_node: layout_node.id(),
            source_layout_slot: layout_node.slot(),
            source_render_node: layout_node.source_render_node(),
            source_projection_node: layout_node.source_projection_node(),
            source_ir_node: layout_node.source_ir_node(),
            rect: UiRect::default(),
            order: layout_node.order(),
        });
    }

    UiLayoutGeometryModel {
        id: UiLayoutGeometryModelId::new(model.id().raw()),
        source_layout_model: model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        nodes,
    }
}
