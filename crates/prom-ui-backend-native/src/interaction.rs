use crate::RawBackendEvent;
use prom_ui::UiProjectedNodeId;

/// An inert piece of routing evidence bundling a physical event with its hit-tested semantic target.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutedInteraction {
    pub target: UiProjectedNodeId,
    pub event: RawBackendEvent,
}

impl RoutedInteraction {
    pub const fn new(target: UiProjectedNodeId, event: RawBackendEvent) -> Self {
        Self { target, event }
    }
}
