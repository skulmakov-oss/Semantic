#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::{action_binding::InteractionActionBindingId, SemanticIntent, UiProjectedNodeId};
use prom_ui_runtime::intent_capability::{
    InertCapabilityEvaluator, IntentCapabilityError, UiCapabilityEvaluator,
};

#[test]
fn inert_evaluator_denies_all_intents() {
    let evaluator = InertCapabilityEvaluator::new();
    let target = UiProjectedNodeId::new(42);
    let intent = SemanticIntent::new(target, InteractionActionBindingId(1));

    let result = evaluator.evaluate_intent(&intent);

    // Validates that the capability boundary denies everything by default in the inert scaffold.
    assert_eq!(result, Err(IntentCapabilityError::MissingCapability));
}
