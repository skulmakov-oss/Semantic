use crate::interaction::RoutedInteraction;
use prom_ui::action_binding::InteractionActionName;
use prom_ui::model::UiIrNodeId;

/// Represents an abstract user intention targeting a specific semantic node.
pub struct SemanticIntent {
    pub target_node: UiIrNodeId,
    pub action_name: InteractionActionName,
}

/// A trait for translating physical events on semantic nodes into semantic intents.
pub trait UiActionMapper<E> {
    /// Purely translates a RoutedInteraction into an actionable SemanticIntent, if a binding exists.
    fn map_interaction(&self, interaction: RoutedInteraction<E>) -> Option<SemanticIntent>;
}

/// A basic implementation of UiActionMapper.
pub struct DefaultActionMapper;

impl<E> UiActionMapper<E> for DefaultActionMapper {
    fn map_interaction(&self, _interaction: RoutedInteraction<E>) -> Option<SemanticIntent> {
        // Without a dynamic intent registry, we cannot map generic physical events.
        // This acts as a placeholder that adheres strictly to the boundary logic:
        // no implicit execution and no hardcoded physical dependencies.
        None
    }
}
