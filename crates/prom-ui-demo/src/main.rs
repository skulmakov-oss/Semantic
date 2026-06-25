use prom_ui::layout::{
    constraint_solver::build_layout_constraint_solver,
    constraints::build_layout_constraints,
    geometry::build_layout_geometry,
    measuring::build_layout_measuring,
    physical_placement::{build_layout_physical_placement, UiLayoutPhysicalPlacementModel},
    size_to_fit::build_layout_size_to_fit,
    sizing::build_layout_sizing,
    sizing_algorithm::build_layout_sizing_algorithm,
    solving::{build_layout_solving, build_layout_solving_result},
};
use prom_ui_backend_native::draw_generation::generate_draw_frame;
use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::{
    Color, DesktopSession, DrawFrame, EventBuffer, InputEventKind, LoopControl, Rect, SessionState,
    WindowConfig,
};

struct DemoState {
    pointer_x: i32,
    pointer_y: i32,
    is_pointer_down: bool,
    key_press_count: u32,
    last_key: Option<u32>,
}

fn build_static_placement() -> UiLayoutPhysicalPlacementModel {
    use prom_ui::model::UiIrNodeId;
    use prom_ui::projection::{
        UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
        UiProjectionArtifactId,
    };

    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(100));
    artifact.set_source_ir_root(UiIrNodeId::new(10));
    artifact.push_node(UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(11),
    ));
    artifact.push_node(UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(2),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(12),
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
    build_layout_physical_placement(&solving_result)
}

fn main() {
    println!("=== Semantic UI Application Boundary — Native Render Demo ===");

    let config = WindowConfig::new("Semantic UI Demo - WGPU Native", 800, 600);
    let backend = NativeBackend::new();

    let mut session =
        DesktopSession::create(backend, config).expect("NativeBackend::create_window must succeed");
    assert_eq!(session.state(), SessionState::Created);
    println!("Session state: {:?}", session.state());

    let placement = build_static_placement();

    // Print physical placement to console initially
    for entry in placement.entries() {
        println!(
            "Placed {:?}: x={}, y={}, w={}, h={}",
            entry.source_render_node(),
            entry.final_rect().x(),
            entry.final_rect().y(),
            entry.final_rect().width(),
            entry.final_rect().height()
        );
    }

    let mut demo_state = DemoState {
        pointer_x: 0,
        pointer_y: 0,
        is_pointer_down: false,
        key_press_count: 0,
        last_key: None,
    };

    session
        .run(move |buf: &mut EventBuffer, out_frame: &mut DrawFrame| {
            // Drain any pending events
            let events = buf.drain();
            for evt in &events {
                match evt.kind {
                    InputEventKind::CloseRequested => {
                        println!("  Event: CloseRequested -> Exiting loop");
                        return LoopControl::ExitRequested;
                    }
                    InputEventKind::PointerMoved { x, y } => {
                        demo_state.pointer_x = x as i32;
                        demo_state.pointer_y = y as i32;
                    }
                    InputEventKind::PointerDown { .. } => {
                        demo_state.is_pointer_down = true;
                    }
                    InputEventKind::PointerUp { .. } => {
                        demo_state.is_pointer_down = false;
                    }
                    InputEventKind::KeyDown { key_code } => {
                        demo_state.key_press_count += 1;
                        demo_state.last_key = Some(key_code);
                    }
                    _ => {}
                }
            }

            // Generate frame commands from the static placement
            *out_frame = generate_draw_frame(&placement);

            // Add input feedback overlays
            let pointer_color = if demo_state.is_pointer_down {
                Color::RED
            } else {
                Color::GREEN
            };

            out_frame.fill_rect(
                Rect::new(demo_state.pointer_x - 5, demo_state.pointer_y - 5, 10, 10),
                pointer_color,
            );

            let key_feedback_text = match demo_state.last_key {
                Some(k) => format!("Keys pressed: {} (Last: {})", demo_state.key_press_count, k),
                None => format!("Keys pressed: {}", demo_state.key_press_count),
            };

            let key_width = 20 + (demo_state.key_press_count.min(20) * 8);

            out_frame.fill_rect(Rect::new(10, 100, key_width, 12), Color::BLUE);

            out_frame.draw_text(key_feedback_text, 10, 100, Color::WHITE);

            LoopControl::Continue
        })
        .expect("event loop must succeed");

    let _ = session.close();
    println!("Session state after close: {:?}", session.state());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_scene_frame_generation_with_feedback() {
        let placement = build_static_placement();

        let demo_state = DemoState {
            pointer_x: 50,
            pointer_y: 50,
            is_pointer_down: true,
            key_press_count: 5,
            last_key: Some(65),
        };

        let mut out_frame = generate_draw_frame(&placement);

        let pointer_color = if demo_state.is_pointer_down {
            Color::RED
        } else {
            Color::GREEN
        };

        out_frame.fill_rect(
            Rect::new(demo_state.pointer_x - 5, demo_state.pointer_y - 5, 10, 10),
            pointer_color,
        );

        // Verify the original placement was drawn
        // Typically it has a background rect and then the items
        assert!(!out_frame.is_empty());

        use prom_ui_runtime::DrawCommand;
        // Verify that the feedback rectangle is exactly what we pushed
        let commands = out_frame.commands();
        let last_cmd = commands.last().unwrap();
        if let DrawCommand::FillRect { rect, color } = last_cmd {
            assert_eq!(rect.x, 45);
            assert_eq!(rect.y, 45);
            assert_eq!(rect.width, 10);
            assert_eq!(rect.height, 10);
            assert_eq!(*color, Color::RED);
        } else {
            panic!("Expected FillRect at the end for pointer feedback");
        }
    }
}
