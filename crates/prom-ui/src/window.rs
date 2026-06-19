/// Abstract lifecycle events that a host windowing system (like winit or a test harness)
/// can send to the UI core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiWindowEvent {
    /// The window surface needs to be redrawn.
    Draw,
    /// The window has been resized to the given physical width and height.
    Resize(u32, u32),
    /// The user requested the window to be closed.
    Close,
}

/// A pure contract representing the UI core's ability to handle window lifecycle events.
/// This abstraction ensures the UI core remains entirely decoupled from `winit`
/// or any other platform-specific windowing libraries.
pub trait UiWindowContract {
    fn handle_event(&mut self, event: UiWindowEvent);
}
