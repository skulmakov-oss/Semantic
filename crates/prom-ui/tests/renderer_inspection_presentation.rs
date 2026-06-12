use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId, UiProjectionTraceRef,
};
use prom_ui::renderer::{
    present_render_diagnostics, present_render_inspection, present_render_markers,
    present_render_trace, render_projection_to_model, UiRenderDiagnosticsPresentation,
    UiRenderInspectionItem, UiRenderInspectionItemId, UiRenderInspectionItemKind,
    UiRenderInspectionPresentation, UiRenderInspectionPresentationId, UiRenderInspectionSection,
    UiRenderInspectionSectionId, UiRenderInspectionSectionKind, UiRenderMarkerPresentation,
    UiRenderModel, UiRenderModelId, UiRenderNodeId, UiRenderTracePresentation,
};

fn create_test_renderer_presentation_bundle() -> (
    UiRenderModel,
    UiRenderDiagnosticsPresentation,
    UiRenderTracePresentation,
    UiRenderMarkerPresentation,
) {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(42));
    artifact.set_source_ir_root(UiIrNodeId::new(10));

    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
    ));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::PropertyCarrier,
    ));
    artifact.push_node(UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(3),
        UiProjectedNodeKind::ActionCarrier,
        UiIrNodeId::new(12),
    ));
    let mut traced_node =
        UiProjectedNode::new(UiProjectedNodeId::new(4), UiProjectedNodeKind::Element);
    traced_node.set_trace(UiProjectionTraceRef::new(400));
    artifact.push_node(traced_node);
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(5),
        UiProjectedNodeKind::EffectBoundaryMarker,
    ));

    let model = render_projection_to_model(&artifact).expect("should build render model");
    let diagnostics = present_render_diagnostics(&model);
    let trace = present_render_trace(&model);
    let markers = present_render_markers(&model);

    (model, diagnostics, trace, markers)
}

#[test]
fn inspection_presentation_builds_from_existing_presentation_models() {
    let (model, diagnostics, trace, markers) = create_test_renderer_presentation_bundle();
    let model_before = model.clone();
    let diagnostics_before = diagnostics.clone();
    let trace_before = trace.clone();
    let markers_before = markers.clone();

    let presentation = present_render_inspection(&model, &diagnostics, &trace, &markers);

    assert_eq!(presentation.source_render_model(), model.id());
    assert_eq!(presentation.source_projection(), model.source_projection());
    assert_eq!(model, model_before);
    assert_eq!(diagnostics, diagnostics_before);
    assert_eq!(trace, trace_before);
    assert_eq!(markers, markers_before);

    let sections = presentation.sections();
    assert_eq!(sections.len(), 4);
    assert_eq!(sections[0].kind(), UiRenderInspectionSectionKind::Overview);
    assert_eq!(
        sections[1].kind(),
        UiRenderInspectionSectionKind::Diagnostics
    );
    assert_eq!(sections[2].kind(), UiRenderInspectionSectionKind::Trace);
    assert_eq!(sections[3].kind(), UiRenderInspectionSectionKind::Markers);

    assert_eq!(sections[0].items().len(), 1);
    assert_eq!(
        sections[0].items()[0].kind(),
        UiRenderInspectionItemKind::OverviewSummary
    );
    assert_eq!(
        sections[0].items()[0].source_ir_node(),
        Some(UiIrNodeId::new(10))
    );

    assert_eq!(sections[1].items().len(), 4);
    assert!(sections[1]
        .items()
        .iter()
        .all(|item| item.kind() == UiRenderInspectionItemKind::DiagnosticItem));
    assert_eq!(
        sections[1].items()[0].source_render_node().unwrap().raw(),
        2
    );
    assert_eq!(
        sections[1].items()[1].source_render_node().unwrap().raw(),
        3
    );
    assert_eq!(
        sections[1].items()[1].source_ir_node(),
        Some(UiIrNodeId::new(12))
    );
    assert_eq!(
        sections[1].items()[2].source_render_node().unwrap().raw(),
        4
    );
    assert_eq!(
        sections[1].items()[3].source_render_node().unwrap().raw(),
        5
    );

    assert!(!sections[2].items().is_empty());
    assert!(sections[2]
        .items()
        .iter()
        .all(|item| item.kind() == UiRenderInspectionItemKind::TraceLink));

    assert_eq!(sections[3].items().len(), 4);
    assert!(sections[3]
        .items()
        .iter()
        .all(|item| item.kind() == UiRenderInspectionItemKind::MarkerItem));
}

#[test]
fn inspection_presentation_deterministic_presentation_id() {
    let (model, diagnostics, trace, markers) = create_test_renderer_presentation_bundle();

    let presentation1 = present_render_inspection(&model, &diagnostics, &trace, &markers);
    let presentation2 = present_render_inspection(&model, &diagnostics, &trace, &markers);

    assert_eq!(presentation1.id(), presentation2.id());
}

#[test]
fn inspection_presentation_deterministic_section_ids_and_order() {
    let (model, diagnostics, trace, markers) = create_test_renderer_presentation_bundle();

    let presentation1 = present_render_inspection(&model, &diagnostics, &trace, &markers);
    let presentation2 = present_render_inspection(&model, &diagnostics, &trace, &markers);

    let sections1 = presentation1.sections();
    let sections2 = presentation2.sections();

    assert_eq!(sections1.len(), sections2.len());
    assert_eq!(
        sections1
            .iter()
            .map(UiRenderInspectionSection::kind)
            .collect::<Vec<_>>(),
        vec![
            UiRenderInspectionSectionKind::Overview,
            UiRenderInspectionSectionKind::Diagnostics,
            UiRenderInspectionSectionKind::Trace,
            UiRenderInspectionSectionKind::Markers,
        ]
    );
    assert_eq!(
        sections1
            .iter()
            .map(UiRenderInspectionSection::id)
            .collect::<Vec<_>>(),
        sections2
            .iter()
            .map(UiRenderInspectionSection::id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn inspection_presentation_deterministic_item_ids() {
    let (model, diagnostics, trace, markers) = create_test_renderer_presentation_bundle();

    let presentation1 = present_render_inspection(&model, &diagnostics, &trace, &markers);
    let presentation2 = present_render_inspection(&model, &diagnostics, &trace, &markers);

    for (section1, section2) in presentation1
        .sections()
        .iter()
        .zip(presentation2.sections().iter())
    {
        assert_eq!(section1.id(), section2.id());
        assert_eq!(section1.kind(), section2.kind());
        assert_eq!(
            section1
                .items()
                .iter()
                .map(UiRenderInspectionItem::id)
                .collect::<Vec<_>>(),
            section2
                .items()
                .iter()
                .map(UiRenderInspectionItem::id)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn inspection_presentation_is_read_only_over_inputs() {
    let (model, diagnostics, trace, markers) = create_test_renderer_presentation_bundle();
    let model_before = model.clone();
    let diagnostics_before = diagnostics.clone();
    let trace_before = trace.clone();
    let markers_before = markers.clone();

    let _presentation = present_render_inspection(&model, &diagnostics, &trace, &markers);

    assert_eq!(model, model_before);
    assert_eq!(diagnostics, diagnostics_before);
    assert_eq!(trace, trace_before);
    assert_eq!(markers, markers_before);
}

#[test]
fn inspection_presentation_section_kind_coverage_is_locked() {
    let (model, diagnostics, trace, markers) = create_test_renderer_presentation_bundle();
    let presentation = present_render_inspection(&model, &diagnostics, &trace, &markers);

    let kinds = presentation
        .sections()
        .iter()
        .map(UiRenderInspectionSection::kind)
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        vec![
            UiRenderInspectionSectionKind::Overview,
            UiRenderInspectionSectionKind::Diagnostics,
            UiRenderInspectionSectionKind::Trace,
            UiRenderInspectionSectionKind::Markers,
        ]
    );
}

#[test]
fn inspection_presentation_item_kind_coverage_is_locked() {
    let (model, diagnostics, trace, markers) = create_test_renderer_presentation_bundle();
    let presentation = present_render_inspection(&model, &diagnostics, &trace, &markers);

    assert_eq!(
        presentation.sections()[0].items()[0].kind(),
        UiRenderInspectionItemKind::OverviewSummary
    );
    assert!(presentation.sections()[1]
        .items()
        .iter()
        .all(|item| item.kind() == UiRenderInspectionItemKind::DiagnosticItem));
    assert!(presentation.sections()[2]
        .items()
        .iter()
        .all(|item| item.kind() == UiRenderInspectionItemKind::TraceLink));
    assert!(presentation.sections()[3]
        .items()
        .iter()
        .all(|item| item.kind() == UiRenderInspectionItemKind::MarkerItem));
}

#[test]
fn inspection_presentation_entrypoint_signature_is_locked() {
    let _: fn(
        &UiRenderModel,
        &UiRenderDiagnosticsPresentation,
        &UiRenderTracePresentation,
        &UiRenderMarkerPresentation,
    ) -> UiRenderInspectionPresentation = present_render_inspection;
}

#[test]
fn inspection_presentation_public_types_are_locked() {
    fn assert_presentation(_: &UiRenderInspectionPresentation) {}
    fn assert_section(_: &UiRenderInspectionSection) {}
    fn assert_item(_: &UiRenderInspectionItem) {}
    fn assert_section_kind(_: UiRenderInspectionSectionKind) {}
    fn assert_item_kind(_: UiRenderInspectionItemKind) {}

    let _ = (
        assert_presentation,
        assert_section,
        assert_item,
        assert_section_kind,
        assert_item_kind,
    );
}

#[test]
fn inspection_presentation_accessor_surface_is_locked() {
    let _: fn(&UiRenderInspectionPresentation) -> UiRenderInspectionPresentationId =
        UiRenderInspectionPresentation::id;
    let _: fn(&UiRenderInspectionPresentation) -> UiRenderModelId =
        UiRenderInspectionPresentation::source_render_model;
    let _: fn(&UiRenderInspectionPresentation) -> UiProjectionArtifactId =
        UiRenderInspectionPresentation::source_projection;
    let _: fn(&UiRenderInspectionPresentation) -> &[UiRenderInspectionSection] =
        UiRenderInspectionPresentation::sections;
}

#[test]
fn inspection_section_accessor_surface_is_locked() {
    let _: fn(&UiRenderInspectionSection) -> UiRenderInspectionSectionId =
        UiRenderInspectionSection::id;
    let _: fn(&UiRenderInspectionSection) -> UiRenderInspectionSectionKind =
        UiRenderInspectionSection::kind;
    let _: fn(&UiRenderInspectionSection) -> &[UiRenderInspectionItem] =
        UiRenderInspectionSection::items;
}

#[test]
fn inspection_item_accessor_surface_is_locked() {
    let _: fn(&UiRenderInspectionItem) -> UiRenderInspectionItemId = UiRenderInspectionItem::id;
    let _: fn(&UiRenderInspectionItem) -> UiRenderInspectionItemKind = UiRenderInspectionItem::kind;
    let _: fn(&UiRenderInspectionItem) -> Option<UiRenderNodeId> =
        UiRenderInspectionItem::source_render_node;
    let _: fn(&UiRenderInspectionItem) -> Option<UiProjectedNodeId> =
        UiRenderInspectionItem::source_projection_node;
    let _: fn(&UiRenderInspectionItem) -> Option<UiIrNodeId> =
        UiRenderInspectionItem::source_ir_node;
}
