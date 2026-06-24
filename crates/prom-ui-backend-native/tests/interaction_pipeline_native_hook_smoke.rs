use prom_ui::{
    action_binding::InteractionActionBindingId,
    layout::{build_layout_geometry, layout_render_model},
    model::UiIrNodeId,
    projection::{
        UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
        UiProjectionArtifactId,
    },
    renderer::render_projection_to_model,
    InteractionAdmittedSemanticAction, SemanticIntent, UiActionDispatcher,
};
use prom_ui_backend_native::{RawBackendEvent, RawButtonState, RawKeyCode};
use prom_ui_runtime::action_mapping::UiActionMapper;
use prom_ui_runtime::intent_admission::RuntimeIntentAdmission;
use prom_ui_runtime::interaction::{RoutedInteraction, UiHitTester};
use prom_ui_runtime::interaction_pipeline::InteractionPipeline;

struct TestHitTester;

impl UiHitTester for TestHitTester {
    fn find_target_at(
        &self,
        _layout: &prom_ui::layout::geometry::UiLayoutGeometryModel,
        x: f64,
        y: f64,
    ) -> Option<UiIrNodeId> {
        if x > 0.0 && y > 0.0 {
            Some(UiIrNodeId::new(1))
        } else {
            None
        }
    }
}

struct TestMapper;

impl UiActionMapper<RawBackendEvent> for TestMapper {
    fn map_interaction(
        &self,
        interaction: RoutedInteraction<RawBackendEvent>,
    ) -> Option<SemanticIntent> {
        if interaction.target.raw() == 1 {
            Some(SemanticIntent::new(
                UiProjectedNodeId::new(interaction.target.raw()),
                InteractionActionBindingId(100),
            ))
        } else {
            None
        }
    }
}

struct TestDispatcher;

impl UiActionDispatcher for TestDispatcher {
    fn dispatch_action(
        &self,
        _action: InteractionAdmittedSemanticAction,
    ) -> Result<(), prom_ui::IntentDispatchError> {
        Ok(())
    }
}

fn test_layout_geometry() -> prom_ui::layout::geometry::UiLayoutGeometryModel {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(1),
        UiProjectedNodeKind::Root,
    ));

    let render_model = render_projection_to_model(&artifact).expect("render model");
    let layout_model = layout_render_model(&render_model);
    build_layout_geometry(&layout_model)
}

#[test]
fn native_event_pipeline_hook_smoke() {
    let pipeline = InteractionPipeline::new(
        TestHitTester,
        TestMapper,
        RuntimeIntentAdmission::new(),
        TestDispatcher,
    );

    let layout = test_layout_geometry();

    let events = vec![
        RawBackendEvent::KeyboardInput {
            key: RawKeyCode::KeyA,
            state: RawButtonState::Pressed,
        },
        RawBackendEvent::PointerMoved { x: -10.0, y: -10.0 },
        RawBackendEvent::PointerMoved { x: 10.0, y: 10.0 },
    ];

    let report = pipeline.process_batch_report(&events, &layout);

    assert_eq!(report.received, 3);
    assert_eq!(report.skipped_no_coordinates, 1); // KeyboardInput has no coordinates
    assert_eq!(report.routed, 1); // Only positive coords route to target id 1
    assert_eq!(report.mapped, 1); // Target 1 gets mapped
    assert_eq!(report.admitted, 0); // Inert evaluator denies everything
    assert_eq!(report.denied, 1);
    assert_eq!(report.dispatched, 0);
}
