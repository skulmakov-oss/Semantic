//! Local admission façade skeleton for `prom-ui-runtime`.
//!
//! This module validates only the local request shape before mapping it to
//! the adapter boundary. It does not perform real capability enforcement,
//! budget accounting, audit admission, or platform execution.

use crate::adapter_boundary::{
    AdapterRequestId, DrawBatchId, FrameId, UiAdapterRequest, UiAdapterResult, UiAdapterTarget,
    UiRuntimeAdapter, UiRuntimeEffect, WindowId,
};

/// Local runtime request shape accepted by the admission façade.
///
/// This is a runtime-local shape, not a public ABI envelope and not a
/// capability authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRuntimeEffectRequest {
    pub request_id: AdapterRequestId,
    pub effect_id: UiRuntimeEffect,
    pub target: UiAdapterTarget,
}

impl UiRuntimeEffectRequest {
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

    pub const fn into_adapter_request(self) -> UiAdapterRequest {
        UiAdapterRequest::new(self.request_id, self.effect_id, self.target)
    }

    pub const fn window_create(request_id: AdapterRequestId) -> Self {
        Self::new(
            request_id,
            UiRuntimeEffect::WindowCreate,
            UiAdapterTarget::new(None, None, None),
        )
    }

    pub const fn window_close(request_id: AdapterRequestId, window_id: WindowId) -> Self {
        Self::new(
            request_id,
            UiRuntimeEffect::WindowClose,
            UiAdapterTarget::window(window_id),
        )
    }

    pub const fn poll_events(request_id: AdapterRequestId) -> Self {
        Self::new(
            request_id,
            UiRuntimeEffect::PollEvents,
            UiAdapterTarget::new(None, None, None),
        )
    }

    pub const fn poll_events_for_window(
        request_id: AdapterRequestId,
        window_id: WindowId,
    ) -> Self {
        Self::new(
            request_id,
            UiRuntimeEffect::PollEvents,
            UiAdapterTarget::window(window_id),
        )
    }

    pub const fn begin_frame(request_id: AdapterRequestId, window_id: WindowId) -> Self {
        Self::new(
            request_id,
            UiRuntimeEffect::BeginFrame,
            UiAdapterTarget::window(window_id),
        )
    }

    pub const fn submit_draw_commands(
        request_id: AdapterRequestId,
        window_id: WindowId,
        frame_id: FrameId,
        draw_batch_id: DrawBatchId,
    ) -> Self {
        Self::new(
            request_id,
            UiRuntimeEffect::SubmitDrawCommands,
            UiAdapterTarget::draw_batch(window_id, frame_id, draw_batch_id),
        )
    }

    pub const fn end_frame(
        request_id: AdapterRequestId,
        window_id: WindowId,
        frame_id: FrameId,
    ) -> Self {
        Self::new(
            request_id,
            UiRuntimeEffect::EndFrame,
            UiAdapterTarget::frame(window_id, frame_id),
        )
    }
}

/// Reasons the local façade can reject an effect request before dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAdmissionRejectKind {
    InvalidTargetForEffect,
}

/// Local façade rejection payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiAdmissionReject {
    pub kind: UiAdmissionRejectKind,
    pub effect_id: UiRuntimeEffect,
}

impl UiAdmissionReject {
    pub const fn new(kind: UiAdmissionRejectKind, effect_id: UiRuntimeEffect) -> Self {
        Self { kind, effect_id }
    }

    pub const fn invalid_target_for_effect(effect_id: UiRuntimeEffect) -> Self {
        Self::new(UiAdmissionRejectKind::InvalidTargetForEffect, effect_id)
    }
}

/// Result of the local admission façade.
///
/// `Rejected` means the request failed local shape validation and never
/// reached the adapter. `Submitted` means the request was shaped correctly
/// and the adapter boundary produced the final adapter result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAdmissionResult {
    Submitted(UiAdapterResult),
    Rejected(UiAdmissionReject),
}

impl UiAdmissionResult {
    pub const fn submitted(result: UiAdapterResult) -> Self {
        Self::Submitted(result)
    }

    pub const fn rejected(reject: UiAdmissionReject) -> Self {
        Self::Rejected(reject)
    }

    pub const fn is_submitted(&self) -> bool {
        matches!(self, Self::Submitted(_))
    }

    pub const fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected(_))
    }

    pub const fn as_submitted(&self) -> Option<&UiAdapterResult> {
        match self {
            Self::Submitted(result) => Some(result),
            Self::Rejected(_) => None,
        }
    }

    pub const fn as_rejected(&self) -> Option<&UiAdmissionReject> {
        match self {
            Self::Rejected(reject) => Some(reject),
            Self::Submitted(_) => None,
        }
    }
}

/// Local façade that validates request shape and then forwards admitted
/// requests to the adapter boundary.
pub struct UiAdmissionFacade<A: UiRuntimeAdapter> {
    adapter: A,
}

impl<A: UiRuntimeAdapter> UiAdmissionFacade<A> {
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }

    pub fn adapter_mut(&mut self) -> &mut A {
        &mut self.adapter
    }

    pub fn submit_admitted(&mut self, request: UiRuntimeEffectRequest) -> UiAdmissionResult {
        if !target_shape_is_valid(request.effect_id, &request.target) {
            return UiAdmissionResult::rejected(UiAdmissionReject::invalid_target_for_effect(
                request.effect_id,
            ));
        }

        let adapter_request = request.into_adapter_request();

        UiAdmissionResult::submitted(self.adapter.submit(adapter_request))
    }
}

fn target_shape_is_valid(effect_id: UiRuntimeEffect, target: &UiAdapterTarget) -> bool {
    match effect_id {
        UiRuntimeEffect::WindowCreate => target.frame_id.is_none() && target.draw_batch_id.is_none(),
        UiRuntimeEffect::WindowClose => {
            target.window_id.is_some()
                && target.frame_id.is_none()
                && target.draw_batch_id.is_none()
        }
        UiRuntimeEffect::PollEvents => {
            target.frame_id.is_none() && target.draw_batch_id.is_none()
        }
        UiRuntimeEffect::BeginFrame => {
            target.window_id.is_some()
                && target.frame_id.is_none()
                && target.draw_batch_id.is_none()
        }
        UiRuntimeEffect::SubmitDrawCommands => {
            target.window_id.is_some()
                && target.frame_id.is_some()
                && target.draw_batch_id.is_some()
        }
        UiRuntimeEffect::EndFrame => {
            target.window_id.is_some()
                && target.frame_id.is_some()
                && target.draw_batch_id.is_none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter_boundary::{
        DrawBatchId, FrameId, RecordingAdapter, UiAdapterFailure, UiAdapterFailureKind,
        UiAdapterReject, UiAdapterRejectKind, UiAdapterResult, UiAdapterTarget, UiAdapterValue,
        UiRuntimeEffect, WindowId,
    };

    fn request(
        request_id: u64,
        effect_id: UiRuntimeEffect,
        target: UiAdapterTarget,
    ) -> UiRuntimeEffectRequest {
        UiRuntimeEffectRequest::new(AdapterRequestId(request_id), effect_id, target)
    }

    #[derive(Debug, Clone, Copy)]
    enum TargetShape {
        Empty,
        WindowOnly,
        Frame,
        DrawBatch,
        FrameWithoutWindow,
        DrawBatchWithoutFrame,
        DrawBatchWithoutWindow,
    }

    fn target_shape(shape: TargetShape) -> UiAdapterTarget {
        match shape {
            TargetShape::Empty => UiAdapterTarget::default(),
            TargetShape::WindowOnly => UiAdapterTarget::window(WindowId(1)),
            TargetShape::Frame => UiAdapterTarget::frame(WindowId(1), FrameId(2)),
            TargetShape::DrawBatch => {
                UiAdapterTarget::draw_batch(WindowId(1), FrameId(2), DrawBatchId(3))
            }
            TargetShape::FrameWithoutWindow => UiAdapterTarget::new(None, Some(FrameId(2)), None),
            TargetShape::DrawBatchWithoutFrame => {
                UiAdapterTarget::new(Some(WindowId(1)), None, Some(DrawBatchId(3)))
            }
            TargetShape::DrawBatchWithoutWindow => {
                UiAdapterTarget::new(None, Some(FrameId(2)), Some(DrawBatchId(3)))
            }
        }
    }

    #[test]
    fn admission_result_helpers_identify_submitted() {
        let result = UiAdmissionResult::submitted(UiAdapterResult::performed(
            UiAdapterValue::Unit,
        ));

        assert!(result.is_submitted());
        assert!(!result.is_rejected());
        assert!(matches!(result.as_submitted(), Some(UiAdapterResult::Performed(UiAdapterValue::Unit))));
        assert_eq!(result.as_rejected(), None);
    }

    #[test]
    fn admission_result_helpers_identify_rejected() {
        let reject = UiAdmissionReject::invalid_target_for_effect(UiRuntimeEffect::EndFrame);
        let result = UiAdmissionResult::rejected(reject);

        assert!(!result.is_submitted());
        assert!(result.is_rejected());
        assert_eq!(result.as_submitted(), None);
        assert_eq!(result.as_rejected(), Some(&reject));
    }

    #[test]
    fn admission_result_as_submitted_returns_adapter_result() {
        let adapter_result = UiAdapterResult::failed(UiAdapterFailureKind::BackendUnavailable);
        let result = UiAdmissionResult::submitted(adapter_result.clone());

        assert_eq!(result.as_submitted(), Some(&adapter_result));
        assert_eq!(result.as_rejected(), None);
    }

    #[test]
    fn admission_result_as_rejected_returns_reject_payload() {
        let reject = UiAdmissionReject::new(
            UiAdmissionRejectKind::InvalidTargetForEffect,
            UiRuntimeEffect::SubmitDrawCommands,
        );
        let result = UiAdmissionResult::rejected(reject);

        assert_eq!(result.as_submitted(), None);
        assert_eq!(result.as_rejected(), Some(&reject));
    }

    #[test]
    fn admission_reject_constructor_sets_kind_and_effect() {
        let reject = UiAdmissionReject::invalid_target_for_effect(UiRuntimeEffect::WindowClose);

        assert_eq!(reject.kind, UiAdmissionRejectKind::InvalidTargetForEffect);
        assert_eq!(reject.effect_id, UiRuntimeEffect::WindowClose);
    }

    #[test]
    fn request_builder_window_create_uses_empty_target() {
        let request = UiRuntimeEffectRequest::window_create(AdapterRequestId(7_000));

        assert_eq!(request.request_id, AdapterRequestId(7_000));
        assert_eq!(request.effect_id, UiRuntimeEffect::WindowCreate);
        assert_eq!(request.target, UiAdapterTarget::default());
    }

    #[test]
    fn request_builder_window_close_uses_window_target() {
        let request = UiRuntimeEffectRequest::window_close(AdapterRequestId(7_001), WindowId(41));

        assert_eq!(request.request_id, AdapterRequestId(7_001));
        assert_eq!(request.effect_id, UiRuntimeEffect::WindowClose);
        assert_eq!(request.target, UiAdapterTarget::window(WindowId(41)));
    }

    #[test]
    fn request_builder_poll_events_uses_empty_target() {
        let request = UiRuntimeEffectRequest::poll_events(AdapterRequestId(7_002));

        assert_eq!(request.request_id, AdapterRequestId(7_002));
        assert_eq!(request.effect_id, UiRuntimeEffect::PollEvents);
        assert_eq!(request.target, UiAdapterTarget::default());
    }

    #[test]
    fn request_builder_poll_events_for_window_uses_window_target() {
        let request =
            UiRuntimeEffectRequest::poll_events_for_window(AdapterRequestId(7_003), WindowId(42));

        assert_eq!(request.request_id, AdapterRequestId(7_003));
        assert_eq!(request.effect_id, UiRuntimeEffect::PollEvents);
        assert_eq!(request.target, UiAdapterTarget::window(WindowId(42)));
    }

    #[test]
    fn request_builder_begin_frame_uses_window_target() {
        let request = UiRuntimeEffectRequest::begin_frame(AdapterRequestId(7_004), WindowId(43));

        assert_eq!(request.request_id, AdapterRequestId(7_004));
        assert_eq!(request.effect_id, UiRuntimeEffect::BeginFrame);
        assert_eq!(request.target, UiAdapterTarget::window(WindowId(43)));
    }

    #[test]
    fn request_builder_submit_draw_commands_uses_draw_batch_target() {
        let request = UiRuntimeEffectRequest::submit_draw_commands(
            AdapterRequestId(7_005),
            WindowId(44),
            FrameId(45),
            DrawBatchId(46),
        );

        assert_eq!(request.request_id, AdapterRequestId(7_005));
        assert_eq!(request.effect_id, UiRuntimeEffect::SubmitDrawCommands);
        assert_eq!(
            request.target,
            UiAdapterTarget::draw_batch(WindowId(44), FrameId(45), DrawBatchId(46))
        );
    }

    #[test]
    fn request_builder_end_frame_uses_frame_target() {
        let request =
            UiRuntimeEffectRequest::end_frame(AdapterRequestId(7_006), WindowId(47), FrameId(48));

        assert_eq!(request.request_id, AdapterRequestId(7_006));
        assert_eq!(request.effect_id, UiRuntimeEffect::EndFrame);
        assert_eq!(request.target, UiAdapterTarget::frame(WindowId(47), FrameId(48)));
    }

    #[test]
    fn request_builders_all_pass_existing_shape_validation() {
        let requests = [
            UiRuntimeEffectRequest::window_create(AdapterRequestId(7_100)),
            UiRuntimeEffectRequest::window_close(AdapterRequestId(7_101), WindowId(51)),
            UiRuntimeEffectRequest::poll_events(AdapterRequestId(7_102)),
            UiRuntimeEffectRequest::poll_events_for_window(AdapterRequestId(7_103), WindowId(52)),
            UiRuntimeEffectRequest::begin_frame(AdapterRequestId(7_104), WindowId(53)),
            UiRuntimeEffectRequest::submit_draw_commands(
                AdapterRequestId(7_105),
                WindowId(54),
                FrameId(55),
                DrawBatchId(56),
            ),
            UiRuntimeEffectRequest::end_frame(AdapterRequestId(7_106), WindowId(57), FrameId(58)),
        ];

        let mut facade = UiAdmissionFacade::new(RecordingAdapter::new());

        for request in requests {
            let result = facade.submit_admitted(request);
            assert!(matches!(result, UiAdmissionResult::Submitted(_)));
        }

        assert_eq!(facade.adapter().requests().len(), 7);
    }

    #[test]
    fn runtime_effect_request_converts_to_adapter_request_without_rewriting_fields() {
        let request = UiRuntimeEffectRequest::new(
            AdapterRequestId(9_999),
            UiRuntimeEffect::SubmitDrawCommands,
            UiAdapterTarget::draw_batch(WindowId(4), FrameId(5), DrawBatchId(6)),
        );

        let adapter_request = request.clone().into_adapter_request();

        assert_eq!(adapter_request.request_id, request.request_id);
        assert_eq!(adapter_request.effect_id, request.effect_id);
        assert_eq!(adapter_request.target, request.target);
    }

    #[test]
    fn window_create_accepts_empty_target() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            1,
            UiRuntimeEffect::WindowCreate,
            UiAdapterTarget::default(),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn window_close_requires_window_target() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            2,
            UiRuntimeEffect::WindowClose,
            UiAdapterTarget::window(WindowId(10)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn poll_events_accepts_empty_or_window_target() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            3,
            UiRuntimeEffect::PollEvents,
            UiAdapterTarget::window(WindowId(11)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn begin_frame_requires_window_only_target() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            4,
            UiRuntimeEffect::BeginFrame,
            UiAdapterTarget::window(WindowId(12)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn submit_draw_commands_requires_draw_batch_target() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            5,
            UiRuntimeEffect::SubmitDrawCommands,
            UiAdapterTarget::draw_batch(WindowId(13), FrameId(14), DrawBatchId(15)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn end_frame_requires_frame_target() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            6,
            UiRuntimeEffect::EndFrame,
            UiAdapterTarget::frame(WindowId(16), FrameId(17)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn invalid_submit_draw_commands_without_draw_batch_is_rejected() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            7,
            UiRuntimeEffect::SubmitDrawCommands,
            UiAdapterTarget::frame(WindowId(20), FrameId(21)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Rejected(UiAdmissionReject {
                kind: UiAdmissionRejectKind::InvalidTargetForEffect,
                effect_id: UiRuntimeEffect::SubmitDrawCommands
            })
        ));
        assert!(facade.adapter().requests().is_empty());
    }

    #[test]
    fn invalid_end_frame_with_draw_batch_is_rejected() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            8,
            UiRuntimeEffect::EndFrame,
            UiAdapterTarget::draw_batch(WindowId(22), FrameId(23), DrawBatchId(24)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Rejected(UiAdmissionReject {
                kind: UiAdmissionRejectKind::InvalidTargetForEffect,
                effect_id: UiRuntimeEffect::EndFrame
            })
        ));
        assert!(facade.adapter().requests().is_empty());
    }

    #[test]
    fn invalid_begin_frame_with_frame_id_is_rejected() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            9,
            UiRuntimeEffect::BeginFrame,
            UiAdapterTarget::frame(WindowId(26), FrameId(27)),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Rejected(UiAdmissionReject {
                kind: UiAdmissionRejectKind::InvalidTargetForEffect,
                effect_id: UiRuntimeEffect::BeginFrame
            })
        ));
        assert!(facade.adapter().requests().is_empty());
    }

    #[test]
    fn invalid_window_close_without_window_is_rejected() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            10,
            UiRuntimeEffect::WindowClose,
            UiAdapterTarget::default(),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Rejected(UiAdmissionReject {
                kind: UiAdmissionRejectKind::InvalidTargetForEffect,
                effect_id: UiRuntimeEffect::WindowClose
            })
        ));
        assert!(facade.adapter().requests().is_empty());
    }

    #[test]
    fn valid_submit_draw_commands_reaches_recording_adapter() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);
        let target = UiAdapterTarget::draw_batch(WindowId(30), FrameId(31), DrawBatchId(32));
        let req = request(11, UiRuntimeEffect::SubmitDrawCommands, target.clone());

        let result = facade.submit_admitted(req);

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
        assert_eq!(facade.adapter().requests()[0].effect_id, UiRuntimeEffect::SubmitDrawCommands);
        assert_eq!(facade.adapter().requests()[0].target, target);
    }

    #[test]
    fn adapter_rejection_passes_through_submitted_result() {
        let adapter = RecordingAdapter::with_result(UiAdapterResult::rejected(
            UiAdapterRejectKind::InvalidRequest,
        ));
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            12,
            UiRuntimeEffect::WindowCreate,
            UiAdapterTarget::default(),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Rejected(UiAdapterReject {
                kind: UiAdapterRejectKind::InvalidRequest
            }))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn adapter_failure_passes_through_submitted_result() {
        let adapter = RecordingAdapter::with_result(UiAdapterResult::failed(
            UiAdapterFailureKind::BackendUnavailable,
        ));
        let mut facade = UiAdmissionFacade::new(adapter);

        let result = facade.submit_admitted(request(
            13,
            UiRuntimeEffect::WindowCreate,
            UiAdapterTarget::default(),
        ));

        assert!(matches!(
            result,
            UiAdmissionResult::Submitted(UiAdapterResult::Failed(UiAdapterFailure {
                kind: UiAdapterFailureKind::BackendUnavailable
            }))
        ));
        assert_eq!(facade.adapter().requests().len(), 1);
    }

    #[test]
    fn valid_window_close_and_poll_events_shapes_pass() {
        let mut facade = UiAdmissionFacade::new(RecordingAdapter::new());

        let close_result = facade.submit_admitted(request(
            14,
            UiRuntimeEffect::WindowClose,
            UiAdapterTarget::window(WindowId(40)),
        ));
        assert!(matches!(
            close_result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));

        let poll_result = facade.submit_admitted(request(
            15,
            UiRuntimeEffect::PollEvents,
            UiAdapterTarget::default(),
        ));
        assert!(matches!(
            poll_result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        assert_eq!(facade.adapter().requests().len(), 2);
    }

    #[test]
    fn target_shape_matrix_matches_contract() {
        let cases = [
            (UiRuntimeEffect::WindowCreate, TargetShape::Empty, true),
            (UiRuntimeEffect::WindowCreate, TargetShape::WindowOnly, true),
            (UiRuntimeEffect::WindowCreate, TargetShape::Frame, false),
            (UiRuntimeEffect::WindowCreate, TargetShape::DrawBatch, false),
            (UiRuntimeEffect::WindowCreate, TargetShape::FrameWithoutWindow, false),
            (
                UiRuntimeEffect::WindowCreate,
                TargetShape::DrawBatchWithoutFrame,
                false,
            ),
            (
                UiRuntimeEffect::WindowCreate,
                TargetShape::DrawBatchWithoutWindow,
                false,
            ),
            (UiRuntimeEffect::WindowClose, TargetShape::Empty, false),
            (UiRuntimeEffect::WindowClose, TargetShape::WindowOnly, true),
            (UiRuntimeEffect::WindowClose, TargetShape::Frame, false),
            (UiRuntimeEffect::WindowClose, TargetShape::DrawBatch, false),
            (UiRuntimeEffect::WindowClose, TargetShape::FrameWithoutWindow, false),
            (
                UiRuntimeEffect::WindowClose,
                TargetShape::DrawBatchWithoutFrame,
                false,
            ),
            (
                UiRuntimeEffect::WindowClose,
                TargetShape::DrawBatchWithoutWindow,
                false,
            ),
            (UiRuntimeEffect::PollEvents, TargetShape::Empty, true),
            (UiRuntimeEffect::PollEvents, TargetShape::WindowOnly, true),
            (UiRuntimeEffect::PollEvents, TargetShape::Frame, false),
            (UiRuntimeEffect::PollEvents, TargetShape::DrawBatch, false),
            (UiRuntimeEffect::PollEvents, TargetShape::FrameWithoutWindow, false),
            (
                UiRuntimeEffect::PollEvents,
                TargetShape::DrawBatchWithoutFrame,
                false,
            ),
            (
                UiRuntimeEffect::PollEvents,
                TargetShape::DrawBatchWithoutWindow,
                false,
            ),
            (UiRuntimeEffect::BeginFrame, TargetShape::Empty, false),
            (UiRuntimeEffect::BeginFrame, TargetShape::WindowOnly, true),
            (UiRuntimeEffect::BeginFrame, TargetShape::Frame, false),
            (UiRuntimeEffect::BeginFrame, TargetShape::DrawBatch, false),
            (UiRuntimeEffect::BeginFrame, TargetShape::FrameWithoutWindow, false),
            (
                UiRuntimeEffect::BeginFrame,
                TargetShape::DrawBatchWithoutFrame,
                false,
            ),
            (
                UiRuntimeEffect::BeginFrame,
                TargetShape::DrawBatchWithoutWindow,
                false,
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::Empty,
                false,
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::WindowOnly,
                false,
            ),
            (UiRuntimeEffect::SubmitDrawCommands, TargetShape::Frame, false),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::DrawBatch,
                true,
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::FrameWithoutWindow,
                false,
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::DrawBatchWithoutFrame,
                false,
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::DrawBatchWithoutWindow,
                false,
            ),
            (UiRuntimeEffect::EndFrame, TargetShape::Empty, false),
            (UiRuntimeEffect::EndFrame, TargetShape::WindowOnly, false),
            (UiRuntimeEffect::EndFrame, TargetShape::Frame, true),
            (UiRuntimeEffect::EndFrame, TargetShape::DrawBatch, false),
            (UiRuntimeEffect::EndFrame, TargetShape::FrameWithoutWindow, false),
            (
                UiRuntimeEffect::EndFrame,
                TargetShape::DrawBatchWithoutFrame,
                false,
            ),
            (
                UiRuntimeEffect::EndFrame,
                TargetShape::DrawBatchWithoutWindow,
                false,
            ),
        ];

        for (index, (effect, shape, expected_valid)) in cases.into_iter().enumerate() {
            let adapter = RecordingAdapter::new();
            let mut facade = UiAdmissionFacade::new(adapter);

            let result = facade.submit_admitted(request(
                1_000 + index as u64,
                effect,
                target_shape(shape),
            ));

            assert_eq!(
                matches!(result, UiAdmissionResult::Submitted(_)),
                expected_valid,
                "effect={effect:?}, shape={shape:?}"
            );
            assert_eq!(
                facade.adapter().requests().len(),
                if expected_valid { 1 } else { 0 },
                "adapter submit count mismatch for effect={effect:?}, shape={shape:?}"
            );
        }
    }

    #[test]
    fn valid_matrix_cases_forward_exact_request() {
        let cases = [
            (
                UiRuntimeEffect::WindowCreate,
                TargetShape::Empty,
                UiAdapterTarget::default(),
            ),
            (
                UiRuntimeEffect::WindowCreate,
                TargetShape::WindowOnly,
                target_shape(TargetShape::WindowOnly),
            ),
            (
                UiRuntimeEffect::WindowClose,
                TargetShape::WindowOnly,
                target_shape(TargetShape::WindowOnly),
            ),
            (
                UiRuntimeEffect::PollEvents,
                TargetShape::Empty,
                UiAdapterTarget::default(),
            ),
            (
                UiRuntimeEffect::PollEvents,
                TargetShape::WindowOnly,
                target_shape(TargetShape::WindowOnly),
            ),
            (
                UiRuntimeEffect::BeginFrame,
                TargetShape::WindowOnly,
                target_shape(TargetShape::WindowOnly),
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::DrawBatch,
                target_shape(TargetShape::DrawBatch),
            ),
            (
                UiRuntimeEffect::EndFrame,
                TargetShape::Frame,
                target_shape(TargetShape::Frame),
            ),
        ];

        for (index, (effect, shape, target)) in cases.into_iter().enumerate() {
            let adapter = RecordingAdapter::new();
            let mut facade = UiAdmissionFacade::new(adapter);
            let request_id = AdapterRequestId(2_000 + index as u64);
            let request = UiRuntimeEffectRequest::new(request_id, effect, target.clone());

            let result = facade.submit_admitted(request);

            assert!(matches!(
                result,
                UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
            ));
            assert_eq!(facade.adapter().requests().len(), 1);

            let forwarded = &facade.adapter().requests()[0];
            assert_eq!(forwarded.request_id, request_id);
            assert_eq!(forwarded.effect_id, effect);
            assert_eq!(forwarded.target, target, "effect={effect:?}, shape={shape:?}");
        }
    }

    #[test]
    fn invalid_matrix_cases_do_not_reach_recording_adapter() {
        let cases = [
            (UiRuntimeEffect::WindowCreate, TargetShape::Frame),
            (UiRuntimeEffect::WindowCreate, TargetShape::DrawBatch),
            (UiRuntimeEffect::WindowClose, TargetShape::Empty),
            (UiRuntimeEffect::WindowClose, TargetShape::Frame),
            (UiRuntimeEffect::WindowClose, TargetShape::DrawBatch),
            (UiRuntimeEffect::PollEvents, TargetShape::Frame),
            (UiRuntimeEffect::PollEvents, TargetShape::DrawBatch),
            (UiRuntimeEffect::PollEvents, TargetShape::FrameWithoutWindow),
            (
                UiRuntimeEffect::PollEvents,
                TargetShape::DrawBatchWithoutFrame,
            ),
            (
                UiRuntimeEffect::PollEvents,
                TargetShape::DrawBatchWithoutWindow,
            ),
            (UiRuntimeEffect::BeginFrame, TargetShape::Empty),
            (UiRuntimeEffect::BeginFrame, TargetShape::Frame),
            (UiRuntimeEffect::BeginFrame, TargetShape::DrawBatch),
            (UiRuntimeEffect::BeginFrame, TargetShape::FrameWithoutWindow),
            (
                UiRuntimeEffect::BeginFrame,
                TargetShape::DrawBatchWithoutFrame,
            ),
            (
                UiRuntimeEffect::BeginFrame,
                TargetShape::DrawBatchWithoutWindow,
            ),
            (UiRuntimeEffect::SubmitDrawCommands, TargetShape::Empty),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::WindowOnly,
            ),
            (UiRuntimeEffect::SubmitDrawCommands, TargetShape::Frame),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::FrameWithoutWindow,
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::DrawBatchWithoutFrame,
            ),
            (
                UiRuntimeEffect::SubmitDrawCommands,
                TargetShape::DrawBatchWithoutWindow,
            ),
            (UiRuntimeEffect::EndFrame, TargetShape::Empty),
            (UiRuntimeEffect::EndFrame, TargetShape::WindowOnly),
            (UiRuntimeEffect::EndFrame, TargetShape::DrawBatch),
            (UiRuntimeEffect::EndFrame, TargetShape::FrameWithoutWindow),
            (UiRuntimeEffect::EndFrame, TargetShape::DrawBatchWithoutFrame),
            (UiRuntimeEffect::EndFrame, TargetShape::DrawBatchWithoutWindow),
        ];

        for (index, (effect, shape)) in cases.into_iter().enumerate() {
            let adapter = RecordingAdapter::new();
            let mut facade = UiAdmissionFacade::new(adapter);

            let result = facade.submit_admitted(request(
                3_000 + index as u64,
                effect,
                target_shape(shape),
            ));

            assert!(matches!(result, UiAdmissionResult::Rejected(_)));
            assert!(
                facade.adapter().requests().is_empty(),
                "invalid shape unexpectedly reached adapter for effect={effect:?}, shape={shape:?}"
            );
        }
    }

    #[test]
    fn window_create_does_not_accept_frame_or_batch_targets() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let frame_result = facade.submit_admitted(request(
            4_000,
            UiRuntimeEffect::WindowCreate,
            target_shape(TargetShape::Frame),
        ));
        assert!(matches!(frame_result, UiAdmissionResult::Rejected(_)));
        assert!(facade.adapter().requests().is_empty());

        let batch_result = facade.submit_admitted(request(
            4_001,
            UiRuntimeEffect::WindowCreate,
            target_shape(TargetShape::DrawBatch),
        ));
        assert!(matches!(batch_result, UiAdmissionResult::Rejected(_)));
        assert!(facade.adapter().requests().is_empty());
    }

    #[test]
    fn poll_events_does_not_accept_frame_or_batch_targets() {
        let adapter = RecordingAdapter::new();
        let mut facade = UiAdmissionFacade::new(adapter);

        let frame_result = facade.submit_admitted(request(
            4_100,
            UiRuntimeEffect::PollEvents,
            target_shape(TargetShape::Frame),
        ));
        assert!(matches!(frame_result, UiAdmissionResult::Rejected(_)));
        assert!(facade.adapter().requests().is_empty());

        let batch_result = facade.submit_admitted(request(
            4_101,
            UiRuntimeEffect::PollEvents,
            target_shape(TargetShape::DrawBatch),
        ));
        assert!(matches!(batch_result, UiAdmissionResult::Rejected(_)));
        assert!(facade.adapter().requests().is_empty());
    }

    #[test]
    fn recording_adapter_facade_smoke_path_records_valid_ui_sequence() {
        // This smoke path checks local façade routing only.
        // It does not assert lifecycle ownership, capability admission,
        // budget accounting, audit admission, or platform execution.
        let mut facade = UiAdmissionFacade::new(RecordingAdapter::new());
        let requests = [
            request(
                5_000,
                UiRuntimeEffect::WindowCreate,
                UiAdapterTarget::default(),
            ),
            request(
                5_001,
                UiRuntimeEffect::PollEvents,
                UiAdapterTarget::window(WindowId(1)),
            ),
            request(
                5_002,
                UiRuntimeEffect::BeginFrame,
                UiAdapterTarget::window(WindowId(1)),
            ),
            request(
                5_003,
                UiRuntimeEffect::SubmitDrawCommands,
                UiAdapterTarget::draw_batch(WindowId(1), FrameId(10), DrawBatchId(100)),
            ),
            request(
                5_004,
                UiRuntimeEffect::EndFrame,
                UiAdapterTarget::frame(WindowId(1), FrameId(10)),
            ),
            request(
                5_005,
                UiRuntimeEffect::WindowClose,
                UiAdapterTarget::window(WindowId(1)),
            ),
        ];

        for request in requests.iter().cloned() {
            let result = facade.submit_admitted(request);
            assert!(matches!(
                result,
                UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
            ));
        }

        let recorded = facade.adapter().requests();
        assert_eq!(recorded.len(), requests.len());

        for (recorded, expected) in recorded.iter().zip(requests.iter()) {
            assert_eq!(
                recorded,
                &UiAdapterRequest::new(
                    expected.request_id,
                    expected.effect_id,
                    expected.target.clone()
                )
            );
        }

        assert_eq!(
            recorded.iter().map(|request| request.effect_id).collect::<alloc::vec::Vec<_>>(),
            alloc::vec![
                UiRuntimeEffect::WindowCreate,
                UiRuntimeEffect::PollEvents,
                UiRuntimeEffect::BeginFrame,
                UiRuntimeEffect::SubmitDrawCommands,
                UiRuntimeEffect::EndFrame,
                UiRuntimeEffect::WindowClose,
            ]
        );
    }

    #[test]
    fn recording_adapter_facade_smoke_path_stops_on_invalid_shape() {
        // This smoke path checks local façade routing only.
        // It does not assert lifecycle ownership, capability admission,
        // budget accounting, audit admission, or platform execution.
        let mut facade = UiAdmissionFacade::new(RecordingAdapter::new());

        let valid_create = request(
            6_000,
            UiRuntimeEffect::WindowCreate,
            UiAdapterTarget::default(),
        );
        let invalid_submit = request(
            6_001,
            UiRuntimeEffect::SubmitDrawCommands,
            UiAdapterTarget::frame(WindowId(1), FrameId(10)),
        );
        let valid_close = request(
            6_002,
            UiRuntimeEffect::WindowClose,
            UiAdapterTarget::window(WindowId(1)),
        );

        let create_result = facade.submit_admitted(valid_create.clone());
        assert!(matches!(
            create_result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        let expected_create = UiAdapterRequest::new(
            valid_create.request_id,
            valid_create.effect_id,
            valid_create.target.clone(),
        );
        assert_eq!(
            facade.adapter().requests(),
            &[expected_create.clone()]
        );

        let invalid_result = facade.submit_admitted(invalid_submit);
        assert!(matches!(
            invalid_result,
            UiAdmissionResult::Rejected(UiAdmissionReject {
                kind: UiAdmissionRejectKind::InvalidTargetForEffect,
                effect_id: UiRuntimeEffect::SubmitDrawCommands
            })
        ));
        assert_eq!(facade.adapter().requests(), &[expected_create.clone()]);

        let close_result = facade.submit_admitted(valid_close.clone());
        assert!(matches!(
            close_result,
            UiAdmissionResult::Submitted(UiAdapterResult::Performed(UiAdapterValue::Unit))
        ));
        let expected_close = UiAdapterRequest::new(
            valid_close.request_id,
            valid_close.effect_id,
            valid_close.target.clone(),
        );
        assert_eq!(
            facade.adapter().requests(),
            &[
                expected_create,
                expected_close,
            ]
        );
    }
}
