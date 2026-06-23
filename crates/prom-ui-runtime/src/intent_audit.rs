use crate::intent_capability::IntentCapabilityError;
use prom_ui::{InteractionAdmittedSemanticAction, SemanticIntent};

/// Errors that can occur during audit tracing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    /// The audit trace could not be serialized into the required format.
    SerializationFailed,
    /// The sink for the audit trace is unavailable.
    SinkUnavailable,
}

/// Records the admission result of capability evaluations.
pub trait UiAuditLogger {
    /// Records the evaluation result for a given intent.
    fn record_intent_evaluation(
        &self,
        intent: &SemanticIntent,
        evaluation: &Result<InteractionAdmittedSemanticAction, IntentCapabilityError>,
    ) -> Result<(), AuditError>;
}

/// An inert audit logger that statically succeeds without emitting any traces.
///
/// Used as the default logger during Wave 0 scaffold to fulfill the audit
/// contract without implementing the actual trace sink.
#[derive(Debug, Clone, Default)]
pub struct InertAuditLogger;

impl InertAuditLogger {
    pub fn new() -> Self {
        Self
    }
}

impl UiAuditLogger for InertAuditLogger {
    fn record_intent_evaluation(
        &self,
        _intent: &SemanticIntent,
        _evaluation: &Result<InteractionAdmittedSemanticAction, IntentCapabilityError>,
    ) -> Result<(), AuditError> {
        // Wave 0: Inert scaffold. Always effectively "audited" to nothing.
        Ok(())
    }
}
