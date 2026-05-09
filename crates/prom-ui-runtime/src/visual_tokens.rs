//! Semantic UI visual token type scaffold.
//!
//! This module defines semantic visual token roles.
//! It does not implement a renderer, theme engine, CSS, Workbench UI,
//! components, widgets, or runtime presentation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiVisualToken {
    pub id: &'static str,
    pub role: UiVisualTokenRole,
    pub domain: UiVisualTokenDomain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiVisualTokenDomain {
    Surface,
    Border,
    Text,
    Accent,
    State,
    Trace,
    Capability,
    Effect,
    Renderer,
    Workbench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiVisualTokenRole {
    SurfaceBase,
    SurfaceRaised,
    SurfaceOverlay,
    BorderSubtle,
    BorderStrong,
    TextPrimary,
    TextSecondary,
    AccentPrimary,
    StateAdmitted,
    StateDenied,
    StateFailed,
    StateQuarantined,
    StateConflict,
    StateStale,
    StatePreview,
    StateSimulation,
    TraceSource,
    TraceAdmission,
    TraceAction,
    TraceEffect,
    TraceAudit,
    CapabilityAvailable,
    CapabilityMissing,
    CapabilityDenied,
    EffectRequested,
    EffectPrepared,
    EffectCommitted,
    EffectRolledBack,
    RendererStaged,
    RendererAttempted,
    RendererPresented,
    WorkbenchLocal,
    WorkbenchProjection,
}

pub const UI_VISUAL_TOKENS: &[UiVisualToken] = &[
    UiVisualToken {
        id: "ui.token.surface.base",
        role: UiVisualTokenRole::SurfaceBase,
        domain: UiVisualTokenDomain::Surface,
    },
    UiVisualToken {
        id: "ui.token.surface.raised",
        role: UiVisualTokenRole::SurfaceRaised,
        domain: UiVisualTokenDomain::Surface,
    },
    UiVisualToken {
        id: "ui.token.surface.overlay",
        role: UiVisualTokenRole::SurfaceOverlay,
        domain: UiVisualTokenDomain::Surface,
    },
    UiVisualToken {
        id: "ui.token.border.subtle",
        role: UiVisualTokenRole::BorderSubtle,
        domain: UiVisualTokenDomain::Border,
    },
    UiVisualToken {
        id: "ui.token.border.strong",
        role: UiVisualTokenRole::BorderStrong,
        domain: UiVisualTokenDomain::Border,
    },
    UiVisualToken {
        id: "ui.token.text.primary",
        role: UiVisualTokenRole::TextPrimary,
        domain: UiVisualTokenDomain::Text,
    },
    UiVisualToken {
        id: "ui.token.text.secondary",
        role: UiVisualTokenRole::TextSecondary,
        domain: UiVisualTokenDomain::Text,
    },
    UiVisualToken {
        id: "ui.token.accent.primary",
        role: UiVisualTokenRole::AccentPrimary,
        domain: UiVisualTokenDomain::Accent,
    },
    UiVisualToken {
        id: "ui.token.state.admitted",
        role: UiVisualTokenRole::StateAdmitted,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.state.denied",
        role: UiVisualTokenRole::StateDenied,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.state.failed",
        role: UiVisualTokenRole::StateFailed,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.state.quarantined",
        role: UiVisualTokenRole::StateQuarantined,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.state.conflict",
        role: UiVisualTokenRole::StateConflict,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.state.stale",
        role: UiVisualTokenRole::StateStale,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.state.preview",
        role: UiVisualTokenRole::StatePreview,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.state.simulation",
        role: UiVisualTokenRole::StateSimulation,
        domain: UiVisualTokenDomain::State,
    },
    UiVisualToken {
        id: "ui.token.trace.source",
        role: UiVisualTokenRole::TraceSource,
        domain: UiVisualTokenDomain::Trace,
    },
    UiVisualToken {
        id: "ui.token.trace.admission",
        role: UiVisualTokenRole::TraceAdmission,
        domain: UiVisualTokenDomain::Trace,
    },
    UiVisualToken {
        id: "ui.token.trace.action",
        role: UiVisualTokenRole::TraceAction,
        domain: UiVisualTokenDomain::Trace,
    },
    UiVisualToken {
        id: "ui.token.trace.effect",
        role: UiVisualTokenRole::TraceEffect,
        domain: UiVisualTokenDomain::Trace,
    },
    UiVisualToken {
        id: "ui.token.trace.audit",
        role: UiVisualTokenRole::TraceAudit,
        domain: UiVisualTokenDomain::Trace,
    },
    UiVisualToken {
        id: "ui.token.capability.available",
        role: UiVisualTokenRole::CapabilityAvailable,
        domain: UiVisualTokenDomain::Capability,
    },
    UiVisualToken {
        id: "ui.token.capability.missing",
        role: UiVisualTokenRole::CapabilityMissing,
        domain: UiVisualTokenDomain::Capability,
    },
    UiVisualToken {
        id: "ui.token.capability.denied",
        role: UiVisualTokenRole::CapabilityDenied,
        domain: UiVisualTokenDomain::Capability,
    },
    UiVisualToken {
        id: "ui.token.effect.requested",
        role: UiVisualTokenRole::EffectRequested,
        domain: UiVisualTokenDomain::Effect,
    },
    UiVisualToken {
        id: "ui.token.effect.prepared",
        role: UiVisualTokenRole::EffectPrepared,
        domain: UiVisualTokenDomain::Effect,
    },
    UiVisualToken {
        id: "ui.token.effect.committed",
        role: UiVisualTokenRole::EffectCommitted,
        domain: UiVisualTokenDomain::Effect,
    },
    UiVisualToken {
        id: "ui.token.effect.rolled_back",
        role: UiVisualTokenRole::EffectRolledBack,
        domain: UiVisualTokenDomain::Effect,
    },
    UiVisualToken {
        id: "ui.token.renderer.staged",
        role: UiVisualTokenRole::RendererStaged,
        domain: UiVisualTokenDomain::Renderer,
    },
    UiVisualToken {
        id: "ui.token.renderer.attempted",
        role: UiVisualTokenRole::RendererAttempted,
        domain: UiVisualTokenDomain::Renderer,
    },
    UiVisualToken {
        id: "ui.token.renderer.presented",
        role: UiVisualTokenRole::RendererPresented,
        domain: UiVisualTokenDomain::Renderer,
    },
    UiVisualToken {
        id: "ui.token.workbench.local",
        role: UiVisualTokenRole::WorkbenchLocal,
        domain: UiVisualTokenDomain::Workbench,
    },
    UiVisualToken {
        id: "ui.token.workbench.projection",
        role: UiVisualTokenRole::WorkbenchProjection,
        domain: UiVisualTokenDomain::Workbench,
    },
];

pub fn ui_visual_tokens() -> &'static [UiVisualToken] {
    UI_VISUAL_TOKENS
}

pub fn find_ui_visual_token(id: &str) -> Option<&'static UiVisualToken> {
    UI_VISUAL_TOKENS.iter().find(|token| token.id == id)
}
