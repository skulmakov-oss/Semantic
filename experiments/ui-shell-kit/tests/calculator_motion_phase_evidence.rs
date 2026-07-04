use ui_shell_kit::calculator_scene::render_calculator_scene_with_phase;
use ui_shell_kit::geometry::UiRect;
use ui_shell_kit::paint::UiFrame;
use ui_shell_kit::snapshot::frame_to_snapshot;
use ui_shell_kit::theme::UiShellTheme;

fn render_phase_snapshot(phase: f32) -> (UiFrame, String) {
    let mut frame = UiFrame::new();
    let theme = UiShellTheme::default();
    let scene = UiRect::new(0, 0, 760, 560);

    let _registry = render_calculator_scene_with_phase(
        &mut frame,
        scene,
        "123",
        None,
        None,
        theme,
        phase,
    );

    let snapshot = frame_to_snapshot(&frame);
    (frame, snapshot)
}

fn assert_motion_snapshot(label: &str, phase: f32) -> String {
    let (frame, snapshot) = render_phase_snapshot(phase);
    assert!(
        !frame.is_empty(),
        "{label} phase should emit at least one draw command"
    );
    assert!(
        frame.stats().total_commands > 0,
        "{label} phase should emit a non-empty command stream"
    );
    assert!(
        !snapshot.is_empty(),
        "{label} phase snapshot should not be empty"
    );
    assert!(
        snapshot.contains("value=\"Semantic Calculator\""),
        "{label} phase snapshot should include the calculator title"
    );
    assert!(
        snapshot.contains("value=\"123\""),
        "{label} phase snapshot should include the calculator display"
    );
    assert!(
        snapshot.contains("value=\"GLASS\""),
        "{label} phase snapshot should include the shell badge"
    );
    snapshot
}

#[test]
fn calculator_motion_phase_evidence_is_deterministic() {
    let entrance_a = assert_motion_snapshot("Entrance", 0.00);
    let entrance_b = assert_motion_snapshot("Entrance", 0.00);
    assert_eq!(
        entrance_a, entrance_b,
        "Entrance phase should be deterministic"
    );

    let settling_a = assert_motion_snapshot("Settling", 0.50);
    let settling_b = assert_motion_snapshot("Settling", 0.50);
    assert_eq!(
        settling_a, settling_b,
        "Settling phase should be deterministic"
    );

    let settled_a = assert_motion_snapshot("Settled", 1.00);
    let settled_b = assert_motion_snapshot("Settled", 1.00);
    assert_eq!(
        settled_a, settled_b,
        "Settled phase should be deterministic"
    );

    assert_ne!(entrance_a, settling_a, "Entrance and Settling should differ");
    assert_ne!(settling_a, settled_a, "Settling and Settled should differ");
    assert_ne!(entrance_a, settled_a, "Entrance and Settled should differ");
}
