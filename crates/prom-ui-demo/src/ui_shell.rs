use prom_ui::layout::UiLayoutGeometryRect;
use prom_ui_runtime::{Color, DrawFrame, Rect};

#[derive(Debug, Clone, Copy)]
pub struct UiShellTheme {
    pub app_bg: Color,
    pub panel_bg: Color,
    pub panel_outline: Color,
    pub display_bg: Color,
    pub display_outline: Color,
    pub button_bg: Color,
    pub button_hovered: Color,
    pub button_pressed: Color,
    pub button_outline: Color,
    pub text: Color,
    pub muted_text: Color,
    pub accent: Color,
    pub danger: Color,
}

#[derive(Debug, Clone, Copy)]
pub enum UiButtonTone {
    Digit,
    Operator,
    Action,
    Equals,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiButtonState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

pub fn default_theme() -> UiShellTheme {
    UiShellTheme {
        app_bg: Color::rgb(28, 31, 41),
        panel_bg: Color::rgb(53, 59, 74),
        panel_outline: Color::rgb(159, 170, 194),
        display_bg: Color::rgb(77, 84, 101),
        display_outline: Color::rgb(140, 182, 255),
        button_bg: Color::rgb(138, 165, 207),
        button_hovered: Color::rgb(159, 194, 233),
        button_pressed: Color::rgb(242, 223, 154),
        button_outline: Color::rgb(125, 133, 154),
        text: Color::WHITE,
        muted_text: Color::rgb(205, 216, 229),
        accent: Color::rgb(176, 210, 149),
        danger: Color::rgb(208, 128, 143),
    }
}

pub fn centered_rect(
    parent: UiLayoutGeometryRect,
    width: u32,
    height: u32,
) -> UiLayoutGeometryRect {
    let width = width.min(parent.width());
    let height = height.min(parent.height());
    let x = parent.x() + ((parent.width() as i32 - width as i32) / 2).max(0);
    let y = parent.y() + ((parent.height() as i32 - height as i32) / 2).max(0);
    UiLayoutGeometryRect::new(x, y, width, height)
}

pub fn inset_rect(rect: UiLayoutGeometryRect, inset: i32) -> UiLayoutGeometryRect {
    let x = rect.x() + inset;
    let y = rect.y() + inset;
    let width = rect.width().saturating_sub((inset.max(0) as u32) * 2);
    let height = rect.height().saturating_sub((inset.max(0) as u32) * 2);
    UiLayoutGeometryRect::new(x, y, width, height)
}

#[allow(clippy::too_many_arguments)]
pub fn grid_cell(
    origin_x: i32,
    origin_y: i32,
    cell_w: u32,
    cell_h: u32,
    gap_x: i32,
    gap_y: i32,
    row: usize,
    col: usize,
) -> UiLayoutGeometryRect {
    UiLayoutGeometryRect::new(
        origin_x + (col as i32 * (cell_w as i32 + gap_x)),
        origin_y + (row as i32 * (cell_h as i32 + gap_y)),
        cell_w,
        cell_h,
    )
}

pub fn draw_app_background(
    frame: &mut DrawFrame,
    bounds: UiLayoutGeometryRect,
    theme: UiShellTheme,
) {
    frame.fill_rect(
        Rect::new(bounds.x(), bounds.y(), bounds.width(), bounds.height()),
        theme.app_bg,
    );
    frame.fill_rect(
        Rect::new(bounds.x(), bounds.y(), bounds.width(), 6),
        Color::rgb(40, 45, 58),
    );
}

pub fn draw_panel(frame: &mut DrawFrame, rect: UiLayoutGeometryRect, theme: UiShellTheme) {
    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
        theme.panel_bg,
    );
    draw_outline(frame, rect, theme.panel_outline, 2);
    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), 26),
        Color::rgb(64, 71, 88),
    );
}

pub fn draw_display(
    frame: &mut DrawFrame,
    rect: UiLayoutGeometryRect,
    value: &str,
    pending: Option<&str>,
    theme: UiShellTheme,
) {
    let inner = inset_rect(rect, 8);

    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
        theme.display_bg,
    );
    draw_outline(frame, rect, theme.display_outline, 2);

    let value_scale = display_scale(inner, value, 6);
    let value_y = inner.y() + ((inner.height() as i32 - (7 * value_scale)) / 2).max(0);
    let value_width = measure_text_width(value, value_scale);
    let value_x = (inner.x() + inner.width() as i32 - value_width - 4).max(inner.x() + 4);
    draw_text_line(frame, value_x, value_y, value, value_scale, theme.text);

    if let Some(pending) = pending {
        let pending_scale = 2;
        let pending_width = measure_text_width(pending, pending_scale);
        let pending_x = inner.x() + inner.width() as i32 - pending_width - 4;
        let pending_y = inner.y() + 4;
        draw_text_line(
            frame,
            pending_x,
            pending_y,
            pending,
            pending_scale,
            theme.muted_text,
        );
    }
}

pub fn draw_button(
    frame: &mut DrawFrame,
    rect: UiLayoutGeometryRect,
    label: &str,
    tone: UiButtonTone,
    state: UiButtonState,
    theme: UiShellTheme,
) {
    let fill = if state.pressed {
        theme.button_pressed
    } else if state.hovered {
        theme.button_hovered
    } else {
        match tone {
            UiButtonTone::Digit => theme.button_bg,
            UiButtonTone::Operator => theme.accent,
            UiButtonTone::Action => theme.danger,
            UiButtonTone::Equals => theme.button_pressed,
        }
    };

    let outline = if state.focused || state.pressed {
        theme.display_outline
    } else if state.hovered {
        theme.text
    } else {
        theme.button_outline
    };

    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
        fill,
    );
    draw_outline(frame, rect, outline, if state.focused { 2 } else { 1 });

    let text_scale = if label.len() > 1 { 3 } else { 4 };
    let text_width = measure_text_width(label, text_scale);
    let text_x = rect.x() + ((rect.width() as i32 - text_width) / 2).max(0);
    let text_y = rect.y() + ((rect.height() as i32 - (7 * text_scale)) / 2).max(0);
    let text_color = if state.pressed {
        Color::BLACK
    } else {
        Color::WHITE
    };
    draw_text_line(frame, text_x, text_y, label, text_scale, text_color);
}

pub(crate) fn draw_text_line(
    frame: &mut DrawFrame,
    x: i32,
    y: i32,
    text: &str,
    scale: i32,
    color: Color,
) {
    let mut cursor_x = x;
    for ch in text.chars() {
        if ch == ' ' {
            cursor_x += (4 * scale) / 2 + scale;
            continue;
        }

        draw_glyph(frame, cursor_x, y, ch, scale, color);
        cursor_x += glyph_advance(scale);
    }
}

fn draw_outline(frame: &mut DrawFrame, rect: UiLayoutGeometryRect, color: Color, thickness: u32) {
    let x = rect.x();
    let y = rect.y();
    let w = rect.width();
    let h = rect.height();
    let thickness_i32 = thickness as i32;

    frame.fill_rect(Rect::new(x, y, w, thickness), color);
    frame.fill_rect(
        Rect::new(x, y + (h as i32) - thickness_i32, w, thickness),
        color,
    );
    frame.fill_rect(Rect::new(x, y, thickness, h), color);
    frame.fill_rect(
        Rect::new(x + (w as i32) - thickness_i32, y, thickness, h),
        color,
    );
}

fn display_scale(rect: UiLayoutGeometryRect, text: &str, max_scale: i32) -> i32 {
    let width_scale = if text.is_empty() {
        max_scale
    } else {
        ((rect.width() as i32 - 24) / measure_text_width(text, 1)).max(1)
    };
    let height_scale = ((rect.height() as i32 - 12) / 7).max(1);
    width_scale.min(height_scale).min(max_scale).max(1)
}

fn measure_text_width(text: &str, scale: i32) -> i32 {
    if text.is_empty() {
        return 0;
    }

    let chars = text.chars().count() as i32;
    chars * glyph_advance(scale)
}

fn glyph_advance(scale: i32) -> i32 {
    6 * scale
}

fn draw_glyph(frame: &mut DrawFrame, x: i32, y: i32, ch: char, scale: i32, color: Color) {
    let rows = glyph_rows(ch.to_ascii_uppercase());
    for (row_index, row_bits) in rows.iter().enumerate() {
        let row_index = row_index as i32;
        for col in 0..5 {
            if row_bits & (1 << (4 - col)) == 0 {
                continue;
            }

            let px = x + (col * scale);
            let py = y + (row_index * scale);
            frame.fill_rect(Rect::new(px, py, scale as u32, scale as u32), color);
        }
    }
}

fn glyph_rows(ch: char) -> [u8; 7] {
    match ch {
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '*' => [
            0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b00000, 0b00000,
        ],
        '/' => [
            0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
        ],
        '=' => [
            0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        _ => [
            0b11111, 0b10001, 0b00100, 0b00100, 0b00100, 0b10001, 0b11111,
        ],
    }
}
