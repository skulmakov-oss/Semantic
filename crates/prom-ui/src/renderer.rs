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

// Marker presentation is inert renderer-local metadata.
// It must not execute action markers, authorize effect markers,
// or mutate semantic/runtime state.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderMarkerPresentation {
    id: UiRenderMarkerPresentationId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    items: Vec<UiRenderMarkerItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderMarkerPresentationId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderMarkerItem {
    id: UiRenderMarkerItemId,
    source_render_node: UiRenderNodeId,
    role: UiRenderMarkerVisualRole,
    emphasis: UiRenderMarkerEmphasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderMarkerItemId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderMarkerVisualRole {
    PropertyIndicator,
    ActionIndicator,
    EffectBoundaryIndicator,
    TraceIndicator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderMarkerEmphasis {
    Subtle,
    Prominent,
}

impl UiRenderMarkerPresentationId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

impl UiRenderMarkerItemId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

impl UiRenderMarkerPresentation {
    pub fn id(&self) -> UiRenderMarkerPresentationId {
        self.id
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn items(&self) -> &[UiRenderMarkerItem] {
        &self.items
    }
}

impl UiRenderMarkerItem {
    pub fn id(&self) -> UiRenderMarkerItemId {
        self.id
    }

    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }

    pub fn role(&self) -> UiRenderMarkerVisualRole {
        self.role
    }

    pub fn emphasis(&self) -> UiRenderMarkerEmphasis {
        self.emphasis
    }
}

pub fn present_render_markers(model: &UiRenderModel) -> UiRenderMarkerPresentation {
    // Domain marker for marker presentation
    let id_val = model.id().raw().wrapping_mul(31).wrapping_add(3);
    let id = UiRenderMarkerPresentationId::new(id_val);

    let mut items = Vec::new();
    let mut item_counter = 0_u64;

    for node in model.nodes() {
        for marker in node.markers() {
            let (role, emphasis) = match marker {
                UiRenderMarker::Property => (
                    UiRenderMarkerVisualRole::PropertyIndicator,
                    UiRenderMarkerEmphasis::Subtle,
                ),
                UiRenderMarker::Action => (
                    UiRenderMarkerVisualRole::ActionIndicator,
                    UiRenderMarkerEmphasis::Prominent,
                ),
                UiRenderMarker::EffectBoundary => (
                    UiRenderMarkerVisualRole::EffectBoundaryIndicator,
                    UiRenderMarkerEmphasis::Prominent,
                ),
                UiRenderMarker::Trace => (
                    UiRenderMarkerVisualRole::TraceIndicator,
                    UiRenderMarkerEmphasis::Subtle,
                ),
            };

            let item_id_val = id.raw().wrapping_mul(17).wrapping_add(item_counter);
            let item_id = UiRenderMarkerItemId::new(item_id_val);
            item_counter += 1;

            items.push(UiRenderMarkerItem {
                id: item_id,
                source_render_node: node.id(),
                role,
                emphasis,
            });
        }
    }

    UiRenderMarkerPresentation {
        id,
        source_render_model: model.id(),
        source_projection: model.source_projection(),
        items,
    }
}

// Inspection presentation is inert renderer-local metadata.
// It composes diagnostics, trace, and marker presentation data for future UI inspection.
// It must not act as debugger state, proof, runtime introspection, event dispatch,
// action execution, effect authorization, or capability admission.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderInspectionPresentation {
    id: UiRenderInspectionPresentationId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    sections: Vec<UiRenderInspectionSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderInspectionPresentationId(u64);

impl UiRenderInspectionPresentationId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderInspectionSection {
    id: UiRenderInspectionSectionId,
    kind: UiRenderInspectionSectionKind,
    items: Vec<UiRenderInspectionItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderInspectionSectionId(u64);

impl UiRenderInspectionSectionId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderInspectionItem {
    id: UiRenderInspectionItemId,
    kind: UiRenderInspectionItemKind,
    source_render_node: Option<UiRenderNodeId>,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiRenderInspectionItemId(u64);

impl UiRenderInspectionItemId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderInspectionSectionKind {
    Overview,
    Diagnostics,
    Trace,
    Markers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRenderInspectionItemKind {
    OverviewSummary,
    DiagnosticItem,
    TraceLink,
    MarkerItem,
}

impl UiRenderInspectionPresentation {
    pub fn id(&self) -> UiRenderInspectionPresentationId {
        self.id
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn sections(&self) -> &[UiRenderInspectionSection] {
        &self.sections
    }
}

impl UiRenderInspectionSection {
    pub fn id(&self) -> UiRenderInspectionSectionId {
        self.id
    }

    pub fn kind(&self) -> UiRenderInspectionSectionKind {
        self.kind
    }

    pub fn items(&self) -> &[UiRenderInspectionItem] {
        &self.items
    }
}

impl UiRenderInspectionItem {
    pub fn id(&self) -> UiRenderInspectionItemId {
        self.id
    }

    pub fn kind(&self) -> UiRenderInspectionItemKind {
        self.kind
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
}

const fn inspection_section_kind_ordinal(kind: UiRenderInspectionSectionKind) -> u64 {
    match kind {
        UiRenderInspectionSectionKind::Overview => 0,
        UiRenderInspectionSectionKind::Diagnostics => 1,
        UiRenderInspectionSectionKind::Trace => 2,
        UiRenderInspectionSectionKind::Markers => 3,
    }
}

const fn inspection_item_kind_ordinal(kind: UiRenderInspectionItemKind) -> u64 {
    match kind {
        UiRenderInspectionItemKind::OverviewSummary => 0,
        UiRenderInspectionItemKind::DiagnosticItem => 1,
        UiRenderInspectionItemKind::TraceLink => 2,
        UiRenderInspectionItemKind::MarkerItem => 3,
    }
}

fn inspection_section_id(
    presentation_id: UiRenderInspectionPresentationId,
    kind: UiRenderInspectionSectionKind,
) -> UiRenderInspectionSectionId {
    UiRenderInspectionSectionId::new(
        presentation_id
            .raw()
            .wrapping_mul(37)
            .wrapping_add(inspection_section_kind_ordinal(kind)),
    )
}

fn inspection_item_id(
    section_id: UiRenderInspectionSectionId,
    kind: UiRenderInspectionItemKind,
    ordinal: u64,
) -> UiRenderInspectionItemId {
    UiRenderInspectionItemId::new(
        section_id
            .raw()
            .wrapping_mul(17)
            .wrapping_add(inspection_item_kind_ordinal(kind))
            .wrapping_add(ordinal),
    )
}

fn build_overview_section(
    presentation_id: UiRenderInspectionPresentationId,
    model: &UiRenderModel,
) -> UiRenderInspectionSection {
    let kind = UiRenderInspectionSectionKind::Overview;
    let section_id = inspection_section_id(presentation_id, kind);
    let mut items = Vec::new();

    items.push(UiRenderInspectionItem {
        id: inspection_item_id(section_id, UiRenderInspectionItemKind::OverviewSummary, 0),
        kind: UiRenderInspectionItemKind::OverviewSummary,
        source_render_node: None,
        source_projection_node: None,
        source_ir_node: model.source_ir_root(),
    });

    UiRenderInspectionSection {
        id: section_id,
        kind,
        items,
    }
}

fn build_diagnostics_section(
    presentation_id: UiRenderInspectionPresentationId,
    diagnostics: &UiRenderDiagnosticsPresentation,
) -> UiRenderInspectionSection {
    let kind = UiRenderInspectionSectionKind::Diagnostics;
    let section_id = inspection_section_id(presentation_id, kind);
    let mut items = Vec::new();

    for (ordinal, item) in diagnostics.items().iter().enumerate() {
        items.push(UiRenderInspectionItem {
            id: inspection_item_id(
                section_id,
                UiRenderInspectionItemKind::DiagnosticItem,
                ordinal as u64,
            ),
            kind: UiRenderInspectionItemKind::DiagnosticItem,
            source_render_node: item.source_render_node(),
            source_projection_node: item.source_projection_node(),
            source_ir_node: item.source_ir_node(),
        });
    }

    UiRenderInspectionSection {
        id: section_id,
        kind,
        items,
    }
}

fn build_trace_section(
    presentation_id: UiRenderInspectionPresentationId,
    trace: &UiRenderTracePresentation,
) -> UiRenderInspectionSection {
    let kind = UiRenderInspectionSectionKind::Trace;
    let section_id = inspection_section_id(presentation_id, kind);
    let mut items = Vec::new();

    for (ordinal, link) in trace.links().iter().enumerate() {
        items.push(UiRenderInspectionItem {
            id: inspection_item_id(
                section_id,
                UiRenderInspectionItemKind::TraceLink,
                ordinal as u64,
            ),
            kind: UiRenderInspectionItemKind::TraceLink,
            source_render_node: link.source_render_node(),
            source_projection_node: link.source_projection_node(),
            source_ir_node: link.source_ir_node(),
        });
    }

    UiRenderInspectionSection {
        id: section_id,
        kind,
        items,
    }
}

fn build_markers_section(
    presentation_id: UiRenderInspectionPresentationId,
    markers: &UiRenderMarkerPresentation,
) -> UiRenderInspectionSection {
    let kind = UiRenderInspectionSectionKind::Markers;
    let section_id = inspection_section_id(presentation_id, kind);
    let mut items = Vec::new();

    for (ordinal, item) in markers.items().iter().enumerate() {
        items.push(UiRenderInspectionItem {
            id: inspection_item_id(
                section_id,
                UiRenderInspectionItemKind::MarkerItem,
                ordinal as u64,
            ),
            kind: UiRenderInspectionItemKind::MarkerItem,
            source_render_node: Some(item.source_render_node()),
            source_projection_node: None,
            source_ir_node: None,
        });
    }

    UiRenderInspectionSection {
        id: section_id,
        kind,
        items,
    }
}

pub fn present_render_inspection(
    model: &UiRenderModel,
    diagnostics: &UiRenderDiagnosticsPresentation,
    trace: &UiRenderTracePresentation,
    markers: &UiRenderMarkerPresentation,
) -> UiRenderInspectionPresentation {
    let id = UiRenderInspectionPresentationId::new(model.id().raw());
    let sections = vec![
        build_overview_section(id, model),
        build_diagnostics_section(id, diagnostics),
        build_trace_section(id, trace),
        build_markers_section(id, markers),
    ];

    UiRenderInspectionPresentation {
        id,
        source_render_model: model.id(),
        source_projection: model.source_projection(),
        sections,
    }
}
