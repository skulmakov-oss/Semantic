//! UI boundary types and capability surface for Semantic desktop applications.
//!
//! This crate owns the UI operation identity, capability taxonomy, and
//! admitted boundary types for the first-wave UI application boundary track.
//!
//! # Current Wave Status
//!
//! Wave 0 scaffolding only. All types are inert markers. Executable admission,
//! runtime session ownership, and drawing surface are deferred to Wave 1+.
//!
//! # Non-Commitments
//!
//! This crate does not claim:
//! - a general widget/layout framework
//! - multi-window, browser, or mobile UI support
//! - a forked graphics stack or shader-language ownership
//! - that UI support is already part of the published `v1.1.1` line
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod interaction;
pub mod action_binding;
pub mod action_binding_trace;
pub mod action_binding_stream;
pub mod intent_stream;
pub mod raw_event;
pub mod trace;

pub use interaction::{
    ElementId, InteractionModifiers, InteractionSource, InteractionTarget, RegionId,
    SurfaceId, WindowId, InteractionIntentDescriptor, InteractionIntentId,
    InteractionIntentKind,
};
pub use action_binding::{
    find_interaction_action_binding_by_action,
    find_interaction_action_binding_by_intent,
    interaction_action_bindings,
    InteractionActionAdmissionPolicy,
    InteractionActionBindingDescriptor,
    InteractionActionBindingId,
    InteractionActionBindingScope,
    InteractionActionEffectPolicy,
    InteractionActionName,
    InteractionActionTargetPolicy,
    InteractionActionTracePolicy,
    INTERACTION_ACTION_BINDINGS,
};
pub use action_binding_trace::{
    trace_interaction_action_binding, InteractionActionBindingTraceReason,
    InteractionActionBindingTraceReport, InteractionActionBindingTraceStatus,
};
pub use action_binding_stream::{
    trace_interaction_action_binding_stream, InteractionActionBindingTraceStreamModel,
    InteractionActionBindingTraceStreamReport, InteractionActionBindingTraceStreamStats,
};
pub use intent_stream::{
    trace_interaction_intent_stream, InteractionIntentStreamModel,
    InteractionIntentStreamReport, InteractionIntentStreamStats,
};
pub use raw_event::{
    map_raw_event_to_interaction_intent, RawKeyCode, RawPointerButton, RawTextInput,
    RawUiEvent, RawUiEventId, RawUiEventKind, RawUiEventPayload, RawUiEventTarget,
};
pub use trace::{
    trace_raw_event_to_interaction_intent, InteractionIntentMappingRule,
    InteractionIntentTraceReason, InteractionIntentTraceReport, InteractionIntentTraceStatus,
};

/// Marker for the UI application boundary capability family.
///
/// Inert at Wave 0. Capability taxonomy and operation identity are deferred
/// to Wave 1 boundary admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiCapabilityKind {
    /// Capability to own and run a single desktop window session.
    DesktopSession,
    /// Capability to poll input events within an admitted desktop session.
    InputPoll,
    /// Capability to emit a frame of draw commands through the admitted surface.
    FrameEmit,
}

/// Marker for admitted UI operation identities.
///
/// Inert at Wave 0. Executable admission through verifier/VM is deferred
/// to Wave 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiOperationId {
    /// Create a single desktop window.
    WindowCreate,
    /// Run the desktop event/frame loop.
    WindowRun,
    /// Close the desktop window and end the session.
    WindowClose,
    /// Poll pending input events for the current frame.
    EventPoll,
    /// Submit a frame of draw commands.
    FrameSubmit,
}

/// Surface class for UI capability kinds.
///
/// All UI capabilities are post-stable. None are part of the published
/// `v1.1.1` stable commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCapabilitySurfaceClass {
    /// Post-stable first-wave UI boundary.
    PostStableFirstWave,
}

pub const fn ui_capability_surface_class(_kind: UiCapabilityKind) -> UiCapabilitySurfaceClass {
    UiCapabilitySurfaceClass::PostStableFirstWave
}

/// Required capability for a given UI operation.
///
/// Inert at Wave 0. This mapping will be wired through the verifier and
/// capability checker in Wave 1.
pub const fn required_ui_capability(op: UiOperationId) -> UiCapabilityKind {
    match op {
        UiOperationId::WindowCreate => UiCapabilityKind::DesktopSession,
        UiOperationId::WindowRun => UiCapabilityKind::DesktopSession,
        UiOperationId::WindowClose => UiCapabilityKind::DesktopSession,
        UiOperationId::EventPoll => UiCapabilityKind::InputPoll,
        UiOperationId::FrameSubmit => UiCapabilityKind::FrameEmit,
    }
}
