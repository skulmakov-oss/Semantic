use crate::action_mapping::UiActionMapper;
use crate::intent_admission::RuntimeIntentAdmission;
use crate::interaction::{RoutedInteraction, UiHitTester};
use prom_ui::layout::geometry::UiLayoutGeometryModel;
use prom_ui::UiActionDispatcher;

/// Extracts physical hit-testing coordinates from a generic raw event.
///
/// This represents the "capture/normalize" phase of the pipeline, translating
/// native or raw UI events into a uniform coordinate space before routing.
pub trait EventCoordinates {
    fn hit_test_coords(&self) -> Option<(f64, f64)>;
}

/// A facade that processes a batch of raw physical events through the complete
/// semantic interaction pipeline:
///
/// `capture/normalize -> route -> map -> admit -> dispatch`
///
/// It acts purely as a coordinator, invoking the independent boundary abstractions
/// without bypassing safety capability gates or creating direct mutations.
#[derive(Debug, Clone)]
pub struct InteractionPipeline<H, M, D> {
    pub hit_tester: H,
    pub action_mapper: M,
    pub admission: RuntimeIntentAdmission,
    pub dispatcher: D,
}

impl<H, M, D> InteractionPipeline<H, M, D> {
    pub fn new(
        hit_tester: H,
        action_mapper: M,
        admission: RuntimeIntentAdmission,
        dispatcher: D,
    ) -> Self {
        Self {
            hit_tester,
            action_mapper,
            admission,
            dispatcher,
        }
    }
}

impl<H, M, D> InteractionPipeline<H, M, D>
where
    H: UiHitTester,
    D: UiActionDispatcher,
{
    /// Process a batch of raw physical events.
    pub fn process_batch<E>(&self, events: &[E], layout: &UiLayoutGeometryModel)
    where
        E: EventCoordinates + Clone,
        M: UiActionMapper<E>,
    {
        for event in events {
            // 1. Capture / Normalize
            // Extract the normalized coordinates for hit-testing.
            // If the event does not have coordinates (e.g., keyboard input),
            // it is skipped by this hit-testing router in Wave 0.
            if let Some((x, y)) = event.hit_test_coords() {
                // 2. Route
                // Perform geometric hit-testing against the rendered UI layout.
                if let Some(target) = self.hit_tester.find_target_at(layout, x, y) {
                    let routed = RoutedInteraction {
                        target,
                        event: event.clone(),
                    };

                    // 3. Map
                    // Map the physical event on a semantic node to a SemanticIntent.
                    if let Some(intent) = self.action_mapper.map_interaction(routed) {
                        // 4. Admit
                        // Verify the semantic intent against component capabilities
                        // and lifecycle gates.
                        if let Ok(admitted) = self.admission.admit_intent(intent) {
                            // 5. Dispatch
                            // Issue the fully authorized interaction to the state manager.
                            let _ = self.dispatcher.dispatch_action(admitted);
                        }
                    }
                }
            }
        }
    }
}
