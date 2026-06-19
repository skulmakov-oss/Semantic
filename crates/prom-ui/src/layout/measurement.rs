use crate::layout::UiSize;

/// A boundary trait for measuring text.
/// 
/// This allows the backend (e.g., wgpu/glyph_brush, CoreText, etc.) to inject 
/// text measuring logic into the layout solver without leaking backend-specific 
/// types (like Font or Glyph) into the semantic foundation.
pub trait UiTextMeasurer {
    /// Measures the precise rectangular dimensions required to render the given `text` 
    /// at the specified `font_size`.
    fn measure_text(&self, text: &str, font_size: u32) -> UiSize;
}
