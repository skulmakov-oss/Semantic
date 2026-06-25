use prom_ui_runtime::{
    DesktopSession, DrawFrame, EventBuffer, FrameToken, FrameTokenIssuer,
    InputEventKind, LoopControl, SessionState, WindowConfig,
};
use prom_ui_backend_native::NativeBackend;
use prom_ui_backend_native::draw_generation::generate_draw_frame;
use prom_ui::{
    layout::{
        constraint_solver::build_layout_constraint_solver,
        constraints::build_layout_constraints,
        geometry::build_layout_geometry,
        measuring::build_layout_measuring,
        physical_placement::build_layout_physical_placement,
        size_to_fit::build_layout_size_to_fit,
        sizing::build_layout_sizing,
        sizing_algorithm::build_layout_sizing_algorithm,
        solving::{build_layout_solving, build_layout_solving_result},
    },
};

fn main() {
    println!("=== Semantic UI Application Boundary — Native Render Demo ===");

    let config = WindowConfig::new("Semantic UI Demo - WGPU Native", 800, 600);
    let backend = NativeBackend::new();

    let mut session =
        DesktopSession::create(backend, config).expect("NativeBackend::create_window must succeed");
    assert_eq!(session.state(), SessionState::Created);
    println!("Session state: {:?}", session.state());

    let mut issuer = FrameTokenIssuer::new();
    let mut frame_tokens: Vec<FrameToken> = Vec::new();

    session
        .run(|buf: &mut EventBuffer, out_frame: &mut DrawFrame| {
            // Drain any pending events
            let events = buf.drain();
            for evt in &events {
                match &evt.kind {
                    InputEventKind::CloseRequested => {
                        println!("  Event: CloseRequested -> Exiting loop");
                        return LoopControl::ExitRequested;
                    }
                    _ => {}
                }
            }

            let token = issuer.next();
            frame_tokens.push(token);

            use prom_ui::projection::{UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact, UiProjectionArtifactId};
            use prom_ui::model::UiIrNodeId;
            
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
            let placement = build_layout_physical_placement(&solving_result);

            // Print physical placement to console
            for entry in placement.entries() {
                println!(
                    "Placed RenderNode({}): x={}, y={}, w={}, h={}",
                    entry.source_render_node().raw(),
                    entry.final_rect().x(),
                    entry.final_rect().y(),
                    entry.final_rect().width(),
                    entry.final_rect().height()
                );
            }

            // Generate frame commands and assign it to the output frame
            *out_frame = generate_draw_frame(&placement);

            if frame_tokens.len() >= 10 {
                return LoopControl::ExitRequested;
            }

            LoopControl::Continue
        })
        .expect("event loop must succeed");

    let _ = session.close();
    println!("Session state after close: {:?}", session.state());
}
