//! UI-DNA2-10 native reference application (Issue #1543).
//!
//! The deterministic, non-native pipeline (compile -> Gate D activate ->
//! layout -> admission -> render) lives in
//! `prom_ui_runtime::reference_contour` (Issue #1365) so it can be driven
//! both by this native binary and by a headless CLI inspector. This module
//! only owns the native windowing loop: constructing a real
//! `NativeBackend`/`DesktopSession`, translating its input events, and
//! printing narration to the console.

#![allow(dead_code)]

use std::time::Instant;

use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::reference_contour::{
    dispatch_input_event, render_frame, ReferenceState, REFERENCE_SOURCE_TEXT,
};
use prom_ui_runtime::{LoopControl, WindowConfig};

pub fn run_ui_dna2_reference() {
    println!("=== UI-DNA2 Reference Application (Issue #1543) ===");

    let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT)
        .expect("reference source text is valid Grammar v0 and activates");
    println!("reference contour compiled, verified, and Gate D activated");

    let config = WindowConfig::new("Semantic UI-DNA2 Reference", 640, 480);
    let backend = NativeBackend::new();
    let mut session = DesktopSessionAlias::create(backend, config)
        .expect("NativeBackend::create_window must succeed");

    let start = Instant::now();
    let mut ran_negative_checks = false;

    session
        .run(move |buf, out_frame| {
            let events = buf.drain();
            for event in &events {
                if dispatch_input_event(&mut state, &event.kind) == LoopControl::ExitRequested {
                    *out_frame = render_frame(&state);
                    return LoopControl::ExitRequested;
                }
            }

            let elapsed = start.elapsed().as_secs_f64();
            if !ran_negative_checks && elapsed >= 2.0 {
                ran_negative_checks = true;
                state.run_invalid_bundle_check();
                state.run_replay_and_staleness_check();
                println!(
                    "invalid-bundle rejected={} replay rejected={} stale rejected={}",
                    state.invalid_bundle_rejected(),
                    state.replay_rejected(),
                    state.stale_rejected()
                );
            }

            *out_frame = render_frame(&state);

            if elapsed >= 120.0 {
                return LoopControl::ExitRequested;
            }
            LoopControl::Continue
        })
        .expect("event loop must succeed");

    let _ = session.close();
    println!("Session closed.");
}

type DesktopSessionAlias = prom_ui_runtime::DesktopSession<NativeBackend>;

#[cfg(test)]
mod tests {
    use super::*;
    use prom_ui::shell_bridge::{activate_projection_bundle_v0_gate_d, BridgeAvailability};
    use prom_ui_runtime::reference_contour::{
        compile_invalid_bundle, compile_reference_bundle, ReferenceLayout, BUTTON_NODE, LABEL_NODE,
        LIST_NODE, ROOT_NODE,
    };
    use prom_ui_runtime::InputEventKind;

    #[test]
    fn reference_source_compiles_to_bundle_and_activates() {
        let bytes = compile_reference_bundle();
        let bundle = activate_projection_bundle_v0_gate_d(&bytes).expect("activates");
        assert_eq!(bundle.root_node(), ROOT_NODE);
        assert_eq!(bundle.nodes().len(), 4);
        assert_eq!(
            bundle.activation_snapshot().collection_anchor_ids(),
            &[LIST_NODE]
        );
        assert_eq!(bundle.accessibility_entries().len(), 4);
    }

    #[test]
    fn invalid_source_is_rejected() {
        assert!(compile_invalid_bundle().is_none());
    }

    #[test]
    fn layout_places_declared_children_in_declared_order() {
        let bytes = compile_reference_bundle();
        let bundle = activate_projection_bundle_v0_gate_d(&bytes).expect("activates");
        let layout = ReferenceLayout::build(&bundle);
        assert_eq!(
            layout.focus_order(),
            vec![LABEL_NODE, BUTTON_NODE, LIST_NODE]
        );
    }

    #[test]
    fn hit_test_resolves_the_correct_band() {
        let bytes = compile_reference_bundle();
        let bundle = activate_projection_bundle_v0_gate_d(&bytes).expect("activates");
        let layout = ReferenceLayout::build(&bundle);
        let button_rect = layout.rect_of(BUTTON_NODE).expect("button rect exists");
        let cx = (button_rect.x + button_rect.width as i32 / 2) as f64;
        let cy = (button_rect.y + button_rect.height as i32 / 2) as f64;
        assert_eq!(layout.hit_test(cx, cy), Some(BUTTON_NODE));
        assert_eq!(layout.hit_test(-100.0, -100.0), None);
    }

    #[test]
    fn admitted_action_commits_visible_patch() {
        let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT).expect("activates");
        assert_eq!(state.shell().node_availability(BUTTON_NODE), None);
        state.trigger(BUTTON_NODE);
        assert!(!state.last_denied());
        assert_eq!(
            state.shell().node_availability(BUTTON_NODE),
            Some(BridgeAvailability::Available)
        );
        assert_eq!(state.shell().collection_entries(LIST_NODE).len(), 1);
    }

    #[test]
    fn denied_action_preserves_state_and_cursor() {
        let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT).expect("activates");
        let cursor_before = state.shell().replay_cursor();
        state.trigger(LABEL_NODE); // LABEL_NODE's action id is not granted
        assert!(state.last_denied());
        assert_eq!(state.shell().node_availability(BUTTON_NODE), None);
        assert_eq!(state.shell().replay_cursor(), cursor_before);
    }

    #[test]
    fn tab_cycles_focus_through_declared_children() {
        let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT).expect("activates");
        let initial = state.focused();
        state.handle_tab();
        let after_one = state.focused();
        assert_ne!(initial, after_one);
        state.handle_tab();
        state.handle_tab();
        assert_eq!(state.focused(), initial);
    }

    #[test]
    fn enter_activates_the_focused_node() {
        let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT).expect("activates");
        state.handle_tab();
        assert_eq!(state.focused(), Some(BUTTON_NODE));
        state.handle_activate();
        assert!(!state.last_denied());
        assert_eq!(
            state.shell().node_availability(BUTTON_NODE),
            Some(BridgeAvailability::Available)
        );
    }

    #[test]
    fn invalid_bundle_check_preserves_active_state() {
        let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT).expect("activates");
        state.trigger(BUTTON_NODE);
        let label_before = state.shell().binding_value(LABEL_NODE, 0);
        let cursor_before = state.shell().replay_cursor();
        state.run_invalid_bundle_check();
        assert!(state.invalid_bundle_rejected());
        assert_eq!(state.shell().binding_value(LABEL_NODE, 0), label_before);
        assert_eq!(state.shell().replay_cursor(), cursor_before);
    }

    #[test]
    fn replay_and_stale_checks_reject_without_mutating_state() {
        let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT).expect("activates");
        let cursor_before_checks = state.shell().replay_cursor();
        state.run_replay_and_staleness_check();
        assert!(state.replay_rejected());
        assert!(state.stale_rejected());
        let cursor_after = state.shell().replay_cursor();
        assert_ne!(cursor_before_checks, cursor_after);
        state.run_replay_and_staleness_check();
        assert_eq!(state.shell().replay_cursor(), cursor_after);
    }

    #[test]
    fn repeated_activation_is_deterministic() {
        let bytes_a = compile_reference_bundle();
        let bytes_b = compile_reference_bundle();
        assert_eq!(bytes_a, bytes_b);
        let bundle_a = activate_projection_bundle_v0_gate_d(&bytes_a).expect("activates");
        let bundle_b = activate_projection_bundle_v0_gate_d(&bytes_b).expect("activates");
        assert_eq!(bundle_a, bundle_b);
    }

    #[test]
    fn dispatch_input_event_matches_native_key_and_pointer_convention() {
        let mut state = ReferenceState::new(REFERENCE_SOURCE_TEXT).expect("activates");
        assert_eq!(
            dispatch_input_event(&mut state, &InputEventKind::KeyDown { key_code: 27 }),
            LoopControl::ExitRequested
        );
    }
}
