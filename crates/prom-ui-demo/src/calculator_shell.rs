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

    pub fn state(&self) -> &CalculatorShellState {
        &self.state
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
        let state = self.state();
        let panel = calculator_bounds(scene_bounds);

        out_frame.fill_rect(
            Rect::new(panel.x(), panel.y(), panel.width(), panel.height()),
            Color::rgb(18, 22, 30),
        );
        draw_outline(out_frame, panel, Color::rgb(108, 118, 148), 2);

        // Header band with icon-only status markers so the native backend can render them.
        out_frame.fill_rect(
            Rect::new(panel.x(), panel.y(), panel.width(), 28),
            Color::rgb(28, 34, 48),
        );
        draw_calculator_icon(out_frame, panel.x() + 14, panel.y() + 6);
        draw_bridge_indicator(out_frame, panel.x() + 286, panel.y() + 6);

        let display =
            UiLayoutGeometryRect::new(panel.x() + 14, panel.y() + 40, panel.width() - 28, 34);
        out_frame.fill_rect(
            Rect::new(display.x(), display.y(), display.width(), display.height()),
            Color::rgb(26, 31, 44),
        );
        draw_outline(out_frame, display, Color::rgb(72, 140, 255), 1);
        draw_digit_glyph(
            out_frame,
            display.x() + 10,
            display.y() + 8,
            3,
            0,
            Color::WHITE,
        );
        draw_last_intent_badge(out_frame, display, state.last_button);

        let buttons = calculator_button_layout(panel);
        for (row, row_buttons) in buttons.iter().enumerate() {
            for (col, button) in row_buttons.iter().enumerate() {
                let rect = button_rect(panel, row, col);
                let is_hovered = state.hovered_button == Some(*button);
                let is_selected = state.selected_button == Some(*button);
                let is_focused = state.focused_button == Some(*button);

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
                draw_button_glyph(out_frame, rect, *button, Color::WHITE);
            }
        }

        draw_press_count_indicator(out_frame, panel, state.press_count);
        draw_inert_calculation_badge(out_frame, panel);
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

fn draw_calculator_icon(out_frame: &mut DrawFrame, x: i32, y: i32) {
    let body = UiLayoutGeometryRect::new(x, y, 26, 16);
    out_frame.fill_rect(
        Rect::new(body.x(), body.y(), body.width(), body.height()),
        Color::rgb(58, 68, 90),
    );
    draw_outline(out_frame, body, Color::rgb(176, 188, 214), 1);
    out_frame.fill_rect(Rect::new(x + 4, y + 3, 18, 4), Color::rgb(176, 188, 214));
    out_frame.fill_rect(Rect::new(x + 4, y + 9, 4, 4), Color::rgb(176, 188, 214));
    out_frame.fill_rect(Rect::new(x + 10, y + 9, 4, 4), Color::rgb(176, 188, 214));
    out_frame.fill_rect(Rect::new(x + 16, y + 9, 4, 4), Color::rgb(176, 188, 214));
}

fn draw_bridge_indicator(out_frame: &mut DrawFrame, x: i32, y: i32) {
    let pill = UiLayoutGeometryRect::new(x, y, 36, 16);
    out_frame.fill_rect(
        Rect::new(pill.x(), pill.y(), pill.width(), pill.height()),
        Color::rgb(48, 52, 62),
    );
    draw_outline(out_frame, pill, Color::rgb(118, 124, 136), 1);
    out_frame.fill_rect(Rect::new(x + 6, y + 6, 24, 4), Color::rgb(118, 124, 136));
    out_frame.fill_rect(Rect::new(x + 16, y + 3, 4, 10), Color::rgb(235, 72, 90));
}

fn draw_inert_calculation_badge(out_frame: &mut DrawFrame, panel: UiLayoutGeometryRect) {
    let badge = UiLayoutGeometryRect::new(panel.x() + 274, panel.y() + 160, 56, 18);
    out_frame.fill_rect(
        Rect::new(badge.x(), badge.y(), badge.width(), badge.height()),
        Color::rgb(40, 44, 54),
    );
    draw_outline(out_frame, badge, Color::rgb(92, 102, 122), 1);
    out_frame.fill_rect(
        Rect::new(badge.x() + 7, badge.y() + 5, 42, 8),
        Color::rgb(92, 102, 122),
    );
    out_frame.fill_rect(
        Rect::new(badge.x() + 13, badge.y() + 8, 30, 2),
        Color::rgb(235, 72, 90),
    );
}

fn draw_press_count_indicator(
    out_frame: &mut DrawFrame,
    panel: UiLayoutGeometryRect,
    press_count: u32,
) {
    let clamped = press_count.min(10);
    for index in 0..clamped {
        let x = panel.x() + 14 + (index as i32 * 10);
        out_frame.fill_rect(
            Rect::new(x, panel.y() + 170, 6, 6),
            Color::rgb(72, 140, 255),
        );
    }
    if clamped == 0 {
        out_frame.fill_rect(
            Rect::new(panel.x() + 14, panel.y() + 170, 6, 6),
            Color::rgb(72, 140, 255),
        );
    }
}

fn draw_last_intent_badge(
    out_frame: &mut DrawFrame,
    display: UiLayoutGeometryRect,
    last_button: Option<CalculatorButton>,
) {
    let badge = UiLayoutGeometryRect::new(display.x() + 188, display.y() + 6, 132, 20);
    out_frame.fill_rect(
        Rect::new(badge.x(), badge.y(), badge.width(), badge.height()),
        Color::rgb(34, 40, 56),
    );
    draw_outline(out_frame, badge, Color::rgb(84, 96, 126), 1);

    if let Some(button) = last_button {
        draw_button_glyph(
            out_frame,
            UiLayoutGeometryRect::new(badge.x() + 6, badge.y() + 2, 18, 16),
            button,
            Color::WHITE,
        );
        out_frame.fill_rect(
            Rect::new(badge.x() + 28, badge.y() + 6, 94, 4),
            Color::rgb(225, 232, 242),
        );
        out_frame.fill_rect(
            Rect::new(badge.x() + 28, badge.y() + 12, 56, 2),
            Color::rgb(194, 206, 223),
        );
    } else {
        out_frame.fill_rect(
            Rect::new(badge.x() + 8, badge.y() + 6, 116, 4),
            Color::rgb(84, 96, 126),
        );
    }
}

fn draw_button_glyph(
    out_frame: &mut DrawFrame,
    rect: UiLayoutGeometryRect,
    button: CalculatorButton,
    color: Color,
) {
    let _button_label = button.label();
    let glyph_scale = 2;
    let glyph_width = 3 * glyph_scale;
    let glyph_height = 5 * glyph_scale;
    let glyph_x = rect.x() + ((rect.width() as i32 - glyph_width) / 2).max(0);
    let glyph_y = rect.y() + ((rect.height() as i32 - glyph_height) / 2).max(0);
    match button {
        CalculatorButton::Digit(digit) => {
            draw_digit_glyph(out_frame, glyph_x, glyph_y, glyph_scale, digit, color)
        }
        CalculatorButton::Divide => {
            draw_divide_glyph(out_frame, glyph_x, glyph_y, glyph_scale, color)
        }
        CalculatorButton::Multiply => {
            draw_multiply_glyph(out_frame, glyph_x, glyph_y, glyph_scale, color)
        }
        CalculatorButton::Subtract => {
            draw_minus_glyph(out_frame, glyph_x, glyph_y, glyph_scale, color)
        }
        CalculatorButton::Add => draw_plus_glyph(out_frame, glyph_x, glyph_y, glyph_scale, color),
        CalculatorButton::Clear => draw_c_glyph(out_frame, glyph_x, glyph_y, glyph_scale, color),
        CalculatorButton::Equals => {
            draw_equals_glyph(out_frame, glyph_x, glyph_y, glyph_scale, color)
        }
    }
}

fn draw_digit_glyph(
    out_frame: &mut DrawFrame,
    x: i32,
    y: i32,
    scale: i32,
    digit: u8,
    color: Color,
) {
    let pattern: [&str; 5] = match digit {
        0 => ["111", "101", "101", "101", "111"],
        1 => ["010", "110", "010", "010", "111"],
        2 => ["111", "001", "111", "100", "111"],
        3 => ["111", "001", "111", "001", "111"],
        4 => ["101", "101", "111", "001", "001"],
        5 => ["111", "100", "111", "001", "111"],
        6 => ["111", "100", "111", "101", "111"],
        7 => ["111", "001", "010", "010", "010"],
        8 => ["111", "101", "111", "101", "111"],
        9 => ["111", "101", "111", "001", "111"],
        _ => ["111", "111", "111", "111", "111"],
    };
    draw_pattern(out_frame, x, y, scale, &pattern, color);
}

fn draw_plus_glyph(out_frame: &mut DrawFrame, x: i32, y: i32, scale: i32, color: Color) {
    draw_pattern(
        out_frame,
        x,
        y,
        scale,
        &["010", "010", "111", "010", "010"],
        color,
    );
}

fn draw_minus_glyph(out_frame: &mut DrawFrame, x: i32, y: i32, scale: i32, color: Color) {
    draw_pattern(
        out_frame,
        x,
        y,
        scale,
        &["000", "000", "111", "000", "000"],
        color,
    );
}

fn draw_equals_glyph(out_frame: &mut DrawFrame, x: i32, y: i32, scale: i32, color: Color) {
    draw_pattern(
        out_frame,
        x,
        y,
        scale,
        &["000", "111", "000", "111", "000"],
        color,
    );
}

fn draw_divide_glyph(out_frame: &mut DrawFrame, x: i32, y: i32, scale: i32, color: Color) {
    draw_pattern(
        out_frame,
        x,
        y,
        scale,
        &["001", "010", "100", "000", "000"],
        color,
    );
}

fn draw_multiply_glyph(out_frame: &mut DrawFrame, x: i32, y: i32, scale: i32, color: Color) {
    draw_pattern(
        out_frame,
        x,
        y,
        scale,
        &["101", "010", "010", "010", "101"],
        color,
    );
}

fn draw_c_glyph(out_frame: &mut DrawFrame, x: i32, y: i32, scale: i32, color: Color) {
    draw_pattern(
        out_frame,
        x,
        y,
        scale,
        &["111", "100", "100", "100", "111"],
        color,
    );
}

fn draw_pattern(
    out_frame: &mut DrawFrame,
    x: i32,
    y: i32,
    scale: i32,
    pattern: &[&str; 5],
    color: Color,
) {
    for (row, bits) in pattern.iter().enumerate() {
        for (col, bit) in bits.chars().enumerate() {
            if bit == '1' {
                out_frame.fill_rect(
                    Rect::new(
                        x + (col as i32 * scale),
                        y + (row as i32 * scale),
                        scale as u32,
                        scale as u32,
                    ),
                    color,
                );
            }
        }
    }
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
        assert_eq!(
            shell.state().hovered_button,
            Some(CalculatorButton::Digit(7))
        );

        shell.handle_pointer_down(40.0, 490.0, scene_bounds);
        assert_eq!(
            shell.state().selected_button,
            Some(CalculatorButton::Digit(7))
        );
        assert_eq!(
            shell.state().focused_button,
            Some(CalculatorButton::Digit(7))
        );
        assert_eq!(shell.state().last_button, Some(CalculatorButton::Digit(7)));
        assert_eq!(shell.state().press_count, 1);
    }

    #[test]
    fn render_contains_calculator_shell_and_bridge_notice() {
        let shell = CalculatorShell::new();
        let mut frame = DrawFrame::new();
        let scene_bounds = UiLayoutGeometryRect::new(0, 0, 760, 420);

        shell.render(&mut frame, scene_bounds);

        assert!(!frame.is_empty());
        assert!(!frame
            .commands()
            .iter()
            .any(|cmd| matches!(cmd, prom_ui_runtime::DrawCommand::DrawText { .. })));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::WHITE
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::rgb(72, 140, 255)
        )));
        assert!(frame.commands().iter().any(|cmd| matches!(
            cmd,
            prom_ui_runtime::DrawCommand::FillRect { color, .. }
                if *color == Color::rgb(235, 72, 90)
        )));
    }
}
