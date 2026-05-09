//! Semantic UI boundary registry scaffold.
//!
//! This module lists admitted UI boundary documents.
//! It does not implement renderer, Workbench UI, components, actions, effects,
//! or runtime behavior.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiBoundary {
    pub id: &'static str,
    pub title: &'static str,
    pub path: &'static str,
    pub layer: UiBoundaryLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiBoundaryLayer {
    Doctrine,
    Tokens,
    Layout,
    Components,
    Interaction,
    FocusSelection,
    Actions,
    EffectsCapabilities,
    TraceAudit,
    ErrorDenialQuarantine,
    RecoveryRollback,
    RendererTranscript,
    WorkbenchConsumption,
    SimulationSnapshot,
    ImplementationGate,
    Ownership,
    NativeBackend,
    RendererAdmission,
}

pub const UI_BOUNDARIES: &[UiBoundary] = &[
    UiBoundary {
        id: "ui.visual_doctrine",
        title: "Semantic UI Visual Design Doctrine",
        path: "docs/architecture/ui_visual_design_doctrine.md",
        layer: UiBoundaryLayer::Doctrine,
    },
    UiBoundary {
        id: "ui.visual_token_system",
        title: "Semantic UI Visual Token System Boundary",
        path: "docs/architecture/ui_visual_token_system_boundary.md",
        layer: UiBoundaryLayer::Tokens,
    },
    UiBoundary {
        id: "ui.layout_primitives",
        title: "Semantic UI Layout Primitive Boundary",
        path: "docs/architecture/ui_layout_primitive_boundary.md",
        layer: UiBoundaryLayer::Layout,
    },
    UiBoundary {
        id: "ui.component_admission",
        title: "Semantic UI Component Admission Boundary",
        path: "docs/architecture/ui_component_admission_boundary.md",
        layer: UiBoundaryLayer::Components,
    },
    UiBoundary {
        id: "ui.interaction_input",
        title: "Semantic UI Interaction and Input Semantic Boundary",
        path: "docs/architecture/ui_interaction_input_semantic_boundary.md",
        layer: UiBoundaryLayer::Interaction,
    },
    UiBoundary {
        id: "ui.focus_selection",
        title: "Semantic UI Focus and Selection Boundary",
        path: "docs/architecture/ui_focus_selection_semantic_boundary.md",
        layer: UiBoundaryLayer::FocusSelection,
    },
    UiBoundary {
        id: "ui.semantic_action",
        title: "Semantic UI Action Boundary",
        path: "docs/architecture/ui_semantic_action_boundary.md",
        layer: UiBoundaryLayer::Actions,
    },
    UiBoundary {
        id: "ui.effect_request_capability",
        title: "Semantic UI Effect Request and Capability Boundary",
        path: "docs/architecture/ui_effect_request_capability_boundary.md",
        layer: UiBoundaryLayer::EffectsCapabilities,
    },
    UiBoundary {
        id: "ui.trace_audit_visual",
        title: "Semantic UI Trace and Audit Visual Boundary",
        path: "docs/architecture/ui_trace_audit_visual_boundary.md",
        layer: UiBoundaryLayer::TraceAudit,
    },
    UiBoundary {
        id: "ui.error_denial_quarantine_visual",
        title: "Semantic UI Error, Denial, and Quarantine Visual Boundary",
        path: "docs/architecture/ui_error_denial_quarantine_visual_boundary.md",
        layer: UiBoundaryLayer::ErrorDenialQuarantine,
    },
    UiBoundary {
        id: "ui.recovery_rollback_visual",
        title: "Semantic UI Recovery and Rollback Visual Boundary",
        path: "docs/architecture/ui_recovery_rollback_visual_boundary.md",
        layer: UiBoundaryLayer::RecoveryRollback,
    },
    UiBoundary {
        id: "ui.renderer_transcript_presentation",
        title: "Semantic UI Renderer Transcript and Presentation Status Boundary",
        path: "docs/architecture/ui_renderer_transcript_presentation_boundary.md",
        layer: UiBoundaryLayer::RendererTranscript,
    },
    UiBoundary {
        id: "ui.workbench_consumption",
        title: "Semantic UI Workbench Consumption Boundary",
        path: "docs/architecture/ui_workbench_consumption_boundary.md",
        layer: UiBoundaryLayer::WorkbenchConsumption,
    },
    UiBoundary {
        id: "ui.simulation_snapshot",
        title: "Semantic UI Simulation and Snapshot Boundary",
        path: "docs/architecture/ui_simulation_snapshot_boundary.md",
        layer: UiBoundaryLayer::SimulationSnapshot,
    },
    UiBoundary {
        id: "ui.boundary_index",
        title: "Semantic UI Boundary Index",
        path: "docs/architecture/ui_boundary_index.md",
        layer: UiBoundaryLayer::ImplementationGate,
    },
    UiBoundary {
        id: "ui.implementation_gate",
        title: "Semantic UI Implementation Gate",
        path: "docs/architecture/ui_implementation_gate.md",
        layer: UiBoundaryLayer::ImplementationGate,
    },
    UiBoundary {
        id: "ui.ownership_map",
        title: "Semantic UI Ownership Map",
        path: "docs/architecture/ui_ownership_map.md",
        layer: UiBoundaryLayer::Ownership,
    },
    UiBoundary {
        id: "ui.renderer_admission",
        title: "Semantic UI Renderer Admission Boundary",
        path: "docs/architecture/ui_renderer_admission_boundary.md",
        layer: UiBoundaryLayer::RendererAdmission,
    },
    UiBoundary {
        id: "ui.native_backend",
        title: "Semantic UI Native Backend Boundary",
        path: "docs/architecture/ui_native_backend_boundary.md",
        layer: UiBoundaryLayer::NativeBackend,
    },
];

pub fn ui_boundaries() -> &'static [UiBoundary] {
    UI_BOUNDARIES
}

pub fn find_ui_boundary(id: &str) -> Option<&'static UiBoundary> {
    UI_BOUNDARIES.iter().find(|boundary| boundary.id == id)
}
