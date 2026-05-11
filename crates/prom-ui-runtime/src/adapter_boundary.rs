//! Spec-aligned skeleton boundary for `prom-ui-runtime`.
//!
//! This module defines the local seam between admitted, normalized UI runtime
//! requests and a future platform adapter.
//!
//! It is intentionally narrow:
//!
//! - it defines logical IDs;
//! - it defines a local `UiRuntimeEffect` taxonomy;
//! - it defines normalized adapter request/result types;
//! - it defines the `UiRuntimeAdapter` trait seam;
//! - it provides `RecordingAdapter` as a deterministic no-op test adapter.
//!
//! ## Explicit non-goals
//!
//! This module does not provide:
//!
//! - real platform backend execution;
//! - OS window creation;
//! - native event loop ownership;
//! - renderer or GPU access;
//! - shader, texture, or font handling;
//! - draw command binary encoding;
//! - capability admission;
//! - budget accounting;
//! - audit admission or persistence;
//! - ABI host-call surface;
//! - VM integration.
//!
//! ## Boundary invariants
//!
//! - `AdapterRequestId`, `WindowId`, `FrameId`, and `DrawBatchId` are logical
//!   session-local IDs.
//! - Logical IDs are not OS handles.
//! - Logical IDs are not capabilities.
//! - `UiRuntimeEffect` is a local runtime boundary enum, not an ABI opcode.
//! - `UiAdapterRequest` is a normalized local request, not a VM instruction.
//! - `UiAdapterResult::Rejected` is an adapter-boundary rejection, not a
//!   capability denial.
//! - `UiAdapterResult::Failed` is an adapter/platform-style failure, not a
//!   façade shape rejection.
//! - `RecordingAdapter` records requests and returns synthetic normalized
//!   results. It is not a backend and must not create windows or render pixels.
//! - The adapter boundary must not expose VM state, source text, compiler IR,
//!   OS handles, GPU handles, callbacks, or platform-native event structs.
//!
//! ## Future extension rule
//!
//! Any future platform adapter, renderer, event loop, ABI integration,
//! capability enforcement, or audit/budget integration must be added in a
//! separate PR with an explicit boundary review.

use alloc::vec::Vec;

/// Logical identifier for a runtime adapter request.
///
/// This is session-local bookkeeping, not a capability, not a platform handle,
/// and not a stable host resource identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdapterRequestId(pub u64);

/// Logical identifier for a runtime-managed window.
///
/// This is a session-local logical ID, not an OS handle, not a capability, and
/// not a stable host resource identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u64);

/// Logical identifier for a runtime-managed frame.
///
/// This is a session-local logical ID, not a platform handle, not a
/// capability, and not a stable host resource identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameId(pub u64);

/// Logical identifier for a bounded draw batch.
///
/// This is a session-local logical ID, not a renderer object, not a
/// capability, and not a stable host resource identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DrawBatchId(pub u64);

/// Local runtime effect kinds admitted by the adapter boundary skeleton.
///
/// This is not an ABI opcode and does not imply real execution semantics.
/// Adding a new effect here does not add a VM opcode, ABI call, capability, or
/// platform implementation by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRuntimeEffect {
    WindowCreate,
    WindowClose,
    PollEvents,
    BeginFrame,
    SubmitDrawCommands,
    EndFrame,
}

/// Normalized target for an admitted runtime request.
///
/// The target uses logical IDs only. It does not carry OS handles or other
/// platform-specific execution details.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UiAdapterTarget {
    pub window_id: Option<WindowId>,
    pub frame_id: Option<FrameId>,
    pub draw_batch_id: Option<DrawBatchId>,
}

impl UiAdapterTarget {
    pub const fn new(
        window_id: Option<WindowId>,
        frame_id: Option<FrameId>,
        draw_batch_id: Option<DrawBatchId>,
    ) -> Self {
        Self {
            window_id,
            frame_id,
            draw_batch_id,
        }
    }

    pub const fn window(window_id: WindowId) -> Self {
        Self::new(Some(window_id), None, None)
    }

    pub const fn frame(window_id: WindowId, frame_id: FrameId) -> Self {
        Self::new(Some(window_id), Some(frame_id), None)
    }

    pub const fn draw_batch(
        window_id: WindowId,
        frame_id: FrameId,
        draw_batch_id: DrawBatchId,
    ) -> Self {
        Self::new(Some(window_id), Some(frame_id), Some(draw_batch_id))
    }
}

/// Normalized runtime request delivered to the adapter boundary.
///
/// This request is assumed to be produced after higher-level admission or
/// local façade validation. It must not carry raw host objects.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiAdapterRequest {
    pub request_id: AdapterRequestId,
    pub effect_id: UiRuntimeEffect,
    pub target: UiAdapterTarget,
}

impl UiAdapterRequest {
    pub const fn new(
        request_id: AdapterRequestId,
        effect_id: UiRuntimeEffect,
        target: UiAdapterTarget,
    ) -> Self {
        Self {
            request_id,
            effect_id,
            target,
        }
    }
}

/// Synthetic value returned by the adapter boundary.
///
/// The skeleton keeps this intentionally small and normalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAdapterValue {
    Unit,
}

/// Reasons the adapter boundary can reject a request before attempting any
/// platform-specific execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAdapterRejectKind {
    UnsupportedEffect,
    InvalidRequest,
}

/// Normalized rejection returned by the adapter boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiAdapterReject {
    pub kind: UiAdapterRejectKind,
}

impl UiAdapterReject {
    pub const fn new(kind: UiAdapterRejectKind) -> Self {
        Self { kind }
    }

    pub const fn unsupported_effect() -> Self {
        Self::new(UiAdapterRejectKind::UnsupportedEffect)
    }

    pub const fn invalid_request() -> Self {
        Self::new(UiAdapterRejectKind::InvalidRequest)
    }
}

/// Reasons the adapter boundary can fail after a request has been admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAdapterFailureKind {
    PlatformFailure,
    BackendUnavailable,
}

/// Normalized failure returned by the adapter boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiAdapterFailure {
    pub kind: UiAdapterFailureKind,
}

impl UiAdapterFailure {
    pub const fn new(kind: UiAdapterFailureKind) -> Self {
        Self { kind }
    }

    pub const fn platform_failure() -> Self {
        Self::new(UiAdapterFailureKind::PlatformFailure)
    }

    pub const fn backend_unavailable() -> Self {
        Self::new(UiAdapterFailureKind::BackendUnavailable)
    }
}

/// Normalized result returned by the adapter boundary.
///
/// `Rejected` and `Failed` are adapter outcomes. They do not represent
/// capability admission decisions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UiAdapterResult {
    Performed(UiAdapterValue),
    Rejected(UiAdapterReject),
    Failed(UiAdapterFailure),
}

impl UiAdapterResult {
    pub const fn performed(value: UiAdapterValue) -> Self {
        Self::Performed(value)
    }

    pub const fn rejected(kind: UiAdapterRejectKind) -> Self {
        Self::Rejected(UiAdapterReject::new(kind))
    }

    pub const fn failed(kind: UiAdapterFailureKind) -> Self {
        Self::Failed(UiAdapterFailure::new(kind))
    }
}

/// Boundary seam for future platform adapters.
///
/// The trait is intentionally small and local to `prom-ui-runtime`. It does
/// not expose VM state, OS handles, renderer internals, ABI details, or
/// capability/audit policy decisions.
pub trait UiRuntimeAdapter {
    fn submit(&mut self, request: UiAdapterRequest) -> UiAdapterResult;
}

/// Deterministic no-op recording adapter for the boundary skeleton.
///
/// It records requests in order and returns a normalized synthetic result.
/// This adapter is deterministic and side-effect-free except for recording
/// submitted requests in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingAdapter {
    requests: Vec<UiAdapterRequest>,
    synthetic_result: UiAdapterResult,
}

impl RecordingAdapter {
    pub fn new() -> Self {
        Self::with_result(UiAdapterResult::performed(UiAdapterValue::Unit))
    }

    pub fn with_result(synthetic_result: UiAdapterResult) -> Self {
        Self {
            requests: Vec::new(),
            synthetic_result,
        }
    }

    pub fn requests(&self) -> &[UiAdapterRequest] {
        &self.requests
    }

    pub fn into_requests(self) -> Vec<UiAdapterRequest> {
        self.requests
    }
}

impl Default for RecordingAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl UiRuntimeAdapter for RecordingAdapter {
    fn submit(&mut self, request: UiAdapterRequest) -> UiAdapterResult {
        self.requests.push(request);
        self.synthetic_result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_adapter_records_requests_in_order() {
        let mut adapter = RecordingAdapter::new();

        let first = UiAdapterRequest::new(
            AdapterRequestId(1),
            UiRuntimeEffect::WindowCreate,
            UiAdapterTarget::window(WindowId(10)),
        );
        let second = UiAdapterRequest::new(
            AdapterRequestId(2),
            UiRuntimeEffect::SubmitDrawCommands,
            UiAdapterTarget::draw_batch(WindowId(10), FrameId(11), DrawBatchId(12)),
        );

        assert_eq!(
            adapter.submit(first.clone()),
            UiAdapterResult::performed(UiAdapterValue::Unit)
        );
        assert_eq!(
            adapter.submit(second.clone()),
            UiAdapterResult::performed(UiAdapterValue::Unit)
        );

        assert_eq!(adapter.requests(), &[first, second]);
    }

    #[test]
    fn recording_adapter_returns_normalized_result() {
        let mut adapter = RecordingAdapter::new();
        let request = UiAdapterRequest::new(
            AdapterRequestId(7),
            UiRuntimeEffect::PollEvents,
            UiAdapterTarget::window(WindowId(42)),
        );

        let result = adapter.submit(request);

        assert_eq!(result, UiAdapterResult::performed(UiAdapterValue::Unit));
    }

    #[test]
    fn window_target_does_not_include_frame_or_batch() {
        let target = UiAdapterTarget::window(WindowId(1));

        assert_eq!(target.window_id, Some(WindowId(1)));
        assert_eq!(target.frame_id, None);
        assert_eq!(target.draw_batch_id, None);
    }

    #[test]
    fn frame_target_does_not_include_draw_batch() {
        let target = UiAdapterTarget::frame(WindowId(1), FrameId(2));

        assert_eq!(target.window_id, Some(WindowId(1)));
        assert_eq!(target.frame_id, Some(FrameId(2)));
        assert_eq!(target.draw_batch_id, None);
    }

    #[test]
    fn draw_batch_target_contains_all_logical_ids() {
        let target = UiAdapterTarget::draw_batch(WindowId(1), FrameId(2), DrawBatchId(3));

        assert_eq!(target.window_id, Some(WindowId(1)));
        assert_eq!(target.frame_id, Some(FrameId(2)));
        assert_eq!(target.draw_batch_id, Some(DrawBatchId(3)));
    }

    #[test]
    fn request_identity_is_not_target_identity() {
        let request_id = AdapterRequestId(1);
        let window_id = WindowId(1);
        let frame_id = FrameId(2);
        let request = UiAdapterRequest::new(
            request_id,
            UiRuntimeEffect::BeginFrame,
            UiAdapterTarget::frame(window_id, frame_id),
        );

        assert_eq!(request.request_id, request_id);
        assert_eq!(request.target.window_id, Some(window_id));
        assert_eq!(request.target.frame_id, Some(frame_id));
        assert_eq!(request.target.draw_batch_id, None);
    }

    #[test]
    fn recording_adapter_preserves_rejected_result() {
        let mut adapter = RecordingAdapter::with_result(UiAdapterResult::rejected(
            UiAdapterRejectKind::UnsupportedEffect,
        ));
        let request = UiAdapterRequest::new(
            AdapterRequestId(5),
            UiRuntimeEffect::SubmitDrawCommands,
            UiAdapterTarget::draw_batch(WindowId(9), FrameId(8), DrawBatchId(7)),
        );

        let result = adapter.submit(request.clone());

        assert_eq!(
            result,
            UiAdapterResult::Rejected(UiAdapterReject::unsupported_effect())
        );
        assert_eq!(adapter.requests(), &[request]);
    }

    #[test]
    fn recording_adapter_preserves_failed_result() {
        let mut adapter =
            RecordingAdapter::with_result(UiAdapterResult::failed(UiAdapterFailureKind::BackendUnavailable));
        let request = UiAdapterRequest::new(
            AdapterRequestId(6),
            UiRuntimeEffect::EndFrame,
            UiAdapterTarget::frame(WindowId(3), FrameId(4)),
        );

        let result = adapter.submit(request.clone());

        assert_eq!(
            result,
            UiAdapterResult::Failed(UiAdapterFailure::backend_unavailable())
        );
        assert_eq!(adapter.requests(), &[request]);
    }

    #[test]
    fn invalid_request_is_rejection_not_failure() {
        let result = UiAdapterResult::rejected(UiAdapterRejectKind::InvalidRequest);

        assert!(matches!(
            result,
            UiAdapterResult::Rejected(UiAdapterReject {
                kind: UiAdapterRejectKind::InvalidRequest
            })
        ));
        assert!(!matches!(result, UiAdapterResult::Failed(_)));
    }

    #[test]
    fn backend_unavailable_is_failure_not_rejection() {
        let result = UiAdapterResult::failed(UiAdapterFailureKind::BackendUnavailable);

        assert!(matches!(
            result,
            UiAdapterResult::Failed(UiAdapterFailure {
                kind: UiAdapterFailureKind::BackendUnavailable
            })
        ));
        assert!(!matches!(result, UiAdapterResult::Rejected(_)));
    }

    #[test]
    fn into_requests_preserves_recorded_order() {
        let mut adapter = RecordingAdapter::new();
        let first = UiAdapterRequest::new(
            AdapterRequestId(11),
            UiRuntimeEffect::WindowClose,
            UiAdapterTarget::window(WindowId(1)),
        );
        let second = UiAdapterRequest::new(
            AdapterRequestId(12),
            UiRuntimeEffect::PollEvents,
            UiAdapterTarget::window(WindowId(2)),
        );
        let third = UiAdapterRequest::new(
            AdapterRequestId(13),
            UiRuntimeEffect::BeginFrame,
            UiAdapterTarget::frame(WindowId(3), FrameId(4)),
        );

        let _ = adapter.submit(first.clone());
        let _ = adapter.submit(second.clone());
        let _ = adapter.submit(third.clone());

        assert_eq!(adapter.into_requests(), vec![first, second, third]);
    }

    #[test]
    fn recording_adapter_default_has_no_requests() {
        let adapter = RecordingAdapter::default();

        assert!(adapter.requests().is_empty());
    }

    #[test]
    fn unit_result_is_normalized_success_value() {
        let result = UiAdapterResult::performed(UiAdapterValue::Unit);

        assert!(matches!(
            result,
            UiAdapterResult::Performed(UiAdapterValue::Unit)
        ));
    }

    #[test]
    fn rejection_and_failure_categories_are_distinct() {
        let rejected = UiAdapterResult::Rejected(UiAdapterReject::unsupported_effect());
        let invalid = UiAdapterResult::Rejected(UiAdapterReject::invalid_request());
        let platform_failure = UiAdapterResult::Failed(UiAdapterFailure::platform_failure());
        let backend_unavailable = UiAdapterResult::Failed(UiAdapterFailure::backend_unavailable());

        assert_ne!(rejected, invalid);
        assert_ne!(platform_failure, backend_unavailable);
        assert_ne!(rejected, platform_failure);
        assert_eq!(UiAdapterRejectKind::UnsupportedEffect, UiAdapterRejectKind::UnsupportedEffect);
        assert_ne!(
            UiAdapterRejectKind::UnsupportedEffect,
            UiAdapterRejectKind::InvalidRequest
        );
        assert_ne!(
            UiAdapterFailureKind::PlatformFailure,
            UiAdapterFailureKind::BackendUnavailable
        );
    }
}
