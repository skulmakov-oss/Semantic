use crate::RawBackendEvent;
use prom_ui::UiProjectedNodeId;
use prom_ui_runtime::interaction_pipeline::EventCoordinates;

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

impl EventCoordinates for RawBackendEvent {
    fn hit_test_coords(&self) -> Option<(f64, f64)> {
        match self {
            Self::PointerMoved { x, y } => Some((*x, *y)),
            _ => None,
        }
    }
}
