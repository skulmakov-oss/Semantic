# Semantic UI Recovery and Rollback Visual Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define recovery, rollback, retry, cancel, and safe continuation visual boundaries before implementation

## 1. Goal

This document defines the visual and semantic boundary for recovery, rollback, retry, cancel, and safe continuation states in Semantic UI.

The project must distinguish:

```text
recovery != rollback
rollback != undo
cancel != failure
retry != blind re-execute
safe recovery requires trace
```

Recovery must be explicit, admitted, inspectable, and trace-aware.

The UI must not offer recovery actions that hide state uncertainty.

## 2. Relationship to error/denial/quarantine boundary

Recovery and rollback visualization depends on the error, denial, and quarantine visual boundary:

```text
docs/architecture/ui_error_denial_quarantine_visual_boundary.md
```

Recovery options may appear after:

- denial;
- failure;
- conflict;
- quarantine;
- invalid state;
- cancelled operation;
- incomplete effect;
- unsafe continuation.

Recovery visualization must preserve the original status meaning.

Recovery must not erase the denial, failure, conflict, or quarantine that caused it.

## 3. Layer separation

| State / operation | Meaning | Required visual behavior |
| --- | --- | --- |
| recovery | safe path toward a valid state | show path, reason, and admission |
| rollback | controlled return to a prior known state | show target state and trace |
| undo | user-level reversal of an operation | not equivalent to rollback |
| cancel | stop before completion/commit | show what was cancelled |
| retry | repeat an operation under explicit conditions | show what changes before retry |
| continue | proceed despite known condition | show risk/admission |
| inspect | examine before action | show preserved state and context |

This preserves:

```text
Recovery is not rollback.
Rollback is not undo.
Cancel is not failure.
Retry is not blind re-execute.
Continue is not silent ignore.
```

## 4. Recovery definition

Recovery is a safe path from an invalid, denied, failed, conflicted, or quarantined state toward a valid or inspectable state.

Recovery may include:

```text
inspect
retry
cancel
rollback
reconfigure
acknowledge
isolate
restore
continue with admission
```

Recovery must show:

1. source condition;
2. recovery target;
3. required admission;
4. preserved state;
5. trace/audit relation;
6. risk or limitation if any.

Recovery is not cosmetic reassurance.

## 5. Rollback definition

Rollback is a controlled transition back to a prior known state or safe checkpoint.

Rollback must define:

- source state;
- target state;
- checkpoint or trace reference;
- scope;
- rollback capability/admission;
- effect relation;
- failure behavior.

Rollback must not be treated as generic undo.

Rollback may affect system state and therefore requires admission and trace.

## 6. Undo boundary

Undo is a user-facing reversal concept.

Rollback is a controlled state transition.

They may overlap, but they are not equivalent.

Example:

```text
user hides panel
  -> undo may restore panel visibility

effect committed
  -> rollback requires effect boundary, capability admission, and trace
```

H11 does not admit undo semantics.

Future undo behavior requires a separate boundary or explicit mapping to rollback/recovery.

## 7. Cancel boundary

Cancel means stopping an operation before completion or before commit.

Cancel is not failure.

Cancel must show:

- what operation was cancelled;
- whether anything was prepared;
- whether anything was committed;
- whether state was preserved;
- whether cancellation produced trace/audit;
- whether retry is possible.

Cancel must not be displayed as generic failure.

## 8. Retry boundary

Retry means attempting an operation again under explicit conditions.

Retry must not be blind re-execution.

A retry must answer:

1. What failed or was denied?
2. What changed since the first attempt?
3. Is the same target still valid?
4. Are capabilities still available?
5. Is lifecycle state still valid?
6. Is retry admitted?
7. What trace links the attempts?

Retry must not bypass admission.

## 9. Continue boundary

Continue means proceeding despite a known condition.

Examples:

```text
continue after non-fatal renderer failure
continue after optional trace projection failure
continue after local UI-only denial
continue with quarantined item excluded
```

Continue requires explicit admission if it affects state, capability, effect, or trace.

Continue must not mean ignoring the problem silently.

## 10. Inspect-before-recover rule

When state is ambiguous, quarantined, conflicted, or partially failed, inspection may be required before recovery.

Example:

```text
conflict detected
  -> quarantine
  -> inspect
  -> admitted recovery action
```

The UI must not offer destructive recovery before inspection when trace/audit context is required.

## 11. Recovery option visibility

Recovery options should be visible only when semantically valid.

Examples:

```text
rollback available
retry available
cancel available
inspect required
continue allowed
recovery unavailable
```

Unavailable recovery must show why if user expectation is affected.

A hidden unavailable option must not look like no-op.

## 12. Trace and audit relationship

Recovery and rollback may require trace/audit records.

Trace/audit must explain:

```text
source condition
  -> recovery option
  -> admission
  -> recovery action
  -> resulting state
```

Rollback trace must show:

```text
current state
  -> checkpoint / prior state
  -> rollback admission
  -> rollback result
```

Visual recovery state is a projection.

Trace/audit remains the source of truth.

## 13. Effect relationship

Effect recovery must distinguish:

```text
effect request denied
effect prepare failed
effect prepared
effect commit denied
effect commit failed
effect committed
effect cancelled
effect rollback requested
effect rollback admitted
effect rollback failed
effect rollback completed
```

Prepared effect recovery is not the same as committed effect rollback.

Committed effect rollback may require separate runtime/effect capability.

## 14. Capability relationship

Recovery actions may require capabilities.

Examples:

```text
ui.capability.retry_effect
ui.capability.cancel_effect
ui.capability.rollback_effect
ui.capability.inspect_quarantine
ui.capability.recover_conflict
```

These are examples, not implementation commitments.

Visual recovery affordance is not capability grant.

Capability admission must remain explicit.

## 15. Quarantine relationship

Quarantine recovery must preserve isolation semantics.

Allowed recovery paths may include:

```text
inspect quarantined target
release quarantine
keep isolated
rollback related effect
discard sandbox state
export diagnostic trace
```

Quarantine recovery must not imply deletion unless deletion is explicitly admitted.

## 16. Conflict relationship

Conflict recovery must show conflicting parties and resolution path.

Potential recovery paths:

```text
choose source
merge with admission
rollback one side
keep both isolated
escalate to inspection
deny operation
```

Conflict recovery must not hide the fact that conflict existed.

## 17. Component relationship

Components may expose recovery/rollback surfaces.

Candidate components:

```text
RecoveryOptionPanel
RollbackTraceView
RetryActionView
CancelEffectView
QuarantineRecoveryPanel
ConflictRecoveryView
SafeContinueNotice
```

Components must not invent recovery meaning.

They display admitted recovery states and trace facts.

## 18. Layout relationship

Layout primitives may provide recovery regions.

Examples:

```text
OverlaySurface
TraceLane
InspectorPane
QuarantineRegion
ConflictBoundary
EffectLane
```

Layout primitives must not own recovery semantics.

They provide spatial structure for recovery inspection and action.

## 19. Renderer relationship

Renderer must not define recovery or rollback meaning.

Renderer may display admitted recovery projections.

Renderer must not decide:

- whether retry is safe;
- whether rollback is available;
- whether cancellation succeeded;
- whether recovery requires trace;
- whether quarantine can be released;
- whether effect rollback is admitted.

Renderer output is not recovery authority.

## 20. Native backend relationship

Native backend may expose native failure facts.

Native backend must not decide recovery meaning.

Example:

```text
native window creation failed
  -> native failure fact
  -> trace/error projection
  -> possible recovery option
```

Native backend may perform platform recovery only after an admitted boundary exists.

## 21. Workbench relationship

Workbench may expose recovery/rollback UI.

Workbench must not define core recovery semantics.

Workbench-specific recovery paths require:

- Workbench-local recovery namespace; or
- explicit admission into core UI recovery contract; or
- separate boundary document.

No Workbench convenience retry should become core recovery semantics.

## 22. Required visual distinction table

Future visual implementation must distinguish:

| Condition / action | Not equivalent to | Required visibility |
| --- | --- | --- |
| recovery | rollback | path and admission |
| rollback | undo | target state and trace |
| cancel | failure | cancelled operation and state |
| retry | blind re-execute | changed condition and admission |
| continue | silent ignore | known condition and risk |
| inspect | recover | preserved state and context |
| quarantine release | deletion | target state and reason |

## 23. Forbidden shortcuts

The system must not:

- treat retry as blind re-execute;
- treat rollback as generic undo;
- treat cancel as failure;
- treat continue as silent ignore;
- release quarantine without admitted recovery;
- hide failed recovery;
- remove trace/audit path from rollback;
- let renderer decide recovery availability;
- let native backend define recovery semantics;
- let Workbench define core rollback behavior;
- show recovery option without reason or admission context.

## 24. Required admission rule

A future recovery/rollback visual implementation PR must define:

1. recovery category;
2. source condition;
3. target state;
4. required admission;
5. required capability if any;
6. trace/audit relation;
7. allowed state transitions;
8. denial/failure behavior;
9. component/layout projection;
10. renderer projection boundary;
11. tests/snapshots where applicable.

No recovery/rollback visual state should be added only as cosmetic guidance.

## 25. Future implementation shape

H11 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_recovery_rollback.md
crates/prom-ui-recovery/
crates/prom-ui-status/
crates/prom-ui-components/
apps/workbench recovery views
renderer recovery projection map
```

Any implementation must preserve:

```text
source condition
  -> recovery option
  -> admission
  -> recovery action
  -> trace/audit relation
  -> resulting state
```

## 26. Current decision

Recovery and rollback visual handling is not implemented in H11.

H11 only defines the boundary.

Current admitted visual/interaction/action architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
  -> focus/selection semantic boundary
  -> semantic action boundary
  -> effect request / UI capability boundary
  -> trace/audit visual boundary
  -> error/denial/quarantine visual boundary
  -> recovery/rollback visual boundary
```

Not yet admitted:

```text
recovery structs
rollback structs
undo model
retry dispatcher
cancel dispatcher
safe continue model
recovery components
rollback components
Workbench recovery views
renderer-owned recovery semantics
native backend-owned rollback semantics
```

## Renderer transcript and presentation status dependency

Renderer recovery must distinguish render retry, presentation retry, surface recreation, and continue-without-presentation.

Renderer transcript and presentation status boundaries are defined separately in:

```text
docs/architecture/ui_renderer_transcript_presentation_boundary.md
```

Retry render must not blindly re-present an unsafe frame.
