use crate::action_binding::InteractionActionBindingId;
use crate::interaction::InteractionIntentKind;
use crate::projection::UiProjectedNodeId;

/// An abstract representation of a mapped user intention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticIntent {
    pub target: UiProjectedNodeId,
    pub action_id: InteractionActionBindingId,
}

impl SemanticIntent {
    pub const fn new(target: UiProjectedNodeId, action_id: InteractionActionBindingId) -> Self {
        Self { target, action_id }
    }
}

/// Purely translates a resolved interaction into an actionable SemanticIntent.
pub trait UiActionMapper {
    fn map_intent(
        &self,
        target: UiProjectedNodeId,
        kind: InteractionIntentKind,
    ) -> Option<SemanticIntent>;
}

/// The default inert semantic action mapper.
#[derive(Debug, Clone, Default)]
pub struct DefaultActionMapper;

impl DefaultActionMapper {
    pub fn new() -> Self {
        Self
    }
}

impl UiActionMapper for DefaultActionMapper {
    fn map_intent(
        &self,
        _target: UiProjectedNodeId,
        _kind: InteractionIntentKind,
    ) -> Option<SemanticIntent> {
        // Wave 0: Inert scaffold. Returns None for now.
        None
    }
}
