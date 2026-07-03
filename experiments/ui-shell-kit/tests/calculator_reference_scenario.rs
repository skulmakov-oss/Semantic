use ui_shell_kit::action::UiAction;
use ui_shell_kit::calculator_controller::CalculatorController;
use ui_shell_kit::calculator_scene::{calculator_button_label, calculator_layout};
use ui_shell_kit::event::{UiEvent, UiEventKind, UiPointerButton};
use ui_shell_kit::geometry::UiRect;
use ui_shell_kit::paint::UiFrame;
use ui_shell_kit::snapshot::frame_to_snapshot;
use ui_shell_kit::theme::UiShellTheme;

fn button_center(rect: UiRect) -> (i32, i32) {
    (rect.x + rect.width as i32 / 2, rect.y + rect.height as i32 / 2)
}

fn press_button(
    controller: &mut CalculatorController,
    layout: &ui_shell_kit::calculator_scene::CalculatorLayout,
    label: &str,
) -> Vec<UiAction> {
    let (_, rect) = layout
        .buttons
        .iter()
        .find(|(button, _)| calculator_button_label(*button) == label)
        .expect("calculator button exists in reference layout");

    let (x, y) = button_center(*rect);
    let event = UiEvent {
        kind: UiEventKind::PointerDown {
            x,
            y,
            button: UiPointerButton::Primary,
        },
    };

    controller.handle_event(event, layout).drain()
}

#[test]
fn calculator_reference_scenario_is_executable() {
    let mut controller = CalculatorController::new();
    let scene = UiRect::new(0, 0, 800, 600);
    let theme = UiShellTheme::default();
    let layout = calculator_layout(scene);

    let mut initial_frame = UiFrame::new();
    controller.render(&mut initial_frame, scene, &theme);
    assert_eq!(controller.state.display, "0");
    assert!(!initial_frame.is_empty(), "initial render should emit draw commands");
    let initial_snapshot = frame_to_snapshot(&initial_frame);
    assert!(
        !initial_snapshot.is_empty(),
        "initial snapshot should not be empty"
    );
    assert!(
        initial_snapshot.contains("value=\"0\""),
        "initial snapshot should include the zero display"
    );
    assert!(
        initial_snapshot.contains("value=\"Semantic Calculator\""),
        "initial snapshot should include the calculator title"
    );

    for label in ["7", "+", "3", "="] {
        let actions = press_button(&mut controller, &layout, label);
        assert!(
            actions.iter().any(|action| matches!(action, UiAction::CalculatorButtonPressed { .. })),
            "pressing {label} should emit a calculator button action"
        );
        assert!(
            actions.iter().any(|action| matches!(action, UiAction::ButtonPressed { .. })),
            "pressing {label} should emit a button press action"
        );
    }

    assert_eq!(controller.state.display, "10");

    let mut final_frame = UiFrame::new();
    controller.render(&mut final_frame, scene, &theme);
    assert!(!final_frame.is_empty(), "final render should emit draw commands");

    let final_snapshot = frame_to_snapshot(&final_frame);
    assert!(
        !final_snapshot.is_empty(),
        "final snapshot should not be empty"
    );
    assert!(
        final_snapshot.contains("value=\"10\""),
        "final snapshot should include the evaluated result"
    );
    assert!(
        final_snapshot.contains("value=\"Semantic Calculator\""),
        "final snapshot should include the calculator title"
    );
}
