pub use prom_ui_runtime::{Color, Rect};
use prom_ui_runtime::{DrawCommand, DrawFrame};

#[derive(Debug, Default)]
pub struct UiFrame {
    inner: DrawFrame,
}

impl UiFrame {
    pub fn new() -> Self {
        Self {
            inner: DrawFrame::new(),
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.inner.clear(color);
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.inner.fill_rect(rect, color);
    }

    pub fn draw_text(
        &mut self,
        text: impl Into<alloc::string::String>,
        x: i32,
        y: i32,
        color: Color,
    ) {
        self.inner.draw_text(text, x, y, color);
    }

    pub fn push(&mut self, command: DrawCommand) {
        self.inner.push(command);
    }

    pub fn commands(&self) -> &[DrawCommand] {
        self.inner.commands()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn into_draw_frame(self) -> DrawFrame {
        self.inner
    }
}

impl core::ops::Deref for UiFrame {
    type Target = DrawFrame;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::DerefMut for UiFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<DrawFrame> for UiFrame {
    fn from(inner: DrawFrame) -> Self {
        Self { inner }
    }
}

impl From<UiFrame> for DrawFrame {
    fn from(frame: UiFrame) -> Self {
        frame.inner
    }
}
