use prom_ui::layout::geometry::UiLayoutGeometryModel;
use prom_ui::model::UiIrNodeId;

pub trait UiHitTester {
    fn find_target_at(&self, layout: &UiLayoutGeometryModel, x: f64, y: f64) -> Option<UiIrNodeId>;
}

pub struct DefaultHitTester;

impl UiHitTester for DefaultHitTester {
    fn find_target_at(&self, layout: &UiLayoutGeometryModel, x: f64, y: f64) -> Option<UiIrNodeId> {
        let mut best_match: Option<(UiIrNodeId, usize)> = None;
        
        // Simple conversion to physical coordinates
        let px = x.round() as i32;
        let py = y.round() as i32;

        for node in layout.nodes() {
            let rect = node.rect();
            let is_inside = px >= rect.x()
                && px < rect.x() + rect.width() as i32
                && py >= rect.y()
                && py < rect.y() + rect.height() as i32;

            if is_inside {
                if let Some(ir_node_id) = node.source_ir_node() {
                    let order = node.order();
                    if let Some((_, best_order)) = best_match {
                        if order > best_order {
                            best_match = Some((ir_node_id, order));
                        }
                    } else {
                        best_match = Some((ir_node_id, order));
                    }
                }
            }
        }

        best_match.map(|(id, _)| id)
    }
}

pub struct RoutedInteraction<E> {
    pub target: UiIrNodeId,
    pub event: E,
}
