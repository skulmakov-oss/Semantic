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
    #[cfg(test)]
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
        draw_digit_glyph(
            out_frame,
            UiLayoutGeometryRect::new(display.x() + 10, display.y() + 10, 20, 16),
            0,
            Color::WHITE,
        );
        let buttons = calculator_button_layout(panel);
        for (row, row_buttons) in buttons.iter().enumerate() {
            for (col, button) in row_buttons.iter().enumerate() {
                let rect = button_rect(panel, row, col);
                let is_hovered = self.state.hovered_button == Some(*button);
                let is_selected = self.state.selected_button == Some(*button);
                let is_focused = self.state.focused_button == Some(*button);

                let fill = if is_selected {
                    Color::rgb(255, 214, 96)
                } else if is_hovered {
                    Color::rgb(44, 84, 120)
                } else {
                    Color::rgb(38, 42, 56)
                };
                let label_panel = button_label_panel_color(*button, is_hovered, is_selected);
                let outline = if is_selected {
                    Color::rgb(255, 250, 210)
                } else if is_hovered {
                    Color::rgb(102, 222, 255)
                } else {
                    Color::rgb(84, 96, 126)
                };
                let glyph_color = button_glyph_color(*button, is_selected);

                out_frame.fill_rect(
                    Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
                    fill,
                );
                let inset = UiLayoutGeometryRect::new(
                    rect.x() + 6,
                    rect.y() + 3,
                    rect.width().saturating_sub(12),
                    rect.height().saturating_sub(6),
                );
                out_frame.fill_rect(
                    Rect::new(inset.x(), inset.y(), inset.width(), inset.height()),
                    label_panel,
                );
                draw_outline(
                    out_frame,
                    rect,
                    outline,
                    if is_selected {
                        3
                    } else if is_focused {
                        2
                    } else {
                        1
                    },
                );
                draw_button_glyph(out_frame, inset, *button, glyph_color);
                if is_selected {
                    draw_selection_spark(out_frame, rect, Color::rgb(255, 255, 235));
                }
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

        if let Some(button) = self.state.last_button {
            let badge = UiLayoutGeometryRect::new(display.x() + 154, display.y() + 1, 154, 30);
            out_frame.fill_rect(
                Rect::new(badge.x(), badge.y(), badge.width(), badge.height()),
                button_label_panel_color(button, false, true),
            );
            draw_outline(out_frame, badge, Color::rgb(255, 250, 210), 3);
            draw_button_glyph(
                out_frame,
                UiLayoutGeometryRect::new(badge.x() + 40, badge.y() + 6, 72, 20),
                button,
                button_glyph_color(button, true),
            );
        }
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

fn draw_selection_spark(out_frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color) {
    let x = rect.x() + rect.width() as i32 - 12;
    let y = rect.y() + 4;
    out_frame.fill_rect(Rect::new(x, y, 6, 2), color);
    out_frame.fill_rect(Rect::new(x + 2, y - 2, 2, 6), color);
}

fn draw_button_glyph(
    out_frame: &mut DrawFrame,
    rect: UiLayoutGeometryRect,
    button: CalculatorButton,
    color: Color,
) {
    match button {
        CalculatorButton::Digit(digit) => draw_digit_glyph(out_frame, rect, digit, color),
        CalculatorButton::Divide => draw_divide_glyph(out_frame, rect, color),
        CalculatorButton::Multiply => draw_multiply_glyph(out_frame, rect, color),
        CalculatorButton::Subtract => draw_subtract_glyph(out_frame, rect, color),
        CalculatorButton::Add => draw_add_glyph(out_frame, rect, color),
        CalculatorButton::Clear => draw_clear_glyph(out_frame, rect, color),
        CalculatorButton::Equals => draw_equals_glyph(out_frame, rect, color),
    }
}

fn button_label_panel_color(
    button: CalculatorButton,
    is_hovered: bool,
    is_selected: bool,
) -> Color {
    match (button, is_selected, is_hovered) {
        (CalculatorButton::Digit(_), true, _) => Color::rgb(175, 208, 255),
        (CalculatorButton::Digit(_), false, true) => Color::rgb(98, 140, 210),
        (CalculatorButton::Digit(_), false, false) => Color::rgb(72, 104, 160),

        (CalculatorButton::Divide, true, _) => Color::rgb(176, 241, 255),
        (CalculatorButton::Divide, false, true) => Color::rgb(98, 188, 204),
        (CalculatorButton::Divide, false, false) => Color::rgb(60, 128, 144),

        (CalculatorButton::Multiply, true, _) => Color::rgb(236, 206, 255),
        (CalculatorButton::Multiply, false, true) => Color::rgb(164, 122, 210),
        (CalculatorButton::Multiply, false, false) => Color::rgb(108, 78, 150),

        (CalculatorButton::Subtract, true, _) => Color::rgb(255, 224, 192),
        (CalculatorButton::Subtract, false, true) => Color::rgb(192, 142, 94),
        (CalculatorButton::Subtract, false, false) => Color::rgb(132, 92, 56),

        (CalculatorButton::Add, true, _) => Color::rgb(220, 255, 192),
        (CalculatorButton::Add, false, true) => Color::rgb(142, 196, 88),
        (CalculatorButton::Add, false, false) => Color::rgb(88, 132, 52),

        (CalculatorButton::Clear, true, _) => Color::rgb(255, 206, 214),
        (CalculatorButton::Clear, false, true) => Color::rgb(206, 92, 112),
        (CalculatorButton::Clear, false, false) => Color::rgb(140, 60, 76),

        (CalculatorButton::Equals, true, _) => Color::rgb(226, 220, 255),
        (CalculatorButton::Equals, false, true) => Color::rgb(146, 132, 210),
        (CalculatorButton::Equals, false, false) => Color::rgb(92, 86, 146),
    }
}

fn button_glyph_color(button: CalculatorButton, is_selected: bool) -> Color {
    if is_selected {
        match button {
            CalculatorButton::Digit(_) => Color::rgb(18, 22, 30),
            CalculatorButton::Divide
            | CalculatorButton::Multiply
            | CalculatorButton::Subtract
            | CalculatorButton::Add
            | CalculatorButton::Clear
            | CalculatorButton::Equals => Color::rgb(18, 22, 30),
        }
    } else {
        Color::WHITE
    }
}

fn draw_digit_glyph(
    out_frame: &mut DrawFrame,
    rect: UiLayoutGeometryRect,
    digit: u8,
    color: Color,
) {
    let segments = match digit {
        0 => [true, true, true, true, true, true, false],
        1 => [false, true, true, false, false, false, false],
        2 => [true, true, false, true, true, false, true],
        3 => [true, true, true, true, false, false, true],
        4 => [false, true, true, false, false, true, true],
        5 => [true, false, true, true, false, true, true],
        6 => [true, false, true, true, true, true, true],
        7 => [true, true, true, false, false, false, false],
        8 => [true, true, true, true, true, true, true],
        9 => [true, true, true, true, false, true, true],
        _ => [true, true, true, true, true, true, true],
    };

    let x = rect.x();
    let y = rect.y();
    let w = rect.width() as i32;
    let h = rect.height() as i32;
    let t_i32 = 2;
    let t_u32 = 2;
    let inner_w = (w - 4).max(0) as u32;
    let half_h = (h / 2 - 2).max(0) as u32;

    if segments[0] {
        out_frame.fill_rect(Rect::new(x + 2, y, inner_w, t_u32), color);
    }
    if segments[1] {
        out_frame.fill_rect(Rect::new(x + w - t_i32, y + 2, t_u32, half_h), color);
    }
    if segments[2] {
        out_frame.fill_rect(Rect::new(x + w - t_i32, y + (h / 2), t_u32, half_h), color);
    }
    if segments[3] {
        out_frame.fill_rect(Rect::new(x + 2, y + h - t_i32, inner_w, t_u32), color);
    }
    if segments[4] {
        out_frame.fill_rect(Rect::new(x, y + (h / 2), t_u32, half_h), color);
    }
    if segments[5] {
        out_frame.fill_rect(Rect::new(x, y + 2, t_u32, half_h), color);
    }
    if segments[6] {
        out_frame.fill_rect(Rect::new(x + 2, y + (h / 2) - 1, inner_w, t_u32), color);
    }
}

fn draw_add_glyph(out_frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color) {
    let cx = rect.x() + rect.width() as i32 / 2;
    let cy = rect.y() + rect.height() as i32 / 2;
    out_frame.fill_rect(Rect::new(cx - 7, cy - 1, 14, 2), color);
    out_frame.fill_rect(Rect::new(cx - 1, cy - 6, 2, 12), color);
}

fn draw_subtract_glyph(out_frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color) {
    let cy = rect.y() + rect.height() as i32 / 2;
    let width = rect.width().saturating_sub(8);
    out_frame.fill_rect(Rect::new(rect.x() + 4, cy - 1, width, 2), color);
}

fn draw_equals_glyph(out_frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color) {
    let cx = rect.x() + rect.width() as i32 / 2;
    out_frame.fill_rect(Rect::new(cx - 7, rect.y() + 5, 14, 2), color);
    out_frame.fill_rect(Rect::new(cx - 7, rect.y() + 10, 14, 2), color);
}

fn draw_divide_glyph(out_frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color) {
    let x = rect.x() + 18;
    let y = rect.y() + 3;
    for step in 0..7 {
        out_frame.fill_rect(Rect::new(x + step * 2, y + step * 2, 2, 2), color);
    }
    out_frame.fill_rect(Rect::new(rect.x() + 8, rect.y() + 2, 3, 3), color);
    out_frame.fill_rect(Rect::new(rect.x() + 20, rect.y() + 13, 3, 3), color);
}

fn draw_multiply_glyph(out_frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color) {
    let x = rect.x() + 10;
    let y = rect.y() + 4;
    for step in 0..6 {
        out_frame.fill_rect(Rect::new(x + step * 2, y + step * 2, 2, 2), color);
        out_frame.fill_rect(Rect::new(x + 10 - step * 2, y + step * 2, 2, 2), color);
    }
}

fn draw_clear_glyph(out_frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color) {
    let x = rect.x() + 4;
    let y = rect.y() + 3;
    let w = rect.width() - 8;
    let h = rect.height() - 6;
    out_frame.fill_rect(Rect::new(x, y, w, 2), color);
    out_frame.fill_rect(Rect::new(x, y + h as i32 - 2, w, 2), color);
    out_frame.fill_rect(Rect::new(x, y, 2, h), color);
    out_frame.fill_rect(Rect::new(x, y + 2, 2, h - 4), color);
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
                if text.contains("Display")
        )));
        let glyph_fill_count = frame
            .commands()
            .iter()
            .filter(|cmd| {
                matches!(
                    cmd,
                    prom_ui_runtime::DrawCommand::FillRect { color, .. }
                        if *color == Color::WHITE
                )
            })
            .count();
        assert!(
            glyph_fill_count >= 8,
            "expected visible glyph fill rects, got {glyph_fill_count}"
        );
    }
}
