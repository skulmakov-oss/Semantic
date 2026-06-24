#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::{action_binding::InteractionActionBindingId, SemanticIntent, UiProjectedNodeId};
use prom_ui_runtime::intent_capability::IntentCapabilityError;
use prom_ui_runtime::RuntimeIntentAdmission;

#[test]
fn default_runtime_admission_denies_all_execution() {
    let admission = RuntimeIntentAdmission::new();
    let target = UiProjectedNodeId::new(42);
    let intent = SemanticIntent::new(target, InteractionActionBindingId(1));

    let result = admission.admit_intent(intent);

    // Validates that execution is disabled by default for the inert scaffold.
    assert_eq!(result, Err(IntentCapabilityError::MissingCapability));
}
