use crate::RawBackendEvent;
use prom_ui::layout::geometry::UiLayoutGeometryModel;
use prom_ui::UiActionDispatcher;
use prom_ui_runtime::action_mapping::UiActionMapper;
use prom_ui_runtime::interaction::UiHitTester;
use prom_ui_runtime::interaction_pipeline::{InteractionPipeline, InteractionPipelineReport};

/// Ticks the generic interaction pipeline with a batch of native events.
///
/// This demonstrates the transport boundary: the native backend only transports
/// physical evidence (`RawBackendEvent`). The actual interpretation, routing, admission, and semantic
/// dispatch is entirely coordinated by the `InteractionPipeline` from the runtime.
pub fn tick_native_interaction_pipeline<H, M, D>(
    events: &[RawBackendEvent],
    layout: &UiLayoutGeometryModel,
    pipeline: &InteractionPipeline<H, M, D>,
) -> InteractionPipelineReport
where
    H: UiHitTester,
    M: UiActionMapper<RawBackendEvent>,
    D: UiActionDispatcher,
{
    pipeline.process_batch_report(events, layout)
}
