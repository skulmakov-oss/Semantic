use crate::interaction::RoutedInteraction;
use prom_ui::InteractionIntentKind;

/// Translates physical backend events into agnostic interaction intent kinds.
pub trait NativeActionTranslator {
    fn translate(&self, interaction: &RoutedInteraction) -> InteractionIntentKind;
}

/// The default inert event-to-intent translator for the native backend.
#[derive(Debug, Clone, Default)]
pub struct DefaultNativeActionTranslator;

impl DefaultNativeActionTranslator {
    pub fn new() -> Self {
        Self
    }
}

impl NativeActionTranslator for DefaultNativeActionTranslator {
    fn translate(&self, interaction: &RoutedInteraction) -> InteractionIntentKind {
        match interaction.event {
            crate::RawBackendEvent::PointerMoved { .. } => InteractionIntentKind::Unknown,
            crate::RawBackendEvent::KeyboardInput {
                state: crate::RawButtonState::Pressed,
                ..
            } => InteractionIntentKind::Activate,
            crate::RawBackendEvent::KeyboardInput {
                state: crate::RawButtonState::Released,
                ..
            } => InteractionIntentKind::Unknown,
            crate::RawBackendEvent::WindowResized { .. } => InteractionIntentKind::Resize,
            crate::RawBackendEvent::CloseRequested => InteractionIntentKind::Close,
        }
    }
}
