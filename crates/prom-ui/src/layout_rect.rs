use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::renderer::UiRenderNodeId;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiLayoutRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl UiLayoutRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn x(&self) -> i32 {
        self.x
    }

    pub const fn y(&self) -> i32 {
        self.y
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutRectEntry {
    render_node_id: UiRenderNodeId,
    source_projection_node_id: UiProjectedNodeId,
    source_ir_node_id: Option<UiIrNodeId>,
    rect: UiLayoutRect,
}

impl UiLayoutRectEntry {
    pub fn new(
        render_node_id: UiRenderNodeId,
        source_projection_node_id: UiProjectedNodeId,
        source_ir_node_id: Option<UiIrNodeId>,
        rect: UiLayoutRect,
    ) -> Self {
        Self {
            render_node_id,
            source_projection_node_id,
            source_ir_node_id,
            rect,
        }
    }

    pub const fn render_node_id(&self) -> UiRenderNodeId {
        self.render_node_id
    }

    pub const fn source_projection_node_id(&self) -> UiProjectedNodeId {
        self.source_projection_node_id
    }

    pub const fn source_ir_node_id(&self) -> Option<UiIrNodeId> {
        self.source_ir_node_id
    }

    pub const fn rect(&self) -> UiLayoutRect {
        self.rect
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiLayoutRectModel {
    entries: Vec<UiLayoutRectEntry>,
}

impl UiLayoutRectModel {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push_entry(&mut self, entry: UiLayoutRectEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[UiLayoutRectEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
