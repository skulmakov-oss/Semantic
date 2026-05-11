//! Spec-aligned skeleton boundary for `prom-ui-runtime`.
//!
//! This module is intentionally narrow: it defines the local runtime seam
//! between admitted UI requests and a future platform adapter without owning
//! any OS window backend, renderer, or ABI surface.

use alloc::vec::Vec;

/// Logical identifier for a runtime adapter request.
///
/// This is session-local bookkeeping, not a capability and not a platform
/// handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdapterRequestId(pub u64);

/// Logical identifier for a runtime-managed window.
///
/// This is a session-local logical ID, not an OS handle and not a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u64);

/// Logical identifier for a runtime-managed frame.
///
/// This is a session-local logical ID, not a platform handle and not a
/// capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameId(pub u64);

/// Logical identifier for a bounded draw batch.
///
/// This is a session-local logical ID, not a renderer object and not a
/// capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DrawBatchId(pub u64);

/// Local runtime effect kinds admitted by the adapter boundary skeleton.
///
/// This is not an ABI opcode and does not imply real execution semantics.
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
/// not expose VM state, OS handles, renderer internals, or ABI details.
pub trait UiRuntimeAdapter {
    fn submit(&mut self, request: UiAdapterRequest) -> UiAdapterResult;
}

/// Deterministic no-op recording adapter for the boundary skeleton.
///
/// It records requests in order and returns a normalized synthetic result.
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
    fn logical_ids_are_separate_newtypes() {
        let request = UiAdapterRequest::new(
            AdapterRequestId(9),
            UiRuntimeEffect::BeginFrame,
            UiAdapterTarget::frame(WindowId(100), FrameId(200)),
        );

        assert_eq!(request.request_id, AdapterRequestId(9));
        assert_eq!(request.target.window_id, Some(WindowId(100)));
        assert_eq!(request.target.frame_id, Some(FrameId(200)));
        assert_eq!(request.target.draw_batch_id, None);
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
        assert_eq!(
            UiAdapterRejectKind::UnsupportedEffect,
            UiAdapterRejectKind::UnsupportedEffect
        );
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
