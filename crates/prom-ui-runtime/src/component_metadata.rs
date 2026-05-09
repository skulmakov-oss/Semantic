//! Semantic UI component metadata scaffold.
//!
//! This module defines component descriptors and semantic component roles.
//! It does not implement widgets, renderer output, Workbench UI,
//! interaction callbacks, actions, effects, or runtime presentation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiComponentDescriptor {
    pub id: &'static str,
    pub kind: UiComponentKind,
    pub domain: UiComponentDomain,
    pub admission: UiComponentAdmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiComponentDomain {
    State,
    Trace,
    Capability,
    Effect,
    Error,
    Recovery,
    Renderer,
    Workbench,
    Simulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiComponentKind {
    StateBadge,
    StateSummary,
    TraceEventRow,
    TraceSummaryPanel,
    CapabilityDecisionView,
    CapabilityMissingBadge,
    EffectRequestView,
    EffectCommitView,
    DenialReasonOverlay,
    ErrorDetailPanel,
    FailureStageView,
    QuarantineNotice,
    ConflictBoundaryView,
    RecoveryOptionPanel,
    RollbackTraceView,
    RendererTranscriptView,
    FramePresentationStatusView,
    WorkbenchProjectionView,
    SnapshotModeBadge,
    SimulationModeBadge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiComponentAdmission {
    CoreCandidate,
    WorkbenchLocal,
    DiagnosticOnly,
}

pub const UI_COMPONENTS: &[UiComponentDescriptor] = &[
    UiComponentDescriptor {
        id: "ui.component.state_badge",
        kind: UiComponentKind::StateBadge,
        domain: UiComponentDomain::State,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.state_summary",
        kind: UiComponentKind::StateSummary,
        domain: UiComponentDomain::State,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.trace_event_row",
        kind: UiComponentKind::TraceEventRow,
        domain: UiComponentDomain::Trace,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.trace_summary_panel",
        kind: UiComponentKind::TraceSummaryPanel,
        domain: UiComponentDomain::Trace,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.capability_decision_view",
        kind: UiComponentKind::CapabilityDecisionView,
        domain: UiComponentDomain::Capability,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.capability_missing_badge",
        kind: UiComponentKind::CapabilityMissingBadge,
        domain: UiComponentDomain::Capability,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.effect_request_view",
        kind: UiComponentKind::EffectRequestView,
        domain: UiComponentDomain::Effect,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.effect_commit_view",
        kind: UiComponentKind::EffectCommitView,
        domain: UiComponentDomain::Effect,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.denial_reason_overlay",
        kind: UiComponentKind::DenialReasonOverlay,
        domain: UiComponentDomain::Error,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.error_detail_panel",
        kind: UiComponentKind::ErrorDetailPanel,
        domain: UiComponentDomain::Error,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.failure_stage_view",
        kind: UiComponentKind::FailureStageView,
        domain: UiComponentDomain::Error,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.quarantine_notice",
        kind: UiComponentKind::QuarantineNotice,
        domain: UiComponentDomain::Error,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.conflict_boundary_view",
        kind: UiComponentKind::ConflictBoundaryView,
        domain: UiComponentDomain::Error,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.recovery_option_panel",
        kind: UiComponentKind::RecoveryOptionPanel,
        domain: UiComponentDomain::Recovery,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.rollback_trace_view",
        kind: UiComponentKind::RollbackTraceView,
        domain: UiComponentDomain::Recovery,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.renderer_transcript_view",
        kind: UiComponentKind::RendererTranscriptView,
        domain: UiComponentDomain::Renderer,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.frame_presentation_status_view",
        kind: UiComponentKind::FramePresentationStatusView,
        domain: UiComponentDomain::Renderer,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.workbench_projection_view",
        kind: UiComponentKind::WorkbenchProjectionView,
        domain: UiComponentDomain::Workbench,
        admission: UiComponentAdmission::WorkbenchLocal,
    },
    UiComponentDescriptor {
        id: "ui.component.snapshot_mode_badge",
        kind: UiComponentKind::SnapshotModeBadge,
        domain: UiComponentDomain::Simulation,
        admission: UiComponentAdmission::CoreCandidate,
    },
    UiComponentDescriptor {
        id: "ui.component.simulation_mode_badge",
        kind: UiComponentKind::SimulationModeBadge,
        domain: UiComponentDomain::Simulation,
        admission: UiComponentAdmission::CoreCandidate,
    },
];

pub fn ui_components() -> &'static [UiComponentDescriptor] {
    UI_COMPONENTS
}

pub fn find_ui_component(id: &str) -> Option<&'static UiComponentDescriptor> {
    UI_COMPONENTS.iter().find(|component| component.id == id)
}
