#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::{action_binding::InteractionActionBindingId, SemanticIntent, UiProjectedNodeId};
use prom_ui_runtime::intent_audit::{InertAuditLogger, UiAuditLogger};
use prom_ui_runtime::intent_capability::IntentCapabilityError;

#[test]
fn inert_audit_logger_handles_capability_denial() {
    let logger = InertAuditLogger::new();
    let target = UiProjectedNodeId::new(42);
    let intent = SemanticIntent::new(target, InteractionActionBindingId(1));
    let evaluation = Err(IntentCapabilityError::MissingCapability);

    let result = logger.record_intent_evaluation(&intent, &evaluation);

    // Validates that the inert audit logger successfully logs a denied evaluation without error.
    assert_eq!(result, Ok(()));
}
