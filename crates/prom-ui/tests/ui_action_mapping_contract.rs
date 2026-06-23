#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::{DefaultActionMapper, InteractionIntentKind, UiActionMapper, UiProjectedNodeId};

#[test]
fn default_action_mapper_is_inert_returns_none() {
    let target = UiProjectedNodeId::new(42);
    let mapper = DefaultActionMapper::new();

    let intent = mapper.map_intent(target, InteractionIntentKind::Activate);
    assert_eq!(intent, None);

    let intent = mapper.map_intent(target, InteractionIntentKind::Unknown);
    assert_eq!(intent, None);
}
