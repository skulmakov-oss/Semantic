use prom_ui_runtime::component_metadata::{
    find_ui_component, ui_components, UiComponentAdmission, UiComponentDomain, UiComponentKind,
};

#[test]
fn ui_component_registry_is_not_empty() {
    assert!(!ui_components().is_empty());
}

#[test]
fn ui_component_registry_contains_required_metadata_roles() {
    let required = [
        "ui.component.state_badge",
        "ui.component.trace_event_row",
        "ui.component.capability_decision_view",
        "ui.component.effect_request_view",
        "ui.component.denial_reason_overlay",
        "ui.component.quarantine_notice",
        "ui.component.conflict_boundary_view",
        "ui.component.recovery_option_panel",
        "ui.component.renderer_transcript_view",
        "ui.component.workbench_projection_view",
        "ui.component.snapshot_mode_badge",
        "ui.component.simulation_mode_badge",
    ];

    for id in required {
        assert!(
            find_ui_component(id).is_some(),
            "missing component metadata: {id}"
        );
    }
}

#[test]
fn ui_component_ids_are_unique() {
    let components = ui_components();

    for (index, left) in components.iter().enumerate() {
        for right in components.iter().skip(index + 1) {
            assert_ne!(left.id, right.id, "duplicate component id: {}", left.id);
        }
    }
}

#[test]
fn ui_component_ids_use_canonical_prefix() {
    for component in ui_components() {
        assert!(
            component.id.starts_with("ui.component."),
            "component id must use ui.component. prefix: {}",
            component.id
        );
    }
}

#[test]
fn ui_component_domains_are_semantic_not_renderer_types() {
    let trace = find_ui_component("ui.component.trace_event_row").unwrap();
    let capability = find_ui_component("ui.component.capability_decision_view").unwrap();
    let renderer = find_ui_component("ui.component.renderer_transcript_view").unwrap();

    assert_eq!(trace.domain, UiComponentDomain::Trace);
    assert_eq!(capability.domain, UiComponentDomain::Capability);
    assert_eq!(renderer.domain, UiComponentDomain::Renderer);
}

#[test]
fn workbench_component_is_marked_workbench_local() {
    let component = find_ui_component("ui.component.workbench_projection_view").unwrap();

    assert_eq!(component.admission, UiComponentAdmission::WorkbenchLocal);
}

#[test]
fn error_denial_quarantine_components_are_distinct_roles() {
    let denial = find_ui_component("ui.component.denial_reason_overlay").unwrap();
    let failure = find_ui_component("ui.component.failure_stage_view").unwrap();
    let quarantine = find_ui_component("ui.component.quarantine_notice").unwrap();

    assert_eq!(denial.kind, UiComponentKind::DenialReasonOverlay);
    assert_eq!(failure.kind, UiComponentKind::FailureStageView);
    assert_eq!(quarantine.kind, UiComponentKind::QuarantineNotice);

    assert_ne!(denial.kind, failure.kind);
    assert_ne!(failure.kind, quarantine.kind);
    assert_ne!(denial.kind, quarantine.kind);
}
