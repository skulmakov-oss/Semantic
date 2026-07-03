mod calculator_shell;
mod demo_interaction;
mod renderer_layout_inspector;

use demo_interaction::{render_demo_frame, DemoInteraction};
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
use prom_ui_runtime::{DesktopSession, EventBuffer, LoopControl, SessionState, WindowConfig};
use renderer_layout_inspector::{demo_inspector_tree, render_inspector_text};
use std::env;

fn build_static_placement() -> UiLayoutPhysicalPlacementModel {
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

fn main() {
    if inspector_requested() {
        let tree = demo_inspector_tree();
        let output = render_inspector_text(&tree, "root");
        println!("{output}");
        return;
    }

    println!("=== Semantic UI Application Boundary - Native Render Demo ===");

    let config = WindowConfig::new("Semantic Calculator", 800, 600);
    let backend = NativeBackend::new();

    let mut session =
        DesktopSession::create(backend, config).expect("NativeBackend::create_window must succeed");
    assert_eq!(session.state(), SessionState::Created);
    println!("Session state: {:?}", session.state());

    let placement = build_static_placement();

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

    let mut interaction = DemoInteraction::new();

    session
        .run(move |buf: &mut EventBuffer, out_frame| {
            let events = buf.drain();
            let control = interaction.apply_events(&events, &placement);
            if control == LoopControl::ExitRequested {
                return control;
            }

            *out_frame = render_demo_frame(&placement, &interaction);
            control
        })
        .expect("event loop must succeed");

    let _ = session.close();
    println!("Session state after close: {:?}", session.state());
}

fn inspector_requested() -> bool {
    if env::args().any(|arg| arg == "inspector" || arg == "--inspector") {
        return true;
    }

    match env::var("PROM_UI_DEMO") {
        Ok(value) => value.eq_ignore_ascii_case("inspector"),
        Err(_) => false,
    }
}
