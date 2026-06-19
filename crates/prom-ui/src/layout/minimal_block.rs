use crate::layout::{UiLayoutGeometryModel, UiRect};
use crate::renderer::{UiRenderModel, UiRenderNodeKind};

/// Minimal block layout solver for deterministic layout verification without Taffy.
///
/// Rules:
/// - Element: width = 100, height = 50
/// - Text: width = 80, height = 20
/// - Everything else (Root, Fragment/Slot): width = 0, height = 0
///
/// Stacks nodes vertically (y = prev.y + prev.height) in the order they appear.
pub fn solve_minimal_blocks(
    geometry_model: &UiLayoutGeometryModel,
    render_model: &UiRenderModel,
) -> UiLayoutGeometryModel {
    let mut solved_nodes = Vec::with_capacity(geometry_model.nodes().len());
    let mut current_y = 0;

    for geometry_node in geometry_model.nodes() {
        let render_node_id = geometry_node.source_render_node();
        let render_node = render_model
            .nodes()
            .iter()
            .find(|n| n.id() == render_node_id)
            .expect("Render node must exist in the provided render model");

        let (width, height) = match render_node.kind() {
            UiRenderNodeKind::Element => (100, 50),
            UiRenderNodeKind::Text => (80, 20),
            UiRenderNodeKind::Root | UiRenderNodeKind::Fragment => (0, 0),
        };

        let rect = UiRect::from_xywh(0, current_y, width, height);

        if height > 0 {
            current_y += height as i32;
        }

        solved_nodes.push(geometry_node.clone().with_rect(rect));
    }

    geometry_model.clone().with_nodes(solved_nodes)
}
