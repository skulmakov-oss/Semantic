use prom_ui::layout::UiLayoutGeometryRect;
use ui_shell_kit::action::UiAction;
use ui_shell_kit::calculator_controller::CalculatorController;
use ui_shell_kit::calculator_scene::calculator_layout;
use ui_shell_kit::event::{UiEvent, UiEventKind, UiPointerButton};
use ui_shell_kit::paint::UiFrame;
use ui_shell_kit::snapshot::frame_to_snapshot;

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
) -> Vec<UiAction> {
    let layout = calculator_layout(scene);
    let (_, rect) = layout
        .buttons
        .iter()
        .find(|(button, _)| button.label() == label)
        .expect("calculator button exists in reference layout");

    let (x, y) = button_center(*rect);
    controller.handle_event(
        UiEvent::new(UiEventKind::PointerDown {
            x,
            y,
            button: UiPointerButton::Primary,
        }),
        scene,
    )
}

#[test]
fn calculator_reference_scenario_is_executable() {
    let mut controller = CalculatorController::new();
    let scene = UiLayoutGeometryRect::new(0, 0, 800, 600);

    let mut initial_frame = UiFrame::new();
    controller.render(&mut initial_frame, scene);
    assert_eq!(controller.display_text(), "0");
    assert!(
        !initial_frame.is_empty(),
        "initial render should emit draw commands"
    );

    let initial_snapshot = frame_to_snapshot(&initial_frame);
    assert!(
        !initial_snapshot.is_empty(),
        "initial snapshot should not be empty"
    );
    assert!(initial_snapshot.contains("FillRect"));

    for label in ["7", "+", "3", "="] {
        let actions = press_button(&mut controller, scene, label);
        assert!(
            actions.iter().any(|action| matches!(
                action,
                UiAction::ButtonPressed(button) if button.label() == label
            )),
            "pressing {label} should emit a button press action"
        );
        assert!(
            actions.iter().any(|action| matches!(
                action,
                UiAction::FocusChanged(Some(button)) if button.label() == label
            )),
            "pressing {label} should update focus"
        );
    }

    assert_eq!(controller.display_text(), "10");

    let mut final_frame = UiFrame::new();
    controller.render(&mut final_frame, scene);
    assert!(
        !final_frame.is_empty(),
        "final render should emit draw commands"
    );

    let final_snapshot = frame_to_snapshot(&final_frame);
    assert!(
        !final_snapshot.is_empty(),
        "final snapshot should not be empty"
    );
    assert!(final_snapshot.contains("FillRect"));
    assert!(final_snapshot.contains("10"));
    assert_ne!(initial_snapshot, final_snapshot);
}
