use prom_ui_runtime::layout_primitives::{
    find_ui_layout_primitive, ui_layout_primitives, UiLayoutPrimitiveDomain,
    UiLayoutPrimitiveKind,
};

#[test]
fn ui_layout_primitive_registry_is_not_empty() {
    assert!(!ui_layout_primitives().is_empty());
}

#[test]
fn ui_layout_primitive_registry_contains_required_roles() {
    let required = [
        "ui.layout.root",
        "ui.layout.pane",
        "ui.layout.panel",
        "ui.layout.lane",
        "ui.layout.region",
        "ui.layout.overlay",
        "ui.layout.inspector_pane",
        "ui.layout.trace_lane",
        "ui.layout.effect_lane",
        "ui.layout.capability_gate_region",
        "ui.layout.conflict_boundary",
        "ui.layout.quarantine_region",
        "ui.layout.system_map",
        "ui.layout.workbench_surface",
    ];

    for id in required {
        assert!(
            find_ui_layout_primitive(id).is_some(),
            "missing layout primitive: {id}"
        );
    }
}

#[test]
fn ui_layout_primitive_ids_are_unique() {
    let primitives = ui_layout_primitives();

    for (index, left) in primitives.iter().enumerate() {
        for right in primitives.iter().skip(index + 1) {
            assert_ne!(
                left.id, right.id,
                "duplicate layout primitive id: {}",
                left.id
            );
        }
    }
}

#[test]
fn ui_layout_primitive_ids_use_canonical_prefix() {
    for primitive in ui_layout_primitives() {
        assert!(
            primitive.id.starts_with("ui.layout."),
            "layout primitive id must use ui.layout. prefix: {}",
            primitive.id
        );
    }
}

#[test]
fn ui_layout_boundary_primitives_are_distinct_semantic_roles() {
    let conflict = find_ui_layout_primitive("ui.layout.conflict_boundary").unwrap();
    let quarantine = find_ui_layout_primitive("ui.layout.quarantine_region").unwrap();
    let capability = find_ui_layout_primitive("ui.layout.capability_gate_region").unwrap();

    assert_eq!(conflict.kind, UiLayoutPrimitiveKind::ConflictBoundary);
    assert_eq!(quarantine.kind, UiLayoutPrimitiveKind::QuarantineRegion);
    assert_eq!(capability.kind, UiLayoutPrimitiveKind::CapabilityGateRegion);

    assert_ne!(conflict.kind, quarantine.kind);
    assert_ne!(quarantine.kind, capability.kind);
    assert_ne!(conflict.kind, capability.kind);
}

#[test]
fn ui_layout_primitives_are_spatial_roles_not_components() {
    let trace_lane = find_ui_layout_primitive("ui.layout.trace_lane").unwrap();
    let inspector = find_ui_layout_primitive("ui.layout.inspector_pane").unwrap();
    let system_map = find_ui_layout_primitive("ui.layout.system_map").unwrap();

    assert_eq!(trace_lane.domain, UiLayoutPrimitiveDomain::Trace);
    assert_eq!(inspector.domain, UiLayoutPrimitiveDomain::Inspector);
    assert_eq!(system_map.domain, UiLayoutPrimitiveDomain::Map);
}
