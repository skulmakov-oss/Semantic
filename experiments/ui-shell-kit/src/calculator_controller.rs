use crate::{
    action::{UiAction, UiActionQueue},
    calculator_scene::{
        calculator_layout, hit_test_button, render_calculator_panel, render_calculator_scene,
        CalculatorButton, CalculatorLayout, CalculatorViewState,
    },
    event::{UiEvent, UiEventKind},
    focus::FocusRing,
    paint::UiFrame,
    theme::{default_theme, UiShellTheme},
};
use prom_ui::layout::UiLayoutGeometryRect;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CalculatorState {
    hovered_button: Option<CalculatorButton>,
    selected_button: Option<CalculatorButton>,
    last_button: Option<CalculatorButton>,
    press_count: u32,
    display_value: alloc::string::String,
    accumulator: Option<i64>,
    pending_operator: Option<CalculatorButton>,
    replace_display_on_next_digit: bool,
    error_state: bool,
}

#[derive(Debug, Clone)]
pub struct CalculatorController {
    state: CalculatorState,
    focus: FocusRing,
}

impl Default for CalculatorController {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorController {
    pub fn new() -> Self {
        Self {
            state: CalculatorState {
                display_value: "0".to_string(),
                ..CalculatorState::default()
            },
            focus: FocusRing::new(),
        }
    }

    pub fn display_text(&self) -> &str {
        &self.state.display_value
    }

    pub fn focus(&self) -> &FocusRing {
        &self.focus
    }

    pub fn handle_event(
        &mut self,
        event: UiEvent,
        scene_bounds: UiLayoutGeometryRect,
    ) -> UiActionQueue {
        let layout = calculator_layout(scene_bounds);
        match event.kind {
            UiEventKind::CloseRequested => UiActionQueue::new(),
            UiEventKind::PointerMoved { x, y } => {
                self.state.hovered_button = hit_test_button(&layout, x, y);
                UiActionQueue::new()
            }
            UiEventKind::PointerDown { x, y, .. } => {
                let pressed = hit_test_button(&layout, x, y);
                self.state.selected_button = pressed;
                self.focus.set(pressed);

                let mut actions = UiActionQueue::new();
                actions.push(UiAction::FocusChanged(pressed));

                if let Some(button) = pressed {
                    self.handle_button_press(button);
                    actions.push(UiAction::ButtonPressed(button));
                }

                actions
            }
            UiEventKind::PointerUp { .. } => {
                self.state.selected_button = None;
                UiActionQueue::new()
            }
        }
    }

    pub fn render(&self, frame: &mut UiFrame, scene_bounds: UiLayoutGeometryRect) {
        let layout = calculator_layout(scene_bounds);
        render_calculator_scene(
            frame,
            &layout,
            CalculatorViewState {
                value: self.display_text(),
                pending: self.pending_operator_text(),
                hovered_button: self.state.hovered_button,
                selected_button: self.state.selected_button,
                focused_button: self.focus.current(),
            },
            default_theme(),
        );
    }

    pub fn render_panel(&self, frame: &mut UiFrame, scene_bounds: UiLayoutGeometryRect) {
        let layout = calculator_layout(scene_bounds);
        render_calculator_panel(
            frame,
            &layout,
            CalculatorViewState {
                value: self.display_text(),
                pending: self.pending_operator_text(),
                hovered_button: self.state.hovered_button,
                selected_button: self.state.selected_button,
                focused_button: self.focus.current(),
            },
            default_theme(),
        );
    }

    pub fn render_with_theme(
        &self,
        frame: &mut UiFrame,
        layout: &CalculatorLayout,
        theme: UiShellTheme,
    ) {
        self.render_layout(frame, layout, theme);
    }

    fn render_layout(&self, frame: &mut UiFrame, layout: &CalculatorLayout, theme: UiShellTheme) {
        render_calculator_panel(
            frame,
            layout,
            CalculatorViewState {
                value: self.display_text(),
                pending: self.pending_operator_text(),
                hovered_button: self.state.hovered_button,
                selected_button: self.state.selected_button,
                focused_button: self.focus.current(),
            },
            theme,
        );
    }

    fn pending_operator_text(&self) -> Option<&'static str> {
        self.state.pending_operator.map(|operator| operator.label())
    }

    fn handle_button_press(&mut self, button: CalculatorButton) {
        self.state.last_button = Some(button);
        self.state.press_count = self.state.press_count.saturating_add(1);

        match button {
            CalculatorButton::Digit(digit) => self.push_digit(digit),
            CalculatorButton::Clear => self.clear(),
            CalculatorButton::Equals => self.evaluate(),
            CalculatorButton::Add
            | CalculatorButton::Subtract
            | CalculatorButton::Multiply
            | CalculatorButton::Divide => self.queue_operator(button),
        }
    }

    fn clear(&mut self) {
        self.state.display_value = "0".to_string();
        self.state.accumulator = None;
        self.state.pending_operator = None;
        self.state.replace_display_on_next_digit = false;
        self.state.error_state = false;
    }

    fn push_digit(&mut self, digit: u8) {
        if digit > 9 {
            return;
        }

        if self.state.error_state {
            self.clear();
        }

        if self.state.replace_display_on_next_digit || self.state.display_value == "0" {
            self.state.display_value = digit.to_string();
        } else {
            self.state.display_value.push(char::from(b'0' + digit));
        }

        self.state.replace_display_on_next_digit = false;
    }

    fn queue_operator(&mut self, operator: CalculatorButton) {
        if self.state.error_state {
            return;
        }

        if let Some(lhs) = self.state.accumulator {
            if let Some(previous_operator) = self.state.pending_operator {
                let Some(rhs) = self.current_display_value() else {
                    self.show_error();
                    return;
                };
                match apply_pending_operation(lhs, rhs, previous_operator) {
                    Some(result) => {
                        self.set_display_from_value(result);
                        self.state.accumulator = Some(result);
                    }
                    None => {
                        self.show_error();
                        return;
                    }
                }
            }
        } else if let Some(current) = self.current_display_value() {
            self.state.accumulator = Some(current);
        }

        self.state.pending_operator = Some(operator);
        self.state.replace_display_on_next_digit = true;
    }

    fn evaluate(&mut self) {
        if self.state.error_state {
            return;
        }

        let Some(lhs) = self.state.accumulator else {
            return;
        };
        let Some(operator) = self.state.pending_operator else {
            return;
        };
        let Some(rhs) = self.current_display_value() else {
            self.show_error();
            return;
        };

        match apply_pending_operation(lhs, rhs, operator) {
            Some(result) => {
                self.set_display_from_value(result);
                self.state.accumulator = Some(result);
                self.state.pending_operator = None;
                self.state.replace_display_on_next_digit = true;
            }
            None => self.show_error(),
        }
    }

    fn set_display_from_value(&mut self, value: i64) {
        self.state.display_value = value.to_string();
        self.state.error_state = false;
    }

    fn show_error(&mut self) {
        self.state.display_value = "ERR".to_string();
        self.state.accumulator = None;
        self.state.pending_operator = None;
        self.state.replace_display_on_next_digit = true;
        self.state.error_state = true;
    }

    fn current_display_value(&self) -> Option<i64> {
        if self.state.error_state {
            return None;
        }

        self.state.display_value.parse::<i64>().ok()
    }
}

fn apply_pending_operation(lhs: i64, rhs: i64, operator: CalculatorButton) -> Option<i64> {
    match operator {
        CalculatorButton::Add => lhs.checked_add(rhs),
        CalculatorButton::Subtract => lhs.checked_sub(rhs),
        CalculatorButton::Multiply => lhs.checked_mul(rhs),
        CalculatorButton::Divide => {
            if rhs == 0 {
                None
            } else {
                lhs.checked_div(rhs)
            }
        }
        _ => None,
    }
}
