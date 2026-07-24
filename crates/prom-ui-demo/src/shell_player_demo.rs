//! Deterministic native Shell Player demonstration.
//!
//! Drives the real Shell Player pipeline end to end through the existing
//! native backend and render loop: activates a session from
//! `prom_ui::shell_bridge::demo_activation_snapshot`, submits one accepted
//! ordered patch batch, submits one deliberately invalid batch, and renders
//! every frame directly from the committed Shell Player state. There is no
//! parallel demo-only mutation path — `DrawFrame` content is read back from
//! `ShellSession` via `binding_value`/`node_availability`/`collection_entries`.

use prom_ui::shell_bridge::{
    admit_projection_patch_batch, demo_activation_snapshot, prepared_manifest_snapshot,
    BridgeAvailability, BridgePatchEnvelope, BridgePatchOperation, BridgeValue,
};
use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::shell_player::{
    create_shell_session, OrderedProjectionPatchBatchEnvelope,
    ProjectionPatchApplicationDisposition, ShellLifecycleCommand, ShellLifecycleStimulus,
    ShellSession, ShellSessionId, ShellSessionLimits, ShellTransitionInput,
};
use prom_ui_runtime::{Color, DrawFrame, InputEventKind, LoopControl, Rect, WindowConfig};
use std::time::Instant;

const LABEL_NODE: u64 = 2;
const LABEL_SLOT: u32 = 0;
const STATUS_NODE: u64 = 3;
const LIST_NODE: u64 = 4;

/// Renders the current committed Shell Player state as filled rectangles.
///
/// `prom-ui-backend-native`'s wgpu presenter (`present_semantic_commands`)
/// only implements `DrawCommand::Clear` and `DrawCommand::FillRect` — text
/// commands are a documented no-op in that pipeline. This renderer therefore
/// expresses every visible fact (activation, committed binding/availability
/// change, committed collection contents, and rejection) as distinct
/// colored blocks rather than text, so the native window shows a real,
/// unambiguous pixel difference at each step. `draw_text` calls are kept
/// alongside for when a text-capable presenter lands; they are inert today.
fn render_shell_player_frame(shell: &ShellSession, rejected_banner: bool) -> DrawFrame {
    let mut frame = DrawFrame::new();
    frame.clear(Color::rgb(20, 20, 26));

    // Status swatch (top band): color reflects committed node availability.
    let status_color = match shell.node_availability(STATUS_NODE) {
        Some(BridgeAvailability::Available) => Color::rgb(32, 148, 84),
        Some(BridgeAvailability::Hidden) => Color::rgb(96, 96, 104),
        Some(BridgeAvailability::Unavailable) => Color::rgb(176, 48, 48),
        Some(BridgeAvailability::Pending) | Some(BridgeAvailability::Stale) => {
            Color::rgb(150, 130, 40)
        }
        None => Color::rgb(60, 60, 68),
    };
    frame.fill_rect(Rect::new(20, 20, 200, 60), status_color);
    frame.draw_text(
        format!("status: {:?}", shell.node_availability(STATUS_NODE)),
        20,
        90,
        Color::WHITE,
    );

    // Label swatch: dim placeholder until the binding is set, bright blue
    // once the committed text binding has a value.
    let label_text = match shell.binding_value(LABEL_NODE, LABEL_SLOT) {
        Some(BridgeValue::Text(text)) => Some(text),
        _ => None,
    };
    let label_color = if label_text.is_some() {
        Color::rgb(70, 130, 220)
    } else {
        Color::rgb(60, 60, 68)
    };
    frame.fill_rect(Rect::new(240, 20, 200, 60), label_color);
    frame.draw_text(
        format!("label: {}", label_text.as_deref().unwrap_or("(unset)")),
        240,
        90,
        Color::WHITE,
    );

    // One swatch per committed collection entry.
    for (index, _entry) in shell.collection_entries(LIST_NODE).iter().enumerate() {
        let x = 20 + (index as i32) * 70;
        frame.fill_rect(Rect::new(x, 130, 60, 40), Color::rgb(200, 170, 40));
    }
    frame.draw_text(
        format!(
            "collection entries: {:?}",
            shell.collection_entries(LIST_NODE)
        ),
        20,
        180,
        Color::WHITE,
    );

    // Replay-cursor swatch: width encodes the numeric position so the
    // absence of advancement after a rejection is a visible non-change.
    let cursor_width = match shell.replay_cursor() {
        prom_ui_runtime::shell_player::ProjectionReplayCursor::Uninitialized => 10,
        prom_ui_runtime::shell_player::ProjectionReplayCursor::At(n) => 10 + (n.min(50) as u32) * 8,
    };
    frame.fill_rect(
        Rect::new(20, 250, cursor_width, 20),
        Color::rgb(120, 200, 220),
    );
    frame.draw_text(
        format!("replay cursor: {:?}", shell.replay_cursor()),
        20,
        280,
        Color::WHITE,
    );

    // Rejection banner: a red bar that appears without altering any of the
    // swatches above, proving the committed visible state was preserved.
    if rejected_banner {
        frame.fill_rect(Rect::new(20, 300, 420, 30), Color::RED);
        frame.draw_text(
            "last batch REJECTED - committed state unchanged",
            20,
            340,
            Color::RED,
        );
    }

    frame
}

fn session_limits() -> ShellSessionLimits {
    ShellSessionLimits {
        max_transition_stimulus_bytes: 4096,
        max_diagnostics_per_transition: 16,
        max_patches_per_transition: 16,
        max_target_references_per_transition: 16,
    }
}

fn outer_envelope(sequence_no: u64) -> OrderedProjectionPatchBatchEnvelope {
    OrderedProjectionPatchBatchEnvelope {
        session_id: ShellSessionId(1),
        sequence_no,
        patch_count: 1,
        stimulus_bytes: 64,
    }
}

/// Runs the deterministic native Shell Player demonstration until the
/// window is closed or the fixed demonstration timeline completes.
pub fn run_shell_player_demo() {
    println!("=== Semantic UI Shell Player Demo ===");

    let snapshot = demo_activation_snapshot();
    println!(
        "activation snapshot: node_anchors={:?} binding_anchors={:?} collection_anchors={:?}",
        snapshot.node_anchor_ids, snapshot.binding_anchor_ids, snapshot.collection_anchor_ids
    );

    let mut shell = create_shell_session(1, session_limits(), &snapshot);
    let activation = shell.apply(ShellTransitionInput {
        stimulus: ShellLifecycleStimulus::Command(ShellLifecycleCommand::Activate),
        stimulus_bytes: 8,
    });
    println!(
        "1) activated projection: disposition={:?} lifecycle={:?} label={:?} status={:?} cursor={:?}",
        activation.disposition,
        shell.lifecycle(),
        shell.binding_value(LABEL_NODE, LABEL_SLOT),
        shell.node_availability(STATUS_NODE),
        shell.replay_cursor(),
    );

    let config = WindowConfig::new("Semantic Shell Player Demo", 640, 420);
    let backend = NativeBackend::new();
    let mut session = DesktopSessionAlias::create(backend, config)
        .expect("NativeBackend::create_window must succeed");

    let start = Instant::now();
    let mut submitted_valid = false;
    let mut submitted_invalid = false;
    let mut rejected_banner = false;

    session
        .run(move |buf, out_frame| {
            let events = buf.drain();
            for event in &events {
                if matches!(event.kind, InputEventKind::CloseRequested) {
                    *out_frame = render_shell_player_frame(&shell, rejected_banner);
                    return LoopControl::ExitRequested;
                }
            }

            let elapsed = start.elapsed().as_secs_f64();

            if !submitted_valid && elapsed >= 2.0 {
                submitted_valid = true;

                let ops = vec![
                    BridgePatchOperation::SetBindingValue {
                        node: LABEL_NODE,
                        slot: LABEL_SLOT,
                        value: BridgeValue::Text("Hello Shell Player".to_string()),
                    },
                    BridgePatchOperation::SetNodeAvailability {
                        node: STATUS_NODE,
                        availability: BridgeAvailability::Available,
                    },
                    BridgePatchOperation::CollectionInsert {
                        collection: LIST_NODE,
                        key: 1,
                        before: None,
                        value: BridgeValue::Text("Item A".to_string()),
                    },
                    BridgePatchOperation::CollectionInsert {
                        collection: LIST_NODE,
                        key: 2,
                        before: None,
                        value: BridgeValue::Text("Item B".to_string()),
                    },
                ];
                let patch_envelope = BridgePatchEnvelope {
                    patch_id: 1,
                    stream_id: 1,
                    document_id: 1,
                    previous_revision: 0,
                    revision: 1,
                    epoch: 1,
                    sequence: 1,
                };
                let batch =
                    admit_projection_patch_batch(patch_envelope, ops).expect("valid batch admits");
                let manifest = prepared_manifest_snapshot(&batch).expect("manifest derives");

                let result = shell.apply_projection_patch_batch(
                    outer_envelope(1),
                    manifest.target_reference_count,
                    &manifest,
                    &batch,
                );
                println!(
                    "2) submitted valid patch batch (4 operations, {} target references)",
                    manifest.target_reference_count
                );
                println!(
                    "3)+4) committed update: disposition={:?} label={:?} status={:?}",
                    result.disposition,
                    shell.binding_value(LABEL_NODE, LABEL_SLOT),
                    shell.node_availability(STATUS_NODE),
                );
                println!(
                    "   collection entries: {:?}",
                    shell.collection_entries(LIST_NODE)
                );
                println!("6) replay cursor advanced to: {:?}", shell.replay_cursor());
                assert_eq!(
                    result.disposition,
                    ProjectionPatchApplicationDisposition::Applied
                );
            }

            if submitted_valid && !submitted_invalid && elapsed >= 5.0 {
                submitted_invalid = true;

                // Deliberately invalid: node 999 was never declared in the
                // activation snapshot, so stage 5 (stable-target membership)
                // rejects this batch.
                let ops = vec![BridgePatchOperation::SetNodeAvailability {
                    node: 999,
                    availability: BridgeAvailability::Unavailable,
                }];
                let patch_envelope = BridgePatchEnvelope {
                    patch_id: 2,
                    stream_id: 1,
                    document_id: 1,
                    previous_revision: 1,
                    revision: 2,
                    epoch: 1,
                    sequence: 2,
                };
                let batch = admit_projection_patch_batch(patch_envelope, ops)
                    .expect("structurally valid batch admits");
                let manifest = prepared_manifest_snapshot(&batch).expect("manifest derives");

                let cursor_before = shell.replay_cursor();
                let label_before = shell.binding_value(LABEL_NODE, LABEL_SLOT);

                let result = shell.apply_projection_patch_batch(
                    outer_envelope(2),
                    manifest.target_reference_count,
                    &manifest,
                    &batch,
                );
                println!("7) submitted deliberately invalid patch batch (undeclared target 999)");
                println!(
                    "8) rejection diagnostic: disposition={:?} diagnostics={:?}",
                    result.disposition, result.diagnostics
                );
                println!(
                    "   committed visible state unchanged: label={:?} (was {:?})",
                    shell.binding_value(LABEL_NODE, LABEL_SLOT),
                    label_before
                );
                println!(
                    "9) replay cursor NOT advanced: {:?} (was {:?})",
                    shell.replay_cursor(),
                    cursor_before
                );
                assert_eq!(
                    result.disposition,
                    ProjectionPatchApplicationDisposition::Rejected
                );
                assert_eq!(shell.replay_cursor(), cursor_before);
                assert_eq!(shell.binding_value(LABEL_NODE, LABEL_SLOT), label_before);
                rejected_banner = true;
            }

            *out_frame = render_shell_player_frame(&shell, rejected_banner);

            if elapsed >= 120.0 {
                return LoopControl::ExitRequested;
            }
            LoopControl::Continue
        })
        .expect("event loop must succeed");

    let _ = session.close();
    println!("Session closed.");
}

// Local alias so this module reads clearly without repeating the generic
// backend type parameter at every call site.
type DesktopSessionAlias = prom_ui_runtime::DesktopSession<NativeBackend>;
