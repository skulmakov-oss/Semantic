#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::{
    action_binding::InteractionActionBindingId, IntentDispatchError, SemanticIntent,
    UiIntentDispatcher, UiProjectedNodeId,
};
use prom_ui_runtime::RuntimeIntentDispatcher;

#[test]
fn default_runtime_dispatcher_denies_all_execution() {
    let dispatcher = RuntimeIntentDispatcher::new();
    let target = UiProjectedNodeId::new(42);
    let intent = SemanticIntent::new(target, InteractionActionBindingId(1));

    let result = dispatcher.dispatch_intent(intent);

    // Validates that execution is disabled by default for the inert scaffold.
    assert_eq!(result, Err(IntentDispatchError::CapabilityDenied));
}
