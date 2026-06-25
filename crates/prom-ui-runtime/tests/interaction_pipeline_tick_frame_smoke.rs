use prom_ui::{
    action_binding::InteractionActionBindingId,
    layout::geometry::UiLayoutGeometryModel,
    layout::{build_layout_geometry, layout_render_model},
    model::UiIrNodeId,
    projection::UiProjectedNodeId,
    projection::{
        UiProjectedNode, UiProjectedNodeKind, UiProjectionArtifact, UiProjectionArtifactId,
    },
    renderer::render_projection_to_model,
    InteractionAdmittedSemanticAction, SemanticIntent, UiActionDispatcher,
};
use prom_ui_runtime::action_mapping::UiActionMapper;
use prom_ui_runtime::intent_admission::RuntimeIntentAdmission;
use prom_ui_runtime::interaction::{RoutedInteraction, UiHitTester};
use prom_ui_runtime::interaction_pipeline::{
    EventCoordinates, InteractionPipeline, InteractionPipelineReport,
};
use prom_ui_runtime::{DesktopSession, InMemoryBackend, InputEvent, InputEventKind, LoopControl};

#[derive(Clone)]
struct TestEvent {
    has_coords: bool,
    x: f64,
    y: f64,
}

impl EventCoordinates for TestEvent {
    fn hit_test_coords(&self) -> Option<(f64, f64)> {
        if self.has_coords {
            Some((self.x, self.y))
        } else {
            None
        }
    }
}

struct TestHitTester;

impl UiHitTester for TestHitTester {
    fn find_target_at(
        &self,
        _layout: &UiLayoutGeometryModel,
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

impl UiActionMapper<TestEvent> for TestMapper {
    fn map_interaction(&self, interaction: RoutedInteraction<TestEvent>) -> Option<SemanticIntent> {
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
        // No-op for inert test
        Ok(())
    }
}

fn test_layout_geometry() -> UiLayoutGeometryModel {
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
fn interaction_pipeline_tick_frame_smoke() {
    let backend = InMemoryBackend::new();
    let config = prom_ui_runtime::WindowConfig::new("Smoke Test", 800, 600);
    let mut session = DesktopSession::create(backend, config).unwrap();

    let pipeline = InteractionPipeline::new(
        TestHitTester,
        TestMapper,
        RuntimeIntentAdmission::new(),
        TestDispatcher,
    );

    let layout = test_layout_geometry();

    // Start the session loop to transition to Running state
    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    // Queue events in the backend to be polled by the frame tick
    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));
    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 2 }));
    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 3 }));
    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::CloseRequested));

    let mut final_report = InteractionPipelineReport::default();

    session
        .tick_in_memory_frame(|events, _frame| {
            // Map the `InputEvent`s polled from the backend to `TestEvent`s
            // that implement `EventCoordinates`.
            let test_events: Vec<TestEvent> = events
                .iter()
                .map(|e| match e.kind {
                    InputEventKind::KeyDown { key_code: 1 } => {
                        // No coordinates -> skipped
                        TestEvent {
                            has_coords: false,
                            x: 0.0,
                            y: 0.0,
                        }
                    }
                    InputEventKind::KeyDown { key_code: 2 } => {
                        // Negative coordinates -> routed = None
                        TestEvent {
                            has_coords: true,
                            x: -10.0,
                            y: -10.0,
                        }
                    }
                    InputEventKind::KeyDown { key_code: 3 } => {
                        // Positive coordinates -> routed -> mapped -> denied (inert)
                        TestEvent {
                            has_coords: true,
                            x: 10.0,
                            y: 10.0,
                        }
                    }
                    _ => {
                        // Treat any other events as having no coordinates
                        TestEvent {
                            has_coords: false,
                            x: 0.0,
                            y: 0.0,
                        }
                    }
                })
                .collect();

            let report = pipeline.process_batch_report(&test_events, &layout);
            final_report = report;

            LoopControl::ExitRequested
        })
        .unwrap();

    // Assert the exact flow of the pipeline
    assert_eq!(final_report.received, 4);
    // 2 events have no coordinates (key 1 and CloseRequested)
    assert_eq!(final_report.skipped_no_coordinates, 2);
    // 1 event is routed (key 3 has positive coords)
    assert_eq!(final_report.routed, 1);
    // 1 event is mapped (target node id 1)
    assert_eq!(final_report.mapped, 1);
    // 0 admitted because RuntimeIntentAdmission uses InertCapabilityEvaluator in Wave 0
    assert_eq!(final_report.admitted, 0);
    // 1 denied due to missing capability
    assert_eq!(final_report.denied, 1);
    // 0 dispatched
    assert_eq!(final_report.dispatched, 0);
}
