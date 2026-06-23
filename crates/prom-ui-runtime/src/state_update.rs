use prom_ui::InteractionAdmittedSemanticAction;

/// Errors that can occur during the execution of a state update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateUpdateError {
    /// The execution layer or host ABI is unavailable to process the state update.
    ExecutionUnavailable,
}

/// Consumes an admitted UI intent and executes the required state mutations or host side-effects.
pub trait RuntimeStateUpdater {
    /// Executes the state mutation representing the admitted intent.
    fn execute_admitted_action(
        &self,
        action: InteractionAdmittedSemanticAction,
    ) -> Result<(), StateUpdateError>;
}

/// An inert state updater that successfully does nothing.
///
/// Used as the default updater during Wave 0 scaffold to complete the "Staircase"
/// intent pipeline without implementing actual mutations.
#[derive(Debug, Clone, Default)]
pub struct InertStateUpdater;

impl InertStateUpdater {
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeStateUpdater for InertStateUpdater {
    fn execute_admitted_action(
        &self,
        _action: InteractionAdmittedSemanticAction,
    ) -> Result<(), StateUpdateError> {
        // Wave 0: Inert scaffold. The pipeline is fully completed and we safely do nothing.
        Ok(())
    }
}
