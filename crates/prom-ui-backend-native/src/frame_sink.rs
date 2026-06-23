use alloc::vec::Vec;
use prom_ui::{UiIrNodeId, UiLayoutRect, UiProjectedNodeId, UiRenderNodeId, UiRenderNodeKind};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct UiBackendFrame {
    entries: Vec<UiBackendFrameEntry>,
}

impl UiBackendFrame {
    pub fn new(entries: Vec<UiBackendFrameEntry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[UiBackendFrameEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiBackendFrameEntry {
    render_node_id: UiRenderNodeId,
    source_projection_node_id: UiProjectedNodeId,
    source_ir_node_id: Option<UiIrNodeId>,
    rect: UiLayoutRect,
    kind: UiRenderNodeKind,
}

impl UiBackendFrameEntry {
    pub fn new(
        render_node_id: UiRenderNodeId,
        source_projection_node_id: UiProjectedNodeId,
        source_ir_node_id: Option<UiIrNodeId>,
        rect: UiLayoutRect,
        kind: UiRenderNodeKind,
    ) -> Self {
        Self {
            render_node_id,
            source_projection_node_id,
            source_ir_node_id,
            rect,
            kind,
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

    pub const fn kind(&self) -> UiRenderNodeKind {
        self.kind
    }
}

pub trait UiFrameSink {
    type Error;

    fn submit_frame(&mut self, frame: &UiBackendFrame) -> Result<(), Self::Error>;
}
