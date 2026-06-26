//! PromUI-only calculator shell.
//!
//! This module renders an inert calculator interface and captures UI-local
//! button intent preview. It does not implement calculator semantics, does not
//! execute Semantic code, and does not call `smc`.
//!
//! Calculator truth belongs to a future Semantic `.sm` logic slice.

use prom_ui::layout::UiLayoutGeometryRect;
use prom_ui_runtime::{Color, DrawFrame, Rect};

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
}

#[derive(Debug, Clone, Default)]
pub struct CalculatorShell {
    state: CalculatorShellState,
}

impl CalculatorShell {
    pub fn new() -> Self {
        Self {
            state: CalculatorShellState::default(),
        }
    }

    pub fn handle_pointer_moved(&mut self, x: f64, y: f64, scene_bounds: UiLayoutGeometryRect) {
        self.state.hovered_button = hit_test_button(scene_bounds, x, y);
    }

    pub fn handle_pointer_down(&mut self, x: f64, y: f64, scene_bounds: UiLayoutGeometryRect) {
        let pressed = hit_test_button(scene_bounds, x, y);
        self.state.selected_button = pressed;
        self.state.focused_button = pressed;

        if let Some(button) = pressed {
            self.state.last_button = Some(button);
            self.state.press_count += 1;
        }
    }

    pub fn handle_pointer_up(&mut self) {}

    pub fn render(&self, out_frame: &mut DrawFrame, scene_bounds: UiLayoutGeometryRect) {
        let panel = calculator_bounds(scene_bounds);

        out_frame.fill_rect(
            Rect::new(panel.x(), panel.y(), panel.width(), panel.height()),
            Color::rgb(18, 22, 30),
        );
        draw_outline(out_frame, panel, Color::rgb(108, 118, 148), 2);
        out_frame.fill_rect(
            Rect::new(panel.x(), panel.y(), panel.width(), 28),
            Color::rgb(28, 34, 48),
        );
        out_frame.draw_text("Calculator", panel.x() + 14, panel.y() + 18, Color::WHITE);
        out_frame.draw_text(
            "Semantic bridge: not connected",
            panel.x() + 122,
            panel.y() + 18,
            Color::rgb(194, 206, 223),
        );

        let display =
            UiLayoutGeometryRect::new(panel.x() + 14, panel.y() + 40, panel.width() - 28, 34);
        out_frame.fill_rect(
            Rect::new(display.x(), display.y(), display.width(), display.height()),
            Color::rgb(26, 31, 44),
        );
        draw_outline(out_frame, display, Color::rgb(72, 140, 255), 1);
        out_frame.draw_text(
            "Display",
            display.x() + 10,
            display.y() + 12,
            Color::rgb(194, 206, 223),
        );
        out_frame.draw_text("0", display.x() + 10, display.y() + 27, Color::WHITE);
        out_frame.draw_text(
            format!(
                "Last calculator intent: {}",
                self.state
                    .last_button
                    .map(CalculatorButton::intent_text)
                    .unwrap_or_else(|| "none".to_string())
            ),
            display.x() + 98,
            display.y() + 27,
            Color::rgb(225, 232, 242),
        );

        let buttons = calculator_button_layout(panel);
        for (row, row_buttons) in buttons.iter().enumerate() {
            for (col, button) in row_buttons.iter().enumerate() {
                let rect = button_rect(panel, row, col);
                let is_hovered = self.state.hovered_button == Some(*button);
                let is_selected = self.state.selected_button == Some(*button);
                let is_focused = self.state.focused_button == Some(*button);

                let fill = if is_selected {
                    Color::rgb(96, 74, 34)
                } else if is_hovered {
                    Color::rgb(44, 84, 120)
                } else {
                    Color::rgb(38, 42, 56)
                };
                let outline = if is_selected {
                    Color::rgb(255, 196, 77)
                } else if is_hovered {
                    Color::rgb(102, 222, 255)
                } else {
                    Color::rgb(84, 96, 126)
                };

                out_frame.fill_rect(
                    Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
                    fill,
                );
                draw_outline(out_frame, rect, outline, if is_focused { 2 } else { 1 });
                out_frame.draw_text(button.label(), rect.x() + 20, rect.y() + 14, Color::WHITE);
            }
        }

        out_frame.draw_text(
            format!("Press count: {}", self.state.press_count),
            panel.x() + 14,
            panel.y() + panel.height() as i32 - 18,
            Color::rgb(194, 206, 223),
        );
        out_frame.draw_text(
            "Calculation: inert",
            panel.x() + 160,
            panel.y() + panel.height() as i32 - 18,
            Color::rgb(225, 232, 242),
        );
    }
}

fn calculator_bounds(scene_bounds: UiLayoutGeometryRect) -> UiLayoutGeometryRect {
    UiLayoutGeometryRect::new(scene_bounds.x() + 16, scene_bounds.y() + 404, 344, 188)
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
    let button_width = 74;
    let button_height = 20;
    let gap_x = 6;
    let gap_y = 4;
    let start_x = panel.x() + 12;
    let start_y = panel.y() + 76;

    UiLayoutGeometryRect::new(
        start_x + (col as i32 * (button_width + gap_x)),
        start_y + (row as i32 * (button_height + gap_y)),
        button_width as u32,
        button_height as u32,
    )
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

fn draw_outline(
    out_frame: &mut DrawFrame,
    rect: UiLayoutGeometryRect,
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

        shell.handle_pointer_moved(40.0, 490.0, scene_bounds);
        assert_eq!(shell.state.hovered_button, Some(CalculatorButton::Digit(7)));

        shell.handle_pointer_down(40.0, 490.0, scene_bounds);
        assert_eq!(
            shell.state.selected_button,
            Some(CalculatorButton::Digit(7))
        );
        assert_eq!(shell.state.focused_button, Some(CalculatorButton::Digit(7)));
        assert_eq!(shell.state.last_button, Some(CalculatorButton::Digit(7)));
        assert_eq!(shell.state.press_count, 1);
    }

    #[test]
    fn render_contains_calculator_shell_and_bridge_notice() {
        let shell = CalculatorShell::new();
        let mut frame = DrawFrame::new();
        let scene_bounds = UiLayoutGeometryRect::new(0, 0, 760, 420);

        shell.render(&mut frame, scene_bounds);

        assert!(!frame.is_empty());
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Calculator")
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Semantic bridge: not connected")
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text.contains("Last calculator intent")
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::DrawText { text, .. }
                if text == "7" || text == "+" || text == "C"
        )));
    }
}
