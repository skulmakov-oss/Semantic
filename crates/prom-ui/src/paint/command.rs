use crate::layout::UiRect;
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl UiColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

/// Represents a deterministic, backend-agnostic drawing instruction.
/// This acts as Boundary 3: The Visible Static UI boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDrawCommand {
    /// Fill a rectangle with a solid color.
    FillRect {
        rect: UiRect,
        color: UiColor,
    },
    /// Draw text within a bounding rectangle using a specific color.
    DrawText {
        rect: UiRect,
        text: String,
        color: UiColor,
    },
}
