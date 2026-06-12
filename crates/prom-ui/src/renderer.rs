//! Inert renderer seed model.
//!
//! Renderer seed is an inert downstream projection consumer.
//!
//! It converts projection artifacts into renderer-local presentation structures only.
//! It does not draw pixels.
//! It does not dispatch events.
//! It does not execute actions.
//! It does not authorize capabilities.
//! It does not mutate Semantic state.

use crate::model::UiIrNodeId;
use crate::projection::{
    UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact, UiProjectionArtifactId,
};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderModel {
    id: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    nodes: Vec<UiRenderNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderModelId(u64);

impl UiRenderModelId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderNode {
    id: UiRenderNodeId,
    source_projection_node: UiProjectedNodeId,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiRenderNodeKind,
    markers: Vec<UiRenderMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderNodeId(u64);

impl UiRenderNodeId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderNodeKind {
    Element,
    Text,
    Fragment,
    Root,
}

// Renderer markers are inert presentation hints only.
// They must not execute actions, authorize effects, or mutate semantic state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderMarker {
    Property,
    Action,
    EffectBoundary,
    Trace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiRenderError {
    EmptyProjection,
}

impl UiRenderModel {
    pub fn id(&self) -> UiRenderModelId {
        self.id
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn nodes(&self) -> &[UiRenderNode] {
        &self.nodes
    }
}

impl UiRenderNode {
    pub fn id(&self) -> UiRenderNodeId {
        self.id
    }

    pub fn source_projection_node(&self) -> UiProjectedNodeId {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiRenderNodeKind {
        self.kind
    }

    pub fn markers(&self) -> &[UiRenderMarker] {
        &self.markers
    }
}

pub fn render_projection_to_model(
    artifact: &UiProjectionArtifact,
) -> Result<UiRenderModel, UiRenderError> {
    if artifact.nodes().is_empty() {
        return Err(UiRenderError::EmptyProjection);
    }

    let id = UiRenderModelId::new(artifact.id().raw());
    let mut nodes = Vec::new();

    for proj_node in artifact.nodes() {
        let mut markers = Vec::new();

        let kind = match proj_node.kind() {
            UiProjectedNodeKind::Element => UiRenderNodeKind::Element,
            UiProjectedNodeKind::Text => UiRenderNodeKind::Text,
            UiProjectedNodeKind::Fragment => UiRenderNodeKind::Fragment,
            UiProjectedNodeKind::Root => UiRenderNodeKind::Root,
            UiProjectedNodeKind::PropertyCarrier => {
                markers.push(UiRenderMarker::Property);
                UiRenderNodeKind::Fragment
            }
            UiProjectedNodeKind::ActionCarrier => {
                markers.push(UiRenderMarker::Action);
                UiRenderNodeKind::Fragment
            }
            UiProjectedNodeKind::EffectBoundaryMarker => {
                markers.push(UiRenderMarker::EffectBoundary);
                UiRenderNodeKind::Fragment
            }
        };

        if proj_node.has_trace() {
            markers.push(UiRenderMarker::Trace);
        }

        let render_node = UiRenderNode {
            id: UiRenderNodeId::new(proj_node.id().raw()),
            source_projection_node: proj_node.id(),
            source_ir_node: proj_node.source_ir_node_id(),
            kind,
            markers,
        };

        nodes.push(render_node);
    }

    Ok(UiRenderModel {
        id,
        source_projection: artifact.id(),
        source_ir_root: artifact.source_ir_root(),
        nodes,
    })
}
