use ui_shell_kit::action::UiAction;
use ui_shell_kit::calculator_controller::CalculatorController;
use ui_shell_kit::calculator_scene::{calculator_button_label, calculator_layout};
use ui_shell_kit::event::{UiEvent, UiEventKind, UiPointerButton};
use ui_shell_kit::geometry::{UiPoint, UiRect};
use ui_shell_kit::paint::UiFrame;
use ui_shell_kit::theme::UiShellTheme;

fn button_center(rect: UiRect) -> UiPoint {
    UiPoint::new(
        rect.x + rect.width as i32 / 2,
        rect.y + rect.height as i32 / 2,
    )
}

fn canonical_button(
    layout: &ui_shell_kit::calculator_scene::CalculatorLayout,
    label: &str,
) -> (UiRect, ui_shell_kit::hit_test::HitTargetId) {
    let (_, rect) = layout
        .buttons
        .iter()
        .find(|(button, _)| calculator_button_label(*button) == label)
        .expect("calculator button exists in reference layout");
    assert!(rect.width > 0, "canonical button {label} must have non-empty width");
    assert!(rect.height > 0, "canonical button {label} must have non-empty height");

    let target_id = layout
        .hit_targets
        .iter()
        .find(|target| target.rect == *rect)
        .expect("button hit target exists in reference layout")
        .id;

    (*rect, target_id)
}

fn assert_stable_hit(
    controller: &CalculatorController,
    point: UiPoint,
    expected: ui_shell_kit::hit_test::HitTargetId,
) {
    let first = controller.registry.hit_test(point);
    let second = controller.registry.hit_test(point);
    assert_eq!(first, Some(expected));
    assert_eq!(second, Some(expected));
}

fn press_button(
    controller: &mut CalculatorController,
    layout: &ui_shell_kit::calculator_scene::CalculatorLayout,
    label: &str,
) -> (Vec<UiAction>, ui_shell_kit::hit_test::HitTargetId) {
    let (rect, target_id) = canonical_button(layout, label);
    let point = button_center(rect);
    assert_stable_hit(controller, point, target_id);

    let event = UiEvent {
        kind: UiEventKind::PointerDown {
            x: point.x,
            y: point.y,
            button: UiPointerButton::Primary,
        },
    };

    let actions = controller.handle_event(event, layout).drain();
    assert!(
        !actions.is_empty(),
        "pressing {label} should emit at least one action",
    );
    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::CalculatorButtonPressed { label: pressed_label } if *pressed_label == label
        )),
        "pressing {label} should emit calculator button evidence",
    );
    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::ButtonPressed { id } if id.0 == target_id.0
        )),
        "pressing {label} should emit button id evidence",
    );
    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::FocusChanged { to: Some(id), .. } if *id == target_id
        )),
        "pressing {label} should emit focus evidence",
    );

    (actions, target_id)
}

#[test]
fn calculator_hit_test_stability_is_executable() {
    let mut controller = CalculatorController::new();
    let scene = UiRect::new(0, 0, 800, 600);
    let theme = UiShellTheme::default();
    let layout = calculator_layout(scene);

    let mut initial_frame = UiFrame::new();
    controller.render(&mut initial_frame, scene, &theme);
    assert_eq!(controller.state.display, "0");
    assert!(controller.focus.current().is_none());

    for label in ["7", "+", "3", "="] {
        let (actions, target_id) = press_button(&mut controller, &layout, label);
        assert!(
            actions.iter().any(|action| matches!(
                action,
                UiAction::CalculatorButtonPressed { .. } | UiAction::ButtonPressed { .. } | UiAction::FocusChanged { .. }
            )),
            "pressing {label} should emit calculator interaction evidence",
        );
        assert_eq!(controller.focus.current(), Some(target_id));
    }

    assert_eq!(controller.state.display, "10");

    let outside = UiPoint::new(799, 599);
    assert_eq!(controller.registry.hit_test(outside), None);
    assert_eq!(controller.registry.hit_test(outside), None);

    let outside_event = UiEvent {
        kind: UiEventKind::PointerDown {
            x: outside.x,
            y: outside.y,
            button: UiPointerButton::Primary,
        },
    };
    let outside_actions = controller.handle_event(outside_event, &layout).drain();
    assert!(
        outside_actions
            .iter()
            .all(|action| !matches!(action, UiAction::CalculatorButtonPressed { .. } | UiAction::ButtonPressed { .. })),
        "outside hit must not emit calculator button actions",
    );
    assert_eq!(controller.state.display, "10");

    let mut final_frame = UiFrame::new();
    controller.render(&mut final_frame, scene, &theme);
    assert!(!final_frame.is_empty());
}
