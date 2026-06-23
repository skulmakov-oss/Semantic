use prom_ui::{IntentDispatchError, SemanticIntent, UiIntentDispatcher};

/// The default runtime dispatcher that securely maps intents to capability gates.
#[derive(Debug, Clone, Default)]
pub struct RuntimeIntentDispatcher;

impl RuntimeIntentDispatcher {
    pub fn new() -> Self {
        Self
    }
}

impl UiIntentDispatcher for RuntimeIntentDispatcher {
    fn dispatch_intent(&self, _intent: SemanticIntent) -> Result<(), IntentDispatchError> {
        // Wave 0: Inert scaffold. Always denies execution to ensure fail-safe capability.
        Err(IntentDispatchError::CapabilityDenied)
    }
}
