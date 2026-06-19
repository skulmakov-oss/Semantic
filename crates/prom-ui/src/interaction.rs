//! Semantic UI interaction intent scaffold.
//!
//! This module defines normalized interaction intent descriptors.
//! It does not execute actions, call the VM, call Host ABI, or implement
//! widget/layout/event-loop behavior.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionIntentId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionIntentKind {
    Activate,
    Cancel,
    Submit,
    Select,
    Deselect,
    Focus,
    Blur,
    Hover,
    Navigate,
    Edit,
    Drag,
    Drop,
    Scroll,
    Resize,
    Close,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionSource {
    Pointer,
    Keyboard,
    TextInput,
    Wheel,
    Window,
    System,
    Synthetic,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ElementId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionTarget {
    Window(WindowId),
    Surface(SurfaceId),
    Element(ElementId),
    Region(RegionId),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct InteractionModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionIntentDescriptor {
    pub id: InteractionIntentId,
    pub source: InteractionSource,
    pub kind: InteractionIntentKind,
    pub target: Option<InteractionTarget>,
    pub modifiers: InteractionModifiers,
}

impl InteractionIntentDescriptor {
    pub const fn new(
        id: InteractionIntentId,
        source: InteractionSource,
        kind: InteractionIntentKind,
        target: Option<InteractionTarget>,
        modifiers: InteractionModifiers,
    ) -> Self {
        Self {
            id,
            source,
            kind,
            target,
            modifiers,
        }
    }

    pub const fn unknown(id: InteractionIntentId) -> Self {
        Self {
            id,
            source: InteractionSource::Unknown,
            kind: InteractionIntentKind::Unknown,
            target: None,
            modifiers: InteractionModifiers {
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            },
        }
    }

    pub const fn is_classified(&self) -> bool {
        !matches!(self.kind, InteractionIntentKind::Unknown)
            && !matches!(self.source, InteractionSource::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_intent_is_not_classified() {
        let intent = InteractionIntentDescriptor::unknown(InteractionIntentId(1));
        assert!(!intent.is_classified());
    }

    #[test]
    fn activate_pointer_intent_is_classified() {
        let intent = InteractionIntentDescriptor::new(
            InteractionIntentId(1),
            InteractionSource::Pointer,
            InteractionIntentKind::Activate,
            Some(InteractionTarget::Element(ElementId(7))),
            InteractionModifiers::default(),
        );

        assert!(intent.is_classified());
    }

    #[test]
    fn modifiers_default_to_false() {
        let modifiers = InteractionModifiers::default();

        assert!(!modifiers.shift);
        assert!(!modifiers.ctrl);
        assert!(!modifiers.alt);
        assert!(!modifiers.meta);
    }
}
