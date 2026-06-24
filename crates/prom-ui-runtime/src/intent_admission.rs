use crate::intent_audit::{InertAuditLogger, UiAuditLogger};
use crate::intent_capability::{
    InertCapabilityEvaluator, IntentCapabilityError, UiCapabilityEvaluator,
};
use prom_ui::{InteractionAdmittedSemanticAction, SemanticIntent};

/// Evaluates and audits a semantic intent, returning an admitted action if authorized.
///
/// The intent admission guard acts as the strict boundary between the inert
/// intention (`SemanticIntent`) and the execution authority (`InteractionAdmittedSemanticAction`).
#[derive(Debug, Clone, Default)]
pub struct RuntimeIntentAdmission {
    evaluator: InertCapabilityEvaluator,
    audit_logger: InertAuditLogger,
}

impl RuntimeIntentAdmission {
    pub fn new() -> Self {
        Self {
            evaluator: InertCapabilityEvaluator::new(),
            audit_logger: InertAuditLogger::new(),
        }
    }

    /// Evaluates the semantic intent against capability gates.
    ///
    /// The result is durably audited before it is returned. Only admitted
    /// actions may proceed to the dispatcher.
    pub fn admit_intent(
        &self,
        intent: SemanticIntent,
    ) -> Result<InteractionAdmittedSemanticAction, IntentCapabilityError> {
        let evaluation = self.evaluator.evaluate_intent(&intent);

        // All capability evaluations must be durably audited before proceeding.
        let _ = self
            .audit_logger
            .record_intent_evaluation(&intent, &evaluation);

        evaluation
    }
}
