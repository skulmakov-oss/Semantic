//! This is a lifecycle-only Shell Player seed.
//! It is not the complete Shell Player implementation.
//! It owns no Semantic truth or authority.

use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShellSessionId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellLifecycle {
    Created,
    Active,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellLifecycleCommand {
    Activate,
    Suspend,
    Resume,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellLifecycleStimulus {
    Command(ShellLifecycleCommand),
    ExplicitNoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShellLifecycleLimits {
    pub(crate) max_transition_stimulus_bytes: usize,
    pub(crate) max_diagnostics_per_transition: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActivatedShellSessionContext {
    pub(crate) session_id: ShellSessionId,
    pub(crate) limits: ShellLifecycleLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShellLocalState {
    pub(crate) session_id: ShellSessionId,
    pub(crate) lifecycle: ShellLifecycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShellTransitionInput {
    pub(crate) stimulus: ShellLifecycleStimulus,
    pub(crate) stimulus_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellTransitionDisposition {
    Applied,
    NoChange,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShellDiagnostic {
    pub(crate) stable_code: &'static str,
    pub(crate) evaluation_stage: usize,
}

pub(crate) const SPV0_SESSION_MISMATCH: &str = "SPV0_SESSION_MISMATCH";
pub(crate) const SPV0_INVALID_LIFECYCLE: &str = "SPV0_INVALID_LIFECYCLE";
pub(crate) const SPV0_SESSION_CLOSED: &str = "SPV0_SESSION_CLOSED";
pub(crate) const SPV0_RESOURCE_LIMIT_EXCEEDED: &str = "SPV0_RESOURCE_LIMIT_EXCEEDED";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShellTransitionResult {
    pub(crate) disposition: ShellTransitionDisposition,
    pub(crate) state: ShellLocalState,
    pub(crate) diagnostics: Vec<ShellDiagnostic>,
    pub(crate) stimulus_bytes: usize,
    pub(crate) logical_diagnostic_count: usize,
    pub(crate) emitted_diagnostic_count: usize,
}

fn apply_diagnostic_cap(
    cap: usize,
    mut logical_diagnostics: Vec<ShellDiagnostic>,
) -> (Vec<ShellDiagnostic>, usize, usize) {
    let logical_count = logical_diagnostics.len();
    if cap == 0 {
        return (Vec::new(), logical_count, 0);
    }
    if logical_count > cap {
        logical_diagnostics.truncate(cap);
    }
    let emitted_count = logical_diagnostics.len();
    (logical_diagnostics, logical_count, emitted_count)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidatedLifecycleAction {
    ExplicitNoOp,
    Activate,
    Suspend,
    Resume,
    Close,
}

fn validate_lifecycle_action(
    lifecycle: ShellLifecycle,
    stimulus: ShellLifecycleStimulus,
) -> Result<ValidatedLifecycleAction, ShellDiagnostic> {
    match stimulus {
        ShellLifecycleStimulus::ExplicitNoOp => Ok(ValidatedLifecycleAction::ExplicitNoOp),
        ShellLifecycleStimulus::Command(cmd) => match (lifecycle, cmd) {
            (ShellLifecycle::Created, ShellLifecycleCommand::Activate) => {
                Ok(ValidatedLifecycleAction::Activate)
            }
            (ShellLifecycle::Created, ShellLifecycleCommand::Close) => {
                Ok(ValidatedLifecycleAction::Close)
            }
            (ShellLifecycle::Active, ShellLifecycleCommand::Suspend) => {
                Ok(ValidatedLifecycleAction::Suspend)
            }
            (ShellLifecycle::Active, ShellLifecycleCommand::Close) => {
                Ok(ValidatedLifecycleAction::Close)
            }
            (ShellLifecycle::Suspended, ShellLifecycleCommand::Resume) => {
                Ok(ValidatedLifecycleAction::Resume)
            }
            (ShellLifecycle::Suspended, ShellLifecycleCommand::Close) => {
                Ok(ValidatedLifecycleAction::Close)
            }
            _ => Err(ShellDiagnostic {
                stable_code: SPV0_INVALID_LIFECYCLE,
                evaluation_stage: 2, // LifecycleEligibility
            }),
        },
    }
}

fn calculate_candidate_lifecycle(
    previous: ShellLifecycle,
    action: ValidatedLifecycleAction,
) -> ShellLifecycle {
    match action {
        ValidatedLifecycleAction::ExplicitNoOp => previous,
        ValidatedLifecycleAction::Activate => ShellLifecycle::Active,
        ValidatedLifecycleAction::Suspend => ShellLifecycle::Suspended,
        ValidatedLifecycleAction::Resume => ShellLifecycle::Active,
        ValidatedLifecycleAction::Close => ShellLifecycle::Closed,
    }
}

#[derive(Debug)]
pub(crate) struct ShellSession {
    context: ActivatedShellSessionContext,
    state: ShellLocalState,
}

impl ShellSession {
    pub(crate) fn new(context: ActivatedShellSessionContext) -> Self {
        let state = ShellLocalState {
            session_id: context.session_id,
            lifecycle: ShellLifecycle::Created,
        };
        Self { context, state }
    }

    pub(crate) fn state(&self) -> &ShellLocalState {
        &self.state
    }

    pub(crate) fn apply(&mut self, input: ShellTransitionInput) -> ShellTransitionResult {
        let result = evaluate_lifecycle_transition(&self.context, &self.state, input);

        if result.disposition == ShellTransitionDisposition::Applied {
            self.state = result.state;
        }

        result
    }
}

pub(crate) fn evaluate_lifecycle_transition(
    context: &ActivatedShellSessionContext,
    previous: &ShellLocalState,
    input: ShellTransitionInput,
) -> ShellTransitionResult {
    // 1. validate session identity
    if context.session_id != previous.session_id {
        let (diagnostics, logical_count, emitted_count) = apply_diagnostic_cap(
            context.limits.max_diagnostics_per_transition,
            alloc::vec![ShellDiagnostic {
                stable_code: SPV0_SESSION_MISMATCH,
                evaluation_stage: 1, // SessionIdentity
            }],
        );
        return ShellTransitionResult {
            disposition: ShellTransitionDisposition::Rejected,
            state: *previous,
            diagnostics,
            stimulus_bytes: input.stimulus_bytes,
            logical_diagnostic_count: logical_count,
            emitted_diagnostic_count: emitted_count,
        };
    }

    // 2. validate lifecycle eligibility
    if previous.lifecycle == ShellLifecycle::Closed {
        let (diagnostics, logical_count, emitted_count) = apply_diagnostic_cap(
            context.limits.max_diagnostics_per_transition,
            alloc::vec![ShellDiagnostic {
                stable_code: SPV0_SESSION_CLOSED,
                evaluation_stage: 2, // LifecycleEligibility
            }],
        );
        return ShellTransitionResult {
            disposition: ShellTransitionDisposition::Rejected,
            state: *previous,
            diagnostics,
            stimulus_bytes: input.stimulus_bytes,
            logical_diagnostic_count: logical_count,
            emitted_diagnostic_count: emitted_count,
        };
    }

    // 2. validate lifecycle eligibility (command combination)
    let action = match validate_lifecycle_action(previous.lifecycle, input.stimulus) {
        Ok(a) => a,
        Err(diag) => {
            let (diagnostics, logical_count, emitted_count) = apply_diagnostic_cap(
                context.limits.max_diagnostics_per_transition,
                alloc::vec![diag],
            );
            return ShellTransitionResult {
                disposition: ShellTransitionDisposition::Rejected,
                state: *previous,
                diagnostics,
                stimulus_bytes: input.stimulus_bytes,
                logical_diagnostic_count: logical_count,
                emitted_diagnostic_count: emitted_count,
            };
        }
    };

    // 3. typed lifecycle envelope is structurally valid
    // This is implicitly guaranteed by Rust's enum representation of input.stimulus and the parsed action.

    // 4. validate max_transition_stimulus_bytes
    if input.stimulus_bytes > context.limits.max_transition_stimulus_bytes {
        let (diagnostics, logical_count, emitted_count) = apply_diagnostic_cap(
            context.limits.max_diagnostics_per_transition,
            alloc::vec![ShellDiagnostic {
                stable_code: SPV0_RESOURCE_LIMIT_EXCEEDED,
                evaluation_stage: 4, // InputResourcePreflight
            }],
        );
        return ShellTransitionResult {
            disposition: ShellTransitionDisposition::Rejected,
            state: *previous,
            diagnostics,
            stimulus_bytes: input.stimulus_bytes,
            logical_diagnostic_count: logical_count,
            emitted_diagnostic_count: emitted_count,
        };
    }

    // Stages 5, 6, and 8 have no work in this lifecycle-only seed.

    // 7. calculate candidate lifecycle state without commit
    let candidate_lifecycle = calculate_candidate_lifecycle(previous.lifecycle, action);

    // 9. return complete candidate state or complete previous state
    let state = ShellLocalState {
        session_id: previous.session_id,
        lifecycle: candidate_lifecycle,
    };

    let disposition = if action == ValidatedLifecycleAction::ExplicitNoOp {
        ShellTransitionDisposition::NoChange
    } else {
        ShellTransitionDisposition::Applied
    };

    // 10. apply diagnostic emission cap
    let (diagnostics, logical_count, emitted_count) =
        apply_diagnostic_cap(context.limits.max_diagnostics_per_transition, Vec::new());

    ShellTransitionResult {
        disposition,
        state,
        diagnostics,
        stimulus_bytes: input.stimulus_bytes,
        logical_diagnostic_count: logical_count,
        emitted_diagnostic_count: emitted_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(cap: usize, max_bytes: usize) -> ActivatedShellSessionContext {
        ActivatedShellSessionContext {
            session_id: ShellSessionId(1),
            limits: ShellLifecycleLimits {
                max_transition_stimulus_bytes: max_bytes,
                max_diagnostics_per_transition: cap,
            },
        }
    }

    fn make_state(lifecycle: ShellLifecycle) -> ShellLocalState {
        ShellLocalState {
            session_id: ShellSessionId(1),
            lifecycle,
        }
    }

    fn cmd(c: ShellLifecycleCommand, bytes: usize) -> ShellTransitionInput {
        ShellTransitionInput {
            stimulus: ShellLifecycleStimulus::Command(c),
            stimulus_bytes: bytes,
        }
    }

    #[test]
    fn test_valid_transitions() {
        let ctx = make_ctx(10, 100);

        let valid_cases = alloc::vec![
            (
                ShellLifecycle::Created,
                ShellLifecycleCommand::Activate,
                ShellLifecycle::Active
            ),
            (
                ShellLifecycle::Created,
                ShellLifecycleCommand::Close,
                ShellLifecycle::Closed
            ),
            (
                ShellLifecycle::Active,
                ShellLifecycleCommand::Suspend,
                ShellLifecycle::Suspended
            ),
            (
                ShellLifecycle::Active,
                ShellLifecycleCommand::Close,
                ShellLifecycle::Closed
            ),
            (
                ShellLifecycle::Suspended,
                ShellLifecycleCommand::Resume,
                ShellLifecycle::Active
            ),
            (
                ShellLifecycle::Suspended,
                ShellLifecycleCommand::Close,
                ShellLifecycle::Closed
            ),
        ];

        for (start, command, expected) in valid_cases {
            let state = make_state(start);
            let input = cmd(command, 10);
            let res = evaluate_lifecycle_transition(&ctx, &state, input);

            assert_eq!(res.disposition, ShellTransitionDisposition::Applied);
            assert_eq!(res.state.lifecycle, expected);
            assert_eq!(res.diagnostics.len(), 0);
            assert_eq!(res.state.session_id, state.session_id);
        }
    }

    #[test]
    fn test_invalid_lifecycle_command() {
        let ctx = make_ctx(10, 100);

        let invalid_cases = alloc::vec![
            (ShellLifecycle::Created, ShellLifecycleCommand::Suspend),
            (ShellLifecycle::Created, ShellLifecycleCommand::Resume),
            (ShellLifecycle::Active, ShellLifecycleCommand::Activate),
            (ShellLifecycle::Active, ShellLifecycleCommand::Resume),
            (ShellLifecycle::Suspended, ShellLifecycleCommand::Activate),
            (ShellLifecycle::Suspended, ShellLifecycleCommand::Suspend),
        ];

        for (start, command) in invalid_cases {
            let state = make_state(start);
            let input = cmd(command, 10);
            let res = evaluate_lifecycle_transition(&ctx, &state, input);

            assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
            assert_eq!(res.state, state);
            assert_eq!(res.logical_diagnostic_count, 1);
            assert_eq!(res.diagnostics[0].stable_code, SPV0_INVALID_LIFECYCLE);
            assert_eq!(res.diagnostics[0].evaluation_stage, 2);
        }
    }

    #[test]
    fn test_closed_state_rejection() {
        let ctx = make_ctx(10, 100);
        let state = make_state(ShellLifecycle::Closed);

        let commands = alloc::vec![
            ShellLifecycleStimulus::ExplicitNoOp,
            ShellLifecycleStimulus::Command(ShellLifecycleCommand::Activate),
            ShellLifecycleStimulus::Command(ShellLifecycleCommand::Suspend),
            ShellLifecycleStimulus::Command(ShellLifecycleCommand::Resume),
            ShellLifecycleStimulus::Command(ShellLifecycleCommand::Close),
        ];

        for stimulus in commands {
            let input = ShellTransitionInput {
                stimulus,
                stimulus_bytes: 10,
            };
            let res = evaluate_lifecycle_transition(&ctx, &state, input);

            assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
            assert_eq!(res.state, state);
            assert_eq!(res.logical_diagnostic_count, 1);
            assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_CLOSED);
        }
    }

    #[test]
    fn test_explicit_noop() {
        let ctx = make_ctx(10, 100);
        let states = alloc::vec![
            ShellLifecycle::Created,
            ShellLifecycle::Active,
            ShellLifecycle::Suspended,
        ];

        for lifecycle in states {
            let state = make_state(lifecycle);
            let input = ShellTransitionInput {
                stimulus: ShellLifecycleStimulus::ExplicitNoOp,
                stimulus_bytes: 10,
            };
            let res = evaluate_lifecycle_transition(&ctx, &state, input);

            assert_eq!(res.disposition, ShellTransitionDisposition::NoChange);
            assert_eq!(res.state, state);
            assert_eq!(res.logical_diagnostic_count, 0);
        }
    }

    #[test]
    fn test_session_mismatch() {
        let ctx = make_ctx(10, 100);
        let mut state = make_state(ShellLifecycle::Created);
        state.session_id = ShellSessionId(999);

        let input = cmd(ShellLifecycleCommand::Activate, 10);
        let res = evaluate_lifecycle_transition(&ctx, &state, input);

        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.state, state);
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_MISMATCH);
        assert_eq!(res.diagnostics[0].evaluation_stage, 1);
    }

    #[test]
    fn test_oversized_stimulus() {
        let ctx = make_ctx(10, 100);
        let state = make_state(ShellLifecycle::Created);

        let input = cmd(ShellLifecycleCommand::Activate, 101);
        let res = evaluate_lifecycle_transition(&ctx, &state, input);

        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.state, state);
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(res.diagnostics[0].evaluation_stage, 4);
    }

    #[test]
    fn test_oversized_stimulus_precedence_over_candidate_calculation() {
        let ctx = make_ctx(10, 100);
        let state = make_state(ShellLifecycle::Created);

        // Valid command, but oversized stimulus
        let input = cmd(ShellLifecycleCommand::Activate, 101);
        let res = evaluate_lifecycle_transition(&ctx, &state, input);

        // Should be rejected due to limits (Stage 4), NOT invalid lifecycle (Stage 2)
        // And the candidate calculation (Stage 7) should not have been applied
        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.state, state); // previous state preserved
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(res.diagnostics[0].evaluation_stage, 4);
    }

    #[test]
    fn test_diagnostic_cap_zero() {
        let ctx = make_ctx(0, 100);
        let state = make_state(ShellLifecycle::Closed);

        let input = cmd(ShellLifecycleCommand::Activate, 10);
        let res = evaluate_lifecycle_transition(&ctx, &state, input);

        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.state, state);
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.emitted_diagnostic_count, 0);
        assert_eq!(res.diagnostics.len(), 0);
    }

    #[test]
    fn test_diagnostic_prefix() {
        let diags = alloc::vec![
            ShellDiagnostic {
                stable_code: "A",
                evaluation_stage: 1
            },
            ShellDiagnostic {
                stable_code: "B",
                evaluation_stage: 2
            },
            ShellDiagnostic {
                stable_code: "C",
                evaluation_stage: 3
            },
        ];

        let (res_diags, logical, emitted) = apply_diagnostic_cap(2, diags.clone());
        assert_eq!(logical, 3);
        assert_eq!(emitted, 2);
        assert_eq!(res_diags.len(), 2);
        assert_eq!(res_diags[0].stable_code, "A");
        assert_eq!(res_diags[1].stable_code, "B");
    }

    #[test]
    fn test_identical_input_produces_identical_results() {
        let ctx = make_ctx(10, 100);
        let state = make_state(ShellLifecycle::Created);
        let input = cmd(ShellLifecycleCommand::Activate, 10);

        let res1 = evaluate_lifecycle_transition(&ctx, &state, input);
        let res2 = evaluate_lifecycle_transition(&ctx, &state, input);

        assert_eq!(res1, res2);
    }

    #[test]
    fn test_valid_transition_does_not_mutate_previous_state() {
        let ctx = make_ctx(10, 100);
        let previous = make_state(ShellLifecycle::Created);
        let input = cmd(ShellLifecycleCommand::Activate, 10);

        let res = evaluate_lifecycle_transition(&ctx, &previous, input);

        assert_eq!(previous.lifecycle, ShellLifecycle::Created);
        assert_eq!(res.state.lifecycle, ShellLifecycle::Active);
    }

    #[test]
    fn test_owner_constructor() {
        let ctx = make_ctx(10, 100);
        let session = ShellSession::new(ctx);
        assert_eq!(session.state().lifecycle, ShellLifecycle::Created);
        assert_eq!(session.state().session_id, ctx.session_id);
    }

    #[test]
    fn test_owner_valid_sequence() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);

        let sequence = alloc::vec![
            (ShellLifecycleCommand::Activate, ShellLifecycle::Active),
            (ShellLifecycleCommand::Suspend, ShellLifecycle::Suspended),
            (ShellLifecycleCommand::Resume, ShellLifecycle::Active),
            (ShellLifecycleCommand::Close, ShellLifecycle::Closed),
        ];

        for (command, expected_state) in sequence {
            let res = session.apply(cmd(command, 10));
            assert_eq!(res.disposition, ShellTransitionDisposition::Applied);
            assert_eq!(session.state().lifecycle, expected_state);
            assert_eq!(res.state, *session.state());
        }
    }

    #[test]
    fn test_owner_invalid_command_preservation() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        let original_state = *session.state();

        let res = session.apply(cmd(ShellLifecycleCommand::Suspend, 10));
        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(*session.state(), original_state);
        assert_eq!(res.state, *session.state());
    }

    #[test]
    fn test_owner_resource_rejection_preservation() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        let original_state = *session.state();

        let res = session.apply(cmd(ShellLifecycleCommand::Activate, 101));
        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(*session.state(), original_state);
    }

    #[test]
    fn test_owner_no_change_preservation() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);

        // Test in Created
        let original_state = *session.state();
        let res = session.apply(ShellTransitionInput {
            stimulus: ShellLifecycleStimulus::ExplicitNoOp,
            stimulus_bytes: 10,
        });
        assert_eq!(res.disposition, ShellTransitionDisposition::NoChange);
        assert_eq!(*session.state(), original_state);
        assert_eq!(res.state, *session.state());

        // Test in Active
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        let active_state = *session.state();
        let res2 = session.apply(ShellTransitionInput {
            stimulus: ShellLifecycleStimulus::ExplicitNoOp,
            stimulus_bytes: 10,
        });
        assert_eq!(res2.disposition, ShellTransitionDisposition::NoChange);
        assert_eq!(*session.state(), active_state);

        // Test in Suspended
        session.apply(cmd(ShellLifecycleCommand::Suspend, 10));
        let suspended_state = *session.state();
        let res3 = session.apply(ShellTransitionInput {
            stimulus: ShellLifecycleStimulus::ExplicitNoOp,
            stimulus_bytes: 10,
        });
        assert_eq!(res3.disposition, ShellTransitionDisposition::NoChange);
        assert_eq!(*session.state(), suspended_state);
    }

    #[test]
    fn test_owner_closed_is_terminal() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Close, 10)); // Created -> Closed

        let closed_state = *session.state();
        assert_eq!(closed_state.lifecycle, ShellLifecycle::Closed);

        let inputs = alloc::vec![
            ShellTransitionInput {
                stimulus: ShellLifecycleStimulus::ExplicitNoOp,
                stimulus_bytes: 10
            },
            cmd(ShellLifecycleCommand::Activate, 10),
            cmd(ShellLifecycleCommand::Suspend, 10),
            cmd(ShellLifecycleCommand::Resume, 10),
            cmd(ShellLifecycleCommand::Close, 10),
        ];

        for input in inputs {
            let res = session.apply(input);
            assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
            assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_CLOSED);
            assert_eq!(*session.state(), closed_state);
        }
    }

    #[test]
    fn test_owner_immutable_context() {
        let original_ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(original_ctx);

        session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        session.apply(cmd(ShellLifecycleCommand::Suspend, 10));
        session.apply(cmd(ShellLifecycleCommand::Resume, 10));

        assert_eq!(session.context, original_ctx);
    }

    #[test]
    fn test_owner_deterministic_sequence() {
        let ctx = make_ctx(10, 100);
        let mut session1 = ShellSession::new(ctx);
        let mut session2 = ShellSession::new(ctx);

        let sequence = alloc::vec![
            cmd(ShellLifecycleCommand::Activate, 10),
            ShellTransitionInput {
                stimulus: ShellLifecycleStimulus::ExplicitNoOp,
                stimulus_bytes: 10
            },
            cmd(ShellLifecycleCommand::Suspend, 10),
            cmd(ShellLifecycleCommand::Resume, 101), // Rejected
            cmd(ShellLifecycleCommand::Resume, 10),
            cmd(ShellLifecycleCommand::Close, 10),
        ];

        for input in sequence {
            let res1 = session1.apply(input);
            let res2 = session2.apply(input);
            assert_eq!(res1, res2);
        }

        assert_eq!(session1.state(), session2.state());
        assert_eq!(session1.context, session2.context);
    }

    #[test]
    fn test_owner_rejection_followed_by_recovery() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);

        let invalid_res = session.apply(cmd(ShellLifecycleCommand::Suspend, 10));
        assert_eq!(
            invalid_res.disposition,
            ShellTransitionDisposition::Rejected
        );

        let valid_res = session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        assert_eq!(valid_res.disposition, ShellTransitionDisposition::Applied);
        assert_eq!(session.state().lifecycle, ShellLifecycle::Active);
    }

    #[test]
    fn test_owner_returned_state_consistency() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);

        // NoChange
        let res1 = session.apply(ShellTransitionInput {
            stimulus: ShellLifecycleStimulus::ExplicitNoOp,
            stimulus_bytes: 10,
        });
        assert_eq!(res1.disposition, ShellTransitionDisposition::NoChange);
        assert_eq!(res1.state, *session.state());

        // Rejected
        let res2 = session.apply(cmd(ShellLifecycleCommand::Suspend, 10));
        assert_eq!(res2.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res2.state, *session.state());

        // Applied
        let res3 = session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        assert_eq!(res3.disposition, ShellTransitionDisposition::Applied);
        assert_eq!(res3.state, *session.state());
    }
}
