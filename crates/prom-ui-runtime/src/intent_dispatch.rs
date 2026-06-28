use crate::state_update::{InertStateUpdater, RuntimeStateUpdater};
use prom_ui::{IntentDispatchError, InteractionAdmittedSemanticAction, UiActionDispatcher};

/// The default runtime dispatcher that securely dispatches execution-authorized actions.
///
/// This dispatcher assumes capability evaluation has already happened in the admission gate.
#[derive(Debug, Clone, Default)]
pub struct RuntimeActionDispatcher {
    state_updater: InertStateUpdater,
}

impl RuntimeActionDispatcher {
    pub fn new() -> Self {
        Self {
            state_updater: InertStateUpdater::new(),
        }
    }
}

impl UiActionDispatcher for RuntimeActionDispatcher {
    fn dispatch_action(
        &self,
        action: InteractionAdmittedSemanticAction,
    ) -> Result<(), IntentDispatchError> {
        self.state_updater
            .execute_admitted_action(action)
            .map_err(|_| IntentDispatchError::StateUpdateFailed)
    }
}
