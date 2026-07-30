use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
use prom_ui_runtime::reference_contour::ReferenceLayout;
use prom_ui_runtime::shell_player::{
    create_shell_session, OrderedProjectionPatchBatchEnvelope,
    ProjectionPatchApplicationDisposition, ShellLifecycleCommand, ShellLifecycleStimulus,
    ShellSession, ShellSessionId, ShellSessionLimits, ShellTransitionInput,
};
use prom_ui_runtime::{
    Color, DesktopSession, DrawFrame, InputEventKind, LoopControl, Rect, SessionState, WindowConfig,
};
use semantic_core_quad::logic_frame;
use semantic_core_quad::QuadState;
use sm_emit::compile_program_to_semcode;
use sm_runtime_core::RecordCarrier;
use sm_vm::{QuadVal, Value};

pub const PROJECTION_SOURCE: &str = include_str!("quad_calc.proj.sm");
pub const CALCULATOR_SEMANTIC_SOURCE: &str = include_str!("calculator.sm");

pub const ROOT_NODE: u64 = 10;
pub const READOUT_NODE: u64 = 11;
pub const TEXT_NODE: u64 = 12;
pub const EVIDENCE_NODE: u64 = 13;

pub const BTN_0: u64 = 20; pub const BTN_1: u64 = 21; pub const BTN_2: u64 = 22; pub const BTN_3: u64 = 23; pub const BTN_4: u64 = 24;
pub const BTN_5: u64 = 25; pub const BTN_6: u64 = 26; pub const BTN_7: u64 = 27; pub const BTN_8: u64 = 28; pub const BTN_9: u64 = 29;

pub const BTN_ADD: u64 = 30; pub const BTN_SUB: u64 = 31; pub const BTN_MUL: u64 = 32; pub const BTN_DIV: u64 = 33;

pub const BTN_N: u64 = 40; pub const BTN_F: u64 = 41; pub const BTN_T: u64 = 42; pub const BTN_S: u64 = 43;

pub const BTN_NOT: u64 = 50; pub const BTN_AND: u64 = 51; pub const BTN_OR: u64 = 52; 
pub const BTN_IMP: u64 = 53; pub const BTN_EQ: u64 = 54; pub const BTN_NEQ: u64 = 55;

pub const BTN_CLEAR: u64 = 60; pub const BTN_BACKSPACE: u64 = 61; pub const BTN_EVAL: u64 = 62; 
pub const BTN_MODE: u64 = 63; pub const BTN_RECOVER: u64 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalcMode {
    Arithmetic = 0,
    QuadLogic = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadOp {
    Not = 0,
    And = 1,
    Or = 2,
    Imp = 3,
    Eq = 4,
    Neq = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithOp {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
}

/// Structured Calculator State Mirror
/// Authoritative state resides strictly in `.sm` state machine transitions.
/// Rust maintains this read-only mirror decoded from structured Semantic VM return values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CalculatorState {
    pub mode: CalcMode,
    pub current_input: i32,
    pub stored_operand: Option<i32>,
    pub pending_operator: Option<ArithOp>,
    pub numeric_result: i32,

    pub quad_left: QuadState,
    pub quad_right: QuadState,
    pub quad_operator: QuadOp,
    pub quad_result: QuadState,

    pub evaluation_state: QuadState, // N, F, T, S
    pub error_code: i32,
    pub error_state: Option<String>,
    pub replace_input: bool,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            mode: CalcMode::Arithmetic,
            current_input: 0,
            stored_operand: None,
            pending_operator: None,
            numeric_result: 0,
            quad_left: QuadState::T,
            quad_right: QuadState::F,
            quad_operator: QuadOp::And,
            quad_result: QuadState::N,
            evaluation_state: QuadState::N,
            error_code: 0,
            error_state: None,
            replace_input: false,
        }
    }
}

pub fn quad_val_to_quad_state(q: QuadVal) -> QuadState {
    match q {
        QuadVal::N => QuadState::N,
        QuadVal::F => QuadState::F,
        QuadVal::T => QuadState::T,
        QuadVal::S => QuadState::S,
    }
}

pub fn quad_state_to_quad_val(qs: QuadState) -> QuadVal {
    match qs {
        QuadState::N => QuadVal::N,
        QuadState::F => QuadVal::F,
        QuadState::T => QuadVal::T,
        QuadState::S => QuadVal::S,
    }
}

pub fn encode_semantic_state(state: &CalculatorState) -> Value {
    Value::Record(RecordCarrier {
        type_name: "CalculatorState".into(),
        slots: vec![
            Value::I32(state.mode as i32),
            Value::I32(state.current_input),
            Value::I32(state.stored_operand.unwrap_or(0)),
            Value::Bool(state.stored_operand.is_some()),
            Value::I32(state.pending_operator.map(|op| op as i32).unwrap_or(0)),
            Value::Bool(state.pending_operator.is_some()),
            Value::I32(state.numeric_result),
            Value::Quad(quad_state_to_quad_val(state.quad_left)),
            Value::Quad(quad_state_to_quad_val(state.quad_right)),
            Value::I32(state.quad_operator as i32),
            Value::Quad(quad_state_to_quad_val(state.quad_result)),
            Value::Quad(quad_state_to_quad_val(state.evaluation_state)),
            Value::I32(state.error_code),
            Value::Bool(state.replace_input),
        ],
    })
}

pub fn decode_semantic_state(val: &Value) -> Result<CalculatorState, String> {
    let Value::Record(RecordCarrier { type_name, slots }) = val else {
        return Err("Expected Record value for CalculatorState".into());
    };
    if type_name != "CalculatorState" || slots.len() < 14 {
        return Err(format!("Invalid CalculatorState record layout: {:?}", val));
    }

    let mode = match slots[0] {
        Value::I32(0) => CalcMode::Arithmetic,
        Value::I32(1) => CalcMode::QuadLogic,
        _ => CalcMode::Arithmetic,
    };
    let current_input = match slots[1] {
        Value::I32(v) => v,
        _ => 0,
    };
    let stored_val = match slots[2] {
        Value::I32(v) => v,
        _ => 0,
    };
    let has_stored = match slots[3] {
        Value::Bool(b) => b,
        _ => false,
    };
    let pending_op_int = match slots[4] {
        Value::I32(v) => v,
        _ => 0,
    };
    let has_pending = match slots[5] {
        Value::Bool(b) => b,
        _ => false,
    };
    let numeric_result = match slots[6] {
        Value::I32(v) => v,
        _ => 0,
    };
    let quad_left = match slots[7] {
        Value::Quad(q) => quad_val_to_quad_state(q),
        _ => QuadState::N,
    };
    let quad_right = match slots[8] {
        Value::Quad(q) => quad_val_to_quad_state(q),
        _ => QuadState::N,
    };
    let quad_operator_int = match slots[9] {
        Value::I32(v) => v,
        _ => 0,
    };
    let quad_result = match slots[10] {
        Value::Quad(q) => quad_val_to_quad_state(q),
        _ => QuadState::N,
    };
    let evaluation_state = match slots[11] {
        Value::Quad(q) => quad_val_to_quad_state(q),
        _ => QuadState::N,
    };
    let error_code = match slots[12] {
        Value::I32(v) => v,
        _ => 0,
    };
    let replace_input = match slots[13] {
        Value::Bool(b) => b,
        _ => false,
    };

    let stored_operand = if has_stored { Some(stored_val) } else { None };
    let pending_operator = if has_pending {
        match pending_op_int {
            0 => Some(ArithOp::Add),
            1 => Some(ArithOp::Sub),
            2 => Some(ArithOp::Mul),
            3 => Some(ArithOp::Div),
            _ => None,
        }
    } else {
        None
    };

    let quad_operator = match quad_operator_int {
        0 => QuadOp::Not,
        1 => QuadOp::And,
        2 => QuadOp::Or,
        3 => QuadOp::Imp,
        4 => QuadOp::Eq,
        5 => QuadOp::Neq,
        _ => QuadOp::And,
    };

    let error_state = match error_code {
        1 => Some("DIV_BY_ZERO_REJECTED".into()),
        2 => Some("CONTRADICTION_STATE_S".into()),
        _ => None,
    };

    Ok(CalculatorState {
        mode,
        current_input,
        stored_operand,
        pending_operator,
        numeric_result,
        quad_left,
        quad_right,
        quad_operator,
        quad_result,
        evaluation_state,
        error_code,
        error_state,
        replace_input,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Digit(i32),
    OpArith(ArithOp),
    SelectQuadLeft(QuadState),
    SelectQuadRight(QuadState),
    SelectQuadOp(QuadOp),
    Evaluate,
    Clear,
    Backspace,
    SwitchMode(CalcMode),
    Recover,
    Contradiction,
}

pub fn encode_semantic_action(action: &Action) -> Value {
    let (kind, payload, quad_val) = match action {
        Action::Digit(d) => (0, *d, QuadVal::N),
        Action::OpArith(op) => (1, *op as i32, QuadVal::N),
        Action::Evaluate => (2, 0, QuadVal::N),
        Action::Clear => (3, 0, QuadVal::N),
        Action::Backspace => (4, 0, QuadVal::N),
        Action::SwitchMode(m) => (5, *m as i32, QuadVal::N),
        Action::Recover => (6, 0, QuadVal::N),
        Action::Contradiction => (7, 0, QuadVal::N),
        Action::SelectQuadLeft(q) => (8, 0, quad_state_to_quad_val(*q)),
        Action::SelectQuadRight(q) => (9, 0, quad_state_to_quad_val(*q)),
        Action::SelectQuadOp(op) => (10, *op as i32, QuadVal::N),
    };

    Value::Record(RecordCarrier {
        type_name: "CalculatorAction".into(),
        slots: vec![Value::I32(kind), Value::I32(payload), Value::Quad(quad_val)],
    })
}

pub struct CalculatorApp {
    pub state: CalculatorState,
    pub shell: ShellSession,
    pub bundle: GateDActivatedBundle,
    pub admission: ReferenceContourAdmission,
    pub semcode_bytes: Vec<u8>,
    pub sequence: u64,
    pub last_denied: bool,
    pub focused_node: Option<u64>,
    pub pointer_pos: (f64, f64),
}

impl CalculatorApp {
    pub fn new() -> Result<Self, String> {
        let bundle_bytes = compile_projection_source_to_bundle_v0(
            501,
            PROJECTION_SOURCE,
            501,
            1,
            1,
            "quad_logic_calculator",
        )
        .map_err(|e| format!("Projection compile failed: {e:?}"))?;

        let bundle = activate_projection_bundle_v0_gate_d(&bundle_bytes)
            .map_err(|e| format!("Gate D activation failed: {e:?}"))?;

        let limits = ShellSessionLimits {
            max_transition_stimulus_bytes: 4096,
            max_diagnostics_per_transition: 16,
            max_patches_per_transition: 16,
            max_target_references_per_transition: 16,
        };

        let mut shell = create_shell_session(1, limits, bundle.activation_snapshot());
        let _ = shell.apply(ShellTransitionInput {
            stimulus: ShellLifecycleStimulus::Command(ShellLifecycleCommand::Activate),
            stimulus_bytes: 8,
        });

        let semcode_bytes = compile_program_to_semcode(CALCULATOR_SEMANTIC_SOURCE)
            .map_err(|e| format!("Semantic compile failed: {e:?}"))?;

        let mut admission_nodes = vec![
            READOUT_NODE, TEXT_NODE, EVIDENCE_NODE,
            BTN_0, BTN_1, BTN_2, BTN_3, BTN_4, BTN_5, BTN_6, BTN_7, BTN_8, BTN_9,
            BTN_ADD, BTN_SUB, BTN_MUL, BTN_DIV,
            BTN_N, BTN_F, BTN_T, BTN_S,
            BTN_NOT, BTN_AND, BTN_OR, BTN_IMP, BTN_EQ, BTN_NEQ,
            BTN_CLEAR, BTN_BACKSPACE, BTN_EVAL, BTN_MODE, BTN_RECOVER,
        ];
        
        let admission = ReferenceContourAdmission::new(admission_nodes);

        Ok(Self {
            state: CalculatorState::default(),
            shell,
            bundle,
            admission,
            semcode_bytes,
            sequence: 0,
            last_denied: false,
            focused_node: Some(READOUT_NODE),
            pointer_pos: (0.0, 0.0),
        })
    }

    pub fn dispatch_action(&mut self, action: Action) -> Result<(), String> {
        self.sequence += 1;
        let node_id = match action {
            Action::Recover => BTN_RECOVER,
            Action::Clear => BTN_CLEAR,
            _ => READOUT_NODE,
        };

        let intent = SemanticIntent::new(
            UiProjectedNodeId::new(node_id),
            InteractionActionBindingId(node_id),
        );
        let invocation = ReferenceActionInvocation {
            intent,
            revision: 1,
            epoch: 1,
            sequence: self.sequence,
        };

        if self.admission.evaluate(&invocation).is_err() {
            self.last_denied = true;
            return Ok(());
        }
        self.last_denied = false;

        println!("[PROOF] InputEvent -> SemanticIntent (Action: {:?}) -> Admitted action", action);
        println!("[PROOF] Current Semantic state: {:?}", self.state);

        // Structured Semantic VM execution: passes current state + action, receives next state record
        self.execute_semantic_action(action)?;
        self.apply_projection_patches();
        Ok(())
    }

    fn execute_semantic_action(&mut self, action: Action) -> Result<(), String> {
        // Step 1: Mandatory bytecode verification
        let ver_semcode = sm_verify::verify_semcode_token(&self.semcode_bytes)
            .map_err(|e| format!("Semantic verifier rejection: {e:?}"))?;

        let entry = ver_semcode
            .require_entry("apply_action")
            .map_err(|e| format!("Semantic entry resolution error: {e:?}"))?;

        // Step 2: Encode typed action and current state
        let state_val = encode_semantic_state(&self.state);
        let action_val = encode_semantic_action(&action);

        // Step 3: Structured verified VM invocation (no Rust transition calculations)
        let next_state_val = sm_vm::run_verified_function_semcode_with_args(
            &entry,
            "apply_action",
            vec![state_val, action_val],
        )
        .map_err(|e| format!("Semantic VM execution error: {e:?}"))?;

        // Step 4: Replace read-only mirror with decoded Semantic state
        self.state = decode_semantic_state(&next_state_val)?;
        println!("[PROOF] Structured next Semantic state: {:?}", self.state);
        Ok(())
    }

    fn apply_projection_patches(&mut self) {
        let evidence_val = match &self.state.error_state {
            Some(err) => format!(
                "ERROR: {err} | EvalState: {:?}",
                self.state.evaluation_state
            ),
            None => format!(
                "EvalState: {:?} | Result: {}",
                self.state.evaluation_state,
                if self.state.mode == CalcMode::Arithmetic {
                    self.state.numeric_result.to_string()
                } else {
                    format!("{:?}", self.state.quad_result)
                }
            ),
        };

        let item_idx = self.shell.collection_entries(EVIDENCE_NODE).len() as u64 + 1;
        let ops = vec![
            BridgePatchOperation::CollectionInsert {
                collection: EVIDENCE_NODE,
                key: item_idx,
                before: None,
                value: BridgeValue::Text(evidence_val),
            },
            BridgePatchOperation::SetNodeAvailability {
                node: BTN_CLEAR,
                availability: BridgeAvailability::Available,
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

        println!("[PROOF] Projection patch generated for UI update: {:?}", envelope);

        if let Ok(batch) = admit_projection_patch_batch(envelope, ops) {
            if let Ok(submission) = prepare_patch_submission(batch) {
                let target_ref_count = prepared_submission_target_reference_count(&submission);
                let outer_env = OrderedProjectionPatchBatchEnvelope {
                    session_id: ShellSessionId(1),
                    sequence_no: self.sequence,
                    patch_count: 1,
                    stimulus_bytes: 64,
                };
                let res = self.shell.apply_projection_patch_batch(
                    outer_env,
                    target_ref_count,
                    &submission,
                );
                debug_assert_eq!(
                    res.disposition,
                    ProjectionPatchApplicationDisposition::Applied
                );
            }
        }
    }

    pub fn compute_projection_digest(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.state.hash(&mut hasher);
        self.sequence.hash(&mut hasher);
        self.last_denied.hash(&mut hasher);
        self.focused_node.hash(&mut hasher);
        hasher.finish()
    }

    fn get_buttons(&self) -> Vec<(u64, Rect, &'static str, Color, Action)> {
        let mut btns = Vec::new();
        let start_x = 20; let start_y = 150; let width = 80; let height = 50; let spacing = 10;
        let btn_color = Color::rgb(45, 50, 70);
        let action_color = Color::rgb(220, 80, 80);
        let op_color = Color::rgb(50, 150, 200);
        let quad_color = Color::rgb(180, 130, 50);

        let r = |c: i32, r: i32, w_span: i32| Rect::new(start_x + c * (width + spacing), start_y + r * (height + spacing), (width * w_span + spacing * (w_span - 1)) as u32, height as u32);

        let next_mode = match self.state.mode {
            CalcMode::Arithmetic => CalcMode::QuadLogic,
            CalcMode::QuadLogic => CalcMode::Arithmetic,
        };
        btns.push((BTN_MODE, r(0, 0, 2), "MODE", action_color, Action::SwitchMode(next_mode)));
        btns.push((BTN_CLEAR, r(2, 0, 1), "C", action_color, Action::Clear));
        btns.push((BTN_RECOVER, r(3, 0, 1), "REC", action_color, Action::Recover));

        if self.state.mode == CalcMode::Arithmetic {
            btns.push((BTN_7, r(0, 1, 1), "7", btn_color, Action::Digit(7)));
            btns.push((BTN_8, r(1, 1, 1), "8", btn_color, Action::Digit(8)));
            btns.push((BTN_9, r(2, 1, 1), "9", btn_color, Action::Digit(9)));
            btns.push((BTN_DIV, r(3, 1, 1), "/", op_color, Action::OpArith(ArithOp::Div)));

            btns.push((BTN_4, r(0, 2, 1), "4", btn_color, Action::Digit(4)));
            btns.push((BTN_5, r(1, 2, 1), "5", btn_color, Action::Digit(5)));
            btns.push((BTN_6, r(2, 2, 1), "6", btn_color, Action::Digit(6)));
            btns.push((BTN_MUL, r(3, 2, 1), "*", op_color, Action::OpArith(ArithOp::Mul)));

            btns.push((BTN_1, r(0, 3, 1), "1", btn_color, Action::Digit(1)));
            btns.push((BTN_2, r(1, 3, 1), "2", btn_color, Action::Digit(2)));
            btns.push((BTN_3, r(2, 3, 1), "3", btn_color, Action::Digit(3)));
            btns.push((BTN_SUB, r(3, 3, 1), "-", op_color, Action::OpArith(ArithOp::Sub)));

            btns.push((BTN_0, r(0, 4, 2), "0", btn_color, Action::Digit(0)));
            btns.push((BTN_EVAL, r(2, 4, 1), "=", action_color, Action::Evaluate));
            btns.push((BTN_ADD, r(3, 4, 1), "+", op_color, Action::OpArith(ArithOp::Add)));
        } else {
            // Left operand
            btns.push((BTN_N, r(0, 1, 1), "L: N", quad_color, Action::SelectQuadLeft(QuadState::N)));
            btns.push((BTN_F, r(1, 1, 1), "L: F", quad_color, Action::SelectQuadLeft(QuadState::F)));
            btns.push((BTN_T, r(2, 1, 1), "L: T", quad_color, Action::SelectQuadLeft(QuadState::T)));
            btns.push((BTN_S, r(3, 1, 1), "L: S", quad_color, Action::SelectQuadLeft(QuadState::S)));

            // Operators
            btns.push((BTN_NOT, r(0, 2, 1), "NOT", op_color, Action::SelectQuadOp(QuadOp::Not)));
            btns.push((BTN_AND, r(1, 2, 1), "AND", op_color, Action::SelectQuadOp(QuadOp::And)));
            btns.push((BTN_OR, r(2, 2, 1), "OR", op_color, Action::SelectQuadOp(QuadOp::Or)));
            btns.push((BTN_IMP, r(3, 2, 1), "IMP", op_color, Action::SelectQuadOp(QuadOp::Imp)));

            // Right operand
            btns.push((BTN_N, r(0, 3, 1), "R: N", quad_color, Action::SelectQuadRight(QuadState::N)));
            btns.push((BTN_F, r(1, 3, 1), "R: F", quad_color, Action::SelectQuadRight(QuadState::F)));
            btns.push((BTN_T, r(2, 3, 1), "R: T", quad_color, Action::SelectQuadRight(QuadState::T)));
            btns.push((BTN_S, r(3, 3, 1), "R: S", quad_color, Action::SelectQuadRight(QuadState::S)));

            // More Ops & Eval
            btns.push((BTN_EQ, r(0, 4, 1), "EQ", op_color, Action::SelectQuadOp(QuadOp::Eq)));
            btns.push((BTN_NEQ, r(1, 4, 1), "NEQ", op_color, Action::SelectQuadOp(QuadOp::Neq)));
            btns.push((BTN_EVAL, r(2, 4, 2), "=", action_color, Action::Evaluate));
        }
        btns
    }

    pub fn handle_input_event(&mut self, event: &InputEventKind) -> LoopControl {
        match event {
            InputEventKind::CloseRequested => LoopControl::ExitRequested,
            InputEventKind::KeyDown { key_code: 27 } => LoopControl::ExitRequested, // Esc
            InputEventKind::PointerMoved { x, y } => {
                self.pointer_pos = (*x, *y);
                LoopControl::Continue
            }
            InputEventKind::PointerDown { button: _ } => {
                let (px, py) = self.pointer_pos;
                let px = px as i32;
                let py = py as i32;

                for (id, rect, _, _, action) in self.get_buttons() {
                    let rx = rect.x;
                    let ry = rect.y;
                    let rw = rect.width as i32;
                    let rh = rect.height as i32;
                    if px >= rx && px <= (rx + rw) && py >= ry && py <= (ry + rh) {
                        self.focused_node = Some(id);
                        let _ = self.dispatch_action(action);
                        break;
                    }
                }
                LoopControl::Continue
            }
            _ => LoopControl::Continue,
        }
    }

    pub fn render_frame(&self) -> DrawFrame {
        let mut frame = DrawFrame::new();
        frame.clear(Color::rgb(15, 18, 28));

        // Background
        frame.fill_rect(Rect::new(0, 0, 400, 520), Color::rgb(25, 30, 45));

        // Display Screen
        let display_rect = Rect::new(20, 20, 350, 110);
        frame.fill_rect(display_rect, Color::rgb(10, 14, 22));

        let mode_str = match self.state.mode {
            CalcMode::Arithmetic => "ARITHMETIC",
            CalcMode::QuadLogic => "QUAD LOGIC",
        };
        frame.draw_text(mode_str, display_rect.x + 10, display_rect.y + 20, Color::rgb(0, 220, 255));

        let display_text = match self.state.mode {
            CalcMode::Arithmetic => format!("{}", self.state.current_input),
            CalcMode::QuadLogic => {
                let left = format!("{:?}", self.state.quad_left);
                let op = format!("{:?}", self.state.quad_operator);
                let right = format!("{:?}", self.state.quad_right);
                if self.state.quad_operator == QuadOp::Not {
                    format!("NOT {}", left)
                } else {
                    format!("{} {} {}", left, op, right)
                }
            }
        };
        frame.draw_text(&display_text, display_rect.x + 10, display_rect.y + 55, Color::WHITE);

        let err_str = self.state.error_state.as_deref().unwrap_or("OK");
        let status_str = format!("Eval: {:?} | Status: {}", self.state.evaluation_state, err_str);
        let status_color = if self.state.error_state.is_some() { Color::rgb(240, 60, 60) } else { Color::rgb(150, 150, 160) };
        frame.draw_text(&status_str, display_rect.x + 10, display_rect.y + 85, status_color);

        // Buttons
        for (_, rect, label, color, _) in self.get_buttons() {
            frame.fill_rect(rect, color);
            let text_x = rect.x + (rect.width as i32 / 2) - (label.len() as i32 * 4);
            let text_y = rect.y + 30;
            frame.draw_text(label, text_x, text_y, Color::WHITE);
        }

        if self.last_denied {
            frame.fill_rect(Rect::new(20, 470, 350, 25), Color::RED);
            frame.draw_text("ACTION DENIED BY BOUNDARY", 30, 488, Color::WHITE);
        }

        frame
    }
}

fn main() {
    if std::env::args().any(|arg| arg == "--proof-run") {
        println!("=== Quad Logic Calculator (Proof Run) ===");
        let mut app = CalculatorApp::new().unwrap();
        app.dispatch_action(Action::Digit(7)).unwrap();
        let frame = app.render_frame();
        println!("[PROOF] Frame contains {} draw commands", frame.commands().len());
        return;
    }
    println!("=== Quad Logic Calculator (Verifiable Local Application) ===");
    let mut app = CalculatorApp::new().expect("CalculatorApp initialization must succeed");

    let config = WindowConfig::new("Quad Logic Calculator", 400, 520);
    let backend = NativeBackend::new();

    let mut session = DesktopSession::create(backend, config)
        .expect("DesktopSession native creation must succeed");
    assert_eq!(session.state(), SessionState::Created);

    let start = Instant::now();
    let res = session.run(move |buf, out_frame| {
        let events = buf.drain();
        for event in &events {
            let ctrl = app.handle_input_event(&event.kind);
            if ctrl == LoopControl::ExitRequested {
                return LoopControl::ExitRequested;
            }
        }

        *out_frame = app.render_frame();

        LoopControl::Continue
    });

    if let Err(e) = res {
        eprintln!("DesktopSession execution error: {e:?}");
    }

    let _ = session.close();
    println!("Session completed cleanly.");
}




