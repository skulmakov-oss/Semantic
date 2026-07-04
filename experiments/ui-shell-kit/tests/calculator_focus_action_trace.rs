use prom_ui::layout::UiLayoutGeometryRect;
use ui_shell_kit::action::UiAction;
use ui_shell_kit::calculator_controller::CalculatorController;
use ui_shell_kit::calculator_scene::{calculator_layout, CalculatorButton};
use ui_shell_kit::event::{UiEvent, UiEventKind, UiPointerButton};
use ui_shell_kit::paint::UiFrame;

fn button_center(rect: UiLayoutGeometryRect) -> (f64, f64) {
    (
        (rect.x() + rect.width() as i32 / 2) as f64,
        (rect.y() + rect.height() as i32 / 2) as f64,
    )
}

fn press_button(
    controller: &mut CalculatorController,
    scene: UiLayoutGeometryRect,
    label: &str,
) -> (Vec<UiAction>, CalculatorButton) {
    let layout = calculator_layout(scene);
    let (button, rect) = layout
        .buttons
        .iter()
        .find(|(button, _)| button.label() == label)
        .copied()
        .expect("calculator button exists in reference layout");

    let (x, y) = button_center(rect);
    let actions = controller.handle_event(
        UiEvent::new(UiEventKind::PointerDown {
            x,
            y,
            button: UiPointerButton::Primary,
        }),
        scene,
    );

    (actions, button)
}

fn action_contains_button_press(actions: &[UiAction], label: &str) -> bool {
    actions
        .iter()
        .any(|action| matches!(action, UiAction::ButtonPressed(button) if button.label() == label))
}

fn action_contains_focus_change(actions: &[UiAction], button: CalculatorButton) -> bool {
    actions
        .iter()
        .any(|action| matches!(action, UiAction::FocusChanged(Some(focused)) if *focused == button))
}

#[test]
fn calculator_focus_action_trace_is_executable() {
    let mut controller = CalculatorController::new();
    let scene = UiLayoutGeometryRect::new(0, 0, 800, 600);

    let mut initial_frame = UiFrame::new();
    controller.render(&mut initial_frame, scene);
    assert_eq!(controller.display_text(), "0");
    assert_eq!(controller.focus().current(), None);

    let (actions_7, target_7) = press_button(&mut controller, scene, "7");
    assert!(action_contains_button_press(&actions_7, "7"));
    assert!(action_contains_focus_change(&actions_7, target_7));
    assert_eq!(controller.focus().current(), Some(target_7));
    assert_eq!(controller.display_text(), "7");

    let (actions_plus, target_plus) = press_button(&mut controller, scene, "+");
    assert!(action_contains_button_press(&actions_plus, "+"));
    assert!(action_contains_focus_change(&actions_plus, target_plus));
    assert_eq!(controller.focus().current(), Some(target_plus));
    assert_eq!(controller.display_text(), "7");

    let (actions_3, target_3) = press_button(&mut controller, scene, "3");
    assert!(action_contains_button_press(&actions_3, "3"));
    assert!(action_contains_focus_change(&actions_3, target_3));
    assert_eq!(controller.focus().current(), Some(target_3));
    assert_eq!(controller.display_text(), "3");

    let (actions_equals, target_equals) = press_button(&mut controller, scene, "=");
    assert!(action_contains_button_press(&actions_equals, "="));
    assert!(action_contains_focus_change(&actions_equals, target_equals));
    assert_eq!(controller.focus().current(), Some(target_equals));
    assert_eq!(controller.display_text(), "10");

    let mut final_frame = UiFrame::new();
    controller.render(&mut final_frame, scene);
    assert!(!final_frame.is_empty());
}
