use crate::admitted_action::InteractionAdmittedSemanticAction;

/// Errors that can occur during the dispatch of an admitted semantic action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentDispatchError {
    /// The requested action is unknown to the dispatcher.
    UnknownAction,
    /// The target node could not be resolved in the current semantic state.
    TargetNotFound,
    /// The runtime failed to apply the state update or execute the side effect.
    StateUpdateFailed,
}

/// Dispatches an admitted semantic action into the runtime execution environment.
///
/// This trait assumes the action has already passed capability and lifecycle admission gates.
pub trait UiActionDispatcher {
    /// Dispatches an execution-authorized semantic action.
    fn dispatch_action(
        &self,
        action: InteractionAdmittedSemanticAction,
    ) -> Result<(), IntentDispatchError>;
}
