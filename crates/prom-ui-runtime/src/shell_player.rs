//! Shell Player: local projection playback for one activated session.
//! It owns no Semantic truth or authority (see module invariants in
//! `docs/spec/ui/shell_player_session_state_v0.md`).

use alloc::vec::Vec;

use crate::active_projection_target_catalog::ActiveProjectionTargetCatalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellSessionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellLifecycle {
    Created,
    Active,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellLifecycleCommand {
    Activate,
    Suspend,
    Resume,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellLifecycleStimulus {
    Command(ShellLifecycleCommand),
    ExplicitNoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellLifecycleLimits {
    pub max_transition_stimulus_bytes: usize,
    pub max_diagnostics_per_transition: usize,
    pub max_patches_per_transition: usize,
    pub max_target_references_per_transition: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActivatedShellSessionContext {
    pub(crate) session_id: ShellSessionId,
    pub(crate) limits: ShellLifecycleLimits,
    pub(crate) catalog: ActiveProjectionTargetCatalog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionReplayCursor {
    Uninitialized,
    At(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellLocalState {
    pub session_id: ShellSessionId,
    pub lifecycle: ShellLifecycle,
    pub replay_cursor: ProjectionReplayCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellTransitionInput {
    pub stimulus: ShellLifecycleStimulus,
    pub stimulus_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellTransitionDisposition {
    Applied,
    NoChange,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellDiagnostic {
    pub stable_code: &'static str,
    pub evaluation_stage: usize,
}

pub(crate) const SPV0_SESSION_MISMATCH: &str = "SPV0_SESSION_MISMATCH";
pub(crate) const SPV0_INVALID_LIFECYCLE: &str = "SPV0_INVALID_LIFECYCLE";
pub(crate) const SPV0_SESSION_CLOSED: &str = "SPV0_SESSION_CLOSED";
pub(crate) const SPV0_RESOURCE_LIMIT_EXCEEDED: &str = "SPV0_RESOURCE_LIMIT_EXCEEDED";
pub(crate) const SPV0_REPLAY_CURSOR_MISMATCH: &str = "SPV0_REPLAY_CURSOR_MISMATCH";
pub(crate) const SPV0_INVALID_STIMULUS: &str = "SPV0_INVALID_STIMULUS";
pub(crate) const SPV0_INVALID_TARGET: &str = "SPV0_INVALID_TARGET";
pub(crate) const SPV0_STATE_INVARIANT_VIOLATION: &str = "SPV0_STATE_INVARIANT_VIOLATION";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellTransitionResult {
    pub disposition: ShellTransitionDisposition,
    pub state: ShellLocalState,
    pub diagnostics: Vec<ShellDiagnostic>,
    pub stimulus_bytes: usize,
    pub logical_diagnostic_count: usize,
    pub emitted_diagnostic_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderedProjectionPatchBatchEnvelope {
    pub session_id: ShellSessionId,
    pub sequence_no: u64,
    pub patch_count: usize,
    pub stimulus_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectionPatchPreflightDisposition {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectionPatchPreflightResult {
    pub(crate) disposition: ProjectionPatchPreflightDisposition,
    pub(crate) sequence_no: u64,
    pub(crate) patch_count: usize,
    pub(crate) stimulus_bytes: usize,
    pub(crate) diagnostics: Vec<ShellDiagnostic>,
    pub(crate) logical_diagnostic_count: usize,
    pub(crate) emitted_diagnostic_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectionReplayCompatibilityDisposition {
    NotApplicable,
    Compatible,
    Mismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectionReplayCompatibilityResult {
    pub(crate) disposition: ProjectionReplayCompatibilityDisposition,
    pub(crate) sequence_no: u64,
    pub(crate) patch_count: usize,
    pub(crate) diagnostics: Vec<ShellDiagnostic>,
    pub(crate) logical_diagnostic_count: usize,
    pub(crate) emitted_diagnostic_count: usize,
}

// Stage-6-only pure evaluator.
// The caller must invoke this only after stages 1 through 5 have succeeded.
// This function performs no stage-1-through-5 validation.
pub(crate) fn evaluate_projection_patch_replay_compatibility(
    replay_cursor: ProjectionReplayCursor,
    envelope: OrderedProjectionPatchBatchEnvelope,
    max_diagnostics_per_transition: usize,
) -> ProjectionReplayCompatibilityResult {
    if envelope.patch_count == 0 {
        return ProjectionReplayCompatibilityResult {
            disposition: ProjectionReplayCompatibilityDisposition::NotApplicable,
            sequence_no: envelope.sequence_no,
            patch_count: 0,
            diagnostics: Vec::new(),
            logical_diagnostic_count: 0,
            emitted_diagnostic_count: 0,
        };
    }

    match replay_cursor {
        ProjectionReplayCursor::Uninitialized => ProjectionReplayCompatibilityResult {
            disposition: ProjectionReplayCompatibilityDisposition::Compatible,
            sequence_no: envelope.sequence_no,
            patch_count: envelope.patch_count,
            diagnostics: Vec::new(),
            logical_diagnostic_count: 0,
            emitted_diagnostic_count: 0,
        },
        ProjectionReplayCursor::At(established) => {
            let compatible = established.checked_add(1) == Some(envelope.sequence_no);
            if compatible {
                ProjectionReplayCompatibilityResult {
                    disposition: ProjectionReplayCompatibilityDisposition::Compatible,
                    sequence_no: envelope.sequence_no,
                    patch_count: envelope.patch_count,
                    diagnostics: Vec::new(),
                    logical_diagnostic_count: 0,
                    emitted_diagnostic_count: 0,
                }
            } else {
                let diagnostic = ShellDiagnostic {
                    stable_code: SPV0_REPLAY_CURSOR_MISMATCH,
                    evaluation_stage: 6,
                };
                let (diagnostics, logical_count, emitted_count) =
                    apply_diagnostic_cap(max_diagnostics_per_transition, alloc::vec![diagnostic]);
                ProjectionReplayCompatibilityResult {
                    disposition: ProjectionReplayCompatibilityDisposition::Mismatch,
                    sequence_no: envelope.sequence_no,
                    patch_count: envelope.patch_count,
                    diagnostics,
                    logical_diagnostic_count: logical_count,
                    emitted_diagnostic_count: emitted_count,
                }
            }
        }
    }
}

pub(crate) fn evaluate_projection_patch_envelope_preflight(
    context: &ActivatedShellSessionContext,
    state: &ShellLocalState,
    envelope: OrderedProjectionPatchBatchEnvelope,
) -> ProjectionPatchPreflightResult {
    // Stage 1 - SessionIdentity
    if context.session_id != state.session_id || envelope.session_id != context.session_id {
        let (diagnostics, logical_count, emitted_count) = apply_diagnostic_cap(
            context.limits.max_diagnostics_per_transition,
            alloc::vec![ShellDiagnostic {
                stable_code: SPV0_SESSION_MISMATCH,
                evaluation_stage: 1, // SessionIdentity
            }],
        );
        return ProjectionPatchPreflightResult {
            disposition: ProjectionPatchPreflightDisposition::Rejected,
            sequence_no: envelope.sequence_no,
            patch_count: envelope.patch_count,
            stimulus_bytes: envelope.stimulus_bytes,
            diagnostics,
            logical_diagnostic_count: logical_count,
            emitted_diagnostic_count: emitted_count,
        };
    }

    // Stage 2 - LifecycleEligibility
    if state.lifecycle == ShellLifecycle::Closed {
        let (diagnostics, logical_count, emitted_count) = apply_diagnostic_cap(
            context.limits.max_diagnostics_per_transition,
            alloc::vec![ShellDiagnostic {
                stable_code: SPV0_SESSION_CLOSED,
                evaluation_stage: 2, // LifecycleEligibility
            }],
        );
        return ProjectionPatchPreflightResult {
            disposition: ProjectionPatchPreflightDisposition::Rejected,
            sequence_no: envelope.sequence_no,
            patch_count: envelope.patch_count,
            stimulus_bytes: envelope.stimulus_bytes,
            diagnostics,
            logical_diagnostic_count: logical_count,
            emitted_diagnostic_count: emitted_count,
        };
    }

    // Stage 3 - TypedOuterEnvelope implicitly checked

    // Stage 4 - InputResourcePreflight
    let patch_count_exceeded = envelope.patch_count > context.limits.max_patches_per_transition;

    let stimulus_bytes_exceeded =
        envelope.stimulus_bytes > context.limits.max_transition_stimulus_bytes;

    if patch_count_exceeded || stimulus_bytes_exceeded {
        let (diagnostics, logical_count, emitted_count) = apply_diagnostic_cap(
            context.limits.max_diagnostics_per_transition,
            alloc::vec![ShellDiagnostic {
                stable_code: SPV0_RESOURCE_LIMIT_EXCEEDED,
                evaluation_stage: 4, // InputResourcePreflight
            }],
        );
        return ProjectionPatchPreflightResult {
            disposition: ProjectionPatchPreflightDisposition::Rejected,
            sequence_no: envelope.sequence_no,
            patch_count: envelope.patch_count,
            stimulus_bytes: envelope.stimulus_bytes,
            diagnostics,
            logical_diagnostic_count: logical_count,
            emitted_diagnostic_count: emitted_count,
        };
    }

    let (diagnostics, logical_count, emitted_count) =
        apply_diagnostic_cap(context.limits.max_diagnostics_per_transition, Vec::new());

    ProjectionPatchPreflightResult {
        disposition: ProjectionPatchPreflightDisposition::Accepted,
        sequence_no: envelope.sequence_no,
        patch_count: envelope.patch_count,
        stimulus_bytes: envelope.stimulus_bytes,
        diagnostics,
        logical_diagnostic_count: logical_count,
        emitted_diagnostic_count: emitted_count,
    }
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

/// Caller-supplied deterministic resource limits for one activated Shell
/// Player session. Public mirror of [`ShellLifecycleLimits`] used only at
/// [`create_shell_session`] construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellSessionLimits {
    pub max_transition_stimulus_bytes: usize,
    pub max_diagnostics_per_transition: usize,
    pub max_patches_per_transition: usize,
    pub max_target_references_per_transition: usize,
}

/// The disposition of one `ProjectionPatch` batch transition through the
/// complete stages 1-9 pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionPatchApplicationDisposition {
    /// The batch was accepted and its effects were committed.
    Applied,
    /// The batch was valid but produced no observable change (an empty
    /// batch never establishes or consumes a replay coordinate).
    NoChange,
    /// The batch was rejected. The complete previous local projection
    /// state and replay cursor are preserved unchanged.
    Rejected,
}

/// The result of one [`ShellSession::apply_projection_patch_batch`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionPatchApplicationResult {
    pub disposition: ProjectionPatchApplicationDisposition,
    pub diagnostics: Vec<ShellDiagnostic>,
    pub logical_diagnostic_count: usize,
    pub emitted_diagnostic_count: usize,
    pub replay_cursor: ProjectionReplayCursor,
}

#[derive(Debug)]
pub struct ShellSession {
    context: ActivatedShellSessionContext,
    state: ShellLocalState,
    local_projection: prom_ui::shell_bridge::LocalProjectionStateHandle,
}

/// Creates and activates one Shell Player session from caller-supplied
/// limits and one activation-target snapshot (contract section 7.1.9.6).
/// The returned session's [`ActiveProjectionTargetCatalog`] is constructed
/// exactly once, here, from `activation_snapshot`, and is immutable for the
/// complete session lifetime.
pub fn create_shell_session(
    session_id: u64,
    limits: ShellSessionLimits,
    activation_snapshot: &prom_ui::shell_bridge::ActivationTargetSnapshot,
) -> ShellSession {
    let context = ActivatedShellSessionContext {
        session_id: ShellSessionId(session_id),
        limits: ShellLifecycleLimits {
            max_transition_stimulus_bytes: limits.max_transition_stimulus_bytes,
            max_diagnostics_per_transition: limits.max_diagnostics_per_transition,
            max_patches_per_transition: limits.max_patches_per_transition,
            max_target_references_per_transition: limits.max_target_references_per_transition,
        },
        catalog: ActiveProjectionTargetCatalog::from_snapshot(activation_snapshot),
    };
    ShellSession::new(context)
}

impl ShellSession {
    pub(crate) fn new(context: ActivatedShellSessionContext) -> Self {
        let state = ShellLocalState {
            session_id: context.session_id,
            lifecycle: ShellLifecycle::Created,
            replay_cursor: ProjectionReplayCursor::Uninitialized,
        };
        Self {
            context,
            state,
            local_projection: prom_ui::shell_bridge::initial_local_projection_state(),
        }
    }

    pub fn state(&self) -> &ShellLocalState {
        &self.state
    }

    pub fn lifecycle(&self) -> ShellLifecycle {
        self.state.lifecycle
    }

    pub fn replay_cursor(&self) -> ProjectionReplayCursor {
        self.state.replay_cursor
    }

    /// Reads the current committed binding value at `(node, slot)`, or
    /// `None` if unset.
    pub fn binding_value(
        &self,
        node: u64,
        slot: u32,
    ) -> Option<prom_ui::shell_bridge::BridgeValue> {
        prom_ui::shell_bridge::binding_value(&self.local_projection, node, slot)
    }

    /// Reads the current committed availability of `node`, or `None` if
    /// unset.
    pub fn node_availability(
        &self,
        node: u64,
    ) -> Option<prom_ui::shell_bridge::BridgeAvailability> {
        prom_ui::shell_bridge::node_availability(&self.local_projection, node)
    }

    /// Reads the current committed ordered entries of `collection`.
    pub fn collection_entries(
        &self,
        collection: u64,
    ) -> Vec<(u64, prom_ui::shell_bridge::BridgeValue)> {
        prom_ui::shell_bridge::collection_entries(&self.local_projection, collection)
    }

    pub fn apply(&mut self, input: ShellTransitionInput) -> ShellTransitionResult {
        let result = evaluate_lifecycle_transition(&self.context, &self.state, input);

        if result.disposition == ShellTransitionDisposition::Applied {
            self.state = result.state;
        }

        result
    }

    pub(crate) fn preflight_projection_patch_envelope(
        &self,
        envelope: OrderedProjectionPatchBatchEnvelope,
    ) -> ProjectionPatchPreflightResult {
        evaluate_projection_patch_envelope_preflight(&self.context, &self.state, envelope)
    }

    /// Runs the complete stages 1-9 `ProjectionPatch` batch transition:
    /// stages 1-4 (session/lifecycle/envelope/resource preflight, reusing
    /// [`Self::preflight_projection_patch_envelope`]), stage 4b (declared
    /// vs actual target-reference count coherence, contract section
    /// 7.1.9.8), stage 4c (target-reference resource limit), stage 5
    /// (stable-target membership against the immutable session catalog,
    /// contract section 7.1.5), stage 6 (replay-cursor compatibility,
    /// reusing [`evaluate_projection_patch_replay_compatibility`]), stage 7
    /// (candidate local-projection-state calculation via
    /// `prom_ui::shell_bridge::apply_admitted_patch_batch`), and stage 9
    /// (atomic commit).
    ///
    /// On any rejection, the complete previous local projection state and
    /// replay cursor are preserved: this method does not mutate `self`
    /// until every stage has succeeded.
    pub fn apply_projection_patch_batch(
        &mut self,
        envelope: OrderedProjectionPatchBatchEnvelope,
        target_reference_count: usize,
        manifest: &prom_ui::shell_bridge::PreparedManifestSnapshot,
        batch: &prom_ui::shell_bridge::AdmittedProjectionPatchBatch,
    ) -> ProjectionPatchApplicationResult {
        let cap = self.context.limits.max_diagnostics_per_transition;

        // Stages 1-4: session identity, lifecycle eligibility, envelope
        // shape, and declared resource bounds (existing pure evaluator).
        let preflight = self.preflight_projection_patch_envelope(envelope);
        if preflight.disposition == ProjectionPatchPreflightDisposition::Rejected {
            return ProjectionPatchApplicationResult {
                disposition: ProjectionPatchApplicationDisposition::Rejected,
                diagnostics: preflight.diagnostics,
                logical_diagnostic_count: preflight.logical_diagnostic_count,
                emitted_diagnostic_count: preflight.emitted_diagnostic_count,
                replay_cursor: self.state.replay_cursor,
            };
        }

        // Stage 4b: declared/actual target-reference count coherence.
        if target_reference_count != manifest.target_reference_count {
            return self.reject_patch_batch(SPV0_INVALID_STIMULUS, 4, cap);
        }

        // Stage 4c: target-reference resource limit.
        if target_reference_count > self.context.limits.max_target_references_per_transition {
            return self.reject_patch_batch(SPV0_RESOURCE_LIMIT_EXCEEDED, 4, cap);
        }

        // Stage 5: stable-target membership, using only the immutable
        // session catalog. Read-only; does not mutate local state.
        let mut invalid_targets = Vec::new();
        for entry in &manifest.entries {
            let valid = match entry.role {
                prom_ui::shell_bridge::BridgeTargetRole::Node { node } => {
                    self.context.catalog.contains_node(node)
                }
                prom_ui::shell_bridge::BridgeTargetRole::Binding { node, slot } => {
                    self.context.catalog.contains_binding(node, slot)
                }
                prom_ui::shell_bridge::BridgeTargetRole::Collection { collection } => {
                    self.context.catalog.contains_collection(collection)
                }
            };
            if !valid {
                invalid_targets.push(ShellDiagnostic {
                    stable_code: SPV0_INVALID_TARGET,
                    evaluation_stage: 5,
                });
            }
        }
        if !invalid_targets.is_empty() {
            let (diagnostics, logical_diagnostic_count, emitted_diagnostic_count) =
                apply_diagnostic_cap(cap, invalid_targets);
            return ProjectionPatchApplicationResult {
                disposition: ProjectionPatchApplicationDisposition::Rejected,
                diagnostics,
                logical_diagnostic_count,
                emitted_diagnostic_count,
                replay_cursor: self.state.replay_cursor,
            };
        }

        // Stage 6: replay-cursor compatibility (existing pure evaluator).
        let compatibility =
            evaluate_projection_patch_replay_compatibility(self.state.replay_cursor, envelope, cap);
        if compatibility.disposition == ProjectionReplayCompatibilityDisposition::Mismatch {
            return ProjectionPatchApplicationResult {
                disposition: ProjectionPatchApplicationDisposition::Rejected,
                diagnostics: compatibility.diagnostics,
                logical_diagnostic_count: compatibility.logical_diagnostic_count,
                emitted_diagnostic_count: compatibility.emitted_diagnostic_count,
                replay_cursor: self.state.replay_cursor,
            };
        }

        // Stage 7: deterministic candidate-state calculation, without
        // committing. Structural application failures (e.g. a collection
        // operation targeting an already-present or missing key) are
        // candidate-state invariant violations.
        let candidate = match prom_ui::shell_bridge::apply_admitted_patch_batch(
            &self.local_projection,
            batch,
        ) {
            Ok(candidate) => candidate,
            Err(_) => return self.reject_patch_batch(SPV0_STATE_INVARIANT_VIOLATION, 8, cap),
        };

        // Stage 9: atomic commit. A zero-patch batch does not establish or
        // consume a replay coordinate (contract section 7.2).
        let disposition = if envelope.patch_count == 0 {
            ProjectionPatchApplicationDisposition::NoChange
        } else {
            self.state.replay_cursor = ProjectionReplayCursor::At(envelope.sequence_no);
            ProjectionPatchApplicationDisposition::Applied
        };
        self.local_projection = candidate;

        ProjectionPatchApplicationResult {
            disposition,
            diagnostics: Vec::new(),
            logical_diagnostic_count: 0,
            emitted_diagnostic_count: 0,
            replay_cursor: self.state.replay_cursor,
        }
    }

    fn reject_patch_batch(
        &self,
        stable_code: &'static str,
        evaluation_stage: usize,
        cap: usize,
    ) -> ProjectionPatchApplicationResult {
        let (diagnostics, logical_diagnostic_count, emitted_diagnostic_count) =
            apply_diagnostic_cap(
                cap,
                alloc::vec![ShellDiagnostic {
                    stable_code,
                    evaluation_stage,
                }],
            );
        ProjectionPatchApplicationResult {
            disposition: ProjectionPatchApplicationDisposition::Rejected,
            diagnostics,
            logical_diagnostic_count,
            emitted_diagnostic_count,
            replay_cursor: self.state.replay_cursor,
        }
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
        replay_cursor: previous.replay_cursor,
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

    fn make_state_with_cursor(
        lifecycle: ShellLifecycle,
        replay_cursor: ProjectionReplayCursor,
    ) -> ShellLocalState {
        ShellLocalState {
            session_id: ShellSessionId(1),
            lifecycle,
            replay_cursor,
        }
    }

    #[test]
    fn test_cursor_1_new_session_uninitialized() {
        let ctx = make_ctx(10, 100);
        let session = ShellSession::new(ctx.clone());
        assert_eq!(session.state().session_id, ctx.session_id);
        assert_eq!(session.state().lifecycle, ShellLifecycle::Created);
        assert_eq!(
            session.state().replay_cursor,
            ProjectionReplayCursor::Uninitialized
        );
    }

    #[test]
    fn test_cursor_2_activate_preserves_uninitialized() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        let res = session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        assert_eq!(res.disposition, ShellTransitionDisposition::Applied);
        assert_eq!(
            session.state().replay_cursor,
            ProjectionReplayCursor::Uninitialized
        );
        assert_eq!(
            res.state.replay_cursor,
            ProjectionReplayCursor::Uninitialized
        );
    }

    #[test]
    fn test_cursor_3_full_lifecycle_preserves_positioned_cursor() {
        let ctx = make_ctx(10, 100);
        let mut state =
            make_state_with_cursor(ShellLifecycle::Created, ProjectionReplayCursor::At(41));

        let transitions = alloc::vec![
            (ShellLifecycleCommand::Activate, ShellLifecycle::Active),
            (ShellLifecycleCommand::Suspend, ShellLifecycle::Suspended),
            (ShellLifecycleCommand::Resume, ShellLifecycle::Active),
            (ShellLifecycleCommand::Close, ShellLifecycle::Closed),
        ];

        for (command, expected_lifecycle) in transitions {
            let res = evaluate_lifecycle_transition(&ctx, &state, cmd(command, 10));
            assert_eq!(res.disposition, ShellTransitionDisposition::Applied);
            assert_eq!(res.state.lifecycle, expected_lifecycle);
            assert_eq!(res.state.replay_cursor, ProjectionReplayCursor::At(41));
            state = res.state;
        }
    }

    #[test]
    fn test_cursor_4_direct_close_preserves_positioned_cursor() {
        let ctx = make_ctx(10, 100);

        let state1 = make_state_with_cursor(ShellLifecycle::Created, ProjectionReplayCursor::At(0));
        let res1 =
            evaluate_lifecycle_transition(&ctx, &state1, cmd(ShellLifecycleCommand::Close, 10));
        assert_eq!(res1.disposition, ShellTransitionDisposition::Applied);
        assert_eq!(res1.state.lifecycle, ShellLifecycle::Closed);
        assert_eq!(res1.state.replay_cursor, ProjectionReplayCursor::At(0));

        let state2 =
            make_state_with_cursor(ShellLifecycle::Suspended, ProjectionReplayCursor::At(0));
        let res2 =
            evaluate_lifecycle_transition(&ctx, &state2, cmd(ShellLifecycleCommand::Close, 10));
        assert_eq!(res2.disposition, ShellTransitionDisposition::Applied);
        assert_eq!(res2.state.lifecycle, ShellLifecycle::Closed);
        assert_eq!(res2.state.replay_cursor, ProjectionReplayCursor::At(0));
    }

    #[test]
    fn test_cursor_5_explicit_noop_preserves_cursor() {
        let ctx = make_ctx(10, 100);
        let state =
            make_state_with_cursor(ShellLifecycle::Active, ProjectionReplayCursor::At(u64::MAX));
        let res = evaluate_lifecycle_transition(
            &ctx,
            &state,
            ShellTransitionInput {
                stimulus: ShellLifecycleStimulus::ExplicitNoOp,
                stimulus_bytes: 10,
            },
        );
        assert_eq!(res.disposition, ShellTransitionDisposition::NoChange);
        assert_eq!(
            res.state.replay_cursor,
            ProjectionReplayCursor::At(u64::MAX)
        );
        assert_eq!(res.state, state);
    }

    #[test]
    fn test_cursor_6_invalid_lifecycle_preserves_cursor() {
        let ctx = make_ctx(10, 100);
        let state = make_state_with_cursor(ShellLifecycle::Created, ProjectionReplayCursor::At(42));
        let res =
            evaluate_lifecycle_transition(&ctx, &state, cmd(ShellLifecycleCommand::Suspend, 10));
        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_INVALID_LIFECYCLE);
        assert_eq!(res.state, state);
        assert_eq!(res.state.replay_cursor, ProjectionReplayCursor::At(42));
    }

    #[test]
    fn test_cursor_7_closed_session_rejection_preserves_cursor() {
        let ctx = make_ctx(10, 100);
        let state = make_state_with_cursor(ShellLifecycle::Closed, ProjectionReplayCursor::At(17));
        let res =
            evaluate_lifecycle_transition(&ctx, &state, cmd(ShellLifecycleCommand::Activate, 10));
        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_CLOSED);
        assert_eq!(res.state, state);
        assert_eq!(res.state.replay_cursor, ProjectionReplayCursor::At(17));
    }

    #[test]
    fn test_cursor_8_resource_rejection_preserves_cursor() {
        let ctx = make_ctx(10, 100);
        let state = make_state_with_cursor(ShellLifecycle::Active, ProjectionReplayCursor::At(23));
        let res =
            evaluate_lifecycle_transition(&ctx, &state, cmd(ShellLifecycleCommand::Suspend, 101));
        assert_eq!(res.disposition, ShellTransitionDisposition::Rejected);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(res.diagnostics[0].evaluation_stage, 4);
        assert_eq!(res.state, state);
        assert_eq!(res.state.replay_cursor, ProjectionReplayCursor::At(23));
    }

    #[test]
    fn test_cursor_9_accepted_preflight_preserves_uninitialized() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        let original_state = *session.state();

        let env = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = session.preflight_projection_patch_envelope(env);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );
        assert_eq!(*session.state(), original_state);
        assert_eq!(
            session.state().replay_cursor,
            ProjectionReplayCursor::Uninitialized
        );
    }

    #[test]
    fn test_cursor_10_rejected_preflight_preserves_cursor() {
        let ctx = make_ctx(10, 100);
        let state = make_state_with_cursor(ShellLifecycle::Active, ProjectionReplayCursor::At(31));

        // Resource overflow
        let env1 = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 100, // exceeds max_patches
            stimulus_bytes: 50,
        };
        let res1 = evaluate_projection_patch_envelope_preflight(&ctx, &state, env1);
        assert_eq!(
            res1.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(state.replay_cursor, ProjectionReplayCursor::At(31));

        // Session mismatch
        let env2 = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(999),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res2 = evaluate_projection_patch_envelope_preflight(&ctx, &state, env2);
        assert_eq!(
            res2.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(state.replay_cursor, ProjectionReplayCursor::At(31));
    }

    #[test]
    fn test_cursor_11_cursor_values_are_opaque() {
        let v1 = ProjectionReplayCursor::At(0);
        let v2 = ProjectionReplayCursor::At(1);
        let v3 = ProjectionReplayCursor::At(u64::MAX);

        let state1 = make_state_with_cursor(ShellLifecycle::Active, v1);
        let state2 = make_state_with_cursor(ShellLifecycle::Active, v2);
        let state3 = make_state_with_cursor(ShellLifecycle::Active, v3);

        assert_eq!(state1.replay_cursor, ProjectionReplayCursor::At(0));
        assert_eq!(state2.replay_cursor, ProjectionReplayCursor::At(1));
        assert_eq!(state3.replay_cursor, ProjectionReplayCursor::At(u64::MAX));

        assert_ne!(state1.replay_cursor, state2.replay_cursor);
        assert_ne!(state2.replay_cursor, state3.replay_cursor);
    }

    #[test]
    fn test_cursor_12_independent_session_determinism() {
        let ctx = make_ctx(10, 100);
        let mut session_a = ShellSession::new(ctx.clone());
        let mut session_b = ShellSession::new(ctx);
        let res_a = session_a.apply(cmd(ShellLifecycleCommand::Activate, 10));
        let res_b = session_b.apply(cmd(ShellLifecycleCommand::Activate, 10));

        assert_eq!(res_a, res_b);
        assert_eq!(session_a.state(), session_b.state());
        assert_eq!(
            session_a.state().replay_cursor,
            session_b.state().replay_cursor
        );

        let env = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let pref_a = session_a.preflight_projection_patch_envelope(env);
        let pref_b = session_b.preflight_projection_patch_envelope(env);

        assert_eq!(pref_a, pref_b);
        assert_eq!(session_a.state(), session_b.state());
        assert_eq!(
            session_a.state().replay_cursor,
            session_b.state().replay_cursor
        );
    }

    #[test]
    fn test_cursor_13_context_remains_cursor_free() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx.clone());
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        assert_eq!(session.context, ctx); // Context structural equality is preserved
    }

    fn make_ctx(cap: usize, max_bytes: usize) -> ActivatedShellSessionContext {
        ActivatedShellSessionContext {
            session_id: ShellSessionId(1),
            limits: ShellLifecycleLimits {
                max_transition_stimulus_bytes: max_bytes,
                max_diagnostics_per_transition: cap,
                max_patches_per_transition: 64,
                max_target_references_per_transition: 64,
            },
            catalog: ActiveProjectionTargetCatalog::empty(),
        }
    }

    fn make_state(lifecycle: ShellLifecycle) -> ShellLocalState {
        ShellLocalState {
            session_id: ShellSessionId(1),
            lifecycle,
            replay_cursor: ProjectionReplayCursor::Uninitialized,
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
        let session = ShellSession::new(ctx.clone());
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
        let mut session = ShellSession::new(original_ctx.clone());

        session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        session.apply(cmd(ShellLifecycleCommand::Suspend, 10));
        session.apply(cmd(ShellLifecycleCommand::Resume, 10));

        assert_eq!(session.context, original_ctx);
    }

    #[test]
    fn test_owner_deterministic_sequence() {
        let ctx = make_ctx(10, 100);
        let mut session1 = ShellSession::new(ctx.clone());
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

    #[test]
    fn test_preflight_1_accepted_within_limits() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );
        assert_eq!(res.diagnostics.len(), 0);
        assert_eq!(res.sequence_no, 42);
        assert_eq!(res.patch_count, 5);
        assert_eq!(res.stimulus_bytes, 50);
    }

    #[test]
    fn test_preflight_2_exact_limits_accepted() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 64,
            stimulus_bytes: 100,
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );
    }

    #[test]
    fn test_preflight_3_patch_count_exceeded() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 65,
            stimulus_bytes: 50,
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.diagnostics[0].stable_code, SPV0_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(res.diagnostics[0].evaluation_stage, 4);
        assert_eq!(res.sequence_no, 42);
        assert_eq!(res.patch_count, 65);
        assert_eq!(res.stimulus_bytes, 50);
    }

    #[test]
    fn test_preflight_4_stimulus_bytes_exceeded() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 101,
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.diagnostics[0].stable_code, SPV0_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(res.diagnostics[0].evaluation_stage, 4);
        assert_eq!(res.sequence_no, 42);
        assert_eq!(res.patch_count, 5);
        assert_eq!(res.stimulus_bytes, 101);
    }

    #[test]
    fn test_preflight_5_both_limits_exceeded() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 65,
            stimulus_bytes: 101,
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.emitted_diagnostic_count, 1);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(res.sequence_no, 42);
        assert_eq!(res.patch_count, 65);
        assert_eq!(res.stimulus_bytes, 101);
    }

    #[test]
    fn test_preflight_6_envelope_session_mismatch() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(999),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_MISMATCH);
        assert_eq!(res.diagnostics[0].evaluation_stage, 1);
    }

    #[test]
    fn test_preflight_7_context_state_mismatch() {
        let ctx = make_ctx(10, 100);
        let state = ShellLocalState {
            session_id: ShellSessionId(999),
            lifecycle: ShellLifecycle::Active,
            replay_cursor: ProjectionReplayCursor::Uninitialized,
        };
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };

        let res = evaluate_projection_patch_envelope_preflight(&ctx, &state, envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_MISMATCH);
        assert_eq!(res.diagnostics[0].evaluation_stage, 1);
    }

    #[test]
    fn test_preflight_8_session_mismatch_precedence() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Close, 10)); // closed lifecycle

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(999), // session mismatch
            sequence_no: 42,
            patch_count: 65,     // resource overflow
            stimulus_bytes: 101, // resource overflow
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_MISMATCH);
        assert_eq!(res.diagnostics[0].evaluation_stage, 1);
    }

    #[test]
    fn test_preflight_9_closed_precedence_over_resources() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Close, 10)); // closed lifecycle

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 65,     // resource overflow
            stimulus_bytes: 101, // resource overflow
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_SESSION_CLOSED);
        assert_eq!(res.diagnostics[0].evaluation_stage, 2);
    }

    #[test]
    fn test_preflight_10_non_closed_lifecycle_eligibility() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx); // Created

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };

        let res1 = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res1.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );

        session.apply(cmd(ShellLifecycleCommand::Activate, 10)); // Active
        let res2 = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res2.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );

        session.apply(cmd(ShellLifecycleCommand::Suspend, 10)); // Suspended
        let res3 = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res3.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );
    }

    #[test]
    fn test_preflight_11_diagnostic_cap_zero() {
        let ctx = make_ctx(0, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 65,
            stimulus_bytes: 101,
        };

        let res = session.preflight_projection_patch_envelope(envelope);
        assert_eq!(
            res.disposition,
            ProjectionPatchPreflightDisposition::Rejected
        );
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.emitted_diagnostic_count, 0);
        assert_eq!(res.diagnostics.len(), 0);
    }

    #[test]
    fn test_preflight_12_opaque_sequence_metadata() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelopes = vec![
            OrderedProjectionPatchBatchEnvelope {
                session_id: ShellSessionId(1),
                sequence_no: 0,
                patch_count: 5,
                stimulus_bytes: 50,
            },
            OrderedProjectionPatchBatchEnvelope {
                session_id: ShellSessionId(1),
                sequence_no: u64::MAX,
                patch_count: 5,
                stimulus_bytes: 50,
            },
        ];

        for envelope in envelopes {
            let res = session.preflight_projection_patch_envelope(envelope);
            assert_eq!(
                res.disposition,
                ProjectionPatchPreflightDisposition::Accepted
            );
            assert_eq!(res.sequence_no, envelope.sequence_no);
        }
    }

    #[test]
    fn test_preflight_13_empty_batch_metadata() {
        // Case A
        let ctx_a = make_ctx(10, 100);
        let mut session_a = ShellSession::new(ctx_a);
        session_a.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope_a = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 0,
            stimulus_bytes: 0,
        };

        let res_a = session_a.preflight_projection_patch_envelope(envelope_a);
        assert_eq!(
            res_a.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );
        assert_eq!(res_a.patch_count, 0);
        assert!(res_a.diagnostics.is_empty());
        assert_eq!(res_a.logical_diagnostic_count, 0);
        assert_eq!(res_a.emitted_diagnostic_count, 0);

        // Case B
        let mut zero_limit_ctx = make_ctx(10, 100);
        zero_limit_ctx.limits.max_patches_per_transition = 0;
        let mut session_b = ShellSession::new(zero_limit_ctx);
        session_b.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope_b = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 0,
            stimulus_bytes: 0,
        };

        let res_b = session_b.preflight_projection_patch_envelope(envelope_b);
        assert_eq!(
            res_b.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );
        assert_eq!(res_b.patch_count, 0);
        assert!(res_b.diagnostics.is_empty());
        assert_eq!(res_b.logical_diagnostic_count, 0);
        assert_eq!(res_b.emitted_diagnostic_count, 0);
    }

    #[test]
    fn test_preflight_14_session_remains_unchanged() {
        let ctx = make_ctx(10, 100);
        let mut session = ShellSession::new(ctx);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let original_state = *session.state();
        let original_context = session.context.clone();

        // Accepted preflight
        let env_accepted = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        session.preflight_projection_patch_envelope(env_accepted);
        assert_eq!(*session.state(), original_state);
        assert_eq!(session.context, original_context);

        // Rejected preflight
        let env_rejected = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(999),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        session.preflight_projection_patch_envelope(env_rejected);
        assert_eq!(*session.state(), original_state);
        assert_eq!(session.context, original_context);
    }

    #[test]
    fn test_preflight_15_determinism() {
        let ctx = make_ctx(10, 100);
        let mut session_a = ShellSession::new(ctx.clone());
        let mut session_b = ShellSession::new(ctx);
        session_a.apply(cmd(ShellLifecycleCommand::Activate, 10));
        session_b.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };

        let result_a = session_a.preflight_projection_patch_envelope(envelope);
        let result_b = session_b.preflight_projection_patch_envelope(envelope);

        assert_eq!(result_a, result_b);
        assert_eq!(session_a.state(), session_b.state());
        assert_eq!(session_a.context, session_b.context);
    }

    #[test]
    fn test_replay_compatibility_1_empty_batch_with_uninitialized() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 0,
            patch_count: 0,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::Uninitialized,
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::NotApplicable
        );
        assert_eq!(res.diagnostics.len(), 0);
        assert_eq!(res.logical_diagnostic_count, 0);
        assert_eq!(res.emitted_diagnostic_count, 0);
    }

    #[test]
    fn test_replay_compatibility_2_empty_batch_ignores_positioned_cursor() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 123,
            patch_count: 0,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(u64::MAX),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::NotApplicable
        );
        assert!(!(res.disposition == ProjectionReplayCompatibilityDisposition::Mismatch));
    }

    #[test]
    fn test_replay_compatibility_3_uninitialized_accepts_sequence_zero() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 0,
            patch_count: 1,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::Uninitialized,
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Compatible
        );
    }

    #[test]
    fn test_replay_compatibility_4_uninitialized_accepts_maximum_sequence() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: u64::MAX,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::Uninitialized,
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Compatible
        );
    }

    #[test]
    fn test_replay_compatibility_5_at_0_accepts_sequence_1() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 1,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(0),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Compatible
        );
    }

    #[test]
    fn test_replay_compatibility_6_positioned_cursor_accepts_exact_successor() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(41),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Compatible
        );
    }

    #[test]
    fn test_replay_compatibility_7_duplicate_is_mismatch() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 41,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(41),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
    }

    #[test]
    fn test_replay_compatibility_8_lower_value_is_mismatch() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 40,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(41),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
    }

    #[test]
    fn test_replay_compatibility_9_gap_is_mismatch() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 43,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(41),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
    }

    #[test]
    fn test_replay_compatibility_10_large_gap_is_mismatch() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: u64::MAX,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(1),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
    }

    #[test]
    fn test_replay_compatibility_11_maximum_cursor_rejects_zero() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 0,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(u64::MAX),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
    }

    #[test]
    fn test_replay_compatibility_12_maximum_cursor_rejects_maximum() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: u64::MAX,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(u64::MAX),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
    }

    #[test]
    fn test_replay_compatibility_13_mismatch_diagnostic_is_exact() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 41,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(41),
            envelope,
            10,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.emitted_diagnostic_count, 1);
        assert_eq!(res.diagnostics.len(), 1);
        assert_eq!(res.diagnostics[0].stable_code, SPV0_REPLAY_CURSOR_MISMATCH);
        assert_eq!(res.diagnostics[0].evaluation_stage, 6);
    }

    #[test]
    fn test_replay_compatibility_14_zero_diagnostic_cap_preserves_mismatch_disposition() {
        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 41,
            patch_count: 5,
            stimulus_bytes: 50,
        };
        let res = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(41),
            envelope,
            0,
        );
        assert_eq!(
            res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
        assert_eq!(res.logical_diagnostic_count, 1);
        assert_eq!(res.emitted_diagnostic_count, 0);
        assert_eq!(res.diagnostics.len(), 0);
    }

    #[test]
    fn test_replay_compatibility_15_metadata_is_preserved_exactly() {
        // NotApplicable
        let env_na = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 12345,
            patch_count: 0,
            stimulus_bytes: 50,
        };
        let res_na = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::Uninitialized,
            env_na,
            10,
        );
        assert_eq!(
            res_na.disposition,
            ProjectionReplayCompatibilityDisposition::NotApplicable
        );
        assert_eq!(res_na.sequence_no, env_na.sequence_no);
        assert_eq!(res_na.patch_count, env_na.patch_count);

        // Compatible
        let env_c = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 999,
            patch_count: 8,
            stimulus_bytes: 50,
        };
        let res_c = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(998),
            env_c,
            10,
        );
        assert_eq!(
            res_c.disposition,
            ProjectionReplayCompatibilityDisposition::Compatible
        );
        assert_eq!(res_c.sequence_no, env_c.sequence_no);
        assert_eq!(res_c.patch_count, env_c.patch_count);

        // Mismatch
        let env_m = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 42,
            patch_count: 7,
            stimulus_bytes: 50,
        };
        let res_m = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(50),
            env_m,
            10,
        );
        assert_eq!(
            res_m.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
        assert_eq!(res_m.sequence_no, env_m.sequence_no);
        assert_eq!(res_m.patch_count, env_m.patch_count);
    }

    #[test]
    fn test_replay_compatibility_16_determinism() {
        let env_na = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 10,
            patch_count: 0,
            stimulus_bytes: 50,
        };
        let res_na1 = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(5),
            env_na,
            10,
        );
        let res_na2 = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(5),
            env_na,
            10,
        );
        assert_eq!(res_na1, res_na2);

        let env_c = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 10,
            patch_count: 1,
            stimulus_bytes: 50,
        };
        let res_c1 = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(9),
            env_c,
            10,
        );
        let res_c2 = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(9),
            env_c,
            10,
        );
        assert_eq!(res_c1, res_c2);

        let env_m = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 10,
            patch_count: 1,
            stimulus_bytes: 50,
        };
        let res_m1 = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(10),
            env_m,
            10,
        );
        let res_m2 = evaluate_projection_patch_replay_compatibility(
            ProjectionReplayCursor::At(10),
            env_m,
            10,
        );
        assert_eq!(res_m1, res_m2);
    }

    #[test]
    fn test_replay_compatibility_17_stage_4_preflight_remains_stage_4_only() {
        let ctx = make_ctx(10, 100);
        let mut state = make_state(ShellLifecycle::Active);
        state.replay_cursor = ProjectionReplayCursor::At(10);

        let envelope = OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no: 10, // Duplicate, invalid for stage 6
            patch_count: 5,
            stimulus_bytes: 50,
        };

        let preflight_res = evaluate_projection_patch_envelope_preflight(&ctx, &state, envelope);
        assert_eq!(
            preflight_res.disposition,
            ProjectionPatchPreflightDisposition::Accepted
        );

        let compat_res =
            evaluate_projection_patch_replay_compatibility(state.replay_cursor, envelope, 10);
        assert_eq!(
            compat_res.disposition,
            ProjectionReplayCompatibilityDisposition::Mismatch
        );
    }

    // ---- stages 1-9 orchestration (`apply_projection_patch_batch`) ------

    fn orchestration_limits(max_target_references_per_transition: usize) -> ShellSessionLimits {
        ShellSessionLimits {
            max_transition_stimulus_bytes: 1000,
            max_diagnostics_per_transition: 10,
            max_patches_per_transition: 64,
            max_target_references_per_transition,
        }
    }

    fn make_active_session() -> ShellSession {
        let snapshot = prom_ui::shell_bridge::demo_activation_snapshot();
        let mut session = create_shell_session(1, orchestration_limits(64), &snapshot);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));
        session
    }

    fn outer_envelope(sequence_no: u64, patch_count: usize) -> OrderedProjectionPatchBatchEnvelope {
        OrderedProjectionPatchBatchEnvelope {
            session_id: ShellSessionId(1),
            sequence_no,
            patch_count,
            stimulus_bytes: 50,
        }
    }

    /// A batch touching two targets declared in `demo_activation_snapshot`:
    /// the label binding (node 2, slot 0) and the status node (node 3).
    fn valid_batch_touching_declared_targets(
        seq: u64,
    ) -> (
        prom_ui::shell_bridge::AdmittedProjectionPatchBatch,
        prom_ui::shell_bridge::PreparedManifestSnapshot,
    ) {
        let ops = alloc::vec![
            prom_ui::shell_bridge::BridgePatchOperation::SetBindingValue {
                node: 2,
                slot: 0,
                value: prom_ui::shell_bridge::BridgeValue::Text("hi".into()),
            },
            prom_ui::shell_bridge::BridgePatchOperation::SetNodeAvailability {
                node: 3,
                availability: prom_ui::shell_bridge::BridgeAvailability::Hidden,
            },
        ];
        let envelope = prom_ui::shell_bridge::BridgePatchEnvelope {
            patch_id: seq,
            stream_id: 1,
            document_id: 1,
            previous_revision: seq - 1,
            revision: seq,
            epoch: 1,
            sequence: seq,
        };
        let batch = prom_ui::shell_bridge::admit_projection_patch_batch(envelope, ops)
            .expect("valid batch");
        let manifest = prom_ui::shell_bridge::prepared_manifest_snapshot(&batch).expect("derives");
        (batch, manifest)
    }

    #[test]
    fn test_orchestration_1_valid_batch_applies_and_commits() {
        let mut session = make_active_session();
        let (batch, manifest) = valid_batch_touching_declared_targets(1);

        let result = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest.target_reference_count,
            &manifest,
            &batch,
        );

        assert_eq!(
            result.disposition,
            ProjectionPatchApplicationDisposition::Applied
        );
        assert_eq!(result.replay_cursor, ProjectionReplayCursor::At(1));
        assert_eq!(session.replay_cursor(), ProjectionReplayCursor::At(1));
        assert_eq!(
            session.binding_value(2, 0),
            Some(prom_ui::shell_bridge::BridgeValue::Text("hi".into()))
        );
        assert_eq!(
            session.node_availability(3),
            Some(prom_ui::shell_bridge::BridgeAvailability::Hidden)
        );
    }

    #[test]
    fn test_orchestration_2_undeclared_target_rejected() {
        let mut session = make_active_session();
        let ops = alloc::vec![
            prom_ui::shell_bridge::BridgePatchOperation::SetNodeAvailability {
                node: 999,
                availability: prom_ui::shell_bridge::BridgeAvailability::Hidden,
            }
        ];
        let envelope = prom_ui::shell_bridge::BridgePatchEnvelope {
            patch_id: 1,
            stream_id: 1,
            document_id: 1,
            previous_revision: 0,
            revision: 1,
            epoch: 1,
            sequence: 1,
        };
        let batch =
            prom_ui::shell_bridge::admit_projection_patch_batch(envelope, ops).expect("valid");
        let manifest = prom_ui::shell_bridge::prepared_manifest_snapshot(&batch).expect("derives");

        let result = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest.target_reference_count,
            &manifest,
            &batch,
        );

        assert_eq!(
            result.disposition,
            ProjectionPatchApplicationDisposition::Rejected
        );
        assert_eq!(result.diagnostics[0].stable_code, SPV0_INVALID_TARGET);
        assert_eq!(result.diagnostics[0].evaluation_stage, 5);
        assert_eq!(result.replay_cursor, ProjectionReplayCursor::Uninitialized);
        assert_eq!(
            session.replay_cursor(),
            ProjectionReplayCursor::Uninitialized
        );
        assert_eq!(session.node_availability(999), None);
    }

    #[test]
    fn test_orchestration_3_declared_actual_count_mismatch_rejected() {
        let mut session = make_active_session();
        let (batch, manifest) = valid_batch_touching_declared_targets(1);
        let wrong_declared_count = manifest.target_reference_count + 1;

        let result = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            wrong_declared_count,
            &manifest,
            &batch,
        );

        assert_eq!(
            result.disposition,
            ProjectionPatchApplicationDisposition::Rejected
        );
        assert_eq!(result.diagnostics[0].stable_code, SPV0_INVALID_STIMULUS);
        assert_eq!(result.diagnostics[0].evaluation_stage, 4);
        assert_eq!(
            session.replay_cursor(),
            ProjectionReplayCursor::Uninitialized
        );
    }

    #[test]
    fn test_orchestration_4_target_reference_resource_limit_rejected() {
        let snapshot = prom_ui::shell_bridge::demo_activation_snapshot();
        let mut session = create_shell_session(1, orchestration_limits(1), &snapshot);
        session.apply(cmd(ShellLifecycleCommand::Activate, 10));

        let (batch, manifest) = valid_batch_touching_declared_targets(1); // 2 target refs > limit of 1
        let result = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest.target_reference_count,
            &manifest,
            &batch,
        );

        assert_eq!(
            result.disposition,
            ProjectionPatchApplicationDisposition::Rejected
        );
        assert_eq!(
            result.diagnostics[0].stable_code,
            SPV0_RESOURCE_LIMIT_EXCEEDED
        );
        assert_eq!(result.diagnostics[0].evaluation_stage, 4);
    }

    #[test]
    fn test_orchestration_5_replay_cursor_mismatch_rejected_preserves_committed_state() {
        let mut session = make_active_session();
        let (batch1, manifest1) = valid_batch_touching_declared_targets(1);
        let result1 = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest1.target_reference_count,
            &manifest1,
            &batch1,
        );
        assert_eq!(
            result1.disposition,
            ProjectionPatchApplicationDisposition::Applied
        );
        assert_eq!(session.replay_cursor(), ProjectionReplayCursor::At(1));

        // Duplicate sequence_no: not a valid successor of At(1).
        let (batch2, manifest2) = valid_batch_touching_declared_targets(1);
        let result2 = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest2.target_reference_count,
            &manifest2,
            &batch2,
        );

        assert_eq!(
            result2.disposition,
            ProjectionPatchApplicationDisposition::Rejected
        );
        assert_eq!(
            result2.diagnostics[0].stable_code,
            SPV0_REPLAY_CURSOR_MISMATCH
        );
        assert_eq!(result2.diagnostics[0].evaluation_stage, 6);
        assert_eq!(session.replay_cursor(), ProjectionReplayCursor::At(1));
        assert_eq!(
            session.binding_value(2, 0),
            Some(prom_ui::shell_bridge::BridgeValue::Text("hi".into()))
        );
    }

    #[test]
    fn test_orchestration_6_structural_application_failure_preserves_committed_state() {
        let mut session = make_active_session();
        let insert_ops = alloc::vec![
            prom_ui::shell_bridge::BridgePatchOperation::CollectionInsert {
                collection: 4,
                key: 10,
                before: None,
                value: prom_ui::shell_bridge::BridgeValue::Unsigned(1),
            }
        ];
        let env1 = prom_ui::shell_bridge::BridgePatchEnvelope {
            patch_id: 1,
            stream_id: 1,
            document_id: 1,
            previous_revision: 0,
            revision: 1,
            epoch: 1,
            sequence: 1,
        };
        let batch1 =
            prom_ui::shell_bridge::admit_projection_patch_batch(env1, insert_ops).expect("valid");
        let manifest1 =
            prom_ui::shell_bridge::prepared_manifest_snapshot(&batch1).expect("derives");
        let result1 = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest1.target_reference_count,
            &manifest1,
            &batch1,
        );
        assert_eq!(
            result1.disposition,
            ProjectionPatchApplicationDisposition::Applied
        );

        // Same key, second (independently valid-looking) transition: fails
        // at candidate-state calculation (stage 7/8), not stage 5.
        let dup_ops = alloc::vec![
            prom_ui::shell_bridge::BridgePatchOperation::CollectionInsert {
                collection: 4,
                key: 10,
                before: None,
                value: prom_ui::shell_bridge::BridgeValue::Unsigned(2),
            }
        ];
        let env2 = prom_ui::shell_bridge::BridgePatchEnvelope {
            patch_id: 2,
            stream_id: 1,
            document_id: 1,
            previous_revision: 1,
            revision: 2,
            epoch: 1,
            sequence: 2,
        };
        let batch2 =
            prom_ui::shell_bridge::admit_projection_patch_batch(env2, dup_ops).expect("valid");
        let manifest2 =
            prom_ui::shell_bridge::prepared_manifest_snapshot(&batch2).expect("derives");
        let result2 = session.apply_projection_patch_batch(
            outer_envelope(2, 1),
            manifest2.target_reference_count,
            &manifest2,
            &batch2,
        );

        assert_eq!(
            result2.disposition,
            ProjectionPatchApplicationDisposition::Rejected
        );
        assert_eq!(
            result2.diagnostics[0].stable_code,
            SPV0_STATE_INVARIANT_VIOLATION
        );
        assert_eq!(result2.diagnostics[0].evaluation_stage, 8);
        assert_eq!(session.replay_cursor(), ProjectionReplayCursor::At(1));
        assert_eq!(
            session.collection_entries(4),
            alloc::vec![(10, prom_ui::shell_bridge::BridgeValue::Unsigned(1))]
        );
    }

    #[test]
    fn test_orchestration_7_committed_state_persists_across_subsequent_queries() {
        let mut session = make_active_session();
        let (batch, manifest) = valid_batch_touching_declared_targets(1);
        session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest.target_reference_count,
            &manifest,
            &batch,
        );

        for _ in 0..3 {
            assert_eq!(
                session.binding_value(2, 0),
                Some(prom_ui::shell_bridge::BridgeValue::Text("hi".into()))
            );
            assert_eq!(
                session.node_availability(3),
                Some(prom_ui::shell_bridge::BridgeAvailability::Hidden)
            );
            assert_eq!(session.replay_cursor(), ProjectionReplayCursor::At(1));
        }
    }

    #[test]
    fn test_orchestration_8_rejection_does_not_block_subsequent_valid_batch() {
        let mut session = make_active_session();
        let bad_ops = alloc::vec![
            prom_ui::shell_bridge::BridgePatchOperation::SetNodeAvailability {
                node: 999,
                availability: prom_ui::shell_bridge::BridgeAvailability::Hidden,
            }
        ];
        let bad_envelope = prom_ui::shell_bridge::BridgePatchEnvelope {
            patch_id: 1,
            stream_id: 1,
            document_id: 1,
            previous_revision: 0,
            revision: 1,
            epoch: 1,
            sequence: 1,
        };
        let bad_batch = prom_ui::shell_bridge::admit_projection_patch_batch(bad_envelope, bad_ops)
            .expect("valid");
        let bad_manifest =
            prom_ui::shell_bridge::prepared_manifest_snapshot(&bad_batch).expect("derives");
        let bad_result = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            bad_manifest.target_reference_count,
            &bad_manifest,
            &bad_batch,
        );
        assert_eq!(
            bad_result.disposition,
            ProjectionPatchApplicationDisposition::Rejected
        );
        assert_eq!(
            session.replay_cursor(),
            ProjectionReplayCursor::Uninitialized
        );

        let (good_batch, good_manifest) = valid_batch_touching_declared_targets(1);
        let good_result = session.apply_projection_patch_batch(
            outer_envelope(1, 1),
            good_manifest.target_reference_count,
            &good_manifest,
            &good_batch,
        );
        assert_eq!(
            good_result.disposition,
            ProjectionPatchApplicationDisposition::Applied
        );
        assert_eq!(session.replay_cursor(), ProjectionReplayCursor::At(1));
    }

    #[test]
    fn test_orchestration_9_determinism() {
        let (batch, manifest) = valid_batch_touching_declared_targets(1);
        let mut session_a = make_active_session();
        let mut session_b = make_active_session();

        let result_a = session_a.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest.target_reference_count,
            &manifest,
            &batch,
        );
        let result_b = session_b.apply_projection_patch_batch(
            outer_envelope(1, 1),
            manifest.target_reference_count,
            &manifest,
            &batch,
        );

        assert_eq!(result_a, result_b);
        assert_eq!(session_a.binding_value(2, 0), session_b.binding_value(2, 0));
        assert_eq!(session_a.replay_cursor(), session_b.replay_cursor());
    }
}
