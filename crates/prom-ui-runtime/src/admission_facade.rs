//! Local admission façade skeleton for `prom-ui-runtime`.
//!
//! This module validates only the local request shape before mapping it to
//! the adapter boundary. It does not perform real capability enforcement,
//! budget accounting, audit admission, or platform execution.

use crate::adapter_boundary::{
    AdapterRequestId, UiAdapterRequest, UiAdapterResult, UiAdapterTarget, UiRuntimeAdapter,
    UiRuntimeEffect,
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
            return UiAdmissionResult::Rejected(UiAdmissionReject {
                kind: UiAdmissionRejectKind::InvalidTargetForEffect,
                effect_id: request.effect_id,
            });
        }

        let adapter_request = UiAdapterRequest::new(
            request.request_id,
            request.effect_id,
            request.target,
        );

        UiAdmissionResult::Submitted(self.adapter.submit(adapter_request))
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
}
