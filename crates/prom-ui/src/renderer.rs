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

// Diagnostics presentation is inert renderer-local metadata.
// It must not rewrite verifier diagnostics, execute actions, authorize effects,
// or mutate semantic/runtime state.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderDiagnosticsPresentation {
    id: UiRenderDiagnosticsPresentationId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    items: Vec<UiRenderDiagnosticItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderDiagnosticsPresentationId(u64);

impl UiRenderDiagnosticsPresentationId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderDiagnosticItem {
    id: UiRenderDiagnosticItemId,
    source_render_node: Option<UiRenderNodeId>,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiRenderDiagnosticKind,
    severity: UiRenderDiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderDiagnosticItemId(u64);

impl UiRenderDiagnosticItemId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderDiagnosticKind {
    DiagnosticMarker,
    TraceMarker,
    PropertyMarker,
    ActionMarker,
    EffectBoundaryMarker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderDiagnosticSeverity {
    Info,
    Warning,
}

impl UiRenderDiagnosticsPresentation {
    pub fn id(&self) -> UiRenderDiagnosticsPresentationId {
        self.id
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn items(&self) -> &[UiRenderDiagnosticItem] {
        &self.items
    }
}

impl UiRenderDiagnosticItem {
    pub fn id(&self) -> UiRenderDiagnosticItemId {
        self.id
    }

    pub fn source_render_node(&self) -> Option<UiRenderNodeId> {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiRenderDiagnosticKind {
        self.kind
    }

    pub fn severity(&self) -> UiRenderDiagnosticSeverity {
        self.severity
    }
}

pub fn present_render_diagnostics(model: &UiRenderModel) -> UiRenderDiagnosticsPresentation {
    let id = UiRenderDiagnosticsPresentationId(model.id().raw());
    let mut items = Vec::new();

    let mut item_counter = 0;
    for node in model.nodes() {
        for marker in node.markers() {
            let (kind, severity) = match marker {
                UiRenderMarker::Property => (
                    UiRenderDiagnosticKind::PropertyMarker,
                    UiRenderDiagnosticSeverity::Info,
                ),
                UiRenderMarker::Action => (
                    UiRenderDiagnosticKind::ActionMarker,
                    UiRenderDiagnosticSeverity::Warning,
                ),
                UiRenderMarker::EffectBoundary => (
                    UiRenderDiagnosticKind::EffectBoundaryMarker,
                    UiRenderDiagnosticSeverity::Warning,
                ),
                UiRenderMarker::Trace => (
                    UiRenderDiagnosticKind::TraceMarker,
                    UiRenderDiagnosticSeverity::Info,
                ),
            };

            // Derive deterministic item ID from node ID and a local ordinal counter for markers
            let mut hasher_val = node.id().raw().wrapping_mul(31);
            hasher_val = hasher_val.wrapping_add(item_counter);
            let item_id = UiRenderDiagnosticItemId(hasher_val);
            item_counter += 1;

            items.push(UiRenderDiagnosticItem {
                id: item_id,
                source_render_node: Some(node.id()),
                source_projection_node: Some(node.source_projection_node()),
                source_ir_node: node.source_ir_node(),
                kind,
                severity,
            });
        }
    }

    UiRenderDiagnosticsPresentation {
        id,
        source_render_model: model.id(),
        source_projection: model.source_projection(),
        items,
    }
}

// Trace presentation is inert renderer-local metadata.
// It must not act as proof, debugger state, verifier output,
// runtime introspection, event dispatch, or semantic authority.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderTracePresentation {
    id: UiRenderTracePresentationId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    links: Vec<UiRenderTraceLink>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderTracePresentationId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderTraceLink {
    id: UiRenderTraceLinkId,
    source_render_node: Option<UiRenderNodeId>,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiRenderTraceLinkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderTraceLinkId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderTraceLinkKind {
    RenderModelToProjection,
    RenderNodeToProjectionNode,
    RenderNodeToIrNode,
}

impl UiRenderTracePresentationId {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl UiRenderTraceLinkId {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl UiRenderTracePresentation {
    pub fn id(&self) -> UiRenderTracePresentationId {
        self.id
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn links(&self) -> &[UiRenderTraceLink] {
        &self.links
    }
}

impl UiRenderTraceLink {
    pub fn id(&self) -> UiRenderTraceLinkId {
        self.id
    }

    pub fn source_render_node(&self) -> Option<UiRenderNodeId> {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiRenderTraceLinkKind {
        self.kind
    }
}

pub fn present_render_trace(model: &UiRenderModel) -> UiRenderTracePresentation {
    // Domain marker for trace presentation
    let id_val = model.id().raw().wrapping_mul(31).wrapping_add(2);
    let id = UiRenderTracePresentationId::new(id_val);

    let mut links = Vec::new();
    let mut link_counter = 0_u64;

    // Model level link
    let link_id_val = id.raw().wrapping_mul(17).wrapping_add(link_counter);
    let link_id = UiRenderTraceLinkId::new(link_id_val);
    link_counter += 1;

    links.push(UiRenderTraceLink {
        id: link_id,
        source_render_node: None,
        source_projection_node: None,
        source_ir_node: model.source_ir_root(),
        kind: UiRenderTraceLinkKind::RenderModelToProjection,
    });

    for node in model.nodes() {
        let link_id_val = id.raw().wrapping_mul(17).wrapping_add(link_counter);
        let link_id = UiRenderTraceLinkId::new(link_id_val);
        link_counter += 1;

        links.push(UiRenderTraceLink {
            id: link_id,
            source_render_node: Some(node.id()),
            source_projection_node: Some(node.source_projection_node()),
            source_ir_node: node.source_ir_node(),
            kind: UiRenderTraceLinkKind::RenderNodeToProjectionNode,
        });

        if node.source_ir_node().is_some() {
            let link_id_val = id.raw().wrapping_mul(17).wrapping_add(link_counter);
            let link_id = UiRenderTraceLinkId::new(link_id_val);
            link_counter += 1;

            links.push(UiRenderTraceLink {
                id: link_id,
                source_render_node: Some(node.id()),
                source_projection_node: Some(node.source_projection_node()),
                source_ir_node: node.source_ir_node(),
                kind: UiRenderTraceLinkKind::RenderNodeToIrNode,
            });
        }
    }

    UiRenderTracePresentation {
        id,
        source_render_model: model.id(),
        source_projection: model.source_projection(),
        links,
    }
}
