use prom_ui::{
    action_binding::InteractionActionBindingId,
    layout::geometry::UiLayoutGeometryModel,
    layout::{build_layout_geometry, layout_render_model},
    model::UiIrNodeId,
    projection::{
        UiProjectedNode, UiProjectedNodeId, UiProjectedNodeKind, UiProjectionArtifact,
        UiProjectionArtifactId,
    },
    renderer::render_projection_to_model,
    InteractionAdmittedSemanticAction, SemanticIntent, UiActionDispatcher,
};
use prom_ui_backend_native::session_hook::tick_native_interaction_pipeline;
use prom_ui_backend_native::{RawBackendEvent, RawButtonState, RawKeyCode};
use prom_ui_runtime::action_mapping::UiActionMapper;
use prom_ui_runtime::intent_admission::RuntimeIntentAdmission;
use prom_ui_runtime::interaction::{RoutedInteraction, UiHitTester};
use prom_ui_runtime::interaction_pipeline::InteractionPipeline;

// Mocks for pipeline components

#[derive(Clone)]
struct MockHitTester;

impl UiHitTester for MockHitTester {
    fn find_target_at(
        &self,
        _layout: &UiLayoutGeometryModel,
        _x: f64,
        _y: f64,
    ) -> Option<UiIrNodeId> {
        Some(UiIrNodeId::new(42))
    }
}

#[derive(Clone)]
struct MockActionMapper;

impl UiActionMapper<RawBackendEvent> for MockActionMapper {
    fn map_interaction(
        &self,
        interaction: RoutedInteraction<RawBackendEvent>,
    ) -> Option<SemanticIntent> {
        match interaction.event {
            RawBackendEvent::PointerMoved { .. } => Some(SemanticIntent::new(
                UiProjectedNodeId::new(interaction.target.raw()),
                InteractionActionBindingId(100),
            )),
            _ => None,
        }
    }
}

#[derive(Clone, Default)]
struct MockDispatcher {
    dispatched_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl UiActionDispatcher for MockDispatcher {
    fn dispatch_action(
        &self,
        _action: InteractionAdmittedSemanticAction,
    ) -> Result<(), prom_ui::IntentDispatchError> {
        self.dispatched_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

fn test_layout_geometry() -> UiLayoutGeometryModel {
    let mut artifact = UiProjectionArtifact::new(UiProjectionArtifactId::new(1));
    artifact.push_node(UiProjectedNode::new(
        UiProjectedNodeId::new(42),
        UiProjectedNodeKind::Root,
    ));

    let render_model = render_projection_to_model(&artifact).expect("render model");
    let layout_model = layout_render_model(&render_model);
    build_layout_geometry(&layout_model)
}

#[test]
fn native_pipeline_session_hook_contract() {
    let hit_tester = MockHitTester;
    let action_mapper = MockActionMapper;
    let admission = RuntimeIntentAdmission::new();
    let dispatcher = MockDispatcher::default();

    let pipeline =
        InteractionPipeline::new(hit_tester, action_mapper, admission, dispatcher.clone());

    let layout = test_layout_geometry();

    // Mixed events batch
    let events = vec![
        RawBackendEvent::PointerMoved { x: 100.0, y: 150.0 }, // Has coordinates, will be hit-tested and admitted (UserSubmit)
        RawBackendEvent::KeyboardInput {
            key: RawKeyCode::Enter,
            state: RawButtonState::Pressed,
        }, // No coordinates mapped yet, skipped by routing in this wave
        RawBackendEvent::CloseRequested,                      // No coordinates mapped, skipped
    ];

    let report = tick_native_interaction_pipeline(&events, &layout, &pipeline);

    // Verify correct processing counts
    assert_eq!(report.received, 3);
    assert_eq!(report.skipped_no_coordinates, 2); // Keyboard and CloseRequested
    assert_eq!(report.routed, 1); // PointerMoved
    assert_eq!(report.mapped, 1); // PointerMoved mapped
                                  // InertCapabilityEvaluator denies all semantic intents by default
    assert_eq!(report.admitted, 0);
    assert_eq!(report.denied, 1);
    assert_eq!(report.dispatched, 0);

    // Ensure dispatcher received 0
    assert_eq!(
        dispatcher
            .dispatched_count
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}
