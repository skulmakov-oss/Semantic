use core::convert::Infallible;

use prom_ui::{UiIrNodeId, UiLayoutRect, UiProjectedNodeId, UiRenderNodeId, UiRenderNodeKind};
use prom_ui_backend_native::{UiBackendFrame, UiBackendFrameEntry, UiFrameSink};

fn sample_entry() -> UiBackendFrameEntry {
    UiBackendFrameEntry::new(
        UiRenderNodeId::new(11),
        UiProjectedNodeId::new(22),
        Some(UiIrNodeId::new(33)),
        UiLayoutRect::new(-4, 8, 120, 48),
        UiRenderNodeKind::Element,
    )
}

type FrameSignatureTuple = (u64, u64, Option<u64>, i32, i32, u32, u32, UiRenderNodeKind);

fn entry_signature(entry: &UiBackendFrameEntry) -> FrameSignatureTuple {
    let rect = entry.rect();
    (
        entry.render_node_id().raw(),
        entry.source_projection_node_id().raw(),
        entry.source_ir_node_id().map(UiIrNodeId::raw),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        entry.kind(),
    )
}

fn frame_signature(frame: &UiBackendFrame) -> Vec<FrameSignatureTuple> {
    frame.entries().iter().map(entry_signature).collect()
}

#[test]
fn backend_frame_entry_preserves_render_layout_source_evidence() {
    let entry = sample_entry();

    assert_eq!(entry.render_node_id(), UiRenderNodeId::new(11));
    assert_eq!(
        entry.source_projection_node_id(),
        UiProjectedNodeId::new(22)
    );
    assert_eq!(entry.source_ir_node_id(), Some(UiIrNodeId::new(33)));
    assert_eq!(entry.rect(), UiLayoutRect::new(-4, 8, 120, 48));
    assert_eq!(entry.kind(), UiRenderNodeKind::Element);
}

#[test]
fn backend_frame_preserves_entry_order() {
    let first = sample_entry();
    let second = UiBackendFrameEntry::new(
        UiRenderNodeId::new(44),
        UiProjectedNodeId::new(55),
        None,
        UiLayoutRect::new(0, 0, 1, 2),
        UiRenderNodeKind::Text,
    );

    let frame = UiBackendFrame::new(vec![first.clone(), second.clone()]);

    assert_eq!(frame.entries(), &[first, second]);
}

#[test]
fn backend_frame_empty_state_is_explicit() {
    let frame = UiBackendFrame::new(Vec::new());

    assert_eq!(frame.len(), 0);
    assert!(frame.is_empty());
    assert!(frame.entries().is_empty());
}

#[test]
fn backend_frame_clone_and_equality_are_stable() {
    let entry = sample_entry();
    let frame = UiBackendFrame::new(vec![entry]);

    assert_eq!(frame.clone(), frame);
    assert_eq!(frame.entries()[0].clone(), frame.entries()[0]);
}

#[derive(Default)]
struct RecordingSink {
    submitted: Vec<UiBackendFrame>,
}

impl UiFrameSink for RecordingSink {
    type Error = Infallible;

    fn submit_frame(&mut self, frame: &UiBackendFrame) -> Result<(), Self::Error> {
        self.submitted.push(frame.clone());
        Ok(())
    }
}

#[test]
fn frame_sink_trait_accepts_inert_frame_evidence() {
    let mut sink = RecordingSink::default();
    let frame = UiBackendFrame::new(vec![sample_entry()]);

    sink.submit_frame(&frame).unwrap();

    assert_eq!(sink.submitted.len(), 1);
    assert_eq!(sink.submitted[0], frame);
}

#[test]
fn frame_sink_submission_does_not_mutate_submitted_frame() {
    let mut sink = RecordingSink::default();
    let frame = UiBackendFrame::new(vec![sample_entry()]);
    let before = frame_signature(&frame);

    sink.submit_frame(&frame).unwrap();

    let after = frame_signature(&frame);
    assert_eq!(before, after);
    assert_eq!(sink.submitted[0], frame);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestSinkError {
    Rejected,
}

struct RejectingSink;

impl UiFrameSink for RejectingSink {
    type Error = TestSinkError;

    fn submit_frame(&mut self, _frame: &UiBackendFrame) -> Result<(), Self::Error> {
        Err(TestSinkError::Rejected)
    }
}

#[test]
fn frame_sink_trait_supports_typed_error_without_execution() {
    let mut sink = RejectingSink;
    let frame = UiBackendFrame::new(vec![sample_entry()]);

    assert_eq!(sink.submit_frame(&frame), Err(TestSinkError::Rejected));
}

#[test]
fn frame_sink_boundary_exposes_no_drawing_window_or_runtime_api() {
    let entry = sample_entry();
    let frame = UiBackendFrame::new(vec![entry.clone()]);

    assert_eq!(frame.len(), 1);
    assert_eq!(frame.entries()[0], entry);
    assert_eq!(frame.entries()[0].kind(), UiRenderNodeKind::Element);
}
