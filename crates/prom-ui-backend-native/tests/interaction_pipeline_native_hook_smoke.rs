use prom_ui::{
    action_binding::InteractionActionBindingId, model::UiIrNodeId, projection::UiProjectedNodeId,
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

#[test]
fn native_event_pipeline_hook_smoke() {
    let pipeline = InteractionPipeline::new(
        TestHitTester,
        TestMapper,
        RuntimeIntentAdmission::new(),
        TestDispatcher,
    );

    let dummy_layout =
        std::mem::MaybeUninit::<prom_ui::layout::geometry::UiLayoutGeometryModel>::uninit();
    let layout = unsafe { &*dummy_layout.as_ptr() };

    let events = vec![
        RawBackendEvent::KeyboardInput {
            key: RawKeyCode::KeyA,
            state: RawButtonState::Pressed,
        },
        RawBackendEvent::PointerMoved { x: -10.0, y: -10.0 },
        RawBackendEvent::PointerMoved { x: 10.0, y: 10.0 },
    ];

    let report = pipeline.process_batch_report(&events, layout);

    assert_eq!(report.received, 3);
    assert_eq!(report.skipped_no_coordinates, 1); // KeyboardInput has no coordinates
    assert_eq!(report.routed, 1); // Only positive coords route to target id 1
    assert_eq!(report.mapped, 1); // Target 1 gets mapped
    assert_eq!(report.admitted, 0); // Inert evaluator denies everything
    assert_eq!(report.denied, 1);
    assert_eq!(report.dispatched, 0);
}
