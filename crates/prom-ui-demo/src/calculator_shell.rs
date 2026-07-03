//! PromUI-only calculator shell.
//!
//! This module renders a local calculator UI prototype and implements
//! UI-local calculator behavior for native interaction testing.
//! It does not execute Semantic code, does not call `smc`, and does not
//! claim to be the authoritative Semantic calculator logic.
//!
//! Calculator truth belongs to a future Semantic `.sm` logic slice.

#[path = "ui_shell.rs"]
pub(crate) mod ui_shell;

use self::ui_shell::{UiButtonState, UiButtonTone};
use prom_ui::layout::UiLayoutGeometryRect;
use prom_ui_runtime::DrawFrame;

#[cfg(test)]
use prom_ui_runtime::Color;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculatorButton {
    Digit(u8),
    Divide,
    Multiply,
    Subtract,
    Add,
    Clear,
    Equals,
}

impl CalculatorButton {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Digit(0) => "0",
            Self::Digit(1) => "1",
            Self::Digit(2) => "2",
            Self::Digit(3) => "3",
            Self::Digit(4) => "4",
            Self::Digit(5) => "5",
            Self::Digit(6) => "6",
            Self::Digit(7) => "7",
            Self::Digit(8) => "8",
            Self::Digit(9) => "9",
            Self::Digit(_) => "?",
            Self::Divide => "/",
            Self::Multiply => "*",
            Self::Subtract => "-",
            Self::Add => "+",
            Self::Clear => "C",
            Self::Equals => "=",
        }
    }

    #[cfg(test)]
    pub fn intent_text(self) -> String {
        match self {
            Self::Digit(digit) => format!("Digit({digit})"),
            Self::Divide => "Divide".to_string(),
            Self::Multiply => "Multiply".to_string(),
            Self::Subtract => "Subtract".to_string(),
            Self::Add => "Add".to_string(),
            Self::Clear => "Clear".to_string(),
            Self::Equals => "Equals".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CalculatorShellState {
    hovered_button: Option<CalculatorButton>,
    selected_button: Option<CalculatorButton>,
    focused_button: Option<CalculatorButton>,
    last_button: Option<CalculatorButton>,
    press_count: u32,
    display_value: String,
    accumulator: Option<i64>,
    pending_operator: Option<CalculatorButton>,
    replace_display_on_next_digit: bool,
    error_state: bool,
}

#[derive(Debug, Clone)]
pub struct CalculatorShell {
    state: CalculatorShellState,
}

impl Default for CalculatorShell {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorShell {
    pub fn new() -> Self {
        Self {
            state: CalculatorShellState {
                display_value: "0".to_string(),
                ..CalculatorShellState::default()
            },
        }
    }

    pub fn display_text(&self) -> &str {
        &self.state.display_value
    }

    pub fn handle_pointer_moved(&mut self, x: f64, y: f64, scene_bounds: UiLayoutGeometryRect) {
        self.state.hovered_button = hit_test_button(scene_bounds, x, y);
    }

    pub fn handle_pointer_down(&mut self, x: f64, y: f64, scene_bounds: UiLayoutGeometryRect) {
        let pressed = hit_test_button(scene_bounds, x, y);
        self.state.selected_button = pressed;
        self.state.focused_button = pressed;

        if let Some(button) = pressed {
            self.handle_button_press(button);
        }
    }

    pub fn handle_pointer_up(&mut self) {
        self.state.selected_button = None;
    }

    pub fn handle_button_press(&mut self, button: CalculatorButton) {
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

    fn pending_operator_text(&self) -> Option<&'static str> {
        self.state.pending_operator.map(|operator| operator.label())
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

    pub fn render(&self, out_frame: &mut DrawFrame, scene_bounds: UiLayoutGeometryRect) {
        self.render_scene(out_frame, scene_bounds);
    }

    pub fn render_scene(&self, out_frame: &mut DrawFrame, scene_bounds: UiLayoutGeometryRect) {
        let theme = ui_shell::default_theme();
        ui_shell::draw_app_background(out_frame, scene_bounds, theme);
        self.render_panel(out_frame, scene_bounds, theme);
    }

    pub(crate) fn render_panel(
        &self,
        out_frame: &mut DrawFrame,
        scene_bounds: UiLayoutGeometryRect,
        theme: ui_shell::UiShellTheme,
    ) {
        let panel = ui_shell::centered_rect(scene_bounds, 420, 420);
        ui_shell::draw_panel(out_frame, panel, theme);

        let display =
            UiLayoutGeometryRect::new(panel.x() + 24, panel.y() + 44, panel.width() - 48, 74);
        ui_shell::draw_display(
            out_frame,
            display,
            self.display_text(),
            self.pending_operator_text(),
            theme,
        );

        let buttons = calculator_button_layout(panel);
        for (row, row_buttons) in buttons.iter().enumerate() {
            for (col, button) in row_buttons.iter().enumerate() {
                let rect = button_rect(panel, row, col);
                let state = UiButtonState {
                    hovered: self.state.hovered_button == Some(*button),
                    pressed: self.state.selected_button == Some(*button),
                    focused: self.state.focused_button == Some(*button),
                };
                ui_shell::draw_button(
                    out_frame,
                    rect,
                    button.label(),
                    button_tone(*button),
                    state,
                    theme,
                );
            }
        }

        ui_shell::draw_text_line(
            out_frame,
            panel.x() + 24,
            panel.y() + 8,
            "Semantic bridge: not connected",
            2,
            theme.muted_text,
        );
    }
}

fn calculator_bounds(scene_bounds: UiLayoutGeometryRect) -> UiLayoutGeometryRect {
    ui_shell::centered_rect(scene_bounds, 420, 420)
}

fn calculator_button_layout(_panel: UiLayoutGeometryRect) -> [[CalculatorButton; 4]; 4] {
    [
        [
            CalculatorButton::Digit(7),
            CalculatorButton::Digit(8),
            CalculatorButton::Digit(9),
            CalculatorButton::Divide,
        ],
        [
            CalculatorButton::Digit(4),
            CalculatorButton::Digit(5),
            CalculatorButton::Digit(6),
            CalculatorButton::Multiply,
        ],
        [
            CalculatorButton::Digit(1),
            CalculatorButton::Digit(2),
            CalculatorButton::Digit(3),
            CalculatorButton::Subtract,
        ],
        [
            CalculatorButton::Clear,
            CalculatorButton::Digit(0),
            CalculatorButton::Equals,
            CalculatorButton::Add,
        ],
    ]
}

fn button_rect(panel: UiLayoutGeometryRect, row: usize, col: usize) -> UiLayoutGeometryRect {
    let button_width = 84;
    let button_height = 58;
    let gap_x = 12;
    let gap_y = 12;
    let start_x = panel.x() + 24;
    let start_y = panel.y() + 150;

    ui_shell::grid_cell(
        start_x,
        start_y,
        button_width as u32,
        button_height as u32,
        gap_x,
        gap_y,
        row,
        col,
    )
}

fn button_tone(button: CalculatorButton) -> UiButtonTone {
    match button {
        CalculatorButton::Digit(_) => UiButtonTone::Digit,
        CalculatorButton::Divide
        | CalculatorButton::Multiply
        | CalculatorButton::Subtract
        | CalculatorButton::Add => UiButtonTone::Operator,
        CalculatorButton::Clear => UiButtonTone::Action,
        CalculatorButton::Equals => UiButtonTone::Equals,
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

fn hit_test_button(scene_bounds: UiLayoutGeometryRect, x: f64, y: f64) -> Option<CalculatorButton> {
    let panel = calculator_bounds(scene_bounds);
    let button_rows = calculator_button_layout(panel);

    for (row, row_buttons) in button_rows.iter().enumerate() {
        for (col, button) in row_buttons.iter().enumerate() {
            let rect = button_rect(panel, row, col);
            let xi = x as i32;
            let yi = y as i32;
            if xi >= rect.x()
                && xi < rect.x() + rect.width() as i32
                && yi >= rect.y()
                && yi < rect.y() + rect.height() as i32
            {
                return Some(*button);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_button_labels_and_intents_are_stable() {
        assert_eq!(CalculatorButton::Digit(7).label(), "7");
        assert_eq!(CalculatorButton::Add.label(), "+");
        assert_eq!(CalculatorButton::Digit(7).intent_text(), "Digit(7)");
        assert_eq!(CalculatorButton::Equals.intent_text(), "Equals");
    }

    #[test]
    fn pointer_motion_and_press_update_button_state() {
        let mut shell = CalculatorShell::new();
        let scene_bounds = UiLayoutGeometryRect::new(0, 0, 760, 420);

        shell.handle_pointer_moved(210.0, 170.0, scene_bounds);
        assert_eq!(shell.state.hovered_button, Some(CalculatorButton::Digit(7)));

        shell.handle_pointer_down(210.0, 170.0, scene_bounds);
        assert_eq!(
            shell.state.selected_button,
            Some(CalculatorButton::Digit(7))
        );
        assert_eq!(shell.state.focused_button, Some(CalculatorButton::Digit(7)));
        assert_eq!(shell.state.last_button, Some(CalculatorButton::Digit(7)));
        assert_eq!(shell.state.press_count, 1);
        assert_eq!(shell.display_text(), "7");
    }

    #[test]
    fn render_contains_calculator_shell_and_bridge_notice() {
        let shell = CalculatorShell::new();
        let mut frame = DrawFrame::new();
        let scene_bounds = UiLayoutGeometryRect::new(0, 0, 760, 420);
        let panel = calculator_bounds(scene_bounds);
        let seven_button = button_rect(panel, 0, 0);
        let add_button = button_rect(panel, 3, 3);

        shell.render_scene(&mut frame, scene_bounds);

        assert!(!frame.is_empty());
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::WHITE
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { rect, color }
                if *color == Color::WHITE
                    && rect.x >= seven_button.x()
                    && rect.x < seven_button.x() + seven_button.width() as i32
                    && rect.y >= seven_button.y()
                    && rect.y < seven_button.y() + seven_button.height() as i32
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { rect, color }
                if *color == Color::WHITE
                    && rect.x >= add_button.x()
                    && rect.x < add_button.x() + add_button.width() as i32
                    && rect.y >= add_button.y()
                    && rect.y < add_button.y() + add_button.height() as i32
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::rgb(205, 216, 229)
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::rgb(53, 59, 74)
        )));
    }

    #[test]
    fn calculator_button_presses_update_display() {
        let mut shell = CalculatorShell::new();

        shell.handle_button_press(CalculatorButton::Digit(1));
        shell.handle_button_press(CalculatorButton::Digit(2));
        shell.handle_button_press(CalculatorButton::Add);
        shell.handle_button_press(CalculatorButton::Digit(3));
        shell.handle_button_press(CalculatorButton::Equals);

        assert_eq!(shell.display_text(), "15");
    }

    #[test]
    fn calculator_division_by_zero_sets_error_display() {
        let mut shell = CalculatorShell::new();

        shell.handle_button_press(CalculatorButton::Digit(8));
        shell.handle_button_press(CalculatorButton::Divide);
        shell.handle_button_press(CalculatorButton::Digit(0));
        shell.handle_button_press(CalculatorButton::Equals);

        assert_eq!(shell.display_text(), "ERR");
    }
}
