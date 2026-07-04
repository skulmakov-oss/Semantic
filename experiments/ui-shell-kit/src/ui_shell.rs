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
        app_bg: Color::rgb(8, 11, 20),
        panel_bg: Color::rgb(18, 23, 36),
        panel_outline: Color::rgb(88, 104, 160),
        display_bg: Color::rgb(13, 18, 31),
        display_outline: Color::rgb(124, 164, 255),
        button_bg: Color::rgb(30, 37, 55),
        button_hovered: Color::rgb(44, 54, 77),
        button_pressed: Color::rgb(118, 96, 49),
        button_outline: Color::rgb(79, 93, 129),
        text: Color::rgb(245, 248, 255),
        muted_text: Color::rgb(173, 184, 214),
        accent: Color::rgb(88, 196, 255),
        danger: Color::rgb(255, 120, 145),
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
        Color::rgb(21, 29, 45),
    );
    frame.fill_rect(
        Rect::new(bounds.x(), bounds.y() + 6, bounds.width(), 1),
        theme.accent,
    );
    frame.fill_rect(
        Rect::new(
            bounds.x() + 18,
            bounds.y() + (bounds.height() as i32 - 24),
            bounds.width() - 36,
            2,
        ),
        Color::rgb(21, 29, 45),
    );
}

pub fn draw_panel(frame: &mut DrawFrame, rect: UiLayoutGeometryRect, theme: UiShellTheme) {
    frame.fill_rect(
        Rect::new(rect.x() + 10, rect.y() + 12, rect.width(), rect.height()),
        Color::rgb(6, 8, 14),
    );
    frame.fill_rect(
        Rect::new(rect.x() + 4, rect.y() + 5, rect.width(), rect.height()),
        Color::rgb(10, 13, 22),
    );
    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
        theme.panel_bg,
    );
    draw_outline(frame, rect, theme.panel_outline, 2);
    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), 30),
        Color::rgb(25, 32, 50),
    );
    frame.fill_rect(
        Rect::new(rect.x(), rect.y() + 30, rect.width(), 2),
        theme.accent,
    );
    frame.fill_rect(
        Rect::new(rect.x() + 16, rect.y() + 40, rect.width() - 32, 1),
        Color::rgb(40, 49, 73),
    );
}

pub fn draw_scene_header(
    frame: &mut DrawFrame,
    panel: UiLayoutGeometryRect,
    title: &str,
    subtitle: &str,
    badge: &str,
    theme: UiShellTheme,
) {
    let header_left = panel.x() + 20;
    let header_top = panel.y() + 18;
    let title_scale = 4;
    let subtitle_scale = 2;
    draw_text_line(
        frame,
        header_left,
        header_top,
        title,
        title_scale,
        theme.text,
    );
    draw_text_line(
        frame,
        header_left,
        header_top + 28,
        subtitle,
        subtitle_scale,
        theme.muted_text,
    );

    let badge_width = measure_text_width(badge, 2) + 18;
    let badge_height = 18;
    let badge_x = panel.x() + panel.width() as i32 - badge_width - 20;
    let badge_y = panel.y() + 18;
    let badge_rect = UiLayoutGeometryRect::new(badge_x, badge_y, badge_width as u32, badge_height);
    draw_badge(frame, badge_rect, badge, theme);
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
        Rect::new(rect.x() + 4, rect.y() + 5, rect.width(), rect.height()),
        Color::rgb(4, 7, 13),
    );
    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
        theme.display_bg,
    );
    draw_outline(frame, rect, theme.display_outline, 2);
    frame.fill_rect(Rect::new(rect.x(), rect.y(), rect.width(), 3), theme.accent);
    frame.fill_rect(
        Rect::new(rect.x() + 1, rect.y() + 3, rect.width() - 2, 1),
        Color::rgb(32, 44, 70),
    );
    frame.fill_rect(
        Rect::new(inner.x(), inner.y(), inner.width(), 1),
        Color::rgb(34, 44, 65),
    );

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
    frame.fill_rect(
        Rect::new(rect.x() + 3, rect.y() + 4, rect.width(), rect.height()),
        Color::rgb(6, 8, 14),
    );
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
    frame.fill_rect(
        Rect::new(rect.x() + 1, rect.y() + 1, rect.width() - 2, 2),
        button_sheen(tone),
    );
    frame.fill_rect(
        Rect::new(
            rect.x() + 1,
            rect.y() + rect.height() as i32 - 3,
            rect.width() - 2,
            2,
        ),
        button_shadow(tone),
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

fn draw_badge(frame: &mut DrawFrame, rect: UiLayoutGeometryRect, label: &str, theme: UiShellTheme) {
    frame.fill_rect(
        Rect::new(rect.x(), rect.y(), rect.width(), rect.height()),
        Color::rgb(25, 34, 53),
    );
    draw_outline(frame, rect, theme.accent, 1);
    frame.fill_rect(
        Rect::new(rect.x() + 1, rect.y() + 1, rect.width() - 2, 2),
        Color::rgb(46, 62, 92),
    );
    let text_scale = 2;
    let text_width = measure_text_width(label, text_scale);
    let text_x = rect.x() + ((rect.width() as i32 - text_width) / 2).max(0);
    let text_y = rect.y() + ((rect.height() as i32 - (7 * text_scale)) / 2).max(0);
    draw_text_line(frame, text_x, text_y, label, text_scale, theme.text);
}

fn button_sheen(tone: UiButtonTone) -> Color {
    match tone {
        UiButtonTone::Digit => Color::rgb(60, 72, 100),
        UiButtonTone::Operator => Color::rgb(72, 126, 172),
        UiButtonTone::Action => Color::rgb(154, 88, 106),
        UiButtonTone::Equals => Color::rgb(110, 160, 128),
    }
}

fn button_shadow(tone: UiButtonTone) -> Color {
    match tone {
        UiButtonTone::Digit => Color::rgb(18, 22, 32),
        UiButtonTone::Operator => Color::rgb(16, 22, 34),
        UiButtonTone::Action => Color::rgb(28, 17, 22),
        UiButtonTone::Equals => Color::rgb(16, 26, 22),
    }
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
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100, 0b00100,
        ],
        _ => [
            0b11111, 0b10001, 0b00100, 0b00100, 0b00100, 0b10001, 0b11111,
        ],
    }
}
