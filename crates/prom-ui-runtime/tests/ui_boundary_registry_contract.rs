use prom_ui_runtime::boundary_registry::{find_ui_boundary, ui_boundaries};
use std::path::Path;

#[test]
fn ui_boundary_registry_is_not_empty() {
    assert!(!ui_boundaries().is_empty());
}

#[test]
fn ui_boundary_registry_contains_required_h_series_boundaries() {
    let required = [
        "ui.visual_doctrine",
        "ui.visual_token_system",
        "ui.layout_primitives",
        "ui.component_admission",
        "ui.interaction_input",
        "ui.focus_selection",
        "ui.semantic_action",
        "ui.effect_request_capability",
        "ui.trace_audit_visual",
        "ui.error_denial_quarantine_visual",
        "ui.recovery_rollback_visual",
        "ui.renderer_transcript_presentation",
        "ui.workbench_consumption",
        "ui.simulation_snapshot",
        "ui.boundary_index",
        "ui.implementation_gate",
    ];

    for id in required {
        assert!(find_ui_boundary(id).is_some(), "missing UI boundary: {id}");
    }
}

#[test]
fn ui_boundary_registry_paths_are_architecture_docs() {
    for boundary in ui_boundaries() {
        assert!(
            boundary.path.starts_with("docs/architecture/"),
            "boundary {} path must point to docs/architecture/: {}",
            boundary.id,
            boundary.path
        );
        assert!(
            boundary.path.ends_with(".md"),
            "boundary {} path must be markdown: {}",
            boundary.id,
            boundary.path
        );
    }
}

#[test]
fn ui_boundary_registry_ids_are_unique() {
    let boundaries = ui_boundaries();

    for (index, left) in boundaries.iter().enumerate() {
        for right in boundaries.iter().skip(index + 1) {
            assert_ne!(left.id, right.id, "duplicate UI boundary id: {}", left.id);
        }
    }
}

#[test]
fn ui_boundary_registry_docs_exist_on_disk() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = Path::new(manifest_dir)
        .ancestors()
        .nth(2)
        .expect("prom-ui-runtime should be under crates/prom-ui-runtime");

    for boundary in ui_boundaries() {
        let path = repo_root.join(boundary.path);
        assert!(
            path.is_file(),
            "boundary doc does not exist for {}: {}",
            boundary.id,
            path.display()
        );
    }
}
