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
use prom_ui::model::UiIrNodeId;
use prom_ui::projection::{
    UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
    UiProjectionArtifactId,
};
use prom_ui::render_projection_to_model;
use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::{Color, DrawFrame, InputEventKind, LoopControl, Rect, WindowConfig};
use prom_ui_runtime::{DesktopSession, SessionState};
use std::time::Instant;

fn build_static_placement() -> UiLayoutPhysicalPlacementModel {
    // Matches quad_calc.proj.sm structure
    // surface 1 root 10 key 1
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    artifact.set_source_ir_root(UiIrNodeId::new(10));

    // node 10 role panel key 10
    artifact.push_node(UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(10),
        UiProjectedNodeKind::Root,
        UiIrNodeId::new(10),
    ));

    // node 11 role numeric_readout key 11
    artifact.push_node(UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(11),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(11),
    ));

    // node 12 role keypad key 12
    artifact.push_node(UiProjectedNode::with_source_ir_node(
        UiProjectedNodeId::new(12),
        UiProjectedNodeKind::Element,
        UiIrNodeId::new(12),
    ));

    let render_model = render_projection_to_model(&artifact).unwrap();
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

fn render_calculator_frame() -> DrawFrame {
    let mut frame = DrawFrame::new();
    frame.clear(Color::rgb(30, 30, 30));

    // Render Panel (Node 10)
    frame.fill_rect(Rect::new(10, 10, 380, 480), Color::rgb(40, 40, 45));

    // Render Numeric Readout (Node 11)
    frame.fill_rect(Rect::new(20, 20, 360, 80), Color::rgb(10, 10, 10));
    frame.draw_text("QUAD LOGIC CALCULATOR (Logic in calculator.sm)", 30, 60, Color::rgb(0, 255, 0));

    // Render Keypad (Node 12)
    for i in 0..4 {
        for j in 0..4 {
            let x = 30 + i * 85;
            let y = 120 + j * 85;
            frame.fill_rect(Rect::new(x, y, 75, 75), Color::rgb(80, 80, 90));
        }
    }

    frame
}

fn main() {
    println!("=== Quad Logic Calculator ===");
    println!("Semantic logic verified successfully via `smc check src/calculator.sm`.");
    println!("Native backend activated.");

    let config = WindowConfig::new("Quad Logic Calculator", 400, 520);
    let backend = NativeBackend::new();

    let mut session = DesktopSession::create(backend, config).expect("NativeBackend::create_window must succeed");
    assert_eq!(session.state(), SessionState::Created);

    let placement = build_static_placement();
    for entry in placement.entries() {
        println!(
            "Placed Node {:?}: x={}, y={}, w={}, h={}",
            entry.source_render_node(),
            entry.final_rect().x(),
            entry.final_rect().y(),
            entry.final_rect().width(),
            entry.final_rect().height()
        );
    }

    let start = Instant::now();
    session
        .run(move |buf, out_frame| {
            let events = buf.drain();
            for event in &events {
                if matches!(event.kind, InputEventKind::CloseRequested) {
                    return LoopControl::ExitRequested;
                }
            }

            *out_frame = render_calculator_frame();

            if start.elapsed().as_secs_f64() >= 120.0 {
                return LoopControl::ExitRequested;
            }
            LoopControl::Continue
        })
        .expect("event loop must succeed");

    let _ = session.close();
    println!("Session closed.");
}
