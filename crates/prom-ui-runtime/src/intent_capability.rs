use prom_ui::{InteractionAdmittedSemanticAction, SemanticIntent};

/// Errors that can occur during UI capability evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentCapabilityError {
    /// The runtime does not grant the required capability for the intent.
    MissingCapability,
    /// The requested action cannot be mapped to a known capability requirement.
    UnknownAction,
}

/// Evaluates if a given UI intent is authorized to execute.
pub trait UiCapabilityEvaluator {
    /// Evaluates the intent against current capabilities and returns an unforgeable
    /// `InteractionAdmittedSemanticAction` if authorized, or a capability error otherwise.
    fn evaluate_intent(
        &self,
        intent: &SemanticIntent,
    ) -> Result<InteractionAdmittedSemanticAction, IntentCapabilityError>;
}

/// An inert capability evaluator that statically denies all intents.
///
/// Used as the default evaluator during Wave 0 scaffold to ensure
/// the application runs securely without enabling mutations.
#[derive(Debug, Clone, Default)]
pub struct InertCapabilityEvaluator;

impl InertCapabilityEvaluator {
    pub fn new() -> Self {
        Self
    }
}

impl UiCapabilityEvaluator for InertCapabilityEvaluator {
    fn evaluate_intent(
        &self,
        _intent: &SemanticIntent,
    ) -> Result<InteractionAdmittedSemanticAction, IntentCapabilityError> {
        // Wave 0: Inert scaffold. Always denies capability.
        Err(IntentCapabilityError::MissingCapability)
    }
}
