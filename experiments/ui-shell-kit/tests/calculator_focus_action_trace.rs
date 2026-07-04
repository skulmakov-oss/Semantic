use ui_shell_kit::action::UiAction;
use ui_shell_kit::calculator_controller::CalculatorController;
use ui_shell_kit::calculator_scene::{calculator_button_label, calculator_layout};
use ui_shell_kit::event::{UiEvent, UiEventKind, UiPointerButton};
use ui_shell_kit::geometry::UiRect;
use ui_shell_kit::paint::UiFrame;
use ui_shell_kit::theme::UiShellTheme;

fn button_center(rect: UiRect) -> (i32, i32) {
    (
        rect.x + rect.width as i32 / 2,
        rect.y + rect.height as i32 / 2,
    )
}

fn press_button(
    controller: &mut CalculatorController,
    layout: &ui_shell_kit::calculator_scene::CalculatorLayout,
    label: &str,
) -> (Vec<UiAction>, ui_shell_kit::hit_test::HitTargetId) {
    let (_button, rect) = layout
        .buttons
        .iter()
        .find(|(button, _)| calculator_button_label(*button) == label)
        .expect("calculator button exists in reference layout");
    let target_id = layout
        .hit_targets
        .iter()
        .find(|target| target.rect == *rect)
        .expect("button hit target exists in reference layout")
        .id;

    let (x, y) = button_center(*rect);
    let event = UiEvent {
        kind: UiEventKind::PointerDown {
            x,
            y,
            button: UiPointerButton::Primary,
        },
    };

    let actions = controller.handle_event(event, layout).drain();
    assert!(
        !actions.is_empty(),
        "pressing {label} should emit at least one action"
    );
    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::ButtonPressed { .. } | UiAction::CalculatorButtonPressed { .. } | UiAction::FocusChanged { .. }
        )),
        "pressing {label} should emit calculator interaction evidence"
    );

    (actions, target_id)
}

fn action_contains_button_press(actions: &[UiAction], label: &'static str) -> bool {
    actions.iter().any(|action| matches!(
        action,
        UiAction::CalculatorButtonPressed { label: pressed_label } if *pressed_label == label
    ))
}

fn action_contains_button_id(actions: &[UiAction], target_id: u32) -> bool {
    actions.iter().any(|action| matches!(
        action,
        UiAction::ButtonPressed { id } if id.0 == target_id
    ))
}

fn action_contains_focus_change(actions: &[UiAction], target_id: ui_shell_kit::hit_test::HitTargetId) -> bool {
    actions.iter().any(|action| matches!(
        action,
        UiAction::FocusChanged { to: Some(id), .. } if *id == target_id
    ))
}

#[test]
fn calculator_focus_action_trace_is_executable() {
    let mut controller = CalculatorController::new();
    let scene = UiRect::new(0, 0, 800, 600);
    let theme = UiShellTheme::default();
    let layout = calculator_layout(scene);

    let mut initial_frame = UiFrame::new();
    controller.render(&mut initial_frame, scene, &theme);
    assert_eq!(controller.state.display, "0");
    assert_eq!(controller.focus.current(), None);

    let (actions_7, target_7) = press_button(&mut controller, &layout, "7");
    assert!(action_contains_button_press(&actions_7, "7"));
    assert!(action_contains_button_id(&actions_7, target_7.0));
    assert!(action_contains_focus_change(&actions_7, target_7));
    assert_eq!(controller.focus.current(), Some(target_7));
    assert_eq!(controller.state.display, "7");

    let (actions_plus, target_plus) = press_button(&mut controller, &layout, "+");
    assert!(action_contains_button_press(&actions_plus, "+"));
    assert!(action_contains_button_id(&actions_plus, target_plus.0));
    assert!(action_contains_focus_change(&actions_plus, target_plus));
    assert_eq!(controller.focus.current(), Some(target_plus));
    assert_eq!(controller.state.display, "7");

    let (actions_3, target_3) = press_button(&mut controller, &layout, "3");
    assert!(action_contains_button_press(&actions_3, "3"));
    assert!(action_contains_button_id(&actions_3, target_3.0));
    assert!(action_contains_focus_change(&actions_3, target_3));
    assert_eq!(controller.focus.current(), Some(target_3));
    assert_eq!(controller.state.display, "3");

    let (actions_equals, target_equals) = press_button(&mut controller, &layout, "=");
    assert!(action_contains_button_press(&actions_equals, "="));
    assert!(action_contains_button_id(&actions_equals, target_equals.0));
    assert!(action_contains_focus_change(&actions_equals, target_equals));
    assert_eq!(controller.focus.current(), Some(target_equals));
    assert_eq!(controller.state.display, "10");

    let mut final_frame = UiFrame::new();
    controller.render(&mut final_frame, scene, &theme);
    assert!(!final_frame.is_empty());
}
