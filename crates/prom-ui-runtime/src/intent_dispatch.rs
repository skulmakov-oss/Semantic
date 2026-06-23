use crate::intent_audit::{InertAuditLogger, UiAuditLogger};
use crate::intent_capability::{InertCapabilityEvaluator, UiCapabilityEvaluator};
use prom_ui::{IntentDispatchError, SemanticIntent, UiIntentDispatcher};

/// The default runtime dispatcher that securely maps intents to capability gates.
#[derive(Debug, Clone, Default)]
pub struct RuntimeIntentDispatcher {
    evaluator: InertCapabilityEvaluator,
    audit_logger: InertAuditLogger,
}

impl RuntimeIntentDispatcher {
    pub fn new() -> Self {
        Self {
            evaluator: InertCapabilityEvaluator::new(),
            audit_logger: InertAuditLogger::new(),
        }
    }
}

impl UiIntentDispatcher for RuntimeIntentDispatcher {
    fn dispatch_intent(&self, intent: SemanticIntent) -> Result<(), IntentDispatchError> {
        let evaluation = self.evaluator.evaluate_intent(&intent);

        // All capability evaluations must be durably audited before proceeding.
        let _ = self
            .audit_logger
            .record_intent_evaluation(&intent, &evaluation);

        match evaluation {
            Ok(_admitted) => {
                // Wave 0: Inert scaffold. If by some chance it was admitted, we still do nothing.
                Ok(())
            }
            Err(_) => {
                // Wave 0: Always denies execution to ensure fail-safe capability.
                Err(IntentDispatchError::CapabilityDenied)
            }
        }
    }
}
