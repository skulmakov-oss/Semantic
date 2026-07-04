use crate::calculator_scene::CalculatorButton;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    ButtonPressed(CalculatorButton),
    FocusChanged(Option<CalculatorButton>),
}

pub type UiActionQueue = alloc::vec::Vec<UiAction>;
