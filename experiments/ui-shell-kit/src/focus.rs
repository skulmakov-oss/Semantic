use crate::calculator_scene::CalculatorButton;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusRing {
    focused_button: Option<CalculatorButton>,
}

impl FocusRing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> Option<CalculatorButton> {
        self.focused_button
    }

    pub fn set(&mut self, focused_button: Option<CalculatorButton>) {
        self.focused_button = focused_button;
    }

    pub fn clear(&mut self) {
        self.focused_button = None;
    }
}
