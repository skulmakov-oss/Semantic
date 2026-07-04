use prom_ui::layout::UiLayoutGeometryRect;
use ui_shell_kit::action::UiAction;
use ui_shell_kit::calculator_controller::CalculatorController;
use ui_shell_kit::calculator_scene::{calculator_layout, hit_test_button, CalculatorButton};
use ui_shell_kit::event::{UiEvent, UiEventKind, UiPointerButton};
use ui_shell_kit::paint::UiFrame;

fn button_center(rect: UiLayoutGeometryRect) -> (f64, f64) {
    (
        (rect.x() + rect.width() as i32 / 2) as f64,
        (rect.y() + rect.height() as i32 / 2) as f64,
    )
}

fn canonical_button(
    layout: &ui_shell_kit::calculator_scene::CalculatorLayout,
    label: &str,
) -> (CalculatorButton, UiLayoutGeometryRect) {
    let (button, rect) = layout
        .buttons
        .iter()
        .copied()
        .find(|(button, _)| button.label() == label)
        .expect("calculator button exists in reference layout");
    assert!(
        rect.width() > 0,
        "canonical button {label} must have non-empty width"
    );
    assert!(
        rect.height() > 0,
        "canonical button {label} must have non-empty height"
    );
    (button, rect)
}

fn press_button(
    controller: &mut CalculatorController,
    scene: UiLayoutGeometryRect,
    label: &str,
) -> (Vec<UiAction>, CalculatorButton) {
    let layout = calculator_layout(scene);
    let (button, rect) = canonical_button(&layout, label);
    let (x, y) = button_center(rect);

    assert_eq!(hit_test_button(&layout, x, y), Some(button));
    assert_eq!(hit_test_button(&layout, x, y), Some(button));

    let actions = controller.handle_event(
        UiEvent::new(UiEventKind::PointerDown {
            x,
            y,
            button: UiPointerButton::Primary,
        }),
        scene,
    );

    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::ButtonPressed(pressed) if *pressed == button
        )),
        "pressing {label} should emit button press evidence"
    );

    (actions, button)
}

#[test]
fn calculator_hit_test_stability_is_executable() {
    let mut controller = CalculatorController::new();
    let scene = UiLayoutGeometryRect::new(0, 0, 800, 600);
    let layout = calculator_layout(scene);

    let mut initial_frame = UiFrame::new();
    controller.render(&mut initial_frame, scene);
    assert_eq!(controller.display_text(), "0");
    assert!(controller.focus().current().is_none());

    for label in ["7", "+", "3", "="] {
        let (actions, button) = press_button(&mut controller, scene, label);
        assert!(
            actions
                .iter()
                .any(|action| matches!(action, UiAction::FocusChanged(Some(focused)) if *focused == button)),
            "pressing {label} should update focus"
        );
        assert_eq!(controller.focus().current(), Some(button));
    }

    assert_eq!(controller.display_text(), "10");

    let outside = UiLayoutGeometryRect::new(0, 0, 800, 600);
    let outside_point = (outside.x() as f64 + 5.0, outside.y() as f64 + 5.0);
    assert_eq!(
        hit_test_button(&layout, outside_point.0, outside_point.1),
        None
    );
    assert_eq!(
        hit_test_button(&layout, outside_point.0, outside_point.1),
        None
    );

    let outside_actions = controller.handle_event(
        UiEvent::new(UiEventKind::PointerDown {
            x: outside_point.0,
            y: outside_point.1,
            button: UiPointerButton::Primary,
        }),
        scene,
    );
    assert!(
        outside_actions
            .iter()
            .all(|action| !matches!(action, UiAction::ButtonPressed(_))),
        "outside hit must not emit calculator button actions"
    );
    assert_eq!(controller.display_text(), "10");

    let mut final_frame = UiFrame::new();
    controller.render(&mut final_frame, scene);
    assert!(!final_frame.is_empty());
}
