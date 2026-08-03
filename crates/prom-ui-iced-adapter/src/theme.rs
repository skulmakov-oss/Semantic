//! The one Prom UI theme, mapped to real Iced styling. Every color/spacing
//! value a screen needs must come from here -- no screen-local raw colors
//! (owner directive section 17).

use iced::Color;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/// Required token categories (owner directive section 17 / the prior
/// Round B report's already-proven color palette, carried forward
/// verbatim so the Quad Logic N/F/T/S visual language does not change
/// meaning mid-migration).
#[derive(Debug, Clone, Copy)]
pub struct PromTheme {
    pub root: Color,
    pub panel: Color,
    pub elevated: Color,
    pub interactive: Color,
    pub border: Color,
    pub divider: Color,
    pub hover: Color,
    pub selected: Color,
    pub focus: Color,
    pub accent: Color,
    pub success: Color,
    pub failure: Color,
    pub warning: Color,
    pub denied: Color,
    pub cancelled: Color,
    pub interrupted: Color,
    pub unknown: Color,
    pub conflict: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_code: Color,
}

impl PromTheme {
    // Values carried over from the prior pass's proven, real design-
    // reference-derived palette (COLOR_BG/COLOR_SURFACE/COLOR_TEAL/etc.
    // in the pre-Iced examples/workbench_semantic renderer), rather than
    // inventing a new, unverified color set.
    pub const DARK: Self = Self {
        root: rgb(0x0e, 0x0e, 0x12),
        panel: rgb(0x15, 0x15, 0x1b),
        elevated: rgb(0x1c, 0x1c, 0x24),
        interactive: rgb(0x22, 0x22, 0x2c),
        border: rgb(0x2a, 0x2a, 0x36),
        divider: rgb(0x24, 0x24, 0x30),
        hover: rgb(0x28, 0x28, 0x34),
        selected: rgb(0x2e, 0xc4, 0xb6),
        focus: rgb(0x4a, 0x9e, 0xff),
        accent: rgb(0x2e, 0xc4, 0xb6),
        success: rgb(0x2e, 0xc4, 0xb6),
        failure: rgb(0xe5, 0x5c, 0x5c),
        warning: rgb(0xe0, 0xa8, 0x3c),
        denied: rgb(0xe0, 0xa8, 0x3c),
        cancelled: rgb(0xe0, 0xa8, 0x3c),
        interrupted: rgb(0xe0, 0xa8, 0x3c),
        unknown: rgb(0x8a, 0x8a, 0x99),
        conflict: rgb(0xe5, 0x5c, 0x5c),
        text_primary: rgb(0xe8, 0xe8, 0xee),
        text_secondary: rgb(0xb0, 0xb0, 0xbc),
        text_muted: rgb(0x8a, 0x8a, 0x99),
        text_code: rgb(0xd0, 0xd0, 0xda),
    };

    /// Resolves a `PromStatus` to a real color -- the one place status
    /// meaning becomes a pixel value.
    pub fn status_color(&self, status: crate::node::PromStatus) -> Color {
        use crate::node::PromStatus::*;
        match status {
            Normal => self.text_primary,
            Loading => self.focus,
            Succeeded => self.success,
            Failed => self.failure,
            Warning => self.warning,
            Denied => self.denied,
            Cancelled => self.cancelled,
            Interrupted => self.interrupted,
            Stale => self.text_muted,
            Unknown => self.unknown,
            Conflict => self.conflict,
        }
    }

    /// Resolves a named `Typography` role to a real point size -- the one
    /// place a font-size number is chosen (owner directive section 17: "no
    /// random direct colors in individual screen view builders" applies
    /// the same way to sizes; screens name a role, never a literal).
    /// Sizes are carried over unchanged from the literals already proven
    /// live in each role's rendering (see `crate::convert`) -- this is a
    /// centralization refactor, not a redesign.
    pub fn typography_size(&self, typography: crate::node::Typography) -> f32 {
        use crate::node::Typography::*;
        match typography {
            Product => 18.0,
            PageTitle => 15.0,
            SectionTitle => 15.0,
            Body => 16.0,
            Metadata => 13.0,
            Control => 12.0,
            Badge => 11.0,
            TableHeader => 12.0,
            TableRow => 14.0,
            Code => 13.0,
            Evidence => 13.0,
            EmptyState => 14.0,
        }
    }

    pub fn accent_color(&self, accent: Option<&'static str>) -> Color {
        match accent {
            Some("teal") => self.success,
            Some("blue") => self.focus,
            Some("amber") => self.warning,
            Some("red") => self.failure,
            Some("muted") => self.text_muted,
            _ => self.accent,
        }
    }
}

impl Default for PromTheme {
    fn default() -> Self {
        Self::DARK
    }
}
