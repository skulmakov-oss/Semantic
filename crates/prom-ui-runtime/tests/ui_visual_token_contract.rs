use prom_ui_runtime::visual_tokens::{find_ui_visual_token, ui_visual_tokens, UiVisualTokenRole};

#[test]
fn ui_visual_token_registry_is_not_empty() {
    assert!(!ui_visual_tokens().is_empty());
}

#[test]
fn ui_visual_token_registry_contains_required_semantic_roles() {
    let required = [
        "ui.token.surface.base",
        "ui.token.surface.raised",
        "ui.token.text.primary",
        "ui.token.state.denied",
        "ui.token.state.failed",
        "ui.token.state.quarantined",
        "ui.token.state.stale",
        "ui.token.state.preview",
        "ui.token.state.simulation",
        "ui.token.trace.audit",
        "ui.token.capability.denied",
        "ui.token.effect.committed",
        "ui.token.renderer.staged",
        "ui.token.renderer.presented",
        "ui.token.workbench.projection",
    ];

    for id in required {
        assert!(find_ui_visual_token(id).is_some(), "missing visual token: {id}");
    }
}

#[test]
fn ui_visual_token_ids_are_unique() {
    let tokens = ui_visual_tokens();

    for (index, left) in tokens.iter().enumerate() {
        for right in tokens.iter().skip(index + 1) {
            assert_ne!(left.id, right.id, "duplicate visual token id: {}", left.id);
        }
    }
}

#[test]
fn ui_visual_token_ids_use_canonical_prefix() {
    for token in ui_visual_tokens() {
        assert!(
            token.id.starts_with("ui.token."),
            "visual token id must use ui.token. prefix: {}",
            token.id
        );
    }
}

#[test]
fn ui_visual_state_tokens_are_distinct_semantic_roles() {
    let denied = find_ui_visual_token("ui.token.state.denied").unwrap();
    let failed = find_ui_visual_token("ui.token.state.failed").unwrap();
    let quarantined = find_ui_visual_token("ui.token.state.quarantined").unwrap();

    assert_eq!(denied.role, UiVisualTokenRole::StateDenied);
    assert_eq!(failed.role, UiVisualTokenRole::StateFailed);
    assert_eq!(quarantined.role, UiVisualTokenRole::StateQuarantined);

    assert_ne!(denied.role, failed.role);
    assert_ne!(failed.role, quarantined.role);
    assert_ne!(denied.role, quarantined.role);
}
