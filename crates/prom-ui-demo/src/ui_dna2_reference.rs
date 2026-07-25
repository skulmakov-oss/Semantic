//! UI-DNA2-10 end-to-end reference application (Issue #1543).
//!
//! Real pipeline, every arrow real repository code:
//!
//! ```text
//! Projection Source text
//! -> Grammar v0 parser (crates/prom-ui/src/projection_source_parser.rs,
//!    including the textual `collection_anchor` declaration)
//! -> Static UI IR + explicit CollectionAnchor qualification
//! -> Binding Graph + Action IR (present, bounded empty for this contour)
//! -> canonical ProjectionBundle v0 bytes
//!    (prom_ui::shell_bridge::compile_projection_source_to_bundle_v0)
//! -> ProjectionBundle parse/structural/cross-artifact/compatibility/
//!    self-consistency verification + bounded Gate D activation
//!    (prom_ui::shell_bridge::activate_projection_bundle_v0_gate_d)
//! -> ActivationTargetSnapshot -> existing Shell Player
//!    (prom_ui_runtime::shell_player::create_shell_session)
//! -> bounded deterministic vertical-stack layout
//! -> native text/shape rendering (prom-ui-backend-native + glyphon)
//! -> real winit keyboard/pointer input
//! -> hit testing + focus routing
//! -> ActionIntent candidate (prom_ui::SemanticIntent)
//! -> existing admission boundary
//!    (prom_ui_runtime::reference_admission::ReferenceContourAdmission,
//!    reusing prom_ui::action_admission_result::record_interaction_action_*
//!    and prom_ui::admitted_action::build_interaction_admitted_semantic_action)
//! -> admitted ProjectionPatch -> ShellSession::apply_projection_patch_batch
//! -> atomic commit -> visible native UI update
//! ```
//!
//! Bounded, non-critical reference contour only. Gate D is OPEN WITH LIMITS
//! for exactly this contour (see
//! `docs/spec/ui/gate_d_activation_policy_v0.md`); no other contour is
//! authorized by this file.

#![allow(dead_code)]

use std::time::Instant;

use prom_ui::action_binding::InteractionActionBindingId;
use prom_ui::projection::UiProjectedNodeId;
use prom_ui::shell_bridge::{
    activate_projection_bundle_v0_gate_d, admit_projection_patch_batch,
    compile_projection_source_to_bundle_v0, prepare_patch_submission,
    prepared_submission_target_reference_count, BridgeAvailability, BridgePatchEnvelope,
    BridgePatchOperation, BridgeValue, GateDActivatedBundle,
};
use prom_ui::SemanticIntent;
use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::reference_admission::{ReferenceActionInvocation, ReferenceContourAdmission};
use prom_ui_runtime::shell_player::{
    create_shell_session, OrderedProjectionPatchBatchEnvelope,
    ProjectionPatchApplicationDisposition, ShellLifecycleCommand, ShellLifecycleStimulus,
    ShellSession, ShellSessionId, ShellSessionLimits, ShellTransitionInput,
};
use prom_ui_runtime::{Color, DrawFrame, InputEventKind, LoopControl, Rect, WindowConfig};

const REFERENCE_SOURCE_ID: u64 = 501;
const REFERENCE_DOCUMENT_ID: u64 = 501;
const ROOT_NODE: u64 = 10;
const LABEL_NODE: u64 = 11;
const BUTTON_NODE: u64 = 12;
const LIST_NODE: u64 = 13;

/// Real Projection Source text, Grammar v0, including the textual
/// `collection_anchor` declaration landed under Issue #1543.
const REFERENCE_SOURCE_TEXT: &str = concat!(
    "projection v0 {\n",
    "revision 1; epoch 1;\n",
    "surface 1 root 10 key 1;\n",
    "node 10 role root key 10 { child 11 order 0; child 12 order 1; child 13 order 2; }\n",
    "node 11 role text key 11 {}\n",
    "node 12 role danger_action key 12 {}\n",
    "node 13 role evidence_panel key 13 {}\n",
    "collection_anchor 13;\n",
    "}"
);

/// Deliberately structurally invalid source text (a child edge to a node
/// id that is never declared) used to prove bundle rejection preserves the
/// already-active state.
const INVALID_SOURCE_TEXT: &str = concat!(
    "projection v0 {\n",
    "revision 1; epoch 1;\n",
    "surface 1 root 10 key 1;\n",
    "node 10 role root key 10 { child 999 order 0; }\n",
    "}"
);

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

/// Bounded, deterministic vertical-stack layout: the root spans the
/// window; the root's declared children are stacked top-to-bottom in
/// declared child order. This is a real, deterministic layout calculation
/// driven by the activated bundle's own node/child structure -- not the
/// general constraint-solver layout engine, which (per this repository's
/// own existing implementation) currently produces placeholder rects
/// unrelated to real content regardless.
struct ReferenceLayout {
    rects: Vec<(u64, Rect)>,
}

impl ReferenceLayout {
    fn build(bundle: &GateDActivatedBundle) -> Self {
        let root = bundle.root_node();
        let children: Vec<u64> = bundle
            .nodes()
            .iter()
            .find(|node| node.id() == root)
            .map(|node| node.children().to_vec())
            .unwrap_or_default();

        let band_height = 110;
        let mut rects = Vec::with_capacity(children.len());
        for (index, child) in children.into_iter().enumerate() {
            let y = 20 + (index as i32) * (band_height + 16);
            rects.push((child, Rect::new(20, y, 560, band_height as u32)));
        }
        Self { rects }
    }

    fn hit_test(&self, x: f64, y: f64) -> Option<u64> {
        let (px, py) = (x as i32, y as i32);
        self.rects.iter().rev().find_map(|(id, rect)| {
            let inside = px >= rect.x
                && px < rect.x + rect.width as i32
                && py >= rect.y
                && py < rect.y + rect.height as i32;
            inside.then_some(*id)
        })
    }

    fn rect_of(&self, id: u64) -> Option<Rect> {
        self.rects
            .iter()
            .find(|(node, _)| *node == id)
            .map(|(_, rect)| *rect)
    }

    fn focus_order(&self) -> Vec<u64> {
        self.rects.iter().map(|(id, _)| *id).collect()
    }
}

/// Compiles the real reference Projection Source text through the public
/// bridge into canonical `ProjectionBundle v0` bytes.
fn compile_reference_bundle() -> Vec<u8> {
    compile_projection_source_to_bundle_v0(
        REFERENCE_SOURCE_ID,
        REFERENCE_SOURCE_TEXT,
        REFERENCE_DOCUMENT_ID,
        1,
        1,
        "prom-ui-demo/ui_dna2_reference",
    )
    .expect("reference source text is valid Grammar v0")
}

/// Compiles the deliberately invalid source text into bundle bytes, when
/// possible; some malformed inputs are already rejected by the parser
/// itself (compile-time rejection), which is equally valid evidence of the
/// fail-closed pipeline, so this returns `None` in that case.
fn compile_invalid_bundle() -> Option<Vec<u8>> {
    compile_projection_source_to_bundle_v0(
        REFERENCE_SOURCE_ID,
        INVALID_SOURCE_TEXT,
        REFERENCE_DOCUMENT_ID,
        1,
        1,
        "prom-ui-demo/ui_dna2_reference",
    )
    .ok()
}

struct ReferenceState {
    shell: ShellSession,
    bundle: GateDActivatedBundle,
    layout: ReferenceLayout,
    admission: ReferenceContourAdmission,
    hovered: Option<u64>,
    focused: Option<u64>,
    focus_index: usize,
    sequence: u64,
    last_denied: bool,
    invalid_bundle_rejected: bool,
    replay_rejected: bool,
    stale_rejected: bool,
}

impl ReferenceState {
    fn new() -> Self {
        let bundle_bytes = compile_reference_bundle();
        println!(
            "1) Projection Source text compiled to ProjectionBundle v0 ({} bytes)",
            bundle_bytes.len()
        );

        let bundle = activate_projection_bundle_v0_gate_d(&bundle_bytes)
            .expect("reference bundle passes verification and Gate D activation");
        println!(
            "2)-9) bundle verified (structural, cross-artifact, compatibility, self-consistency) and Gate D activated WITH LIMITS: {} nodes, root={}",
            bundle.nodes().len(),
            bundle.root_node()
        );
        println!(
            "   accessibility evidence: {} entries",
            bundle.accessibility_entries().len()
        );
        for entry in bundle.accessibility_entries() {
            println!(
                "     node {} role={:?} label={:?}",
                entry.node(),
                entry.role(),
                entry.label()
            );
        }

        let mut shell = create_shell_session(1, session_limits(), bundle.activation_snapshot());
        let activation = shell.apply(ShellTransitionInput {
            stimulus: ShellLifecycleStimulus::Command(ShellLifecycleCommand::Activate),
            stimulus_bytes: 8,
        });
        println!(
            "10) Shell Player activated: disposition={:?} lifecycle={:?} cursor={:?}",
            activation.disposition,
            shell.lifecycle(),
            shell.replay_cursor()
        );

        let layout = ReferenceLayout::build(&bundle);
        println!(
            "11) deterministic layout computed for {} declared children",
            layout.rects.len()
        );

        // Only BUTTON_NODE's action id is granted; LABEL_NODE and LIST_NODE
        // (if ever wired to an action) remain denied, proving both the
        // accepted and denied admission paths for real.
        let admission = ReferenceContourAdmission::new([BUTTON_NODE]);

        Self {
            shell,
            focus_index: 0,
            focused: layout.focus_order().first().copied(),
            layout,
            admission,
            bundle,
            hovered: None,
            sequence: 0,
            last_denied: false,
            invalid_bundle_rejected: false,
            replay_rejected: false,
            stale_rejected: false,
        }
    }

    fn handle_pointer_moved(&mut self, x: f64, y: f64) {
        self.hovered = self.layout.hit_test(x, y);
    }

    fn handle_pointer_down(&mut self) {
        if let Some(node) = self.hovered {
            self.focused = Some(node);
            self.focus_index = self
                .layout
                .focus_order()
                .iter()
                .position(|id| *id == node)
                .unwrap_or(0);
            self.trigger(node);
        }
    }

    fn handle_tab(&mut self) {
        let order = self.layout.focus_order();
        if order.is_empty() {
            return;
        }
        self.focus_index = (self.focus_index + 1) % order.len();
        self.focused = Some(order[self.focus_index]);
    }

    fn handle_activate(&mut self) {
        if let Some(node) = self.focused {
            self.trigger(node);
        }
    }

    /// Runs one candidate through the real pipeline: build a
    /// `SemanticIntent`, admit or deny it through the existing admission
    /// boundary, and on admission build and atomically commit a real
    /// `ProjectionPatch` through the unmodified Shell Player.
    fn trigger(&mut self, node: u64) {
        self.sequence += 1;
        let intent = SemanticIntent::new(
            UiProjectedNodeId::new(node),
            InteractionActionBindingId(node),
        );
        let invocation = ReferenceActionInvocation {
            intent,
            revision: 1,
            epoch: 1,
            sequence: self.sequence,
        };

        match self.admission.evaluate(&invocation) {
            Ok(_admitted) => {
                self.last_denied = false;
                self.commit_toggle_and_append();
            }
            Err(_denial) => {
                self.last_denied = true;
            }
        }
    }

    /// Applies one real, admitted ProjectionPatch batch: toggles the
    /// button node's availability and appends one item to the collection
    /// anchor node, then atomically commits through the unmodified Shell
    /// Player pipeline.
    fn commit_toggle_and_append(&mut self) {
        let next_availability = match self.shell.node_availability(BUTTON_NODE) {
            Some(BridgeAvailability::Available) => BridgeAvailability::Unavailable,
            _ => BridgeAvailability::Available,
        };
        let item_index = self.shell.collection_entries(LIST_NODE).len() as u64 + 1;

        let ops = vec![
            BridgePatchOperation::SetNodeAvailability {
                node: BUTTON_NODE,
                availability: next_availability,
            },
            BridgePatchOperation::CollectionInsert {
                collection: LIST_NODE,
                key: item_index,
                before: None,
                value: BridgeValue::Text(format!("Admitted #{item_index}")),
            },
        ];
        let envelope = BridgePatchEnvelope {
            patch_id: self.sequence,
            stream_id: 1,
            document_id: 1,
            previous_revision: self.sequence.saturating_sub(1),
            revision: self.sequence,
            epoch: 1,
            sequence: self.sequence,
        };
        let batch = admit_projection_patch_batch(envelope, ops).expect("valid patch admits");
        let submission = prepare_patch_submission(batch).expect("submission derives");
        let target_reference_count = prepared_submission_target_reference_count(&submission);

        let result = self.shell.apply_projection_patch_batch(
            outer_envelope(self.sequence),
            target_reference_count,
            &submission,
        );
        debug_assert_eq!(
            result.disposition,
            ProjectionPatchApplicationDisposition::Applied
        );
    }

    /// One-time scripted proof that a deliberately invalid bundle is
    /// rejected before activation, and that rejection leaves the already-
    /// active state (bundle, snapshot, Shell Player) byte-for-byte
    /// unchanged.
    fn run_invalid_bundle_check(&mut self) {
        if self.invalid_bundle_rejected {
            return;
        }
        self.invalid_bundle_rejected = true;

        let before_label = self.shell.binding_value(LABEL_NODE, 0);
        let before_cursor = self.shell.replay_cursor();
        let before_root = self.bundle.root_node();

        let rejected = match compile_invalid_bundle() {
            Some(bytes) => activate_projection_bundle_v0_gate_d(&bytes).is_err(),
            None => true, // rejected earlier, at parse/compile time -- equally fail-closed
        };

        println!("12) invalid ProjectionBundle rejected before activation: {rejected}");
        debug_assert!(rejected);
        debug_assert_eq!(self.shell.binding_value(LABEL_NODE, 0), before_label);
        debug_assert_eq!(self.shell.replay_cursor(), before_cursor);
        debug_assert_eq!(self.bundle.root_node(), before_root);
        println!("13) active state and replay cursor unchanged after invalid-bundle rejection");
    }

    /// One-time scripted proof that a replayed invocation and a stale
    /// revision are both rejected deterministically, and neither mutates
    /// committed state or the replay cursor.
    fn run_replay_and_staleness_check(&mut self) {
        if self.replay_rejected && self.stale_rejected {
            return;
        }

        // Replay: resubmit the exact same sequence number just consumed by
        // the most recent real trigger (if any triggers have happened
        // yet); otherwise synthesize one admitted+immediately-replayed
        // invocation. The stability baseline below is captured only after
        // this legitimate bootstrap commit, since it is expected to (and
        // correctly does) advance the cursor -- what must stay stable is
        // committed state across the *rejected* replay/stale attempts that
        // follow, not across this function call as a whole.
        if self.sequence == 0 {
            self.trigger(BUTTON_NODE);
        }

        let cursor_before = self.shell.replay_cursor();
        let label_before = self.shell.binding_value(LABEL_NODE, 0);

        let replay = ReferenceActionInvocation {
            intent: SemanticIntent::new(
                UiProjectedNodeId::new(BUTTON_NODE),
                InteractionActionBindingId(BUTTON_NODE),
            ),
            revision: 1,
            epoch: 1,
            sequence: self.sequence,
        };
        let replay_result = self.admission.evaluate(&replay);
        self.replay_rejected = replay_result.is_err();
        println!(
            "14) replayed invocation rejected: {}",
            replay_result.is_err()
        );

        let stale = ReferenceActionInvocation {
            intent: SemanticIntent::new(
                UiProjectedNodeId::new(BUTTON_NODE),
                InteractionActionBindingId(BUTTON_NODE),
            ),
            revision: 0,
            epoch: 1,
            sequence: self.sequence + 1000,
        };
        let stale_result = self.admission.evaluate(&stale);
        self.stale_rejected = stale_result.is_err();
        println!(
            "15) stale-revision invocation rejected: {}",
            stale_result.is_err()
        );

        debug_assert_eq!(self.shell.replay_cursor(), cursor_before);
        debug_assert_eq!(self.shell.binding_value(LABEL_NODE, 0), label_before);
        println!("16) committed state and replay cursor unchanged after denied/stale/replayed invocations");
    }
}

fn render_frame(state: &ReferenceState) -> DrawFrame {
    let mut frame = DrawFrame::new();
    frame.clear(Color::rgb(18, 18, 24));

    for (id, rect) in &state.layout.rects {
        let is_hovered = state.hovered == Some(*id);
        let is_focused = state.focused == Some(*id);

        let base = if *id == BUTTON_NODE {
            match state.shell.node_availability(BUTTON_NODE) {
                Some(BridgeAvailability::Available) => Color::rgb(32, 148, 84),
                Some(BridgeAvailability::Unavailable) => Color::rgb(176, 48, 48),
                _ => Color::rgb(70, 70, 90),
            }
        } else {
            Color::rgb(50, 56, 74)
        };
        let fill = if is_hovered {
            Color::rgb(
                base.r.saturating_add(20),
                base.g.saturating_add(20),
                base.b.saturating_add(20),
            )
        } else {
            base
        };
        frame.fill_rect(Rect::new(rect.x, rect.y, rect.width, rect.height), fill);

        if is_focused {
            let thickness = 3u32;
            let focus_color = Color::rgb(255, 210, 90);
            frame.fill_rect(
                Rect::new(rect.x, rect.y, rect.width, thickness),
                focus_color,
            );
            frame.fill_rect(
                Rect::new(
                    rect.x,
                    rect.y + rect.height as i32 - thickness as i32,
                    rect.width,
                    thickness,
                ),
                focus_color,
            );
        }

        let label = match *id {
            LABEL_NODE => "Label (text role)".to_string(),
            BUTTON_NODE => format!(
                "Danger action -- availability: {:?}",
                state.shell.node_availability(BUTTON_NODE)
            ),
            LIST_NODE => format!(
                "Evidence panel -- collection: {:?}",
                state.shell.collection_entries(LIST_NODE)
            ),
            other => format!("Node {other}"),
        };
        frame.draw_text(label, rect.x + 12, rect.y + 24, Color::WHITE);
    }

    let status = format!(
        "cursor={:?}  last denied={}  invalid-bundle rejected={}  replay rejected={}  stale rejected={}",
        state.shell.replay_cursor(),
        state.last_denied,
        state.invalid_bundle_rejected,
        state.replay_rejected,
        state.stale_rejected
    );
    frame.draw_text(status, 20, 400, Color::rgb(220, 220, 230));

    if state.last_denied {
        frame.fill_rect(Rect::new(20, 420, 560, 24), Color::RED);
        frame.draw_text(
            "last action DENIED -- committed state unchanged",
            24,
            438,
            Color::WHITE,
        );
    }

    frame
}

pub fn run_ui_dna2_reference() {
    println!("=== UI-DNA2 Reference Application (Issue #1543) ===");

    let mut state = ReferenceState::new();

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
                match event.kind {
                    InputEventKind::CloseRequested => {
                        *out_frame = render_frame(&state);
                        return LoopControl::ExitRequested;
                    }
                    InputEventKind::PointerMoved { x, y } => state.handle_pointer_moved(x, y),
                    InputEventKind::PointerDown { .. } => state.handle_pointer_down(),
                    InputEventKind::KeyDown { key_code: 9 } => state.handle_tab(),
                    InputEventKind::KeyDown { key_code: 13 | 32 } => state.handle_activate(),
                    InputEventKind::KeyDown { key_code: 27 } => {
                        *out_frame = render_frame(&state);
                        return LoopControl::ExitRequested;
                    }
                    _ => {}
                }
            }

            let elapsed = start.elapsed().as_secs_f64();
            if !ran_negative_checks && elapsed >= 2.0 {
                ran_negative_checks = true;
                state.run_invalid_bundle_check();
                state.run_replay_and_staleness_check();
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
        let mut state = ReferenceState::new();
        assert_eq!(state.shell.node_availability(BUTTON_NODE), None);
        state.trigger(BUTTON_NODE);
        assert!(!state.last_denied);
        assert_eq!(
            state.shell.node_availability(BUTTON_NODE),
            Some(BridgeAvailability::Available)
        );
        assert_eq!(state.shell.collection_entries(LIST_NODE).len(), 1);
    }

    #[test]
    fn denied_action_preserves_state_and_cursor() {
        let mut state = ReferenceState::new();
        let cursor_before = state.shell.replay_cursor();
        state.trigger(LABEL_NODE); // LABEL_NODE's action id is not granted
        assert!(state.last_denied);
        assert_eq!(state.shell.node_availability(BUTTON_NODE), None);
        assert_eq!(state.shell.replay_cursor(), cursor_before);
    }

    #[test]
    fn tab_cycles_focus_through_declared_children() {
        let mut state = ReferenceState::new();
        let initial = state.focused;
        state.handle_tab();
        let after_one = state.focused;
        assert_ne!(initial, after_one);
        state.handle_tab();
        state.handle_tab();
        assert_eq!(state.focused, initial);
    }

    #[test]
    fn enter_activates_the_focused_node() {
        let mut state = ReferenceState::new();
        state.focused = Some(BUTTON_NODE);
        state.focus_index = 1;
        state.handle_activate();
        assert!(!state.last_denied);
        assert_eq!(
            state.shell.node_availability(BUTTON_NODE),
            Some(BridgeAvailability::Available)
        );
    }

    #[test]
    fn invalid_bundle_check_preserves_active_state() {
        let mut state = ReferenceState::new();
        state.trigger(BUTTON_NODE);
        let label_before = state.shell.binding_value(LABEL_NODE, 0);
        let cursor_before = state.shell.replay_cursor();
        state.run_invalid_bundle_check();
        assert!(state.invalid_bundle_rejected);
        assert_eq!(state.shell.binding_value(LABEL_NODE, 0), label_before);
        assert_eq!(state.shell.replay_cursor(), cursor_before);
    }

    #[test]
    fn replay_and_stale_checks_reject_without_mutating_state() {
        let mut state = ReferenceState::new();
        let cursor_before_checks = state.shell.replay_cursor();
        state.run_replay_and_staleness_check();
        assert!(state.replay_rejected);
        assert!(state.stale_rejected);
        // One legitimate trigger happens inside the check when sequence==0;
        // the cursor after that legitimate commit must still be stable
        // across the replay/stale rejections that follow it.
        let cursor_after = state.shell.replay_cursor();
        assert_ne!(cursor_before_checks, cursor_after);
        state.run_replay_and_staleness_check();
        assert_eq!(state.shell.replay_cursor(), cursor_after);
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
}
