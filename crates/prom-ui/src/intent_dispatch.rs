use crate::action_mapping::SemanticIntent;

/// Errors that can occur during the dispatch of a semantic intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentDispatchError {
    /// The intent was denied by the runtime capability rules.
    CapabilityDenied,
    /// The requested action is unknown to the dispatcher.
    UnknownAction,
    /// The target node could not be resolved in the current semantic state.
    TargetNotFound,
}

/// Dispatches a mapped `SemanticIntent` into the runtime execution environment.
///
/// This trait acts as the secure switchboard, leaving capability checks to the implementor.
pub trait UiIntentDispatcher {
    /// Attempts to securely dispatch a semantic intent.
    fn dispatch_intent(&self, intent: SemanticIntent) -> Result<(), IntentDispatchError>;
}
