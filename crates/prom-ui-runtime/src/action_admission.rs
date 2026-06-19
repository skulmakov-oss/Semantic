use prom_ui::action_admission::{ActionAdmissionGuard, ActionAdmissionResult, AdmittedAction};
use prom_ui::action_request::ActionRequestDescriptor;

pub struct RuntimeAdmissionController;

impl RuntimeAdmissionController {
    pub fn new() -> Self {
        Self
    }
}

impl ActionAdmissionGuard for RuntimeAdmissionController {
    fn admit_request(&self, request: ActionRequestDescriptor) -> ActionAdmissionResult {
        ActionAdmissionResult::Admitted(AdmittedAction { request_id: request.id })
    }
}
