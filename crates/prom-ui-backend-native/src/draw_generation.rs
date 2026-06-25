use prom_ui::layout::UiLayoutPhysicalPlacementModel;
use prom_ui_runtime::{Color, DrawFrame, Rect};

/// Map a physical layout placement model into a renderer draw frame.
pub fn generate_draw_frame(placement_model: &UiLayoutPhysicalPlacementModel) -> DrawFrame {
    let mut frame = DrawFrame::new();
    
    // Default background clear
    frame.clear(Color::rgb(30, 30, 35));

    for entry in placement_model.entries() {
        let rect = entry.final_rect();
        
        let x = rect.x();
        let y = rect.y();
        let width = rect.width();
        let height = rect.height();
        
        // Skip zero size
        if width == 0 || height == 0 {
            continue;
        }

        // Just use a default color for now since style isn't fully propagated yet
        let color = Color::rgb(70, 130, 180);

        frame.fill_rect(Rect::new(x, y, width, height), color);
    }
    
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use prom_ui::layout::constraint_solver::build_layout_constraint_solver;
    use prom_ui::layout::constraints::build_layout_constraints;
    use prom_ui::layout::geometry::build_layout_geometry;
    use prom_ui::layout::measuring::build_layout_measuring;
    use prom_ui::layout::physical_placement::build_layout_physical_placement;
    use prom_ui::layout::size_to_fit::build_layout_size_to_fit;
    use prom_ui::layout::sizing::build_layout_sizing;
    use prom_ui::layout::sizing_algorithm::build_layout_sizing_algorithm;
    use prom_ui::layout::solving::{build_layout_solving, build_layout_solving_result};
    use prom_ui::layout::UiLayoutModel;

    #[test]
    fn test_generate_draw_frame_from_placement() {
        use prom_ui::projection::{UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact, UiProjectionArtifactId};
        use prom_ui::model::UiIrNodeId;
        
        let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
        artifact.set_source_ir_root(UiIrNodeId::new(10));
        artifact.push_node(UiProjectedNode::with_source_ir_node(
            UiProjectedNodeId::new(1),
            UiProjectedNodeKind::Root,
            UiIrNodeId::new(11),
        ));
        let render_model = prom_ui::render_projection_to_model(&artifact).unwrap();
        let layout_model = prom_ui::layout::layout_render_model(&render_model);
        
        let _geometry = build_layout_geometry(&layout_model);
        let _constraints = build_layout_constraints(&layout_model);
        let sizing = build_layout_sizing(&layout_model);
        let sizing_algo = build_layout_sizing_algorithm(&sizing);
        let measuring = build_layout_measuring(&sizing_algo);
        let size_to_fit = build_layout_size_to_fit(&measuring);
        let constraint_solver = build_layout_constraint_solver(&size_to_fit);
        let solving = build_layout_solving(&constraint_solver);
        let solving_result = build_layout_solving_result(&solving);
        let placement = build_layout_physical_placement(&solving_result);
        
        let frame = generate_draw_frame(&placement);
        
        // It should have at least the Clear command
        assert!(!frame.is_empty());
    }
}
