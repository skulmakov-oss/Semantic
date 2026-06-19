use prom_ui::action_dispatch_route::ActionRuntimeDispatcher;
use prom_ui::action_admission::AdmittedAction;
use prom_ui::effect_request::{EffectRuntimeDispatcher, InteractionEffectRequestDescriptor};

pub struct HostActionDispatcher;

impl HostActionDispatcher {
    pub fn new() -> Self {
        Self
    }
}

impl ActionRuntimeDispatcher for HostActionDispatcher {
    fn dispatch_action(&self, action: AdmittedAction) {
        println!("Runtime executing action: {:?}", action.request_id);
    }
}

impl EffectRuntimeDispatcher for HostActionDispatcher {
    fn dispatch_effect(&self, request: InteractionEffectRequestDescriptor) {
        println!("Runtime executing effect: {:?}", request.requested_effect);
    }
}
