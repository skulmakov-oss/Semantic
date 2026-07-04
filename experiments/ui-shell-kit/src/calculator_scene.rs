use crate::{paint::UiFrame, theme::UiButtonState, UiButtonTone, UiShellTheme};
use prom_ui::layout::UiLayoutGeometryRect;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculatorLayout {
    pub scene_bounds: UiLayoutGeometryRect,
    pub panel: UiLayoutGeometryRect,
    pub display: UiLayoutGeometryRect,
    pub buttons: alloc::vec::Vec<(CalculatorButton, UiLayoutGeometryRect)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalculatorViewState<'a> {
    pub value: &'a str,
    pub pending: Option<&'a str>,
    pub hovered_button: Option<CalculatorButton>,
    pub selected_button: Option<CalculatorButton>,
    pub focused_button: Option<CalculatorButton>,
}

pub fn calculator_layout(scene_bounds: UiLayoutGeometryRect) -> CalculatorLayout {
    let panel = crate::ui_shell::centered_rect(scene_bounds, 500, 500);
    let display = UiLayoutGeometryRect::new(panel.x() + 24, panel.y() + 92, panel.width() - 48, 90);
    let mut buttons = alloc::vec::Vec::new();
    let order = [
        CalculatorButton::Digit(7),
        CalculatorButton::Digit(8),
        CalculatorButton::Digit(9),
        CalculatorButton::Divide,
        CalculatorButton::Digit(4),
        CalculatorButton::Digit(5),
        CalculatorButton::Digit(6),
        CalculatorButton::Multiply,
        CalculatorButton::Digit(1),
        CalculatorButton::Digit(2),
        CalculatorButton::Digit(3),
        CalculatorButton::Subtract,
        CalculatorButton::Clear,
        CalculatorButton::Digit(0),
        CalculatorButton::Equals,
        CalculatorButton::Add,
    ];

    for (index, button) in order.iter().copied().enumerate() {
        let row = index / 4;
        let col = index % 4;
        buttons.push((button, button_rect(panel, row, col)));
    }

    CalculatorLayout {
        scene_bounds,
        panel,
        display,
        buttons,
    }
}

pub fn hit_test_button(layout: &CalculatorLayout, x: f64, y: f64) -> Option<CalculatorButton> {
    let xi = x as i32;
    let yi = y as i32;
    for (button, rect) in &layout.buttons {
        if xi >= rect.x()
            && xi < rect.x() + rect.width() as i32
            && yi >= rect.y()
            && yi < rect.y() + rect.height() as i32
        {
            return Some(*button);
        }
    }
    None
}

pub fn render_calculator_scene(
    frame: &mut UiFrame,
    layout: &CalculatorLayout,
    view: CalculatorViewState<'_>,
    theme: UiShellTheme,
) {
    crate::ui_shell::draw_app_background(frame, layout.scene_bounds, theme);
    render_calculator_panel(frame, layout, view, theme);
}

pub fn render_calculator_panel(
    frame: &mut UiFrame,
    layout: &CalculatorLayout,
    view: CalculatorViewState<'_>,
    theme: UiShellTheme,
) {
    crate::ui_shell::draw_panel(frame, layout.panel, theme);
    crate::ui_shell::draw_scene_header(frame, layout.panel, "UI KIT", "PREMIUM", "LIVE", theme);
    crate::ui_shell::draw_display(frame, layout.display, view.value, view.pending, theme);

    for (button, rect) in &layout.buttons {
        let state = UiButtonState {
            hovered: view.hovered_button == Some(*button),
            pressed: view.selected_button == Some(*button),
            focused: view.focused_button == Some(*button),
        };
        crate::ui_shell::draw_button(
            frame,
            *rect,
            button.label(),
            button_tone(*button),
            state,
            theme,
        );
    }
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

fn button_rect(panel: UiLayoutGeometryRect, row: usize, col: usize) -> UiLayoutGeometryRect {
    let button_width = 90;
    let button_height = 60;
    let gap_x = 12;
    let gap_y = 12;
    let start_x = panel.x() + 24;
    let start_y = panel.y() + 212;

    crate::ui_shell::grid_cell(
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
