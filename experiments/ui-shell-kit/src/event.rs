#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiPointerButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiEventKind {
    PointerMoved {
        x: f64,
        y: f64,
    },
    PointerDown {
        x: f64,
        y: f64,
        button: UiPointerButton,
    },
    PointerUp {
        x: f64,
        y: f64,
        button: UiPointerButton,
    },
    CloseRequested,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiEvent {
    pub kind: UiEventKind,
}

impl UiEvent {
    pub fn new(kind: UiEventKind) -> Self {
        Self { kind }
    }
}
