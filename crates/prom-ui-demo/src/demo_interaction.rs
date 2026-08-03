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
use ui_shell_kit::{
    calculator_scene::calculator_layout, CalculatorController, UiEvent, UiEventKind, UiFrame,
    UiPointerButton,
};

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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DemoSceneMode {
    #[default]
    Calculator,
    Diagnostics,
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
    calculator: CalculatorController,
    scene_mode: DemoSceneMode,
}

impl DemoInteraction {
    pub fn new() -> Self {
        Self {
            state: DemoState::default(),
            bindings: DemoActionBindings::new(),
            admission_gate: RuntimeIntentAdmission::new(),
            calculator: CalculatorController::new(),
            scene_mode: DemoSceneMode::Calculator,
        }
    }

    #[cfg(test)]
    pub fn new_diagnostics() -> Self {
        Self {
            scene_mode: DemoSceneMode::Diagnostics,
            ..Self::new()
        }
    }

    pub fn state(&self) -> &DemoState {
        &self.state
    }

    pub fn calculator(&self) -> &CalculatorController {
        &self.calculator
    }

    pub fn seed_calculator_smoke(&mut self, placement: &UiLayoutPhysicalPlacementModel) {
        let Some(scene_bounds) = calculator_scene_bounds(placement) else {
            return;
        };

        let layout = calculator_layout(scene_bounds);
        let scripted_buttons = ["7", "+", "3", "="];

        let mut events = Vec::new();
        for label in scripted_buttons {
            let Some((_, rect)) = layout
                .buttons
                .iter()
                .find(|(button, _)| button.label() == label)
            else {
                continue;
            };

            let x = rect.x() as f64 + (rect.width() as f64 / 2.0);
            let y = rect.y() as f64 + (rect.height() as f64 / 2.0);
            events.push(InputEvent::new(InputEventKind::PointerMoved { x, y }));
            events.push(InputEvent::new(InputEventKind::PointerDown { button: 1 }));
            events.push(InputEvent::new(InputEventKind::PointerUp { button: 1 }));
        }

        let _ = self.apply_events(&events, placement);
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
        match self.scene_mode {
            DemoSceneMode::Calculator => self.handle_calculator_event(event, placement),
            DemoSceneMode::Diagnostics => self.handle_diagnostics_event(event, placement),
        }
    }

    fn handle_calculator_event(
        &mut self,
        event: &InputEvent,
        placement: &UiLayoutPhysicalPlacementModel,
    ) -> LoopControl {
        match event.kind {
            InputEventKind::CloseRequested => LoopControl::ExitRequested,
            InputEventKind::KeyDown { key_code: 27 } => LoopControl::ExitRequested,
            InputEventKind::PointerMoved { x, y } => {
                self.state.pointer_x = x as i32;
                self.state.pointer_y = y as i32;
                if let Some(bounds) = calculator_scene_bounds(placement) {
                    let _ = self
                        .calculator
                        .handle_event(UiEvent::new(UiEventKind::PointerMoved { x, y }), bounds);
                }
                LoopControl::Continue
            }
            InputEventKind::PointerDown { .. } => {
                if let Some(bounds) = calculator_scene_bounds(placement) {
                    self.state.pointer_x = self.state.pointer_x.max(0);
                    self.state.pointer_y = self.state.pointer_y.max(0);
                    let _ = self.calculator.handle_event(
                        UiEvent::new(UiEventKind::PointerDown {
                            x: self.state.pointer_x as f64,
                            y: self.state.pointer_y as f64,
                            button: UiPointerButton::Primary,
                        }),
                        bounds,
                    );
                }
                self.state.is_pointer_down = true;
                LoopControl::Continue
            }
            InputEventKind::PointerUp { .. } => {
                self.state.is_pointer_down = false;
                if let Some(bounds) = calculator_scene_bounds(placement) {
                    let _ = self.calculator.handle_event(
                        UiEvent::new(UiEventKind::PointerUp {
                            x: self.state.pointer_x as f64,
                            y: self.state.pointer_y as f64,
                            button: UiPointerButton::Primary,
                        }),
                        bounds,
                    );
                }
                LoopControl::Continue
            }
            InputEventKind::KeyDown { .. } => LoopControl::Continue,
            InputEventKind::KeyUp { .. } => LoopControl::Continue,
            InputEventKind::Scroll { .. } => LoopControl::Continue,
            InputEventKind::TextInput { .. } => LoopControl::Continue,
            InputEventKind::Resized { .. } => LoopControl::Continue,
            InputEventKind::ScaleFactorChanged { .. } => LoopControl::Continue,
        }
    }

    fn handle_diagnostics_event(
        &mut self,
        event: &InputEvent,
        placement: &UiLayoutPhysicalPlacementModel,
    ) -> LoopControl {
        match event.kind {
            InputEventKind::CloseRequested => LoopControl::ExitRequested,
            InputEventKind::KeyDown { key_code: 27 } => LoopControl::ExitRequested,
            InputEventKind::PointerMoved { x, y } => {
                self.handle_pointer_moved(x, y, placement);
                LoopControl::Continue
            }
            InputEventKind::PointerDown { .. } => {
                self.handle_pointer_down();
                if let Some(bounds) = placement_bounds(placement) {
                    let _ = self.calculator.handle_event(
                        UiEvent::new(UiEventKind::PointerDown {
                            x: self.state.pointer_x as f64,
                            y: self.state.pointer_y as f64,
                            button: UiPointerButton::Primary,
                        }),
                        bounds,
                    );
                }
                LoopControl::Continue
            }
            InputEventKind::PointerUp { .. } => {
                self.handle_pointer_up();
                if let Some(bounds) = placement_bounds(placement) {
                    let _ = self.calculator.handle_event(
                        UiEvent::new(UiEventKind::PointerUp {
                            x: self.state.pointer_x as f64,
                            y: self.state.pointer_y as f64,
                            button: UiPointerButton::Primary,
                        }),
                        bounds,
                    );
                }
                LoopControl::Continue
            }
            InputEventKind::KeyDown { key_code } => {
                self.handle_key_down(key_code);
                LoopControl::Continue
            }
            InputEventKind::KeyUp { .. } => LoopControl::Continue,
            InputEventKind::Scroll { .. } => LoopControl::Continue,
            InputEventKind::TextInput { .. } => LoopControl::Continue,
            InputEventKind::Resized { .. } => LoopControl::Continue,
            InputEventKind::ScaleFactorChanged { .. } => LoopControl::Continue,
        }
    }

    fn handle_pointer_moved(&mut self, x: f64, y: f64, placement: &UiLayoutPhysicalPlacementModel) {
        self.state.pointer_x = x as i32;
        self.state.pointer_y = y as i32;
        self.state.hovered_node = hit_test_placement(x, y, placement);
        if let Some(bounds) = placement_bounds(placement) {
            let _ = self
                .calculator
                .handle_event(UiEvent::new(UiEventKind::PointerMoved { x, y }), bounds);
        }
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
    match interaction.scene_mode {
        DemoSceneMode::Calculator => {
            let mut ui_frame = UiFrame::new();
            let Some(bounds) = calculator_scene_bounds(placement) else {
                return DrawFrame::new();
            };

            interaction.calculator().render(&mut ui_frame, bounds);
            ui_frame.into_draw_frame()
        }
        DemoSceneMode::Diagnostics => {
            let mut out_frame = generate_draw_frame(placement);
            render_feedback_overlays(
                &mut out_frame,
                placement,
                interaction.state(),
                interaction.calculator(),
            );
            out_frame
        }
    }
}

fn render_feedback_overlays(
    out_frame: &mut DrawFrame,
    placement: &UiLayoutPhysicalPlacementModel,
    state: &DemoState,
    calculator: &CalculatorController,
) {
    let bounds = placement_bounds(placement);

    if let Some(bounds) = bounds {
        draw_scene_backdrop(out_frame, bounds);
        draw_scene_header(out_frame, bounds);
        draw_scene_cards(out_frame, placement, state);
        draw_status_area(out_frame, bounds, state);

        let mut ui_frame = UiFrame::new();
        calculator.render_panel(&mut ui_frame, bounds);
        append_ui_frame(out_frame, &ui_frame);
        draw_pointer_marker(out_frame, state);
    }
}

fn append_ui_frame(out_frame: &mut DrawFrame, ui_frame: &UiFrame) {
    for command in ui_frame.commands() {
        out_frame.push(command.clone());
    }
}

fn draw_hovered_feedback(out_frame: &mut DrawFrame, rect: prom_ui::layout::UiLayoutGeometryRect) {
    draw_outline(out_frame, rect, Color::rgb(102, 222, 255), 2);
}

fn draw_selected_feedback(out_frame: &mut DrawFrame, rect: prom_ui::layout::UiLayoutGeometryRect) {
    draw_outline(out_frame, rect, Color::rgb(255, 196, 77), 2);
}

fn draw_focused_feedback(out_frame: &mut DrawFrame, rect: prom_ui::layout::UiLayoutGeometryRect) {
    let x = rect.x() - 4;
    let y = rect.y() - 4;
    let w = rect.width() + 8;
    let h = rect.height() + 8;
    let thickness = 2;

    draw_outline(
        out_frame,
        prom_ui::layout::UiLayoutGeometryRect::new(x, y, w, h),
        Color::WHITE,
        thickness,
    );
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
        DispatchFeedback::IntentBuilt => Color::rgb(72, 140, 255),
        DispatchFeedback::Denied => Color::rgb(235, 72, 90),
    };

    out_frame.fill_rect(Rect::new(rect.x() + 5, rect.y() - 10, 20, 8), color);
}

fn placement_bounds(
    placement: &UiLayoutPhysicalPlacementModel,
) -> Option<prom_ui::layout::UiLayoutGeometryRect> {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for entry in placement.entries() {
        let rect = entry.final_rect();
        min_x = min_x.min(rect.x());
        min_y = min_y.min(rect.y());
        max_x = max_x.max(rect.x() + rect.width() as i32);
        max_y = max_y.max(rect.y() + rect.height() as i32);
    }

    if min_x == i32::MAX {
        return None;
    }

    Some(prom_ui::layout::UiLayoutGeometryRect::new(
        min_x - 28,
        min_y - 28,
        (max_x - min_x + 56) as u32,
        (max_y - min_y + 172) as u32,
    ))
}

fn calculator_scene_bounds(
    placement: &UiLayoutPhysicalPlacementModel,
) -> Option<prom_ui::layout::UiLayoutGeometryRect> {
    placement_bounds(placement).map(|bounds| {
        prom_ui::layout::UiLayoutGeometryRect::new(
            bounds.x(),
            bounds.y(),
            bounds.width().max(760),
            bounds.height().max(560),
        )
    })
}

fn draw_scene_backdrop(out_frame: &mut DrawFrame, bounds: prom_ui::layout::UiLayoutGeometryRect) {
    let canvas_width = bounds.width().max(760);
    let canvas_height = bounds.height().max(420);

    out_frame.fill_rect(
        Rect::new(bounds.x(), bounds.y(), canvas_width, canvas_height),
        Color::rgb(16, 19, 27),
    );
    out_frame.fill_rect(
        Rect::new(bounds.x(), bounds.y(), canvas_width, 36),
        Color::rgb(30, 36, 50),
    );
}

fn draw_scene_header(out_frame: &mut DrawFrame, bounds: prom_ui::layout::UiLayoutGeometryRect) {
    out_frame.draw_text(
        "Semantic UI demo slice",
        bounds.x() + 16,
        bounds.y() + 22,
        Color::WHITE,
    );
    out_frame.draw_text(
        "Hover, click, focus, and type to update the status bar below.",
        bounds.x() + 16,
        bounds.y() + 46,
        Color::rgb(205, 214, 230),
    );
}

fn draw_scene_cards(
    out_frame: &mut DrawFrame,
    placement: &UiLayoutPhysicalPlacementModel,
    state: &DemoState,
) {
    let Some(bounds) = placement_bounds(placement) else {
        return;
    };

    let left_card =
        prom_ui::layout::UiLayoutGeometryRect::new(bounds.x() + 16, bounds.y() + 64, 300, 176);
    let right_card =
        prom_ui::layout::UiLayoutGeometryRect::new(bounds.x() + 336, bounds.y() + 64, 300, 176);

    for (index, entry) in placement.entries().iter().enumerate() {
        let node_id = entry.source_render_node();
        let rect = entry.final_rect();
        let card = if index == 0 { left_card } else { right_card };
        let is_hovered = Some(node_id) == state.hovered_node;
        let is_selected = Some(node_id) == state.selected_node;
        let is_focused = Some(node_id) == state.focused_node;

        let fill = card_fill(node_id, is_hovered, is_selected);
        let outline = card_outline(node_id, is_hovered, is_selected);

        out_frame.fill_rect(
            Rect::new(card.x(), card.y(), card.width(), card.height()),
            fill,
        );
        draw_outline(out_frame, card, outline, 2);
        out_frame.fill_rect(
            Rect::new(card.x() + 14, card.y() + 12, card.width() - 28, 22),
            Color::rgb(24, 28, 38),
        );

        if is_focused {
            draw_focused_feedback(out_frame, card);
        }

        draw_scene_card_text(out_frame, card, node_id, state);
        draw_layout_preview(out_frame, card, rect);

        if is_selected {
            draw_selected_feedback(out_frame, card);
            draw_dispatch_feedback(out_frame, card, state.dispatch_feedback);
        } else if is_hovered {
            draw_hovered_feedback(out_frame, card);
        }
    }
}

fn draw_scene_card_text(
    out_frame: &mut DrawFrame,
    card: prom_ui::layout::UiLayoutGeometryRect,
    node_id: UiRenderNodeId,
    state: &DemoState,
) {
    let label = node_label(node_id);
    let state_label = if Some(node_id) == state.selected_node {
        "selected"
    } else if Some(node_id) == state.hovered_node {
        "hovered"
    } else if Some(node_id) == state.focused_node {
        "focused"
    } else {
        "idle"
    };

    out_frame.draw_text(label, card.x() + 14, card.y() + 28, Color::WHITE);
    out_frame.draw_text(
        format!("State: {}", state_label),
        card.x() + 14,
        card.y() + 52,
        Color::rgb(225, 232, 242),
    );
    out_frame.draw_text(
        format!("Render node {:?}", node_id),
        card.x() + 14,
        card.y() + 74,
        Color::rgb(190, 201, 219),
    );
}

fn draw_layout_preview(
    out_frame: &mut DrawFrame,
    card: prom_ui::layout::UiLayoutGeometryRect,
    rect: prom_ui::layout::UiLayoutGeometryRect,
) {
    let preview_width = rect.width().clamp(48, 180);
    let preview_height = rect.height().clamp(14, 28);

    out_frame.fill_rect(
        Rect::new(card.x() + 14, card.y() + 112, preview_width, preview_height),
        Color::rgb(70, 130, 180),
    );
    out_frame.draw_text(
        format!("Placement {}x{}", rect.width(), rect.height()),
        card.x() + 18,
        card.y() + 110,
        Color::WHITE,
    );
    out_frame.fill_rect(
        Rect::new(card.x() + 14, card.y() + 148, 64, 10),
        Color::rgb(92, 102, 122),
    );
}

fn draw_status_area(
    out_frame: &mut DrawFrame,
    bounds: prom_ui::layout::UiLayoutGeometryRect,
    state: &DemoState,
) {
    let canvas_width = bounds.width().max(760);
    let status =
        prom_ui::layout::UiLayoutGeometryRect::new(bounds.x(), bounds.y() + 264, canvas_width, 128);

    out_frame.fill_rect(
        Rect::new(status.x(), status.y(), status.width(), status.height()),
        Color::rgb(22, 26, 36),
    );
    draw_outline(out_frame, status, Color::rgb(84, 96, 126), 2);
    out_frame.fill_rect(
        Rect::new(status.x() + 14, status.y() + 12, status.width() - 28, 22),
        Color::rgb(31, 36, 48),
    );

    out_frame.draw_text(
        "Demo status",
        status.x() + 16,
        status.y() + 28,
        Color::WHITE,
    );

    out_frame.draw_text(
        format!(
            "Pointer: ({}, {})   Hover: {}   Selected: {}   Focused: {}",
            state.pointer_x,
            state.pointer_y,
            render_node_label(state.hovered_node),
            render_node_label(state.selected_node),
            render_node_label(state.focused_node)
        ),
        status.x() + 16,
        status.y() + 54,
        Color::rgb(228, 233, 241),
    );

    out_frame.draw_text(
        format!(
            "Keys pressed: {}   Last key: {}   Denied count: {}",
            state.key_press_count,
            state
                .last_key
                .map(|key| key.to_string())
                .unwrap_or_else(|| "none".to_string()),
            state.denied_count
        ),
        status.x() + 16,
        status.y() + 76,
        Color::rgb(196, 207, 223),
    );

    let key_bar_width = 48 + state.key_press_count.min(20) * 12;
    out_frame.fill_rect(
        Rect::new(status.x() + 16, status.y() + 100, key_bar_width, 10),
        Color::rgb(72, 140, 255),
    );
    out_frame.draw_text(
        "Key counter",
        status.x() + 18,
        status.y() + 98,
        Color::WHITE,
    );

    let denied_color = if state.denied_count > 0 {
        Color::rgb(235, 72, 90)
    } else {
        Color::rgb(92, 102, 122)
    };
    let denied_width = 24 + state.denied_count.min(8) * 14;
    out_frame.fill_rect(
        Rect::new(status.x() + 250, status.y() + 100, denied_width, 10),
        denied_color,
    );
    out_frame.draw_text(
        "Denied marker",
        status.x() + 252,
        status.y() + 98,
        Color::WHITE,
    );

    if matches!(state.dispatch_feedback, DispatchFeedback::Denied) {
        out_frame.fill_rect(
            Rect::new(status.x() + 420, status.y() + 94, 22, 18),
            Color::rgb(235, 72, 90),
        );
        out_frame.draw_text("!", status.x() + 426, status.y() + 107, Color::WHITE);
    }
}

fn draw_pointer_marker(out_frame: &mut DrawFrame, state: &DemoState) {
    let pointer_color = if state.is_pointer_down {
        Color::rgb(235, 72, 90)
    } else {
        Color::rgb(76, 201, 130)
    };

    out_frame.fill_rect(
        Rect::new(state.pointer_x - 4, state.pointer_y - 4, 8, 8),
        pointer_color,
    );
}

fn card_fill(node_id: UiRenderNodeId, is_hovered: bool, is_selected: bool) -> Color {
    if is_selected {
        Color::rgb(96, 74, 34)
    } else if is_hovered {
        Color::rgb(44, 84, 120)
    } else if node_id == UiRenderNodeId::new(1) {
        Color::rgb(38, 58, 94)
    } else {
        Color::rgb(38, 82, 64)
    }
}

fn card_outline(node_id: UiRenderNodeId, is_hovered: bool, is_selected: bool) -> Color {
    if is_selected {
        Color::rgb(255, 196, 77)
    } else if is_hovered {
        Color::rgb(102, 222, 255)
    } else if node_id == UiRenderNodeId::new(1) {
        Color::rgb(112, 146, 255)
    } else {
        Color::rgb(116, 210, 156)
    }
}

fn node_label(node_id: UiRenderNodeId) -> &'static str {
    if node_id == UiRenderNodeId::new(1) {
        "Primary region"
    } else if node_id == UiRenderNodeId::new(2) {
        "Action region"
    } else {
        "Demo region"
    }
}

fn render_node_label(node: Option<UiRenderNodeId>) -> String {
    node.map(node_label).unwrap_or("none").to_string()
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
        DemoInteraction::new_diagnostics()
    }

    fn test_calculator_interaction() -> DemoInteraction {
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
    fn escape_key_requests_exit_without_updating_state() {
        let mut interaction = test_interaction();
        let placement = test_placement();

        let events = vec![InputEvent::new(InputEventKind::KeyDown { key_code: 27 })];
        let control = interaction.apply_events(&events, &placement);

        assert_eq!(control, LoopControl::ExitRequested);
        assert_eq!(interaction.state.key_press_count, 0);
        assert_eq!(interaction.state.last_key, None);
        assert_eq!(interaction.state.dispatch_feedback, DispatchFeedback::None);
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
        let theme = ui_shell_kit::default_theme();

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
                if *color == Color::rgb(16, 19, 27)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::rgb(235, 72, 90)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::rgb(255, 196, 77)
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Demo status")
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Pointer:")
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Denied marker")
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == theme.muted_text
        )));
    }

    #[test]
    fn primary_calculator_scene_omits_demo_status_and_cards() {
        let placement = test_placement();
        let interaction = test_calculator_interaction();
        let theme = ui_shell_kit::default_theme();

        let frame = render_demo_frame(&placement, &interaction);
        let commands = frame.commands();

        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::WHITE
        )));
        assert!(commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == theme.muted_text
        )));
        assert!(!commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Demo status")
        )));
        assert!(!commands.iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Primary region")
        )));
    }
}
