use crate::layout_rect::{UiLayoutRect, UiLayoutRectModel};
use crate::projection::UiProjectedNodeId;

/// Architectural trait for mapping physical coordinates to semantic targets via the layout projection.
pub trait UiHitTester {
    /// Find the highest Z-order node that intersects the given physical coordinates.
    fn find_target_at(
        &self,
        layout: &UiLayoutRectModel,
        x: f64,
        y: f64,
    ) -> Option<UiProjectedNodeId>;
}

/// The default hit-testing strategy for Semantic UI.
#[derive(Debug, Clone, Default)]
pub struct DefaultHitTester;

impl DefaultHitTester {
    pub fn new() -> Self {
        Self
    }

    fn rect_contains(rect: &UiLayoutRect, x: f64, y: f64) -> bool {
        let rx = rect.x() as f64;
        let ry = rect.y() as f64;
        let rw = rect.width() as f64;
        let rh = rect.height() as f64;

        x >= rx && x < rx + rw && y >= ry && y < ry + rh
    }
}

impl UiHitTester for DefaultHitTester {
    fn find_target_at(
        &self,
        layout: &UiLayoutRectModel,
        x: f64,
        y: f64,
    ) -> Option<UiProjectedNodeId> {
        // Iterate in reverse because later entries in the layout model are drawn on top (higher Z-order)
        for entry in layout.entries().iter().rev() {
            if Self::rect_contains(&entry.rect(), x, y) {
                return Some(entry.source_projection_node_id());
            }
        }
        None
    }
}
