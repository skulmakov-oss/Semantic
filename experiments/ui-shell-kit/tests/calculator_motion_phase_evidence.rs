use prom_ui::layout::UiLayoutGeometryRect;
use ui_shell_kit::calculator_controller::CalculatorController;
use ui_shell_kit::calculator_scene::calculator_layout;
use ui_shell_kit::event::{UiEvent, UiEventKind, UiPointerButton};
use ui_shell_kit::paint::UiFrame;
use ui_shell_kit::snapshot::frame_to_snapshot;

fn render_snapshot(
    controller: &CalculatorController,
    scene: UiLayoutGeometryRect,
) -> (UiFrame, String) {
    let mut frame = UiFrame::new();
    controller.render(&mut frame, scene);
    let snapshot = frame_to_snapshot(&frame);
    (frame, snapshot)
}

#[test]
fn calculator_shell_render_evidence_is_deterministic() {
    let mut controller = CalculatorController::new();
    let scene = UiLayoutGeometryRect::new(0, 0, 800, 600);

    let (frame_a, snapshot_a) = render_snapshot(&controller, scene);
    let (frame_b, snapshot_b) = render_snapshot(&controller, scene);
    assert!(!frame_a.is_empty());
    assert!(!frame_b.is_empty());
    assert_eq!(snapshot_a, snapshot_b);
    assert!(snapshot_a.contains("FillRect"));
    assert!(snapshot_a.contains("0"));

    let press = |controller: &mut CalculatorController, label: &str| {
        let layout = calculator_layout(scene);
        let (_, rect) = layout
            .buttons
            .iter()
            .find(|(button, _)| button.label() == label)
            .expect("calculator button exists in reference layout");
        let x = (rect.x() + rect.width() as i32 / 2) as f64;
        let y = (rect.y() + rect.height() as i32 / 2) as f64;
        controller.handle_event(
            UiEvent::new(UiEventKind::PointerDown {
                x,
                y,
                button: UiPointerButton::Primary,
            }),
            scene,
        );
    };

    for label in ["7", "+", "3", "="] {
        press(&mut controller, label);
    }

    let (final_frame_a, final_snapshot_a) = render_snapshot(&controller, scene);
    let (final_frame_b, final_snapshot_b) = render_snapshot(&controller, scene);
    assert!(!final_frame_a.is_empty());
    assert!(!final_frame_b.is_empty());
    assert_eq!(final_snapshot_a, final_snapshot_b);
    assert!(final_snapshot_a.contains("FillRect"));
    assert!(final_snapshot_a.contains("10"));
    assert_ne!(snapshot_a, final_snapshot_a);
}
