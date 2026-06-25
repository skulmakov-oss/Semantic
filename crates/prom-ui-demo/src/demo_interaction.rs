use prom_ui::action_binding::InteractionActionBindingId;
use prom_ui::layout::physical_placement::{hit_test_placement, UiLayoutPhysicalPlacementModel};
use prom_ui::projection::UiProjectedNodeId;
use prom_ui::renderer::UiRenderNodeId;
use prom_ui::SemanticIntent;
use prom_ui_backend_native::draw_generation::generate_draw_frame;
use prom_ui_runtime::{
    Color, DrawFrame, InputEvent, InputEventKind, LoopControl, Rect, RuntimeIntentAdmission,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DispatchFeedback {
    #[default]
    None,
    IntentBuilt,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DemoState {
    pointer_x: i32,
    pointer_y: i32,
    is_pointer_down: bool,
    key_press_count: u32,
    last_key: Option<u32>,
    hovered_node: Option<UiRenderNodeId>,
    selected_node: Option<UiRenderNodeId>,
    focused_node: Option<UiRenderNodeId>,
    dispatch_feedback: DispatchFeedback,
    denied_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DemoActionBindings {
    bindings: HashMap<UiRenderNodeId, (UiProjectedNodeId, InteractionActionBindingId)>,
}

impl DemoActionBindings {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(
            UiRenderNodeId::new(1),
            (UiProjectedNodeId::new(1), InteractionActionBindingId(100)),
        );
        bindings.insert(
            UiRenderNodeId::new(2),
            (UiProjectedNodeId::new(2), InteractionActionBindingId(200)),
        );
        Self { bindings }
    }

    pub fn build_intent(&self, render_node: UiRenderNodeId) -> Option<SemanticIntent> {
        self.bindings
            .get(&render_node)
            .map(|(projected_node, binding_id)| SemanticIntent::new(*projected_node, *binding_id))
    }
}

pub struct DemoInteraction {
    state: DemoState,
    bindings: DemoActionBindings,
    admission_gate: RuntimeIntentAdmission,
}

impl DemoInteraction {
    pub fn new() -> Self {
        Self {
            state: DemoState::default(),
            bindings: DemoActionBindings::new(),
            admission_gate: RuntimeIntentAdmission::new(),
        }
    }

    pub fn state(&self) -> &DemoState {
        &self.state
    }

    pub fn apply_events(
        &mut self,
        events: &[InputEvent],
        placement: &UiLayoutPhysicalPlacementModel,
    ) -> LoopControl {
        for event in events {
            if self.handle_event(event, placement) == LoopControl::ExitRequested {
                return LoopControl::ExitRequested;
            }
        }

        LoopControl::Continue
    }

    fn handle_event(
        &mut self,
        event: &InputEvent,
        placement: &UiLayoutPhysicalPlacementModel,
    ) -> LoopControl {
        match event.kind {
            InputEventKind::CloseRequested => LoopControl::ExitRequested,
            InputEventKind::PointerMoved { x, y } => {
                self.handle_pointer_moved(x, y, placement);
                LoopControl::Continue
            }
            InputEventKind::PointerDown { .. } => {
                self.handle_pointer_down();
                LoopControl::Continue
            }
            InputEventKind::PointerUp { .. } => {
                self.handle_pointer_up();
                LoopControl::Continue
            }
            InputEventKind::KeyDown { key_code } => {
                self.handle_key_down(key_code);
                LoopControl::Continue
            }
            InputEventKind::KeyUp { .. } => LoopControl::Continue,
        }
    }

    fn handle_pointer_moved(&mut self, x: f64, y: f64, placement: &UiLayoutPhysicalPlacementModel) {
        self.state.pointer_x = x as i32;
        self.state.pointer_y = y as i32;
        self.state.hovered_node = hit_test_placement(x, y, placement);
    }

    fn handle_pointer_down(&mut self) {
        self.state.is_pointer_down = true;
        self.state.selected_node = self.state.hovered_node;
        self.state.focused_node = self.state.hovered_node;

        if let Some(selected) = self.state.selected_node {
            self.trigger_binding(selected);
        }

        if self.state.selected_node.is_none() {
            self.state.dispatch_feedback = DispatchFeedback::None;
        }
    }

    fn handle_pointer_up(&mut self) {
        self.state.is_pointer_down = false;
    }

    fn handle_key_down(&mut self, key_code: u32) {
        self.state.key_press_count += 1;
        self.state.last_key = Some(key_code);

        if key_code == 13 || key_code == 32 {
            if let Some(focused) = self.state.focused_node {
                self.trigger_binding(focused);
            } else {
                self.state.dispatch_feedback = DispatchFeedback::None;
            }
        }
    }

    fn trigger_binding(&mut self, render_node: UiRenderNodeId) {
        if let Some(intent) = self.bindings.build_intent(render_node) {
            self.state.dispatch_feedback = DispatchFeedback::IntentBuilt;
            match self.admission_gate.admit_intent(intent) {
                Ok(_) => {}
                Err(_) => {
                    self.state.dispatch_feedback = DispatchFeedback::Denied;
                    self.state.denied_count += 1;
                }
            }
        } else {
            self.state.dispatch_feedback = DispatchFeedback::None;
        }
    }
}

pub fn render_demo_frame(
    placement: &UiLayoutPhysicalPlacementModel,
    interaction: &DemoInteraction,
) -> DrawFrame {
    let mut out_frame = generate_draw_frame(placement);
    render_feedback_overlays(&mut out_frame, placement, interaction.state());
    out_frame
}

fn render_feedback_overlays(
    out_frame: &mut DrawFrame,
    placement: &UiLayoutPhysicalPlacementModel,
    state: &DemoState,
) {
    let pointer_color = if state.is_pointer_down {
        Color::RED
    } else {
        Color::GREEN
    };

    for entry in placement.entries() {
        let node_id = entry.source_render_node();
        let rect = entry.final_rect();

        if Some(node_id) == state.selected_node {
            draw_selected_feedback(out_frame, rect);
            draw_dispatch_feedback(out_frame, rect, state.dispatch_feedback);
        } else if Some(node_id) == state.hovered_node {
            draw_hovered_feedback(out_frame, rect);
        }

        if Some(node_id) == state.focused_node {
            draw_focused_feedback(out_frame, rect);
        }
    }

    draw_pointer_status(out_frame, state, pointer_color);
    draw_key_status(out_frame, state);
}

fn draw_hovered_feedback(out_frame: &mut DrawFrame, rect: prom_ui::layout::UiLayoutGeometryRect) {
    draw_outline(out_frame, rect, Color::WHITE, 2);
}

fn draw_selected_feedback(out_frame: &mut DrawFrame, rect: prom_ui::layout::UiLayoutGeometryRect) {
    draw_outline(out_frame, rect, Color::rgb(255, 255, 0), 2);
}

fn draw_focused_feedback(out_frame: &mut DrawFrame, rect: prom_ui::layout::UiLayoutGeometryRect) {
    draw_outline(out_frame, rect, Color::WHITE, 2);
}

fn draw_dispatch_feedback(
    out_frame: &mut DrawFrame,
    rect: prom_ui::layout::UiLayoutGeometryRect,
    feedback: DispatchFeedback,
) {
    if matches!(feedback, DispatchFeedback::None) {
        return;
    }

    let color = match feedback {
        DispatchFeedback::None => Color::rgb(128, 128, 128),
        DispatchFeedback::IntentBuilt => Color::BLUE,
        DispatchFeedback::Denied => Color::RED,
    };

    out_frame.fill_rect(Rect::new(rect.x() + 5, rect.y() - 10, 20, 8), color);
}

fn draw_pointer_status(out_frame: &mut DrawFrame, state: &DemoState, pointer_color: Color) {
    out_frame.draw_text(
        format!(
            "Pointer: ({}, {}), Keys: {}, Denied: {}",
            state.pointer_x, state.pointer_y, state.key_press_count, state.denied_count
        ),
        10,
        30,
        pointer_color,
    );

    if state.hovered_node.is_none() {
        out_frame.fill_rect(
            Rect::new(state.pointer_x - 5, state.pointer_y - 5, 10, 10),
            pointer_color,
        );
    }
}

fn draw_key_status(out_frame: &mut DrawFrame, state: &DemoState) {
    let key_feedback_text = match state.last_key {
        Some(key) => format!("Keys pressed: {} (Last: {})", state.key_press_count, key),
        None => format!("Keys pressed: {}", state.key_press_count),
    };

    let key_width = 20 + (state.key_press_count.min(20) * 8);
    out_frame.fill_rect(Rect::new(10, 100, key_width, 12), Color::BLUE);
    out_frame.draw_text(key_feedback_text, 10, 100, Color::WHITE);
}

fn draw_outline(
    out_frame: &mut DrawFrame,
    rect: prom_ui::layout::UiLayoutGeometryRect,
    color: Color,
    thickness: u32,
) {
    let x = rect.x();
    let y = rect.y();
    let w = rect.width();
    let h = rect.height();
    let thickness_i32 = thickness as i32;

    out_frame.fill_rect(Rect::new(x, y, w, thickness), color);
    out_frame.fill_rect(
        Rect::new(x, y + (h as i32) - thickness_i32, w, thickness),
        color,
    );
    out_frame.fill_rect(Rect::new(x, y, thickness, h), color);
    out_frame.fill_rect(
        Rect::new(x + (w as i32) - thickness_i32, y, thickness, h),
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use prom_ui::renderer::UiRenderNodeId;
    use prom_ui_runtime::{InputEvent, InputEventKind};

    fn test_interaction() -> DemoInteraction {
        DemoInteraction::new()
    }

    fn test_placement() -> UiLayoutPhysicalPlacementModel {
        super::super::build_static_placement()
    }

    #[test]
    fn pointer_moved_updates_hovered_node() {
        let mut interaction = test_interaction();
        let placement = test_placement();

        let events = vec![InputEvent::new(InputEventKind::PointerMoved {
            x: 15.0,
            y: 25.0,
        })];

        let control = interaction.apply_events(&events, &placement);
        assert_eq!(control, LoopControl::Continue);
        assert_eq!(interaction.state.hovered_node, Some(UiRenderNodeId::new(2)));
    }

    #[test]
    fn pointer_down_updates_selected_and_focused_node() {
        let mut interaction = test_interaction();
        let placement = test_placement();

        let events = vec![
            InputEvent::new(InputEventKind::PointerMoved { x: 15.0, y: 25.0 }),
            InputEvent::new(InputEventKind::PointerDown { button: 1 }),
        ];

        let control = interaction.apply_events(&events, &placement);
        assert_eq!(control, LoopControl::Continue);
        assert_eq!(
            interaction.state.selected_node,
            Some(UiRenderNodeId::new(2))
        );
        assert_eq!(interaction.state.focused_node, Some(UiRenderNodeId::new(2)));
        assert_eq!(
            interaction.state.dispatch_feedback,
            DispatchFeedback::Denied
        );
        assert_eq!(interaction.state.denied_count, 1);
    }

    #[test]
    fn ordinary_key_does_not_trigger_intent() {
        let mut interaction = test_interaction();
        let placement = test_placement();

        interaction.state.focused_node = Some(UiRenderNodeId::new(2));

        let events = vec![InputEvent::new(InputEventKind::KeyDown { key_code: 65 })];
        let control = interaction.apply_events(&events, &placement);

        assert_eq!(control, LoopControl::Continue);
        assert_eq!(interaction.state.key_press_count, 1);
        assert_eq!(interaction.state.last_key, Some(65));
        assert_eq!(interaction.state.dispatch_feedback, DispatchFeedback::None);
        assert_eq!(interaction.state.denied_count, 0);
    }

    #[test]
    fn enter_or_space_on_focused_node_triggers_denied_feedback() {
        let mut interaction = test_interaction();
        let placement = test_placement();

        let focus_events = vec![
            InputEvent::new(InputEventKind::PointerMoved { x: 15.0, y: 25.0 }),
            InputEvent::new(InputEventKind::PointerDown { button: 1 }),
        ];
        assert_eq!(
            interaction.apply_events(&focus_events, &placement),
            LoopControl::Continue
        );

        interaction.state.dispatch_feedback = DispatchFeedback::None;
        interaction.state.denied_count = 0;

        let events = vec![InputEvent::new(InputEventKind::KeyDown { key_code: 13 })];
        let control = interaction.apply_events(&events, &placement);

        assert_eq!(control, LoopControl::Continue);
        assert_eq!(interaction.state.key_press_count, 1);
        assert_eq!(interaction.state.last_key, Some(13));
        assert_eq!(
            interaction.state.dispatch_feedback,
            DispatchFeedback::Denied
        );
        assert_eq!(interaction.state.denied_count, 1);
    }

    #[test]
    fn render_feedback_adds_borders_and_markers() {
        let placement = test_placement();
        let mut interaction = test_interaction();

        interaction.state.pointer_x = 15;
        interaction.state.pointer_y = 25;
        interaction.state.hovered_node = Some(UiRenderNodeId::new(2));
        interaction.state.selected_node = Some(UiRenderNodeId::new(2));
        interaction.state.focused_node = Some(UiRenderNodeId::new(2));
        interaction.state.dispatch_feedback = DispatchFeedback::Denied;
        interaction.state.is_pointer_down = true;

        let frame = render_demo_frame(&placement, &interaction);
        let commands = frame.commands();

        assert!(!commands.is_empty());
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::rgb(255, 255, 0)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. } if *color == Color::RED
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. } if *color == Color::WHITE
        )));
    }
}
